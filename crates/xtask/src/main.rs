use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use image::{ImageBuffer, Rgba};
#[cfg(windows)]
use windows_capture::{
    capture::{Context as CaptureContext, GraphicsCaptureApiHandler},
    frame::{Frame, ImageFormat as CaptureImageFormat},
    graphics_capture_api::InternalCaptureControl,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window as CaptureWindow,
};

#[cfg(windows)]
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, ClientToScreen, CreateCompatibleBitmap,
        CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HGDIOBJ,
        ReleaseDC, SRCCOPY, SelectObject,
    },
    Storage::Xps::{PW_CLIENTONLY, PrintWindow},
    UI::WindowsAndMessaging::{
        BringWindowToTop, EnumWindows, GetClientRect, GetWindowThreadProcessId, HWND_BOTTOM,
        HWND_TOP, IsWindowVisible, PostMessageW, SW_RESTORE, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_SHOWWINDOW, SetForegroundWindow, SetWindowPos, ShowWindow, WM_LBUTTONDOWN,
        WM_LBUTTONUP, WM_MOUSEMOVE,
    },
};
#[cfg(windows)]
use windows::core::BOOL;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const REFERENCE_COMMIT: &str = "8fc317939f5c8bbef4afbe309ae897abbc221631";

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct CompareOptions {
    expected: PathBuf,
    actual: PathBuf,
    max_different_pixels_percent: f64,
    channel_tolerance: u8,
    ignore_rects: Vec<Rect>,
}

#[derive(Debug, Default)]
struct CheckParityOptions {
    refresh_reference: bool,
    allow_window_capture: bool,
}

#[derive(Debug)]
struct CaptureR3CodeOptions {
    exe: PathBuf,
    output: PathBuf,
    screen: Option<String>,
    theme: Option<String>,
    delay: Duration,
    direct: bool,
    offscreen: bool,
    allow_window_capture: bool,
}

#[derive(Debug)]
struct CaptureReferenceOptions {
    repo: PathBuf,
    home: PathBuf,
    output_dir: PathBuf,
    startup_timeout: Duration,
}

#[derive(Debug)]
struct GenerateParityInventoryOptions {
    repo: PathBuf,
    output: PathBuf,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };
    let args: Vec<String> = args.collect();

    match command.as_str() {
        "check-parity" => check_parity(parse_check_parity_options(&args)?),
        "compare-screenshots" => compare_screenshots(parse_compare_options(&args)?),
        "capture-r3code-window" => capture_r3code_window(parse_capture_r3code_options(&args)?),
        "capture-reference-browser" => {
            capture_reference_browser(parse_capture_reference_options(&args)?)
        }
        "generate-parity-inventory" => {
            generate_parity_inventory(parse_generate_parity_inventory_options(&args)?)
        }
        _ => {
            print_usage();
            Err(format!("unknown xtask command: {command}").into())
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  cargo run -p xtask -- check-parity --allow-window-capture [--refresh-reference]
  cargo run -p xtask -- compare-screenshots --expected <png> --actual <png> [--channel-tolerance <n>] [--ignore-rect x,y,w,h] [--max-different-pixels-percent <n>]
  cargo run -p xtask -- capture-r3code-window --allow-window-capture [--screen draft|composer-focused|composer-menu|composer-inline-tokens|active-chat|project-scripts-menu|running-turn|pending-approval|pending-user-input|terminal-drawer|diff-panel|branch-toolbar|sidebar-options-menu|open-in-menu|git-actions-menu|provider-model-picker|settings|settings-diagnostics|command-palette|settings-theme-menu|settings-dark|settings-back|settings-keybindings|settings-keybindings-add|settings-providers|settings-source-control|settings-connections|settings-archive] [--theme light|dark|system] [--output <png>]
  cargo run -p xtask -- capture-reference-browser
  cargo run -p xtask -- generate-parity-inventory [--repo .omx/upstream-t3code] [--output docs/reference/PARITY_FILE_INVENTORY.md]"
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask must live under crates/xtask")
        .to_path_buf()
}

fn resolve_repo_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root().join(path)
    }
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed with {status}: {command:?}").into())
    }
}

fn parse_check_parity_options(args: &[String]) -> Result<CheckParityOptions> {
    let mut options = CheckParityOptions::default();
    for arg in args {
        match arg.as_str() {
            "--refresh-reference" => options.refresh_reference = true,
            "--allow-window-capture" => options.allow_window_capture = true,
            other => return Err(format!("unknown check-parity option: {other}").into()),
        }
    }
    Ok(options)
}

fn check_parity(options: CheckParityOptions) -> Result<()> {
    if !options.allow_window_capture {
        return Err(
            "check-parity launches native capture windows; rerun with --allow-window-capture"
                .into(),
        );
    }

    let root = repo_root();

    run(Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .current_dir(&root))?;
    run(Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(&root))?;
    run(Command::new("cargo")
        .args(["build", "-p", "r3_app"])
        .current_dir(&root))?;

    if options.refresh_reference {
        capture_reference_browser(CaptureReferenceOptions::default())?;
    }

    capture_r3code_window(CaptureR3CodeOptions {
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-empty-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-window.png"),
        max_different_pixels_percent: 2.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("command-palette".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-command-palette-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-command-palette-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-command-palette-window.png"),
        max_different_pixels_percent: 5.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("draft".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-draft-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-draft-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-draft-window.png"),
        max_different_pixels_percent: 1.75,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("composer-focused".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-composer-focused-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-composer-focused-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-composer-focused-window.png"),
        max_different_pixels_percent: 1.9,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("active-chat".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-active-chat-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-active-chat-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-active-chat-window.png"),
        max_different_pixels_percent: 4.1,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("project-scripts-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-project-scripts-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-project-scripts-menu-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-project-scripts-menu-window.png"),
        max_different_pixels_percent: 4.2,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("composer-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-composer-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-composer-menu-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-composer-menu-window.png"),
        max_different_pixels_percent: 4.5,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("composer-inline-tokens".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-composer-inline-tokens-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-composer-inline-tokens-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-composer-inline-tokens-window.png"),
        max_different_pixels_percent: 2.2,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("running-turn".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-running-turn-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-running-turn-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-running-turn-window.png"),
        max_different_pixels_percent: 4.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("pending-approval".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-pending-approval-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-pending-approval-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-pending-approval-window.png"),
        max_different_pixels_percent: 5.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("pending-user-input".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-pending-user-input-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-pending-user-input-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-pending-user-input-window.png"),
        max_different_pixels_percent: 5.1,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("terminal-drawer".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-terminal-drawer-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-terminal-drawer-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-terminal-drawer-window.png"),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("diff-panel".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-diff-panel-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-diff-panel-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-diff-panel-window.png"),
        max_different_pixels_percent: 8.8,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("branch-toolbar".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-branch-toolbar-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-branch-toolbar-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-branch-toolbar-window.png"),
        max_different_pixels_percent: 3.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("sidebar-options-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-sidebar-options-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-sidebar-options-menu-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-sidebar-options-menu-window.png"),
        max_different_pixels_percent: 3.7,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("open-in-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-open-in-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-open-in-menu-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-open-in-menu-window.png"),
        max_different_pixels_percent: 3.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("git-actions-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-git-actions-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-git-actions-menu-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-git-actions-menu-window.png"),
        max_different_pixels_percent: 3.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("provider-model-picker".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-provider-model-picker-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-provider-model-picker-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-provider-model-picker-window.png"),
        max_different_pixels_percent: 4.4,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-settings-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-window.png"),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-keybindings".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-keybindings-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-keybindings-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-keybindings-window.png"),
        max_different_pixels_percent: 6.35,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-keybindings-add".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path(
            "reference/screenshots/r3code-settings-keybindings-add-window.png",
        ),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-keybindings-add-reference.png",
        ),
        actual: resolve_repo_path(
            "reference/screenshots/r3code-settings-keybindings-add-window.png",
        ),
        max_different_pixels_percent: 6.45,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-providers".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-providers-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-providers-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-providers-window.png"),
        max_different_pixels_percent: 5.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-source-control".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path(
            "reference/screenshots/r3code-settings-source-control-window.png",
        ),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-source-control-reference.png",
        ),
        actual: resolve_repo_path(
            "reference/screenshots/r3code-settings-source-control-window.png",
        ),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-connections".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-connections-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-connections-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-connections-window.png"),
        max_different_pixels_percent: 4.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-diagnostics".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-diagnostics-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-diagnostics-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-diagnostics-window.png"),
        max_different_pixels_percent: 5.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-archive".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-archive-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-archive-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-archive-window.png"),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-back".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-back-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-empty-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-back-window.png"),
        max_different_pixels_percent: 2.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-theme-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-theme-menu-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/upstream-settings-theme-menu-reference.png",
        ),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-theme-menu-window.png"),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 8,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("settings-dark".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-dark-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-settings-dark-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-dark-window.png"),
        max_different_pixels_percent: 6.0,
        channel_tolerance: 11,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        theme: Some("dark".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-dark-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/upstream-empty-dark-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-dark-window.png"),
        max_different_pixels_percent: 2.0,
        channel_tolerance: 11,
        ignore_rects: vec![Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 45,
        }],
    })?;

    println!("R3Code parity checks passed.");
    Ok(())
}

fn parse_compare_options(args: &[String]) -> Result<CompareOptions> {
    let mut expected = None;
    let mut actual = None;
    let mut max_different_pixels_percent = 1.0;
    let mut channel_tolerance = 0;
    let mut ignore_rects = Vec::new();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--expected" => {
                index += 1;
                expected = Some(resolve_repo_path(required_arg(args, index, "--expected")?));
            }
            "--actual" => {
                index += 1;
                actual = Some(resolve_repo_path(required_arg(args, index, "--actual")?));
            }
            "--max-different-pixels-percent" => {
                index += 1;
                max_different_pixels_percent =
                    required_arg(args, index, "--max-different-pixels-percent")?.parse()?;
            }
            "--channel-tolerance" => {
                index += 1;
                channel_tolerance = required_arg(args, index, "--channel-tolerance")?.parse()?;
            }
            "--ignore-rect" => {
                index += 1;
                ignore_rects.push(parse_rect(required_arg(args, index, "--ignore-rect")?)?);
            }
            other => return Err(format!("unknown compare option: {other}").into()),
        }
        index += 1;
    }

    Ok(CompareOptions {
        expected: expected.ok_or("--expected is required")?,
        actual: actual.ok_or("--actual is required")?,
        max_different_pixels_percent,
        channel_tolerance,
        ignore_rects,
    })
}

fn parse_capture_r3code_options(args: &[String]) -> Result<CaptureR3CodeOptions> {
    let mut options = CaptureR3CodeOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--exe" => {
                index += 1;
                options.exe = resolve_repo_path(required_arg(args, index, "--exe")?);
            }
            "--output" => {
                index += 1;
                options.output = resolve_repo_path(required_arg(args, index, "--output")?);
            }
            "--screen" => {
                index += 1;
                options.screen = Some(required_arg(args, index, "--screen")?.to_string());
            }
            "--theme" => {
                index += 1;
                options.theme = Some(required_arg(args, index, "--theme")?.to_string());
            }
            "--delay-seconds" => {
                index += 1;
                options.delay =
                    Duration::from_secs(required_arg(args, index, "--delay-seconds")?.parse()?);
            }
            "--direct" => {
                options.direct = true;
            }
            "--offscreen" => {
                options.offscreen = true;
            }
            "--allow-window-capture" => {
                options.allow_window_capture = true;
            }
            other => return Err(format!("unknown capture-r3code-window option: {other}").into()),
        }
        index += 1;
    }
    Ok(options)
}

fn parse_capture_reference_options(args: &[String]) -> Result<CaptureReferenceOptions> {
    let mut options = CaptureReferenceOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                options.repo = PathBuf::from(required_arg(args, index, "--repo")?);
            }
            "--home" => {
                index += 1;
                options.home = PathBuf::from(required_arg(args, index, "--home")?);
            }
            "--output-dir" => {
                index += 1;
                options.output_dir = resolve_repo_path(required_arg(args, index, "--output-dir")?);
            }
            "--startup-timeout-seconds" => {
                index += 1;
                options.startup_timeout = Duration::from_secs(
                    required_arg(args, index, "--startup-timeout-seconds")?.parse()?,
                );
            }
            other => {
                return Err(format!("unknown capture-reference-browser option: {other}").into());
            }
        }
        index += 1;
    }
    Ok(options)
}

fn parse_generate_parity_inventory_options(
    args: &[String],
) -> Result<GenerateParityInventoryOptions> {
    let mut options = GenerateParityInventoryOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                options.repo = resolve_repo_path(required_arg(args, index, "--repo")?);
            }
            "--output" => {
                index += 1;
                options.output = resolve_repo_path(required_arg(args, index, "--output")?);
            }
            other => {
                return Err(format!("unknown generate-parity-inventory option: {other}").into());
            }
        }
        index += 1;
    }
    Ok(options)
}

impl Default for CaptureR3CodeOptions {
    fn default() -> Self {
        Self {
            exe: resolve_repo_path("target/debug/r3code.exe"),
            output: resolve_repo_path("reference/screenshots/r3code-window.png"),
            screen: None,
            theme: None,
            delay: Duration::from_secs(6),
            direct: false,
            offscreen: false,
            allow_window_capture: false,
        }
    }
}

impl Default for CaptureReferenceOptions {
    fn default() -> Self {
        let temp = env::temp_dir();
        Self {
            repo: temp.join("upstream-inspect"),
            home: temp.join("upstream-reference-home"),
            output_dir: resolve_repo_path("reference/screenshots"),
            startup_timeout: Duration::from_secs(90),
        }
    }
}

impl Default for GenerateParityInventoryOptions {
    fn default() -> Self {
        Self {
            repo: resolve_repo_path(".omx/upstream-t3code"),
            output: resolve_repo_path("docs/reference/PARITY_FILE_INVENTORY.md"),
        }
    }
}

fn required_arg<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a str> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{name} requires a value").into())
}

fn parse_rect(value: &str) -> Result<Rect> {
    let parts: Vec<u32> = value
        .split(',')
        .map(str::trim)
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    if parts.len() != 4 {
        return Err(format!("invalid rectangle '{value}', expected x,y,width,height").into());
    }
    Ok(Rect {
        x: parts[0],
        y: parts[1],
        width: parts[2],
        height: parts[3],
    })
}

fn compare_screenshots(options: CompareOptions) -> Result<()> {
    let expected = image::open(&options.expected)?.to_rgba8();
    let actual = image::open(&options.actual)?.to_rgba8();

    if expected.dimensions() != actual.dimensions() {
        return Err(format!(
            "image dimensions differ. Expected {}x{}, actual {}x{}",
            expected.width(),
            expected.height(),
            actual.width(),
            actual.height()
        )
        .into());
    }

    let mut different_pixels = 0u64;
    let mut ignored_pixels = 0u64;
    let total_pixels = u64::from(expected.width()) * u64::from(expected.height());

    for y in 0..expected.height() {
        for x in 0..expected.width() {
            if options
                .ignore_rects
                .iter()
                .any(|rect| point_in_rect(x, y, *rect))
            {
                ignored_pixels += 1;
                continue;
            }
            let left = expected.get_pixel(x, y).0;
            let right = actual.get_pixel(x, y).0;
            if pixel_different(left, right, options.channel_tolerance) {
                different_pixels += 1;
            }
        }
    }

    let compared_pixels = total_pixels - ignored_pixels;
    let different_percent = (different_pixels as f64 / compared_pixels as f64) * 100.0;
    println!(
        "Different pixels: {different_pixels}/{compared_pixels} ({different_percent:.3}%). Ignored: {ignored_pixels}. Channel tolerance: {}. Limit: {:.3}%.",
        options.channel_tolerance, options.max_different_pixels_percent
    );

    if different_percent > options.max_different_pixels_percent {
        return Err(format!(
            "screenshot comparison failed: {different_percent:.3}% > {:.3}%",
            options.max_different_pixels_percent
        )
        .into());
    }

    Ok(())
}

fn point_in_rect(x: u32, y: u32, rect: Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn pixel_different(left: [u8; 4], right: [u8; 4], tolerance: u8) -> bool {
    left.iter()
        .zip(right)
        .any(|(a, b)| a.abs_diff(b) > tolerance)
}

#[cfg(windows)]
fn capture_r3code_window(mut options: CaptureR3CodeOptions) -> Result<()> {
    if !options.allow_window_capture {
        return Err(
            "capture-r3code-window launches a native capture window; rerun with --allow-window-capture"
                .into(),
        );
    }

    if !options.direct {
        options.direct = true;
        options.offscreen = true;
    }
    capture_r3code_window_direct(&options)
}

#[cfg(windows)]
fn capture_r3code_window_direct(options: &CaptureR3CodeOptions) -> Result<()> {
    fs::create_dir_all(
        options
            .output
            .parent()
            .ok_or("capture output must have a parent directory")?,
    )?;

    let mut command = Command::new(&options.exe);
    if let Some(screen) = &options.screen {
        match screen.as_str() {
            "command-palette" => {}
            "settings-theme-menu"
            | "settings-dark"
            | "settings-back"
            | "settings-keybindings"
            | "settings-keybindings-add"
            | "settings-providers"
            | "settings-source-control"
            | "settings-connections"
            | "settings-archive" => {
                command.env("R3CODE_SCREEN", "settings");
            }
            _ => {
                command.env("R3CODE_SCREEN", screen);
            }
        }
    }
    if let Some(theme) = &options.theme {
        command.env("R3CODE_THEME", theme);
    }
    let mut child = command.spawn()?;

    let result = (|| -> Result<()> {
        let wait_started = Instant::now();
        let hwnd = wait_window_for_pid(child.id(), options.delay)?;
        if options.offscreen {
            prepare_window_for_offscreen_capture(hwnd);
        } else {
            prepare_window_for_capture(hwnd);
        }
        if let Some(remaining) = options.delay.checked_sub(wait_started.elapsed()) {
            thread::sleep(remaining);
        }
        if options.screen.as_deref() == Some("command-palette") {
            click_command_palette_trigger(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-theme-menu") {
            click_settings_theme_select(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-dark") {
            click_settings_theme_select(hwnd)?;
            thread::sleep(Duration::from_millis(150));
            click_settings_theme_dark_option(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-back") {
            click_settings_back(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-keybindings") {
            click_settings_keybindings_nav(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-keybindings-add") {
            click_settings_keybindings_nav(hwnd)?;
            thread::sleep(Duration::from_millis(250));
            click_settings_keybindings_add(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-providers") {
            click_settings_providers_nav(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-source-control") {
            click_settings_source_control_nav(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-connections") {
            click_settings_connections_nav(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-archive") {
            click_settings_archive_nav(hwnd)?;
            thread::sleep(Duration::from_millis(350));
        }
        if capture_window_with_graphics_capture(hwnd, &options.output).is_err() {
            let image = capture_client_area(hwnd)?;
            image.save(&options.output)?;
        }
        println!("{}", options.output.display());
        Ok(())
    })();

    stop_child(&mut child);
    result
}

#[cfg(not(windows))]
fn capture_r3code_window(_options: CaptureR3CodeOptions) -> Result<()> {
    Err("capture-r3code-window is currently implemented for Windows only".into())
}

#[cfg(windows)]
fn prepare_window_for_capture(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
    }
    thread::sleep(Duration::from_millis(350));
}

#[cfg(windows)]
fn prepare_window_for_offscreen_capture(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_BOTTOM),
            -20000,
            -20000,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    thread::sleep(Duration::from_millis(350));
}

#[cfg(windows)]
fn click_command_palette_trigger(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 76, 74)
}

#[cfg(windows)]
fn click_settings_theme_select(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 1050, 135)
}

#[cfg(windows)]
fn click_settings_theme_dark_option(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 1050, 229)
}

#[cfg(windows)]
fn click_settings_back(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 76, 778)
}

#[cfg(windows)]
fn click_settings_keybindings_nav(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 78, 102)
}

#[cfg(windows)]
fn click_settings_keybindings_add(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 1208, 84)
}

#[cfg(windows)]
fn click_settings_providers_nav(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 78, 134)
}

#[cfg(windows)]
fn click_settings_source_control_nav(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 78, 166)
}

#[cfg(windows)]
fn click_settings_connections_nav(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 78, 198)
}

#[cfg(windows)]
fn click_settings_archive_nav(hwnd: HWND) -> Result<()> {
    send_client_click(hwnd, 78, 232)
}

#[cfg(windows)]
fn send_client_click(hwnd: HWND, x: i32, y: i32) -> Result<()> {
    let position = mouse_position_lparam(x, y)?;
    unsafe {
        PostMessageW(Some(hwnd), WM_MOUSEMOVE, WPARAM(0), position)?;
        PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(1), position)?;
        PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), position)?;
    }
    Ok(())
}

#[cfg(windows)]
fn mouse_position_lparam(x: i32, y: i32) -> Result<LPARAM> {
    if !(0..=i16::MAX as i32).contains(&x) || !(0..=i16::MAX as i32).contains(&y) {
        return Err(format!("mouse position is outside client coordinate range: {x},{y}").into());
    }
    Ok(LPARAM((((y as u32) << 16) | (x as u32)) as isize))
}

#[cfg(windows)]
fn crop_capture_to_client_size(path: &Path, width: u32, height: u32) -> Result<()> {
    let image = image::open(path)?.to_rgba8();
    if image.width() == width && image.height() == height {
        return Ok(());
    }

    if image.width() < width || image.height() < height {
        return Err(format!(
            "captured image is smaller than client bounds: {}x{} < {width}x{height}",
            image.width(),
            image.height()
        )
        .into());
    }

    let crop_x = (image.width() - width) / 2;
    let crop_y = (image.height() - height) / 2;
    let cropped = image::imageops::crop_imm(&image, crop_x, crop_y, width, height).to_image();
    cropped.save(path)?;
    Ok(())
}

#[cfg(windows)]
fn capture_window_with_graphics_capture(hwnd: HWND, output: &Path) -> Result<()> {
    struct SingleFrameCapture {
        output: PathBuf,
    }

    impl GraphicsCaptureApiHandler for SingleFrameCapture {
        type Error = Box<dyn std::error::Error + Send + Sync>;
        type Flags = PathBuf;

        fn new(ctx: CaptureContext<Self::Flags>) -> std::result::Result<Self, Self::Error> {
            Ok(Self { output: ctx.flags })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> std::result::Result<(), Self::Error> {
            frame.save_as_image(&self.output, CaptureImageFormat::Png)?;
            capture_control.stop();
            Ok(())
        }
    }

    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect)?;
    }
    let width = (rect.right - rect.left) as u32;
    let height = (rect.bottom - rect.top) as u32;

    let window = CaptureWindow::from_raw_hwnd(hwnd.0.cast());
    let settings = Settings::new(
        window,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Include,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        output.to_path_buf(),
    );
    SingleFrameCapture::start(settings)?;
    crop_capture_to_client_size(output, width, height)?;
    Ok(())
}

#[cfg(windows)]
fn wait_window_for_pid(pid: u32, timeout: Duration) -> Result<HWND> {
    let deadline = Instant::now() + timeout.max(Duration::from_secs(1));
    loop {
        match find_window_for_pid(pid) {
            Ok(hwnd) => return Ok(hwnd),
            Err(error) if Instant::now() >= deadline => return Err(error),
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }
}

#[cfg(windows)]
fn find_window_for_pid(pid: u32) -> Result<HWND> {
    struct Search {
        pid: u32,
        hwnd: HWND,
        area: i32,
    }

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut Search) };
        let mut window_pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        }
        if window_pid == search.pid && unsafe { IsWindowVisible(hwnd).as_bool() } {
            let mut rect = RECT::default();
            if unsafe { GetClientRect(hwnd, &mut rect) }.is_err() {
                return BOOL(1);
            }
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width <= 0 || height <= 0 {
                return BOOL(1);
            }
            let area = width.saturating_mul(height);
            if area <= search.area {
                return BOOL(1);
            }
            search.hwnd = hwnd;
            search.area = area;
        }
        BOOL(1)
    }

    let mut search = Search {
        pid,
        hwnd: HWND(std::ptr::null_mut()),
        area: 0,
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_window),
            LPARAM((&mut search as *mut Search) as isize),
        );
    }
    if search.hwnd.0.is_null() {
        Err(format!("R3Code did not expose a visible main window for pid {pid}").into())
    } else {
        Ok(search.hwnd)
    }
}

#[cfg(windows)]
fn capture_client_area(hwnd: HWND) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect)?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err(format!("invalid client bounds {width}x{height}").into());
        }

        let window_dc = GetDC(Some(hwnd));
        if window_dc.is_invalid() {
            return Err("GetDC(hwnd) failed".into());
        }
        let memory_dc = CreateCompatibleDC(Some(window_dc));
        if memory_dc.is_invalid() {
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("CreateCompatibleDC failed".into());
        }
        let bitmap = CreateCompatibleBitmap(window_dc, width, height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("CreateCompatibleBitmap failed".into());
        }

        let old_object = SelectObject(memory_dc, HGDIOBJ(bitmap.0));
        let printed = PrintWindow(hwnd, memory_dc, PW_CLIENTONLY).as_bool();
        if !printed {
            let mut origin = POINT { x: 0, y: 0 };
            if !ClientToScreen(hwnd, &mut origin).as_bool() {
                let _ = SelectObject(memory_dc, old_object);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err("PrintWindow and ClientToScreen failed".into());
            }

            let screen_dc = GetDC(None);
            if screen_dc.is_invalid() {
                let _ = SelectObject(memory_dc, old_object);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err("PrintWindow failed and GetDC failed".into());
            }
            let copied = BitBlt(
                memory_dc,
                0,
                0,
                width,
                height,
                Some(screen_dc),
                origin.x,
                origin.y,
                SRCCOPY,
            );
            let _ = ReleaseDC(None, screen_dc);
            if copied.is_err() {
                let _ = SelectObject(memory_dc, old_object);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err("PrintWindow and BitBlt failed".into());
            }
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; (width * height * 4) as usize];
        let rows = GetDIBits(
            memory_dc,
            bitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );

        let _ = SelectObject(memory_dc, old_object);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(memory_dc);
        let _ = ReleaseDC(Some(hwnd), window_dc);

        if rows == 0 {
            return Err("GetDIBits failed".into());
        }

        let mut rgba = Vec::with_capacity(bgra.len());
        for pixel in bgra.chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }

        ImageBuffer::from_raw(width as u32, height as u32, rgba)
            .ok_or_else(|| "failed to create captured image".into())
    }
}

#[derive(Debug)]
struct InventoryRow {
    path: String,
    rust_target: &'static str,
    status: &'static str,
    proof: &'static str,
    remaining_gap: &'static str,
}

fn generate_parity_inventory(options: GenerateParityInventoryOptions) -> Result<()> {
    if !options.repo.exists() {
        return Err(format!(
            "upstream checkout not found at {}; run capture-reference-browser or pass --repo",
            options.repo.display()
        )
        .into());
    }

    let head = command_stdout(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&options.repo),
    )?;
    let mut source_files = Vec::new();
    for child in ["apps", "packages"] {
        let path = options.repo.join(child);
        if path.exists() {
            collect_inventory_source_files(&path, &options.repo, &mut source_files)?;
        }
    }
    source_files.sort();

    let rows = source_files
        .into_iter()
        .map(|path| classify_inventory_path(&path))
        .collect::<Vec<_>>();

    let mut status_counts = std::collections::BTreeMap::<&'static str, usize>::new();
    for row in &rows {
        *status_counts.entry(row.status).or_default() += 1;
    }

    let mut body = String::new();
    body.push_str("# T3 Code File-Level Parity Inventory\n\n");
    body.push_str("Generated by `cargo run -p xtask -- generate-parity-inventory`.\n\n");
    body.push_str("Do not edit individual rows by hand; update `crates/xtask/src/main.rs` classification rules or regenerate from the pinned upstream checkout.\n\n");
    body.push_str(&format!("- Upstream commit: `{}`\n", head.trim()));
    body.push_str("- Inventory root: `apps/` and `packages/`\n");
    body.push_str(&format!("- Tracked files: `{}`\n\n", rows.len()));
    body.push_str("## Status Counts\n\n");
    body.push_str("| Status | Files |\n| --- | ---: |\n");
    for (status, count) in status_counts {
        body.push_str(&format!("| `{status}` | {count} |\n"));
    }
    body.push_str("\n## Files\n\n");
    body.push_str("| Upstream file | Rust target | Status | Current proof | Remaining gap |\n");
    body.push_str("| --- | --- | --- | --- | --- |\n");
    for row in rows {
        body.push_str(&format!(
            "| `{}` | {} | `{}` | {} | {} |\n",
            escape_markdown_table_cell(&row.path),
            escape_markdown_table_cell(row.rust_target),
            row.status,
            escape_markdown_table_cell(row.proof),
            escape_markdown_table_cell(row.remaining_gap)
        ));
    }

    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&options.output, body)?;
    println!("Wrote {}", options.output.display());
    Ok(())
}

fn collect_inventory_source_files(dir: &Path, root: &Path, files: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name == "node_modules" || file_name == "dist" || file_name == ".turbo" {
            continue;
        }
        if path.is_dir() {
            collect_inventory_source_files(&path, root, files)?;
        } else if is_inventory_source_file(&path) {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            files.push(relative);
        }
    }
    Ok(())
}

fn is_inventory_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "json" | "mjs" | "cjs" | "css" | "html" | "md")
    )
}

fn classify_inventory_path(path: &str) -> InventoryRow {
    let (rust_target, status, proof, remaining_gap) = if path
        == "apps/web/src/contextMenuFallback.ts"
        || path == "apps/web/src/contextMenuFallback.test.ts"
    {
        (
            "context-menu fallback button planning, destructive/delete leaf styling, disabled handling, submenu chevron metadata, click resolution, clamp positioning, and submenu flip positioning in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core context_menu_fallback_helpers_match_upstream_contract`",
            "Wire helpers into a live GPUI/native context menu fallback path and exact pointer/keyboard cleanup behavior.",
        )
    } else if path == "apps/web/src/clientPersistenceStorage.ts"
        || path == "apps/web/src/clientPersistenceStorage.test.ts"
    {
        (
            "R3-branded browser persistence storage keys, saved-environment registry document transforms, secret-preserving registry rewrites, secret read/write/remove behavior, and desktop SSH record preservation in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core client_persistence_storage_preserves_saved_environment_secrets`",
            "Wire transforms into live GPUI browser/local persistence and schema-backed client settings decode/encode.",
        )
    } else if path == "apps/web/src/hooks/useMediaQuery.ts" {
        (
            "media-query breakpoint table, min/max query resolution, colon query parsing, raw parenthesized query passthrough, structured min/max/pointer query construction, unknown query fallback, and server snapshot false behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core media_query_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI responsive layout subscriptions and viewport-driven shell rendering.",
        )
    } else if path == "apps/web/src/hooks/useCommitOnBlur.ts" {
        (
            "commit-on-blur input draft state machine for unfocused upstream sync, focused edit preservation, change handling, changed-only blur commit, and Enter prevent-default/blur behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core commit_on_blur_helpers_match_upstream_contract`",
            "Wire state helper into live GPUI settings inputs and exact focus/keyboard event handling.",
        )
    } else if path == "apps/web/src/hooks/useCopyToClipboard.ts" {
        (
            "copy-to-clipboard state helper for default/custom timeout, unavailable Clipboard API errors, empty-value no-op, successful write copied state, prior timeout clearing, optional reset scheduling, write-error callback/logging, timeout reset, and cleanup in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core copy_to_clipboard_helpers_match_upstream_contract`",
            "Wire helper into live GPUI clipboard actions and real async clipboard/write timer behavior.",
        )
    } else if path == "apps/web/src/hooks/useHandleNewThread.ts" {
        (
            "new-thread hook planning for preferred project ordering/default project ref, logical project key fallback, stored draft reuse and same-route no-op, option-scoped draft context patches, active unpromoted draft reuse, new draft/thread creation metadata, sticky-state application, and draft navigation decisions in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core handle_new_thread_helpers_match_upstream_draft_reuse_and_creation`",
            "Wire helpers into live GPUI command palette/sidebar new-thread actions, composer draft store mutations, generated IDs/timestamps, and router navigation.",
        )
    } else if path == "apps/web/src/hooks/useLocalStorage.ts" {
        (
            "R3-branded same-tab local-storage change event, isomorphic in-memory storage behavior, JSON encode/decode helpers, initial read fallback/error logging, null-as-remove setter plan, queued custom change event, key-change resync, storage/custom subscription names, and matching event filters in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core local_storage_helpers_match_upstream_contract_with_r3_event_name`",
            "Wire helpers into live GPUI/local persistence hooks with schema-backed decode/encode and actual same-tab/cross-tab subscriptions.",
        )
    } else if path == "apps/web/src/hooks/useSettings.ts" {
        (
            "settings hook client/server routing contract: server-settings key set, unified patch splitting, client-over-server merge order, client settings hydration de-dupe/start decisions, persisted/default snapshot merge, hydration/persist error scopes, updateSettings server optimistic/RPC decisions, client persistence merge, and reset patch behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core settings_hook_helpers_match_upstream_client_server_routing`",
            "Wire helpers into live GPUI settings subscriptions, local API persistence, server config updates, and selector-level re-render behavior.",
        )
    } else if path == "apps/web/src/hooks/useTheme.ts" {
        (
            "R3-branded theme storage key, stored theme parsing, default/server snapshot, system-dark resolution, browser chrome surface/color/meta decisions, DOM apply plan, desktop bridge de-dupe/rejection handling, set-theme emission, subscription keys, storage-event filter, and system preference change behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core web_theme_helpers_match_upstream_contract_with_r3_storage_key`",
            "Wire helpers into live GPUI theme persistence, OS appearance subscriptions, browser chrome/meta updates, and desktop bridge theme sync.",
        )
    } else if path == "apps/web/src/hooks/useThreadActions.ts" {
        (
            "thread action hook planning for archive/unarchive command dispatch, running-thread archive guard, route-matched new-thread navigation, deleted thread-key filtering, fallback thread selection after delete, direct archived-thread delete, session stop/terminal close/draft cleanup, orphaned worktree confirmation/removal, git invalidation, and delete confirmation copy in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core thread_actions_helpers_match_upstream_archive_delete_flow`",
            "Wire helpers into live GPUI sidebar/thread menus, environment API commands, local dialog confirmations, router navigation, terminal/composer cleanup, and worktree removal toasts.",
        )
    } else if path == "apps/web/src/hooks/useTurnDiffSummaries.ts" {
        (
            "turn diff summaries hook helper for empty active-thread fallback, active-thread summary passthrough, and inferred checkpoint turn-count mapping by completedAt order in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core turn_diff_summaries_hook_helpers_match_upstream_contract`",
            "Wire helper into live GPUI active-thread diff summary state and memoized checkpoint turn-count rendering.",
        )
    } else if path == "apps/web/src/env.ts" {
        (
            "Electron web-runtime detection for module-load window presence and desktopBridge/nativeApi preload bridge availability in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core web_env_electron_detection_matches_upstream_bridge_contract`",
            "Wire helper into live GPUI/web runtime initialization wherever desktop/browser behavior branches.",
        )
    } else if path == "apps/web/src/keybindings.ts" || path == "apps/web/src/keybindings.test.ts" {
        (
            "client shortcut matching, when-clause evaluation, physical key aliases, effective-label conflict resolution, thread/model picker jump hints, command wrappers, terminal clear/delete/navigation control bytes, and platform modifier mapping in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core keybindings_client_shortcut_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI keyboard event dispatch, terminal focus/open state, model picker state, and terminal input passthrough.",
        )
    } else if path == "apps/web/src/observability/clientTracing.ts" {
        (
            "client tracing OTLP path, default/min export interval, resource attributes, config key, active-delegate reuse, stale-generation disposal, warning formatting, and reset-state contracts in crates/r3_core/src/observability.rs",
            "partial",
            "`cargo test -p r3_core ports_client_tracing_configuration_contracts`",
            "Wire the plan into live GPUI tracing runtime setup with HTTP URL resolution, exporter delegate lifecycle, async serialization layer, and authenticated bootstrap.",
        )
    } else if path == "apps/web/src/threadDerivation.ts" {
        (
            "thread derivation from environment state by shell/session/turn/message/activity/proposed-plan/diff ID maps, missing-record filtering, latest-turn and pending-plan projection, and missing-shell fallback in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core derives_thread_from_environment_state_in_id_order`; `cargo test -p r3_core missing_thread_shell_returns_none`",
            "Port referential caching semantics for unchanged shell/session/collection identities and wire into live GPUI selectors.",
        )
    } else if path == "apps/web/src/types.ts" {
        (
            "web thread/message/session/proposed-plan/diff/terminal shape structs, session phase variants, runtime/interaction defaults, default terminal height/id, and max terminal group size in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core default_terminal_state_matches_upstream_contract`; `cargo test -p r3_core derives_thread_from_environment_state_in_id_order`",
            "Add remaining live modelSelection/project shape wiring and keep generated Rust DTOs aligned as upstream contract fields evolve.",
        )
    } else if path == "apps/web/src/localApi.ts" || path == "apps/web/src/localApi.test.ts" {
        (
            "local API cached/native/primary/browser selection, ensureLocalApi error, unavailable backend rejection, desktop bridge versus browser fallbacks for dialogs/context menu/persistence/openExternal, openExternal error copy, and reset-test coverage in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core local_api_selection_and_fallback_contracts_match_upstream`",
            "Wire exact async RPC forwarding, environment API composition, browser storage side effects, desktop bridge calls, and reset hooks into live GPUI runtime.",
        )
    } else if path == "apps/web/src/orchestrationRecovery.ts"
        || path == "apps/web/src/orchestrationRecovery.test.ts"
    {
        (
            "orchestration recovery coordinator state machine for bootstrap snapshots, sequence-gap replay, deferred live events, applied batch sorting, replay failure fallback, no-progress replay retry/backoff, retry cap, and frontier reset in crates/r3_core/src/orchestration.rs",
            "partial",
            "`cargo test -p r3_core orchestration_recovery`",
            "Wire the coordinator into live GPUI websocket shell subscriptions, snapshot/replay RPC calls, and retry timers.",
        )
    } else if path == "apps/web/src/orchestrationEventEffects.ts"
        || path == "apps/web/src/orchestrationEventEffects.test.ts"
    {
        (
            "orchestration batch effects for draft promotion, deleted-thread clearing, terminal state removal, provider invalidation on turn-diff/revert events, final lifecycle outcome wins, and insertion-order preservation in crates/r3_core/src/orchestration.rs",
            "partial",
            "`cargo test -p r3_core orchestration_batch_effects_match_upstream_lifecycle_rules`",
            "Wire effects into live GPUI store updates, provider query invalidation, draft promotion, deleted-thread cleanup, and terminal state cleanup after websocket replay batches.",
        )
    } else if path == "apps/web/src/rpc/atomRegistry.ts" {
        (
            "app-wide Effect AtomRegistry factory, React RegistryContext provider binding, exported mutable registry, and dispose-before-recreate test reset contract in crates/r3_core/src/rpc.rs",
            "partial",
            "`cargo test -p r3_core ports_app_atom_registry_provider_and_reset_contract`",
            "Wire the registry contract into live GPUI/reactivity state ownership and reset hooks for integration tests.",
        )
    } else if path == "apps/web/src/rpc/protocol.ts" {
        (
            "websocket RPC protocol URL resolution, Effect layer contract constants, reconnect retry policy, active lifecycle gating, request hook tracking/ack behavior, heartbeat timeout cleanup, protocol error cleanup, and R3-branded connection error copy in crates/r3_core/src/rpc.rs",
            "partial",
            "`cargo test -p r3_core ws_rpc_protocol`",
            "Wire the pure protocol state transitions into the live GPUI websocket transport, request hooks, heartbeat hooks, and backend websocket constructor lifecycle.",
        )
    } else if path == "apps/web/src/rpc/serverState.ts"
        || path == "apps/web/src/rpc/serverState.test.ts"
    {
        (
            "server config atom labels, default selectors, snapshot replay notifications, provider/settings/keybinding merge rules, welcome replay, fallback fetch suppression, cleanup subscription plan, and test reset ID behavior in crates/r3_core/src/rpc.rs",
            "partial",
            "`cargo test -p r3_core server_state_`",
            "Wire pure server-state transitions into live GPUI/reactivity subscriptions, websocket RPC client sync, and hook selectors.",
        )
    } else if path == "apps/web/src/rpc/wsRpcClient.ts"
        || path == "apps/web/src/rpc/wsRpcClient.test.ts"
    {
        (
            "websocket RPC client facade table for terminal/projects/filesystem/source-control/shell/vcs/git/server/orchestration methods, subscribe tags, no-arg empty inputs, settings patch wrapping, diagnostics tracing suppression, reconnect backoff reset, VCS status stream reduction, git stream final-result enforcement, and omitted replayEvents facade in crates/r3_core/src/rpc.rs",
            "partial",
            "`cargo test -p r3_core ws_rpc_client`; `cargo test -p r3_core shared::tests::ports_shared_git_and_source_control_helpers`",
            "Wire facade bindings into the live GPUI websocket transport client and end-to-end subscription/request execution.",
        )
    } else if path == "apps/web/src/rpc/wsTransport.ts"
        || path == "apps/web/src/rpc/wsTransport.test.ts"
    {
        (
            "websocket transport session lifecycle, disposed request/reconnect errors, reconnect tracked-request clearing, heartbeat freshness, active-session gating, intentional close detection, stream retry delay, transport-versus-application failure retry decisions, one-time disconnect warnings, listener-error swallowing, and onResubscribe tag gating in crates/r3_core/src/rpc.rs",
            "partial",
            "`cargo test -p r3_core ws_transport`; `cargo test -p r3_core transport_error_filtering_matches_upstream_patterns`; `cargo test -p r3_core ws_rpc_protocol`",
            "Wire the pure transport state machine into live Effect ManagedRuntime sessions, Scope close/dispose ordering, websocket callbacks, and end-to-end RPC stream execution.",
        )
    } else if path == "apps/web/src/markdown-links.ts"
        || path == "apps/web/src/markdown-links.test.ts"
    {
        (
            "markdown file URI rewrite, encoded-path preservation, file-link target resolution, line/column hash mapping, Windows path normalization, relative display labels, and app-route rejection in crates/r3_core/src/markdown.rs",
            "partial",
            "`cargo test -p r3_core markdown_links_helpers_match_upstream_contract`",
            "Wire exact link metadata into all live GPUI markdown click, tooltip, context-menu, and open-in-editor interactions.",
        )
    } else if path.starts_with("apps/web/src/components/ChatMarkdown") {
        (
            "chat markdown file-link rewrite/label contracts and GPUI assistant markdown rendering in crates/r3_core/src/markdown.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test -p r3_core markdown`; `cargo check --workspace`",
            "Port full ReactMarkdown/remark-gfm coverage, Shiki/diff highlighter cache, clipboard copy timers, preferred-editor open actions, context menu, tooltips, skill inline rendering, browser interaction tests, and exact CSS pixel styling.",
        )
    } else if path == "apps/web/src/components/AnimatedHeight.tsx" {
        (
            "AnimatedHeight data slot, transition class, fallback timeout, measured-height rounding, clipping transition state, and clear-on-transition-end behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace animated_height_contract_matches_upstream_component`",
            "Wire exact measured-height animation into any live GPUI surfaces that need dynamic height transitions.",
        )
    } else if path == "apps/web/src/components/AppSidebarLayout.tsx" {
        (
            "AppSidebarLayout provider/sidebar classes, resizable width constraints, width storage key, shortcut-modifier window listener wiring, and desktop open-settings menu routing in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace app_sidebar_layout_contract_matches_upstream_component`; `cargo test --workspace shortcut_modifier_state_matches_upstream_keyboard_logic`",
            "Wire the remaining live GPUI sidebar shell, resize persistence, desktop menu bridge subscription, and navigation effects.",
        )
    } else if path == "apps/web/src/components/chat/MessagesTimeline.logic.ts"
        || path == "apps/web/src/components/chat/MessagesTimeline.logic.test.ts"
    {
        (
            "message timeline duration boundaries, compact work labels, assistant copy visibility, terminal assistant-message selection, row derivation, and stable-row reuse helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core messages_timeline_logic`",
            "Port browser rendering, proposed-plan UI lifecycle, exact diff/revert controls, live scroll behavior, and remaining UI wiring.",
        )
    } else if path == "apps/web/src/components/chat/MessagesTimeline.test.tsx"
        || path == "apps/web/src/components/chat/MessagesTimeline.tsx"
    {
        (
            "MessagesTimeline rendered user-message collapse attributes, footer/copy affordances, inline terminal chip labels, context-compaction work logs, and workspace-relative changed-file previews in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace messages_timeline_component_contracts_match_upstream_rendering`; `cargo test --workspace messages_timeline_logic_helpers_match_upstream`",
            "Wire the remaining LegendList scroll lifecycle, proposed-plan UI lifecycle, exact diff/revert controls, image expansion, live timers, and GPUI row styling.",
        )
    } else if path == "apps/web/src/components/ProviderUpdateLaunchNotification.logic.ts"
        || path == "apps/web/src/components/ProviderUpdateLaunchNotification.logic.test.ts"
    {
        (
            "provider update candidate, one-click eligibility, toast, snapshot collection, and sidebar pill decision helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_update`",
            "Wire the decision helpers into live GPUI provider update UI and port exact component interactions.",
        )
    } else if path == "apps/web/src/components/ui/toast.logic.ts"
        || path == "apps/web/src/components/ui/toast.logic.test.ts"
    {
        (
            "toast collapsed-content, visible-stack layout, and thread-scoped visibility helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core toast_layout`",
            "Wire helpers into the live GPUI toast renderer and port exact toast animations/styles.",
        )
    } else if path == "apps/web/src/components/ui/qr-code.tsx"
        || path == "apps/web/src/components/ui/qr-code.test.tsx"
    {
        (
            "QRCodeSvg Nayuki encoding, SVG path run-length rendering, default high-contrast foreground/background fills, custom color fills, title/role/class attributes, and no-currentColor rendering in crates/r3_core/src/shared.rs",
            "partial",
            "`cargo test -p r3_core shared::tests::ports_shared_nayuki_qr_code_contracts`",
            "Wire the SVG renderer into live GPUI/browser pairing surfaces wherever upstream renders QRCodeSvg.",
        )
    } else if path == "apps/web/src/providerUpdateDismissal.ts"
        || path == "apps/web/src/providerUpdateDismissal.test.ts"
    {
        (
            "provider update dismissal storage key and notification-key transition helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_update_dismissals`",
            "Wire dismissal persistence into the live GPUI provider update notification surface.",
        )
    } else if path == "apps/web/src/rpc/wsConnectionState.ts"
        || path == "apps/web/src/rpc/wsConnectionState.test.ts"
    {
        (
            "websocket connection UI state, reconnect backoff, error/close hint, and retry scheduling helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core websocket_connection_state`",
            "Wire helpers into the live GPUI websocket connection surface and real socket lifecycle.",
        )
    } else if path == "apps/web/src/rpc/requestLatencyState.ts"
        || path == "apps/web/src/rpc/requestLatencyState.test.ts"
    {
        (
            "slow RPC ack request tracking, threshold promotion, acknowledgement, subscribe filtering, and capacity eviction helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core request_latency_state`",
            "Wire helpers into the live GPUI RPC layer timers and connection diagnostics UI.",
        )
    } else if path == "apps/web/src/lib/terminalStateCleanup.ts"
        || path == "apps/web/src/lib/terminalStateCleanup.test.ts"
    {
        (
            "active terminal thread retention cleanup helper in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core terminal_state_cleanup`",
            "Wire helper into the live GPUI terminal state store cleanup path.",
        )
    } else if path == "apps/web/src/lib/archivedThreadsState.ts" {
        (
            "archived-thread environment key sort/parse and refresh matching helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core archived_threads_environment_keys`",
            "Wire helpers into live GPUI archived-thread snapshot loading and refresh.",
        )
    } else if path == "apps/web/src/lib/processDiagnosticsState.ts"
        || path == "apps/web/src/lib/traceDiagnosticsState.ts"
    {
        (
            "diagnostics state stale/idle TTL constants and error fallback helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core diagnostics_state_constants`",
            "Wire helpers into live GPUI diagnostics SWR loading and refresh controls.",
        )
    } else if path == "apps/web/src/proposedPlan.ts" || path == "apps/web/src/proposedPlan.test.ts"
    {
        (
            "proposed plan markdown heading extraction, displayed markdown stripping, collapsed preview truncation, implementation prompt/submission, implementation thread title, filename sanitization, and export newline helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core proposed_plan_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI proposed-plan cards, export/download action, and implementation-thread creation.",
        )
    } else if path == "apps/web/src/pullRequestReference.ts"
        || path == "apps/web/src/pullRequestReference.test.ts"
    {
        (
            "pull request reference parser for GitHub/GitLab/Azure DevOps URLs, raw `42`/`#42` references, and `gh`/`glab`/`az` checkout commands in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core pull_request_reference_parser_matches_upstream_contract`",
            "Wire parser into the live GPUI pull-request checkout/reference entry points.",
        )
    } else if path == "apps/web/src/providerInstances.ts"
        || path == "apps/web/src/providerInstances.test.ts"
    {
        (
            "provider instance entry projection, display-name fallback, accent-color normalization, default-first sorting, exact instance lookup, model-list lookup, selectable fallback, and instance-to-driver resolution in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_instance_helpers_match_upstream_contract`; `cargo test -p r3_core provider_instance_projection_matches_upstream_logic`",
            "Wire helpers into every live GPUI provider instance picker, settings row, model list, and composer routing path.",
        )
    } else if path == "apps/web/src/providerModels.ts" {
        (
            "provider model helpers for driver label formatting, default-instance model lookup, provider display fallback, interaction toggle defaulting, enabled checks, selectable-provider fallback, model capabilities, and default server model selection in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_models_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI provider settings, composer provider state, provider model capability descriptors, and picker defaults.",
        )
    } else if path == "apps/web/src/pendingUserInput.ts"
        || path == "apps/web/src/pendingUserInput.test.ts"
    {
        (
            "pending user input draft normalization, custom-answer precedence, single-select and multi-select resolution, option toggling, complete answer map creation, answered counts, first-unanswered selection, and active progress derivation in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core pending_user_input`",
            "Wire helpers into live GPUI pending-user-input panel state, answer submission, and multi-question navigation.",
        )
    } else if path == "apps/web/src/modelOrdering.ts"
        || path == "apps/web/src/modelOrdering.test.ts"
    {
        (
            "provider model key construction, provider-instance model ordering by favorites/modelOrder/original order, and cross-provider item ordering by favorite group, instance order, and original provider-model order in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core model_ordering`",
            "Wire ordering helpers into all live GPUI model picker lists, favorites views, and provider instance sections.",
        )
    } else if path == "apps/web/src/modelSelection.ts"
        || path == "apps/web/src/modelSelection.test.ts"
    {
        (
            "instance-scoped model selection helpers for custom-model isolation, hidden built-in filtering, per-instance model ordering, selectable fallback, custom instance preservation, and default text-generation fallback in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core model_selection_instance_scoping_matches_upstream_contract`",
            "Wire helpers into the live GPUI composer/provider picker settings path, provider option descriptor dispatch, and persisted text-generation model selection updates.",
        )
    } else if path == "apps/web/src/providerSkillPresentation.ts"
        || path == "apps/web/src/providerSkillPresentation.test.ts"
        || path == "apps/web/src/providerSkillSearch.ts"
        || path == "apps/web/src/providerSkillSearch.test.ts"
    {
        (
            "provider skill display-name fallback, plugin/app install source detection with path separator normalization, scope labels, enabled-only filtering, `$` query normalization, exact/prefix/boundary/includes/fuzzy ranking, and deterministic tie-breaks in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_skill_presentation_and_search_match_upstream_contract`",
            "Wire helpers into live GPUI skill picker/menu surfaces, install-source labels, and provider skill search UI.",
        )
    } else if path == "apps/web/src/projectScripts.ts"
        || path == "apps/web/src/projectScripts.test.ts"
    {
        (
            "project script run command construction/parsing, 24-character script-id slugification/truncation/dedupe, primary/setup script selection, runtime env override semantics, and worktree-preferred cwd resolution in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core project_scripts_helpers_match_upstream_logic`; `cargo test -p r3_core project_script_runtime_context_matches_upstream_logic`",
            "Wire helpers into live GPUI project script creation, settings persistence, terminal launch, and worktree setup flows.",
        )
    } else if path == "apps/web/src/timestampFormat.ts"
        || path == "apps/web/src/timestampFormat.test.ts"
    {
        (
            "timestamp format option contracts plus relative elapsed, relative-until, and expires-in labels for seconds/minutes/hours/days in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core timestamp_format_helpers_match_upstream_contract`",
            "Wire helpers into all live GPUI timestamp labels, expiry countdowns, and user-configured time-format rendering.",
        )
    } else if path == "apps/web/src/threadRoutes.ts" || path == "apps/web/src/threadRoutes.test.ts"
    {
        (
            "thread route parameter builders, scoped route-ref resolution, server-before-draft target precedence, draft target fallback, and empty-param null handling in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core thread_routes_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI route parsing, navigation, history, and draft/server thread selection.",
        )
    } else if path == "apps/web/src/threadSelectionStore.ts"
        || path == "apps/web/src/threadSelectionStore.test.ts"
    {
        (
            "sidebar thread multi-selection state transitions for toggle, anchor-only click, shift-range selection, missing-anchor/target fallback, stable anchor extension, clear, removal, and has-selection checks in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core thread_selection_store_matches_upstream_contract`",
            "Wire state helper into the live GPUI sidebar thread multi-select interactions and bulk actions.",
        )
    } else if path == "apps/web/src/sidebarProjectGrouping.ts" {
        (
            "sidebar project physical-to-logical key mapping and grouped project snapshots with primary-environment representative selection, grouped labels, local/remote/mixed presence, member refs, and deduped remote labels in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core sidebar_project_grouping_matches_upstream_contract`; `cargo test -p r3_core logical_project_helpers_match_upstream_grouping_rules`",
            "Wire snapshot projection into the live GPUI sidebar project list, grouping settings, drag ordering, and environment badges.",
        )
    } else if path == "apps/web/src/sourceControlPresentation.ts" {
        (
            "source-control provider presentation wrapper with provider-name override, change-request terminology, and GitHub/GitLab/Azure DevOps/Bitbucket/generic icon mapping in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core source_control_presentation_matches_upstream_contract`; `cargo test -p r3_core shared::tests::ports_shared_git_and_source_control_helpers`",
            "Wire presentation into all live GPUI source-control buttons, settings rows, empty states, and change-request actions.",
        )
    } else if path == "apps/web/src/versionSkew.ts" || path == "apps/web/src/versionSkew.test.ts" {
        (
            "version mismatch normalization, R3Code-branded mismatch hint, environment/client/server dismissal keys, duplicate-safe dismissal insertion, dismissed checks, and error-message hint append behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core version_skew_helpers_match_upstream_contract_with_r3_hint`",
            "Wire helpers into live GPUI server-config mismatch banner, local dismissal persistence, and connection error hint rendering.",
        )
    } else if path == "apps/web/src/shortcutModifierState.ts"
        || path == "apps/web/src/shortcutModifierState.test.ts"
    {
        (
            "shortcut modifier equality, key normalization, and keyboard-event sync helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core shortcut_modifier_state`",
            "Wire helpers into live GPUI keyboard modifier tracking.",
        )
    } else if path == "apps/web/src/modelPickerOpenState.ts" {
        (
            "model picker open default state and no-op setter transition helper in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core model_picker_open_state`",
            "Wire helper into the live GPUI model picker open/close store.",
        )
    } else if path == "apps/web/src/components/settings/providerDriverMeta.ts" {
        (
            "provider driver client metadata, settings schema names, badge labels, and lookup helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_presentation_metadata`",
            "Wire metadata into the live GPUI provider settings renderer.",
        )
    } else if path == "apps/web/src/components/settings/providerStatus.ts" {
        (
            "provider status summary/version helpers plus exact status dot styles in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_status_summary`; `cargo test -p r3_core provider_presentation_metadata`",
            "Wire styles and summaries into the live GPUI provider settings cards.",
        )
    } else if path == "apps/web/src/components/chat/providerIconUtils.ts" {
        (
            "provider icon mapping, available picker options, and model display-label helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_presentation_metadata`; `cargo test -p r3_core model_picker`",
            "Wire icon mapping into live GPUI model picker/provider trigger rendering.",
        )
    } else if path == "apps/web/src/editorPreferences.ts" {
        (
            "preferred-editor storage key, stored editor selection, upstream editor-order fallback, and persistence decision helper in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core preferred_editor_resolution`",
            "Wire helper into live GPUI editor preference storage and shell open action.",
        )
    } else if path == "apps/web/src/branding.ts" || path == "apps/web/src/branding.test.ts" {
        (
            "hosted app channel normalization, injected desktop branding, display-name fallback, and app-version resolution in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core web_branding`",
            "Wire helper into live GPUI hosted/desktop branding initialization.",
        )
    } else if path == "apps/web/src/rightPanelLayout.ts" {
        (
            "right panel inline-layout media query and sheet class constants in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core right_panel_and_terminal_focus`",
            "Wire constants into the live GPUI right-panel layout.",
        )
    } else if path == "apps/web/src/lib/terminalFocus.ts"
        || path == "apps/web/src/lib/terminalFocus.test.ts"
    {
        (
            "terminal focus helper class/selector constants and active-element focus decision in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core right_panel_and_terminal_focus`",
            "Wire helper into live GPUI/xterm focus tracking.",
        )
    } else if path == "apps/web/src/terminalActivity.ts"
        || path == "apps/web/src/terminalActivity.test.ts"
    {
        (
            "terminal running-subprocess event projection and pending event filters in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core terminal_activity_and_event_filters`",
            "Wire event projection into live GPUI terminal activity updates.",
        )
    } else if path == "apps/web/src/hostedPairing.ts"
        || path == "apps/web/src/hostedPairing.test.ts"
    {
        (
            "hosted static app detection, hosted pairing request parsing, hosted pairing URL construction, and channel selection URL helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core hosted_pairing_helpers`",
            "Wire helpers into live GPUI hosted pairing bootstrap.",
        )
    } else if path == "apps/web/src/pairingUrl.ts" {
        (
            "pairing token get/strip/set URL helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core parses_remote_pairing_fields`",
            "Wire helpers into live GPUI pairing URL handling.",
        )
    } else if path == "apps/web/src/components/settings/pairingUrls.ts"
        || path == "apps/web/src/components/settings/pairingUrls.test.ts"
    {
        (
            "desktop and hosted settings pairing URL resolution helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core selects_and_resolves_advertised_pairing_endpoints`; `cargo test -p r3_core hosted_pairing_helpers`",
            "Wire helpers into live GPUI connection settings pairing links.",
        )
    } else if path == "apps/web/src/worktreeCleanup.ts"
        || path == "apps/web/src/worktreeCleanup.test.ts"
    {
        (
            "orphaned worktree path detection and worktree path display formatting helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core worktree_cleanup_helpers`",
            "Wire helpers into live GPUI thread/worktree deletion flow.",
        )
    } else if path == "apps/web/src/lib/diffRendering.ts"
        || path == "apps/web/src/lib/diffRendering.test.ts"
    {
        (
            "diff theme-name resolution, UTF-16 FNV-1a patch hashing, and patch cache key construction in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core diff_rendering_and_model_highlight`",
            "Wire helpers into live GPUI diff highlighter/cache rendering.",
        )
    } else if path == "apps/web/src/lib/lruCache.ts" || path == "apps/web/src/lib/lruCache.test.ts"
    {
        (
            "string-keyed LRU cache promotion, replacement, entry-count eviction, memory-budget eviction, and clear behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core lru_cache_matches_upstream_entry_and_memory_eviction`",
            "Wire cache into live GPUI markdown/highlighter surfaces where upstream uses lruCache.ts.",
        )
    } else if path == "apps/web/src/lib/windowControlsOverlay.ts" {
        (
            "window controls overlay `wco` class sync and geometrychange listener decisions in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core right_panel_and_terminal_focus_helpers`",
            "Wire live GPUI/Electron titlebar overlay state into shell root class handling.",
        )
    } else if path == "apps/web/src/components/chat/DiffStatLabel.tsx" {
        (
            "DiffStatLabel non-zero predicate plus exact +additions/-deletions fragment text and class contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core turn_diff_stats_sum_only_files_with_numeric_values`",
            "Wire core label segments into live GPUI changed-files and timeline diff labels.",
        )
    } else if path == "apps/web/src/components/chat/ProviderStatusBanner.tsx" {
        (
            "provider status banner visibility, provider label fallback, alert variant, title, message fallback, and class/icon contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_status_summary`",
            "Wire core banner plan into live GPUI chat provider status surface.",
        )
    } else if path == "apps/web/src/components/chat/ThreadErrorBanner.tsx" {
        (
            "thread error banner nullability, alert description, optional dismiss action, class/icon, and aria-label contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core transport_error_filtering`",
            "Wire core banner plan into live GPUI thread error surface.",
        )
    } else if path == "apps/web/src/components/chat/modelPickerModelHighlights.ts" {
        (
            "model picker new-model highlight key lookup in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core diff_rendering_and_model_highlight`",
            "Wire highlight lookup into live GPUI model picker list rows.",
        )
    } else if path == "apps/web/src/rpc/transportError.ts"
        || path == "apps/web/src/rpc/transportError.test.ts"
    {
        (
            "transport connection error pattern detection and thread error sanitization helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core transport_error_filtering`",
            "Wire helper into live GPUI thread error surfaces.",
        )
    } else if path == "apps/web/src/commandPaletteStore.ts" {
        (
            "command palette open state, toggle behavior, add-project open intent request IDs, and intent clearing helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core command_palette_store_transitions`",
            "Wire helpers into the live GPUI command palette store.",
        )
    } else if path == "apps/web/src/components/chat/composerMenuHighlight.ts"
        || path == "apps/web/src/components/chat/composerMenuHighlight.test.ts"
        || path == "apps/web/src/components/composerFooterLayout.ts"
        || path == "apps/web/src/components/composerFooterLayout.test.ts"
    {
        (
            "composer menu active-highlight resolution, highlight nudging, and composer footer compact breakpoint helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core composer_menu_grouping_highlight`",
            "Wire footer breakpoint helpers into live GPUI composer layout.",
        )
    } else if path == "apps/web/src/lib/projectPaths.ts"
        || path == "apps/web/src/lib/projectPaths.test.ts"
    {
        (
            "project path dispatch/comparison normalization, explicit-relative detection/resolution, browse query gating, title inference, path matching, and browse navigation helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core project_paths_helpers_match_upstream_contract`",
            "Wire helpers into live GPUI project picker/filesystem browse flow.",
        )
    } else if path == "apps/web/src/lib/utils.ts" || path == "apps/web/src/lib/utils.test.ts" {
        (
            "web platform detection helpers for macOS/iOS, Windows, and Linux in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core keybinding_shortcuts_and_when_expressions`",
            "Port cn/twMerge behavior and UUID-backed command/project/thread/draft/message ID factories where live GPUI needs them.",
        )
    } else if path == "apps/web/src/lib/chatThreadActions.ts"
        || path == "apps/web/src/lib/chatThreadActions.test.ts"
    {
        (
            "chat thread action project-ref resolution plus contextual/default new-thread option planning in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core chat_thread_action_plans_match_upstream_context_resolution`",
            "Wire plans into live GPUI new-thread commands and async handler dispatch.",
        )
    } else if path == "apps/web/src/lib/contextWindow.ts"
        || path == "apps/web/src/lib/contextWindow.test.ts"
    {
        (
            "context-window activity snapshot derivation, token usage percentages/remaining counts, compaction flag defaulting, and compact token formatting in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core context_window_snapshot_and_token_format`",
            "Wire snapshot derivation into live GPUI ContextWindowMeter state.",
        )
    } else if path == "apps/web/src/lib/desktopUpdateReactQuery.ts"
        || path == "apps/web/src/lib/desktopUpdateReactQuery.test.ts"
    {
        (
            "desktop update React Query cache keys and state query stale/refetch options in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop_updates_runtime_contracts`",
            "Wire live query cache updates and desktopBridge update-state subscription into GPUI settings state.",
        )
    } else if path == "apps/web/src/components/desktopUpdate.logic.ts"
        || path == "apps/web/src/components/desktopUpdate.logic.test.ts"
    {
        (
            "desktop update button visibility/action/disabled/tooltip predicates, update action error/toast decisions, Apple Silicon Intel-build warning copy, install confirmation copy with R3Code naming, and check-eligibility rules in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop_updates_runtime_contracts`",
            "Wire helpers into live GPUI desktop update controls, confirmation dialogs, retry toasts, and desktopBridge action dispatch.",
        )
    } else if path == "apps/web/src/lib/gitReactQuery.ts"
        || path == "apps/web/src/lib/gitReactQuery.test.ts"
    {
        (
            "git React Query key factories, cwd-scoped mutation keys, branch-search option metadata, and invalidation target selection in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core git_react_query_keys_match_upstream_scoping`",
            "Wire live environment API query/mutation execution, progress callbacks, and query-client invalidation into GPUI git flows.",
        )
    } else if path == "apps/web/src/lib/gitStatusState.ts"
        || path == "apps/web/src/lib/gitStatusState.test.ts"
    {
        (
            "git-status target keys, empty/initial snapshots, shared watch refcounts, per-environment state isolation, pending transitions, refresh in-flight/debounce decisions, and reset behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core git_status_state_machine_matches_upstream_contract`",
            "Wire state machine into live GPUI atom/runtime subscriptions, websocket VCS status stream, environment connection listeners, and async refresh RPCs.",
        )
    } else if path == "apps/web/src/lib/providerReactQuery.ts"
        || path == "apps/web/src/lib/providerReactQuery.test.ts"
    {
        (
            "provider checkpoint-diff query keys, full-thread vs turn-diff request decode, enabled/stale metadata, error normalization, and retry/backoff helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core provider_react_query_checkpoint_diff_matches_upstream_contract`",
            "Wire live environment API query execution and query-client retry behavior into GPUI diff panel state.",
        )
    } else if path == "apps/web/src/lib/projectReactQuery.ts" {
        (
            "project search-entries query keys, default limit/stale metadata, enabled/request-availability split, and empty placeholder contract in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core project_react_query_search_entries_matches_upstream_contract`",
            "Wire live environment API project search execution and previous-data placeholder behavior into GPUI workspace entry search.",
        )
    } else if path == "apps/web/src/lib/projectScriptKeybindings.ts"
        || path == "apps/web/src/lib/projectScriptKeybindings.test.ts"
    {
        (
            "project script keybinding trim/null decode, invalid-value error, script command validation, and latest matching binding value lookup in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core project_script_keybindings_match_upstream_contract`",
            "Wire helpers into live GPUI project script settings keybinding editor and persistence flow.",
        )
    } else if path == "apps/web/src/lib/storage.ts" {
        (
            "memory state storage get/set/remove, storage shape fallback resolution, debounced set/flush/remove semantics, and default debounce wait in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core state_storage_helpers_match_upstream_contract`",
            "Wire debounced storage adapter into live GPUI persistence surfaces that replace browser storage.",
        )
    } else if path == "apps/web/src/lib/terminalContext.ts"
        || path == "apps/web/src/lib/terminalContext.test.ts"
    {
        (
            "terminal context text normalization, labels, numbered context blocks, prompt append/extract/display state, inline placeholder insertion/removal/materialization, expiry filtering, and preview titles in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core terminal_context`; `cargo test -p r3_core inline_terminal_context`",
            "Wire terminal context helpers into live GPUI terminal snapshot capture, composer chips, and displayed user-message rendering.",
        )
    } else if path == "apps/web/src/lib/turnDiffTree.ts"
        || path == "apps/web/src/lib/turnDiffTree.test.ts"
    {
        (
            "turn-diff stat summing, path separator normalization, numeric/base-name sorting, directory stat aggregation, single-directory-chain compaction, missing-stat preservation, and whitespace-preserving path segments in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core turn_diff_tree`; `cargo test -p r3_core turn_diff_stats`",
            "Wire helper output into live GPUI diff panel expansion, selection, and exact row rendering for all upstream tree states.",
        )
    } else if path == "apps/web/src/lib/threadSort.ts"
        || path == "apps/web/src/lib/threadSort.test.ts"
    {
        (
            "thread-sort timestamp parsing, latest-user-message fallback, recency/created ordering, descending id tiebreak, and latest active project-thread selection in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core thread_sort_helpers_match_upstream_contract`",
            "Wire input-level thread sort helpers into GPUI sidebar/project selection paths beyond summary-only sorting.",
        )
    } else if path == "apps/web/src/lib/sourceControlDiscoveryState.ts" {
        (
            "web source-control discovery primary-target selection, stale/idle auto-refresh metadata, atom label, and primary/local/remote client fallback planning in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core web_source_control_discovery_state_wrapper_matches_upstream_contract`",
            "Wire plans into live GPUI environment registry, local API fallback, atom/SWR refresh, and source-control UI state.",
        )
    } else if path.starts_with("apps/web/src/logicalProject") {
        (
            "logical project path normalization, physical/grouping/order keys, repository-scoped keys, settings override resolution, ref fallback, and group-label helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core logical_project`",
            "Wire helpers into live GPUI sidebar grouping, persisted settings, drag ordering, and project state updates.",
        )
    } else if matches!(
        path,
        "apps/web/src/store.ts" | "apps/web/src/store.test.ts" | "apps/web/src/storeSelectors.ts"
    ) {
        (
            "web store state ownership/cap constants, thread derivation, projection-backed environment state, scoped thread/project refs, and selector lookup contracts in crates/r3_core/src/lib.rs plus projection store behavior in crates/r3_core/src/persistence.rs",
            "partial",
            "`cargo test -p r3_core web_store`; `cargo test --workspace` projection store tests",
            "Wire the native GPUI store runtime to the same environment-scoped mutation API, selector memoization boundary, shell/detail stream ownership split, and event replay semantics.",
        )
    } else if path == "apps/web/src/components/WebSocketConnectionSurface.logic.test.ts" {
        (
            "WebSocket connection surface auto-reconnect predicates for online/focus triggers, exhausted reconnect loops, stalled retry restart detection, and upstream websocket state-machine integration in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace websocket_connection_state_matches_upstream_reconnect_logic`",
            "Wire the predicates into live GPUI window online/offline/focus handlers, toast actions, debounce timing, and primary connection reconnect side effects.",
        )
    } else if path == "apps/web/src/components/WebSocketConnectionSurface.tsx" {
        (
            "WebSocket connection surface UI-state derivation, reconnect/backoff status, auto-reconnect predicates, slow RPC ack tracking, toast copy/title timing helpers, and child passthrough surface in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace websocket_connection_state_matches_upstream_reconnect_logic`; `cargo test --workspace request_latency_state_matches_upstream_ack_tracking`",
            "Wire live GPUI toast lifecycle, browser online/focus listeners, debounce timers, manual reconnect action, and slow request expandable details.",
        )
    } else if path == "apps/web/src/components/ChatView.browser.tsx"
        || path == "apps/web/src/components/ChatView.tsx"
    {
        (
            "ChatView no-active-thread fallback, root/header/input/timeline wrapper class contracts, scroll-to-bottom pill, banner mounting, branch toolbar gate, plan sidebar placement, mounted terminal drawer gate, and expanded image dialog gate in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace chat_view_render_contract_matches_upstream_shell`; `cargo test --workspace composer_send_state_and_expired_terminal_copy_match_upstream`",
            "Wire the full live GPUI ChatView render tree, browser layout assertions, local dispatch lifecycle, scroll virtualization, route canonicalization, and every interaction covered by the upstream browser test.",
        )
    } else if path == "apps/web/src/components/ChatView.logic.test.ts"
        || path == "apps/web/src/components/ChatView.logic.ts"
    {
        (
            "ChatView composer send-state trimming for expired terminal pills, expired-context toast copy, send env-mode resolution, mounted terminal thread reconciliation, local-dispatch acknowledgement, and current-thread error routing in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace composer_send_state_and_expired_terminal_copy_match_upstream`; `cargo test --workspace terminal_split_and_new_group_behaviors_match_upstream_store`; `cargo test --workspace chat_thread_action_plans_match_upstream_context_resolution`",
            "Wire remaining async wait-for-started-thread behavior, live store subscription race handling, local dispatch lifecycle, and exact GPUI ChatView rendering.",
        )
    } else if path == "apps/web/src/components/BranchToolbar.logic.test.ts"
        || path == "apps/web/src/components/BranchToolbar.logic.ts"
    {
        (
            "BranchToolbar draft env-mode transitions, toolbar values, workspace labels, branch selection targets, remote/local ref dedupe, and picker filtering in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace branch_toolbar_env_mode_and_value_match_upstream_logic`; `cargo test --workspace branch_toolbar_labels_match_upstream_logic`",
            "Wire the remaining branch popover rendering, live checkout actions, saved draft mutation, and exact GPUI branch toolbar layout.",
        )
    } else if path == "apps/web/src/components/BranchToolbar.tsx" {
        (
            "BranchToolbar top-level render gate, root/desktop/mobile class contracts, environment picker visibility, env-mode lock rule, mobile icon/label decisions, and menu group labels in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace branch_toolbar_render_contract_matches_upstream_component`; `cargo test --workspace branch_toolbar_env_mode_and_value_match_upstream_logic`; `cargo test --workspace branch_toolbar_labels_match_upstream_logic`",
            "Wire exact GPUI desktop/mobile branch toolbar rendering plus live selector callbacks and responsive media state.",
        )
    } else if path == "apps/web/src/components/BranchToolbarBranchSelector.tsx" {
        (
            "BranchToolbarBranchSelector combobox classes, trigger labels, search/empty/status copy, virtualization/fetch thresholds, and ref badge priority in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace branch_toolbar_branch_selector_render_contract_matches_upstream_component`; `cargo test --workspace branch_picker_helpers_match_upstream_filtering`; `cargo test --workspace branch_selection_target_matches_upstream_worktree_rules`",
            "Wire remaining live git ref queries, optimistic branch mutation, draft/server branch persistence, PR checkout item actions, query invalidation, and exact combobox rendering.",
        )
    } else if path == "apps/web/src/components/BranchToolbarEnvModeSelector.tsx"
        || path == "apps/web/src/components/BranchToolbarEnvironmentSelector.tsx"
    {
        (
            "BranchToolbar env-mode/environment selector locked spans, select trigger classes and aria labels, group labels, active icon/label fallbacks, and item contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace branch_toolbar_selectors_match_upstream_component_contracts`; `cargo test --workspace branch_toolbar_labels_match_upstream_logic`",
            "Wire exact GPUI select primitives, popup positioning, selection callbacks, and desktop toolbar integration.",
        )
    } else if path == "apps/web/src/components/CommandPalette.logic.test.ts"
        || path == "apps/web/src/components/CommandPalette.logic.ts"
    {
        (
            "CommandPalette recent-thread sorting, title/context search ranking, archived filtering, action-only queries, project/thread injection, and store transitions in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace command_palette_builds_recent_threads_with_upstream_sort_and_timestamp_rules`; `cargo test --workspace command_palette_search_ranks_titles_over_context_and_filters_archived_threads`; `cargo test --workspace command_palette_filters_action_only_queries_and_injects_projects_and_threads`; `cargo test --workspace command_palette_store_transitions_match_upstream_zustand_logic`",
            "Wire live GPUI command palette keyboard handling, async data refresh, project creation flow, and exact list rendering.",
        )
    } else if path == "apps/web/src/components/CommandPalette.tsx"
        || path == "apps/web/src/components/CommandPaletteResults.tsx"
    {
        (
            "CommandPalette popup/input/panel/footer shell contracts, add-project repository/confirm states, file-manager action, results empty states, row classes, disabled/active row behavior, timestamp, shortcut, and submenu-chevron rendering in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace command_palette_render_contracts_match_upstream_components`; `cargo test --workspace command_palette_filters_action_only_queries_and_injects_projects_and_threads`",
            "Wire exact GPUI command primitives, browser keyboard/focus behavior, add-project browse/clone flows, source-control lookup, and live row rendering parity.",
        )
    } else if path == "apps/web/src/components/ComposerPromptEditor.tsx" {
        (
            "ComposerPromptEditor Lexical namespace, node type/version/text-content contracts, inline-token DOM and tooltip classes, content-editable and placeholder classes, plugin order, terminal-context placeholder behavior, and surround-symbol table in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace composer_prompt_editor_render_contract_matches_upstream_component`; `cargo test --workspace composer_inline_token_adjacency_matches_upstream_contract`",
            "Wire full GPUI editor input behavior, controlled cursor/selection updates, paste handling, inline token editing, keyboard command interception, and browser composition/dead-key behavior.",
        )
    } else if path == "apps/web/src/components/DiffWorkerPoolProvider.tsx" {
        (
            "DiffWorkerPoolProvider worker import, provider/hook contract, navigator hardware-concurrency pool sizing, AST cache size, tokenizer limit, theme resolution, and theme-sync merge/skip/catch behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace diff_worker_pool_provider_contract_matches_upstream_component`; `cargo test --workspace diff_panel_render_contract_matches_upstream_components`",
            "Wire actual GPUI/background diff worker execution, async theme option updates, worker lifecycle disposal, and rendered diff cache integration.",
        )
    } else if path == "apps/web/src/components/DiffPanel.tsx"
        || path == "apps/web/src/components/DiffPanelShell.tsx"
    {
        (
            "DiffPanel and DiffPanelShell mode/root/header classes, drag-region rules, turn-strip controls, toggle labels, empty/loading/raw/file render states, virtualizer settings, collapse buttons, diff style/overflow/theme, and center-state copy in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace diff_panel_render_contract_matches_upstream_components`; `cargo test --workspace diff_route_search_matches_upstream_parser_contract`",
            "Wire exact GPUI diff rendering, turn strip scrolling, file collapse state, checkpoint diff query lifecycle, raw patch fallback, editor-open actions, and browser visual parity.",
        )
    } else if path == "apps/web/src/components/GitActionsControl.browser.tsx"
        || path == "apps/web/src/components/GitActionsControl.tsx"
    {
        (
            "GitActionsControl render/action shell plus browser contracts for thread-scoped progress toast data, 250ms focus/visibility refresh debounce, live server/draft branch sync, worktree-base guard, and menu/action classes in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace git_actions_browser_contracts_match_upstream_component`; `cargo test --workspace git_action_menu_items_match_upstream_clean_main_status`; current screenshot gates where captured",
            "Wire full commit/push/PR dialogs, publish repository wizard, disabled-reason tooltips, progress event subscription, source-control refresh, editor-open actions, and live git mutations.",
        )
    } else if path == "apps/web/src/components/GitActionsControl.logic.test.ts"
        || path == "apps/web/src/components/GitActionsControl.logic.ts"
    {
        (
            "GitActionsControl menu enablement, quick-action derivation, progress-stage sequencing, default-ref confirmation dialog copy, live branch update rules, and temporary worktree branch guards in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace git_action_menu_items_match_upstream_clean_main_status`; `cargo test --workspace git_action_logic_matches_upstream_quick_actions_and_dialogs`; `cargo test --workspace git_actions_browser_contracts_match_upstream_component`",
            "Port remaining provider-specific menu labels, disabled-reason tooltips, publish readiness helpers, full runStackedAction result handling, progress event subscription, and live GPUI action execution.",
        )
    } else if path == "apps/web/src/components/Icons.tsx" {
        (
            "provider/editor/model icon export list, SVG viewBox contracts, generated gradient/mask/clip/filter IDs, theme fill classes, Antigravity data-url image usage, native asset coverage map, and live editor/provider icon call sites in crates/r3_core/src/lib.rs, crates/r3_ui/src/shell.rs, and crates/r3_ui/assets/icons",
            "covered",
            "`cargo test --workspace upstream_icon_contracts_match_web_components`",
            "Maintain asset parity when upstream adds or changes shared web icon exports.",
        )
    } else if path == "apps/web/src/components/JetBrainsIcons.tsx" {
        (
            "JetBrains icon export list, shared useSvgGradientIds prefix/count contract, 64x64 viewBox, black tile path/fill, white glyph fill, native SVG asset paths, and editor icon call sites in crates/r3_core/src/lib.rs, crates/r3_ui/src/shell.rs, and crates/r3_ui/assets/icons",
            "covered",
            "`cargo test --workspace upstream_jetbrains_icon_contracts_match_shared_helper`",
            "Maintain asset parity when upstream adds or changes JetBrains product SVG exports.",
        )
    } else if path == "apps/web/src/components/Sidebar.logic.test.ts"
        || path == "apps/web/src/components/Sidebar.logic.ts"
    {
        (
            "Sidebar project grouping modes, labels, repository grouping, logical project helpers, and sidebar environment grouping selectors in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace sidebar_project_grouping_matches_upstream_contract`; `cargo test --workspace logical_project_helpers_match_upstream_grouping_rules`; `cargo test --workspace environment_grouping_selectors_match_upstream_contract`",
            "Wire remaining live sidebar rendering, drag/reorder persistence, context menus, thread previews, and exact GPUI navigation states.",
        )
    } else if path == "apps/web/src/components/ThreadTerminalDrawer.test.ts"
        || path == "apps/web/src/components/ThreadTerminalDrawer.tsx"
    {
        (
            "ThreadTerminalDrawer terminal selection action positioning, multi-click delay, mouseup gate, snapshot replay filtering, and pending-event replay filtering in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace thread_terminal_drawer_selection_actions_match_upstream_contract`; `cargo test --workspace terminal_activity_and_event_filters_match_upstream_helpers`",
            "Wire remaining live xterm selection capture, helper textarea behavior, terminal drawer rendering, and exact GPUI action chip placement.",
        )
    } else if path == "apps/web/src/components/chat/ComposerPrimaryActions.test.ts"
        || path == "apps/web/src/components/chat/ComposerPrimaryActions.tsx"
    {
        (
            "ComposerPrimaryActions pending user-input primary button labels for responding, compact submit/next, non-compact next-question, singular submit-answer, and plural submit-answers states in crates/r3_core/src/lib.rs plus GPUI shell usage in crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace composer_primary_action_labels_match_upstream_contract`",
            "Wire the remaining primary action UI states, pointer-focus preservation, stop/refine/implement controls, and exact GPUI button/icon rendering.",
        )
    } else if path == "apps/web/src/components/chat/ChatHeader.test.ts"
        || path == "apps/web/src/components/chat/ChatHeader.tsx"
    {
        (
            "ChatHeader open-in picker visibility for primary-environment projects plus editor option filtering in crates/r3_core/src/lib.rs and GPUI header usage in crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace open_in_picker_visibility_and_options_match_upstream_logic`",
            "Wire remaining header action layout, project script controls, git actions, terminal/diff tooltips, and exact GPUI responsive behavior.",
        )
    } else if path == "apps/web/src/components/chat/ChangedFilesTree.test.tsx"
        || path == "apps/web/src/components/chat/ChangedFilesTree.tsx"
    {
        (
            "ChangedFilesTree compacted directory labels plus collapsed/expanded initial visibility for single-chain, branch-point, and mixed root/nested paths in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace changed_files_tree_initial_labels_match_upstream_expansion_contract`; `cargo test --workspace turn_diff_tree_compacts_directory_chains_and_sorts_numerically`",
            "Wire directory toggle state, per-directory overrides, diff-open callbacks, file icons, and exact GPUI tree rendering.",
        )
    } else if path == "apps/web/src/components/chat/composerProviderState.test.tsx"
        || path == "apps/web/src/components/chat/composerProviderState.tsx"
    {
        (
            "composerProviderState descriptor defaulting, selection overrides, stale selection dropping, dispatch option rebuilding, ultrathink class activation, and no-target traits render guard in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace composer_provider_state_matches_upstream_descriptor_contracts`",
            "Wire traits picker/menu controls into live GPUI composer targets, prompt mutation, draft/thread routing, and exact traits UI rendering.",
        )
    } else if path == "apps/web/src/components/chat/ComposerPendingTerminalContexts.test.tsx"
        || path == "apps/web/src/components/chat/ComposerPendingTerminalContexts.tsx"
    {
        (
            "Composer pending terminal-context expiry detection, preview title, range labels, inline labels, and expired-send filtering in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace terminal_context_expiry_and_preview_match_upstream_contract`; `cargo test --workspace composer_send_state_and_expired_terminal_copy_match_upstream`",
            "Wire exact GPUI chip data attributes, expired styling, removal interactions, and composer chip layout.",
        )
    } else if path == "apps/web/src/components/chat/composerSlashCommandSearch.test.ts"
        || path == "apps/web/src/components/chat/composerSlashCommandSearch.ts"
    {
        (
            "composer slash-command search exact provider command priority, description matching, boundary-aware fuzzy matching, and replacement contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace composer_menu_item_derivation_matches_upstream_contract`; `cargo test --workspace composer_slash_command_and_replacement_match_upstream_contract`",
            "Wire remaining live composer command menu rendering, provider slash command execution, and exact keyboard navigation.",
        )
    } else if path == "apps/web/src/components/chat/modelPickerSearch.test.ts"
        || path == "apps/web/src/components/chat/modelPickerSearch.ts"
    {
        (
            "model picker search text, score ordering, trigger filtering, selection locking, jump hints, and selected-instance state in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace model_picker_search_sorting_and_selection_match_upstream_logic`; `cargo test --workspace model_picker_trigger_filtering_and_locking_match_upstream_logic`",
            "Wire remaining live GPUI picker search input, provider rail, focus management, and exact popup rendering.",
        )
    } else if path == "apps/web/src/components/chat/userMessageTerminalContexts.test.ts"
        || path == "apps/web/src/components/chat/userMessageTerminalContexts.ts"
    {
        (
            "user message inline terminal-context label formatting, joined plain-text labels, and embedded label detection in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace terminal_context_formatting_matches_upstream_contract`",
            "Wire exact GPUI user-message terminal context rendering and rich text spans.",
        )
    } else if path == "apps/web/src/components/settings/KeybindingsSettings.logic.test.ts"
        || path == "apps/web/src/components/settings/KeybindingsSettings.logic.ts"
    {
        (
            "KeybindingsSettings searchable rows, keyboard event capture, shortcut serialization, when-expression parsing/printing, command labels, variable/command options, unknown variable reporting, default-source marking, conflict labels, and editor draft helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace keybinding_rows_options_and_conflicts_match_upstream_logic`; `cargo test --workspace keybinding_default_rows_match_upstream_settings_projection`; `cargo test --workspace keybinding_editor_draft_helpers_match_upstream_panel_state`",
            "Wire live GPUI keybinding editor controls, add/remove persistence, conflict display, and exact settings panel rendering.",
        )
    } else if path == "apps/web/src/components/settings/SettingsPanels.logic.test.ts"
        || path == "apps/web/src/components/settings/SettingsPanels.logic.ts"
    {
        (
            "SettingsPanels diagnostics description collapse and provider-instance update patch defaults/custom-instance behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace build_provider_instance_update_patch_matches_upstream_settings_panels_logic`; `cargo test --workspace formats_diagnostics_settings_description_like_upstream`",
            "Wire settings patches into live GPUI provider settings mutations, server persistence, and exact settings panel routing/rendering.",
        )
    } else if path == "apps/web/src/components/settings/ProviderSettingsForm.test.ts"
        || path == "apps/web/src/components/settings/ProviderSettingsForm.tsx"
    {
        (
            "ProviderSettingsForm visible field derivation, schema annotation metadata, config string/boolean reads, and empty/default field omission behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace provider_settings_form_helpers_match_upstream_contract`",
            "Wire derived fields into the live GPUI provider settings form, card/dialog variants, commit-on-blur controls, and exact input/switch styling.",
        )
    } else if path == "apps/web/src/components/settings/ProviderInstanceCard.test.ts"
        || path == "apps/web/src/components/settings/ProviderInstanceCard.tsx"
    {
        (
            "ProviderInstanceCard model display derivation that keeps live server models while replacing stale live custom rows with current config custom models in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace provider_models_for_display_match_upstream_instance_card_contract`",
            "Wire the remaining provider instance card controls, environment variable drafts, accent swatches, auth copy, update actions, and exact GPUI card rendering.",
        )
    } else if path == "apps/web/src/components/ui/sidebar.test.tsx"
        || path == "apps/web/src/components/ui/sidebar.tsx"
    {
        (
            "Sidebar UI menu button/action/sub-button data slots and pointer cursor class merge behavior in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace sidebar_interactive_cursor_classes_match_upstream_contract`",
            "Wire any remaining sidebar primitive sizing, tooltip, hover, rail, input, skeleton, badge, and exact GPUI style states.",
        )
    } else if path == "apps/web/src/components/NoActiveThreadState.tsx" {
        (
            "NoActiveThreadState header/electron/web classes, title copy, empty/card/header/title/description classes, and live GPUI empty-state styling in crates/r3_core/src/lib.rs and crates/r3_ui/src/shell.rs",
            "covered",
            "`cargo test --workspace no_active_thread_state_contract_matches_upstream_component`; current screenshot gates where captured",
            "Maintain this contract if upstream changes the no-active-thread empty state.",
        )
    } else if path.starts_with("apps/web/src/components/")
        || path.starts_with("apps/web/src/composer")
        || path.starts_with("apps/web/src/diff")
        || path.starts_with("apps/web/src/editor")
        || path.starts_with("apps/web/src/filePath")
        || path.starts_with("apps/web/src/history")
        || path.starts_with("apps/web/src/session-logic")
        || path.starts_with("apps/web/src/terminal")
        || path.starts_with("apps/web/src/uiState")
        || path.starts_with("apps/web/src/vscode-icons")
    {
        (
            "crates/r3_core/src/lib.rs; crates/r3_ui/src/shell.rs",
            "partial",
            "`cargo test --workspace`; current screenshot gates where captured",
            "Replace seeded/static state with live GPUI state and port remaining component behavior.",
        )
    } else if path == "apps/web/src/environments/runtime/connection.ts" {
        (
            "web environment connection bootstrap gate, lifecycle/config subscription rules, identity guard, reconnect/reset, terminal-event scoping, and dispose cleanup contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace web_environment_connection_runtime_contract`",
            "Wire the runtime connection service to live GPUI/WebSocket clients, actual subscriptions, metadata refresh, and saved-environment lifecycle side effects.",
        )
    } else if path == "apps/web/src/environments/runtime/catalog.ts" {
        (
            "saved-environment registry persistence, store update, sorted listing, HTTP base/URL resolution, bearer-token secret preservation, runtime state ensure/patch/clear defaults, and reset contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace saved_environment_registry`",
            "Wire saved-environment registry/runtime state to live GPUI stores, local persistence IO, auth token lifecycle, and runtime connection side effects.",
        )
    } else if path == "apps/web/src/environments/runtime/service.addSavedEnvironment.test.ts" {
        (
            "addSavedEnvironment saved-record label/timestamp/desktop-SSH fallback, credential persistence rollback/restore, stale desktop-SSH record removal, SSH 401 bearer reissue, pending cancellation, disconnect/remove credential behavior, desktop bridge bootstrap inputs, pairing-token failure rollback, runtime error patching, and metadata rename contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_runtime_service_saved_connection_flows_match_upstream_contract`; `cargo test --workspace environment_runtime_service_record_metadata_flows_match_upstream_contract`; `cargo test --workspace client_persistence_storage_preserves_saved_environment_secrets`",
            "Wire these pure plans into live GPUI/WebSocket connection creation, desktop bridge calls, persisted token IO, pending-promise cancellation, and metadata refresh side effects.",
        )
    } else if path == "apps/web/src/environments/runtime/service.savedEnvironments.test.ts" {
        (
            "saved environment startup service ref-count lifecycle, hydration/saved-registry sync coalescing, initial config snapshot preference, saved credential loading, remote session validation, and saved connection start planning in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_runtime_service_lifecycle_matches_upstream_contract`; `cargo test --workspace environment_runtime_service_saved_connection_flows_match_upstream_contract`",
            "Wire hydration promises, registry subscriptions, initial config snapshots, WsRpcClient creation, and saved-environment startup into the live GPUI runtime service.",
        )
    } else if path == "apps/web/src/environments/runtime/service.test.ts" {
        (
            "terminal-event draft/server-thread archive filtering plus projection snapshot/event sequence and updatedAt gating contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_runtime_service_helpers_match_upstream_contract`; `cargo test --workspace terminal_activity_and_event_filters_match_upstream_helpers`",
            "Wire these gates into live environment connection event application and GPUI terminal/projection stores.",
        )
    } else if path == "apps/web/src/environments/runtime/service.threadSubscriptions.test.ts" {
        (
            "thread-detail subscription retain/release ref-counting, idle warm-cache eviction delay/capacity, non-idle retention, reset/disconnect disposal, browser-resume reconnect cooldown, primary environment-id start gate, and saved reconnect reattachment planning in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_runtime_service_helpers_match_upstream_contract`; `cargo test --workspace environment_runtime_service_lifecycle_matches_upstream_contract`; `cargo test --workspace environment_runtime_service_record_metadata_flows_match_upstream_contract`",
            "Wire retained thread subscriptions to actual live subscribeThread handles across saved reconnect replacement, service reset, and environment removal.",
        )
    } else if path == "apps/web/src/environments/runtime/service.ts" {
        (
            "environment runtime service constants, projection snapshot/event version gating, thread-detail subscription key/ref-count/idle-capacity rules, SSH HTTP status parsing, saved runtime state transition patches, saved-environment sync scheduler queueing, service start/release ref-count lifecycle, browser resume reconnect cooldown, saved connection missing-credential/auth-recovery/disconnect/remove/sync plans, addSavedEnvironment record fallback/stale-removal rules, desktop SSH prepared-record/bootstrap error contracts, metadata refresh patch/rename behavior, register/remove connection plans, and selected pairing URL helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_runtime_service`",
            "Wire the runtime service to live GPUI/WebSocket clients, query invalidation throttling, saved-environment connection sync, browser resume reconnects, and actual thread-detail subscription handles.",
        )
    } else if matches!(
        path,
        "apps/web/src/environments/remote/api.ts"
            | "apps/web/src/environments/remote/api.test.ts"
            | "apps/web/src/environments/remote/target.ts"
    ) {
        (
            "remote environment pairing target normalization, hosted/manual pairing target resolution, auth endpoint URL/method/header/body request plans, auth error message fallback parsing, fetch failure/status messages, and websocket token URL shaping in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace remote_environment_target_and_api_helpers_match_upstream_contract`",
            "Wire remote API helpers into live HTTP fetch, browser URL resolution, credential bootstrap, and websocket token issuance in GPUI runtime flows.",
        )
    } else if matches!(
        path,
        "apps/web/src/environments/primary/auth.ts"
            | "apps/web/src/environments/primary/bootstrap.test.ts"
            | "apps/web/src/environments/primary/target.ts"
    ) {
        (
            "primary environment target resolution, loopback/dev-server proxy rules, desktop/configured/window-origin base URL derivation, primary HTTP URL building, auth bootstrap timing/retry constants, transient bootstrap classification, friendly bootstrap error messages, and auth gate decisions in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace primary_environment_target_and_auth_helpers_match_upstream_contract`",
            "Wire primary target/auth helpers into live desktop bridge bootstrap, browser history URL mutation, fetch credentials, in-flight auth bootstrap promise caching, and GPUI auth gate state.",
        )
    } else if path == "apps/web/src/environmentApi.ts" {
        (
            "environment API facade method surface, readEnvironmentApi window/environment-id/override/connection decision order, and ensureEnvironmentApi error message in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_api_facade_matches_upstream_contract`",
            "Wire the facade into live WsRpcClient instances, runtime connection lookup, and test override storage in GPUI runtime.",
        )
    } else if path == "apps/web/src/environmentGrouping.test.ts" {
        (
            "environment grouping selectors for all projects/sidebar threads across environments, single/multiple project-ref thread lookup, null/missing environment handling, logical repository grouping, repository-path/separate grouping, settings overrides, and sidebar project group snapshots in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace environment_grouping_selectors_match_upstream_contract`; `cargo test --workspace logical_project_helpers_match_upstream_grouping_rules`; `cargo test --workspace sidebar_project_grouping_matches_upstream_contract`",
            "Wire pure grouping selectors into live GPUI store selectors and memoization boundaries.",
        )
    } else if path == "apps/web/src/authBootstrap.test.ts" {
        (
            "primary auth bootstrap request URL/method/body/credentials plans, pairing token submission validation, pairing link/client list/revoke endpoint shapes, revoke-others POST behavior, transient retry/gate helpers, and authenticated-session wait timeout constants in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace primary_auth_endpoint_request_helpers_match_upstream_contract`; `cargo test --workspace primary_environment_target_and_auth_helpers_match_upstream_contract`",
            "Wire these request contracts into live fetch, browser history mutation, promise memoization, retry timers, and authenticated gate caching.",
        )
    } else if path.starts_with("apps/web/src/environments/")
        || path.starts_with("apps/web/src/environment")
        || path.starts_with("apps/web/src/auth")
    {
        (
            "selected helpers plus pairing URL get/strip/set contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` for selected pairing/environment helpers",
            "Port runtime connection service, auth bootstrap, subscriptions, and saved environments.",
        )
    } else if path == "apps/web/src/routeTree.gen.ts" {
        (
            "generated TanStack route tree IDs, paths, full paths, parent IDs, root/chat/settings child ordering, and FileRouteTypes fullPath/to/id unions in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core generated_route_tree_matches_upstream_tanstack_output`; `cargo test -p r3_core thread_routes_helpers_match_upstream_contract`",
            "Wire generated route graph into live GPUI routing, history/deep-link handling, and route-driven screen selection.",
        )
    } else if path.starts_with("apps/web/src/routes/")
        || path.starts_with("apps/web/src/main")
        || path.starts_with("apps/web/src/router")
    {
        (
            "screen selection and route structs in r3_core/r3_ui",
            "partial",
            "`cargo test --workspace` for selected route parsers",
            "Port real routing, history, deep links, and route-driven state.",
        )
    } else if matches!(
        path,
        "apps/web/package.json"
            | "apps/web/tsconfig.json"
            | "apps/web/components.json"
            | "apps/web/index.html"
            | "apps/web/vite.config.ts"
            | "apps/web/vitest.browser.config.ts"
            | "apps/web/vercel.ts"
    ) {
        (
            "web package metadata, dependency/version maps, tsconfig/plugin settings, components registry, boot HTML shell, Vite/Vitest build-test config, and Vercel channel routing contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Wire the Rust/GPUI app packaging, hosted web deployment, brand asset application, and browser-test launch path to these contracts instead of the upstream Vite/Vercel toolchain.",
        )
    } else if matches!(
        path,
        "apps/web/src/index.css"
            | "apps/web/src/vite-env.d.ts"
            | "apps/web/public/mockServiceWorker.js"
            | "apps/web/test/authHttpHandlers.ts"
            | "apps/web/test/wsRpcHarness.ts"
    ) {
        (
            "web Tailwind/global CSS tokens and selectors, Vite env/window declarations, MSW worker metadata, auth HTTP test handlers, and browser WebSocket RPC harness contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Wire native GPUI styling/test harness equivalents to these exact global theme hooks, browser env assumptions, mock-auth routes, MSW lifecycle semantics, and RPC harness stream/unary behavior.",
        )
    } else if path.starts_with("apps/web/") {
        (
            "crates/r3_ui/src/shell.rs",
            "partial",
            "Current visual gates only cover selected states",
            "Classify exact web app behavior and add missing GPUI/screenshots.",
        )
    } else if path.starts_with("apps/server/src/diagnostics/") {
        (
            "diagnostics helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` diagnostics tests",
            "Wire live process and trace diagnostics.",
        )
    } else if path.starts_with("apps/server/src/sourceControl/") {
        (
            "source-control presentation plus provider helper contracts, change-request schemas/provider normalizers, GitHub/GitLab/Azure command plans, GitHub non-open PR list stdout/error decisions, GitHub/GitLab/Azure CLI error normalization contracts, Bitbucket API request/error/checkout plans, Bitbucket default target-branch decisions, Bitbucket API discovery/auth contracts, repository lookup/clone/publish decision contracts, clone destination inspection decisions, publish ensure-remote/push planning, repository error mapping/detail fallback, provider registry binding/unsupported decisions, owner/ref source branch parsing, context fallback, provider error messages, discovery helper contracts, safe auth-line filtering, auth trimming, detail-from-cause fallback, strict CLI host parsing, CLI/VCS probe command plans, VCS probe item mapping, provider CLI discovery specs, GitHub/GitLab/Azure auth parsers, and provider context remote-selection/cache constants in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` source-control tests",
            "Port live provider discovery/cache execution, context-bound provider wrappers, Bitbucket API auth execution, PR/MR workflows, and mutations.",
        )
    } else if path.starts_with("apps/server/src/git/") || path.starts_with("apps/server/src/vcs/") {
        (
            "branch/git presentation helpers, VCS contract shapes, Git workspace-file/remotes/check-ignore and checkpoint command-plan contracts, repository detection cache keys, project config kind resolution, and driver registry fallback/error contracts in crates/r3_core/src/lib.rs plus VCS process default/error/truncation contracts in crates/r3_core/src/process.rs",
            "partial",
            "`cargo test -p r3_core vcs`; `cargo test -p r3_core process`; branch/git menu tests",
            "Port live git status, refs, worktrees, commits, push, checkout, workspace file execution, and VCS process service wiring.",
        )
    } else if path.starts_with("apps/server/src/terminal/") {
        (
            "terminal state contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` terminal state tests",
            "Port PTY backend, history persistence, process activity, resize/write/kill/restart.",
        )
    } else if path.starts_with("apps/server/src/checkpointing/") {
        (
            "diff summary/tree helpers in crates/r3_core/src/lib.rs plus checkpoint diff query planning, canonical checkpoint refs, narrow full-thread context, and error/default contracts in crates/r3_core/src/orchestration.rs",
            "partial",
            "`cargo test -p r3_core orchestration`; `cargo test --workspace` diff tests",
            "Port live checkpoint store execution, diff blob persistence, and generated patch retrieval.",
        )
    } else if path.starts_with("apps/server/src/workspace/")
        || path == "packages/contracts/src/project.ts"
        || path == "packages/contracts/src/filesystem.ts"
    {
        (
            "workspace path, search, browse, and write-file contracts in crates/r3_core/src/workspace.rs",
            "partial",
            "`cargo test -p r3_core workspace`",
            "Port VCS-backed workspace indexing, cache invalidation, live RPC wiring, project registry add/list/remove, and full filesystem edge cases.",
        )
    } else if path.starts_with("apps/server/src/project/") {
        (
            "project summary/script helpers, setup script runner terminal/env decisions, favicon resolver candidate/source/href/path-boundary contracts, and repository identity resolver command plans, cache policy, primary remote selection, and identity derivation in crates/r3_core/src/lib.rs plus workspace file contracts in crates/r3_core/src/workspace.rs",
            "partial",
            "`cargo test -p r3_core project_favicon`; `cargo test -p r3_core project_setup`; `cargo test --workspace` project script tests, `cargo test -p r3_core vcs`, and `cargo test -p r3_core workspace`",
            "Port live repository identity cache execution, live filesystem-backed favicon resolver, terminal-backed setup runner execution, project registry add/list/remove, and live workspace discovery.",
        )
    } else if path.starts_with("apps/server/src/auth/") {
        (
            "auth descriptor, HTTP/websocket credential selection with URLSearchParams-compatible websocket token query decoding, owner access-control/error mapping, HTTP route plans, auth HTTP error/success/session-state response and CORS/cookie contracts, AuthControlPlane CLI session defaults plus pairing/session listing rules, bootstrap credential issue/consume decision contracts, pairing-token request body detection, secret-store filesystem contracts plus filesystem get/set/get-or-create/remove helpers, cookie, client metadata, pairing-token, HMAC-signed session/websocket token issue/verify helpers with default TTLs and claim decode/expiry checks, persisted token-to-session repository verification decisions, verified-session credential assembly, session-claim, access-stream change fan-in/current-session marking, session-credential change, connected-session count, auth session/pairing-link persistence, and pairing helpers in crates/r3_core/src/auth.rs plus crates/r3_core/src/persistence.rs",
            "partial",
            "`cargo test -p r3_core auth`; `cargo test -p r3_core persistence`; `cargo test --workspace` pairing/auth helper tests",
            "Wire full live secret-store service permissions/concurrency layer, live repository-backed auth service execution, concrete HTTP auth exchange execution, atomic live bootstrap consume/emit behavior, websocket upgrade execution, live auth PubSub streams, and persisted runtime integration.",
        )
    } else if matches!(
        path,
        "apps/server/src/provider/Services/ProviderService.ts"
            | "apps/server/src/provider/Layers/ProviderService.ts"
    ) {
        (
            "provider service request/input/result contracts plus start-session resolution, start-session execution/completion planning, empty send-turn planning, routable-session recovery planning, session/send/stop binding payload planning, listSessions merge/mismatch rules, rollback no-op/recovery planning, stale-session stop planning, capabilities/instance-info reads, runtime event instance correlation/fan-out planning, adapter-call execution planning, and shutdown reconciliation planning in crates/r3_core/src/orchestration.rs",
            "partial",
            "`cargo test -p r3_core orchestration`",
            "Port live adapter registry, session directory persistence execution, concrete adapter execution, and runtime event stream transport.",
        )
    } else if path.starts_with("apps/server/src/provider/") {
        (
            "provider display/model helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` provider/model tests",
            "Port provider drivers, registries, session runtimes, probes, maintenance, and adapters.",
        )
    } else if matches!(
        path,
        "apps/server/src/orchestration/decider.ts"
            | "apps/server/src/orchestration/decider.delete.test.ts"
            | "apps/server/src/orchestration/decider.projectScripts.test.ts"
            | "apps/server/src/orchestration/commandInvariants.ts"
            | "apps/server/src/orchestration/commandInvariants.test.ts"
            | "apps/server/src/orchestration/Errors.ts"
            | "apps/server/src/orchestration/Layers/OrchestrationEngine.ts"
            | "apps/server/src/orchestration/Layers/OrchestrationEngine.test.ts"
            | "apps/server/src/orchestration/Services/OrchestrationEngine.ts"
            | "apps/server/src/orchestration/Layers/ProviderCommandReactor.ts"
            | "apps/server/src/orchestration/Layers/ProviderCommandReactor.test.ts"
            | "apps/server/src/orchestration/Services/ProviderCommandReactor.ts"
            | "apps/server/src/orchestration/Layers/ThreadDeletionReactor.ts"
            | "apps/server/src/orchestration/Layers/ThreadDeletionReactor.test.ts"
            | "apps/server/src/orchestration/Services/ThreadDeletionReactor.ts"
            | "apps/server/src/orchestration/Layers/OrchestrationReactor.ts"
            | "apps/server/src/orchestration/Layers/OrchestrationReactor.test.ts"
            | "apps/server/src/orchestration/Services/OrchestrationReactor.ts"
            | "apps/server/src/orchestration/Layers/ProviderRuntimeIngestion.ts"
            | "apps/server/src/orchestration/Layers/ProviderRuntimeIngestion.test.ts"
            | "apps/server/src/orchestration/Services/ProviderRuntimeIngestion.ts"
            | "apps/server/src/orchestration/Layers/CheckpointReactor.ts"
            | "apps/server/src/orchestration/Layers/CheckpointReactor.test.ts"
            | "apps/server/src/orchestration/Services/CheckpointReactor.ts"
            | "apps/server/src/orchestration/Layers/RuntimeReceiptBus.ts"
            | "apps/server/src/orchestration/Services/RuntimeReceiptBus.ts"
            | "apps/server/src/orchestration/Normalizer.ts"
            | "apps/server/src/orchestration/Schemas.ts"
            | "apps/server/src/orchestration/http.ts"
            | "apps/server/src/orchestration/runtimeLayer.ts"
    ) {
        (
            "orchestration command decider, composite reactor order, provider runtime ingestion command planner plus queue/drain and ordered batch bridges, guarded helper/activity/session/assistant-delta/assistant-buffer-flush/assistant-complete/fallback/pause-finalize/turn-complete-finalize/proposed-plan/proposed-plan-finalize/diff/thread-metadata command mapping, store execution, persisted-event reactor intent bridge/batch API, provider service request mapping, runtime receipt bus, checkpoint reactor filters/status/cwd/revert plans, normalizer attachment plans, schema aliases, runtime layer composition, and orchestration HTTP route contracts in crates/r3_core",
            "partial",
            "`cargo test -p r3_core orchestration`; `cargo test -p r3_core persistence`",
            "Port exact event IDs, full read-model projection during command sequences, live reactor workers, subscriptions, provider runtime wiring, real checkpoint git side effects, attachment file writes, and live HTTP dispatch/auth execution.",
        )
    } else if path.starts_with("apps/server/integration/") {
        (
            "integration harness provider runtime event normalization, provider-service request/fan-out, orchestration layer plan, fixture runtime event, and wait/retry contracts across crates/r3_core/src/orchestration.rs and crates/r3_core/src/persistence.rs",
            "partial",
            "`cargo test -p r3_core orchestration`; `cargo test -p r3_core persistence`",
            "Port live temp git workspace setup, managed Effect runtime, SQLite-backed integration harness, real adapter registry, runtime receipt bus, provider stream collection, and full end-to-end integration tests.",
        )
    } else if matches!(
        path,
        "apps/server/src/orchestration/Layers/ProjectionSnapshotQuery.ts"
            | "apps/server/src/orchestration/Layers/ProjectionSnapshotQuery.test.ts"
            | "apps/server/src/orchestration/Services/ProjectionSnapshotQuery.ts"
            | "apps/server/src/orchestration/projector.ts"
            | "apps/server/src/orchestration/projector.test.ts"
            | "apps/server/src/orchestration/Layers/ProjectionPipeline.ts"
            | "apps/server/src/orchestration/Layers/ProjectionPipeline.test.ts"
            | "apps/server/src/orchestration/Services/ProjectionPipeline.ts"
    ) {
        (
            "projection shell mapper and event projector in crates/r3_core",
            "partial",
            "`cargo test -p r3_core projection`; `cargo test -p r3_core persistence`",
            "Port full command/detail snapshots, projector event coverage, attachment side effects, checkpoint/pending/session projectors, and repository wiring.",
        )
    } else if matches!(
        path,
        "apps/server/src/persistence/Migrations/001_OrchestrationEvents.ts"
            | "apps/server/src/persistence/Migrations/002_OrchestrationCommandReceipts.ts"
            | "apps/server/src/persistence/Migrations/003_CheckpointDiffBlobs.ts"
            | "apps/server/src/persistence/Migrations/004_ProviderSessionRuntime.ts"
            | "apps/server/src/persistence/Layers/OrchestrationEventStore.test.ts"
            | "apps/server/src/persistence/Layers/OrchestrationEventStore.ts"
            | "apps/server/src/persistence/Services/OrchestrationEventStore.ts"
            | "apps/server/src/persistence/Layers/OrchestrationCommandReceipts.ts"
            | "apps/server/src/persistence/Services/OrchestrationCommandReceipts.ts"
            | "apps/server/src/persistence/Errors.ts"
            | "apps/server/src/persistence/Layers/Sqlite.ts"
            | "apps/server/src/persistence/Migrations.ts"
            | "apps/server/src/persistence/Migrations/005_Projections.ts"
            | "apps/server/src/persistence/Migrations/006_ProjectionThreadSessionRuntimeModeColumns.ts"
            | "apps/server/src/persistence/Migrations/007_ProjectionThreadMessageAttachments.ts"
            | "apps/server/src/persistence/Migrations/008_ProjectionThreadActivitySequence.ts"
            | "apps/server/src/persistence/Migrations/009_ProviderSessionRuntimeMode.ts"
            | "apps/server/src/persistence/Migrations/010_ProjectionThreadsRuntimeMode.ts"
            | "apps/server/src/persistence/Migrations/011_OrchestrationThreadCreatedRuntimeMode.ts"
            | "apps/server/src/persistence/Migrations/012_ProjectionThreadsInteractionMode.ts"
            | "apps/server/src/persistence/Migrations/013_ProjectionThreadProposedPlans.ts"
            | "apps/server/src/persistence/Migrations/014_ProjectionThreadProposedPlanImplementation.ts"
            | "apps/server/src/persistence/Migrations/015_ProjectionTurnsSourceProposedPlan.ts"
            | "apps/server/src/persistence/Migrations/016_CanonicalizeModelSelections.ts"
            | "apps/server/src/persistence/Migrations/016_CanonicalizeModelSelections.test.ts"
            | "apps/server/src/persistence/Migrations/017_ProjectionThreadsArchivedAt.ts"
            | "apps/server/src/persistence/Migrations/018_ProjectionThreadsArchivedAtIndex.ts"
            | "apps/server/src/persistence/Migrations/019_ProjectionSnapshotLookupIndexes.ts"
            | "apps/server/src/persistence/Migrations/019_ProjectionSnapshotLookupIndexes.test.ts"
            | "apps/server/src/persistence/Migrations/020_AuthAccessManagement.ts"
            | "apps/server/src/persistence/Migrations/021_AuthSessionClientMetadata.ts"
            | "apps/server/src/persistence/Migrations/022_AuthSessionLastConnectedAt.ts"
            | "apps/server/src/persistence/Migrations/023_ProjectionThreadShellSummary.ts"
            | "apps/server/src/persistence/Migrations/024_BackfillProjectionThreadShellSummary.ts"
            | "apps/server/src/persistence/Migrations/024_BackfillProjectionThreadShellSummary.test.ts"
            | "apps/server/src/persistence/Migrations/025_CleanupInvalidProjectionPendingApprovals.ts"
            | "apps/server/src/persistence/Migrations/025_CleanupInvalidProjectionPendingApprovals.test.ts"
            | "apps/server/src/persistence/Migrations/026_CanonicalizeModelSelectionOptions.ts"
            | "apps/server/src/persistence/Migrations/026_CanonicalizeModelSelectionOptions.test.ts"
            | "apps/server/src/persistence/Migrations/027_ProviderSessionRuntimeInstanceId.ts"
            | "apps/server/src/persistence/Migrations/027_028_ProviderInstanceIdColumns.test.ts"
            | "apps/server/src/persistence/Migrations/028_ProjectionThreadSessionInstanceId.ts"
            | "apps/server/src/persistence/Migrations/029_ProjectionThreadDetailOrderingIndexes.ts"
            | "apps/server/src/persistence/Migrations/029_ProjectionThreadDetailOrderingIndexes.test.ts"
            | "apps/server/src/persistence/Migrations/030_ProjectionThreadShellArchiveIndexes.ts"
            | "apps/server/src/persistence/NodeSqliteClient.ts"
            | "apps/server/src/persistence/NodeSqliteClient.test.ts"
            | "apps/server/src/persistence/Layers/AuthPairingLinks.ts"
            | "apps/server/src/persistence/Layers/AuthSessions.ts"
            | "apps/server/src/persistence/Layers/ProjectionPendingApprovals.ts"
            | "apps/server/src/persistence/Layers/ProjectionCheckpoints.ts"
            | "apps/server/src/persistence/Layers/ProjectionProjects.ts"
            | "apps/server/src/persistence/Layers/ProjectionRepositories.test.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreads.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreadSessions.ts"
            | "apps/server/src/persistence/Layers/ProviderSessionRuntime.ts"
            | "apps/server/src/persistence/Layers/ProjectionTurns.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreadMessages.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreadMessages.test.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreadActivities.ts"
            | "apps/server/src/persistence/Layers/ProjectionThreadProposedPlans.ts"
            | "apps/server/src/persistence/Layers/ProjectionState.ts"
            | "apps/server/src/persistence/Services/ProjectionPendingApprovals.ts"
            | "apps/server/src/persistence/Services/ProjectionCheckpoints.ts"
            | "apps/server/src/persistence/Services/ProjectionProjects.ts"
            | "apps/server/src/persistence/Services/ProjectionThreads.ts"
            | "apps/server/src/persistence/Services/ProjectionThreadSessions.ts"
            | "apps/server/src/persistence/Services/ProviderSessionRuntime.ts"
            | "apps/server/src/persistence/Services/ProjectionTurns.ts"
            | "apps/server/src/persistence/Services/ProjectionThreadMessages.ts"
            | "apps/server/src/persistence/Services/ProjectionThreadActivities.ts"
            | "apps/server/src/persistence/Services/ProjectionThreadProposedPlans.ts"
            | "apps/server/src/persistence/Services/ProjectionState.ts"
            | "apps/server/src/persistence/Services/AuthPairingLinks.ts"
            | "apps/server/src/persistence/Services/AuthSessions.ts"
    ) {
        (
            "projection SQLite store plus provider_session_runtime table/repository, auth session/pairing-link tables, auth repository helpers, session-directory binding helpers, persistence error tags/messages, sqlite runtime/client config, setup pragmas, node sqlite compatibility gate, and 30-entry migration ordering/filtering in crates/r3_core/src/persistence.rs",
            "partial",
            "`cargo test -p r3_core persistence`",
            "Port typed orchestration events, projector integration, live auth service integration, live provider runtime integration, live Effect SQL layer execution, and complete migration SQL/backfill semantics.",
        )
    } else if path == "packages/contracts/src/auth.ts" {
        (
            "auth descriptor, HTTP/websocket credential selection with URLSearchParams-compatible websocket token query decoding, owner access-control/error mapping, HTTP route plans, auth HTTP error/success/session-state response and CORS/cookie contracts, AuthControlPlane CLI session defaults plus pairing/session listing rules, bootstrap credential issue/consume decision contracts, pairing-token request body detection, secret-store filesystem contracts plus filesystem get/set/get-or-create/remove helpers, cookie, client metadata, pairing-token, HMAC-signed session/websocket token issue/verify helpers with default TTLs and claim decode/expiry checks, persisted token-to-session repository verification decisions, verified-session credential assembly, session-claim, access-stream change fan-in/current-session marking, session-credential change, connected-session count, and auth persistence contracts in crates/r3_core/src/auth.rs plus crates/r3_core/src/persistence.rs",
            "partial",
            "`cargo test -p r3_core auth`; `cargo test -p r3_core persistence`",
            "Wire full live secret-store service permissions/concurrency layer, live repository-backed auth service execution, concrete HTTP auth exchange execution, atomic live bootstrap consume/emit behavior, websocket upgrade execution, live auth PubSub streams, and persisted runtime integration.",
        )
    } else if path == "apps/server/src/processRunner.ts"
        || path == "apps/server/src/processRunner.test.ts"
        || path == "apps/server/src/open.ts"
        || path == "apps/server/src/open.test.ts"
        || path == "apps/server/src/process/externalLauncher.ts"
        || path == "apps/server/src/process/externalLauncher.test.ts"
        || path == "packages/contracts/src/editor.ts"
        || path == "packages/shared/src/shell.ts"
        || path == "packages/shared/src/shell.test.ts"
    {
        (
            "process runner output limits, timeout synthetic-result behavior, command availability, browser launch, and external editor launch contracts in crates/r3_core/src/process.rs",
            "partial",
            "`cargo test -p r3_core process`",
            "Wire live RPC shell.openInEditor, browser opener process spawning, process-tree kill behavior, and every shell environment probe into the Rust runtime.",
        )
    } else if matches!(
        path,
        "apps/server/src/pathExpansion.ts"
            | "apps/server/src/pathExpansion.test.ts"
            | "apps/server/src/startupAccess.ts"
            | "apps/server/src/startupAccess.test.ts"
    ) {
        (
            "server path expansion and headless startup access contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live ServerConfig/HttpServer/ServerAuth access issuance and runtime pairing credential issuance.",
        )
    } else if matches!(
        path,
        "apps/server/src/atomicWrite.ts"
            | "apps/server/src/stream/collectUint8StreamText.ts"
            | "apps/server/src/stream/collectUint8StreamText.test.ts"
    ) {
        (
            "server stream text collection and atomic-write contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live Effect Stream decoder flushing, filesystem temp-directory scope cleanup, fsync behavior if needed, and cross-device rename error handling.",
        )
    } else if matches!(
        path,
        "apps/server/src/cliAuthFormat.ts" | "apps/server/src/cliAuthFormat.test.ts"
    ) {
        (
            "server CLI auth formatting contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire exact DateTime conversion types, CLI command integration, and JSON object field omission parity.",
        )
    } else if path == "apps/server/src/config.ts" {
        (
            "server runtime mode/defaults and derived-path contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live Effect layers, directory creation, static-dir lookup, Config service, and test-layer construction.",
        )
    } else if path == "apps/server/package.json"
        || path == "apps/server/tsconfig.json"
        || path == "apps/server/tsdown.config.ts"
        || path == "apps/server/vitest.config.ts"
        || path == "apps/server/scripts/cli.ts"
    {
        (
            "server package metadata, package scripts, dependency/package role lists, tsdown build config, vitest runtime config, and build/publish CLI plans in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server_package`",
            "Wire real TypeScript compiler config, package publishing file rewrites, catalog dependency resolution, icon override file IO, npm publish execution, and binary/package release artifacts.",
        )
    } else if path == "apps/server/scripts/acp-mock-agent.ts"
        || path == "apps/server/scripts/cursor-acp-model-mismatch-probe.ts"
    {
        (
            "ACP mock-agent state/config/env contracts and Cursor ACP model-mismatch probe request plan in crates/r3_core/src/effect_acp.rs",
            "partial",
            "`cargo test -p r3_core acp_mock`",
            "Wire live Bun/Node script execution, stdio JSON-RPC process control, request log/exit log file IO, permission/elicitation callback behavior, and real Cursor agent probing.",
        )
    } else if path == "apps/server/src/bootstrap.ts" || path == "apps/server/src/bootstrap.test.ts"
    {
        (
            "server bootstrap fd path and envelope decode contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live fd readiness probing, stream duplication/fallback, readline timeout cleanup, and schema-specific decoding.",
        )
    } else if path == "apps/server/src/server.ts" || path == "apps/server/src/server.test.ts" {
        (
            "server layer composition, route membership, WebSocket RPC route plan, Bun/Node HTTP/PTY/platform adapter selection, reactor/provider/runtime dependency groups, runtime-state/tailscale side-effect plan, and launch provider contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live Effect layer graph, HTTP server launch, route handlers, real WebSocket execution, runtime state acquire/release, Tailscale Serve side effects, browser OTLP tests, and bootstrap worktree dispatch flow.",
        )
    } else if path == "apps/server/src/serverRuntimeStartup.ts"
        || path == "apps/server/src/serverRuntimeStartup.test.ts"
    {
        (
            "server runtime startup model/welcome/command-gate contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live command queue workers, readiness Deferreds, lifecycle events, reactors, heartbeat telemetry, auto-bootstrap dispatch, auth pairing URL, and browser/headless side effects.",
        )
    } else if path == "apps/server/src/serverLifecycleEvents.ts"
        || path == "apps/server/src/serverLifecycleEvents.test.ts"
        || path == "apps/server/src/serverRuntimeState.ts"
    {
        (
            "server lifecycle snapshot/sequence and persisted runtime-state contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live PubSub streams, Ref state, runtime-state file persistence/read/clear, and DateTime/process pid integration.",
        )
    } else if path == "apps/server/src/serverSettings.ts"
        || path == "apps/server/src/serverSettings.test.ts"
    {
        (
            "server settings provider environment secret/redaction contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live settings schema normalization, sparse default stripping, file watch/cache/pubsub runtime, atomic writes, secret-store materialization/persistence, deep patch merge, and text-generation provider fallback.",
        )
    } else if path == "apps/server/src/cli/config.ts"
        || path == "apps/server/src/cli/config.test.ts"
    {
        (
            "server CLI config precedence and duration shorthand contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live Effect ConfigProvider/env reading, bootstrap fd fallback, filesystem directory creation, static-dir lookup, persisted observability loading, and full ServerConfig assembly.",
        )
    } else if path == "apps/server/src/serverLogger.ts" {
        (
            "server logger minimum-level and logger-list layer contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live Effect Logger layer, References.MinimumLogLevel, consolePretty output, and tracer logger integration.",
        )
    } else if path == "apps/server/src/bin.ts"
        || path == "apps/server/src/bin.test.ts"
        || path == "apps/server/src/cli/server.ts"
        || path == "apps/server/src/cli/auth.ts"
        || path == "apps/server/src/cli/project.ts"
    {
        (
            "CLI command topology, serve run-plan, auth/project command descriptors, and project dev-url rejection contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live effect/unstable CLI parsing, Node runtime layer, auth/project command execution, TTL schema errors, live/offline project mutation dispatch, runtime-state probing, and versioned binary packaging.",
        )
    } else if path == "apps/server/src/observability/Attributes.ts"
        || path == "apps/server/src/observability/Attributes.test.ts"
        || path == "apps/server/src/observability/Metrics.ts"
        || path == "apps/server/src/observability/Metrics.test.ts"
        || path == "apps/server/src/observability/RpcInstrumentation.ts"
        || path == "apps/server/src/observability/RpcInstrumentation.test.ts"
        || path == "apps/server/src/observability/Layers/Observability.ts"
        || path == "apps/server/src/observability/Services/BrowserTraceCollector.ts"
    {
        (
            "observability metric attribute/outcome/model-label, metric spec/update, RPC instrumentation, layer assembly, OTLP trace JSON decode, and browser trace collector contracts in crates/r3_core/src/observability.rs",
            "partial",
            "`cargo test -p r3_core observability`",
            "Wire live Effect Metric snapshots, Clock/TestClock durations, span/tracer runtime, Stream onExit instrumentation, disabled-tracer service behavior, local trace sink rotation, OTLP exporters, and Effect service layers.",
        )
    } else if path == "apps/server/src/telemetry/Identify.ts"
        || path == "apps/server/src/telemetry/Layers/AnalyticsService.ts"
        || path == "apps/server/src/telemetry/Layers/AnalyticsService.test.ts"
        || path == "apps/server/src/telemetry/Services/AnalyticsService.ts"
    {
        (
            "telemetry identifier priority, anonymous-id persistence plan, analytics buffer/flush/payload, and service-tag contracts in crates/r3_core/src/telemetry.rs",
            "partial",
            "`cargo test -p r3_core telemetry`",
            "Wire live SHA-256 hashing, Codex/Claude file reads, filesystem writes, Effect ConfigProvider, Ref buffer, periodic scoped flush, HttpClient PostHog submission, and finalizer behavior.",
        )
    } else if path == "apps/server/src/os-jank.ts"
        || path == "apps/server/src/environment/Layers/ServerEnvironmentLabel.ts"
        || path == "apps/server/src/environment/Layers/ServerEnvironmentLabel.test.ts"
        || path == "apps/server/src/environment/Layers/ServerEnvironment.ts"
        || path == "apps/server/src/environment/Layers/ServerEnvironment.test.ts"
        || path == "apps/server/src/environment/Services/ServerEnvironment.ts"
    {
        (
            "server base-dir/path, environment-label, and environment-descriptor contracts in crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core server`",
            "Wire live login-shell PATH hydration, Windows environment repair, launchctl fallback, macOS scutil, Linux /etc/machine-info reads, hostnamectl process execution, filesystem environment-id persistence, Effect service layer, and package metadata sourcing.",
        )
    } else if path == "apps/server/src/keybindings.ts"
        || path == "apps/server/src/keybindings.test.ts"
    {
        (
            "server keybindings parse/merge/default-sync/upsert/remove contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core keybinding`",
            "Wire file-backed keybindings.json IO, lenient JSON/schema diagnostics, atomic writes, cache invalidation, filesystem watch/debounce, PubSub streams, startup Deferreds, and RPC handlers.",
        )
    } else if path.starts_with("apps/server/src/orchestration/")
        || path.starts_with("apps/server/src/persistence/")
        || path.starts_with("apps/server/src/server")
        || path.starts_with("apps/server/src/bootstrap")
        || path.starts_with("apps/server/src/bin")
        || path.starts_with("apps/server/src/cli/")
        || path.starts_with("apps/server/src/config")
        || path.starts_with("apps/server/src/process")
        || path.starts_with("apps/server/src/open")
        || path.starts_with("apps/server/src/observability/")
    {
        (
            "none",
            "missing",
            "None",
            "Port server runtime layer or create equivalent Rust module and tests.",
        )
    } else if path.starts_with("apps/server/src/textGeneration/") {
        (
            "text generation policy, preset, prompt, and sanitizer contracts in crates/r3_core/src/text_generation.rs",
            "partial",
            "`cargo test -p r3_core text_generation`",
            "Port live provider-backed text generation for Codex, Claude, Cursor, and OpenCode plus registry dispatch.",
        )
    } else if path.starts_with("apps/server/src/attachment")
        || path.starts_with("apps/server/src/imageMime")
    {
        (
            "attachment path/store, HTTP route response decision plus filesystem response helper, and image MIME contracts in crates/r3_core/src/attachments.rs",
            "partial",
            "`cargo test -p r3_core attachments`",
            "Port live upload/write integration, HTTP transport wiring, and persisted attachment side effects.",
        )
    } else if path.starts_with("apps/server/src/http")
        || path.starts_with("apps/server/src/ws")
        || path == "packages/contracts/src/rpc.ts"
    {
        (
            "transport-agnostic HTTP route plus environment response, attachment route decisions, static/dev redirect/path-guard/fallback/content-type helpers with real static file/index read response helper, browser API CORS constants/preflight/header/layer/merge contracts, project favicon route decisions plus URLSearchParams-compatible cwd query decoding and file/fallback response helper, OTLP traces proxy route decisions, and WebSocket RPC method/group/schema/handler/dispatch/lifecycle contracts in crates/r3_core/src/rpc.rs plus crates/r3_core/src/server.rs",
            "partial",
            "`cargo test -p r3_core rpc`",
            "Port live HTTP server, WebSocket upgrade/auth handling, concrete handler execution, runtime schema decoding, subscriptions, OTLP collector record/export execution, and actual CORS middleware attachment.",
        )
    } else if path.starts_with("apps/server/") {
        (
            "none",
            "missing",
            "None",
            "Classify and port server package/build/runtime surface.",
        )
    } else if path.starts_with("apps/desktop/src/ssh/") || path.starts_with("packages/ssh/src/") {
        (
            "selected SSH parsing, config/known_hosts discovery, command arg, output-line, connection-key, remote package-spec, askpass, child-env, auth-failure, tunnel JSON decode, and remote runner/launch/pairing/stop script-builder contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` SSH parse tests",
            "Port live SSH discovery/tunnel execution, password prompts, and remote API/session bootstrap.",
        )
    } else if path.starts_with("apps/desktop/src/backend/tailscale")
        || path.starts_with("packages/tailscale/src/")
    {
        (
            "Tailscale IPv4/MagicDNS parsing, HTTPS URL, serve command, and advertised endpoint contracts in crates/r3_core/src/desktop.rs plus pairing endpoint helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core desktop`; `cargo test --workspace` endpoint tests",
            "Port live Tailscale CLI execution, status timeout handling, HTTPS probe, serve enable/disable effects, and server exposure wiring.",
        )
    } else if matches!(
        path,
        "apps/desktop/src/backend/DesktopServerExposure.ts"
            | "apps/desktop/src/backend/DesktopServerExposure.test.ts"
    ) {
        (
            "desktop server exposure runtime state, LAN host resolution, backend config, and advertised endpoint contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live settings persistence service, Effect layer wiring, Tailscale endpoint provider integration, and relaunch orchestration.",
        )
    } else if matches!(
        path,
        "apps/desktop/src/backend/DesktopBackendManager.ts"
            | "apps/desktop/src/backend/DesktopBackendManager.test.ts"
    ) {
        (
            "desktop backend manager constants, readiness URL, restart backoff, snapshot, and backend-start contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live process spawning, fd3 bootstrap streaming, HTTP readiness retry loop, scoped restart supervision, and output logging.",
        )
    } else if matches!(
        path,
        "apps/desktop/src/main.ts"
            | "apps/desktop/src/app/DesktopApp.ts"
            | "apps/desktop/src/app/DesktopLifecycle.ts"
            | "apps/desktop/src/app/DesktopAppIdentity.ts"
            | "apps/desktop/src/app/DesktopAppIdentity.test.ts"
            | "apps/desktop/src/app/DesktopAssets.ts"
            | "apps/desktop/src/app/DesktopConfig.ts"
            | "apps/desktop/src/app/DesktopEnvironment.ts"
            | "apps/desktop/src/app/DesktopEnvironment.test.ts"
            | "apps/desktop/src/app/DesktopState.ts"
            | "apps/desktop/src/app/DesktopObservability.ts"
            | "apps/desktop/src/app/DesktopObservability.test.ts"
            | "apps/desktop/src/backend/DesktopBackendConfiguration.ts"
            | "apps/desktop/src/backend/DesktopBackendConfiguration.test.ts"
            | "apps/desktop/src/updates/DesktopUpdates.ts"
            | "apps/desktop/src/updates/DesktopUpdates.test.ts"
            | "apps/desktop/src/updates/updateChannels.ts"
            | "apps/desktop/src/updates/updateMachine.ts"
            | "apps/desktop/src/updates/updateMachine.test.ts"
    ) {
        (
            "desktop app bootstrap/lifecycle/assets/identity/config/environment/state/observability/update-channel/update-runtime/backend-start decision contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live Electron/GPUI app integration, Effect layers, filesystem-backed asset lookup, shutdown deferreds, event listeners, fatal-startup UI side effects, rotating log file IO, trace sink/tracer wiring, backend process supervision, settings IO, update runtime side effects, menus, protocols, and IPC.",
        )
    } else if matches!(
        path,
        "apps/desktop/src/electron/ElectronApp.ts"
            | "apps/desktop/src/electron/ElectronApp.test.ts"
            | "apps/desktop/src/electron/ElectronDialog.ts"
            | "apps/desktop/src/electron/ElectronDialog.test.ts"
            | "apps/desktop/src/electron/ElectronMenu.ts"
            | "apps/desktop/src/electron/ElectronMenu.test.ts"
            | "apps/desktop/src/electron/ElectronProtocol.ts"
            | "apps/desktop/src/electron/ElectronProtocol.test.ts"
            | "apps/desktop/src/electron/ElectronSafeStorage.ts"
            | "apps/desktop/src/electron/ElectronShell.ts"
            | "apps/desktop/src/electron/ElectronShell.test.ts"
            | "apps/desktop/src/electron/ElectronTheme.ts"
            | "apps/desktop/src/electron/ElectronTheme.test.ts"
            | "apps/desktop/src/electron/ElectronUpdater.ts"
            | "apps/desktop/src/electron/ElectronUpdater.test.ts"
            | "apps/desktop/src/electron/ElectronWindow.ts"
            | "apps/desktop/src/electron/ElectronWindow.test.ts"
    ) {
        (
            "Electron app metadata/listener/switch, dialog confirm/pick-folder, menu normalization, protocol static-file routing, safe-storage errors, safe shell URL, native-theme, updater error/listener/property, and BrowserWindow ownership/reveal/send contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live Electron service wrappers, async rejection paths, clipboard side effects, native menu icons/popups, protocol registration/unregistration, safe-storage encryption/decryption, native theme state, scoped listener wiring, and GPUI BrowserWindow-equivalent lifecycle.",
        )
    } else if path.starts_with("apps/desktop/src/shell/") {
        (
            "desktop shell environment login-shell/PowerShell probe, marker extraction, PATH merge, and environment patch contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live process probing, timeout/termination behavior, process.env mutation, and platform-specific shell integration.",
        )
    } else if path == "apps/desktop/src/preload.ts" || path.starts_with("apps/desktop/src/ipc/") {
        (
            "desktop IPC channel list, handler registration order, preload bridge method table, listener guards, and SSH-cancel unwrap contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live GPUI/Electron IPC bridge, browser preload exposure, payload schemas, client settings IO, saved-environment secrets, server exposure controls, SSH runtime, update runtime, and window/menu handlers.",
        )
    } else if path.starts_with("apps/desktop/src/settings/") {
        (
            "desktop app settings, update-channel migration, sparse settings document, saved-environment registry, and secret-preservation contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live settings file IO, lenient JSONC decode/encode, full client settings schema, Electron safe-storage encryption/decryption, and IPC method wiring.",
        )
    } else if path.starts_with("apps/desktop/src/window/") {
        (
            "desktop native menu template, menu action labels, main-window option, titlebar, and background-color contracts in crates/r3_core/src/desktop.rs",
            "partial",
            "`cargo test -p r3_core desktop`",
            "Port live GPUI/Electron window creation, context menu spellcheck/media/link behavior, window-open guard, reveal lifecycle, menu click effects, and update dialogs.",
        )
    } else if path.starts_with("apps/desktop/") {
        (
            "desktop package metadata, scripts, tsdown entries, Electron launcher/dev/smoke/wait-resource plans in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Wire native GPUI packaging/release artifacts, live dev Electron restart loop equivalent, macOS bundle/icon patching, smoke-test process execution, and exact installer metadata.",
        )
    } else if path == "packages/shared/package.json" || path == "packages/shared/tsconfig.json" {
        (
            "shared package metadata, scripts, export map, dependency/version map, and tsconfig include/extends contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Generate native Rust package exports and align build/typecheck outputs with crate/package layout.",
        )
    } else if path == "packages/shared/src/keybindings.ts" {
        (
            "shared keybinding defaults, shortcut parser, when parser, resolved config compiler, and max-count contracts in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core keybinding`",
            "Port generated shared-package exports and keep server/UI callers on one Rust source of truth.",
        )
    } else if path.starts_with("packages/shared/") {
        (
            "shared-package string, CLI args, path, semver, git remote/branch/status, source-control terminology/provider detection, search ranking, TCP port helper, Struct deep-merge, schemaJson object/strict/unknown/lenient/pretty transformation contracts, server settings patch helpers, deterministic worker state/runtime-plan contracts, rotating-log write/rotation/prune plans, trace sink buffering plus Effect/OTLP trace record conversion and OTLP trace JSON decode contracts, shell command-availability, process, model, project-script, Nayuki QR text/binary/segment/advanced-codeword/module contracts, and keybinding contracts in crates/r3_core",
            "partial",
            "`cargo test -p r3_core shared`; `cargo test -p r3_core process`; selected r3_core model/search/keybinding tests",
            "Wire actual Effect queue fibers, Effect Schema runtime integration, logging/observability runtime layers with filesystem append/rename IO, generated package exports, and live network/service integration.",
        )
    } else if path == "packages/client-runtime/package.json"
        || path == "packages/client-runtime/tsconfig.json"
    {
        (
            "client-runtime package metadata, scripts, export map, dependency/version map, and tsconfig include/extends contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Wire browser AtomRegistry/reactivity runtime, async refresh deduplication with real RPC clients, generated contract types, and native package export generation.",
        )
    } else if path.starts_with("packages/client-runtime/") {
        (
            "client-runtime advertised endpoint, known-environment, scoped-ref, pairing URL get/strip/set, source-control discovery Atom labels/keepAlive, and state/refresh/reset decision helpers in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` client runtime helper tests",
            "Wire browser AtomRegistry/reactivity runtime, async refresh deduplication with real RPC clients, package exports, and generated contract types.",
        )
    } else if path == "packages/contracts/src/vcs.ts" {
        (
            "VCS driver, freshness, repository identity, remote-list, process-error, repository-detection, and unsupported-operation contract shapes/messages in crates/r3_core/src/lib.rs plus VCS process run contracts in crates/r3_core/src/process.rs",
            "partial",
            "`cargo test -p r3_core vcs`; `cargo test -p r3_core process`",
            "Port generated Effect schemas, exact DateTime types, schema decoders/encoders, and live VCS service integration.",
        )
    } else if path == "packages/contracts/src/sourceControl.ts" {
        (
            "source-control provider/auth/context contracts, change-request state/schema/normalizers, repository clone URL/visibility/info schemas, repository lookup plus clone/publish input/result contracts, VCS/provider discovery result contracts, provider/repository tagged error cause/message contracts, provider registration/binding/unsupported-operation contracts, Bitbucket API discovery/auth contracts, and provider operation wire values in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test -p r3_core source_control`",
            "Port generated Effect schemas, exact runtime validators/encoders, and any remaining live source-control RPC schema wiring.",
        )
    } else if path.starts_with("packages/contracts/src/") {
        (
            "selected structs/enums in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` selected contract tests",
            "Port every schema/contract or generate Rust equivalents.",
        )
    } else if matches!(
        path,
        "packages/effect-acp/package.json"
            | "packages/effect-acp/tsconfig.json"
            | "packages/effect-codex-app-server/package.json"
            | "packages/effect-codex-app-server/tsconfig.json"
    ) {
        (
            "Effect ACP and Codex app-server package metadata, scripts, export maps, build entrypoints, dependency/version maps, and tsconfig include/extends contracts in crates/r3_core/src/effect_acp.rs",
            "partial",
            "`cargo test -p r3_core effect_acp`",
            "Port generated schemas/types, Effect RPC clients, stdio transport, agent/client lifecycle, mock peers, probe examples, live package wiring, and native package export generation.",
        )
    } else if path.starts_with("packages/effect-codex-app-server/")
        || path.starts_with("packages/effect-acp/")
    {
        (
            "protocol method tables, wire-message routing, ACP request-error mapping, terminal plan, and package export/build-entrypoint contracts in crates/r3_core/src/effect_acp.rs",
            "partial",
            "`cargo test -p r3_core effect_acp`",
            "Port generated schemas/types, Effect RPC clients, stdio transport, agent/client lifecycle, mock peers, probe examples, and live package wiring.",
        )
    } else if matches!(
        path,
        "packages/ssh/package.json"
            | "packages/ssh/tsconfig.json"
            | "packages/tailscale/package.json"
            | "packages/tailscale/tsconfig.json"
    ) {
        (
            "SSH and Tailscale package metadata, scripts, export maps, dependency/version maps, and tsconfig include/extends contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Wire package build/typecheck/test surfaces and native module boundaries to the Rust crate layout while preserving SSH/Tailscale runtime behavior contracts.",
        )
    } else if path.starts_with("packages/tailscale/") || path.starts_with("packages/ssh/") {
        (
            "selected helpers, including SSH config/command/auth/tunnel contracts and pairing endpoint helpers, in crates/r3_core/src/lib.rs",
            "partial",
            "`cargo test --workspace` selected helper tests",
            "Port full package behavior.",
        )
    } else if path == "packages/contracts/package.json"
        || path == "packages/contracts/tsconfig.json"
    {
        (
            "contracts package metadata, scripts, export map, dependency/version map, and tsconfig include/extends contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Generate or port complete Rust schema package exports and align build/typecheck outputs with the native crate layout.",
        )
    } else if path.starts_with("packages/") {
        (
            "none",
            "missing",
            "None",
            "Classify package role and port or document intentional exclusion.",
        )
    } else if path.starts_with("apps/marketing/") {
        (
            "marketing package metadata, Astro scripts, and GitHub latest-release URL/cache contracts in crates/r3_core/src/package_surfaces.rs",
            "partial",
            "`cargo test -p r3_core package_surfaces`",
            "Port or intentionally ship the marketing site in Rust/web release flow, including Astro pages, static assets, fetch/sessionStorage runtime, and public release URL ownership.",
        )
    } else {
        (
            "none",
            "missing",
            "None",
            "Unclassified upstream file; assign owner before claiming exact parity.",
        )
    };

    InventoryRow {
        path: path.to_string(),
        rust_target,
        status,
        proof,
        remaining_gap,
    }
}

fn escape_markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn capture_reference_browser(options: CaptureReferenceOptions) -> Result<()> {
    if !options.repo.exists() {
        run(Command::new("git").args([
            "clone",
            concat!("https://github.com/pingdotgg/", "t3", "code.git"),
            options.repo.to_string_lossy().as_ref(),
        ]))?;
    }
    run(Command::new("git")
        .args(["fetch", "--depth=1", "origin", REFERENCE_COMMIT])
        .current_dir(&options.repo))?;
    run(Command::new("git")
        .args(["checkout", "--detach", REFERENCE_COMMIT])
        .current_dir(&options.repo))?;

    if options.home.exists() {
        fs::remove_dir_all(&options.home)?;
    }
    fs::create_dir_all(&options.home)?;
    fs::create_dir_all(&options.output_dir)?;

    let playwright_path = options
        .repo
        .join("node_modules")
        .join(".bun")
        .join("node_modules")
        .join("playwright");
    run(Command::new("bun")
        .arg("install")
        .current_dir(&options.repo))?;

    let commit = command_stdout(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&options.repo),
    )?;
    let stdout_path = env::temp_dir().join("upstream-reference.out.log");
    let stderr_path = env::temp_dir().join("upstream-reference.err.log");
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);

    let stdout = fs::File::create(&stdout_path)?;
    let stderr = fs::File::create(&stderr_path)?;
    let mut child = Command::new("bun")
        .args(["run", "dev", "--no-browser"])
        .current_dir(&options.repo)
        .env(concat!("T3", "CODE_HOME"), &options.home)
        .env(concat!("T3", "CODE_NO_BROWSER"), "1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;

    let result = (|| -> Result<()> {
        let mut pairing_url = wait_for_pairing_url(
            &mut child,
            &stdout_path,
            &stderr_path,
            options.startup_timeout,
        )?;
        thread::sleep(Duration::from_secs(15));
        if let Some(status) = child.try_wait()? {
            let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
            let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
            return Err(format!(
                "Reference dev server exited after pairing URL was available. Exit={status}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
            )
            .into());
        }
        if let Ok(stdout) = fs::read_to_string(&stdout_path) {
            if let Some(latest_pairing_url) = extract_pairing_url(&stdout) {
                pairing_url = latest_pairing_url;
            }
        }
        let script_path = env::temp_dir().join("capture-reference-browser.cjs");
        fs::write(&script_path, browser_capture_script())?;
        run(Command::new("node")
            .arg(&script_path)
            .env("PLAYWRIGHT_PATH", &playwright_path)
            .env("PAIRING_URL", pairing_url)
            .env("OUTPUT_DIR", &options.output_dir)
            .env("REFERENCE_PROJECT_PATH", &options.repo))?;

        fs::write(
            options.output_dir.join("CAPTURE_MANIFEST.txt"),
            format!(
                "Upstream reference repository: {}\nReference commit: {}\nIsolated reference home: {}\nOutput directory: {}\nCaptured:\n- upstream-empty-reference.png\n- upstream-command-palette-reference.png\n- upstream-draft-reference.png\n- upstream-composer-focused-reference.png\n- upstream-composer-menu-reference.png\n- upstream-composer-inline-tokens-reference.png\n- upstream-provider-model-picker-reference.png\n- upstream-branch-toolbar-reference.png\n- upstream-sidebar-options-menu-reference.png\n- upstream-open-in-menu-reference.png\n- upstream-git-actions-menu-reference.png\n- upstream-active-chat-reference.png\n- upstream-project-scripts-menu-reference.png\n- upstream-running-turn-reference.png\n- upstream-terminal-drawer-reference.png\n- upstream-diff-panel-reference.png\n- upstream-pending-user-input-reference.png\n- upstream-pending-approval-reference.png\n- upstream-settings-reference.png\n- upstream-settings-keybindings-reference.png\n- upstream-settings-keybindings-add-reference.png\n- upstream-settings-providers-reference.png\n- upstream-settings-source-control-reference.png\n- upstream-settings-connections-reference.png\n- upstream-settings-diagnostics-reference.png\n- upstream-settings-archive-reference.png\n- upstream-settings-theme-menu-reference.png\n- upstream-settings-dark-reference.png\n- upstream-empty-dark-reference.png\n",
                options.repo.display(),
                commit.trim(),
                options.home.display(),
                options.output_dir.display()
            ),
        )?;
        println!(
            "Captured upstream reference screenshots from {}",
            commit.trim()
        );
        Ok(())
    })();

    kill_process_tree(child.id());
    stop_child(&mut child);
    result
}

fn wait_for_pairing_url(
    child: &mut Child,
    stdout_path: &Path,
    stderr_path: &Path,
    timeout: Duration,
) -> Result<String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            let stdout = fs::read_to_string(stdout_path).unwrap_or_default();
            let stderr = fs::read_to_string(stderr_path).unwrap_or_default();
            return Err(format!(
                "Reference dev server exited before pairing URL was available. Exit={status}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
            )
            .into());
        }

        if let Ok(stdout) = fs::read_to_string(stdout_path) {
            if let Some(url) = extract_pairing_url(&stdout) {
                return Ok(url);
            }
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err("timed out waiting for reference pairing URL".into())
}

fn extract_pairing_url(text: &str) -> Option<String> {
    text.lines().rev().find_map(|line| {
        let marker = "pairingUrl: ";
        line.find(marker)
            .map(|index| line[index + marker.len()..].trim().to_string())
    })
}

fn browser_capture_script() -> &'static str {
    r#"const { chromium } = require(process.env.PLAYWRIGHT_PATH);
const path = require("path");

(async () => {
  const appOrigin = new URL(process.env.PAIRING_URL).origin;
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  async function dismissUpdatesToast() {
    if (await page.getByText("Updates Available").first().isVisible({ timeout: 500 }).catch(() => false)) {
      await page.mouse.click(1242, 89);
      await page.waitForTimeout(250);
    }
  }
  async function seedActiveChatReference() {
    await page.evaluate(async () => {
      const { useStore } = await import("/src/store.ts");
      const { useUiStateStore } = await import("/src/uiStateStore.ts");
      const { useTerminalStateStore } = await import("/src/terminalStateStore.ts");
      const environmentId = "local";
      const projectId = "project-r3code";
      const threadId = "thread-r3code-ui-shell";
      const now = "2026-03-04T12:00:12.000Z";
      const modelSelection = { instanceId: "codex", model: "gpt-5.4-mini" };
      const project = {
        id: projectId,
        title: "r3code",
        workspaceRoot: "C:\\Users\\bunny\\Downloads\\r3code",
        repositoryIdentity: null,
        defaultModelSelection: modelSelection,
        scripts: [
          { id: "script-test", name: "test", command: "cargo test --workspace", icon: "test-tube", createdAt: now, updatedAt: now },
          { id: "script-parity", name: "parity", command: "cargo run -p xtask -- check-parity --allow-window-capture", icon: "sparkles", createdAt: now, updatedAt: now },
        ],
        createdAt: "2026-03-04T11:59:00.000Z",
        updatedAt: now,
        deletedAt: null,
      };
      const session = {
        threadId,
        status: "ready",
        providerName: "codex",
        runtimeMode: "full-access",
        activeTurnId: null,
        lastError: null,
        updatedAt: now,
      };
      const latestTurn = {
        turnId: "turn-r3code-ui-shell-2",
        state: "completed",
        requestedAt: "2026-03-04T12:00:09.000Z",
        startedAt: "2026-03-04T12:00:10.000Z",
        completedAt: "2026-03-04T12:05:18.000Z",
        assistantMessageId: "msg-assistant-r3code-ui-shell",
      };
      const threadShell = {
        id: threadId,
        projectId,
        title: "Port R3Code UI shell",
        modelSelection,
        interactionMode: "default",
        runtimeMode: "full-access",
        branch: "main",
        worktreePath: null,
        latestTurn,
        createdAt: "2026-03-04T11:59:00.000Z",
        updatedAt: now,
        archivedAt: null,
        deletedAt: null,
        session,
        latestUserMessageAt: "2026-03-04T12:00:09.000Z",
        hasPendingApprovals: false,
        hasPendingUserInput: false,
        hasActionableProposedPlan: false,
      };
      const checkpoints = [
        {
          turnId: "turn-r3code-ui-shell-2",
          completedAt: "2026-03-04T12:05:18.000Z",
          status: "completed",
          assistantMessageId: "msg-assistant-r3code-ui-shell",
          checkpointTurnCount: 2,
          checkpointRef: "checkpoint-turn-2",
          files: [
            { path: "crates/r3_ui/src/shell.rs", kind: "modified", additions: 126, deletions: 18 },
            { path: "crates/r3_core/src/lib.rs", kind: "modified", additions: 74, deletions: 4 },
            { path: "docs/reference/PARITY_PLAN.md", kind: "modified", additions: 8, deletions: 0 },
          ],
        },
        {
          turnId: "turn-r3code-ui-shell-1",
          completedAt: "2026-03-04T12:01:42.000Z",
          status: "completed",
          assistantMessageId: "msg-assistant-r3code-ui-shell",
          checkpointTurnCount: 1,
          checkpointRef: "checkpoint-turn-1",
          files: [
            { path: "crates/r3_ui/assets/icons/diff.svg", kind: "added", additions: 1, deletions: 0 },
            { path: "crates/r3_ui/src/assets.rs", kind: "modified", additions: 6, deletions: 1 },
          ],
        },
      ];
      const thread = {
        ...threadShell,
        messages: [
          {
            id: "msg-user-r3code-ui-shell",
            role: "user",
            text: "Make the Rust port match the original UI exactly.",
            turnId: "turn-r3code-ui-shell-2",
            streaming: false,
            createdAt: "2026-03-04T12:00:09.000Z",
            updatedAt: "2026-03-04T12:00:09.000Z",
          },
          {
            id: "msg-assistant-r3code-ui-shell",
            role: "assistant",
            text: "Building a static GPUI shell first, then replacing mock data with Rust state.",
            turnId: "turn-r3code-ui-shell-2",
            streaming: false,
            createdAt: now,
            updatedAt: "2026-03-04T12:05:18.000Z",
          },
        ],
        activities: [],
        proposedPlans: [],
        checkpoints,
      };
      useStore.setState({ activeEnvironmentId: environmentId, environmentStateById: {} });
      useStore.getState().syncServerShellSnapshot(
        {
          snapshotSequence: 100,
          projects: [project],
          threads: [threadShell],
          updatedAt: now,
        },
        environmentId,
      );
      useStore.getState().syncServerThreadDetail(thread, environmentId);
      useUiStateStore.setState({
        projectExpandedById: { [projectId]: true },
        projectOrder: [projectId],
        threadLastVisitedAtById: { [threadId]: Date.parse(now) },
      });
      useTerminalStateStore.persist.clearStorage();
      useTerminalStateStore.setState({
        terminalStateByThreadKey: {},
        terminalLaunchContextByThreadKey: {},
        terminalEventEntriesByKey: {},
        nextTerminalEventId: 1,
      });
    });
  }
  async function seedPendingUserInputReference() {
    await seedActiveChatReference();
    await page.evaluate(async () => {
      const { useStore } = await import("/src/store.ts");
      const environmentId = "local";
      const threadId = "thread-r3code-ui-shell";
      const activity = {
        id: "activity-user-input-requested",
        tone: "info",
        kind: "user-input.requested",
        summary: "User input requested",
        payload: {
          requestId: "req-browser-user-input",
          questions: [
            {
              id: "scope",
              header: "Scope",
              question: "What should this change cover?",
              options: [
                { label: "Tight", description: "Touch only the footer layout logic." },
                { label: "Broad", description: "Also adjust the related composer controls." },
              ],
            },
            {
              id: "risk",
              header: "Risk",
              question: "How aggressive should the imaginary plan be?",
              options: [
                { label: "Conservative", description: "Favor reliability and low-risk changes." },
                { label: "Balanced", description: "Mix quick wins with one structural improvement." },
              ],
            },
          ],
        },
        turnId: null,
        sequence: 1,
        createdAt: "2026-03-04T12:16:40.000Z",
      };
      useStore.setState((state) => {
        const environmentState = state.environmentStateById[environmentId];
        if (!environmentState) return state;
        const currentShell = environmentState.threadShellById[threadId];
        const currentSummary = environmentState.sidebarThreadSummaryById[threadId];
        const currentSession = environmentState.threadSessionById[threadId];
        return {
          ...state,
          environmentStateById: {
            ...state.environmentStateById,
            [environmentId]: {
              ...environmentState,
              threadShellById: {
                ...environmentState.threadShellById,
                [threadId]: currentShell
                  ? { ...currentShell, interactionMode: "plan" }
                  : currentShell,
              },
              threadSessionById: {
                ...environmentState.threadSessionById,
                [threadId]: currentSession
                  ? { ...currentSession, status: "running", orchestrationStatus: "running" }
                  : currentSession,
              },
              activityIdsByThreadId: {
                ...environmentState.activityIdsByThreadId,
                [threadId]: [activity.id],
              },
              activityByThreadId: {
                ...environmentState.activityByThreadId,
                [threadId]: { [activity.id]: activity },
              },
              sidebarThreadSummaryById: {
                ...environmentState.sidebarThreadSummaryById,
                [threadId]: currentSummary
                  ? {
                      ...currentSummary,
                      interactionMode: "plan",
                      hasPendingUserInput: true,
                      session: currentSummary.session
                        ? {
                            ...currentSummary.session,
                            status: "running",
                            orchestrationStatus: "running",
                          }
                        : currentSummary.session,
                    }
                  : currentSummary,
              },
            },
          },
        };
      });
    });
  }
  async function seedPendingApprovalReference() {
    await seedActiveChatReference();
    await page.evaluate(async () => {
      const { useStore } = await import("/src/store.ts");
      const environmentId = "local";
      const threadId = "thread-r3code-ui-shell";
      const activities = [
        {
          id: "approval-command-run-tests",
          tone: "approval",
          kind: "approval.requested",
          summary: "Command approval requested",
          payload: {
            requestId: "approval-command-run-tests",
            requestKind: "command",
            detail: "cargo test --workspace",
          },
          turnId: null,
          sequence: 1,
          createdAt: "2026-03-04T12:00:20.000Z",
        },
        {
          id: "approval-file-change",
          tone: "approval",
          kind: "approval.requested",
          summary: "File-change approval requested",
          payload: {
            requestId: "approval-file-change",
            requestKind: "file-change",
            detail: "Allow editing crates/r3_ui/src/shell.rs",
          },
          turnId: null,
          sequence: 2,
          createdAt: "2026-03-04T12:00:23.000Z",
        },
      ];
      const activityById = Object.fromEntries(activities.map((activity) => [activity.id, activity]));
      useStore.setState((state) => {
        const environmentState = state.environmentStateById[environmentId];
        if (!environmentState) return state;
        const currentShell = environmentState.threadShellById[threadId];
        const currentSummary = environmentState.sidebarThreadSummaryById[threadId];
        const currentSession = environmentState.threadSessionById[threadId];
        return {
          ...state,
          environmentStateById: {
            ...state.environmentStateById,
            [environmentId]: {
              ...environmentState,
              threadShellById: {
                ...environmentState.threadShellById,
                [threadId]: currentShell
                  ? { ...currentShell, interactionMode: "default" }
                  : currentShell,
              },
              threadSessionById: {
                ...environmentState.threadSessionById,
                [threadId]: currentSession
                  ? { ...currentSession, status: "running", orchestrationStatus: "running" }
                  : currentSession,
              },
              activityIdsByThreadId: {
                ...environmentState.activityIdsByThreadId,
                [threadId]: activities.map((activity) => activity.id),
              },
              activityByThreadId: {
                ...environmentState.activityByThreadId,
                [threadId]: activityById,
              },
              sidebarThreadSummaryById: {
                ...environmentState.sidebarThreadSummaryById,
                [threadId]: currentSummary
                  ? {
                      ...currentSummary,
                      interactionMode: "default",
                      hasPendingApprovals: true,
                      session: currentSummary.session
                        ? {
                            ...currentSummary.session,
                            status: "running",
                            orchestrationStatus: "running",
                          }
                        : currentSummary.session,
                    }
                  : currentSummary,
              },
            },
          },
        };
      });
    });
  }
  async function seedRunningTurnReference() {
    await seedActiveChatReference();
    await page.evaluate(async () => {
      const { useStore } = await import("/src/store.ts");
      const environmentId = "local";
      const threadId = "thread-r3code-ui-shell";
      const turnId = "turn-running-1";
      const latestTurn = {
        turnId,
        state: "running",
        requestedAt: "2026-03-04T12:10:00.000Z",
        startedAt: "2026-03-04T12:10:01.000Z",
        completedAt: null,
        assistantMessageId: null,
      };
      const message = {
        id: "msg-user-running-turn",
        role: "user",
        text: "Run the parity harness and fix any failures.",
        turnId,
        streaming: false,
        createdAt: "2026-03-04T12:10:00.000Z",
        updatedAt: "2026-03-04T12:10:00.000Z",
      };
      const activities = [
        {
          id: "activity-thinking",
          kind: "task.progress",
          summary: "Inspecting changed surfaces",
          tone: "thinking",
          payload: {
            summary: "Inspecting changed surfaces",
            detail: "Reading upstream MessagesTimeline work log behavior",
          },
          turnId,
          sequence: 1,
          createdAt: "2026-03-04T12:10:02.000Z",
        },
        {
          id: "activity-command",
          kind: "tool.completed",
          summary: "Ran command",
          tone: "tool",
          payload: {
            command: "cargo test --workspace",
            title: "terminal",
            itemType: "command_execution",
            toolCallId: "tool-run-tests",
          },
          turnId,
          sequence: 2,
          createdAt: "2026-03-04T12:10:08.000Z",
        },
        {
          id: "activity-files",
          kind: "tool.completed",
          summary: "Edited files",
          tone: "tool",
          payload: {
            changedFiles: ["crates/r3_core/src/lib.rs", "crates/r3_ui/src/shell.rs"],
            title: "file change",
            itemType: "file_change",
            toolCallId: "tool-edit-files",
          },
          turnId,
          sequence: 3,
          createdAt: "2026-03-04T12:10:14.000Z",
        },
      ];
      const activityById = Object.fromEntries(activities.map((activity) => [activity.id, activity]));
      useStore.setState((state) => {
        const environmentState = state.environmentStateById[environmentId];
        if (!environmentState) return state;
        const currentShell = environmentState.threadShellById[threadId];
        const currentSummary = environmentState.sidebarThreadSummaryById[threadId];
        const currentSession = environmentState.threadSessionById[threadId];
        const nextSession = currentSession
          ? {
              ...currentSession,
              status: "running",
              orchestrationStatus: "running",
              activeTurnId: turnId,
            }
          : currentSession;
        return {
          ...state,
          environmentStateById: {
            ...state.environmentStateById,
            [environmentId]: {
              ...environmentState,
              threadShellById: {
                ...environmentState.threadShellById,
                [threadId]: currentShell ? { ...currentShell, latestTurn } : currentShell,
              },
              threadSessionById: {
                ...environmentState.threadSessionById,
                [threadId]: nextSession,
              },
              threadTurnStateById: {
                ...environmentState.threadTurnStateById,
                [threadId]: { latestTurn },
              },
              messageIdsByThreadId: {
                ...environmentState.messageIdsByThreadId,
                [threadId]: [message.id],
              },
              messageByThreadId: {
                ...environmentState.messageByThreadId,
                [threadId]: { [message.id]: message },
              },
              activityIdsByThreadId: {
                ...environmentState.activityIdsByThreadId,
                [threadId]: activities.map((activity) => activity.id),
              },
              activityByThreadId: {
                ...environmentState.activityByThreadId,
                [threadId]: activityById,
              },
              turnDiffIdsByThreadId: {
                ...environmentState.turnDiffIdsByThreadId,
                [threadId]: [],
              },
              turnDiffSummaryByThreadId: {
                ...environmentState.turnDiffSummaryByThreadId,
                [threadId]: {},
              },
              sidebarThreadSummaryById: {
                ...environmentState.sidebarThreadSummaryById,
                [threadId]: currentSummary
                  ? {
                      ...currentSummary,
                      session: nextSession,
                      latestTurn,
                      latestUserMessageAt: message.createdAt,
                    }
                  : currentSummary,
              },
            },
          },
        };
      });
    });
  }
  async function seedTerminalDrawerReference() {
    await seedActiveChatReference();
    await page.evaluate(async () => {
      const { useTerminalStateStore } = await import("/src/terminalStateStore.ts");
      const { __setEnvironmentApiOverrideForTests, readEnvironmentApi } = await import("/src/environmentApi.ts");
      const environmentId = "local";
      const threadId = "thread-r3code-ui-shell";
      const threadKey = `${environmentId}:${threadId}`;
      const cwd = "C:\\Users\\bunny\\Downloads\\r3code";
      const snapshots = {
        default: {
          threadId,
          terminalId: "default",
          cwd,
          worktreePath: null,
          status: "running",
          pid: 24012,
          history: "PS C:\\Users\\bunny\\Downloads\\r3code> cargo check --workspace\r\n",
          exitCode: null,
          exitSignal: null,
          updatedAt: "2026-03-04T12:00:14.000Z",
        },
        "terminal-2": {
          threadId,
          terminalId: "terminal-2",
          cwd,
          worktreePath: null,
          status: "running",
          pid: 24028,
          history: "Running upstream capture fixture...\r\n",
          exitCode: null,
          exitSignal: null,
          updatedAt: "2026-03-04T12:00:14.000Z",
        },
      };
      const existingApi = readEnvironmentApi(environmentId);
      __setEnvironmentApiOverrideForTests(environmentId, {
        ...(existingApi ?? {}),
        terminal: {
          async open(input) {
            const terminalId = input?.terminalId === "terminal-2" ? "terminal-2" : "default";
            return snapshots[terminalId];
          },
          async write() {},
          async resize() {},
          async clear() {},
          async restart(input) {
            const terminalId = input?.terminalId === "terminal-2" ? "terminal-2" : "default";
            return snapshots[terminalId];
          },
          async close() {},
          onEvent() {
            return () => undefined;
          },
        },
      });
      useTerminalStateStore.setState({
        terminalStateByThreadKey: {
          [threadKey]: {
            terminalOpen: true,
            terminalHeight: 280,
            terminalIds: ["default", "terminal-2"],
            runningTerminalIds: ["terminal-2"],
            activeTerminalId: "terminal-2",
            terminalGroups: [{ id: "group-default", terminalIds: ["default", "terminal-2"] }],
            activeTerminalGroupId: "group-default",
          },
        },
        terminalLaunchContextByThreadKey: {
          [threadKey]: {
            cwd,
            worktreePath: null,
          },
        },
        terminalEventEntriesByKey: {},
        nextTerminalEventId: 1,
      });
    });
  }
  async function seedDiffPanelReference() {
    await seedActiveChatReference();
    await page.evaluate(async () => {
      const { __setEnvironmentApiOverrideForTests, readEnvironmentApi } = await import("/src/environmentApi.ts");
      const environmentId = "local";
      const threadId = "thread-r3code-ui-shell";
      const patch = [
        "diff --git a/crates/r3_core/src/lib.rs b/crates/r3_core/src/lib.rs",
        "index c2b4d10..f4ab233 100644",
        "--- a/crates/r3_core/src/lib.rs",
        "+++ b/crates/r3_core/src/lib.rs",
        "@@ -10,6 +10,7 @@ pub struct ThreadTerminalState {",
        "     pub terminal_open: bool,",
        "+    pub active_terminal_group_id: String,",
        " }",
        "diff --git a/crates/r3_ui/src/shell.rs b/crates/r3_ui/src/shell.rs",
        "index 5a4a1b3..b5d7c91 100644",
        "--- a/crates/r3_ui/src/shell.rs",
        "+++ b/crates/r3_ui/src/shell.rs",
        "@@ -42,7 +42,8 @@ fn render_terminal_drawer() {",
        "-    draw_static_terminal();",
        "+    draw_split_terminal();",
        "+    draw_terminal_sidebar();",
        " }",
      ].join("\n");
      const existingApi = readEnvironmentApi(environmentId);
      __setEnvironmentApiOverrideForTests(environmentId, {
        ...(existingApi ?? {}),
        orchestration: {
          ...(existingApi?.orchestration ?? {}),
          async getTurnDiff(input) {
            return {
              threadId,
              fromTurnCount: input.fromTurnCount,
              toTurnCount: input.toTurnCount,
              diff: patch,
            };
          },
          async getFullThreadDiff(input) {
            return {
              threadId,
              fromTurnCount: 0,
              toTurnCount: input.toTurnCount,
              diff: patch,
            };
          },
        },
      });
    });
  }
  async function seedBranchToolbarReference() {
    await page.evaluate(async () => {
      const { useComposerDraftStore } = await import("/src/composerDraftStore.ts");
      const { __setEnvironmentApiOverrideForTests, readEnvironmentApi } = await import("/src/environmentApi.ts");
      const draftId = window.location.pathname.split("/").filter(Boolean).pop();
      if (!draftId) throw new Error("Unable to resolve current draft id.");
      const draftSession = useComposerDraftStore.getState().getDraftSession(draftId);
      if (!draftSession) throw new Error(`Unable to resolve draft session ${draftId}.`);
      const { useStore } = await import("/src/store.ts");
      const environmentId = draftSession.environmentId;
      const environmentState = useStore.getState().environmentStateById[environmentId];
      const projectId = Object.keys(environmentState?.projectById ?? {})[0];
      if (!projectId) throw new Error(`Unable to resolve active project for ${environmentId}.`);
      const cwd = "C:\\Users\\bunny\\Downloads\\r3code";
      const refs = [
        {
          name: "main",
          current: true,
          isDefault: true,
          isRemote: false,
          remoteName: undefined,
          worktreePath: null,
        },
        {
          name: "feature/parity-branch-toolbar",
          current: false,
          isDefault: false,
          isRemote: false,
          remoteName: undefined,
          worktreePath: "C:\\Users\\bunny\\Downloads\\r3code\\.t3\\worktrees\\branch-toolbar",
        },
        {
          name: "origin/main",
          current: false,
          isDefault: true,
          isRemote: true,
          remoteName: "origin",
          worktreePath: null,
        },
        {
          name: "origin/feature/remote-only",
          current: false,
          isDefault: false,
          isRemote: true,
          remoteName: "origin",
          worktreePath: null,
        },
      ];
      const status = {
        isRepo: true,
        sourceControlProvider: { kind: "git", displayName: "Git" },
        hasPrimaryRemote: true,
        isDefaultRef: true,
        refName: "main",
        hasWorkingTreeChanges: false,
        workingTree: { files: [], insertions: 0, deletions: 0 },
        hasUpstream: true,
        aheadCount: 0,
        behindCount: 0,
        aheadOfDefaultCount: 0,
        pr: null,
      };
      const existingApi = readEnvironmentApi(environmentId);
      __setEnvironmentApiOverrideForTests(environmentId, {
        ...(existingApi ?? {}),
        vcs: {
          ...(existingApi?.vcs ?? {}),
          async listRefs() {
            return {
              isRepo: true,
              hasPrimaryRemote: true,
              nextCursor: null,
              totalCount: refs.length,
              refs,
            };
          },
          async refreshStatus() {
            return status;
          },
          onStatus(_input, callback) {
            callback(status);
            return () => undefined;
          },
        },
      });
      useComposerDraftStore.getState().setDraftThreadContext(draftId, {
        projectRef: { environmentId, projectId },
        branch: null,
        worktreePath: null,
        envMode: "worktree",
      });
      useComposerDraftStore.setState((state) => ({
        draftThreadsByThreadKey: {
          ...state.draftThreadsByThreadKey,
          [draftId]: {
            ...state.draftThreadsByThreadKey[draftId],
            environmentId,
            projectId,
            logicalProjectKey: `${environmentId}:${projectId}`,
            branch: null,
            worktreePath: null,
            envMode: "worktree",
          },
        },
      }));
    });
  }
  await page.goto(process.env.PAIRING_URL, { waitUntil: "domcontentloaded", timeout: 30000 });
  await page.getByText("Pick a thread to continue").waitFor({ timeout: 15000 });
  await page.waitForLoadState("networkidle", { timeout: 30000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-empty-reference.png"), fullPage: true });
  await page.getByTestId("command-palette-trigger").click();
  await page.getByPlaceholder("Search commands, projects, and threads...").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-command-palette-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.getByTestId("command-palette-trigger").click();
  const palette = page.getByTestId("command-palette");
  await palette.getByText("Add project", { exact: true }).click();
  await palette.getByText("Local folder", { exact: true }).click();
  const addProjectPlaceholder = "Enter path (e.g. ~/projects/my-app)";
  await page.getByPlaceholder(addProjectPlaceholder).fill(process.env.REFERENCE_PROJECT_PATH);
  await palette.getByRole("button", { name: "Add (Enter)" }).waitFor({ timeout: 15000 });
  await page.keyboard.press("Enter");
  await page.waitForURL(/\/draft\/[^/]+$/, { timeout: 30000 });
  await page.getByText("Send a message to start the conversation.").waitFor({ timeout: 30000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-draft-reference.png"), fullPage: true });
  await page.getByTestId("composer-editor").click();
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-composer-focused-reference.png"), fullPage: true });
  await page.getByTestId("composer-editor").fill("/");
  await page.locator('[data-composer-item-id="slash:model"]').waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-composer-menu-reference.png"), fullPage: true });
  await page.getByTestId("composer-editor").fill("");
  await page.locator('[data-composer-item-id="slash:model"]').waitFor({ state: "detached", timeout: 15000 });
  await page.evaluate(async () => {
    const { useComposerDraftStore } = await import("/src/composerDraftStore.ts");
    const draftId = window.location.pathname.split("/").filter(Boolean).pop();
    if (!draftId) throw new Error("Unable to resolve current draft id.");
    useComposerDraftStore.getState().setPrompt(draftId, "use @AGENTS.md and $agent-browser ");
  });
  await page.locator('[data-composer-mention-chip="true"]').waitFor({ timeout: 15000 });
  await page.locator('[data-composer-skill-chip="true"]').waitFor({ timeout: 15000 });
  await page.waitForFunction(() => {
    const icon = document.querySelector('[data-composer-mention-chip="true"] img');
    return !icon || icon.complete;
  }, undefined, { timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-composer-inline-tokens-reference.png"), fullPage: true });
  await page.evaluate(async () => {
    const { useComposerDraftStore } = await import("/src/composerDraftStore.ts");
    const draftId = window.location.pathname.split("/").filter(Boolean).pop();
    if (draftId) useComposerDraftStore.getState().setPrompt(draftId, "");
  });
  await page.locator('[data-chat-provider-model-picker="true"]').click();
  await page.locator(".model-picker-list").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-provider-model-picker-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.locator(".model-picker-list").waitFor({ state: "detached", timeout: 15000 }).catch(() => undefined);
  await seedBranchToolbarReference();
  await page.getByText("New worktree", { exact: true }).waitFor({ timeout: 15000 });
  await page.getByText("From main", { exact: true }).waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-branch-toolbar-reference.png"), fullPage: true });
  await page.locator('[data-slot="sidebar-group"]').filter({ has: page.getByText("Projects", { exact: true }) }).locator("button").first().click();
  await page.getByText("Sort projects", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("Visible threads", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("Group projects", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-sidebar-options-menu-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.locator('[data-slot="menu-popup"]').last().waitFor({ state: "detached", timeout: 15000 }).catch(() => undefined);
  await page.mouse.click(500, 500);
  await page.getByRole("button", { name: "Copy options" }).last().click();
  await page.locator('[data-slot="menu-popup"]').last().waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-open-in-menu-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Git action options" }).last().click();
  await page.locator('[data-slot="menu-popup"]').last().waitFor({ timeout: 15000 });
  await page.getByText("Commit", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("Push", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("Create PR", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-git-actions-menu-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedActiveChatReference();
  await page.getByRole("heading", { name: "Port R3Code UI shell" }).waitFor({ timeout: 15000 });
  await page.getByText("Make the Rust port match the original UI exactly.").waitFor({ timeout: 15000 });
  await page.getByText("Building a static GPUI shell first, then replacing mock data with Rust state.").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-active-chat-reference.png"), fullPage: true });
  await page.getByRole("button", { name: "Script actions" }).click();
  await page.locator('[data-slot="menu-popup"]').last().waitFor({ timeout: 15000 });
  await page.getByText("test", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("parity", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.getByText("Add action", { exact: true }).last().waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-project-scripts-menu-reference.png"), fullPage: true });
  await page.keyboard.press("Escape");
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedRunningTurnReference();
  await page.getByText("Run the parity harness and fix any failures.").waitFor({ timeout: 15000 });
  await page.getByText("Inspecting changed surfaces").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-running-turn-reference.png"), fullPage: true });
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedTerminalDrawerReference();
  await page.locator(".thread-terminal-drawer").waitFor({ timeout: 15000 });
  await page.waitForTimeout(900);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-terminal-drawer-reference.png"), fullPage: true });
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await page.goto(new URL("/local/thread-r3code-ui-shell?diff=1&diffTurnId=turn-r3code-ui-shell-2&diffFilePath=crates/r3_ui/src/shell.rs", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedDiffPanelReference();
  await page.getByText("Turn 2").waitFor({ timeout: 15000 });
  await page.getByText("crates/r3_ui/src/shell.rs").waitFor({ timeout: 15000 });
  await page.waitForTimeout(900);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-diff-panel-reference.png"), fullPage: true });
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedPendingUserInputReference();
  await page.getByText("What should this change cover?").waitFor({ timeout: 15000 });
  await page.getByText("Tight").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-pending-user-input-reference.png"), fullPage: true });
  await page.goto(new URL("/local/thread-r3code-ui-shell", appOrigin).toString(), { waitUntil: "domcontentloaded", timeout: 30000 });
  await seedPendingApprovalReference();
  await page.locator('[data-chat-composer-form="true"]').getByText("PENDING APPROVAL", { exact: true }).waitFor({ timeout: 15000 });
  await page.getByText("Command approval requested").waitFor({ timeout: 15000 });
  await page.getByRole("button", { name: "Approve once" }).waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-pending-approval-reference.png"), fullPage: true });
  await page.goto(new URL("/settings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByLabel("Theme preference").waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/keybindings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("Command").first().waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-keybindings-reference.png"), fullPage: true });
  await page.getByLabel("Add keybinding").click();
  await page.getByLabel("Cancel new keybinding").waitFor({ timeout: 15000 });
  await page.waitForTimeout(350);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-keybindings-add-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/providers", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByLabel("Refresh provider status").waitFor({ timeout: 15000 });
  await page.waitForTimeout(500);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-providers-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/source-control", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("VERSION CONTROL").first().waitFor({ timeout: 15000 });
  await page.waitForTimeout(1500);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-source-control-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/connections", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("Manage local backend").first().waitFor({ timeout: 15000 });
  await page.waitForTimeout(1000);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-connections-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/diagnostics", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("Live Processes").first().waitFor({ timeout: 15000 });
  await page.getByText("Trace Diagnostics").first().waitFor({ timeout: 15000 });
  await page.waitForTimeout(1500);
  await dismissUpdatesToast();
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-diagnostics-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/archived", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("No archived threads").first().waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-archive-reference.png"), fullPage: true });
  await page.goto(new URL("/settings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByLabel("Theme preference").waitFor({ timeout: 15000 });
  await page.getByLabel("Theme preference").click();
  await page.getByRole("option", { name: "Light" }).click();
  await page.waitForFunction(() => !document.documentElement.classList.contains("dark"), undefined, { timeout: 15000 });
  await page.getByLabel("Theme preference").click();
  await page.getByRole("option", { name: "Light" }).waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-theme-menu-reference.png"), fullPage: true });
  await page.getByRole("option", { name: "Dark" }).click();
  await page.waitForFunction(() => document.documentElement.classList.contains("dark"), undefined, { timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-dark-reference.png"), fullPage: true });
  await page.goto(appOrigin, { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("Pick a thread to continue").waitFor({ timeout: 15000 });
  await page.waitForFunction(() => document.documentElement.classList.contains("dark"), undefined, { timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-empty-dark-reference.png"), fullPage: true });
  await browser.close();
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
"#
}

fn command_stdout(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("command failed with {}: {command:?}", output.status).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn kill_process_tree(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

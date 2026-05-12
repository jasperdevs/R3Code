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
  cargo run -p xtask -- capture-r3code-window --allow-window-capture [--screen draft|composer-menu|composer-inline-tokens|active-chat|running-turn|pending-approval|pending-user-input|terminal-drawer|diff-panel|branch-toolbar|provider-model-picker|settings|settings-diagnostics|command-palette|settings-theme-menu|settings-dark|settings-back|settings-keybindings|settings-providers|settings-source-control|settings-connections|settings-archive] [--theme light|dark|system] [--output <png>]
  cargo run -p xtask -- capture-reference-browser"
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
        screen: Some("active-chat".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-active-chat-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
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
        screen: Some("running-turn".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-running-turn-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("pending-approval".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-pending-approval-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("pending-user-input".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-pending-user-input-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("terminal-drawer".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-terminal-drawer-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("diff-panel".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-diff-panel-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
    })?;

    capture_r3code_window(CaptureR3CodeOptions {
        screen: Some("branch-toolbar".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-branch-toolbar-window.png"),
        allow_window_capture: true,
        ..CaptureR3CodeOptions::default()
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
        max_different_pixels_percent: 9.0,
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
                "Upstream reference repository: {}\nReference commit: {}\nIsolated reference home: {}\nOutput directory: {}\nCaptured:\n- upstream-empty-reference.png\n- upstream-command-palette-reference.png\n- upstream-draft-reference.png\n- upstream-composer-menu-reference.png\n- upstream-composer-inline-tokens-reference.png\n- upstream-provider-model-picker-reference.png\n- upstream-settings-reference.png\n- upstream-settings-keybindings-reference.png\n- upstream-settings-providers-reference.png\n- upstream-settings-source-control-reference.png\n- upstream-settings-connections-reference.png\n- upstream-settings-diagnostics-reference.png\n- upstream-settings-archive-reference.png\n- upstream-settings-theme-menu-reference.png\n- upstream-settings-dark-reference.png\n",
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
  await page.goto(new URL("/settings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByLabel("Theme preference").waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-reference.png"), fullPage: true });
  await page.goto(new URL("/settings/keybindings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByText("Command").first().waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "upstream-settings-keybindings-reference.png"), fullPage: true });
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

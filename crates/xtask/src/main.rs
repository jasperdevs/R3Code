use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use image::{ImageBuffer, Rgba};

#[cfg(windows)]
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, ClientToScreen, CreateCompatibleBitmap,
        CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HGDIOBJ,
        ReleaseDC, SRCCOPY, SelectObject,
    },
    UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
        VIRTUAL_KEY, VK_CONTROL, VK_DOWN, VK_K, VK_RETURN, VK_T,
    },
    UI::WindowsAndMessaging::{
        BringWindowToTop, EnumWindows, GetClientRect, GetWindowThreadProcessId, HWND_TOP,
        IsWindowVisible, SW_RESTORE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SetForegroundWindow,
        SetWindowPos, ShowWindow,
    },
};
#[cfg(windows)]
use windows::core::BOOL;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

#[derive(Debug)]
struct CaptureR3CodeOptions {
    exe: PathBuf,
    output: PathBuf,
    screen: Option<String>,
    theme: Option<String>,
    delay: Duration,
}

#[derive(Debug)]
struct CaptureT3CodeOptions {
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
        "check-parity" => check_parity(args.iter().any(|arg| arg == "--refresh-t3code-reference")),
        "compare-screenshots" => compare_screenshots(parse_compare_options(&args)?),
        "capture-r3code-window" => capture_r3code_window(parse_capture_r3code_options(&args)?),
        "capture-t3code-browser" => capture_t3code_browser(parse_capture_t3code_options(&args)?),
        _ => {
            print_usage();
            Err(format!("unknown xtask command: {command}").into())
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  cargo run -p xtask -- check-parity [--refresh-t3code-reference]
  cargo run -p xtask -- compare-screenshots --expected <png> --actual <png> [--channel-tolerance <n>] [--ignore-rect x,y,w,h] [--max-different-pixels-percent <n>]
  cargo run -p xtask -- capture-r3code-window [--screen settings|command-palette|settings-theme-menu|settings-dark] [--theme light|dark|system] [--output <png>]
  cargo run -p xtask -- capture-t3code-browser"
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

fn check_parity(refresh_t3code_reference: bool) -> Result<()> {
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

    if refresh_t3code_reference {
        capture_t3code_browser(CaptureT3CodeOptions::default())?;
    }

    capture_r3code_window(CaptureR3CodeOptions {
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-window.png"),
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/t3code-empty-reference.png"),
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
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/t3code-command-palette-reference.png"),
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
        screen: Some("settings".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-window.png"),
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/t3code-settings-reference.png"),
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
        screen: Some("settings-theme-menu".to_string()),
        theme: Some("light".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-settings-theme-menu-window.png"),
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path(
            "reference/screenshots/t3code-settings-theme-menu-reference.png",
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
        ..CaptureR3CodeOptions::default()
    })?;
    compare_screenshots(CompareOptions {
        expected: resolve_repo_path("reference/screenshots/t3code-settings-dark-reference.png"),
        actual: resolve_repo_path("reference/screenshots/r3code-settings-dark-window.png"),
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
        theme: Some("dark".to_string()),
        output: resolve_repo_path("reference/screenshots/r3code-dark-window.png"),
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
            other => return Err(format!("unknown capture-r3code-window option: {other}").into()),
        }
        index += 1;
    }
    Ok(options)
}

fn parse_capture_t3code_options(args: &[String]) -> Result<CaptureT3CodeOptions> {
    let mut options = CaptureT3CodeOptions::default();
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
            other => return Err(format!("unknown capture-t3code-browser option: {other}").into()),
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
        }
    }
}

impl Default for CaptureT3CodeOptions {
    fn default() -> Self {
        let temp = env::temp_dir();
        Self {
            repo: temp.join("t3code-inspect"),
            home: temp.join("t3code-reference-home"),
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
fn capture_r3code_window(options: CaptureR3CodeOptions) -> Result<()> {
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
            "settings-theme-menu" | "settings-dark" => {
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
    thread::sleep(options.delay);

    let result = (|| -> Result<()> {
        let hwnd = find_window_for_pid(child.id())?;
        prepare_window_for_capture(hwnd);
        if options.screen.as_deref() == Some("command-palette") {
            send_command_palette_shortcut()?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-theme-menu") {
            send_settings_theme_menu_shortcut()?;
            thread::sleep(Duration::from_millis(350));
        } else if options.screen.as_deref() == Some("settings-dark") {
            send_settings_dark_shortcut()?;
            thread::sleep(Duration::from_millis(350));
        }
        let image = capture_client_area(hwnd)?;
        image.save(&options.output)?;
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
fn send_command_palette_shortcut() -> Result<()> {
    send_key_sequence(&[VK_CONTROL, VK_K])
}

#[cfg(windows)]
fn send_settings_theme_menu_shortcut() -> Result<()> {
    send_key_sequence(&[VK_CONTROL, VK_T])
}

#[cfg(windows)]
fn send_settings_dark_shortcut() -> Result<()> {
    send_settings_theme_menu_shortcut()?;
    thread::sleep(Duration::from_millis(100));
    send_key_tap(VK_DOWN)?;
    thread::sleep(Duration::from_millis(60));
    send_key_tap(VK_RETURN)
}

#[cfg(windows)]
fn send_key_sequence(keys: &[VIRTUAL_KEY]) -> Result<()> {
    let mut inputs = Vec::with_capacity(keys.len() * 2);
    for key in keys {
        inputs.push(keyboard_input(*key, KEYBD_EVENT_FLAGS(0)));
    }
    for key in keys.iter().rev() {
        inputs.push(keyboard_input(*key, KEYEVENTF_KEYUP));
    }
    send_inputs(&inputs, "key")
}

#[cfg(windows)]
fn send_key_tap(key: VIRTUAL_KEY) -> Result<()> {
    let inputs = [
        keyboard_input(key, KEYBD_EVENT_FLAGS(0)),
        keyboard_input(key, KEYEVENTF_KEYUP),
    ];
    send_inputs(&inputs, "key")
}

#[cfg(windows)]
fn send_inputs(inputs: &[INPUT], input_kind: &str) -> Result<()> {
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err(format!("SendInput sent {sent}/{} {input_kind} events", inputs.len()).into())
    }
}

#[cfg(windows)]
fn keyboard_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(windows)]
fn find_window_for_pid(pid: u32) -> Result<HWND> {
    struct Search {
        pid: u32,
        hwnd: HWND,
    }

    unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut Search) };
        let mut window_pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        }
        if window_pid == search.pid && unsafe { IsWindowVisible(hwnd).as_bool() } {
            search.hwnd = hwnd;
            return BOOL(0);
        }
        BOOL(1)
    }

    let mut search = Search {
        pid,
        hwnd: HWND(std::ptr::null_mut()),
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

        let mut origin = POINT { x: 0, y: 0 };
        if !ClientToScreen(hwnd, &mut origin).as_bool() {
            return Err("ClientToScreen failed".into());
        }

        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            return Err("GetDC failed".into());
        }
        let memory_dc = CreateCompatibleDC(Some(screen_dc));
        if memory_dc.is_invalid() {
            let _ = ReleaseDC(None, screen_dc);
            return Err("CreateCompatibleDC failed".into());
        }
        let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(None, screen_dc);
            return Err("CreateCompatibleBitmap failed".into());
        }

        let old_object = SelectObject(memory_dc, HGDIOBJ(bitmap.0));
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
        if copied.is_err() {
            let _ = SelectObject(memory_dc, old_object);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(None, screen_dc);
            return Err("BitBlt failed".into());
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
        let _ = ReleaseDC(None, screen_dc);

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

fn capture_t3code_browser(options: CaptureT3CodeOptions) -> Result<()> {
    if !options.repo.exists() {
        run(Command::new("git").args([
            "clone",
            "--depth=1",
            "https://github.com/pingdotgg/t3code.git",
            options.repo.to_string_lossy().as_ref(),
        ]))?;
    }

    fs::create_dir_all(&options.home)?;
    fs::create_dir_all(&options.output_dir)?;

    let playwright_path = options
        .repo
        .join("node_modules")
        .join(".bun")
        .join("node_modules")
        .join("playwright");
    if !playwright_path.exists() {
        run(Command::new("bun")
            .arg("install")
            .current_dir(&options.repo))?;
    }

    let commit = command_stdout(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&options.repo),
    )?;
    let stdout_path = env::temp_dir().join("t3code-reference.out.log");
    let stderr_path = env::temp_dir().join("t3code-reference.err.log");
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);

    let stdout = fs::File::create(&stdout_path)?;
    let stderr = fs::File::create(&stderr_path)?;
    let mut child = Command::new("bun")
        .args(["run", "dev", "--no-browser"])
        .current_dir(&options.repo)
        .env("T3CODE_HOME", &options.home)
        .env("T3CODE_NO_BROWSER", "1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;

    let result = (|| -> Result<()> {
        let pairing_url = wait_for_pairing_url(
            &mut child,
            &stdout_path,
            &stderr_path,
            options.startup_timeout,
        )?;
        let script_path = env::temp_dir().join("capture-t3code-browser.cjs");
        fs::write(&script_path, browser_capture_script())?;
        run(Command::new("node")
            .arg(&script_path)
            .env("PLAYWRIGHT_PATH", &playwright_path)
            .env("PAIRING_URL", pairing_url)
            .env("OUTPUT_DIR", &options.output_dir))?;

        fs::write(
            options.output_dir.join("CAPTURE_MANIFEST.txt"),
            format!(
                "T3Code reference repository: {}\nReference commit: {}\nIsolated T3CODE_HOME: {}\nOutput directory: {}\nCaptured:\n- t3code-empty-reference.png\n- t3code-command-palette-reference.png\n- t3code-settings-reference.png\n- t3code-settings-theme-menu-reference.png\n- t3code-settings-dark-reference.png\n",
                options.repo.display(),
                commit.trim(),
                options.home.display(),
                options.output_dir.display()
            ),
        )?;
        println!(
            "Captured T3Code reference screenshots from {}",
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
                "T3Code dev exited before pairing URL was available. Exit={status}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
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

    Err("timed out waiting for T3Code pairing URL".into())
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
  await page.goto(process.env.PAIRING_URL, { waitUntil: "domcontentloaded", timeout: 30000 });
  await page.getByText("Pick a thread to continue").waitFor({ timeout: 15000 });
  await page.waitForLoadState("networkidle", { timeout: 30000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-empty-reference.png"), fullPage: true });
  await page.getByTestId("command-palette-trigger").click();
  await page.getByPlaceholder("Search commands, projects, and threads...").waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-command-palette-reference.png"), fullPage: true });
  await page.goto(new URL("/settings", appOrigin).toString(), { waitUntil: "networkidle", timeout: 30000 });
  await page.getByLabel("Theme preference").waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-settings-reference.png"), fullPage: true });
  await page.getByLabel("Theme preference").click();
  await page.getByRole("option", { name: "Light" }).click();
  await page.waitForFunction(() => !document.documentElement.classList.contains("dark"), undefined, { timeout: 15000 });
  await page.getByLabel("Theme preference").click();
  await page.getByRole("option", { name: "Light" }).waitFor({ timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-settings-theme-menu-reference.png"), fullPage: true });
  await page.getByRole("option", { name: "Dark" }).click();
  await page.waitForFunction(() => document.documentElement.classList.contains("dark"), undefined, { timeout: 15000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-settings-dark-reference.png"), fullPage: true });
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

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::EditorId;

pub const DEFAULT_MAX_BUFFER_BYTES: usize = 8 * 1024 * 1024;

const WINDOWS_COMMAND_NOT_FOUND_PATTERNS: &[&str] = &[
    "is not recognized as an internal or external command",
    "n.o . reconhecido como um comando interno",
    "non . riconosciuto come comando interno o esterno",
    "n.est pas reconnu en tant que commande interne",
    "no se reconoce como un comando interno o externo",
    "wird nicht als interner oder externer befehl",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOutputMode {
    Error,
    Truncate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRunOptions {
    pub cwd: Option<PathBuf>,
    pub timeout_ms: u64,
    pub env: Option<BTreeMap<String, String>>,
    pub stdin: Option<String>,
    pub allow_non_zero_exit: bool,
    pub max_buffer_bytes: usize,
    pub output_mode: ProcessOutputMode,
}

impl Default for ProcessRunOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            timeout_ms: 60_000,
            env: None,
            stdin: None,
            allow_non_zero_exit: false,
            max_buffer_bytes: DEFAULT_MAX_BUFFER_BYTES,
            output_mode: ProcessOutputMode::Error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRunResult {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
    pub signal: Option<String>,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRunError {
    pub message: String,
}

impl std::fmt::Display for ProcessRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ProcessRunError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorLaunchStyle {
    DirectPath,
    Goto,
    LineColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDefinition {
    pub id: EditorId,
    pub id_slug: &'static str,
    pub label: &'static str,
    pub commands: Option<&'static [&'static str]>,
    pub base_args: &'static [&'static str],
    pub launch_style: EditorLaunchStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenInEditorInput {
    pub cwd: String,
    pub editor: EditorId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLaunch {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessLaunchOptions {
    pub detached: bool,
    pub shell: bool,
    pub stdin: &'static str,
    pub stdout: &'static str,
    pub stderr: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub options: ProcessLaunchOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenError {
    pub message: String,
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OpenError {}

pub fn run_process(
    command: &str,
    args: &[&str],
    options: ProcessRunOptions,
) -> Result<ProcessRunResult, ProcessRunError> {
    let mut cmd = spawn_command(command, args);
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env) = &options.env {
        cmd.env_clear();
        cmd.envs(env);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|error| normalize_spawn_error(command, args, error))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = stdout.map(read_pipe_on_thread);
    let stderr_reader = stderr.map(read_pipe_on_thread);

    if let Some(stdin) = &options.stdin {
        if let Some(mut child_stdin) = child.stdin.take() {
            child_stdin.write_all(stdin.as_bytes()).map_err(|error| {
                let _ = child.kill();
                normalize_stdin_error(command, args, error)
            })?;
        }
    }
    drop(child.stdin.take());

    let started_at = Instant::now();
    let timeout = Duration::from_millis(options.timeout_ms);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| normalize_spawn_error(command, args, error))?
        {
            break status;
        }
        if started_at.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| normalize_spawn_error(command, args, error))?;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout_bytes = stdout_reader.map(join_pipe_reader).unwrap_or_else(Vec::new);
    let stderr_bytes = stderr_reader.map(join_pipe_reader).unwrap_or_else(Vec::new);
    let stdout = apply_output_limit(command, args, "stdout", stdout_bytes, &options)?;
    let stderr = apply_output_limit(command, args, "stderr", stderr_bytes, &options)?;

    let result = ProcessRunResult {
        stdout: stdout.text,
        stderr: stderr.text,
        code: status.code(),
        signal: None,
        timed_out,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
    };

    if !options.allow_non_zero_exit && (timed_out || result.code.is_some_and(|code| code != 0)) {
        return Err(normalize_exit_error(command, args, &result));
    }
    Ok(result)
}

pub fn is_windows_command_not_found(platform: &str, code: Option<i32>, stderr: &str) -> bool {
    if platform != "win32" {
        return false;
    }
    if code == Some(9009) {
        return true;
    }
    let normalized = stderr.to_ascii_lowercase();
    WINDOWS_COMMAND_NOT_FOUND_PATTERNS
        .iter()
        .any(|pattern| normalized.contains(pattern))
}

pub fn command_label(command: &str, args: &[&str]) -> String {
    std::iter::once(command)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn editor_definitions() -> Vec<EditorDefinition> {
    vec![
        editor(
            EditorId::Cursor,
            "cursor",
            "Cursor",
            Some(&["cursor"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::Trae,
            "trae",
            "Trae",
            Some(&["trae"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::Kiro,
            "kiro",
            "Kiro",
            Some(&["kiro"]),
            &["ide"],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::VsCode,
            "vscode",
            "VS Code",
            Some(&["code"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::VsCodeInsiders,
            "vscode-insiders",
            "VS Code Insiders",
            Some(&["code-insiders"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::VsCodium,
            "vscodium",
            "VSCodium",
            Some(&["codium"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::Zed,
            "zed",
            "Zed",
            Some(&["zed", "zeditor"]),
            &[],
            EditorLaunchStyle::DirectPath,
        ),
        editor(
            EditorId::Antigravity,
            "antigravity",
            "Antigravity",
            Some(&["agy"]),
            &[],
            EditorLaunchStyle::Goto,
        ),
        editor(
            EditorId::Idea,
            "idea",
            "IntelliJ IDEA",
            Some(&["idea"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::Aqua,
            "aqua",
            "Aqua",
            Some(&["aqua"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::CLion,
            "clion",
            "CLion",
            Some(&["clion"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::DataGrip,
            "datagrip",
            "DataGrip",
            Some(&["datagrip"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::DataSpell,
            "dataspell",
            "DataSpell",
            Some(&["dataspell"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::GoLand,
            "goland",
            "GoLand",
            Some(&["goland"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::PhpStorm,
            "phpstorm",
            "PhpStorm",
            Some(&["phpstorm"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::PyCharm,
            "pycharm",
            "PyCharm",
            Some(&["pycharm"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::Rider,
            "rider",
            "Rider",
            Some(&["rider"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::RubyMine,
            "rubymine",
            "RubyMine",
            Some(&["rubymine"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::RustRover,
            "rustrover",
            "RustRover",
            Some(&["rustrover"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::WebStorm,
            "webstorm",
            "WebStorm",
            Some(&["webstorm"]),
            &[],
            EditorLaunchStyle::LineColumn,
        ),
        editor(
            EditorId::FileManager,
            "file-manager",
            "File Manager",
            None,
            &[],
            EditorLaunchStyle::DirectPath,
        ),
    ]
}

pub fn resolve_editor_launch(
    input: &OpenInEditorInput,
    platform: &str,
    env: &BTreeMap<String, String>,
) -> Result<EditorLaunch, OpenError> {
    let Some(editor) = editor_definitions()
        .into_iter()
        .find(|editor| editor.id == input.editor)
    else {
        return Err(OpenError {
            message: format!("Unknown editor: {:?}", input.editor),
        });
    };

    if let Some(commands) = editor.commands {
        let command = resolve_available_command(commands, platform, env)
            .unwrap_or_else(|| commands[0].to_string());
        return Ok(EditorLaunch {
            command,
            args: resolve_editor_args(&editor, &input.cwd),
        });
    }

    if editor.id != EditorId::FileManager {
        return Err(OpenError {
            message: format!("Unsupported editor: {}", editor.id_slug),
        });
    }

    Ok(EditorLaunch {
        command: file_manager_command_for_platform(platform).to_string(),
        args: vec![input.cwd.clone()],
    })
}

pub fn resolve_available_editors(platform: &str, env: &BTreeMap<String, String>) -> Vec<EditorId> {
    let mut available = Vec::new();
    for editor in editor_definitions() {
        if let Some(commands) = editor.commands {
            if resolve_available_command(commands, platform, env).is_some() {
                available.push(editor.id);
            }
        } else if is_command_available(file_manager_command_for_platform(platform), platform, env) {
            available.push(editor.id);
        }
    }
    available
}

pub fn is_command_available(command: &str, platform: &str, env: &BTreeMap<String, String>) -> bool {
    resolve_command_path(command, platform, env).is_some()
}

pub fn resolve_command_path(
    command: &str,
    platform: &str,
    env: &BTreeMap<String, String>,
) -> Option<PathBuf> {
    let windows_path_extensions = if platform == "win32" {
        resolve_windows_path_extensions(env)
    } else {
        Vec::new()
    };
    let command_candidates =
        resolve_command_candidates(command, platform, &windows_path_extensions);

    if command.contains('/') || command.contains('\\') {
        return command_candidates
            .into_iter()
            .map(PathBuf::from)
            .find(|candidate| is_executable_file(candidate, platform, &windows_path_extensions));
    }

    let path_value = resolve_path_environment_variable(env);
    if path_value.is_empty() {
        return None;
    }
    for path_entry in path_value
        .split(path_delimiter_for_platform(platform))
        .map(|entry| strip_wrapping_quotes(entry.trim()))
        .filter(|entry| !entry.is_empty())
    {
        for candidate in &command_candidates {
            let candidate_path = Path::new(&path_entry).join(candidate);
            if is_executable_file(&candidate_path, platform, &windows_path_extensions) {
                return Some(candidate_path);
            }
        }
    }
    None
}

pub fn file_manager_command_for_platform(platform: &str) -> &'static str {
    match platform {
        "darwin" => "open",
        "win32" => "explorer",
        _ => "xdg-open",
    }
}

pub fn detached_ignore_stdio_options(shell: bool) -> ProcessLaunchOptions {
    ProcessLaunchOptions {
        detached: true,
        shell,
        stdin: "ignore",
        stdout: "ignore",
        stderr: "ignore",
    }
}

pub fn resolve_browser_launch(
    target: &str,
    platform: &str,
    env: &BTreeMap<String, String>,
) -> ProcessLaunch {
    if platform == "darwin" {
        return ProcessLaunch {
            command: "open".to_string(),
            args: vec![target.to_string()],
            options: detached_ignore_stdio_options(false),
        };
    }

    if platform == "win32" {
        return resolve_windows_browser_launch(target, &resolve_powershell_path(env));
    }

    if should_use_windows_browser_from_wsl(platform, env) {
        return resolve_windows_browser_launch(target, resolve_wsl_powershell_path());
    }

    ProcessLaunch {
        command: "xdg-open".to_string(),
        args: vec![target.to_string()],
        options: detached_ignore_stdio_options(false),
    }
}

pub fn resolve_windows_browser_launch(target: &str, command: &str) -> ProcessLaunch {
    let encoded_command = encode_utf16_le_base64(&format!(
        "$ProgressPreference = 'SilentlyContinue'; Start {}",
        escape_powershell_string_literal(target)
    ));
    ProcessLaunch {
        command: command.to_string(),
        args: vec![
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-EncodedCommand".to_string(),
            encoded_command,
        ],
        options: detached_ignore_stdio_options(false),
    }
}

pub fn resolve_powershell_path(env: &BTreeMap<String, String>) -> String {
    format!(
        "{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
        env.get("SYSTEMROOT")
            .or_else(|| env.get("windir"))
            .map(String::as_str)
            .unwrap_or(r"C:\Windows")
    )
}

pub fn resolve_wsl_powershell_path() -> &'static str {
    "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe"
}

pub fn should_use_windows_browser_from_wsl(platform: &str, env: &BTreeMap<String, String>) -> bool {
    platform == "linux"
        && (env.contains_key("WSL_DISTRO_NAME") || env.contains_key("WSL_INTEROP"))
        && !env.contains_key("SSH_CONNECTION")
        && !env.contains_key("SSH_TTY")
        && !env.contains_key("container")
}

pub fn escape_powershell_string_literal(input: &str) -> String {
    format!("'{}'", input.replace('\'', "''"))
}

pub fn encode_utf16_le_base64(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len() * 2);
    for unit in input.encode_utf16() {
        bytes.push((unit & 0xff) as u8);
        bytes.push((unit >> 8) as u8);
    }
    encode_base64(&bytes)
}

pub fn windows_detached_shell_args(args: &[String]) -> Vec<String> {
    args.iter().map(|arg| format!("\"{arg}\"")).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LimitedOutput {
    text: String,
    truncated: bool,
}

fn editor(
    id: EditorId,
    id_slug: &'static str,
    label: &'static str,
    commands: Option<&'static [&'static str]>,
    base_args: &'static [&'static str],
    launch_style: EditorLaunchStyle,
) -> EditorDefinition {
    EditorDefinition {
        id,
        id_slug,
        label,
        commands,
        base_args,
        launch_style,
    }
}

fn spawn_command(command: &str, args: &[&str]) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command).args(args);
        cmd
    } else {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd
    }
}

fn read_pipe_on_thread<R>(mut pipe: R) -> thread::JoinHandle<Vec<u8>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        bytes
    })
}

fn join_pipe_reader(reader: thread::JoinHandle<Vec<u8>>) -> Vec<u8> {
    reader.join().unwrap_or_default()
}

fn normalize_spawn_error(command: &str, args: &[&str], error: std::io::Error) -> ProcessRunError {
    if error.kind() == std::io::ErrorKind::NotFound {
        return ProcessRunError {
            message: format!("Command not found: {command}"),
        };
    }
    ProcessRunError {
        message: format!("Failed to run {}: {}", command_label(command, args), error),
    }
}

fn normalize_stdin_error(command: &str, args: &[&str], error: std::io::Error) -> ProcessRunError {
    ProcessRunError {
        message: format!(
            "Failed to write stdin for {}: {}",
            command_label(command, args),
            error
        ),
    }
}

fn normalize_buffer_error(
    command: &str,
    args: &[&str],
    stream: &str,
    max_buffer_bytes: usize,
) -> ProcessRunError {
    ProcessRunError {
        message: format!(
            "{} exceeded {stream} buffer limit ({max_buffer_bytes} bytes).",
            command_label(command, args)
        ),
    }
}

fn normalize_exit_error(
    command: &str,
    args: &[&str],
    result: &ProcessRunResult,
) -> ProcessRunError {
    if is_windows_command_not_found("win32", result.code, &result.stderr) {
        return ProcessRunError {
            message: format!("Command not found: {command}"),
        };
    }

    let reason = if result.timed_out {
        "timed out".to_string()
    } else {
        format!(
            "failed (code={}, signal={})",
            result
                .code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "null".to_string()),
            result.signal.as_deref().unwrap_or("null")
        )
    };
    let stderr = result.stderr.trim();
    let detail = if stderr.is_empty() {
        String::new()
    } else {
        format!(" {stderr}")
    };
    ProcessRunError {
        message: format!("{} {reason}.{detail}", command_label(command, args)),
    }
}

fn apply_output_limit(
    command: &str,
    args: &[&str],
    stream: &str,
    bytes: Vec<u8>,
    options: &ProcessRunOptions,
) -> Result<LimitedOutput, ProcessRunError> {
    if bytes.len() <= options.max_buffer_bytes {
        return Ok(LimitedOutput {
            text: String::from_utf8_lossy(&bytes).to_string(),
            truncated: false,
        });
    }

    if options.output_mode == ProcessOutputMode::Error {
        return Err(normalize_buffer_error(
            command,
            args,
            stream,
            options.max_buffer_bytes,
        ));
    }

    Ok(LimitedOutput {
        text: String::from_utf8_lossy(&bytes[..options.max_buffer_bytes]).to_string(),
        truncated: true,
    })
}

fn parse_target_path_and_position(target: &str) -> Option<(&str, &str, Option<&str>)> {
    let (prefix, last) = target.rsplit_once(':')?;
    if last.is_empty() || !last.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if let Some((path, line)) = prefix.rsplit_once(':') {
        if !line.is_empty() && line.chars().all(|ch| ch.is_ascii_digit()) {
            return Some((path, line, Some(last)));
        }
    }
    Some((prefix, last, None))
}

fn resolve_command_editor_args(editor: &EditorDefinition, target: &str) -> Vec<String> {
    let parsed = parse_target_path_and_position(target);
    match editor.launch_style {
        EditorLaunchStyle::DirectPath => vec![target.to_string()],
        EditorLaunchStyle::Goto => {
            if parsed.is_some() {
                vec!["--goto".to_string(), target.to_string()]
            } else {
                vec![target.to_string()]
            }
        }
        EditorLaunchStyle::LineColumn => {
            let Some((path, line, column)) = parsed else {
                return vec![target.to_string()];
            };
            let mut args = vec!["--line".to_string(), line.to_string()];
            if let Some(column) = column {
                args.push("--column".to_string());
                args.push(column.to_string());
            }
            args.push(path.to_string());
            args
        }
    }
}

fn resolve_editor_args(editor: &EditorDefinition, target: &str) -> Vec<String> {
    editor
        .base_args
        .iter()
        .map(|arg| (*arg).to_string())
        .chain(resolve_command_editor_args(editor, target))
        .collect()
}

fn resolve_available_command(
    commands: &[&str],
    platform: &str,
    env: &BTreeMap<String, String>,
) -> Option<String> {
    commands
        .iter()
        .find(|command| is_command_available(command, platform, env))
        .map(|command| (*command).to_string())
}

fn strip_wrapping_quotes(value: &str) -> String {
    value.trim_matches('"').to_string()
}

fn path_delimiter_for_platform(platform: &str) -> char {
    if platform == "win32" { ';' } else { ':' }
}

fn resolve_path_environment_variable(env: &BTreeMap<String, String>) -> String {
    env.get("PATH")
        .or_else(|| env.get("Path"))
        .or_else(|| env.get("path"))
        .cloned()
        .unwrap_or_default()
}

fn resolve_windows_path_extensions(env: &BTreeMap<String, String>) -> Vec<String> {
    let fallback = vec![
        ".COM".to_string(),
        ".EXE".to_string(),
        ".BAT".to_string(),
        ".CMD".to_string(),
    ];
    let Some(raw) = env.get("PATHEXT") else {
        return fallback;
    };
    let mut parsed = Vec::new();
    for entry in raw.split(';') {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = if trimmed.starts_with('.') {
            trimmed.to_ascii_uppercase()
        } else {
            format!(".{}", trimmed.to_ascii_uppercase())
        };
        if !parsed.contains(&normalized) {
            parsed.push(normalized);
        }
    }
    if parsed.is_empty() { fallback } else { parsed }
}

fn resolve_command_candidates(
    command: &str,
    platform: &str,
    windows_path_extensions: &[String],
) -> Vec<String> {
    if platform != "win32" {
        return vec![command.to_string()];
    }

    let extension = Path::new(command)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext.to_ascii_uppercase()))
        .unwrap_or_default();

    let mut candidates = Vec::new();
    if !extension.is_empty() && windows_path_extensions.contains(&extension) {
        let command_without_extension = &command[..command.len() - extension.len()];
        push_unique(&mut candidates, command.to_string());
        push_unique(
            &mut candidates,
            format!("{command_without_extension}{extension}"),
        );
        push_unique(
            &mut candidates,
            format!(
                "{command_without_extension}{}",
                extension.to_ascii_lowercase()
            ),
        );
        return candidates;
    }

    for candidate_extension in windows_path_extensions {
        push_unique(&mut candidates, format!("{command}{candidate_extension}"));
        push_unique(
            &mut candidates,
            format!("{command}{}", candidate_extension.to_ascii_lowercase()),
        );
    }
    candidates
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    let mut index = 0;
    while index < input.len() {
        let b0 = input[index];
        let b1 = input.get(index + 1).copied().unwrap_or(0);
        let b2 = input.get(index + 2).copied().unwrap_or(0);
        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if index + 1 < input.len() {
            output.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if index + 2 < input.len() {
            output.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
        index += 3;
    }
    output
}

fn is_executable_file(
    file_path: &Path,
    platform: &str,
    windows_path_extensions: &[String],
) -> bool {
    let Ok(metadata) = fs::metadata(file_path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    if platform == "win32" {
        let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) else {
            return false;
        };
        return windows_path_extensions.contains(&format!(".{}", extension.to_ascii_uppercase()));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("r3-process-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ports_upstream_process_output_limit_modes() {
        let bytes = vec![b'x'; 2048];
        let options = ProcessRunOptions {
            max_buffer_bytes: 128,
            ..ProcessRunOptions::default()
        };
        let error =
            apply_output_limit("node", &["-e"], "stdout", bytes.clone(), &options).unwrap_err();
        assert!(error.message.contains("exceeded stdout buffer limit"));

        let truncated = apply_output_limit(
            "node",
            &["-e"],
            "stdout",
            bytes,
            &ProcessRunOptions {
                max_buffer_bytes: 128,
                output_mode: ProcessOutputMode::Truncate,
                ..ProcessRunOptions::default()
            },
        )
        .unwrap();
        assert_eq!(truncated.text.len(), 128);
        assert!(truncated.truncated);
    }

    #[test]
    fn detects_windows_command_not_found_like_upstream() {
        assert!(is_windows_command_not_found(
            "win32",
            Some(1),
            "wird nicht als interner oder externer Befehl, betriebsfahiges Programm oder Batch-Datei erkannt",
        ));
        assert!(is_windows_command_not_found("win32", Some(9009), ""));
        assert!(!is_windows_command_not_found("linux", Some(9009), ""));
    }

    #[test]
    fn resolves_editor_launch_commands_and_position_args() {
        let empty_env = BTreeMap::new();
        assert_eq!(
            resolve_editor_launch(
                &OpenInEditorInput {
                    cwd: "/tmp/workspace".to_string(),
                    editor: EditorId::Kiro,
                },
                "darwin",
                &empty_env,
            )
            .unwrap(),
            EditorLaunch {
                command: "kiro".to_string(),
                args: vec!["ide".to_string(), "/tmp/workspace".to_string()],
            }
        );

        assert_eq!(
            resolve_editor_launch(
                &OpenInEditorInput {
                    cwd: "/tmp/workspace/src/open.ts:71:5".to_string(),
                    editor: EditorId::Cursor,
                },
                "darwin",
                &empty_env,
            )
            .unwrap()
            .args,
            vec![
                "--goto".to_string(),
                "/tmp/workspace/src/open.ts:71:5".to_string()
            ],
        );

        assert_eq!(
            resolve_editor_launch(
                &OpenInEditorInput {
                    cwd: "/tmp/workspace/src/open.ts:71:5".to_string(),
                    editor: EditorId::Idea,
                },
                "darwin",
                &empty_env,
            )
            .unwrap()
            .args,
            vec![
                "--line".to_string(),
                "71".to_string(),
                "--column".to_string(),
                "5".to_string(),
                "/tmp/workspace/src/open.ts".to_string(),
            ],
        );

        assert_eq!(
            resolve_editor_launch(
                &OpenInEditorInput {
                    cwd: "/tmp/workspace/src/open.ts:71:5".to_string(),
                    editor: EditorId::Zed,
                },
                "linux",
                &empty_env,
            )
            .unwrap()
            .args,
            vec!["/tmp/workspace/src/open.ts:71:5".to_string()],
        );
    }

    #[test]
    fn maps_file_manager_to_platform_commands() {
        let empty_env = BTreeMap::new();
        for (platform, command) in [
            ("darwin", "open"),
            ("win32", "explorer"),
            ("linux", "xdg-open"),
        ] {
            assert_eq!(
                resolve_editor_launch(
                    &OpenInEditorInput {
                        cwd: "/tmp/workspace".to_string(),
                        editor: EditorId::FileManager,
                    },
                    platform,
                    &empty_env,
                )
                .unwrap(),
                EditorLaunch {
                    command: command.to_string(),
                    args: vec!["/tmp/workspace".to_string()],
                }
            );
        }
    }

    #[test]
    fn resolves_browser_launchers_like_upstream_external_launcher() {
        let target = "https://example.com/some path?name=o'hara";
        assert_eq!(
            resolve_browser_launch(target, "darwin", &BTreeMap::new()),
            ProcessLaunch {
                command: "open".to_string(),
                args: vec![target.to_string()],
                options: detached_ignore_stdio_options(false),
            }
        );
        assert_eq!(
            resolve_browser_launch(target, "linux", &BTreeMap::new()),
            ProcessLaunch {
                command: "xdg-open".to_string(),
                args: vec![target.to_string()],
                options: detached_ignore_stdio_options(false),
            }
        );

        let windows =
            resolve_browser_launch(target, "win32", &env(&[("SYSTEMROOT", r"C:\Windows")]));
        assert_eq!(
            windows.command,
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
        );
        assert_eq!(
            windows.args,
            vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-EncodedCommand".to_string(),
                encode_utf16_le_base64(
                    "$ProgressPreference = 'SilentlyContinue'; Start 'https://example.com/some path?name=o''hara'"
                ),
            ]
        );
        assert_eq!(windows.options, detached_ignore_stdio_options(false));

        let wsl = resolve_browser_launch(
            "https://example.com",
            "linux",
            &env(&[("WSL_DISTRO_NAME", "Ubuntu")]),
        );
        assert_eq!(wsl.command, resolve_wsl_powershell_path());
        assert!(wsl.options.detached);

        let wsl_over_ssh = resolve_browser_launch(
            "https://example.com",
            "linux",
            &env(&[
                ("WSL_DISTRO_NAME", "Ubuntu"),
                ("SSH_CONNECTION", "client server"),
            ]),
        );
        assert_eq!(wsl_over_ssh.command, "xdg-open");
    }

    #[test]
    fn ports_external_launcher_powershell_helpers() {
        assert_eq!(
            escape_powershell_string_literal("C:\\Users\\o'hara"),
            "'C:\\Users\\o''hara'"
        );
        assert_eq!(
            resolve_powershell_path(&env(&[("windir", r"D:\Windows")])),
            r"D:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
        );
        assert_eq!(
            encode_utf16_le_base64("Start 'x'"),
            "UwB0AGEAcgB0ACAAJwB4ACcA"
        );
        assert!(should_use_windows_browser_from_wsl(
            "linux",
            &env(&[("WSL_INTEROP", "/run/WSL/1_interop")])
        ));
        assert!(!should_use_windows_browser_from_wsl(
            "linux",
            &env(&[("WSL_INTEROP", "/run/WSL/1_interop"), ("container", "1")])
        ));
    }

    #[test]
    fn resolves_windows_commands_with_pathext() {
        let dir = temp_dir("pathext");
        fs::write(dir.join("code.CMD"), "@echo off\r\n").unwrap();
        fs::write(dir.join("npm"), "echo nope\r\n").unwrap();
        let env = env(&[
            ("PATH", &dir.to_string_lossy()),
            ("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
        ]);

        assert!(is_command_available("code", "win32", &env));
        assert!(!is_command_available("npm", "win32", &env));
        assert!(
            resolve_command_path("code", "win32", &env)
                .unwrap()
                .ends_with("code.CMD")
        );
    }

    #[test]
    fn resolves_available_editors_in_upstream_order() {
        let dir = temp_dir("editors");
        for command in [
            "trae.CMD",
            "kiro.CMD",
            "code-insiders.CMD",
            "codium.CMD",
            "aqua.CMD",
            "clion.CMD",
            "datagrip.CMD",
            "dataspell.CMD",
            "goland.CMD",
            "phpstorm.CMD",
            "pycharm.CMD",
            "rider.CMD",
            "rubymine.CMD",
            "rustrover.CMD",
            "webstorm.CMD",
            "explorer.CMD",
        ] {
            fs::write(dir.join(command), "@echo off\r\n").unwrap();
        }
        let env = env(&[
            ("PATH", &dir.to_string_lossy()),
            ("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
        ]);

        assert_eq!(
            resolve_available_editors("win32", &env),
            vec![
                EditorId::Trae,
                EditorId::Kiro,
                EditorId::VsCodeInsiders,
                EditorId::VsCodium,
                EditorId::Aqua,
                EditorId::CLion,
                EditorId::DataGrip,
                EditorId::DataSpell,
                EditorId::GoLand,
                EditorId::PhpStorm,
                EditorId::PyCharm,
                EditorId::Rider,
                EditorId::RubyMine,
                EditorId::RustRover,
                EditorId::WebStorm,
                EditorId::FileManager,
            ]
        );
    }

    #[test]
    fn quotes_windows_detached_args_like_upstream_launcher() {
        assert_eq!(
            windows_detached_shell_args(&["C:\\my workspace".to_string(), "--flag".to_string()]),
            vec!["\"C:\\my workspace\"".to_string(), "\"--flag\"".to_string()]
        );
    }
}

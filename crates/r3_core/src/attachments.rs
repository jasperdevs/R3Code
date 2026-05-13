use std::{
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::ChatAttachment;

pub const ATTACHMENTS_ROUTE_PREFIX: &str = "/attachments";
pub const ATTACHMENTS_ROUTE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";
const ATTACHMENT_ID_THREAD_SEGMENT_MAX_CHARS: usize = 80;
const ATTACHMENT_FILENAME_EXTENSIONS: &[&str] = &[
    ".avif", ".bmp", ".gif", ".heic", ".heif", ".ico", ".jpeg", ".jpg", ".png", ".svg", ".tiff",
    ".webp", ".bin",
];

static ATTACHMENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBase64DataUrl {
    pub mime_type: String,
    pub base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentRouteDecision {
    Invalid {
        body: &'static str,
        status: u16,
    },
    NotFound {
        body: &'static str,
        status: u16,
    },
    File {
        path: PathBuf,
        status: u16,
        cache_control: &'static str,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentRouteFileResponse {
    Invalid {
        body: &'static str,
        status: u16,
    },
    NotFound {
        body: &'static str,
        status: u16,
    },
    File {
        path: PathBuf,
        bytes: Vec<u8>,
        status: u16,
        cache_control: &'static str,
    },
    InternalServerError {
        body: &'static str,
        status: u16,
    },
}

pub fn normalize_attachment_relative_path(raw_relative_path: &str) -> Option<String> {
    let normalized = normalize_attachment_path_segments(raw_relative_path);
    let stripped = normalized
        .trim_start_matches(['/', '\\'])
        .replace('\\', "/");
    if stripped.is_empty() || stripped.starts_with("..") || stripped.contains('\0') {
        return None;
    }
    Some(stripped)
}

pub fn attachment_route_decision(
    raw_relative_path: &str,
    resolved_file_path: Option<&Path>,
    resolved_path_is_file: bool,
    file_response_failed: bool,
) -> AttachmentRouteDecision {
    let Some(normalized_relative_path) = normalize_attachment_relative_path(raw_relative_path)
    else {
        return AttachmentRouteDecision::Invalid {
            body: "Invalid attachment path",
            status: 400,
        };
    };
    let is_id_lookup =
        !normalized_relative_path.contains('/') && !normalized_relative_path.contains('.');
    let Some(path) = resolved_file_path else {
        return if is_id_lookup {
            AttachmentRouteDecision::NotFound {
                body: "Not Found",
                status: 404,
            }
        } else {
            AttachmentRouteDecision::Invalid {
                body: "Invalid attachment path",
                status: 400,
            }
        };
    };
    if !resolved_path_is_file {
        return AttachmentRouteDecision::NotFound {
            body: "Not Found",
            status: 404,
        };
    }
    if file_response_failed {
        return AttachmentRouteDecision::InternalServerError {
            body: "Internal Server Error",
            status: 500,
        };
    }
    AttachmentRouteDecision::File {
        path: path.to_path_buf(),
        status: 200,
        cache_control: ATTACHMENTS_ROUTE_CACHE_CONTROL,
    }
}

pub fn attachment_route_file_response(
    attachments_dir: &Path,
    raw_relative_path: &str,
) -> AttachmentRouteFileResponse {
    let Some(normalized_relative_path) = normalize_attachment_relative_path(raw_relative_path)
    else {
        return AttachmentRouteFileResponse::Invalid {
            body: "Invalid attachment path",
            status: 400,
        };
    };
    let is_id_lookup =
        !normalized_relative_path.contains('/') && !normalized_relative_path.contains('.');
    let file_path = if is_id_lookup {
        let Some(path) = resolve_attachment_path_by_id(attachments_dir, &normalized_relative_path)
        else {
            return AttachmentRouteFileResponse::NotFound {
                body: "Not Found",
                status: 404,
            };
        };
        path
    } else {
        let Some(path) =
            resolve_attachment_relative_path(attachments_dir, &normalized_relative_path)
        else {
            return AttachmentRouteFileResponse::Invalid {
                body: "Invalid attachment path",
                status: 400,
            };
        };
        path
    };
    if !std::fs::metadata(&file_path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return AttachmentRouteFileResponse::NotFound {
            body: "Not Found",
            status: 404,
        };
    }
    match std::fs::read(&file_path) {
        Ok(bytes) => AttachmentRouteFileResponse::File {
            path: file_path,
            bytes,
            status: 200,
            cache_control: ATTACHMENTS_ROUTE_CACHE_CONTROL,
        },
        Err(_) => AttachmentRouteFileResponse::InternalServerError {
            body: "Internal Server Error",
            status: 500,
        },
    }
}

fn normalize_attachment_path_segments(raw_relative_path: &str) -> String {
    let normalized = raw_relative_path.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if parts.last().is_some_and(|previous| *previous != "..") {
                parts.pop();
            } else {
                parts.push("..");
            }
        } else {
            parts.push(part);
        }
    }
    parts.join("/")
}

pub fn resolve_attachment_relative_path(
    attachments_dir: &Path,
    relative_path: &str,
) -> Option<PathBuf> {
    let normalized = normalize_attachment_relative_path(relative_path)?;
    let attachments_root = absolutize(attachments_dir);
    let file_path = attachments_root.join(Path::new(&normalized));
    if path_starts_with(&file_path, &attachments_root) {
        Some(file_path)
    } else {
        None
    }
}

pub fn to_safe_thread_attachment_segment(thread_id: &str) -> Option<String> {
    let mut segment = String::new();
    let mut previous_dash = false;
    for character in thread_id.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            segment.push(character);
            previous_dash = false;
        } else if character == '-' {
            if !previous_dash {
                segment.push('-');
                previous_dash = true;
            }
        } else if !previous_dash {
            segment.push('-');
            previous_dash = true;
        }
        if segment.len() >= ATTACHMENT_ID_THREAD_SEGMENT_MAX_CHARS {
            break;
        }
    }
    let trimmed = segment.trim_matches(['-', '_']).to_string();
    let trimmed = trimmed.trim_end_matches(['-', '_']).to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

pub fn create_attachment_id(thread_id: &str) -> Option<String> {
    let thread_segment = to_safe_thread_attachment_segment(thread_id)?;
    Some(format!("{thread_segment}-{}", pseudo_uuid_v4()))
}

pub fn parse_thread_segment_from_attachment_id(attachment_id: &str) -> Option<String> {
    let normalized = normalize_attachment_relative_path(attachment_id)?;
    if normalized.contains('/') || normalized.contains('.') {
        return None;
    }
    let uuid_start = normalized.len().checked_sub(36)?;
    if uuid_start == 0 || normalized.as_bytes().get(uuid_start - 1) != Some(&b'-') {
        return None;
    }
    let segment = &normalized[..uuid_start - 1];
    let uuid = &normalized[uuid_start..];
    if is_valid_uuid_like(uuid) && is_valid_thread_segment(segment) {
        Some(segment.to_lowercase())
    } else {
        None
    }
}

pub fn attachment_relative_path(attachment: &ChatAttachment) -> String {
    match attachment {
        ChatAttachment::Image(image) => {
            let extension = infer_image_extension(&image.mime_type, Some(&image.name));
            format!("{}{}", image.id, extension)
        }
    }
}

pub fn resolve_attachment_path(
    attachments_dir: &Path,
    attachment: &ChatAttachment,
) -> Option<PathBuf> {
    resolve_attachment_relative_path(attachments_dir, &attachment_relative_path(attachment))
}

pub fn resolve_attachment_path_by_id(
    attachments_dir: &Path,
    attachment_id: &str,
) -> Option<PathBuf> {
    let normalized_id = normalize_attachment_relative_path(attachment_id)?;
    if normalized_id.contains('/') || normalized_id.contains('.') {
        return None;
    }
    for extension in ATTACHMENT_FILENAME_EXTENSIONS {
        let maybe_path = resolve_attachment_relative_path(
            attachments_dir,
            &format!("{normalized_id}{extension}"),
        )?;
        if maybe_path.exists() {
            return Some(maybe_path);
        }
    }
    None
}

pub fn parse_attachment_id_from_relative_path(relative_path: &str) -> Option<String> {
    let normalized = normalize_attachment_relative_path(relative_path)?;
    if normalized.contains('/') {
        return None;
    }
    let extension_index = normalized.rfind('.')?;
    if extension_index == 0 {
        return None;
    }
    let id = &normalized[..extension_index];
    (!id.is_empty() && !id.contains('.')).then(|| id.to_string())
}

pub fn parse_base64_data_url(data_url: &str) -> Option<ParsedBase64DataUrl> {
    let trimmed = data_url.trim();
    let payload = trimmed.strip_prefix("data:")?;
    let (header, base64) = payload.split_once(',')?;
    let mut header_parts = header
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if header_parts.len() < 2 {
        return None;
    }
    if header_parts.pop()?.to_lowercase() != "base64" {
        return None;
    }
    let mime_type = header_parts.first()?.to_lowercase();
    if mime_type.is_empty() {
        return None;
    }
    let base64 = base64.split_whitespace().collect::<String>();
    if base64.is_empty()
        || !base64.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '/' | '=')
        })
    {
        return None;
    }
    Some(ParsedBase64DataUrl { mime_type, base64 })
}

pub fn infer_image_extension(mime_type: &str, file_name: Option<&str>) -> &'static str {
    match mime_type.to_lowercase().as_str() {
        "image/avif" => ".avif",
        "image/bmp" => ".bmp",
        "image/gif" => ".gif",
        "image/heic" => ".heic",
        "image/heif" => ".heif",
        "image/jpeg" | "image/jpg" => ".jpg",
        "image/png" => ".png",
        "image/svg+xml" => ".svg",
        "image/tiff" => ".tiff",
        "image/webp" => ".webp",
        _ => safe_extension_from_file_name(file_name).unwrap_or(".bin"),
    }
}

fn safe_extension_from_file_name(file_name: Option<&str>) -> Option<&'static str> {
    let file_name = file_name?.trim();
    let extension = file_name.rsplit_once('.')?.1.to_lowercase();
    match extension.as_str() {
        "avif" => Some(".avif"),
        "bmp" => Some(".bmp"),
        "gif" => Some(".gif"),
        "heic" => Some(".heic"),
        "heif" => Some(".heif"),
        "ico" => Some(".ico"),
        "jpeg" => Some(".jpeg"),
        "jpg" => Some(".jpg"),
        "png" => Some(".png"),
        "svg" => Some(".svg"),
        "tiff" => Some(".tiff"),
        "webp" => Some(".webp"),
        _ => None,
    }
}

fn pseudo_uuid_v4() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let counter = ATTACHMENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed) as u128;
    let value = nanos ^ counter.rotate_left(17);
    let hex = format!("{value:032x}");
    format!(
        "{}-{}-4{}-8{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[13..16],
        &hex[17..20],
        &hex[20..32]
    )
}

fn is_valid_thread_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.len() > ATTACHMENT_ID_THREAD_SEGMENT_MAX_CHARS {
        return false;
    }
    let mut previous_dash = false;
    for character in segment.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            previous_dash = false;
        } else if character == '-' {
            if previous_dash {
                return false;
            }
            previous_dash = true;
        } else {
            return false;
        }
    }
    !segment.starts_with(['-', '_']) && !segment.ends_with(['-', '_'])
}

fn is_valid_uuid_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for index in [8, 13, 18, 23] {
        if bytes[index] != b'-' {
            return false;
        }
    }
    value
        .chars()
        .enumerate()
        .all(|(index, character)| [8, 13, 18, 23].contains(&index) || character.is_ascii_hexdigit())
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path_components = path.components().collect::<Vec<_>>();
    let root_components = root.components().collect::<Vec<_>>();
    path_components.len() >= root_components.len()
        && path_components
            .iter()
            .zip(root_components.iter())
            .all(|(left, right)| components_equal(left, right))
}

fn components_equal(left: &Component<'_>, right: &Component<'_>) -> bool {
    left.as_os_str() == right.as_os_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatImageAttachment;
    use std::{env, fs, time::SystemTime};

    fn temp_dir() -> PathBuf {
        let root = env::temp_dir().join(format!(
            "r3code-attachment-store-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn parses_base64_data_urls_like_upstream() {
        assert_eq!(
            parse_base64_data_url("data:image/png;charset=utf-8;base64,SGVs bG8=\n"),
            Some(ParsedBase64DataUrl {
                mime_type: "image/png".to_string(),
                base64: "SGVsbG8=".to_string(),
            })
        );
        assert_eq!(
            parse_base64_data_url("data:image/png;charset=utf-8,hello"),
            None
        );
        assert_eq!(parse_base64_data_url("data:;base64,SGVsbG8="), None);
    }

    #[test]
    fn infers_safe_image_extensions() {
        assert_eq!(infer_image_extension("image/jpeg", Some("x.png")), ".jpg");
        assert_eq!(
            infer_image_extension("constructor", Some("icon.webp")),
            ".webp"
        );
        assert_eq!(
            infer_image_extension("constructor", Some("unsafe.exe")),
            ".bin"
        );
    }

    #[test]
    fn sanitizes_and_parses_attachment_ids_without_prefix_collisions() {
        let attachment_id = create_attachment_id("Thread.Foo/unsafe space").unwrap();
        let thread_segment = parse_thread_segment_from_attachment_id(&attachment_id).unwrap();

        assert_eq!(thread_segment, "thread-foo-unsafe-space");
        assert_eq!(
            parse_thread_segment_from_attachment_id("foo-00000000-0000-4000-8000-000000000001"),
            Some("foo".to_string())
        );
        assert_eq!(
            parse_thread_segment_from_attachment_id("foo-bar-00000000-0000-4000-8000-000000000002"),
            Some("foo-bar".to_string())
        );
        assert_eq!(parse_thread_segment_from_attachment_id("../bad"), None);
    }

    #[test]
    fn resolves_attachment_paths_by_existing_extension() {
        let root = temp_dir();
        let attachment_id = "thread-1-attachment";
        let png_path = root.join(format!("{attachment_id}.png"));
        fs::write(&png_path, b"hello").unwrap();

        assert_eq!(
            normalize_attachment_relative_path("nested/../thread-1-attachment.png"),
            Some("thread-1-attachment.png".to_string())
        );
        assert_eq!(normalize_attachment_relative_path("../secret.png"), None);
        assert_eq!(normalize_attachment_relative_path("bad\0path.png"), None);
        assert_eq!(
            resolve_attachment_path_by_id(&root, attachment_id),
            Some(png_path.clone())
        );
        assert_eq!(resolve_attachment_path_by_id(&root, "missing"), None);
        assert_eq!(resolve_attachment_path_by_id(&root, "../secret"), None);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ports_attachment_route_response_decisions() {
        let file_path = PathBuf::from("C:/r3/attachments/thread-1-attachment.png");
        assert_eq!(
            attachment_route_decision("../secret.png", None, false, false),
            AttachmentRouteDecision::Invalid {
                body: "Invalid attachment path",
                status: 400,
            }
        );
        assert_eq!(
            attachment_route_decision("missing-id", None, false, false),
            AttachmentRouteDecision::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        assert_eq!(
            attachment_route_decision("missing.png", None, false, false),
            AttachmentRouteDecision::Invalid {
                body: "Invalid attachment path",
                status: 400,
            }
        );
        assert_eq!(
            attachment_route_decision("thread-1-attachment", Some(&file_path), false, false),
            AttachmentRouteDecision::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        assert_eq!(
            attachment_route_decision("thread-1-attachment", Some(&file_path), true, false),
            AttachmentRouteDecision::File {
                path: file_path.clone(),
                status: 200,
                cache_control: ATTACHMENTS_ROUTE_CACHE_CONTROL,
            }
        );
        assert_eq!(
            attachment_route_decision("thread-1-attachment", Some(&file_path), true, true),
            AttachmentRouteDecision::InternalServerError {
                body: "Internal Server Error",
                status: 500,
            }
        );

        let root = temp_dir();
        let png_path = root.join("thread-1-attachment.png");
        fs::write(&png_path, [9_u8, 8, 7]).unwrap();
        assert_eq!(
            attachment_route_file_response(&root, "thread-1-attachment"),
            AttachmentRouteFileResponse::File {
                path: png_path.clone(),
                bytes: vec![9, 8, 7],
                status: 200,
                cache_control: ATTACHMENTS_ROUTE_CACHE_CONTROL,
            }
        );
        fs::create_dir_all(root.join("nested")).unwrap();
        let nested_path = root.join("nested/screen.webp");
        fs::write(&nested_path, [1_u8, 2]).unwrap();
        assert_eq!(
            attachment_route_file_response(&root, "nested/screen.webp"),
            AttachmentRouteFileResponse::File {
                path: nested_path,
                bytes: vec![1, 2],
                status: 200,
                cache_control: ATTACHMENTS_ROUTE_CACHE_CONTROL,
            }
        );
        assert_eq!(
            attachment_route_file_response(&root, "../secret.png"),
            AttachmentRouteFileResponse::Invalid {
                body: "Invalid attachment path",
                status: 400,
            }
        );
        assert_eq!(
            attachment_route_file_response(&root, "missing-id"),
            AttachmentRouteFileResponse::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        assert_eq!(
            attachment_route_file_response(&root, "missing.png"),
            AttachmentRouteFileResponse::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        fs::create_dir_all(root.join("directory.png")).unwrap();
        assert_eq!(
            attachment_route_file_response(&root, "directory.png"),
            AttachmentRouteFileResponse::NotFound {
                body: "Not Found",
                status: 404,
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builds_attachment_relative_paths_and_parses_ids() {
        let attachment = ChatAttachment::Image(ChatImageAttachment {
            id: "thread-1-attachment".to_string(),
            name: "screen.png".to_string(),
            mime_type: "image/png".to_string(),
            size_bytes: 1,
            preview_url: None,
        });

        assert_eq!(
            attachment_relative_path(&attachment),
            "thread-1-attachment.png"
        );
        assert_eq!(
            parse_attachment_id_from_relative_path("thread-1-attachment.png"),
            Some("thread-1-attachment".to_string())
        );
        assert_eq!(
            parse_attachment_id_from_relative_path("nested/file.png"),
            None
        );
    }
}

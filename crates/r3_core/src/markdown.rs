use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownFileLinkMeta {
    pub file_path: String,
    pub target_path: String,
    pub display_path: String,
    pub basename: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownRenderedFileLink {
    pub href: String,
    pub target_path: String,
    pub display_path: String,
    pub file_path: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatMarkdownInline {
    Text(String),
    Code(String),
    Strikethrough(String),
    Link {
        label: String,
        href: String,
        file: Option<MarkdownRenderedFileLink>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatMarkdownBlock {
    Heading {
        level: u8,
        inlines: Vec<ChatMarkdownInline>,
    },
    Paragraph(Vec<ChatMarkdownInline>),
    ListItem(Vec<ChatMarkdownInline>),
    OrderedListItem {
        number: u32,
        inlines: Vec<ChatMarkdownInline>,
    },
    TaskListItem {
        checked: bool,
        inlines: Vec<ChatMarkdownInline>,
    },
    Blockquote(Vec<ChatMarkdownInline>),
    Table {
        header: Vec<Vec<ChatMarkdownInline>>,
        rows: Vec<Vec<Vec<ChatMarkdownInline>>>,
    },
    CodeBlock {
        language: String,
        code: String,
        copy_label: &'static str,
    },
}

const POSIX_FILE_ROOT_PREFIXES: &[&str] = &[
    "/Users/",
    "/home/",
    "/tmp/",
    "/var/",
    "/etc/",
    "/opt/",
    "/mnt/",
    "/Volumes/",
    "/private/",
    "/root/",
];

pub fn normalize_markdown_link_destination(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn rewrite_markdown_file_uri_href(href: Option<&str>) -> Option<String> {
    let href = normalize_markdown_link_destination(href?);
    let rest = href.strip_prefix("file://")?;
    let (path, hash) = split_hash(rest);
    if path.is_empty() {
        return None;
    }
    Some(format!("{}{}", decode_uri_component(path), hash))
}

pub fn resolve_markdown_file_link_meta(
    href: Option<&str>,
    cwd: Option<&str>,
) -> Option<MarkdownFileLinkMeta> {
    let raw_href = normalize_markdown_link_destination(href?);
    if raw_href.is_empty() || raw_href.starts_with('#') {
        return None;
    }

    let rewritten_file_uri = rewrite_markdown_file_uri_href(Some(&raw_href));
    let source = rewritten_file_uri.as_deref().unwrap_or(&raw_href);
    let (path_without_hash, hash) = split_hash(source);
    let decoded_path = normalize_windows_drive_path(&decode_uri_component(
        path_without_hash
            .split('?')
            .next()
            .unwrap_or(path_without_hash),
    ));
    if decoded_path.is_empty() || has_external_scheme(&decoded_path) {
        return None;
    }
    if !is_likely_path_candidate(&decoded_path) {
        return None;
    }

    let path_with_position = append_line_column_from_hash(&decoded_path, hash);
    let target_path = if is_relative_path(&path_with_position) {
        resolve_relative_path(&path_with_position, cwd?)?
    } else {
        path_with_position
    };
    let (file_path, line, column) = split_path_and_position(&target_path);
    let basename = basename_of_path(&file_path).to_string();
    let display_path = format_workspace_relative_path(&target_path, cwd);

    Some(MarkdownFileLinkMeta {
        file_path,
        target_path,
        display_path,
        basename,
        line,
        column,
    })
}

pub fn extract_markdown_link_hrefs(text: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(open_label) = text[cursor..].find('[').map(|index| cursor + index) else {
            break;
        };
        let Some(close_label) = text[open_label + 1..]
            .find(']')
            .map(|index| open_label + 1 + index)
        else {
            break;
        };
        if !text[close_label + 1..].starts_with('(') {
            cursor = close_label + 1;
            continue;
        }
        let href_start = close_label + 2;
        let Some(close_href) = text[href_start..].find(')').map(|index| href_start + index) else {
            break;
        };
        let href = text[href_start..close_href]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim();
        if !href.is_empty() {
            hrefs.push(href.to_string());
        }
        cursor = close_href + 1;
    }
    hrefs
}

pub fn build_file_link_parent_suffix_by_path(file_paths: &[String]) -> BTreeMap<String, String> {
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file_path in file_paths {
        let basename = basename_of_path(file_path);
        if basename.is_empty() {
            continue;
        }
        groups
            .entry(basename.to_string())
            .or_default()
            .insert(file_path.clone());
    }

    let mut suffix_by_path = BTreeMap::new();
    for group in groups.values() {
        if group.len() < 2 {
            continue;
        }
        let unique_paths = group.iter().cloned().collect::<Vec<_>>();
        for file_path in &unique_paths {
            let segments = path_parent_segments(file_path);
            if segments.is_empty() {
                continue;
            }
            let mut resolved_depth = segments.len();
            for depth in 1..=segments.len() {
                let candidate = segments[segments.len() - depth..].join("/");
                let collision = unique_paths.iter().any(|other_path| {
                    if other_path == file_path {
                        return false;
                    }
                    let other_segments = path_parent_segments(other_path);
                    other_segments
                        .len()
                        .checked_sub(depth)
                        .is_some_and(|start| other_segments[start..].join("/") == candidate)
                });
                if !collision {
                    resolved_depth = depth;
                    break;
                }
            }
            let suffix_depth = segments.len().min(resolved_depth.max(2));
            suffix_by_path.insert(
                file_path.clone(),
                segments[segments.len() - suffix_depth..].join("/"),
            );
        }
    }
    suffix_by_path
}

pub fn render_chat_markdown_blocks(text: &str, cwd: Option<&str>) -> Vec<ChatMarkdownBlock> {
    let metas = markdown_file_link_meta_by_href(text, cwd);
    let suffixes = build_file_link_parent_suffix_by_path(
        &metas
            .values()
            .map(|meta| meta.file_path.clone())
            .collect::<Vec<_>>(),
    );
    let mut blocks = Vec::new();
    let mut paragraph = String::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(language) = line.trim_start().strip_prefix("```") {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            let mut code = String::new();
            for code_line in lines.by_ref() {
                if code_line.trim_start().starts_with("```") {
                    break;
                }
                code.push_str(code_line);
                code.push('\n');
            }
            blocks.push(ChatMarkdownBlock::CodeBlock {
                language: extract_fence_language(language.trim()),
                code,
                copy_label: "Copy code",
            });
            continue;
        }

        if line.trim().is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            continue;
        }

        if let Some((level, heading_text)) = parse_heading_line(line) {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            blocks.push(ChatMarkdownBlock::Heading {
                level,
                inlines: parse_markdown_inlines(heading_text, &metas, &suffixes),
            });
            continue;
        }

        if is_table_header_candidate(line)
            && lines.peek().is_some_and(|next| is_table_separator(next))
        {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            let header = parse_table_row(line, &metas, &suffixes);
            lines.next();
            let mut rows = Vec::new();
            while let Some(next) = lines.peek() {
                if next.trim().is_empty() || !next.contains('|') {
                    break;
                }
                rows.push(parse_table_row(
                    lines.next().unwrap_or(""),
                    &metas,
                    &suffixes,
                ));
            }
            blocks.push(ChatMarkdownBlock::Table { header, rows });
            continue;
        }

        if let Some(quote_text) = line.trim_start().strip_prefix("> ") {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            blocks.push(ChatMarkdownBlock::Blockquote(parse_markdown_inlines(
                quote_text, &metas, &suffixes,
            )));
            continue;
        }

        if let Some((checked, task_text)) = parse_task_list_line(line) {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            blocks.push(ChatMarkdownBlock::TaskListItem {
                checked,
                inlines: parse_markdown_inlines(task_text, &metas, &suffixes),
            });
            continue;
        }

        if let Some((number, ordered_text)) = parse_ordered_list_line(line) {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            blocks.push(ChatMarkdownBlock::OrderedListItem {
                number,
                inlines: parse_markdown_inlines(ordered_text, &metas, &suffixes),
            });
            continue;
        }

        if let Some(list_text) = line.trim_start().strip_prefix("- ") {
            flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
            blocks.push(ChatMarkdownBlock::ListItem(parse_markdown_inlines(
                list_text, &metas, &suffixes,
            )));
            continue;
        }

        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(line.trim());
    }

    flush_paragraph(&mut blocks, &mut paragraph, &metas, &suffixes);
    blocks
}

fn markdown_file_link_meta_by_href(
    text: &str,
    cwd: Option<&str>,
) -> BTreeMap<String, MarkdownFileLinkMeta> {
    let mut meta_by_href = BTreeMap::new();
    for href in extract_markdown_link_hrefs(text) {
        let normalized_href = normalize_markdown_link_href_key(&href);
        if meta_by_href.contains_key(&normalized_href) {
            continue;
        }
        if let Some(meta) = resolve_markdown_file_link_meta(Some(&normalized_href), cwd) {
            meta_by_href.insert(normalized_href, meta);
        }
    }
    meta_by_href
}

fn parse_markdown_inlines(
    text: &str,
    metas: &BTreeMap<String, MarkdownFileLinkMeta>,
    suffixes: &BTreeMap<String, String>,
) -> Vec<ChatMarkdownInline> {
    let mut output = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let next_code = text[cursor..].find('`').map(|index| cursor + index);
        let next_link = text[cursor..].find('[').map(|index| cursor + index);
        let next_strike = text[cursor..].find("~~").map(|index| cursor + index);
        let next = [next_code, next_link, next_strike]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or_else(|| {
                push_text(&mut output, &text[cursor..]);
                text.len()
            });
        if next == text.len() {
            break;
        }
        if next > cursor {
            push_text(&mut output, &text[cursor..next]);
        }
        if text[next..].starts_with('`') {
            if let Some(close) = text[next + 1..].find('`').map(|index| next + 1 + index) {
                output.push(ChatMarkdownInline::Code(text[next + 1..close].to_string()));
                cursor = close + 1;
                continue;
            }
            push_text(&mut output, "`");
            cursor = next + 1;
            continue;
        }

        if text[next..].starts_with("~~") {
            if let Some(close) = text[next + 2..].find("~~").map(|index| next + 2 + index) {
                output.push(ChatMarkdownInline::Strikethrough(
                    text[next + 2..close].to_string(),
                ));
                cursor = close + 2;
                continue;
            }
            push_text(&mut output, "~~");
            cursor = next + 2;
            continue;
        }

        let Some(close_label) = text[next + 1..].find(']').map(|index| next + 1 + index) else {
            push_text(&mut output, "[");
            cursor = next + 1;
            continue;
        };
        if !text[close_label + 1..].starts_with('(') {
            push_text(&mut output, &text[next..=close_label]);
            cursor = close_label + 1;
            continue;
        }
        let href_start = close_label + 2;
        let Some(close_href) = text[href_start..].find(')').map(|index| href_start + index) else {
            push_text(&mut output, &text[next..]);
            break;
        };
        let label = text[next + 1..close_label].to_string();
        let href = text[href_start..close_href]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        let normalized_href = normalize_markdown_link_href_key(&href);
        let file = metas
            .get(&normalized_href)
            .map(|meta| rendered_file_link(meta, suffixes));
        let rendered_href = file
            .as_ref()
            .map(|file| file.href.clone())
            .unwrap_or_else(|| normalize_markdown_link_destination(&href));
        output.push(ChatMarkdownInline::Link {
            label,
            href: rendered_href,
            file,
        });
        cursor = close_href + 1;
    }
    output
}

fn parse_table_row(
    line: &str,
    metas: &BTreeMap<String, MarkdownFileLinkMeta>,
    suffixes: &BTreeMap<String, String>,
) -> Vec<Vec<ChatMarkdownInline>> {
    let trimmed = line.trim().trim_matches('|');
    trimmed
        .split('|')
        .map(|cell| parse_markdown_inlines(cell.trim(), metas, suffixes))
        .collect()
}

fn is_table_header_candidate(line: &str) -> bool {
    line.matches('|').count() >= 1 && !is_table_separator(line)
}

fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|');
    if trimmed.is_empty() {
        return false;
    }
    trimmed.split('|').all(|cell| {
        let cell = cell.trim();
        cell.len() >= 3
            && cell.chars().all(|ch| matches!(ch, '-' | ':' | ' '))
            && cell.chars().any(|ch| ch == '-')
    })
}

fn parse_heading_line(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let text = trimmed[level..].strip_prefix(' ')?;
    Some((level as u8, text.trim()))
}

fn parse_task_list_line(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("- [")?;
    let marker = rest.chars().next()?;
    let rest = rest.get(1..)?;
    let text = rest.strip_prefix("] ")?;
    match marker {
        'x' | 'X' => Some((true, text)),
        ' ' => Some((false, text)),
        _ => None,
    }
}

fn parse_ordered_list_line(line: &str) -> Option<(u32, &str)> {
    let trimmed = line.trim_start();
    let dot_index = trimmed.find(". ")?;
    let number = trimmed[..dot_index].parse::<u32>().ok()?;
    Some((number, &trimmed[dot_index + 2..]))
}

fn rendered_file_link(
    meta: &MarkdownFileLinkMeta,
    suffixes: &BTreeMap<String, String>,
) -> MarkdownRenderedFileLink {
    let mut label_parts = vec![meta.basename.clone()];
    if let Some(parent_suffix) = suffixes.get(&meta.file_path) {
        if !parent_suffix.is_empty() {
            label_parts.push(parent_suffix.clone());
        }
    }
    if let Some(line) = meta.line {
        let mut line_part = format!("L{line}");
        if let Some(column) = meta.column {
            line_part.push_str(&format!(":C{column}"));
        }
        label_parts.push(line_part);
    }

    MarkdownRenderedFileLink {
        href: meta.target_path.clone(),
        target_path: meta.target_path.clone(),
        display_path: meta.display_path.clone(),
        file_path: meta.file_path.clone(),
        label: label_parts.join(" · "),
    }
}

fn normalize_markdown_link_href_key(href: &str) -> String {
    let normalized = normalize_markdown_link_destination(href);
    rewrite_markdown_file_uri_href(Some(&normalized)).unwrap_or(normalized)
}

fn flush_paragraph(
    blocks: &mut Vec<ChatMarkdownBlock>,
    paragraph: &mut String,
    metas: &BTreeMap<String, MarkdownFileLinkMeta>,
    suffixes: &BTreeMap<String, String>,
) {
    let text = paragraph.trim();
    if !text.is_empty() {
        blocks.push(ChatMarkdownBlock::Paragraph(parse_markdown_inlines(
            text, metas, suffixes,
        )));
    }
    paragraph.clear();
}

fn push_text(output: &mut Vec<ChatMarkdownInline>, text: &str) {
    if text.is_empty() {
        return;
    }
    match output.last_mut() {
        Some(ChatMarkdownInline::Text(existing)) => existing.push_str(text),
        _ => output.push(ChatMarkdownInline::Text(text.to_string())),
    }
}

fn split_hash(value: &str) -> (&str, &str) {
    if let Some(index) = value.find('#') {
        (&value[..index], &value[index..])
    } else {
        (value, "")
    }
}

fn append_line_column_from_hash(path: &str, hash: &str) -> String {
    if hash.is_empty() || has_position_suffix(path) {
        return path.to_string();
    }
    let Some(rest) = hash.strip_prefix("#L").or_else(|| hash.strip_prefix("#l")) else {
        return path.to_string();
    };
    let (line, column) = match rest.find(['C', 'c']) {
        Some(index) => (&rest[..index], Some(&rest[index + 1..])),
        None => (rest, None),
    };
    if line.chars().all(|ch| ch.is_ascii_digit()) && !line.is_empty() {
        if let Some(column) = column {
            if column.chars().all(|ch| ch.is_ascii_digit()) && !column.is_empty() {
                return format!("{path}:{line}:{column}");
            }
        } else {
            return format!("{path}:{line}");
        }
    }
    path.to_string()
}

fn split_path_and_position(target: &str) -> (String, Option<u32>, Option<u32>) {
    let parts = target.rsplit(':').take(3).collect::<Vec<_>>();
    if parts.len() >= 2 {
        let last = parts[0].parse::<u32>().ok();
        let second = parts[1].parse::<u32>().ok();
        if let Some(line) = second {
            let path_len = target.len() - parts[0].len() - parts[1].len() - 2;
            return (target[..path_len].to_string(), Some(line), last);
        }
        if let Some(line) = last {
            let path_len = target.len() - parts[0].len() - 1;
            return (target[..path_len].to_string(), Some(line), None);
        }
    }
    (target.to_string(), None, None)
}

fn resolve_relative_path(path: &str, cwd: &str) -> Option<String> {
    if path.starts_with("~/") {
        return Some(path.to_string());
    }
    let mut base = cwd.replace('\\', "/");
    while base.ends_with('/') {
        base.pop();
    }
    let mut relative = path;
    while let Some(stripped) = relative.strip_prefix("./") {
        relative = stripped;
    }
    Some(format!("{base}/{relative}"))
}

fn format_workspace_relative_path(target_path: &str, cwd: Option<&str>) -> String {
    let Some(cwd) = cwd else {
        return target_path.to_string();
    };
    let normalized_target = target_path.replace('\\', "/");
    let normalized_cwd = cwd.replace('\\', "/");
    let cwd_prefix = format!("{}/", normalized_cwd.trim_end_matches('/'));
    normalized_target
        .strip_prefix(&cwd_prefix)
        .unwrap_or(&normalized_target)
        .to_string()
}

fn extract_fence_language(language: &str) -> String {
    let raw = language.split_whitespace().next().unwrap_or("text");
    if raw.is_empty() {
        "text".to_string()
    } else if raw == "gitignore" {
        "ini".to_string()
    } else {
        raw.to_string()
    }
}

fn is_likely_path_candidate(path: &str) -> bool {
    is_windows_drive_path(path)
        || path.starts_with("\\\\")
        || path.starts_with("~/")
        || path.starts_with("./")
        || path.starts_with("../")
        || (path.starts_with('/') && looks_like_posix_filesystem_path(path))
        || looks_like_relative_file_path(path)
}

fn looks_like_posix_filesystem_path(path: &str) -> bool {
    POSIX_FILE_ROOT_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
        || has_position_suffix(path)
        || basename_of_path(path).contains('.')
}

fn looks_like_relative_file_path(path: &str) -> bool {
    if path.contains('/') {
        return path
            .split('/')
            .all(|segment| !segment.is_empty() && is_safe_path_segment(segment));
    }
    basename_of_path(path).contains('.')
}

fn is_safe_path_segment(segment: &str) -> bool {
    segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn is_relative_path(path: &str) -> bool {
    path.starts_with("~/")
        || path.starts_with("./")
        || path.starts_with("../")
        || (!path.starts_with('/') && !is_windows_drive_path(path) && !path.starts_with("\\\\"))
}

fn has_external_scheme(path: &str) -> bool {
    let Some(index) = path.find(':') else {
        return false;
    };
    let scheme = &path[..index];
    if scheme.len() == 1 && scheme.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    !path[index + 1..]
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == ':')
}

fn is_windows_drive_path(path: &str) -> bool {
    path.len() >= 3
        && path.as_bytes()[0].is_ascii_alphabetic()
        && path.as_bytes()[1] == b':'
        && matches!(path.as_bytes()[2], b'/' | b'\\')
}

fn normalize_windows_drive_path(path: &str) -> String {
    if path.len() >= 4 && path.as_bytes()[0] == b'/' && path.as_bytes()[2] == b':' {
        path[1..].to_string()
    } else {
        path.to_string()
    }
}

fn has_position_suffix(path: &str) -> bool {
    let parts = path.rsplit(':').take(2).collect::<Vec<_>>();
    parts.len() == 2 && parts[0].chars().all(|ch| ch.is_ascii_digit()) && !parts[0].is_empty()
}

fn basename_of_path(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn path_parent_segments(path: &str) -> Vec<String> {
    let mut segments = path
        .replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    segments.pop();
    segments
}

fn decode_uri_component(value: &str) -> String {
    let mut output = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                output.push(hex as char);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index] as char);
        index += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_chat_markdown_file_uri_rewrite_and_anchor_contracts() {
        let path =
            "/Users/yashsingh/p/sco/claude-code-extract/src/utils/permissions/PermissionRule.ts";
        assert_eq!(
            rewrite_markdown_file_uri_href(Some(&format!("file://{path}"))),
            Some(path.to_string())
        );
        let line = resolve_markdown_file_link_meta(
            Some(&format!("file://{path}#L1")),
            Some("/repo/project"),
        )
        .unwrap();
        assert_eq!(line.basename, "PermissionRule.ts");
        assert_eq!(line.target_path, format!("{path}:1"));
        assert_eq!(line.line, Some(1));
        assert_eq!(line.column, None);

        let column = resolve_markdown_file_link_meta(
            Some(&format!("file://{path}#L1C7")),
            Some("/repo/project"),
        )
        .unwrap();
        assert_eq!(column.target_path, format!("{path}:1:7"));
        assert_eq!(column.line, Some(1));
        assert_eq!(column.column, Some(7));
    }

    #[test]
    fn ports_chat_markdown_duplicate_file_labels_and_web_link_contracts() {
        let text = "See [MessagesTimeline.tsx](file:///Users/yashsingh/p/t3code/apps/web/src/components/chat/MessagesTimeline.tsx) and [MessagesTimeline.tsx](file:///Users/yashsingh/p/t3code/apps/web/src/components/MessagesTimeline.tsx). [OpenAI](https://openai.com/docs)";
        let blocks = render_chat_markdown_blocks(text, Some("/repo/project"));
        let ChatMarkdownBlock::Paragraph(inlines) = &blocks[0] else {
            panic!("expected paragraph");
        };
        let labels = inlines
            .iter()
            .filter_map(|inline| match inline {
                ChatMarkdownInline::Link {
                    file: Some(file), ..
                } => Some(file.label.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                "MessagesTimeline.tsx · components/chat",
                "MessagesTimeline.tsx · src/components"
            ]
        );
        assert!(inlines.iter().any(|inline| matches!(
            inline,
            ChatMarkdownInline::Link { href, file: None, .. } if href == "https://openai.com/docs"
        )));
    }

    #[test]
    fn ports_chat_markdown_code_fence_contracts() {
        let blocks = render_chat_markdown_blocks("Before\n\n```gitignore\n.env\n```\n", None);
        assert_eq!(
            blocks,
            vec![
                ChatMarkdownBlock::Paragraph(vec![ChatMarkdownInline::Text("Before".to_string())]),
                ChatMarkdownBlock::CodeBlock {
                    language: "ini".to_string(),
                    code: ".env\n".to_string(),
                    copy_label: "Copy code",
                }
            ]
        );
    }

    #[test]
    fn ports_chat_markdown_heading_and_gfm_table_contracts() {
        let blocks = render_chat_markdown_blocks(
            "## Summary\n\n| File | Status |\n| --- | --- |\n| `src/main.rs` | [OpenAI](https://openai.com/docs) |\n",
            Some("/repo/project"),
        );
        assert_eq!(
            blocks[0],
            ChatMarkdownBlock::Heading {
                level: 2,
                inlines: vec![ChatMarkdownInline::Text("Summary".to_string())],
            }
        );
        let ChatMarkdownBlock::Table { header, rows } = &blocks[1] else {
            panic!("expected table");
        };
        assert_eq!(
            header,
            &vec![
                vec![ChatMarkdownInline::Text("File".to_string())],
                vec![ChatMarkdownInline::Text("Status".to_string())],
            ]
        );
        assert_eq!(
            rows[0][0],
            vec![ChatMarkdownInline::Code("src/main.rs".to_string())]
        );
        assert!(matches!(
            &rows[0][1][0],
            ChatMarkdownInline::Link { href, file: None, .. } if href == "https://openai.com/docs"
        ));
    }

    #[test]
    fn ports_chat_markdown_task_ordered_blockquote_and_strike_contracts() {
        let blocks =
            render_chat_markdown_blocks("> quoted\n- [x] ~~done~~\n- [ ] todo\n2. second\n", None);
        assert_eq!(
            blocks[0],
            ChatMarkdownBlock::Blockquote(vec![ChatMarkdownInline::Text("quoted".to_string())])
        );
        assert_eq!(
            blocks[1],
            ChatMarkdownBlock::TaskListItem {
                checked: true,
                inlines: vec![ChatMarkdownInline::Strikethrough("done".to_string())],
            }
        );
        assert_eq!(
            blocks[2],
            ChatMarkdownBlock::TaskListItem {
                checked: false,
                inlines: vec![ChatMarkdownInline::Text("todo".to_string())],
            }
        );
        assert_eq!(
            blocks[3],
            ChatMarkdownBlock::OrderedListItem {
                number: 2,
                inlines: vec![ChatMarkdownInline::Text("second".to_string())],
            }
        );
    }
}

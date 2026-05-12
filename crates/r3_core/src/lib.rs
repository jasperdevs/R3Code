pub const APP_NAME: &str = "R3Code";

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedThreadRef {
    pub environment_id: String,
    pub thread_id: String,
}

impl ScopedThreadRef {
    pub fn new(environment_id: impl Into<String>, thread_id: impl Into<String>) -> Self {
        Self {
            environment_id: environment_id.into(),
            thread_id: thread_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedProjectRef {
    pub environment_id: String,
    pub project_id: String,
}

impl ScopedProjectRef {
    pub fn new(environment_id: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            environment_id: environment_id.into(),
            project_id: project_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRoute {
    Index,
    Thread(ThreadRouteTarget),
}

impl ChatRoute {
    pub fn renders_chat_view(&self) -> bool {
        matches!(self, Self::Thread(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadRouteTarget {
    Server { thread_ref: ScopedThreadRef },
    Draft { draft_id: String },
}

pub fn resolve_thread_route_target(
    environment_id: Option<&str>,
    thread_id: Option<&str>,
    draft_id: Option<&str>,
) -> Option<ThreadRouteTarget> {
    match (environment_id, thread_id) {
        (Some(environment_id), Some(thread_id))
            if !environment_id.is_empty() && !thread_id.is_empty() =>
        {
            Some(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new(environment_id, thread_id),
            })
        }
        _ => draft_id
            .filter(|draft_id| !draft_id.is_empty())
            .map(|draft_id| ThreadRouteTarget::Draft {
                draft_id: draft_id.to_string(),
            }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftThreadEnvMode {
    Local,
    Worktree,
}

pub const INLINE_TERMINAL_CONTEXT_PLACEHOLDER: char = '\u{FFFC}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalContextSelection {
    pub terminal_id: String,
    pub terminal_label: String,
    pub line_start: i64,
    pub line_end: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalContextDraft {
    pub id: String,
    pub thread_id: String,
    pub terminal_id: String,
    pub terminal_label: String,
    pub line_start: i64,
    pub line_end: i64,
    pub text: String,
    pub created_at: String,
}

impl TerminalContextDraft {
    pub fn selection(&self) -> TerminalContextSelection {
        TerminalContextSelection {
            terminal_id: self.terminal_id.clone(),
            terminal_label: self.terminal_label.clone(),
            line_start: self.line_start,
            line_end: self.line_end,
            text: self.text.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTerminalContextEntry {
    pub header: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedTerminalContexts {
    pub prompt_text: String,
    pub context_count: usize,
    pub preview_title: Option<String>,
    pub contexts: Vec<ParsedTerminalContextEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedUserMessageState {
    pub visible_text: String,
    pub copy_text: String,
    pub context_count: usize,
    pub preview_title: Option<String>,
    pub contexts: Vec<ParsedTerminalContextEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerSendState {
    pub trimmed_prompt: String,
    pub sendable_terminal_contexts: Vec<TerminalContextDraft>,
    pub expired_terminal_context_count: usize,
    pub has_sendable_content: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpiredTerminalContextToastVariant {
    Omitted,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiredTerminalContextToastCopy {
    pub title: String,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineTerminalContextInsertion {
    pub prompt: String,
    pub cursor: usize,
    pub context_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineTerminalContextRemoval {
    pub prompt: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerPromptSegment {
    Text {
        text: String,
    },
    Mention {
        path: String,
    },
    Skill {
        name: String,
    },
    TerminalContext {
        context: Option<TerminalContextDraft>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerTriggerKind {
    Path,
    SlashCommand,
    Skill,
}

impl ComposerTriggerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::SlashCommand => "slash-command",
            Self::Skill => "skill",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerSlashCommand {
    Model,
    Plan,
    Default,
}

impl ComposerSlashCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Plan => "plan",
            Self::Default => "default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerTrigger {
    pub kind: ComposerTriggerKind,
    pub query: String,
    pub range_start: usize,
    pub range_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRangeReplacement {
    pub text: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectEntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    pub path: String,
    pub kind: ProjectEntryKind,
    pub parent_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderSlashCommandInput {
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderSlashCommand {
    pub name: String,
    pub description: Option<String>,
    pub input: Option<ServerProviderSlashCommandInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderSkill {
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub scope: Option<String>,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub short_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerCommandItem {
    Path {
        id: String,
        path: String,
        path_kind: ProjectEntryKind,
        label: String,
        description: String,
    },
    SlashCommand {
        id: String,
        command: ComposerSlashCommand,
        label: String,
        description: String,
    },
    ProviderSlashCommand {
        id: String,
        provider: String,
        command: ServerProviderSlashCommand,
        label: String,
        description: String,
    },
    Skill {
        id: String,
        provider: String,
        skill: ServerProviderSkill,
        label: String,
        description: String,
    },
}

impl ComposerCommandItem {
    pub fn id(&self) -> &str {
        match self {
            Self::Path { id, .. }
            | Self::SlashCommand { id, .. }
            | Self::ProviderSlashCommand { id, .. }
            | Self::Skill { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerCommandGroup {
    pub id: String,
    pub label: Option<String>,
    pub items: Vec<ComposerCommandItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerCommandSelection {
    pub range_start: usize,
    pub range_end: usize,
    pub replacement: String,
    pub interaction_mode: Option<ComposerSlashCommand>,
    pub open_model_picker: bool,
    pub focus_editor_after_replace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerMenuNudgeDirection {
    ArrowDown,
    ArrowUp,
}

pub fn normalize_terminal_context_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim_matches('\n').to_string()
}

pub fn has_terminal_context_text(text: &str) -> bool {
    !normalize_terminal_context_text(text).is_empty()
}

pub fn is_terminal_context_expired(text: &str) -> bool {
    !has_terminal_context_text(text)
}

pub fn filter_terminal_contexts_with_text(
    contexts: &[TerminalContextDraft],
) -> Vec<TerminalContextDraft> {
    contexts
        .iter()
        .filter(|context| has_terminal_context_text(&context.text))
        .cloned()
        .collect()
}

fn preview_terminal_context_text(text: &str) -> String {
    let normalized = normalize_terminal_context_text(text);
    if normalized.is_empty() {
        return String::new();
    }
    let mut visible_lines = normalized.lines().take(3).collect::<Vec<_>>();
    if normalized.lines().count() > 3 {
        visible_lines.push("...");
    }
    let preview = visible_lines.join("\n");
    if preview.chars().count() > 180 {
        format!("{}...", preview.chars().take(177).collect::<String>())
    } else {
        preview
    }
}

pub fn normalize_terminal_context_selection(
    selection: &TerminalContextSelection,
) -> Option<TerminalContextSelection> {
    let text = normalize_terminal_context_text(&selection.text);
    let terminal_id = selection.terminal_id.trim();
    let terminal_label = selection.terminal_label.trim();
    if text.is_empty() || terminal_id.is_empty() || terminal_label.is_empty() {
        return None;
    }
    let line_start = selection.line_start.max(1);
    let line_end = selection.line_end.max(line_start);
    Some(TerminalContextSelection {
        terminal_id: terminal_id.to_string(),
        terminal_label: terminal_label.to_string(),
        line_start,
        line_end,
        text,
    })
}

pub fn format_terminal_context_range(line_start: i64, line_end: i64) -> String {
    if line_start == line_end {
        format!("line {line_start}")
    } else {
        format!("lines {line_start}-{line_end}")
    }
}

pub fn format_terminal_context_label(selection: &TerminalContextSelection) -> String {
    format!(
        "{} {}",
        selection.terminal_label,
        format_terminal_context_range(selection.line_start, selection.line_end)
    )
}

pub fn format_inline_terminal_context_label(selection: &TerminalContextSelection) -> String {
    let terminal_label = selection
        .terminal_label
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");
    let range = if selection.line_start == selection.line_end {
        selection.line_start.to_string()
    } else {
        format!("{}-{}", selection.line_start, selection.line_end)
    };
    format!("@{terminal_label}:{range}")
}

pub fn build_terminal_context_preview_title(
    contexts: &[TerminalContextSelection],
) -> Option<String> {
    let previews = contexts
        .iter()
        .filter_map(|context| normalize_terminal_context_selection(context))
        .filter_map(|context| {
            let preview = preview_terminal_context_text(&context.text);
            if preview.is_empty() {
                Some(format_terminal_context_label(&context))
            } else {
                Some(format!(
                    "{}\n{}",
                    format_terminal_context_label(&context),
                    preview
                ))
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    (!previews.is_empty()).then_some(previews)
}

fn build_terminal_context_body_lines(selection: &TerminalContextSelection) -> Vec<String> {
    normalize_terminal_context_text(&selection.text)
        .lines()
        .enumerate()
        .map(|(index, line)| format!("  {} | {}", selection.line_start + index as i64, line))
        .collect()
}

pub fn build_terminal_context_block(contexts: &[TerminalContextSelection]) -> String {
    let normalized_contexts = contexts
        .iter()
        .filter_map(normalize_terminal_context_selection)
        .collect::<Vec<_>>();
    if normalized_contexts.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for (index, context) in normalized_contexts.iter().enumerate() {
        lines.push(format!("- {}:", format_terminal_context_label(context)));
        lines.extend(build_terminal_context_body_lines(context));
        if index < normalized_contexts.len() - 1 {
            lines.push(String::new());
        }
    }

    let mut block = vec!["<terminal_context>".to_string()];
    block.extend(lines);
    block.push("</terminal_context>".to_string());
    block.join("\n")
}

pub fn materialize_inline_terminal_context_prompt(
    prompt: &str,
    contexts: &[TerminalContextSelection],
) -> String {
    let mut next_context_index = 0;
    let mut result = String::new();

    for character in prompt.chars() {
        if character != INLINE_TERMINAL_CONTEXT_PLACEHOLDER {
            result.push(character);
            continue;
        }
        if let Some(context) = contexts.get(next_context_index) {
            result.push_str(&format_inline_terminal_context_label(context));
        }
        next_context_index += 1;
    }

    result
}

pub fn append_terminal_contexts_to_prompt(
    prompt: &str,
    contexts: &[TerminalContextSelection],
) -> String {
    let trimmed_prompt = materialize_inline_terminal_context_prompt(prompt, contexts)
        .trim()
        .to_string();
    let context_block = build_terminal_context_block(contexts);
    if context_block.is_empty() {
        return trimmed_prompt;
    }
    if trimmed_prompt.is_empty() {
        context_block
    } else {
        format!("{trimmed_prompt}\n\n{context_block}")
    }
}

pub fn extract_trailing_terminal_contexts(prompt: &str) -> ExtractedTerminalContexts {
    let trimmed_end = prompt.trim_end();
    let close_tag = "\n</terminal_context>";
    if !trimmed_end.ends_with(close_tag) {
        return ExtractedTerminalContexts {
            prompt_text: prompt.to_string(),
            context_count: 0,
            preview_title: None,
            contexts: Vec::new(),
        };
    }

    let body_end = trimmed_end.len() - close_tag.len();
    let open_tag = "<terminal_context>\n";
    let Some(open_index) = trimmed_end[..body_end].rfind(open_tag) else {
        return ExtractedTerminalContexts {
            prompt_text: prompt.to_string(),
            context_count: 0,
            preview_title: None,
            contexts: Vec::new(),
        };
    };

    let body = &trimmed_end[open_index + open_tag.len()..body_end];
    let contexts = parse_terminal_context_entries(body);
    let preview_title = if contexts.is_empty() {
        None
    } else {
        Some(
            contexts
                .iter()
                .map(|context| {
                    if context.body.is_empty() {
                        context.header.clone()
                    } else {
                        format!("{}\n{}", context.header, context.body)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
        )
    };

    ExtractedTerminalContexts {
        prompt_text: prompt[..open_index].trim_end_matches('\n').to_string(),
        context_count: contexts.len(),
        preview_title,
        contexts,
    }
}

pub fn derive_displayed_user_message_state(prompt: &str) -> DisplayedUserMessageState {
    let extracted = extract_trailing_terminal_contexts(prompt);
    DisplayedUserMessageState {
        visible_text: extracted.prompt_text,
        copy_text: prompt.to_string(),
        context_count: extracted.context_count,
        preview_title: extracted.preview_title,
        contexts: extracted.contexts,
    }
}

fn parse_terminal_context_entries(block: &str) -> Vec<ParsedTerminalContextEntry> {
    let mut entries = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;

    let commit_current = |entries: &mut Vec<ParsedTerminalContextEntry>,
                          current: &mut Option<(String, Vec<String>)>| {
        if let Some((header, body_lines)) = current.take() {
            entries.push(ParsedTerminalContextEntry {
                header,
                body: body_lines.join("\n").trim_end().to_string(),
            });
        }
    };

    for raw_line in block.split('\n') {
        if let Some(header) = raw_line
            .strip_prefix("- ")
            .and_then(|line| line.strip_suffix(':'))
        {
            commit_current(&mut entries, &mut current);
            current = Some((header.to_string(), Vec::new()));
            continue;
        }
        let Some((_, body_lines)) = current.as_mut() else {
            continue;
        };
        if let Some(line) = raw_line.strip_prefix("  ") {
            body_lines.push(line.to_string());
        } else if raw_line.is_empty() {
            body_lines.push(String::new());
        }
    }

    commit_current(&mut entries, &mut current);
    entries
}

pub fn count_inline_terminal_context_placeholders(prompt: &str) -> usize {
    prompt
        .chars()
        .filter(|character| *character == INLINE_TERMINAL_CONTEXT_PLACEHOLDER)
        .count()
}

pub fn ensure_inline_terminal_context_placeholders(
    prompt: &str,
    terminal_context_count: usize,
) -> String {
    let missing_count =
        terminal_context_count.saturating_sub(count_inline_terminal_context_placeholders(prompt));
    if missing_count == 0 {
        return prompt.to_string();
    }
    format!(
        "{}{}",
        INLINE_TERMINAL_CONTEXT_PLACEHOLDER
            .to_string()
            .repeat(missing_count),
        prompt
    )
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn char_at(text: &str, char_index: usize) -> Option<char> {
    text.chars().nth(char_index)
}

fn is_inline_terminal_context_boundary_whitespace(character: Option<char>) -> bool {
    matches!(character, None | Some(' ' | '\n' | '\t' | '\r'))
}

pub fn insert_inline_terminal_context_placeholder(
    prompt: &str,
    cursor_input: isize,
) -> InlineTerminalContextInsertion {
    let char_len = prompt.chars().count();
    let cursor = cursor_input.max(0) as usize;
    let cursor = cursor.min(char_len);
    let needs_leading_space = !is_inline_terminal_context_boundary_whitespace(
        cursor
            .checked_sub(1)
            .and_then(|index| char_at(prompt, index)),
    );
    let replacement = format!(
        "{}{} ",
        if needs_leading_space { " " } else { "" },
        INLINE_TERMINAL_CONTEXT_PLACEHOLDER
    );
    let range_end = if char_at(prompt, cursor) == Some(' ') {
        cursor + 1
    } else {
        cursor
    };
    let cursor_byte = char_to_byte_index(prompt, cursor);
    let range_end_byte = char_to_byte_index(prompt, range_end);
    let next_prompt = format!(
        "{}{}{}",
        &prompt[..cursor_byte],
        replacement,
        &prompt[range_end_byte..]
    );
    InlineTerminalContextInsertion {
        prompt: next_prompt,
        cursor: cursor + replacement.chars().count(),
        context_index: count_inline_terminal_context_placeholders(
            &prompt[..char_to_byte_index(prompt, cursor)],
        ),
    }
}

pub fn strip_inline_terminal_context_placeholders(prompt: &str) -> String {
    prompt.replace(INLINE_TERMINAL_CONTEXT_PLACEHOLDER, "")
}

pub fn remove_inline_terminal_context_placeholder(
    prompt: &str,
    context_index: isize,
) -> InlineTerminalContextRemoval {
    if context_index < 0 {
        return InlineTerminalContextRemoval {
            prompt: prompt.to_string(),
            cursor: prompt.chars().count(),
        };
    }

    let mut placeholder_index = 0;
    for (char_index, (byte_index, character)) in prompt.char_indices().enumerate() {
        if character != INLINE_TERMINAL_CONTEXT_PLACEHOLDER {
            continue;
        }
        if placeholder_index == context_index as usize {
            let next_byte_index = byte_index + character.len_utf8();
            return InlineTerminalContextRemoval {
                prompt: format!("{}{}", &prompt[..byte_index], &prompt[next_byte_index..]),
                cursor: char_index,
            };
        }
        placeholder_index += 1;
    }

    InlineTerminalContextRemoval {
        prompt: prompt.to_string(),
        cursor: prompt.chars().count(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InlineTokenMatch {
    Mention {
        value: String,
        start: usize,
        end: usize,
    },
    Skill {
        value: String,
        start: usize,
        end: usize,
    },
}

impl InlineTokenMatch {
    fn start(&self) -> usize {
        match self {
            Self::Mention { start, .. } | Self::Skill { start, .. } => *start,
        }
    }
}

fn push_composer_text_segment(segments: &mut Vec<ComposerPromptSegment>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(ComposerPromptSegment::Text {
        text: previous_text,
    }) = segments.last_mut()
    {
        previous_text.push_str(text);
        return;
    }
    segments.push(ComposerPromptSegment::Text {
        text: text.to_string(),
    });
}

fn is_prompt_token_boundary(text_chars: &[char], index: usize) -> bool {
    index == 0 || text_chars[index - 1].is_whitespace()
}

fn find_mention_match_at(text_chars: &[char], index: usize) -> Option<InlineTokenMatch> {
    if text_chars.get(index) != Some(&'@') || !is_prompt_token_boundary(text_chars, index) {
        return None;
    }

    let mut end = index + 1;
    while let Some(character) = text_chars.get(end) {
        if character.is_whitespace() || *character == '@' {
            break;
        }
        end += 1;
    }
    if end == index + 1
        || !text_chars
            .get(end)
            .is_some_and(|character| character.is_whitespace())
    {
        return None;
    }
    Some(InlineTokenMatch::Mention {
        value: text_chars[index + 1..end].iter().collect(),
        start: index,
        end,
    })
}

fn is_skill_name_start(character: char) -> bool {
    character.is_ascii_alphabetic()
}

fn is_skill_name_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, ':' | '_' | '-')
}

fn find_skill_match_at(text_chars: &[char], index: usize) -> Option<InlineTokenMatch> {
    if text_chars.get(index) != Some(&'$') || !is_prompt_token_boundary(text_chars, index) {
        return None;
    }

    let Some(first_name_char) = text_chars.get(index + 1).copied() else {
        return None;
    };
    if !is_skill_name_start(first_name_char) {
        return None;
    }

    let mut end = index + 2;
    while let Some(character) = text_chars.get(end).copied() {
        if !is_skill_name_continue(character) {
            break;
        }
        end += 1;
    }
    if !text_chars
        .get(end)
        .is_some_and(|character| character.is_whitespace())
    {
        return None;
    }
    Some(InlineTokenMatch::Skill {
        value: text_chars[index + 1..end].iter().collect(),
        start: index,
        end,
    })
}

fn collect_inline_token_matches(text: &str) -> Vec<InlineTokenMatch> {
    let text_chars = text.chars().collect::<Vec<_>>();
    let mut matches = Vec::new();

    for index in 0..text_chars.len() {
        if let Some(token_match) = find_mention_match_at(&text_chars, index) {
            matches.push(token_match);
        }
        if let Some(token_match) = find_skill_match_at(&text_chars, index) {
            matches.push(token_match);
        }
    }

    matches.sort_by_key(InlineTokenMatch::start);
    matches
}

fn split_prompt_text_into_composer_segments(text: &str) -> Vec<ComposerPromptSegment> {
    if text.is_empty() {
        return Vec::new();
    }

    let token_matches = collect_inline_token_matches(text);
    let mut segments = Vec::new();
    let mut cursor = 0;

    for token_match in token_matches {
        if token_match.start() < cursor {
            continue;
        }

        if token_match.start() > cursor {
            let start_byte = char_to_byte_index(text, cursor);
            let end_byte = char_to_byte_index(text, token_match.start());
            push_composer_text_segment(&mut segments, &text[start_byte..end_byte]);
        }

        match token_match {
            InlineTokenMatch::Mention { value, end, .. } => {
                segments.push(ComposerPromptSegment::Mention { path: value });
                cursor = end;
            }
            InlineTokenMatch::Skill { value, end, .. } => {
                segments.push(ComposerPromptSegment::Skill { name: value });
                cursor = end;
            }
        }
    }

    let text_len = text.chars().count();
    if cursor < text_len {
        let cursor_byte = char_to_byte_index(text, cursor);
        push_composer_text_segment(&mut segments, &text[cursor_byte..]);
    }

    segments
}

fn split_prompt_into_composer_segments_with_contexts(
    prompt: &str,
    terminal_contexts: &[TerminalContextDraft],
) -> Vec<ComposerPromptSegment> {
    if prompt.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut text_start = 0;
    let mut terminal_context_index = 0;

    for (char_index, (byte_index, character)) in prompt.char_indices().enumerate() {
        if character != INLINE_TERMINAL_CONTEXT_PLACEHOLDER {
            continue;
        }

        if char_index > text_start {
            let text_start_byte = char_to_byte_index(prompt, text_start);
            segments.extend(split_prompt_text_into_composer_segments(
                &prompt[text_start_byte..byte_index],
            ));
        }

        segments.push(ComposerPromptSegment::TerminalContext {
            context: terminal_contexts.get(terminal_context_index).cloned(),
        });
        terminal_context_index += 1;
        text_start = char_index + 1;
    }

    if text_start < prompt.chars().count() {
        let text_start_byte = char_to_byte_index(prompt, text_start);
        segments.extend(split_prompt_text_into_composer_segments(
            &prompt[text_start_byte..],
        ));
    }

    segments
}

pub fn split_prompt_into_composer_segments(prompt: &str) -> Vec<ComposerPromptSegment> {
    split_prompt_into_composer_segments_with_contexts(prompt, &[])
}

pub fn split_prompt_into_composer_segments_for_terminal_contexts(
    prompt: &str,
    terminal_contexts: &[TerminalContextDraft],
) -> Vec<ComposerPromptSegment> {
    split_prompt_into_composer_segments_with_contexts(prompt, terminal_contexts)
}

pub fn selection_touches_mention_boundary(prompt: &str, start: usize, end: usize) -> bool {
    if prompt.is_empty() || start >= end {
        return false;
    }

    let mut text_start = 0;
    let prompt_len = prompt.chars().count();

    for (char_index, (byte_index, character)) in prompt.char_indices().enumerate() {
        if character != INLINE_TERMINAL_CONTEXT_PLACEHOLDER {
            continue;
        }
        if char_index > text_start {
            let text_start_byte = char_to_byte_index(prompt, text_start);
            if text_slice_selection_touches_mention_boundary(
                prompt,
                &prompt[text_start_byte..byte_index],
                text_start,
                start,
                end,
            ) {
                return true;
            }
        }
        text_start = char_index + 1;
    }

    if text_start < prompt_len {
        let text_start_byte = char_to_byte_index(prompt, text_start);
        return text_slice_selection_touches_mention_boundary(
            prompt,
            &prompt[text_start_byte..],
            text_start,
            start,
            end,
        );
    }

    false
}

fn text_slice_selection_touches_mention_boundary(
    prompt: &str,
    text: &str,
    prompt_offset: usize,
    start: usize,
    end: usize,
) -> bool {
    collect_inline_token_matches(text)
        .into_iter()
        .any(|token_match| match token_match {
            InlineTokenMatch::Mention {
                start: mention_start,
                end: mention_end,
                ..
            } => {
                let mention_start = prompt_offset + mention_start;
                let mention_end = prompt_offset + mention_end;
                let before_mention_index = mention_start.checked_sub(1);
                let touches_before = before_mention_index.is_some_and(|index| {
                    char_at(prompt, index).is_some_and(char::is_whitespace)
                        && start <= index
                        && index < end
                });
                let touches_after = mention_end < prompt.chars().count()
                    && char_at(prompt, mention_end).is_some_and(char::is_whitespace)
                    && start <= mention_end
                    && mention_end < end;
                touches_before || touches_after
            }
            InlineTokenMatch::Skill { .. } => false,
        })
}

fn composer_text_len(text: &str) -> usize {
    text.chars().count()
}

fn clamp_composer_cursor(text: &str, cursor_input: f64) -> usize {
    let text_len = composer_text_len(text);
    if !cursor_input.is_finite() {
        return text_len;
    }
    cursor_input.floor().max(0.0).min(text_len as f64) as usize
}

fn is_composer_whitespace(character: Option<char>) -> bool {
    matches!(
        character,
        Some(' ' | '\n' | '\t' | '\r' | INLINE_TERMINAL_CONTEXT_PLACEHOLDER)
    )
}

fn token_start_for_cursor(text: &str, cursor: usize) -> usize {
    let mut index = cursor;
    while index > 0 && !is_composer_whitespace(char_at(text, index - 1)) {
        index -= 1;
    }
    index
}

fn is_inline_token_segment(segment: &ComposerPromptSegment) -> bool {
    !matches!(segment, ComposerPromptSegment::Text { .. })
}

fn expanded_inline_token_length(segment: &ComposerPromptSegment) -> usize {
    match segment {
        ComposerPromptSegment::Mention { path } => composer_text_len(path) + 1,
        ComposerPromptSegment::Skill { name } => composer_text_len(name) + 1,
        ComposerPromptSegment::TerminalContext { .. } => 1,
        ComposerPromptSegment::Text { text } => composer_text_len(text),
    }
}

fn collapsed_segment_length(segment: &ComposerPromptSegment) -> usize {
    match segment {
        ComposerPromptSegment::Text { text } => composer_text_len(text),
        ComposerPromptSegment::Mention { .. }
        | ComposerPromptSegment::Skill { .. }
        | ComposerPromptSegment::TerminalContext { .. } => 1,
    }
}

pub fn expand_collapsed_composer_cursor(text: &str, cursor_input: f64) -> usize {
    let collapsed_cursor = clamp_composer_cursor(text, cursor_input);
    let segments = split_prompt_into_composer_segments(text);
    if segments.is_empty() {
        return collapsed_cursor;
    }

    let mut remaining = collapsed_cursor;
    let mut expanded_cursor = 0;

    for segment in segments {
        if is_inline_token_segment(&segment) {
            let expanded_length = expanded_inline_token_length(&segment);
            if remaining <= 1 {
                return expanded_cursor + if remaining == 0 { 0 } else { expanded_length };
            }
            remaining -= 1;
            expanded_cursor += expanded_length;
            continue;
        }

        let segment_length = collapsed_segment_length(&segment);
        if remaining <= segment_length {
            return expanded_cursor + remaining;
        }
        remaining -= segment_length;
        expanded_cursor += segment_length;
    }

    expanded_cursor
}

fn clamp_collapsed_composer_cursor_for_segments(
    segments: &[ComposerPromptSegment],
    cursor_input: f64,
) -> usize {
    let collapsed_length = segments.iter().map(collapsed_segment_length).sum::<usize>();
    if !cursor_input.is_finite() {
        return collapsed_length;
    }
    cursor_input.floor().max(0.0).min(collapsed_length as f64) as usize
}

pub fn clamp_collapsed_composer_cursor(text: &str, cursor_input: f64) -> usize {
    clamp_collapsed_composer_cursor_for_segments(
        &split_prompt_into_composer_segments(text),
        cursor_input,
    )
}

pub fn collapse_expanded_composer_cursor(text: &str, cursor_input: f64) -> usize {
    let expanded_cursor = clamp_composer_cursor(text, cursor_input);
    let segments = split_prompt_into_composer_segments(text);
    if segments.is_empty() {
        return expanded_cursor;
    }

    let mut remaining = expanded_cursor;
    let mut collapsed_cursor = 0;

    for segment in segments {
        if is_inline_token_segment(&segment) {
            let expanded_length = expanded_inline_token_length(&segment);
            if remaining == 0 {
                return collapsed_cursor;
            }
            if remaining <= expanded_length {
                return collapsed_cursor + 1;
            }
            remaining -= expanded_length;
            collapsed_cursor += 1;
            continue;
        }

        let segment_length = collapsed_segment_length(&segment);
        if remaining <= segment_length {
            return collapsed_cursor + remaining;
        }
        remaining -= segment_length;
        collapsed_cursor += segment_length;
    }

    collapsed_cursor
}

pub fn is_collapsed_cursor_adjacent_to_inline_token(
    text: &str,
    cursor_input: f64,
    direction: ComposerCursorAdjacencyDirection,
) -> bool {
    let segments = split_prompt_into_composer_segments(text);
    if !segments.iter().any(is_inline_token_segment) {
        return false;
    }

    let cursor = clamp_collapsed_composer_cursor_for_segments(&segments, cursor_input);
    let mut collapsed_offset = 0;

    for segment in segments {
        if is_inline_token_segment(&segment) {
            if direction == ComposerCursorAdjacencyDirection::Left && cursor == collapsed_offset + 1
            {
                return true;
            }
            if direction == ComposerCursorAdjacencyDirection::Right && cursor == collapsed_offset {
                return true;
            }
        }
        collapsed_offset += collapsed_segment_length(&segment);
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerCursorAdjacencyDirection {
    Left,
    Right,
}

pub fn is_collapsed_cursor_adjacent_to_mention(
    text: &str,
    cursor_input: f64,
    direction: ComposerCursorAdjacencyDirection,
) -> bool {
    is_collapsed_cursor_adjacent_to_inline_token(text, cursor_input, direction)
}

fn slice_chars(text: &str, start: usize, end: usize) -> &str {
    if start >= end {
        let end_byte = char_to_byte_index(text, end);
        return &text[end_byte..end_byte];
    }
    let start_byte = char_to_byte_index(text, start);
    let end_byte = char_to_byte_index(text, end);
    &text[start_byte..end_byte]
}

fn last_newline_before_or_at(text: &str, end_index: usize) -> Option<usize> {
    text.chars()
        .enumerate()
        .take(end_index.saturating_add(1))
        .filter_map(|(index, character)| (character == '\n').then_some(index))
        .last()
}

pub fn detect_composer_trigger(text: &str, cursor_input: f64) -> Option<ComposerTrigger> {
    let cursor = clamp_composer_cursor(text, cursor_input);
    let line_search_end = cursor.saturating_sub(1);
    let line_start = last_newline_before_or_at(text, line_search_end)
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_prefix = slice_chars(text, line_start, cursor);

    if let Some(command_query) = line_prefix.strip_prefix('/') {
        if command_query
            .chars()
            .all(|character| !character.is_whitespace())
        {
            return Some(ComposerTrigger {
                kind: ComposerTriggerKind::SlashCommand,
                query: command_query.to_string(),
                range_start: line_start,
                range_end: cursor,
            });
        }
    }

    let token_start = token_start_for_cursor(text, cursor);
    let token = slice_chars(text, token_start, cursor);
    if let Some(query) = token.strip_prefix('$') {
        return Some(ComposerTrigger {
            kind: ComposerTriggerKind::Skill,
            query: query.to_string(),
            range_start: token_start,
            range_end: cursor,
        });
    }
    token.strip_prefix('@').map(|query| ComposerTrigger {
        kind: ComposerTriggerKind::Path,
        query: query.to_string(),
        range_start: token_start,
        range_end: cursor,
    })
}

pub fn parse_standalone_composer_slash_command(text: &str) -> Option<ComposerSlashCommand> {
    match text.trim().to_ascii_lowercase().as_str() {
        "/plan" => Some(ComposerSlashCommand::Plan),
        "/default" => Some(ComposerSlashCommand::Default),
        _ => None,
    }
}

pub fn replace_text_range(
    text: &str,
    range_start: f64,
    range_end: f64,
    replacement: &str,
) -> TextRangeReplacement {
    let safe_start = clamp_composer_cursor(text, range_start);
    let safe_end = clamp_composer_cursor(text, range_end).max(safe_start);
    let start_byte = char_to_byte_index(text, safe_start);
    let end_byte = char_to_byte_index(text, safe_end);
    let next_text = format!(
        "{}{}{}",
        &text[..start_byte],
        replacement,
        &text[end_byte..]
    );

    TextRangeReplacement {
        text: next_text,
        cursor: safe_start + composer_text_len(replacement),
    }
}

pub fn extend_replacement_range_for_trailing_space(
    text: &str,
    range_end: usize,
    replacement: &str,
) -> usize {
    if !replacement.ends_with(' ') {
        return range_end;
    }
    if char_at(text, range_end) == Some(' ') {
        range_end + 1
    } else {
        range_end
    }
}

fn basename_of_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(path)
}

fn title_case_skill_words(value: &str) -> String {
    value
        .split(|character: char| character.is_whitespace() || matches!(character, ':' | '_' | '-'))
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_provider_skill_display_name(skill: &ServerProviderSkill) -> String {
    skill
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|display_name| !display_name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| title_case_skill_words(&skill.name))
}

fn normalize_path_separators(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn format_provider_skill_install_source(skill: &ServerProviderSkill) -> Option<String> {
    let normalized_path = normalize_path_separators(&skill.path);
    if normalized_path.contains("/.codex/plugins/") || normalized_path.contains("/.agents/plugins/")
    {
        return Some("App".to_string());
    }

    let normalized_scope = skill.scope.as_deref()?.trim().to_ascii_lowercase();
    if normalized_scope.is_empty() {
        return None;
    }

    Some(match normalized_scope.as_str() {
        "system" => "System".to_string(),
        "project" | "workspace" | "local" => "Project".to_string(),
        "user" | "personal" => "Personal".to_string(),
        _ => title_case_skill_words(&normalized_scope),
    })
}

pub fn normalize_search_query_trim_leading(input: &str, leading: char) -> String {
    input
        .trim()
        .trim_start_matches(leading)
        .to_ascii_lowercase()
}

fn score_slash_command_item(item: &ComposerCommandItem, query: &str) -> Option<usize> {
    let (primary_value, description) = match item {
        ComposerCommandItem::SlashCommand {
            command,
            description,
            ..
        } => (
            command.as_str().to_ascii_lowercase(),
            description.to_ascii_lowercase(),
        ),
        ComposerCommandItem::ProviderSlashCommand {
            command,
            description,
            ..
        } => (
            command.name.to_ascii_lowercase(),
            description.to_ascii_lowercase(),
        ),
        _ => return None,
    };

    [
        score_query_match_with_boundary_markers(
            &primary_value,
            query,
            0,
            Some(2),
            Some(4),
            Some(6),
            Some(100),
            &["-", "_", "/"],
        ),
        score_query_match(&description, query, 20, Some(22), Some(24), Some(26), None),
    ]
    .into_iter()
    .flatten()
    .min()
}

pub fn search_slash_command_items(
    items: &[ComposerCommandItem],
    query: &str,
) -> Vec<ComposerCommandItem> {
    let normalized_query = normalize_search_query_trim_leading(query, '/');
    let slash_items = items
        .iter()
        .filter(|item| {
            matches!(
                item,
                ComposerCommandItem::SlashCommand { .. }
                    | ComposerCommandItem::ProviderSlashCommand { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    if normalized_query.is_empty() {
        return slash_items;
    }

    let mut ranked = slash_items
        .into_iter()
        .filter_map(|item| {
            let score = score_slash_command_item(&item, &normalized_query)?;
            let tie_breaker = match &item {
                ComposerCommandItem::SlashCommand { command, .. } => {
                    format!("0\0{}", command.as_str())
                }
                ComposerCommandItem::ProviderSlashCommand {
                    provider, command, ..
                } => {
                    format!("1\0{}\0{}", command.name, provider)
                }
                _ => String::new(),
            };
            Some((item, score, tie_breaker))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.2.cmp(&right.2)));
    ranked.into_iter().map(|(item, _, _)| item).collect()
}

fn score_provider_skill(skill: &ServerProviderSkill, query: &str) -> Option<usize> {
    let normalized_name = skill.name.to_ascii_lowercase();
    let normalized_label = format_provider_skill_display_name(skill).to_ascii_lowercase();
    let normalized_short_description = skill
        .short_description
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let normalized_description = skill
        .description
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let normalized_scope = skill
        .scope
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    [
        score_query_match_with_boundary_markers(
            &normalized_name,
            query,
            0,
            Some(2),
            Some(4),
            Some(6),
            Some(100),
            &["-", "_", "/"],
        ),
        score_query_match(
            &normalized_label,
            query,
            1,
            Some(3),
            Some(5),
            Some(7),
            Some(110),
        ),
        score_query_match(
            &normalized_short_description,
            query,
            20,
            Some(22),
            Some(24),
            Some(26),
            None,
        ),
        score_query_match(
            &normalized_description,
            query,
            30,
            Some(32),
            Some(34),
            Some(36),
            None,
        ),
        score_query_match(&normalized_scope, query, 40, Some(42), None, Some(44), None),
    ]
    .into_iter()
    .flatten()
    .min()
}

pub fn search_provider_skills(
    skills: &[ServerProviderSkill],
    query: &str,
) -> Vec<ServerProviderSkill> {
    search_provider_skills_with_limit(skills, query, usize::MAX)
}

pub fn search_provider_skills_with_limit(
    skills: &[ServerProviderSkill],
    query: &str,
    limit: usize,
) -> Vec<ServerProviderSkill> {
    let enabled_skills = skills
        .iter()
        .filter(|skill| skill.enabled)
        .cloned()
        .collect::<Vec<_>>();
    let normalized_query = normalize_search_query_trim_leading(query, '$');

    if normalized_query.is_empty() {
        return enabled_skills.into_iter().take(limit).collect();
    }

    let mut ranked = enabled_skills
        .into_iter()
        .filter_map(|skill| {
            let score = score_provider_skill(&skill, &normalized_query)?;
            let tie_breaker = format!(
                "{}\0{}",
                format_provider_skill_display_name(&skill).to_ascii_lowercase(),
                skill.name
            );
            Some((skill, score, tie_breaker))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.2.cmp(&right.2)));
    ranked
        .into_iter()
        .take(limit)
        .map(|(skill, _, _)| skill)
        .collect()
}

pub fn builtin_composer_slash_command_items() -> Vec<ComposerCommandItem> {
    vec![
        ComposerCommandItem::SlashCommand {
            id: "slash:model".to_string(),
            command: ComposerSlashCommand::Model,
            label: "/model".to_string(),
            description: "Switch response model for this thread".to_string(),
        },
        ComposerCommandItem::SlashCommand {
            id: "slash:plan".to_string(),
            command: ComposerSlashCommand::Plan,
            label: "/plan".to_string(),
            description: "Switch this thread into plan mode".to_string(),
        },
        ComposerCommandItem::SlashCommand {
            id: "slash:default".to_string(),
            command: ComposerSlashCommand::Default,
            label: "/default".to_string(),
            description: "Switch this thread back to normal build mode".to_string(),
        },
    ]
}

pub fn build_composer_menu_items(
    composer_trigger: Option<&ComposerTrigger>,
    workspace_entries: &[ProjectEntry],
    selected_provider: &str,
    provider_slash_commands: &[ServerProviderSlashCommand],
    provider_skills: &[ServerProviderSkill],
) -> Vec<ComposerCommandItem> {
    let Some(composer_trigger) = composer_trigger else {
        return Vec::new();
    };

    match composer_trigger.kind {
        ComposerTriggerKind::Path => workspace_entries
            .iter()
            .map(|entry| ComposerCommandItem::Path {
                id: format!(
                    "path:{}:{}",
                    match entry.kind {
                        ProjectEntryKind::File => "file",
                        ProjectEntryKind::Directory => "directory",
                    },
                    entry.path
                ),
                path: entry.path.clone(),
                path_kind: entry.kind,
                label: basename_of_path(&entry.path).to_string(),
                description: entry.parent_path.clone().unwrap_or_default(),
            })
            .collect(),
        ComposerTriggerKind::SlashCommand => {
            let mut slash_command_items = builtin_composer_slash_command_items();
            slash_command_items.extend(provider_slash_commands.iter().map(|command| {
                ComposerCommandItem::ProviderSlashCommand {
                    id: format!(
                        "provider-slash-command:{}:{}",
                        selected_provider, command.name
                    ),
                    provider: selected_provider.to_string(),
                    command: command.clone(),
                    label: format!("/{}", command.name),
                    description: command
                        .description
                        .clone()
                        .or_else(|| command.input.as_ref().map(|input| input.hint.clone()))
                        .unwrap_or_else(|| "Run provider command".to_string()),
                }
            }));

            let query = composer_trigger.query.trim().to_ascii_lowercase();
            if query.is_empty() {
                slash_command_items
            } else {
                search_slash_command_items(&slash_command_items, &query)
            }
        }
        ComposerTriggerKind::Skill => {
            search_provider_skills(provider_skills, &composer_trigger.query)
                .into_iter()
                .map(|skill| ComposerCommandItem::Skill {
                    id: format!("skill:{}:{}", selected_provider, skill.name),
                    provider: selected_provider.to_string(),
                    label: format_provider_skill_display_name(&skill),
                    description: skill
                        .short_description
                        .clone()
                        .or_else(|| skill.description.clone())
                        .or_else(|| skill.scope.as_ref().map(|scope| format!("{scope} skill")))
                        .unwrap_or_else(|| "Run provider skill".to_string()),
                    skill,
                })
                .collect()
        }
    }
}

pub fn group_composer_command_items(
    items: &[ComposerCommandItem],
    trigger_kind: Option<ComposerTriggerKind>,
    group_slash_command_sections: bool,
) -> Vec<ComposerCommandGroup> {
    if trigger_kind == Some(ComposerTriggerKind::Skill) {
        return if items.is_empty() {
            Vec::new()
        } else {
            vec![ComposerCommandGroup {
                id: "skills".to_string(),
                label: Some("Skills".to_string()),
                items: items.to_vec(),
            }]
        };
    }

    if trigger_kind != Some(ComposerTriggerKind::SlashCommand) || !group_slash_command_sections {
        return vec![ComposerCommandGroup {
            id: "default".to_string(),
            label: None,
            items: items.to_vec(),
        }];
    }

    let built_in_items = items
        .iter()
        .filter(|item| matches!(item, ComposerCommandItem::SlashCommand { .. }))
        .cloned()
        .collect::<Vec<_>>();
    let provider_items = items
        .iter()
        .filter(|item| matches!(item, ComposerCommandItem::ProviderSlashCommand { .. }))
        .cloned()
        .collect::<Vec<_>>();

    let mut groups = Vec::new();
    if !built_in_items.is_empty() {
        groups.push(ComposerCommandGroup {
            id: "built-in".to_string(),
            label: Some("Built-in".to_string()),
            items: built_in_items,
        });
    }
    if !provider_items.is_empty() {
        groups.push(ComposerCommandGroup {
            id: "provider".to_string(),
            label: Some("Provider".to_string()),
            items: provider_items,
        });
    }
    groups
}

pub fn composer_menu_search_key(trigger: Option<&ComposerTrigger>) -> Option<String> {
    trigger.map(|trigger| {
        format!(
            "{}:{}",
            trigger.kind.as_str(),
            trigger.query.trim().to_ascii_lowercase()
        )
    })
}

pub fn resolve_composer_menu_active_item_id(
    items: &[ComposerCommandItem],
    highlighted_item_id: Option<&str>,
    current_search_key: Option<&str>,
    highlighted_search_key: Option<&str>,
) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    if current_search_key == highlighted_search_key {
        if let Some(highlighted_item_id) = highlighted_item_id {
            if items.iter().any(|item| item.id() == highlighted_item_id) {
                return Some(highlighted_item_id.to_string());
            }
        }
    }

    items.first().map(|item| item.id().to_string())
}

pub fn nudge_composer_menu_highlight(
    items: &[ComposerCommandItem],
    highlighted_item_id: Option<&str>,
    direction: ComposerMenuNudgeDirection,
) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let highlighted_index =
        highlighted_item_id.and_then(|item_id| items.iter().position(|item| item.id() == item_id));
    let normalized_index = highlighted_index.unwrap_or(match direction {
        ComposerMenuNudgeDirection::ArrowDown => items.len().saturating_sub(1),
        ComposerMenuNudgeDirection::ArrowUp => 0,
    });
    let next_index = match direction {
        ComposerMenuNudgeDirection::ArrowDown => (normalized_index + 1) % items.len(),
        ComposerMenuNudgeDirection::ArrowUp => (normalized_index + items.len() - 1) % items.len(),
    };
    items.get(next_index).map(|item| item.id().to_string())
}

pub fn resolve_composer_command_selection(
    text: &str,
    trigger: &ComposerTrigger,
    item: &ComposerCommandItem,
) -> Option<ComposerCommandSelection> {
    match item {
        ComposerCommandItem::Path { path, .. } => {
            let replacement = format!("@{path} ");
            Some(ComposerCommandSelection {
                range_start: trigger.range_start,
                range_end: extend_replacement_range_for_trailing_space(
                    text,
                    trigger.range_end,
                    &replacement,
                ),
                replacement,
                interaction_mode: None,
                open_model_picker: false,
                focus_editor_after_replace: true,
            })
        }
        ComposerCommandItem::SlashCommand { command, .. } => {
            let (interaction_mode, open_model_picker, focus_editor_after_replace) = match command {
                ComposerSlashCommand::Model => (None, true, false),
                ComposerSlashCommand::Plan => (Some(ComposerSlashCommand::Plan), false, true),
                ComposerSlashCommand::Default => (Some(ComposerSlashCommand::Default), false, true),
            };
            Some(ComposerCommandSelection {
                range_start: trigger.range_start,
                range_end: trigger.range_end,
                replacement: String::new(),
                interaction_mode,
                open_model_picker,
                focus_editor_after_replace,
            })
        }
        ComposerCommandItem::ProviderSlashCommand { command, .. } => {
            let replacement = format!("/{} ", command.name);
            Some(ComposerCommandSelection {
                range_start: trigger.range_start,
                range_end: extend_replacement_range_for_trailing_space(
                    text,
                    trigger.range_end,
                    &replacement,
                ),
                replacement,
                interaction_mode: None,
                open_model_picker: false,
                focus_editor_after_replace: true,
            })
        }
        ComposerCommandItem::Skill { skill, .. } => {
            let replacement = format!("${} ", skill.name);
            Some(ComposerCommandSelection {
                range_start: trigger.range_start,
                range_end: extend_replacement_range_for_trailing_space(
                    text,
                    trigger.range_end,
                    &replacement,
                ),
                replacement,
                interaction_mode: None,
                open_model_picker: false,
                focus_editor_after_replace: true,
            })
        }
    }
}

pub fn derive_composer_send_state(
    prompt: &str,
    image_count: usize,
    terminal_contexts: &[TerminalContextDraft],
) -> ComposerSendState {
    let trimmed_prompt = strip_inline_terminal_context_placeholders(prompt)
        .trim()
        .to_string();
    let sendable_terminal_contexts = filter_terminal_contexts_with_text(terminal_contexts);
    let expired_terminal_context_count = terminal_contexts.len() - sendable_terminal_contexts.len();
    ComposerSendState {
        has_sendable_content: !trimmed_prompt.is_empty()
            || image_count > 0
            || !sendable_terminal_contexts.is_empty(),
        trimmed_prompt,
        sendable_terminal_contexts,
        expired_terminal_context_count,
    }
}

pub fn build_expired_terminal_context_toast_copy(
    expired_terminal_context_count: usize,
    variant: ExpiredTerminalContextToastVariant,
) -> ExpiredTerminalContextToastCopy {
    let count = expired_terminal_context_count.max(1);
    let noun = if count == 1 {
        "Expired terminal context"
    } else {
        "Expired terminal contexts"
    };
    match variant {
        ExpiredTerminalContextToastVariant::Empty => ExpiredTerminalContextToastCopy {
            title: format!("{noun} won't be sent"),
            description: "Remove it or re-add it to include terminal output.",
        },
        ExpiredTerminalContextToastVariant::Omitted => ExpiredTerminalContextToastCopy {
            title: format!("{noun} omitted from message"),
            description: "Re-add it if you want that terminal output included.",
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchToolbarEnvironmentOption {
    pub environment_id: String,
    pub project_id: String,
    pub label: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsRef {
    pub name: String,
    pub current: bool,
    pub is_default: bool,
    pub is_remote: bool,
    pub remote_name: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchSelectionTarget {
    pub checkout_cwd: String,
    pub next_worktree_path: Option<String>,
    pub reuse_existing_worktree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchToolbarState {
    pub environment_id: String,
    pub environment_label: String,
    pub environment_is_primary: bool,
    pub show_environment_picker: bool,
    pub effective_env_mode: DraftThreadEnvMode,
    pub env_locked: bool,
    pub env_mode_locked: bool,
    pub active_worktree_path: Option<String>,
    pub workspace_label: &'static str,
    pub branch_label: String,
    pub resolved_active_branch: Option<String>,
}

impl DraftThreadEnvMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Local => Self::Worktree,
            Self::Worktree => Self::Local,
        }
    }
}

fn normalize_display_label(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn is_generic_local_environment_label(label: &str) -> bool {
    matches!(
        label.trim().to_ascii_lowercase().as_str(),
        "local" | "local environment"
    )
}

pub fn resolve_environment_option_label(
    is_primary: bool,
    environment_id: &str,
    runtime_label: Option<&str>,
    saved_label: Option<&str>,
) -> String {
    let runtime_label = normalize_display_label(runtime_label);
    let saved_label = normalize_display_label(saved_label);

    if is_primary {
        return [runtime_label, saved_label]
            .into_iter()
            .flatten()
            .find(|label| !is_generic_local_environment_label(label))
            .unwrap_or("This device")
            .to_string();
    }

    runtime_label
        .or(saved_label)
        .unwrap_or(environment_id)
        .to_string()
}

pub fn resolve_env_mode_label(mode: DraftThreadEnvMode) -> &'static str {
    match mode {
        DraftThreadEnvMode::Local => "Current checkout",
        DraftThreadEnvMode::Worktree => "New worktree",
    }
}

pub fn resolve_current_workspace_label(active_worktree_path: Option<&str>) -> &'static str {
    if active_worktree_path.is_some() {
        "Current worktree"
    } else {
        resolve_env_mode_label(DraftThreadEnvMode::Local)
    }
}

pub fn resolve_locked_workspace_label(active_worktree_path: Option<&str>) -> &'static str {
    if active_worktree_path.is_some() {
        "Worktree"
    } else {
        "Local checkout"
    }
}

pub fn resolve_effective_env_mode(
    active_worktree_path: Option<&str>,
    has_server_thread: bool,
    draft_thread_env_mode: Option<DraftThreadEnvMode>,
) -> DraftThreadEnvMode {
    if !has_server_thread {
        if active_worktree_path.is_some() {
            return DraftThreadEnvMode::Local;
        }
        return if draft_thread_env_mode == Some(DraftThreadEnvMode::Worktree) {
            DraftThreadEnvMode::Worktree
        } else {
            DraftThreadEnvMode::Local
        };
    }

    if active_worktree_path.is_some() {
        DraftThreadEnvMode::Worktree
    } else {
        DraftThreadEnvMode::Local
    }
}

pub fn resolve_draft_env_mode_after_branch_change(
    next_worktree_path: Option<&str>,
    current_worktree_path: Option<&str>,
    effective_env_mode: DraftThreadEnvMode,
) -> DraftThreadEnvMode {
    if next_worktree_path.is_some() {
        return DraftThreadEnvMode::Worktree;
    }
    if effective_env_mode == DraftThreadEnvMode::Worktree && current_worktree_path.is_none() {
        return DraftThreadEnvMode::Worktree;
    }
    DraftThreadEnvMode::Local
}

pub fn resolve_branch_toolbar_value(
    env_mode: DraftThreadEnvMode,
    active_worktree_path: Option<&str>,
    active_thread_branch: Option<&str>,
    current_git_branch: Option<&str>,
) -> Option<String> {
    if env_mode == DraftThreadEnvMode::Worktree && active_worktree_path.is_none() {
        return active_thread_branch
            .or(current_git_branch)
            .map(str::to_string);
    }
    current_git_branch
        .or(active_thread_branch)
        .map(str::to_string)
}

pub fn branch_toolbar_trigger_label(
    active_worktree_path: Option<&str>,
    effective_env_mode: DraftThreadEnvMode,
    resolved_active_branch: Option<&str>,
) -> String {
    let Some(resolved_active_branch) = resolved_active_branch else {
        return "Select ref".to_string();
    };
    if effective_env_mode == DraftThreadEnvMode::Worktree && active_worktree_path.is_none() {
        return format!("From {resolved_active_branch}");
    }
    resolved_active_branch.to_string()
}

pub fn resolve_branch_selection_target(
    active_project_cwd: &str,
    active_worktree_path: Option<&str>,
    ref_name: &VcsRef,
) -> BranchSelectionTarget {
    if let Some(worktree_path) = ref_name.worktree_path.as_deref() {
        return BranchSelectionTarget {
            checkout_cwd: worktree_path.to_string(),
            next_worktree_path: if worktree_path == active_project_cwd {
                None
            } else {
                Some(worktree_path.to_string())
            },
            reuse_existing_worktree: true,
        };
    }

    let next_worktree_path = if active_worktree_path.is_some() && ref_name.is_default {
        None
    } else {
        active_worktree_path.map(str::to_string)
    };

    BranchSelectionTarget {
        checkout_cwd: next_worktree_path
            .clone()
            .unwrap_or_else(|| active_project_cwd.to_string()),
        next_worktree_path,
        reuse_existing_worktree: false,
    }
}

pub fn derive_local_branch_name_from_remote_ref(branch_name: &str) -> String {
    let Some(first_separator_index) = branch_name.find('/') else {
        return branch_name.to_string();
    };
    if first_separator_index == 0 || first_separator_index == branch_name.len() - 1 {
        return branch_name.to_string();
    }
    branch_name[first_separator_index + 1..].to_string()
}

fn derive_local_branch_name_candidates_from_remote_ref(
    branch_name: &str,
    remote_name: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::new();
    let first_slash_candidate = derive_local_branch_name_from_remote_ref(branch_name);
    if !first_slash_candidate.is_empty() {
        candidates.push(first_slash_candidate);
    }

    if let Some(remote_name) = remote_name {
        let remote_prefix = format!("{remote_name}/");
        if branch_name.starts_with(&remote_prefix) && branch_name.len() > remote_prefix.len() {
            let candidate = branch_name[remote_prefix.len()..].to_string();
            if !candidates.iter().any(|existing| existing == &candidate) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

pub fn dedupe_remote_branches_with_local_matches(refs: &[VcsRef]) -> Vec<VcsRef> {
    let local_branch_names = refs
        .iter()
        .filter(|ref_name| !ref_name.is_remote)
        .map(|ref_name| ref_name.name.as_str())
        .collect::<Vec<_>>();

    refs.iter()
        .filter(|ref_name| {
            if !ref_name.is_remote {
                return true;
            }
            if ref_name.remote_name.as_deref() != Some("origin") {
                return true;
            }
            let local_branch_candidates = derive_local_branch_name_candidates_from_remote_ref(
                &ref_name.name,
                ref_name.remote_name.as_deref(),
            );
            !local_branch_candidates.iter().any(|candidate| {
                local_branch_names
                    .iter()
                    .any(|local_name| *local_name == candidate.as_str())
            })
        })
        .cloned()
        .collect()
}

pub fn should_include_branch_picker_item(
    item_value: &str,
    normalized_query: &str,
    create_branch_item_value: Option<&str>,
    checkout_pull_request_item_value: Option<&str>,
) -> bool {
    if normalized_query.is_empty() {
        return true;
    }
    if create_branch_item_value == Some(item_value) {
        return true;
    }
    if checkout_pull_request_item_value == Some(item_value) {
        return true;
    }
    item_value.to_ascii_lowercase().contains(normalized_query)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    ApprovalRequired,
    AutoAcceptEdits,
    FullAccess,
}

impl Default for RuntimeMode {
    fn default() -> Self {
        Self::FullAccess
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderInteractionMode {
    Default,
    Plan,
}

impl Default for ProviderInteractionMode {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftSessionState {
    pub draft_id: String,
    pub thread_ref: ScopedThreadRef,
    pub project_ref: ScopedProjectRef,
    pub logical_project_key: String,
    pub created_at: String,
    pub runtime_mode: RuntimeMode,
    pub interaction_mode: ProviderInteractionMode,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
    pub env_mode: DraftThreadEnvMode,
    pub promoted_to: Option<ScopedThreadRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: String,
    pub environment_id: String,
    pub name: String,
    pub path: String,
    pub scripts: Vec<ProjectScript>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectScriptIcon {
    Play,
    Test,
    Lint,
    Configure,
    Build,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectScript {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: ProjectScriptIcon,
    pub run_on_worktree_create: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorId {
    Cursor,
    Trae,
    Kiro,
    VsCode,
    VsCodeInsiders,
    VsCodium,
    Zed,
    Antigravity,
    Idea,
    Aqua,
    CLion,
    DataGrip,
    DataSpell,
    GoLand,
    PhpStorm,
    PyCharm,
    Rider,
    RubyMine,
    RustRover,
    WebStorm,
    FileManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorOption {
    pub label: &'static str,
    pub id: EditorId,
}

pub const DEFAULT_PROVIDER_DRIVER_KIND: &str = "codex";
pub const DEFAULT_MODEL: &str = "gpt-5.4";
pub const DEFAULT_GIT_TEXT_GENERATION_MODEL: &str = "gpt-5.4-mini";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderState {
    Ready,
    Warning,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderAuthStatus {
    Authenticated,
    Unauthenticated,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderAuth {
    pub status: ServerProviderAuthStatus,
    pub kind: Option<String>,
    pub label: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderModel {
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub sub_provider: Option<String>,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerProviderVersionAdvisoryStatus {
    Unknown,
    Current,
    BehindLatest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProviderVersionAdvisory {
    pub status: ServerProviderVersionAdvisoryStatus,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_command: Option<String>,
    pub can_update: bool,
    pub checked_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderVersionAdvisoryEmphasis {
    Normal,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatusSummary {
    pub headline: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderVersionAdvisoryPresentation {
    pub detail: String,
    pub update_command: Option<String>,
    pub emphasis: ProviderVersionAdvisoryEmphasis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerProvider {
    pub instance_id: String,
    pub driver: String,
    pub display_name: Option<String>,
    pub accent_color: Option<String>,
    pub badge_label: Option<String>,
    pub continuation_group_key: Option<String>,
    pub show_interaction_mode_toggle: bool,
    pub enabled: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub status: ServerProviderState,
    pub auth: ServerProviderAuth,
    pub checked_at: String,
    pub message: Option<String>,
    pub availability: ServerProviderAvailability,
    pub unavailable_reason: Option<String>,
    pub models: Vec<ServerProviderModel>,
    pub version_advisory: Option<ServerProviderVersionAdvisory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInstanceEntry {
    pub instance_id: String,
    pub driver_kind: String,
    pub display_name: String,
    pub accent_color: Option<String>,
    pub continuation_group_key: Option<String>,
    pub enabled: bool,
    pub installed: bool,
    pub status: ServerProviderState,
    pub is_default: bool,
    pub is_available: bool,
    pub models: Vec<ServerProviderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelFavorite {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelPickerSelectedInstance {
    Favorites,
    Instance(String),
}

impl ModelPickerSelectedInstance {
    pub fn instance_id(&self) -> Option<&str> {
        match self {
            Self::Favorites => None,
            Self::Instance(instance_id) => Some(instance_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerItem {
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub sub_provider: Option<String>,
    pub instance_id: String,
    pub driver_kind: String,
    pub instance_display_name: String,
    pub instance_accent_color: Option<String>,
    pub continuation_group_key: Option<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerState {
    pub active_entry: Option<ProviderInstanceEntry>,
    pub trigger_title: String,
    pub trigger_subtitle: Option<String>,
    pub trigger_label: String,
    pub show_instance_badge: bool,
    pub selected_instance: ModelPickerSelectedInstance,
    pub is_locked: bool,
    pub show_locked_instance_sidebar: bool,
    pub show_sidebar: bool,
    pub sidebar_entries: Vec<ProviderInstanceEntry>,
    pub locked_header_label: Option<String>,
    pub filtered_models: Vec<ModelPickerItem>,
}

pub fn default_instance_id_for_driver(driver: &str) -> String {
    driver.to_string()
}

pub fn default_model_by_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "codex" => Some(DEFAULT_MODEL),
        "claudeAgent" => Some("claude-sonnet-4-6"),
        "cursor" => Some("auto"),
        "opencode" => Some("openai/gpt-5"),
        _ => None,
    }
}

pub fn default_git_text_generation_model_by_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "codex" => Some(DEFAULT_GIT_TEXT_GENERATION_MODEL),
        "claudeAgent" => Some("claude-haiku-4-5"),
        "cursor" => Some("composer-2"),
        "opencode" => Some("openai/gpt-5"),
        _ => None,
    }
}

pub fn provider_display_name(driver: &str) -> String {
    match driver {
        "codex" => "Codex".to_string(),
        "claudeAgent" => "Claude".to_string(),
        "cursor" => "Cursor".to_string(),
        "opencode" => "OpenCode".to_string(),
        _ => format_provider_driver_kind_label(driver),
    }
}

pub fn get_provider_summary(provider: Option<&ServerProvider>) -> ProviderStatusSummary {
    let Some(provider) = provider else {
        return ProviderStatusSummary {
            headline: "Checking provider status".to_string(),
            detail: Some(
                "Waiting for the server to report installation and authentication details."
                    .to_string(),
            ),
        };
    };

    if !provider.enabled {
        return ProviderStatusSummary {
            headline: "Disabled".to_string(),
            detail: Some(provider.message.clone().unwrap_or_else(|| {
                format!(
                    "This provider is installed but disabled for new sessions in {}.",
                    APP_NAME
                )
            })),
        };
    }

    if !provider.installed {
        return ProviderStatusSummary {
            headline: "Not found".to_string(),
            detail: Some(
                provider
                    .message
                    .clone()
                    .unwrap_or_else(|| "CLI not detected on PATH.".to_string()),
            ),
        };
    }

    if provider.auth.status == ServerProviderAuthStatus::Authenticated {
        let auth_label = provider
            .auth
            .label
            .as_deref()
            .or(provider.auth.kind.as_deref());
        return ProviderStatusSummary {
            headline: auth_label
                .map(|label| format!("Authenticated · {label}"))
                .unwrap_or_else(|| "Authenticated".to_string()),
            detail: provider.message.clone(),
        };
    }

    if provider.auth.status == ServerProviderAuthStatus::Unauthenticated {
        return ProviderStatusSummary {
            headline: "Not authenticated".to_string(),
            detail: provider.message.clone(),
        };
    }

    if provider.status == ServerProviderState::Warning {
        return ProviderStatusSummary {
            headline: "Needs attention".to_string(),
            detail: Some(provider.message.clone().unwrap_or_else(|| {
                "The provider is installed, but the server could not fully verify it.".to_string()
            })),
        };
    }

    if provider.status == ServerProviderState::Error {
        return ProviderStatusSummary {
            headline: "Unavailable".to_string(),
            detail: Some(
                provider
                    .message
                    .clone()
                    .unwrap_or_else(|| "The provider failed its startup checks.".to_string()),
            ),
        };
    }

    ProviderStatusSummary {
        headline: "Available".to_string(),
        detail: Some(provider.message.clone().unwrap_or_else(|| {
            "Installed and ready, but authentication could not be verified.".to_string()
        })),
    }
}

pub fn get_provider_version_label(version: Option<&str>) -> Option<String> {
    let version = version?;
    if version.is_empty() {
        None
    } else if version.starts_with('v') {
        Some(version.to_string())
    } else {
        Some(format!("v{version}"))
    }
}

pub fn get_provider_version_advisory_presentation(
    advisory: Option<&ServerProviderVersionAdvisory>,
) -> Option<ProviderVersionAdvisoryPresentation> {
    let advisory = advisory?;
    if matches!(
        advisory.status,
        ServerProviderVersionAdvisoryStatus::Current | ServerProviderVersionAdvisoryStatus::Unknown
    ) {
        return None;
    }

    let version_label = get_provider_version_label(advisory.latest_version.as_deref());
    Some(ProviderVersionAdvisoryPresentation {
        detail: advisory.message.clone().unwrap_or_else(|| {
            version_label
                .map(|label| format!("Update available: install {label}."))
                .unwrap_or_else(|| {
                    "Update available: install the latest provider version.".to_string()
                })
        }),
        update_command: advisory.update_command.clone(),
        emphasis: ProviderVersionAdvisoryEmphasis::Normal,
    })
}

pub fn format_provider_driver_kind_label(provider: &str) -> String {
    title_case_words(&split_label_words(provider))
}

pub fn provider_instance_initials(label: &str) -> String {
    let words = split_label_words(label);
    if words.is_empty() {
        return String::new();
    }
    if words.len() == 1 {
        return words[0]
            .chars()
            .take(2)
            .flat_map(char::to_uppercase)
            .collect();
    }
    words
        .iter()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn split_label_words(value: &str) -> Vec<String> {
    let mut normalized = String::new();
    let mut previous_lowercase = false;
    for ch in value.trim().chars() {
        if ch == '_' || ch == '-' {
            normalized.push(' ');
            previous_lowercase = false;
            continue;
        }
        if ch.is_ascii_uppercase() && previous_lowercase {
            normalized.push(' ');
        }
        previous_lowercase = ch.is_ascii_lowercase();
        normalized.push(ch);
    }
    normalized
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

fn title_case_words(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect::<String>(),
                None => String::new(),
            }
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_provider_accent_color(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.len() != 7 || !trimmed.starts_with('#') {
        return None;
    }
    if trimmed[1..].chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn resolve_instance_display_name(
    snapshot: &ServerProvider,
    instance_id: &str,
    driver_kind: &str,
    is_default: bool,
) -> String {
    let trimmed_snapshot_name = snapshot.display_name.as_deref().map(str::trim);
    let kind_label = provider_display_name(driver_kind);
    if let Some(name) = trimmed_snapshot_name.filter(|name| !name.is_empty()) {
        if name != kind_label {
            return name.to_string();
        }
    }
    if !is_default {
        let humanized = title_case_words(&split_label_words(instance_id));
        if !humanized.is_empty() {
            return humanized;
        }
    }
    trimmed_snapshot_name
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or(kind_label)
}

pub fn derive_provider_instance_entries(
    providers: &[ServerProvider],
) -> Vec<ProviderInstanceEntry> {
    providers
        .iter()
        .map(|snapshot| {
            let instance_id = snapshot.instance_id.clone();
            let driver_kind = snapshot.driver.clone();
            let default_id = default_instance_id_for_driver(&driver_kind);
            let is_default = instance_id == default_id;
            ProviderInstanceEntry {
                display_name: resolve_instance_display_name(
                    snapshot,
                    &instance_id,
                    &driver_kind,
                    is_default,
                ),
                accent_color: normalize_provider_accent_color(snapshot.accent_color.as_deref()),
                continuation_group_key: snapshot.continuation_group_key.clone(),
                enabled: snapshot.enabled,
                installed: snapshot.installed,
                status: snapshot.status,
                is_default,
                is_available: snapshot.availability == ServerProviderAvailability::Available,
                models: snapshot.models.clone(),
                instance_id,
                driver_kind,
            }
        })
        .collect()
}

pub fn sort_provider_instance_entries(
    entries: &[ProviderInstanceEntry],
) -> Vec<ProviderInstanceEntry> {
    let mut by_kind = BTreeMap::<String, Vec<ProviderInstanceEntry>>::new();
    let mut kind_order = Vec::<String>::new();
    for entry in entries {
        if !by_kind.contains_key(&entry.driver_kind) {
            kind_order.push(entry.driver_kind.clone());
        }
        by_kind
            .entry(entry.driver_kind.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut sorted = Vec::new();
    for kind in kind_order {
        let Some(bucket) = by_kind.remove(&kind) else {
            continue;
        };
        sorted.extend(bucket.iter().filter(|entry| entry.is_default).cloned());
        sorted.extend(bucket.iter().filter(|entry| !entry.is_default).cloned());
    }
    sorted
}

pub fn get_provider_instance_entry(
    providers: &[ServerProvider],
    instance_id: &str,
) -> Option<ProviderInstanceEntry> {
    derive_provider_instance_entries(providers)
        .into_iter()
        .find(|entry| entry.instance_id == instance_id)
}

pub fn resolve_selectable_provider_instance(
    providers: &[ServerProvider],
    instance_id: Option<&str>,
) -> Option<String> {
    let entries = derive_provider_instance_entries(providers);
    if let Some(instance_id) = instance_id {
        if entries
            .iter()
            .any(|entry| entry.instance_id == instance_id && entry.enabled && entry.is_available)
        {
            return Some(instance_id.to_string());
        }
    }
    entries
        .iter()
        .find(|entry| entry.enabled && entry.is_available)
        .map(|entry| entry.instance_id.clone())
}

pub fn resolve_provider_driver_kind_for_instance_selection(
    providers: &[ServerProvider],
    selection: Option<&str>,
) -> Option<String> {
    derive_provider_instance_entries(providers)
        .into_iter()
        .find(|entry| Some(entry.instance_id.as_str()) == selection)
        .map(|entry| entry.driver_kind)
}

pub fn get_display_model_name(model: &ServerProviderModel, prefer_short_name: bool) -> String {
    if prefer_short_name {
        if let Some(short_name) = model
            .short_name
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            return short_name.to_string();
        }
    }
    model.name.clone()
}

pub fn get_trigger_display_model_name(model: &ServerProviderModel) -> String {
    get_display_model_name(model, true)
}

pub fn get_trigger_display_model_label(model: &ServerProviderModel) -> String {
    let title = get_trigger_display_model_name(model);
    model
        .sub_provider
        .as_deref()
        .filter(|sub_provider| !sub_provider.is_empty())
        .map(|sub_provider| format!("{sub_provider} · {title}"))
        .unwrap_or(title)
}

pub fn provider_model_key(instance_id: &str, slug: &str) -> String {
    format!("{instance_id}:{slug}")
}

pub fn split_instance_model_key(key: &str) -> (String, String) {
    key.split_once(':')
        .map(|(instance_id, slug)| (instance_id.to_string(), slug.to_string()))
        .unwrap_or_else(|| (key.to_string(), String::new()))
}

fn favorite_model_key_set(favorites: &[ProviderModelFavorite]) -> Vec<String> {
    favorites
        .iter()
        .map(|favorite| provider_model_key(&favorite.provider, &favorite.model))
        .collect()
}

fn is_favorite_model_key(favorites: &[String], instance_id: &str, slug: &str) -> bool {
    let key = provider_model_key(instance_id, slug);
    favorites.iter().any(|favorite| favorite == &key)
}

pub fn normalize_search_query(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

pub fn score_subsequence_match(value: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }

    let value_chars = value.chars().collect::<Vec<_>>();
    let query_chars = query.chars().collect::<Vec<_>>();
    let mut query_index = 0usize;
    let mut first_match_index = None::<usize>;
    let mut previous_match_index = None::<usize>;
    let mut gap_penalty = 0usize;

    for (value_index, value_char) in value_chars.iter().enumerate() {
        if query_index >= query_chars.len() || value_char != &query_chars[query_index] {
            continue;
        }

        if first_match_index.is_none() {
            first_match_index = Some(value_index);
        }
        if let Some(previous) = previous_match_index {
            gap_penalty += value_index.saturating_sub(previous + 1);
        }

        previous_match_index = Some(value_index);
        query_index += 1;
        if query_index == query_chars.len() {
            let first = first_match_index.unwrap_or(0);
            let span_penalty = value_index + 1 - first - query_chars.len();
            let length_penalty = value_chars.len().saturating_sub(query_chars.len()).min(64);
            return Some(first * 2 + gap_penalty * 3 + span_penalty + length_penalty);
        }
    }

    None
}

fn length_penalty(value: &str, query: &str) -> usize {
    value
        .chars()
        .count()
        .saturating_sub(query.chars().count())
        .min(64)
}

fn find_boundary_match_index_with_markers(
    value: &str,
    query: &str,
    boundary_markers: &[&str],
) -> Option<usize> {
    boundary_markers
        .iter()
        .filter_map(|marker| {
            value
                .find(&format!("{marker}{query}"))
                .map(|index| index + marker.len())
        })
        .min()
}

pub fn score_query_match_with_boundary_markers(
    value: &str,
    query: &str,
    exact_base: usize,
    prefix_base: Option<usize>,
    boundary_base: Option<usize>,
    includes_base: Option<usize>,
    fuzzy_base: Option<usize>,
    boundary_markers: &[&str],
) -> Option<usize> {
    if value.is_empty() || query.is_empty() {
        return None;
    }
    if value == query {
        return Some(exact_base);
    }
    if let Some(prefix_base) = prefix_base {
        if value.starts_with(query) {
            return Some(prefix_base + length_penalty(value, query));
        }
    }
    if let Some(boundary_base) = boundary_base {
        if let Some(boundary_index) =
            find_boundary_match_index_with_markers(value, query, boundary_markers)
        {
            return Some(boundary_base + boundary_index * 2 + length_penalty(value, query));
        }
    }
    if let Some(includes_base) = includes_base {
        if let Some(includes_index) = value.find(query) {
            return Some(includes_base + includes_index * 2 + length_penalty(value, query));
        }
    }
    if let Some(fuzzy_base) = fuzzy_base {
        if let Some(fuzzy_score) = score_subsequence_match(value, query) {
            return Some(fuzzy_base + fuzzy_score);
        }
    }
    None
}

pub fn score_query_match(
    value: &str,
    query: &str,
    exact_base: usize,
    prefix_base: Option<usize>,
    boundary_base: Option<usize>,
    includes_base: Option<usize>,
    fuzzy_base: Option<usize>,
) -> Option<usize> {
    score_query_match_with_boundary_markers(
        value,
        query,
        exact_base,
        prefix_base,
        boundary_base,
        includes_base,
        fuzzy_base,
        &[" ", "-", "_", "/"],
    )
}

pub fn build_model_picker_search_text(model: &ModelPickerItem) -> String {
    normalize_search_query(
        &[
            model.name.as_str(),
            model.short_name.as_deref().unwrap_or(""),
            model.sub_provider.as_deref().unwrap_or(""),
            model.driver_kind.as_str(),
            model.instance_display_name.as_str(),
        ]
        .iter()
        .filter(|value| !value.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" "),
    )
}

pub fn score_model_picker_search(model: &ModelPickerItem, query: &str) -> Option<isize> {
    const FAVORITE_SCORE_BOOST: isize = 24;
    let tokens = normalize_search_query(query)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return Some(0);
    }

    let fields = [
        normalize_search_query(&model.name),
        model
            .short_name
            .as_deref()
            .map(normalize_search_query)
            .unwrap_or_default(),
        model
            .sub_provider
            .as_deref()
            .map(normalize_search_query)
            .unwrap_or_default(),
        normalize_search_query(&model.driver_kind),
        normalize_search_query(&model.instance_display_name),
        build_model_picker_search_text(model),
    ];

    let mut score = 0isize;
    for token in tokens {
        let token_score = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| !field.is_empty())
            .filter_map(|(index, field)| {
                let field_base = index * 10;
                score_query_match(
                    field,
                    &token,
                    field_base,
                    Some(field_base + 2),
                    Some(field_base + 4),
                    Some(field_base + 6),
                    (token.len() >= 3).then_some(field_base + 100),
                )
            })
            .min()?;
        score += token_score as isize;
    }

    Some(if model.is_favorite {
        score - FAVORITE_SCORE_BOOST
    } else {
        score
    })
}

pub fn sort_provider_model_items(
    items: &[ModelPickerItem],
    favorite_model_keys: &[String],
    group_favorites: bool,
    instance_order: &[String],
) -> Vec<ModelPickerItem> {
    let instance_rank = instance_order
        .iter()
        .enumerate()
        .map(|(index, instance_id)| (instance_id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let original_rank = items
        .iter()
        .enumerate()
        .map(|(index, item)| (provider_model_key(&item.instance_id, &item.slug), index))
        .collect::<BTreeMap<_, _>>();
    let mut indexed = items.to_vec();
    indexed.sort_by(|left, right| {
        if group_favorites {
            let left_fav =
                is_favorite_model_key(favorite_model_keys, &left.instance_id, &left.slug);
            let right_fav =
                is_favorite_model_key(favorite_model_keys, &right.instance_id, &right.slug);
            if left_fav != right_fav {
                return right_fav.cmp(&left_fav);
            }
        }

        let left_instance_rank = instance_rank
            .get(&left.instance_id)
            .copied()
            .unwrap_or(usize::MAX);
        let right_instance_rank = instance_rank
            .get(&right.instance_id)
            .copied()
            .unwrap_or(usize::MAX);
        left_instance_rank.cmp(&right_instance_rank).then_with(|| {
            let left_key = provider_model_key(&left.instance_id, &left.slug);
            let right_key = provider_model_key(&right.instance_id, &right.slug);
            original_rank
                .get(&left_key)
                .copied()
                .unwrap_or(usize::MAX)
                .cmp(&original_rank.get(&right_key).copied().unwrap_or(usize::MAX))
        })
    });
    indexed
}

pub fn normalize_model_slug(model: Option<&str>, provider: &str) -> Option<String> {
    let trimmed = model?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let aliased = match provider {
        "codex" => match trimmed {
            "gpt-5-codex" | "5.4" => Some("gpt-5.4"),
            "5.3" | "gpt-5.3" => Some("gpt-5.3-codex"),
            "5.3-spark" | "gpt-5.3-spark" => Some("gpt-5.3-codex-spark"),
            _ => None,
        },
        "claudeAgent" => match trimmed {
            "opus" | "opus-4.7" | "claude-opus-4.7" => Some("claude-opus-4-7"),
            "opus-4.6" | "claude-opus-4.6" | "claude-opus-4-6-20251117" => Some("claude-opus-4-6"),
            "sonnet" | "sonnet-4.6" | "claude-sonnet-4.6" | "claude-sonnet-4-6-20251117" => {
                Some("claude-sonnet-4-6")
            }
            "haiku" | "haiku-4.5" | "claude-haiku-4.5" | "claude-haiku-4-5-20251001" => {
                Some("claude-haiku-4-5")
            }
            _ => None,
        },
        "cursor" => match trimmed {
            "composer" => Some("composer-2"),
            "composer-1" => Some("composer-1.5"),
            "composer-1.5" => Some("composer-1.5"),
            "opus-4.6-thinking" | "opus-4.6" => Some("claude-opus-4-6"),
            "sonnet-4.6-thinking" | "sonnet-4.6" => Some("claude-sonnet-4-6"),
            "opus-4.5-thinking" | "opus-4.5" => Some("claude-opus-4-5"),
            _ => None,
        },
        _ => None,
    };
    Some(aliased.unwrap_or(trimmed).to_string())
}

pub fn resolve_selectable_model(
    provider: &str,
    value: Option<&str>,
    options: &[ServerProviderModel],
) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(direct) = options.iter().find(|option| option.slug == trimmed) {
        return Some(direct.slug.clone());
    }
    if let Some(by_name) = options
        .iter()
        .find(|option| option.name.eq_ignore_ascii_case(trimmed))
    {
        return Some(by_name.slug.clone());
    }
    let normalized = normalize_model_slug(Some(trimmed), provider)?;
    options
        .iter()
        .find(|option| option.slug == normalized)
        .map(|option| option.slug.clone())
}

fn matches_locked_provider(
    entry: &ProviderInstanceEntry,
    locked_provider: Option<&str>,
    locked_continuation_group_key: Option<&str>,
) -> bool {
    let Some(locked_provider) = locked_provider else {
        return true;
    };
    if entry.driver_kind != locked_provider {
        return false;
    }
    locked_continuation_group_key
        .filter(|key| !key.is_empty())
        .map(|key| entry.continuation_group_key.as_deref() == Some(key))
        .unwrap_or(true)
}

pub fn resolve_model_picker_state(
    snapshot: &AppSnapshot,
    search_query: &str,
    selected_instance: Option<ModelPickerSelectedInstance>,
    locked_provider: Option<&str>,
    locked_continuation_group_key: Option<&str>,
) -> ModelPickerState {
    let entries = derive_provider_instance_entries(&snapshot.providers);
    let active_entry = entries
        .iter()
        .find(|entry| entry.instance_id == snapshot.selected_provider_instance_id)
        .cloned();
    let selected_options = active_entry
        .as_ref()
        .map(|entry| entry.models.as_slice())
        .unwrap_or(&[]);
    let selected_model = selected_options
        .iter()
        .find(|option| option.slug == snapshot.selected_model)
        .or_else(|| selected_options.first());
    let trigger_title = selected_model
        .map(get_trigger_display_model_name)
        .unwrap_or_else(|| snapshot.selected_model.clone());
    let trigger_subtitle = selected_model.and_then(|model| model.sub_provider.clone());
    let trigger_label = selected_model
        .map(get_trigger_display_model_label)
        .unwrap_or_else(|| snapshot.selected_model.clone());
    let duplicate_driver_count = active_entry
        .as_ref()
        .map(|active| {
            entries
                .iter()
                .filter(|entry| entry.driver_kind == active.driver_kind)
                .count()
        })
        .unwrap_or(0);
    let show_instance_badge = active_entry
        .as_ref()
        .map(|entry| entry.accent_color.is_some() || duplicate_driver_count > 1)
        .unwrap_or(false);
    let favorite_keys = favorite_model_key_set(&snapshot.model_favorites);
    let selected_instance = selected_instance.unwrap_or_else(|| {
        if locked_provider.is_some() {
            ModelPickerSelectedInstance::Instance(snapshot.selected_provider_instance_id.clone())
        } else if !snapshot.model_favorites.is_empty() {
            ModelPickerSelectedInstance::Favorites
        } else {
            ModelPickerSelectedInstance::Instance(snapshot.selected_provider_instance_id.clone())
        }
    });
    let is_locked = locked_provider.is_some();
    let locked_instance_entries = locked_provider
        .map(|_| {
            entries
                .iter()
                .filter(|entry| {
                    matches_locked_provider(entry, locked_provider, locked_continuation_group_key)
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let show_locked_instance_sidebar = is_locked && locked_instance_entries.len() > 1;
    let is_searching = !search_query.trim().is_empty();
    let show_sidebar = !is_searching && (!is_locked || show_locked_instance_sidebar);
    let sidebar_entries = if show_locked_instance_sidebar {
        locked_instance_entries.clone()
    } else {
        entries.clone()
    };
    let mut flat_models = Vec::<ModelPickerItem>::new();
    for entry in &entries {
        if entry.status != ServerProviderState::Ready {
            continue;
        }
        for model in &entry.models {
            flat_models.push(ModelPickerItem {
                slug: model.slug.clone(),
                name: model.name.clone(),
                short_name: model.short_name.clone(),
                sub_provider: model.sub_provider.clone(),
                instance_id: entry.instance_id.clone(),
                driver_kind: entry.driver_kind.clone(),
                instance_display_name: entry.display_name.clone(),
                instance_accent_color: entry.accent_color.clone(),
                continuation_group_key: entry.continuation_group_key.clone(),
                is_favorite: is_favorite_model_key(&favorite_keys, &entry.instance_id, &model.slug),
            });
        }
    }

    let filtered_models = if is_searching {
        let mut ranked = flat_models
            .into_iter()
            .filter(|model| {
                locked_provider
                    .map(|_| {
                        matches_locked_provider(
                            &ProviderInstanceEntry {
                                instance_id: model.instance_id.clone(),
                                driver_kind: model.driver_kind.clone(),
                                display_name: model.instance_display_name.clone(),
                                accent_color: model.instance_accent_color.clone(),
                                continuation_group_key: model.continuation_group_key.clone(),
                                enabled: true,
                                installed: true,
                                status: ServerProviderState::Ready,
                                is_default: false,
                                is_available: true,
                                models: Vec::new(),
                            },
                            locked_provider,
                            locked_continuation_group_key,
                        )
                    })
                    .unwrap_or(true)
            })
            .filter_map(|model| {
                score_model_picker_search(&model, search_query).map(|score| {
                    let tie_breaker = build_model_picker_search_text(&model);
                    (model, score, tie_breaker)
                })
            })
            .collect::<Vec<_>>();
        ranked.sort_by(
            |(left, left_score, left_tie), (right, right_score, right_tie)| {
                left_score
                    .cmp(right_score)
                    .then_with(|| right.is_favorite.cmp(&left.is_favorite))
                    .then_with(|| left_tie.cmp(right_tie))
            },
        );
        ranked.into_iter().map(|(model, _, _)| model).collect()
    } else {
        let mut result = flat_models
            .into_iter()
            .filter(|model| {
                locked_provider
                    .map(|_| {
                        model.driver_kind == locked_provider.unwrap_or_default()
                            && locked_continuation_group_key
                                .filter(|key| !key.is_empty())
                                .map(|key| model.continuation_group_key.as_deref() == Some(key))
                                .unwrap_or(true)
                    })
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();

        if is_locked {
            if show_locked_instance_sidebar {
                if let Some(instance_id) = selected_instance.instance_id() {
                    result.retain(|model| model.instance_id == instance_id);
                }
            }
        } else if selected_instance == ModelPickerSelectedInstance::Favorites {
            result.retain(|model| model.is_favorite);
        } else if let Some(instance_id) = selected_instance.instance_id() {
            result.retain(|model| model.instance_id == instance_id);
        }

        let instance_order = if selected_instance == ModelPickerSelectedInstance::Favorites {
            entries
                .iter()
                .map(|entry| entry.instance_id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        sort_provider_model_items(
            &result,
            &favorite_keys,
            selected_instance != ModelPickerSelectedInstance::Favorites,
            &instance_order,
        )
    };

    let locked_header_label = if is_locked && !show_locked_instance_sidebar {
        let matches = entries
            .iter()
            .filter(|entry| {
                matches_locked_provider(entry, locked_provider, locked_continuation_group_key)
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            None
        } else {
            matches
                .iter()
                .find(|entry| entry.instance_id == snapshot.selected_provider_instance_id)
                .copied()
                .or_else(|| matches.first().copied())
                .map(|entry| entry.display_name.clone())
        }
    } else {
        None
    };

    ModelPickerState {
        active_entry,
        trigger_title,
        trigger_subtitle,
        trigger_label,
        show_instance_badge,
        selected_instance,
        is_locked,
        show_locked_instance_sidebar,
        show_sidebar,
        sidebar_entries,
        locked_header_label,
        filtered_models,
    }
}

const MAX_SCRIPT_ID_LENGTH: usize = 64;

pub fn command_for_project_script(script_id: &str) -> String {
    format!("script.{script_id}.run")
}

pub fn project_script_id_from_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    let prefix = "script.";
    let suffix = ".run";
    if !trimmed.starts_with(prefix) || !trimmed.ends_with(suffix) {
        return None;
    }
    let script_id = &trimmed[prefix.len()..trimmed.len() - suffix.len()];
    if script_id.is_empty() {
        None
    } else {
        Some(script_id.to_string())
    }
}

fn normalize_script_id(value: &str) -> String {
    let mut cleaned = String::new();
    let mut last_was_dash = false;
    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            cleaned.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !cleaned.is_empty() {
            cleaned.push('-');
            last_was_dash = true;
        }
    }
    while cleaned.ends_with('-') {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        return "script".to_string();
    }
    if cleaned.len() <= MAX_SCRIPT_ID_LENGTH {
        return cleaned;
    }
    let mut truncated = cleaned[..MAX_SCRIPT_ID_LENGTH]
        .trim_end_matches('-')
        .to_string();
    if truncated.is_empty() {
        truncated = "script".to_string();
    }
    truncated
}

pub fn next_project_script_id<I, S>(name: &str, existing_ids: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let taken = existing_ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .collect::<Vec<_>>();
    let base_id = normalize_script_id(name);
    if !taken.iter().any(|id| id == &base_id) {
        return base_id;
    }

    for suffix in 2..10_000 {
        let candidate = format!("{base_id}-{suffix}");
        let safe_candidate = if candidate.len() <= MAX_SCRIPT_ID_LENGTH {
            candidate
        } else {
            let suffix_len = suffix.to_string().len();
            let prefix_len = MAX_SCRIPT_ID_LENGTH.saturating_sub(suffix_len + 1).max(1);
            format!("{}-{suffix}", &base_id[..prefix_len])
        };
        if !taken.iter().any(|id| id == &safe_candidate) {
            return safe_candidate;
        }
    }

    base_id
}

pub fn primary_project_script(scripts: &[ProjectScript]) -> Option<&ProjectScript> {
    scripts
        .iter()
        .find(|script| !script.run_on_worktree_create)
        .or_else(|| scripts.first())
}

pub fn setup_project_script(scripts: &[ProjectScript]) -> Option<&ProjectScript> {
    scripts.iter().find(|script| script.run_on_worktree_create)
}

pub fn project_script_cwd(project_cwd: &str, worktree_path: Option<&str>) -> String {
    worktree_path.unwrap_or(project_cwd).to_string()
}

pub fn project_script_runtime_env(
    project_cwd: &str,
    worktree_path: Option<&str>,
    extra_env: &[(&str, &str)],
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([("T3CODE_PROJECT_ROOT".to_string(), project_cwd.to_string())]);
    if let Some(worktree_path) = worktree_path {
        env.insert(
            "T3CODE_WORKTREE_PATH".to_string(),
            worktree_path.to_string(),
        );
    }
    for (key, value) in extra_env {
        env.insert((*key).to_string(), (*value).to_string());
    }
    env
}

pub fn should_show_open_in_picker(
    active_project_name: Option<&str>,
    active_thread_environment_id: &str,
    primary_environment_id: Option<&str>,
) -> bool {
    active_project_name.is_some()
        && primary_environment_id
            .map(|primary| primary == active_thread_environment_id)
            .unwrap_or(false)
}

pub fn resolve_editor_options(platform: &str, available_editors: &[EditorId]) -> Vec<EditorOption> {
    editor_options(platform)
        .iter()
        .copied()
        .filter(|option| available_editors.iter().any(|editor| editor == &option.id))
        .collect()
}

fn editor_options(platform: &str) -> Vec<EditorOption> {
    let file_manager_label = if platform.to_ascii_lowercase().contains("win") {
        "Explorer"
    } else if platform.to_ascii_lowercase().contains("mac") {
        "Finder"
    } else {
        "Files"
    };

    vec![
        EditorOption {
            label: "Cursor",
            id: EditorId::Cursor,
        },
        EditorOption {
            label: "Trae",
            id: EditorId::Trae,
        },
        EditorOption {
            label: "Kiro",
            id: EditorId::Kiro,
        },
        EditorOption {
            label: "VS Code",
            id: EditorId::VsCode,
        },
        EditorOption {
            label: "VS Code Insiders",
            id: EditorId::VsCodeInsiders,
        },
        EditorOption {
            label: "VSCodium",
            id: EditorId::VsCodium,
        },
        EditorOption {
            label: "Zed",
            id: EditorId::Zed,
        },
        EditorOption {
            label: "Antigravity",
            id: EditorId::Antigravity,
        },
        EditorOption {
            label: "IntelliJ IDEA",
            id: EditorId::Idea,
        },
        EditorOption {
            label: "Aqua",
            id: EditorId::Aqua,
        },
        EditorOption {
            label: "CLion",
            id: EditorId::CLion,
        },
        EditorOption {
            label: "DataGrip",
            id: EditorId::DataGrip,
        },
        EditorOption {
            label: "DataSpell",
            id: EditorId::DataSpell,
        },
        EditorOption {
            label: "GoLand",
            id: EditorId::GoLand,
        },
        EditorOption {
            label: "PhpStorm",
            id: EditorId::PhpStorm,
        },
        EditorOption {
            label: "PyCharm",
            id: EditorId::PyCharm,
        },
        EditorOption {
            label: "Rider",
            id: EditorId::Rider,
        },
        EditorOption {
            label: "RubyMine",
            id: EditorId::RubyMine,
        },
        EditorOption {
            label: "RustRover",
            id: EditorId::RustRover,
        },
        EditorOption {
            label: "WebStorm",
            id: EditorId::WebStorm,
        },
        EditorOption {
            label: file_manager_label,
            id: EditorId::FileManager,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub id: String,
    pub environment_id: String,
    pub project_id: String,
    pub title: String,
    pub project_name: String,
    pub status: ThreadStatus,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub latest_user_message_at: Option<String>,
    pub has_pending_approvals: bool,
    pub has_pending_user_input: bool,
    pub has_actionable_proposed_plan: bool,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle,
    Running,
    NeedsInput,
    Failed,
}

pub const RECENT_COMMAND_PALETTE_THREAD_LIMIT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarThreadSortOrder {
    UpdatedAt,
    CreatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteItemKind {
    Action,
    Submenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteItem {
    pub kind: CommandPaletteItemKind,
    pub value: String,
    pub search_terms: Vec<String>,
    pub title: String,
    pub description: Option<String>,
    pub timestamp: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteGroup {
    pub value: String,
    pub label: String,
    pub items: Vec<CommandPaletteItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteMode {
    Root,
    RootBrowse,
    Submenu,
    SubmenuBrowse,
}

impl CommandPaletteItem {
    pub fn action(
        value: impl Into<String>,
        search_terms: Vec<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            kind: CommandPaletteItemKind::Action,
            value: value.into(),
            search_terms,
            title: title.into(),
            description: None,
            timestamp: None,
            disabled: false,
        }
    }

    pub fn submenu(
        value: impl Into<String>,
        search_terms: Vec<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            kind: CommandPaletteItemKind::Submenu,
            value: value.into(),
            search_terms,
            title: title.into(),
            description: None,
            timestamp: None,
            disabled: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        let description = description.into();
        if !description.is_empty() {
            self.description = Some(description);
        }
        self
    }

    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        let timestamp = timestamp.into();
        if !timestamp.is_empty() {
            self.timestamp = Some(timestamp);
        }
        self
    }
}

pub fn normalize_command_palette_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn rank_command_palette_search_field(field: &str, normalized_query: &str) -> Option<i32> {
    let normalized_field = normalize_command_palette_search_text(field);
    if normalized_field.is_empty() || !normalized_field.contains(normalized_query) {
        return None;
    }
    if normalized_field == normalized_query {
        return Some(3);
    }
    if normalized_field.starts_with(normalized_query) {
        return Some(2);
    }
    Some(1)
}

fn rank_command_palette_item_match(item: &CommandPaletteItem, normalized_query: &str) -> i32 {
    let terms = item
        .search_terms
        .iter()
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return 0;
    }

    for (index, field) in terms.iter().enumerate() {
        if let Some(field_rank) = rank_command_palette_search_field(field, normalized_query) {
            return 1_000 - (index as i32 * 100) + field_rank;
        }
    }

    0
}

pub fn filter_command_palette_groups(
    active_groups: &[CommandPaletteGroup],
    query: &str,
    is_in_submenu: bool,
    project_search_items: &[CommandPaletteItem],
    thread_search_items: &[CommandPaletteItem],
) -> Vec<CommandPaletteGroup> {
    let is_actions_filter = query.starts_with('>');
    let search_query = if is_actions_filter {
        &query[1..]
    } else {
        query
    };
    let normalized_query = normalize_command_palette_search_text(search_query);

    if normalized_query.is_empty() {
        if is_actions_filter {
            return active_groups
                .iter()
                .filter(|group| group.value == "actions")
                .cloned()
                .collect();
        }
        return active_groups.to_vec();
    }

    let mut searchable_groups = active_groups
        .iter()
        .filter(|group| {
            if is_actions_filter {
                group.value == "actions"
            } else {
                is_in_submenu || group.value != "recent-threads"
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    if !is_in_submenu && !is_actions_filter {
        if !project_search_items.is_empty() {
            searchable_groups.push(CommandPaletteGroup {
                value: "projects-search".to_string(),
                label: "Projects".to_string(),
                items: project_search_items.to_vec(),
            });
        }
        if !thread_search_items.is_empty() {
            searchable_groups.push(CommandPaletteGroup {
                value: "threads-search".to_string(),
                label: "Threads".to_string(),
                items: thread_search_items.to_vec(),
            });
        }
    }

    searchable_groups
        .into_iter()
        .filter_map(|group| {
            let mut ranked_items = group
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    let haystack =
                        normalize_command_palette_search_text(&item.search_terms.join(" "));
                    if !haystack.contains(&normalized_query) {
                        return None;
                    }
                    Some((
                        index,
                        rank_command_palette_item_match(item, &normalized_query),
                        item.clone(),
                    ))
                })
                .collect::<Vec<_>>();

            ranked_items.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            let items = ranked_items
                .into_iter()
                .map(|(_, _, item)| item)
                .collect::<Vec<_>>();
            if items.is_empty() {
                None
            } else {
                Some(CommandPaletteGroup { items, ..group })
            }
        })
        .collect()
}

pub fn build_project_action_items(
    projects: &[ProjectSummary],
    value_prefix: &str,
) -> Vec<CommandPaletteItem> {
    projects
        .iter()
        .map(|project| {
            CommandPaletteItem::action(
                format!("{}:{}:{}", value_prefix, project.environment_id, project.id),
                vec![project.name.clone(), project.path.clone()],
                project.name.clone(),
            )
            .with_description(project.path.clone())
        })
        .collect()
}

pub fn build_thread_action_items(
    threads: &[ThreadSummary],
    active_thread_id: Option<&str>,
    projects: &[ProjectSummary],
    sort_order: SidebarThreadSortOrder,
    now_iso: &str,
    limit: Option<usize>,
) -> Vec<CommandPaletteItem> {
    let project_title_by_id = projects
        .iter()
        .map(|project| (project.id.as_str(), project.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut sorted_threads = threads
        .iter()
        .filter(|thread| thread.archived_at.is_none())
        .collect::<Vec<_>>();
    sorted_threads.sort_by(|left, right| {
        let left_timestamp = get_thread_sort_timestamp(left, sort_order);
        let right_timestamp = get_thread_sort_timestamp(right, sort_order);
        right_timestamp
            .cmp(&left_timestamp)
            .then(right.id.cmp(&left.id))
    });

    let visible_threads: Box<dyn Iterator<Item = &ThreadSummary> + '_> = if let Some(limit) = limit
    {
        Box::new(sorted_threads.into_iter().take(limit))
    } else {
        Box::new(sorted_threads.into_iter())
    };

    visible_threads
        .map(|thread| {
            let project_title = project_title_by_id
                .get(thread.project_id.as_str())
                .copied()
                .or_else(|| {
                    if thread.project_name.is_empty() {
                        None
                    } else {
                        Some(thread.project_name.as_str())
                    }
                });
            let mut description_parts = Vec::new();
            if let Some(project_title) = project_title {
                description_parts.push(project_title.to_string());
            }
            if let Some(branch) = &thread.branch {
                description_parts.push(format!("#{branch}"));
            }
            if active_thread_id == Some(thread.id.as_str()) {
                description_parts.push("Current thread".to_string());
            }

            let display_timestamp = thread
                .latest_user_message_at
                .as_deref()
                .unwrap_or(thread.updated_at.as_str());

            CommandPaletteItem::action(
                format!("thread:{}", thread.id),
                vec![
                    thread.title.clone(),
                    project_title.unwrap_or_default().to_string(),
                    thread.branch.clone().unwrap_or_default(),
                ],
                thread.title.clone(),
            )
            .with_description(description_parts.join(" · "))
            .with_timestamp(format_relative_time_label_at(display_timestamp, now_iso))
        })
        .collect()
}

pub fn build_root_command_palette_groups(
    action_items: Vec<CommandPaletteItem>,
    recent_thread_items: Vec<CommandPaletteItem>,
) -> Vec<CommandPaletteGroup> {
    let mut groups = Vec::new();
    if !action_items.is_empty() {
        groups.push(CommandPaletteGroup {
            value: "actions".to_string(),
            label: "Actions".to_string(),
            items: action_items,
        });
    }
    if !recent_thread_items.is_empty() {
        groups.push(CommandPaletteGroup {
            value: "recent-threads".to_string(),
            label: "Recent Threads".to_string(),
            items: recent_thread_items,
        });
    }
    groups
}

pub fn get_command_palette_mode(
    current_view_present: bool,
    is_browsing: bool,
) -> CommandPaletteMode {
    match (current_view_present, is_browsing) {
        (true, true) => CommandPaletteMode::SubmenuBrowse,
        (true, false) => CommandPaletteMode::Submenu,
        (false, true) => CommandPaletteMode::RootBrowse,
        (false, false) => CommandPaletteMode::Root,
    }
}

pub fn get_command_palette_input_placeholder(mode: CommandPaletteMode) -> &'static str {
    match mode {
        CommandPaletteMode::Root => "Search commands, projects, and threads...",
        CommandPaletteMode::RootBrowse => "Enter project path (e.g. ~/projects/my-app)",
        CommandPaletteMode::Submenu => "Search...",
        CommandPaletteMode::SubmenuBrowse => "Enter path (e.g. ~/projects/my-app)",
    }
}

fn get_thread_sort_timestamp(thread: &ThreadSummary, sort_order: SidebarThreadSortOrder) -> i64 {
    if sort_order == SidebarThreadSortOrder::CreatedAt {
        return iso_utc_timestamp_seconds(&thread.created_at).unwrap_or(i64::MIN);
    }

    thread
        .latest_user_message_at
        .as_deref()
        .and_then(iso_utc_timestamp_seconds)
        .or_else(|| iso_utc_timestamp_seconds(&thread.updated_at))
        .or_else(|| iso_utc_timestamp_seconds(&thread.created_at))
        .unwrap_or(i64::MIN)
}

pub fn format_relative_time_label_at(iso_date: &str, now_iso: &str) -> String {
    let Some(now_seconds) = iso_utc_timestamp_seconds(now_iso) else {
        return "just now".to_string();
    };
    let Some(date_seconds) = iso_utc_timestamp_seconds(iso_date) else {
        return "just now".to_string();
    };
    let diff = now_seconds.saturating_sub(date_seconds);
    if date_seconds > now_seconds || diff < 60 {
        return "just now".to_string();
    }
    let minutes = diff / 60;
    if minutes < 60 {
        return format!("{minutes}m ago");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    format!("{}d ago", hours / 24)
}

fn iso_utc_timestamp_seconds(iso: &str) -> Option<i64> {
    let date_time = iso.strip_suffix('Z').unwrap_or(iso);
    let year = date_time.get(0..4)?.parse::<i32>().ok()?;
    let month = date_time.get(5..7)?.parse::<u32>().ok()?;
    let day = date_time.get(8..10)?.parse::<u32>().ok()?;
    let hour = date_time.get(11..13)?.parse::<u32>().ok()?;
    let minute = date_time.get(14..16)?.parse::<u32>().ok()?;
    let second = date_time.get(17..19)?.parse::<u32>().ok()?;

    if date_time.get(4..5) != Some("-")
        || date_time.get(7..8) != Some("-")
        || date_time.get(10..11) != Some("T")
        || date_time.get(13..14) != Some(":")
        || date_time.get(16..17) != Some(":")
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    Some(days * 86_400 + hour as i64 * 3_600 + minute as i64 * 60 + second as i64)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let month_days = [31_u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let max_day = if month == 2 && is_leap_year(year) {
        29
    } else {
        *month_days.get(month.checked_sub(1)? as usize)?
    };
    if day > max_day {
        return None;
    }

    let year = year - (month <= 2) as i32;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some((era * 146_097 + day_of_era - 719_468) as i64)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSshEnvironmentTarget {
    pub alias: String,
    pub hostname: String,
    pub username: Option<String>,
    pub port: Option<u16>,
}

pub fn format_desktop_ssh_target(target: &DesktopSshEnvironmentTarget) -> String {
    let authority = if let Some(username) = target.username.as_deref() {
        format!("{username}@{}", target.hostname)
    } else {
        target.hostname.clone()
    };
    if let Some(port) = target.port {
        format!("{authority}:{port}")
    } else {
        authority
    }
}

pub fn parse_manual_desktop_ssh_target(
    host: &str,
    username: &str,
    port: &str,
) -> Result<DesktopSshEnvironmentTarget, String> {
    let raw_host = host.trim();
    if raw_host.is_empty() {
        return Err("SSH host or alias is required.".to_string());
    }

    let mut hostname = raw_host.to_string();
    let mut username = trimmed_non_empty(username).map(str::to_string);
    let mut parsed_port = None;
    let mut parsed_port_was_provided = false;

    if let Some(at_index) = hostname.rfind('@') {
        if at_index > 0 {
            let inline_username = hostname[..at_index].trim().to_string();
            hostname = hostname[at_index + 1..].trim().to_string();
            if username.is_none() && !inline_username.is_empty() {
                username = Some(inline_username);
            }
        }
    }

    if let Some((bracketed_host, bracketed_port)) = parse_bracketed_host_port(&hostname) {
        hostname = bracketed_host;
        if let Some(port) = bracketed_port {
            parsed_port = Some(port);
            parsed_port_was_provided = true;
        }
    } else if let Some((host_part, port_part)) = hostname.split_once(':') {
        if !host_part.contains(':')
            && !port_part.contains(':')
            && !port_part.is_empty()
            && port_part.chars().all(|ch| ch.is_ascii_digit())
        {
            let next_hostname = host_part.trim().to_string();
            parsed_port = port_part.parse::<i64>().ok();
            hostname = next_hostname;
            parsed_port_was_provided = true;
        }
    }

    let raw_port = port.trim();
    if !raw_port.is_empty() {
        parsed_port = parse_js_base10_int(raw_port);
        parsed_port_was_provided = true;
    }

    if hostname.is_empty() {
        return Err("SSH host or alias is required.".to_string());
    }

    let port = if parsed_port_was_provided {
        let Some(port) = parsed_port else {
            return Err("SSH port must be between 1 and 65535.".to_string());
        };
        if !(1..=65_535).contains(&port) {
            return Err("SSH port must be between 1 and 65535.".to_string());
        }
        Some(port as u16)
    } else {
        None
    };

    Ok(DesktopSshEnvironmentTarget {
        alias: hostname.clone(),
        hostname,
        username,
        port,
    })
}

fn parse_bracketed_host_port(value: &str) -> Option<(String, Option<i64>)> {
    let rest = value.strip_prefix('[')?;
    let closing_index = rest.find(']')?;
    let hostname = rest[..closing_index].trim().to_string();
    let suffix = &rest[closing_index + 1..];
    if suffix.is_empty() {
        return Some((hostname, None));
    }
    let raw_port = suffix.strip_prefix(':')?;
    if raw_port.is_empty() || !raw_port.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((hostname, raw_port.parse::<i64>().ok()))
}

fn parse_js_base10_int(value: &str) -> Option<i64> {
    let value = value.trim_start();
    let mut sign = 1_i64;
    let mut start = 0_usize;
    if let Some(first) = value.as_bytes().first().copied() {
        if first == b'-' {
            sign = -1;
            start = 1;
        } else if first == b'+' {
            start = 1;
        }
    }

    let digits = value[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok().map(|value| value * sign)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePairingFields {
    pub host: String,
    pub pairing_code: String,
}

pub fn parse_pairing_url_fields(input: &str) -> Option<RemotePairingFields> {
    let parsed = ParsedPairingUrl::parse(input.trim())?;
    let token = parsed.pairing_token()?;

    if let Some(host) = parsed.query_param("host") {
        let host = host.trim().to_string();
        if !host.is_empty() && !token.trim().is_empty() {
            return Some(RemotePairingFields {
                host,
                pairing_code: token.trim().to_string(),
            });
        }
    }

    Some(RemotePairingFields {
        host: parsed.origin,
        pairing_code: token.trim().to_string(),
    })
}

pub fn parse_remote_pairing_fields(
    host: &str,
    pairing_code: &str,
) -> Result<RemotePairingFields, String> {
    if let Some(parsed) = parse_pairing_url_fields(host) {
        return Ok(parsed);
    }

    let host = host.trim();
    let pairing_code = pairing_code.trim();
    if host.is_empty() {
        return Err("Enter a backend host.".to_string());
    }
    if pairing_code.is_empty() {
        return Err("Enter a pairing code.".to_string());
    }
    Ok(RemotePairingFields {
        host: host.to_string(),
        pairing_code: pairing_code.to_string(),
    })
}

pub fn format_desktop_ssh_connection_error(error_message: Option<&str>) -> String {
    const FALLBACK: &str = "Failed to connect SSH host.";
    let raw_message = error_message.unwrap_or(FALLBACK);
    let without_ipc_prefix = raw_message
        .strip_prefix("Error invoking remote method 'desktop:ensure-ssh-environment':")
        .map(str::trim_start)
        .unwrap_or(raw_message);
    let without_tagged_prefix =
        strip_ssh_tagged_error_prefix(without_ipc_prefix).unwrap_or(without_ipc_prefix);
    let message = without_tagged_prefix.trim();
    if message.is_empty() {
        FALLBACK.to_string()
    } else {
        message.to_string()
    }
}

fn strip_ssh_tagged_error_prefix(value: &str) -> Option<&str> {
    let suffix = value.strip_prefix("Ssh")?;
    let marker = suffix.find("Error:")?;
    let tag = &suffix[..marker];
    if tag.is_empty() || !tag.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(suffix[marker + "Error:".len()..].trim_start())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvertisedEndpointStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedHttpsAppCompatibility {
    Compatible,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvertisedEndpoint {
    pub id: String,
    pub provider_id: String,
    pub label: String,
    pub http_base_url: String,
    pub reachability: String,
    pub status: AdvertisedEndpointStatus,
    pub is_default: bool,
    pub hosted_https_app: HostedHttpsAppCompatibility,
}

pub fn is_tailscale_https_endpoint(endpoint: &AdvertisedEndpoint) -> bool {
    endpoint.id.starts_with("tailscale-magicdns:")
}

pub fn endpoint_default_preference_key(endpoint: &AdvertisedEndpoint) -> String {
    if endpoint.id.starts_with("desktop-loopback:") {
        return "desktop-core:loopback:http".to_string();
    }
    if endpoint.id.starts_with("desktop-lan:") {
        return "desktop-core:lan:http".to_string();
    }
    if endpoint.id.starts_with("tailscale-ip:") {
        return "tailscale:ip:http".to_string();
    }
    if is_tailscale_https_endpoint(endpoint) {
        return "tailscale:magicdns:https".to_string();
    }

    let scheme = ParsedPairingUrl::parse(&endpoint.http_base_url)
        .map(|url| url.scheme)
        .unwrap_or_else(|| "unknown".to_string());
    format!(
        "{}:{}:{}:{}",
        endpoint.provider_id, endpoint.reachability, scheme, endpoint.label
    )
}

pub fn select_pairing_endpoint<'a>(
    endpoints: &'a [AdvertisedEndpoint],
    default_endpoint_key: Option<&str>,
) -> Option<&'a AdvertisedEndpoint> {
    let available = endpoints
        .iter()
        .filter(|endpoint| endpoint.status != AdvertisedEndpointStatus::Unavailable)
        .collect::<Vec<_>>();

    if let Some(default_endpoint_key) = default_endpoint_key {
        if let Some(endpoint) = available
            .iter()
            .copied()
            .find(|endpoint| endpoint_default_preference_key(endpoint) == default_endpoint_key)
        {
            return Some(endpoint);
        }
    }

    available
        .iter()
        .copied()
        .find(|endpoint| endpoint.is_default)
        .or_else(|| {
            available
                .iter()
                .copied()
                .find(|endpoint| endpoint.reachability != "loopback")
        })
        .or_else(|| {
            available.iter().copied().find(|endpoint| {
                endpoint.hosted_https_app == HostedHttpsAppCompatibility::Compatible
            })
        })
}

pub fn resolve_desktop_pairing_url(endpoint_url: &str, credential: &str) -> Option<String> {
    let parsed = ParsedPairingUrl::parse(endpoint_url)?;
    Some(format!(
        "{}/pair#token={}",
        parsed.origin,
        form_url_encode_component(credential)
    ))
}

pub fn resolve_hosted_pairing_url(endpoint_url: &str, credential: &str) -> Option<String> {
    let parsed = ParsedPairingUrl::parse(endpoint_url)?;
    if parsed.scheme != "https" {
        return None;
    }
    Some(format!(
        "https://app.t3.codes/pair?host={}#token={}",
        form_url_encode_component(endpoint_url),
        form_url_encode_component(credential)
    ))
}

pub fn resolve_advertised_endpoint_pairing_url(
    endpoint: &AdvertisedEndpoint,
    credential: &str,
) -> Option<String> {
    if endpoint.hosted_https_app == HostedHttpsAppCompatibility::Compatible {
        return resolve_hosted_pairing_url(&endpoint.http_base_url, credential)
            .or_else(|| resolve_desktop_pairing_url(&endpoint.http_base_url, credential));
    }
    resolve_desktop_pairing_url(&endpoint.http_base_url, credential)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPairingLinkRecord {
    pub id: String,
    pub created_at: String,
}

pub fn sort_desktop_pairing_links(
    links: &[ServerPairingLinkRecord],
) -> Vec<ServerPairingLinkRecord> {
    let mut sorted = links.to_vec();
    sorted.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    sorted
}

pub fn upsert_desktop_pairing_link(
    current: &[ServerPairingLinkRecord],
    next: ServerPairingLinkRecord,
) -> Vec<ServerPairingLinkRecord> {
    let mut updated = current.to_vec();
    if let Some(existing_index) = updated
        .iter()
        .position(|pairing_link| pairing_link.id == next.id)
    {
        updated[existing_index] = next;
    } else {
        updated.push(next);
    }
    sort_desktop_pairing_links(&updated)
}

pub fn remove_desktop_pairing_link(
    current: &[ServerPairingLinkRecord],
    id: &str,
) -> Vec<ServerPairingLinkRecord> {
    current
        .iter()
        .filter(|pairing_link| pairing_link.id != id)
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerClientSessionRecord {
    pub session_id: String,
    pub issued_at: String,
    pub current: bool,
    pub connected: bool,
}

pub fn sort_desktop_client_sessions(
    sessions: &[ServerClientSessionRecord],
) -> Vec<ServerClientSessionRecord> {
    let mut sorted = sessions.to_vec();
    sorted.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then(right.connected.cmp(&left.connected))
            .then(right.issued_at.cmp(&left.issued_at))
    });
    sorted
}

pub fn upsert_desktop_client_session(
    current: &[ServerClientSessionRecord],
    next: ServerClientSessionRecord,
) -> Vec<ServerClientSessionRecord> {
    let mut updated = current.to_vec();
    if let Some(existing_index) = updated
        .iter()
        .position(|client_session| client_session.session_id == next.session_id)
    {
        updated[existing_index] = next;
    } else {
        updated.push(next);
    }
    sort_desktop_client_sessions(&updated)
}

pub fn remove_desktop_client_session(
    current: &[ServerClientSessionRecord],
    session_id: &str,
) -> Vec<ServerClientSessionRecord> {
    current
        .iter()
        .filter(|client_session| client_session.session_id != session_id)
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsDescriptionInput<'a> {
    pub local_tracing_enabled: bool,
    pub otlp_traces_enabled: bool,
    pub otlp_traces_url: Option<&'a str>,
    pub otlp_metrics_enabled: bool,
    pub otlp_metrics_url: Option<&'a str>,
}

pub fn collapse_otel_signals_url(traces_url: &str, metrics_url: &str) -> Option<String> {
    let traces_suffix = "/traces";
    let metrics_suffix = "/metrics";
    if !traces_url.ends_with(traces_suffix) || !metrics_url.ends_with(metrics_suffix) {
        return None;
    }

    let traces_base = &traces_url[..traces_url.len() - traces_suffix.len()];
    let metrics_base = &metrics_url[..metrics_url.len() - metrics_suffix.len()];
    if traces_base != metrics_base {
        return None;
    }

    Some(format!("{traces_base}/{{traces,metrics}}"))
}

pub fn format_diagnostics_description(input: DiagnosticsDescriptionInput<'_>) -> String {
    let mode = if input.local_tracing_enabled {
        "Local trace file"
    } else {
        "Terminal logs only"
    };
    let traces_url = input
        .otlp_traces_enabled
        .then_some(input.otlp_traces_url)
        .flatten();
    let metrics_url = input
        .otlp_metrics_enabled
        .then_some(input.otlp_metrics_url)
        .flatten();

    match (traces_url, metrics_url) {
        (Some(traces_url), Some(metrics_url)) => {
            if let Some(collapsed_url) = collapse_otel_signals_url(traces_url, metrics_url) {
                format!("{mode}. Exporting OTEL to {collapsed_url}.")
            } else {
                format!(
                    "{mode}. Exporting OTEL traces to {traces_url} and metrics to {metrics_url}."
                )
            }
        }
        (Some(traces_url), None) => format!("{mode}. Exporting OTEL traces to {traces_url}."),
        (None, Some(metrics_url)) => format!("{mode}. Exporting OTEL metrics to {metrics_url}."),
        (None, None) => format!("{mode}."),
    }
}

pub fn format_diagnostics_count(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}

pub fn format_diagnostics_duration_ms(value: f64) -> String {
    if value < 1_000.0 {
        return format!("{} ms", value.round() as i64);
    }
    if value >= 10_000.0 {
        format!("{:.1} s", value / 1_000.0)
    } else {
        format!("{:.2} s", value / 1_000.0)
    }
}

pub fn format_diagnostics_bytes(value: u64) -> String {
    if value < 1024 {
        return format!("{value} B");
    }
    let units = ["KB", "MB", "GB"];
    let mut unit_index = 0_usize;
    let mut next = value as f64 / 1024.0;
    while next >= 1024.0 && unit_index < units.len() - 1 {
        next /= 1024.0;
        unit_index += 1;
    }
    if next >= 10.0 {
        format!("{next:.1} {}", units[unit_index])
    } else {
        format!("{next:.2} {}", units[unit_index])
    }
}

pub fn shorten_trace_id(trace_id: &str) -> String {
    if trace_id.len() <= 32 {
        return trace_id.to_string();
    }
    format!("{}...{}", &trace_id[..18], &trace_id[trace_id.len() - 10..])
}

pub fn is_stale_process_signal_message(message: Option<&str>) -> bool {
    message
        .map(|message| message.contains("not a live descendant"))
        .unwrap_or(false)
}

const POSIX_PROCESS_QUERY_COMMAND: &str = "pid=,ppid=,pgid=,stat=,pcpu=,rss=,etime=,command=";

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDiagnosticsRow {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: Option<i32>,
    pub status: String,
    pub cpu_percent: f64,
    pub rss_bytes: u64,
    pub elapsed: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDiagnosticsEntry {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: Option<i32>,
    pub status: String,
    pub cpu_percent: f64,
    pub rss_bytes: u64,
    pub elapsed: String,
    pub command: String,
    pub depth: usize,
    pub child_pids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDiagnosticsResult {
    pub server_pid: u32,
    pub read_at: String,
    pub process_count: usize,
    pub total_rss_bytes: u64,
    pub total_cpu_percent: f64,
    pub processes: Vec<ProcessDiagnosticsEntry>,
    pub error: Option<String>,
}

pub fn parse_posix_process_rows(output: &str) -> Vec<ProcessDiagnosticsRow> {
    output.lines().filter_map(parse_posix_process_row).collect()
}

fn parse_posix_process_row(line: &str) -> Option<ProcessDiagnosticsRow> {
    if line.trim().is_empty() {
        return None;
    }

    let fields = split_posix_process_fields(line)?;
    let pid = parse_positive_u32(&fields[0])?;
    let ppid = parse_non_negative_u32(&fields[1])?;
    let pgid = fields[2].parse::<i32>().ok()?;
    let status = fields[3].clone();
    let cpu_percent = fields[4].parse::<f64>().ok()?;
    if !cpu_percent.is_finite() {
        return None;
    }
    let rss_kib = fields[5].parse::<u64>().ok()?;
    let elapsed = fields[6].clone();
    let command = fields[7].clone();

    if status.is_empty() || elapsed.is_empty() || command.is_empty() {
        return None;
    }

    Some(ProcessDiagnosticsRow {
        pid,
        ppid,
        pgid: Some(pgid),
        status,
        cpu_percent,
        rss_bytes: rss_kib.saturating_mul(1024),
        elapsed,
        command,
    })
}

fn split_posix_process_fields(line: &str) -> Option<Vec<String>> {
    let mut fields = Vec::with_capacity(8);
    let mut index = 0_usize;
    let bytes = line.as_bytes();

    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }

    for _ in 0..7 {
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if start == index {
            return None;
        }
        fields.push(line[start..index].to_string());
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
    }

    let command = line.get(index..)?.trim_end();
    if command.is_empty() {
        return None;
    }
    fields.push(command.to_string());
    Some(fields)
}

pub fn parse_windows_process_rows(output: &str) -> Vec<ProcessDiagnosticsRow> {
    if output.trim().is_empty() {
        return Vec::new();
    }

    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(output) else {
        return Vec::new();
    };
    match parsed {
        serde_json::Value::Array(records) => records
            .iter()
            .filter_map(normalize_windows_process_row)
            .collect(),
        record => normalize_windows_process_row(&record).into_iter().collect(),
    }
}

fn normalize_windows_process_row(value: &serde_json::Value) -> Option<ProcessDiagnosticsRow> {
    let record = value.as_object()?;
    let pid = record
        .get("ProcessId")?
        .as_u64()
        .and_then(to_positive_u32)?;
    let ppid = record
        .get("ParentProcessId")?
        .as_u64()
        .and_then(to_non_negative_u32)?;
    let command = record
        .get("CommandLine")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| record.get("Name").and_then(serde_json::Value::as_str))?
        .to_string();
    let working_set = record
        .get("WorkingSetSize")
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite())
        .map(|value| value.max(0.0).round() as u64)
        .unwrap_or(0);
    let cpu_percent = record
        .get("PercentProcessorTime")
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite())
        .map(|value| value.max(0.0))
        .unwrap_or(0.0);
    let status = record
        .get("Status")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("Live")
        .to_string();

    Some(ProcessDiagnosticsRow {
        pid,
        ppid,
        pgid: None,
        status,
        cpu_percent,
        rss_bytes: working_set,
        elapsed: String::new(),
        command,
    })
}

pub fn build_process_descendant_entries(
    rows: &[ProcessDiagnosticsRow],
    server_pid: u32,
) -> Vec<ProcessDiagnosticsEntry> {
    let mut children_by_parent = BTreeMap::<u32, Vec<ProcessDiagnosticsRow>>::new();
    for row in rows {
        children_by_parent
            .entry(row.ppid)
            .or_default()
            .push(row.clone());
    }

    for children in children_by_parent.values_mut() {
        children.sort_by_key(|row| row.pid);
    }

    let mut entries = Vec::new();
    let mut visited = Vec::<u32>::new();
    let mut stack = children_by_parent
        .get(&server_pid)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| (row, 0_usize))
        .collect::<Vec<_>>();

    while let Some((row, depth)) = stack.first().cloned() {
        stack.remove(0);
        if visited.contains(&row.pid) {
            continue;
        }
        visited.push(row.pid);

        let children = children_by_parent
            .get(&row.pid)
            .cloned()
            .unwrap_or_default();
        entries.push(ProcessDiagnosticsEntry {
            pid: row.pid,
            ppid: row.ppid,
            pgid: row.pgid,
            status: row.status,
            cpu_percent: row.cpu_percent,
            rss_bytes: row.rss_bytes,
            elapsed: if row.elapsed.is_empty() {
                "n/a".to_string()
            } else {
                row.elapsed
            },
            command: row.command,
            depth,
            child_pids: children.iter().map(|child| child.pid).collect(),
        });

        stack.splice(
            0..0,
            children
                .into_iter()
                .map(|child| (child, depth + 1))
                .collect::<Vec<_>>(),
        );
    }

    entries
}

pub fn aggregate_process_diagnostics(
    server_pid: u32,
    rows: &[ProcessDiagnosticsRow],
    read_at: &str,
) -> ProcessDiagnosticsResult {
    make_process_diagnostics_result(server_pid, rows, read_at, None)
}

pub fn make_process_diagnostics_result(
    server_pid: u32,
    rows: &[ProcessDiagnosticsRow],
    read_at: &str,
    error: Option<&str>,
) -> ProcessDiagnosticsResult {
    let rows = rows
        .iter()
        .filter(|row| !is_diagnostics_query_process(row, server_pid))
        .cloned()
        .collect::<Vec<_>>();
    let processes = build_process_descendant_entries(&rows, server_pid);
    let total_rss_bytes = processes.iter().map(|process| process.rss_bytes).sum();
    let total_cpu_percent = processes
        .iter()
        .map(|process| process.cpu_percent)
        .sum::<f64>();

    ProcessDiagnosticsResult {
        server_pid,
        read_at: read_at.to_string(),
        process_count: processes.len(),
        total_rss_bytes,
        total_cpu_percent,
        processes,
        error: error.map(str::to_string),
    }
}

pub fn is_diagnostics_query_process(row: &ProcessDiagnosticsRow, server_pid: u32) -> bool {
    if row.ppid != server_pid {
        return false;
    }

    let command = row.command.trim();
    let command_lower = command.to_ascii_lowercase();
    let posix_query = command.contains(POSIX_PROCESS_QUERY_COMMAND)
        && (command.starts_with("ps -axo ")
            || command.contains("/ps -axo ")
            || command.contains("\\ps -axo "));
    let windows_query = (command_lower.contains("powershell ")
        || command_lower.contains("powershell.exe"))
        && command_lower.contains("get-ciminstance win32_process");
    posix_query || windows_query
}

fn parse_positive_u32(value: &str) -> Option<u32> {
    let value = value.parse::<u32>().ok()?;
    (value > 0).then_some(value)
}

fn parse_non_negative_u32(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

fn to_positive_u32(value: u64) -> Option<u32> {
    let value = u32::try_from(value).ok()?;
    (value > 0).then_some(value)
}

fn to_non_negative_u32(value: u64) -> Option<u32> {
    u32::try_from(value).ok()
}

const DEFAULT_SLOW_SPAN_THRESHOLD_MS: f64 = 1_000.0;
const TRACE_TOP_LIMIT: usize = 10;
const TRACE_RECENT_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsFile {
    pub path: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDiagnosticsErrorSummary {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsSpanSummary {
    pub name: String,
    pub count: u64,
    pub failure_count: u64,
    pub total_duration_ms: f64,
    pub average_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsSpanOccurrence {
    pub name: String,
    pub duration_ms: f64,
    pub ended_at: String,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsRecentFailure {
    pub name: String,
    pub cause: String,
    pub duration_ms: f64,
    pub ended_at: String,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsFailureSummary {
    pub name: String,
    pub cause: String,
    pub count: u64,
    pub last_seen_at: String,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsLogEvent {
    pub span_name: String,
    pub level: String,
    pub message: String,
    pub seen_at: String,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceDiagnosticsResult {
    pub trace_file_path: String,
    pub scanned_file_paths: Vec<String>,
    pub read_at: String,
    pub record_count: u64,
    pub parse_error_count: u64,
    pub first_span_at: Option<String>,
    pub last_span_at: Option<String>,
    pub failure_count: u64,
    pub interruption_count: u64,
    pub slow_span_threshold_ms: f64,
    pub slow_span_count: u64,
    pub log_level_counts: BTreeMap<String, u64>,
    pub top_spans_by_count: Vec<TraceDiagnosticsSpanSummary>,
    pub slowest_spans: Vec<TraceDiagnosticsSpanOccurrence>,
    pub common_failures: Vec<TraceDiagnosticsFailureSummary>,
    pub latest_failures: Vec<TraceDiagnosticsRecentFailure>,
    pub latest_warning_and_error_logs: Vec<TraceDiagnosticsLogEvent>,
    pub partial_failure: Option<bool>,
    pub error: Option<TraceDiagnosticsErrorSummary>,
}

pub struct TraceDiagnosticsInput<'a> {
    pub trace_file_path: &'a str,
    pub files: &'a [TraceDiagnosticsFile],
    pub scanned_file_paths: Option<Vec<String>>,
    pub slow_span_threshold_ms: Option<f64>,
    pub read_at: &'a str,
    pub error: Option<TraceDiagnosticsErrorSummary>,
    pub partial_failure: bool,
}

#[derive(Debug, Clone)]
struct MutableTraceSpanSummary {
    count: u64,
    failure_count: u64,
    total_duration_ms: f64,
    max_duration_ms: f64,
}

pub fn to_rotated_trace_paths(trace_file_path: &str, max_files: i32) -> Vec<String> {
    let backup_count = max_files.max(0) as usize;
    let mut paths = (0..backup_count)
        .map(|index| format!("{trace_file_path}.{}", backup_count - index))
        .collect::<Vec<_>>();
    paths.push(trace_file_path.to_string());
    paths
}

pub fn aggregate_trace_diagnostics(input: TraceDiagnosticsInput<'_>) -> TraceDiagnosticsResult {
    let slow_span_threshold_ms = input
        .slow_span_threshold_ms
        .unwrap_or(DEFAULT_SLOW_SPAN_THRESHOLD_MS);
    let scanned_file_paths = input
        .scanned_file_paths
        .unwrap_or_else(|| input.files.iter().map(|file| file.path.clone()).collect());

    if input.files.is_empty() {
        return make_empty_trace_diagnostics(
            input.trace_file_path,
            scanned_file_paths,
            input.read_at,
            slow_span_threshold_ms,
            input.error.or_else(|| {
                Some(TraceDiagnosticsErrorSummary {
                    kind: "trace-file-not-found".to_string(),
                    message: "No local trace files were found.".to_string(),
                })
            }),
            input.partial_failure,
        );
    }

    let mut parse_error_count = 0_u64;
    let mut record_count = 0_u64;
    let mut failure_count = 0_u64;
    let mut interruption_count = 0_u64;
    let mut slow_span_count = 0_u64;
    let mut first_span_at_millis = None::<i128>;
    let mut last_span_at_millis = None::<i128>;
    let mut spans_by_name = BTreeMap::<String, MutableTraceSpanSummary>::new();
    let mut failures_by_key = BTreeMap::<String, TraceDiagnosticsFailureSummary>::new();
    let mut latest_failures = Vec::<TraceDiagnosticsRecentFailure>::new();
    let mut slowest_spans = Vec::<TraceDiagnosticsSpanOccurrence>::new();
    let mut latest_warning_and_error_logs = Vec::<TraceDiagnosticsLogEvent>::new();
    let mut log_level_counts = BTreeMap::<String, u64>::new();

    for file in input.files {
        for line in file.text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) else {
                parse_error_count += 1;
                continue;
            };
            let Some(record) = parsed.as_object() else {
                parse_error_count += 1;
                continue;
            };

            let name = trace_string_value(record.get("name"));
            let trace_id = trace_string_value(record.get("traceId"));
            let span_id = trace_string_value(record.get("spanId"));
            let duration_ms = trace_number_value(record.get("durationMs"));
            let ended_at_millis = unix_nano_to_millis(record.get("endTimeUnixNano"))
                .and_then(|millis| unix_millis_to_iso(millis).map(|iso| (millis, iso)));
            let started_at_millis = unix_nano_to_millis(record.get("startTimeUnixNano"));

            let (
                Some(name),
                Some(trace_id),
                Some(span_id),
                Some(duration_ms),
                Some((ended_at_millis, ended_at)),
            ) = (name, trace_id, span_id, duration_ms, ended_at_millis)
            else {
                parse_error_count += 1;
                continue;
            };

            record_count += 1;
            if let Some(started_at_millis) = started_at_millis {
                first_span_at_millis = Some(
                    first_span_at_millis
                        .map(|current| current.min(started_at_millis))
                        .unwrap_or(started_at_millis),
                );
            }
            last_span_at_millis = Some(
                last_span_at_millis
                    .map(|current| current.max(ended_at_millis))
                    .unwrap_or(ended_at_millis),
            );

            let exit = record.get("exit");
            let exit_tag = read_trace_exit_tag(exit);
            let is_failure = exit_tag.as_deref() == Some("Failure");
            let is_interrupted = exit_tag.as_deref() == Some("Interrupted");
            if is_failure {
                failure_count += 1;
            }
            if is_interrupted {
                interruption_count += 1;
            }

            let span_summary =
                spans_by_name
                    .entry(name.clone())
                    .or_insert(MutableTraceSpanSummary {
                        count: 0,
                        failure_count: 0,
                        total_duration_ms: 0.0,
                        max_duration_ms: 0.0,
                    });
            span_summary.count += 1;
            span_summary.total_duration_ms += duration_ms;
            span_summary.max_duration_ms = span_summary.max_duration_ms.max(duration_ms);
            if is_failure {
                span_summary.failure_count += 1;
            }

            let span_item = TraceDiagnosticsSpanOccurrence {
                name: name.clone(),
                duration_ms,
                ended_at: ended_at.clone(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
            };
            if duration_ms >= slow_span_threshold_ms {
                slow_span_count += 1;
            }
            insert_bounded_slowest_span(&mut slowest_spans, span_item);

            if is_failure {
                let cause = read_trace_exit_cause(exit);
                latest_failures.push(TraceDiagnosticsRecentFailure {
                    name: name.clone(),
                    cause: cause.clone(),
                    duration_ms,
                    ended_at: ended_at.clone(),
                    trace_id: trace_id.clone(),
                    span_id: span_id.clone(),
                });

                let failure_key = format!("{name}\0{cause}");
                let existing = failures_by_key.get(&failure_key);
                let is_latest_failure = existing
                    .and_then(|failure| iso_utc_timestamp_millis(&failure.last_seen_at))
                    .map(|current| ended_at_millis > current)
                    .unwrap_or(true);
                failures_by_key.insert(
                    failure_key,
                    TraceDiagnosticsFailureSummary {
                        name: name.clone(),
                        cause,
                        count: existing.map(|failure| failure.count).unwrap_or(0) + 1,
                        last_seen_at: if is_latest_failure {
                            ended_at.clone()
                        } else {
                            existing.unwrap().last_seen_at.clone()
                        },
                        trace_id: if is_latest_failure {
                            trace_id.clone()
                        } else {
                            existing.unwrap().trace_id.clone()
                        },
                        span_id: if is_latest_failure {
                            span_id.clone()
                        } else {
                            existing.unwrap().span_id.clone()
                        },
                    },
                );
            }

            if let Some(events) = record.get("events").and_then(serde_json::Value::as_array) {
                for event in events {
                    let Some(event_record) = event.as_object() else {
                        continue;
                    };
                    let level = event_record
                        .get("attributes")
                        .and_then(serde_json::Value::as_object)
                        .and_then(|attributes| attributes.get("effect.logLevel"))
                        .and_then(|value| trace_string_value(Some(value)));
                    let Some(level) = level else {
                        continue;
                    };

                    *log_level_counts.entry(level.clone()).or_default() += 1;
                    let normalized_level = level.to_ascii_lowercase();
                    if !matches!(
                        normalized_level.as_str(),
                        "warning" | "warn" | "error" | "fatal"
                    ) {
                        continue;
                    }

                    let seen_at = unix_nano_to_millis(event_record.get("timeUnixNano"))
                        .and_then(unix_millis_to_iso)
                        .unwrap_or_else(|| ended_at.clone());
                    let message = event_record
                        .get("name")
                        .and_then(|value| trace_string_value(Some(value)))
                        .map(|value| value.trim().to_string())
                        .unwrap_or_else(|| "Log event".to_string());
                    latest_warning_and_error_logs.push(TraceDiagnosticsLogEvent {
                        span_name: name.clone(),
                        level,
                        message,
                        seen_at,
                        trace_id: trace_id.clone(),
                        span_id: span_id.clone(),
                    });
                }
            }
        }
    }

    let mut top_spans_by_count = spans_by_name
        .into_iter()
        .map(|(name, span)| TraceDiagnosticsSpanSummary {
            name,
            count: span.count,
            failure_count: span.failure_count,
            total_duration_ms: span.total_duration_ms,
            average_duration_ms: if span.count > 0 {
                span.total_duration_ms / span.count as f64
            } else {
                0.0
            },
            max_duration_ms: span.max_duration_ms,
        })
        .collect::<Vec<_>>();
    top_spans_by_count.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| compare_f64_desc(right.max_duration_ms, left.max_duration_ms))
    });
    top_spans_by_count.truncate(TRACE_TOP_LIMIT);

    let mut common_failures = failures_by_key.into_values().collect::<Vec<_>>();
    common_failures.sort_by(|left, right| {
        right.count.cmp(&left.count).then_with(|| {
            iso_utc_timestamp_millis(&right.last_seen_at)
                .cmp(&iso_utc_timestamp_millis(&left.last_seen_at))
        })
    });
    common_failures.truncate(TRACE_TOP_LIMIT);

    latest_failures.sort_by(|left, right| {
        iso_utc_timestamp_millis(&right.ended_at).cmp(&iso_utc_timestamp_millis(&left.ended_at))
    });
    latest_failures.truncate(TRACE_RECENT_LIMIT);

    latest_warning_and_error_logs.sort_by(|left, right| {
        iso_utc_timestamp_millis(&right.seen_at).cmp(&iso_utc_timestamp_millis(&left.seen_at))
    });
    latest_warning_and_error_logs.truncate(TRACE_RECENT_LIMIT);

    TraceDiagnosticsResult {
        trace_file_path: input.trace_file_path.to_string(),
        scanned_file_paths,
        read_at: input.read_at.to_string(),
        record_count,
        parse_error_count,
        first_span_at: first_span_at_millis.and_then(unix_millis_to_iso),
        last_span_at: last_span_at_millis.and_then(unix_millis_to_iso),
        failure_count,
        interruption_count,
        slow_span_threshold_ms,
        slow_span_count,
        log_level_counts,
        top_spans_by_count,
        slowest_spans,
        common_failures,
        latest_failures,
        latest_warning_and_error_logs,
        partial_failure: input.partial_failure.then_some(true),
        error: input.error,
    }
}

fn make_empty_trace_diagnostics(
    trace_file_path: &str,
    scanned_file_paths: Vec<String>,
    read_at: &str,
    slow_span_threshold_ms: f64,
    error: Option<TraceDiagnosticsErrorSummary>,
    partial_failure: bool,
) -> TraceDiagnosticsResult {
    TraceDiagnosticsResult {
        trace_file_path: trace_file_path.to_string(),
        scanned_file_paths,
        read_at: read_at.to_string(),
        record_count: 0,
        parse_error_count: 0,
        first_span_at: None,
        last_span_at: None,
        failure_count: 0,
        interruption_count: 0,
        slow_span_threshold_ms,
        slow_span_count: 0,
        log_level_counts: BTreeMap::new(),
        top_spans_by_count: Vec::new(),
        slowest_spans: Vec::new(),
        common_failures: Vec::new(),
        latest_failures: Vec::new(),
        latest_warning_and_error_logs: Vec::new(),
        partial_failure: partial_failure.then_some(true),
        error,
    }
}

fn insert_bounded_slowest_span(
    slowest_spans: &mut Vec<TraceDiagnosticsSpanOccurrence>,
    span: TraceDiagnosticsSpanOccurrence,
) {
    if slowest_spans.len() >= TRACE_TOP_LIMIT
        && span.duration_ms <= slowest_spans.last().unwrap().duration_ms
    {
        return;
    }

    slowest_spans.push(span);
    slowest_spans.sort_by(|left, right| compare_f64_desc(right.duration_ms, left.duration_ms));
    if slowest_spans.len() > TRACE_TOP_LIMIT {
        slowest_spans.truncate(TRACE_TOP_LIMIT);
    }
}

fn trace_string_value(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?.as_str()?;
    (!value.trim().is_empty()).then(|| value.to_string())
}

fn trace_number_value(value: Option<&serde_json::Value>) -> Option<f64> {
    value
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite())
}

fn read_trace_exit_tag(exit: Option<&serde_json::Value>) -> Option<String> {
    exit.and_then(serde_json::Value::as_object)
        .and_then(|exit| exit.get("_tag"))
        .and_then(|value| trace_string_value(Some(value)))
}

fn read_trace_exit_cause(exit: Option<&serde_json::Value>) -> String {
    exit.and_then(serde_json::Value::as_object)
        .and_then(|exit| exit.get("cause"))
        .and_then(|value| trace_string_value(Some(value)))
        .map(|cause| cause.trim().to_string())
        .unwrap_or_else(|| "Failure".to_string())
}

fn unix_nano_to_millis(value: Option<&serde_json::Value>) -> Option<i128> {
    let value = trace_string_value(value)?;
    let nanos = value.parse::<i128>().ok()?;
    Some(nanos / 1_000_000)
}

fn unix_millis_to_iso(millis: i128) -> Option<String> {
    let seconds = millis.div_euclid(1_000);
    let millis_part = millis.rem_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days.try_into().ok()?);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis_part:03}Z"
    ))
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + (month <= 2) as i64;
    (year as i32, month as u32, day as u32)
}

fn iso_utc_timestamp_millis(iso: &str) -> Option<i128> {
    let date_time = iso.strip_suffix('Z').unwrap_or(iso);
    let seconds = iso_utc_timestamp_seconds(iso)? as i128;
    let millis = if date_time.get(19..20) == Some(".") {
        date_time.get(20..23)?.parse::<i128>().ok()?
    } else {
        0
    };
    Some(seconds * 1_000 + millis)
}

fn compare_f64_desc(left: f64, right: f64) -> std::cmp::Ordering {
    left.partial_cmp(&right)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn trimmed_non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() { None } else { Some(value) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPairingUrl {
    scheme: String,
    origin: String,
    query: String,
    hash: String,
}

impl ParsedPairingUrl {
    fn parse(input: &str) -> Option<Self> {
        if input.is_empty() {
            return None;
        }
        let url_like_input = if input.starts_with("//") {
            format!("https:{input}")
        } else if has_url_scheme_prefix(input) {
            input.to_string()
        } else {
            format!("https://{input}")
        };

        let scheme_end = url_like_input.find("://")?;
        let scheme = url_like_input[..scheme_end].to_ascii_lowercase();
        if scheme.is_empty() {
            return None;
        }
        let after_scheme = &url_like_input[scheme_end + 3..];
        let authority_end = after_scheme
            .find(|ch| matches!(ch, '/' | '?' | '#'))
            .unwrap_or(after_scheme.len());
        let authority = &after_scheme[..authority_end];
        if authority.is_empty() {
            return None;
        }
        let remainder = &after_scheme[authority_end..];
        let query = extract_url_query(remainder).unwrap_or_default();
        let hash = extract_url_hash(remainder).unwrap_or_default();

        Some(Self {
            scheme: scheme.clone(),
            origin: format!("{scheme}://{authority}"),
            query,
            hash,
        })
    }

    fn query_param(&self, name: &str) -> Option<String> {
        get_url_param(&self.query, name)
    }

    fn hash_param(&self, name: &str) -> Option<String> {
        get_url_param(&self.hash, name)
    }

    fn pairing_token(&self) -> Option<String> {
        self.hash_param("token")
            .filter(|token| !token.trim().is_empty())
            .or_else(|| {
                self.query_param("token")
                    .filter(|token| !token.trim().is_empty())
            })
    }
}

fn has_url_scheme_prefix(input: &str) -> bool {
    let Some((scheme, _)) = input.split_once("://") else {
        return false;
    };
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-'))
}

fn extract_url_query(remainder: &str) -> Option<String> {
    let query_start = remainder.find('?')? + 1;
    let query_end = remainder[query_start..]
        .find('#')
        .map(|index| query_start + index)
        .unwrap_or(remainder.len());
    Some(remainder[query_start..query_end].to_string())
}

fn extract_url_hash(remainder: &str) -> Option<String> {
    let hash_start = remainder.find('#')? + 1;
    Some(remainder[hash_start..].to_string())
}

fn get_url_param(params: &str, name: &str) -> Option<String> {
    params.split('&').find_map(|part| {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        (percent_decode_form_component(key) == name).then(|| percent_decode_form_component(value))
    })
}

fn percent_decode_form_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0_usize;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let high = hex_value(bytes[index + 1]);
                let low = hex_value(bytes[index + 2]);
                if let (Some(high), Some(low)) = (high, low) {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn form_url_encode_component(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(byte as char);
        } else if byte == b' ' {
            output.push('+');
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Disconnected,
    Connecting,
    Ready,
    Running,
    Error,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn display_author(self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Assistant => APP_NAME,
            Self::System => "System",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatImageAttachment {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAttachment {
    Image(ChatImageAttachment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub text: String,
    pub attachments: Vec<ChatAttachment>,
    pub turn_id: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub streaming: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityTone {
    Thinking,
    Tool,
    Info,
    Error,
    Approval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalRequestKind {
    Command,
    FileRead,
    FileChange,
}

impl ApprovalRequestKind {
    pub fn from_request_type(request_type: &str) -> Option<Self> {
        match request_type {
            "command_execution_approval" | "exec_command_approval" | "dynamic_tool_call" => {
                Some(Self::Command)
            }
            "file_read_approval" => Some(Self::FileRead),
            "file_change_approval" | "apply_patch_approval" => Some(Self::FileChange),
            _ => None,
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::Command => "Command approval requested",
            Self::FileRead => "File-read approval requested",
            Self::FileChange => "File-change approval requested",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActivityPayload {
    pub request_id: Option<String>,
    pub request_kind: Option<ApprovalRequestKind>,
    pub request_type: Option<String>,
    pub detail: Option<String>,
    pub command: Option<String>,
    pub raw_command: Option<String>,
    pub changed_files: Vec<String>,
    pub title: Option<String>,
    pub item_type: Option<String>,
    pub tool_call_id: Option<String>,
    pub questions: Vec<UserInputQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadActivity {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub tone: ActivityTone,
    pub payload: ActivityPayload,
    pub turn_id: Option<String>,
    pub sequence: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApproval {
    pub request_id: String,
    pub request_kind: ApprovalRequestKind,
    pub created_at: String,
    pub detail: Option<String>,
}

pub const MAX_VISIBLE_WORK_LOG_ENTRIES: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkLogEntry {
    pub id: String,
    pub activity_kind: String,
    pub created_at: String,
    pub label: String,
    pub detail: Option<String>,
    pub command: Option<String>,
    pub raw_command: Option<String>,
    pub changed_files: Vec<String>,
    pub tone: ActivityTone,
    pub tool_title: Option<String>,
    pub item_type: Option<String>,
    pub request_kind: Option<ApprovalRequestKind>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInputQuestionOption {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInputQuestion {
    pub id: String,
    pub header: String,
    pub question: String,
    pub options: Vec<UserInputQuestionOption>,
    pub multi_select: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUserInput {
    pub request_id: String,
    pub created_at: String,
    pub questions: Vec<UserInputQuestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PendingUserInputDraftAnswer {
    pub selected_option_labels: Vec<String>,
    pub custom_answer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingUserInputAnswer {
    Text(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUserInputProgress {
    pub question_index: usize,
    pub active_question: Option<UserInputQuestion>,
    pub active_draft: Option<PendingUserInputDraftAnswer>,
    pub selected_option_labels: Vec<String>,
    pub custom_answer: String,
    pub resolved_answer: Option<PendingUserInputAnswer>,
    pub using_custom_answer: bool,
    pub answered_question_count: usize,
    pub is_last_question: bool,
    pub is_complete: bool,
    pub can_advance: bool,
}

fn normalize_draft_answer(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_selected_option_labels(value: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for label in value {
        let trimmed = label.trim();
        if trimmed.is_empty() || normalized.iter().any(|entry| entry == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

pub fn resolve_pending_user_input_answer(
    question: &UserInputQuestion,
    draft: Option<&PendingUserInputDraftAnswer>,
) -> Option<PendingUserInputAnswer> {
    if let Some(custom_answer) =
        normalize_draft_answer(draft.and_then(|draft| draft.custom_answer.as_deref()))
    {
        return Some(PendingUserInputAnswer::Text(custom_answer));
    }

    let selected_option_labels = draft
        .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
        .unwrap_or_default();
    if question.multi_select {
        return if selected_option_labels.is_empty() {
            None
        } else {
            Some(PendingUserInputAnswer::Multiple(selected_option_labels))
        };
    }

    selected_option_labels
        .first()
        .cloned()
        .map(PendingUserInputAnswer::Text)
}

pub fn set_pending_user_input_custom_answer(
    draft: Option<&PendingUserInputDraftAnswer>,
    custom_answer: impl Into<String>,
) -> PendingUserInputDraftAnswer {
    let custom_answer = custom_answer.into();
    let selected_option_labels = if custom_answer.trim().is_empty() {
        draft
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    PendingUserInputDraftAnswer {
        selected_option_labels,
        custom_answer: Some(custom_answer),
    }
}

pub fn toggle_pending_user_input_option_selection(
    question: &UserInputQuestion,
    draft: Option<&PendingUserInputDraftAnswer>,
    option_label: impl Into<String>,
) -> PendingUserInputDraftAnswer {
    let option_label = option_label.into();
    if question.multi_select {
        let mut selected_option_labels = draft
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default();
        if selected_option_labels
            .iter()
            .any(|label| label == &option_label)
        {
            selected_option_labels.retain(|label| label != &option_label);
        } else {
            selected_option_labels.push(option_label);
        }

        return PendingUserInputDraftAnswer {
            selected_option_labels,
            custom_answer: Some(String::new()),
        };
    }

    PendingUserInputDraftAnswer {
        selected_option_labels: vec![option_label],
        custom_answer: Some(String::new()),
    }
}

pub fn build_pending_user_input_answers(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> Option<BTreeMap<String, PendingUserInputAnswer>> {
    let mut answers = BTreeMap::new();

    for question in questions {
        let answer = resolve_pending_user_input_answer(question, draft_answers.get(&question.id))?;
        answers.insert(question.id.clone(), answer);
    }

    Some(answers)
}

pub fn count_answered_pending_user_input_questions(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> usize {
    questions
        .iter()
        .filter(|question| {
            resolve_pending_user_input_answer(question, draft_answers.get(&question.id)).is_some()
        })
        .count()
}

pub fn find_first_unanswered_pending_user_input_question_index(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
) -> usize {
    questions
        .iter()
        .position(|question| {
            resolve_pending_user_input_answer(question, draft_answers.get(&question.id)).is_none()
        })
        .unwrap_or_else(|| questions.len().saturating_sub(1))
}

pub fn derive_pending_user_input_progress(
    questions: &[UserInputQuestion],
    draft_answers: &BTreeMap<String, PendingUserInputDraftAnswer>,
    question_index: usize,
) -> PendingUserInputProgress {
    let normalized_question_index = if questions.is_empty() {
        0
    } else {
        question_index.min(questions.len() - 1)
    };
    let active_question = questions.get(normalized_question_index).cloned();
    let active_draft = active_question
        .as_ref()
        .and_then(|question| draft_answers.get(&question.id).cloned());
    let resolved_answer = active_question
        .as_ref()
        .and_then(|question| resolve_pending_user_input_answer(question, active_draft.as_ref()));
    let custom_answer = active_draft
        .as_ref()
        .and_then(|draft| draft.custom_answer.clone())
        .unwrap_or_default();
    let answered_question_count =
        count_answered_pending_user_input_questions(questions, draft_answers);
    let is_last_question = questions.is_empty() || normalized_question_index >= questions.len() - 1;

    PendingUserInputProgress {
        question_index: normalized_question_index,
        active_question,
        selected_option_labels: active_draft
            .as_ref()
            .map(|draft| normalize_selected_option_labels(&draft.selected_option_labels))
            .unwrap_or_default(),
        active_draft,
        using_custom_answer: custom_answer.trim().len() > 0,
        custom_answer,
        can_advance: resolved_answer.is_some(),
        resolved_answer,
        answered_question_count,
        is_last_question,
        is_complete: build_pending_user_input_answers(questions, draft_answers).is_some(),
    }
}

pub const DEFAULT_THREAD_TERMINAL_HEIGHT: u32 = 280;
pub const DEFAULT_THREAD_TERMINAL_ID: &str = "default";
pub const MAX_TERMINALS_PER_GROUP: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalGroup {
    pub id: String,
    pub terminal_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalState {
    pub terminal_open: bool,
    pub terminal_height: u32,
    pub terminal_ids: Vec<String>,
    pub running_terminal_ids: Vec<String>,
    pub active_terminal_id: String,
    pub terminal_groups: Vec<ThreadTerminalGroup>,
    pub active_terminal_group_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTerminalLaunchContext {
    pub cwd: String,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSessionSnapshot {
    pub thread_id: String,
    pub terminal_id: String,
    pub cwd: String,
    pub worktree_path: Option<String>,
    pub status: String,
    pub pid: Option<u32>,
    pub history: String,
    pub exit_code: Option<i32>,
    pub exit_signal: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    Output {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        data: String,
    },
    Activity {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        has_running_subprocess: bool,
    },
    Error {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        message: String,
    },
    Cleared {
        thread_id: String,
        terminal_id: String,
        created_at: String,
    },
    Exited {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        exit_code: Option<i32>,
        exit_signal: Option<String>,
    },
    Started {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        snapshot: TerminalSessionSnapshot,
    },
    Restarted {
        thread_id: String,
        terminal_id: String,
        created_at: String,
        snapshot: TerminalSessionSnapshot,
    },
}

impl TerminalEvent {
    pub fn terminal_id(&self) -> &str {
        match self {
            Self::Output { terminal_id, .. }
            | Self::Activity { terminal_id, .. }
            | Self::Error { terminal_id, .. }
            | Self::Cleared { terminal_id, .. }
            | Self::Exited { terminal_id, .. }
            | Self::Started { terminal_id, .. }
            | Self::Restarted { terminal_id, .. } => terminal_id,
        }
    }

    pub fn created_at(&self) -> &str {
        match self {
            Self::Output { created_at, .. }
            | Self::Activity { created_at, .. }
            | Self::Error { created_at, .. }
            | Self::Cleared { created_at, .. }
            | Self::Exited { created_at, .. }
            | Self::Started { created_at, .. }
            | Self::Restarted { created_at, .. } => created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalEventEntry {
    pub id: u64,
    pub event: TerminalEvent,
}

pub fn terminal_group_id(terminal_id: &str) -> String {
    format!("group-{terminal_id}")
}

pub fn create_default_thread_terminal_state() -> ThreadTerminalState {
    ThreadTerminalState {
        terminal_open: false,
        terminal_height: DEFAULT_THREAD_TERMINAL_HEIGHT,
        terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        running_terminal_ids: Vec::new(),
        active_terminal_id: DEFAULT_THREAD_TERMINAL_ID.to_string(),
        terminal_groups: vec![ThreadTerminalGroup {
            id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
            terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        }],
        active_terminal_group_id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
    }
}

fn normalize_terminal_ids(terminal_ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty() || normalized.iter().any(|id| id == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    if normalized.is_empty() {
        normalized.push(DEFAULT_THREAD_TERMINAL_ID.to_string());
    }
    normalized
}

fn normalize_running_terminal_ids(
    running_terminal_ids: &[String],
    terminal_ids: &[String],
) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in running_terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty()
            || !terminal_ids.iter().any(|id| id == trimmed)
            || normalized.iter().any(|id| id == trimmed)
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn assign_unique_terminal_group_id(base_id: &str, used_group_ids: &mut Vec<String>) -> String {
    let base_id = if base_id.trim().is_empty() {
        terminal_group_id(DEFAULT_THREAD_TERMINAL_ID)
    } else {
        base_id.trim().to_string()
    };
    let mut candidate = base_id.clone();
    let mut index = 2;
    while used_group_ids.iter().any(|id| id == &candidate) {
        candidate = format!("{base_id}-{index}");
        index += 1;
    }
    used_group_ids.push(candidate.clone());
    candidate
}

fn normalize_terminal_group_ids(terminal_ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal_id in terminal_ids {
        let trimmed = terminal_id.trim();
        if trimmed.is_empty() || normalized.iter().any(|id| id == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn normalize_terminal_groups(
    terminal_groups: &[ThreadTerminalGroup],
    terminal_ids: &[String],
) -> Vec<ThreadTerminalGroup> {
    let mut assigned_terminal_ids = Vec::<String>::new();
    let mut used_group_ids = Vec::<String>::new();
    let mut next_groups = Vec::<ThreadTerminalGroup>::new();

    for group in terminal_groups {
        let mut group_terminal_ids = Vec::new();
        for terminal_id in normalize_terminal_group_ids(&group.terminal_ids) {
            if !terminal_ids.iter().any(|id| id == &terminal_id)
                || assigned_terminal_ids.iter().any(|id| id == &terminal_id)
            {
                continue;
            }
            assigned_terminal_ids.push(terminal_id.clone());
            group_terminal_ids.push(terminal_id);
        }
        if group_terminal_ids.is_empty() {
            continue;
        }
        let base_group_id = if group.id.trim().is_empty() {
            terminal_group_id(&group_terminal_ids[0])
        } else {
            group.id.trim().to_string()
        };
        next_groups.push(ThreadTerminalGroup {
            id: assign_unique_terminal_group_id(&base_group_id, &mut used_group_ids),
            terminal_ids: group_terminal_ids,
        });
    }

    for terminal_id in terminal_ids {
        if assigned_terminal_ids
            .iter()
            .any(|assigned| assigned == terminal_id)
        {
            continue;
        }
        next_groups.push(ThreadTerminalGroup {
            id: assign_unique_terminal_group_id(
                &terminal_group_id(terminal_id),
                &mut used_group_ids,
            ),
            terminal_ids: vec![terminal_id.clone()],
        });
    }

    if next_groups.is_empty() {
        return vec![ThreadTerminalGroup {
            id: terminal_group_id(DEFAULT_THREAD_TERMINAL_ID),
            terminal_ids: vec![DEFAULT_THREAD_TERMINAL_ID.to_string()],
        }];
    }
    next_groups
}

fn find_terminal_group_index_by_terminal_id(
    terminal_groups: &[ThreadTerminalGroup],
    terminal_id: &str,
) -> Option<usize> {
    terminal_groups
        .iter()
        .position(|group| group.terminal_ids.iter().any(|id| id == terminal_id))
}

pub fn normalize_thread_terminal_state(state: &ThreadTerminalState) -> ThreadTerminalState {
    let terminal_ids = normalize_terminal_ids(&state.terminal_ids);
    let running_terminal_ids =
        normalize_running_terminal_ids(&state.running_terminal_ids, &terminal_ids);
    let active_terminal_id = if terminal_ids
        .iter()
        .any(|terminal_id| terminal_id == &state.active_terminal_id)
    {
        state.active_terminal_id.clone()
    } else {
        terminal_ids[0].clone()
    };
    let terminal_groups = normalize_terminal_groups(&state.terminal_groups, &terminal_ids);
    let active_group_id_from_state = terminal_groups
        .iter()
        .any(|group| group.id == state.active_terminal_group_id)
        .then(|| state.active_terminal_group_id.clone());
    let active_group_id_from_terminal = terminal_groups
        .iter()
        .find(|group| {
            group
                .terminal_ids
                .iter()
                .any(|id| id == &active_terminal_id)
        })
        .map(|group| group.id.clone());

    ThreadTerminalState {
        terminal_open: state.terminal_open,
        terminal_height: if state.terminal_height > 0 {
            state.terminal_height
        } else {
            DEFAULT_THREAD_TERMINAL_HEIGHT
        },
        terminal_ids,
        running_terminal_ids,
        active_terminal_id,
        active_terminal_group_id: active_group_id_from_state
            .or(active_group_id_from_terminal)
            .unwrap_or_else(|| terminal_groups[0].id.clone()),
        terminal_groups,
    }
}

fn upsert_terminal_into_groups(
    state: &ThreadTerminalState,
    terminal_id: &str,
    split: bool,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    let terminal_id = terminal_id.trim();
    if terminal_id.is_empty() {
        return normalized;
    }

    let is_new_terminal = !normalized
        .terminal_ids
        .iter()
        .any(|existing| existing == terminal_id);
    let mut terminal_ids = normalized.terminal_ids.clone();
    if is_new_terminal {
        terminal_ids.push(terminal_id.to_string());
    }
    let mut terminal_groups = normalized.terminal_groups.clone();

    if let Some(existing_group_index) =
        find_terminal_group_index_by_terminal_id(&terminal_groups, terminal_id)
    {
        terminal_groups[existing_group_index]
            .terminal_ids
            .retain(|id| id != terminal_id);
        if terminal_groups[existing_group_index]
            .terminal_ids
            .is_empty()
        {
            terminal_groups.remove(existing_group_index);
        }
    }

    if !split {
        let mut used_group_ids = terminal_groups
            .iter()
            .map(|group| group.id.clone())
            .collect::<Vec<_>>();
        let next_group_id =
            assign_unique_terminal_group_id(&terminal_group_id(terminal_id), &mut used_group_ids);
        terminal_groups.push(ThreadTerminalGroup {
            id: next_group_id.clone(),
            terminal_ids: vec![terminal_id.to_string()],
        });
        return normalize_thread_terminal_state(&ThreadTerminalState {
            terminal_open: true,
            terminal_ids,
            active_terminal_id: terminal_id.to_string(),
            terminal_groups,
            active_terminal_group_id: next_group_id,
            ..normalized
        });
    }

    let mut active_group_index = terminal_groups
        .iter()
        .position(|group| group.id == normalized.active_terminal_group_id)
        .or_else(|| {
            find_terminal_group_index_by_terminal_id(
                &terminal_groups,
                &normalized.active_terminal_id,
            )
        });
    if active_group_index.is_none() {
        let mut used_group_ids = terminal_groups
            .iter()
            .map(|group| group.id.clone())
            .collect::<Vec<_>>();
        let group_id = assign_unique_terminal_group_id(
            &terminal_group_id(&normalized.active_terminal_id),
            &mut used_group_ids,
        );
        terminal_groups.push(ThreadTerminalGroup {
            id: group_id,
            terminal_ids: vec![normalized.active_terminal_id.clone()],
        });
        active_group_index = Some(terminal_groups.len() - 1);
    }

    let Some(active_group_index) = active_group_index else {
        return normalized;
    };
    let destination_group = &mut terminal_groups[active_group_index];
    if is_new_terminal
        && !destination_group
            .terminal_ids
            .iter()
            .any(|id| id == terminal_id)
        && destination_group.terminal_ids.len() >= MAX_TERMINALS_PER_GROUP
    {
        return normalized;
    }
    if !destination_group
        .terminal_ids
        .iter()
        .any(|id| id == terminal_id)
    {
        if let Some(anchor_index) = destination_group
            .terminal_ids
            .iter()
            .position(|id| id == &normalized.active_terminal_id)
        {
            destination_group
                .terminal_ids
                .insert(anchor_index + 1, terminal_id.to_string());
        } else {
            destination_group.terminal_ids.push(terminal_id.to_string());
        }
    }
    let active_terminal_group_id = destination_group.id.clone();

    normalize_thread_terminal_state(&ThreadTerminalState {
        terminal_open: true,
        terminal_ids,
        active_terminal_id: terminal_id.to_string(),
        terminal_groups,
        active_terminal_group_id,
        ..normalized
    })
}

pub fn set_thread_terminal_open(state: &ThreadTerminalState, open: bool) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if normalized.terminal_open == open {
        return normalized;
    }
    ThreadTerminalState {
        terminal_open: open,
        ..normalized
    }
}

pub fn set_thread_terminal_height(state: &ThreadTerminalState, height: u32) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if height == 0 || normalized.terminal_height == height {
        return normalized;
    }
    ThreadTerminalState {
        terminal_height: height,
        ..normalized
    }
}

pub fn split_thread_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    upsert_terminal_into_groups(state, terminal_id, true)
}

pub fn new_thread_terminal(state: &ThreadTerminalState, terminal_id: &str) -> ThreadTerminalState {
    upsert_terminal_into_groups(state, terminal_id, false)
}

pub fn set_thread_active_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let active_terminal_group_id = normalized
        .terminal_groups
        .iter()
        .find(|group| group.terminal_ids.iter().any(|id| id == terminal_id))
        .map(|group| group.id.clone())
        .unwrap_or_else(|| normalized.active_terminal_group_id.clone());
    if normalized.active_terminal_id == terminal_id
        && normalized.active_terminal_group_id == active_terminal_group_id
    {
        return normalized;
    }
    ThreadTerminalState {
        active_terminal_id: terminal_id.to_string(),
        active_terminal_group_id,
        ..normalized
    }
}

pub fn close_thread_terminal(
    state: &ThreadTerminalState,
    terminal_id: &str,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let remaining_terminal_ids = normalized
        .terminal_ids
        .iter()
        .filter(|id| id.as_str() != terminal_id)
        .cloned()
        .collect::<Vec<_>>();
    if remaining_terminal_ids.is_empty() {
        return create_default_thread_terminal_state();
    }
    let closed_terminal_index = normalized
        .terminal_ids
        .iter()
        .position(|id| id == terminal_id)
        .unwrap_or(0);
    let next_active_terminal_id = if normalized.active_terminal_id == terminal_id {
        remaining_terminal_ids
            .get(closed_terminal_index.min(remaining_terminal_ids.len() - 1))
            .cloned()
            .unwrap_or_else(|| remaining_terminal_ids[0].clone())
    } else {
        normalized.active_terminal_id.clone()
    };
    let terminal_groups = normalized
        .terminal_groups
        .iter()
        .filter_map(|group| {
            let terminal_ids = group
                .terminal_ids
                .iter()
                .filter(|id| id.as_str() != terminal_id)
                .cloned()
                .collect::<Vec<_>>();
            (!terminal_ids.is_empty()).then(|| ThreadTerminalGroup {
                id: group.id.clone(),
                terminal_ids,
            })
        })
        .collect::<Vec<_>>();
    let active_terminal_group_id = terminal_groups
        .iter()
        .find(|group| {
            group
                .terminal_ids
                .iter()
                .any(|id| id == &next_active_terminal_id)
        })
        .map(|group| group.id.clone())
        .unwrap_or_else(|| terminal_group_id(&next_active_terminal_id));

    normalize_thread_terminal_state(&ThreadTerminalState {
        terminal_ids: remaining_terminal_ids,
        running_terminal_ids: normalized
            .running_terminal_ids
            .into_iter()
            .filter(|id| id != terminal_id)
            .collect(),
        active_terminal_id: next_active_terminal_id,
        terminal_groups,
        active_terminal_group_id,
        ..normalized
    })
}

pub fn set_thread_terminal_activity(
    state: &ThreadTerminalState,
    terminal_id: &str,
    has_running_subprocess: bool,
) -> ThreadTerminalState {
    let normalized = normalize_thread_terminal_state(state);
    if !normalized.terminal_ids.iter().any(|id| id == terminal_id) {
        return normalized;
    }
    let already_running = normalized
        .running_terminal_ids
        .iter()
        .any(|id| id == terminal_id);
    if already_running == has_running_subprocess {
        return normalized;
    }
    let mut running_terminal_ids = normalized.running_terminal_ids.clone();
    if has_running_subprocess {
        running_terminal_ids.push(terminal_id.to_string());
    } else {
        running_terminal_ids.retain(|id| id != terminal_id);
    }
    ThreadTerminalState {
        running_terminal_ids,
        ..normalized
    }
}

pub fn terminal_running_subprocess_from_event(event: &TerminalEvent) -> Option<bool> {
    match event {
        TerminalEvent::Activity {
            has_running_subprocess,
            ..
        } => Some(*has_running_subprocess),
        TerminalEvent::Started { .. }
        | TerminalEvent::Restarted { .. }
        | TerminalEvent::Exited { .. } => Some(false),
        TerminalEvent::Output { .. }
        | TerminalEvent::Error { .. }
        | TerminalEvent::Cleared { .. } => None,
    }
}

pub fn select_terminal_event_entries_after_snapshot(
    entries: &[TerminalEventEntry],
    snapshot_updated_at: &str,
) -> Vec<TerminalEventEntry> {
    entries
        .iter()
        .filter(|entry| entry.event.created_at() > snapshot_updated_at)
        .cloned()
        .collect()
}

pub fn select_pending_terminal_event_entries(
    entries: &[TerminalEventEntry],
    last_applied_terminal_event_id: u64,
) -> Vec<TerminalEventEntry> {
    entries
        .iter()
        .filter(|entry| entry.id > last_applied_terminal_event_id)
        .cloned()
        .collect()
}

fn activity_lifecycle_rank(kind: &str) -> i32 {
    if kind.ends_with(".started") || kind == "tool.started" {
        return 0;
    }
    if kind.ends_with(".progress") || kind.ends_with(".updated") {
        return 1;
    }
    if kind.ends_with(".completed") || kind.ends_with(".resolved") {
        return 2;
    }
    1
}

fn sorted_activities(activities: &[ThreadActivity]) -> Vec<ThreadActivity> {
    let mut ordered = activities.to_vec();
    ordered.sort_by(|left, right| match (left.sequence, right.sequence) {
        (Some(left_sequence), Some(right_sequence)) if left_sequence != right_sequence => {
            left_sequence.cmp(&right_sequence)
        }
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        _ => left
            .created_at
            .cmp(&right.created_at)
            .then_with(|| {
                activity_lifecycle_rank(&left.kind).cmp(&activity_lifecycle_rank(&right.kind))
            })
            .then_with(|| left.id.cmp(&right.id)),
    });
    ordered
}

fn normalize_compact_tool_label(value: &str) -> String {
    let trimmed = value.trim();
    for suffix in [" complete", " completed"] {
        if trimmed.to_ascii_lowercase().ends_with(suffix) {
            return trimmed[..trimmed.len() - suffix.len()].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn is_plan_boundary_tool_activity(activity: &ThreadActivity) -> bool {
    matches!(activity.kind.as_str(), "tool.updated" | "tool.completed")
        && activity
            .payload
            .detail
            .as_deref()
            .map(|detail| detail.starts_with("ExitPlanMode:"))
            .unwrap_or(false)
}

fn is_excluded_work_log_activity(activity: &ThreadActivity) -> bool {
    matches!(
        activity.kind.as_str(),
        "tool.started" | "task.started" | "context-window.updated"
    ) || activity.summary == "Checkpoint captured"
        || is_plan_boundary_tool_activity(activity)
}

fn work_log_tone(activity: &ThreadActivity) -> ActivityTone {
    if activity.kind == "task.progress" {
        ActivityTone::Thinking
    } else if activity.tone == ActivityTone::Approval {
        ActivityTone::Info
    } else {
        activity.tone
    }
}

fn work_log_collapse_key(entry: &WorkLogEntry) -> Option<String> {
    if let Some(tool_call_id) = entry.tool_call_id.as_ref() {
        return Some(format!("tool:{tool_call_id}"));
    }
    let label = normalize_compact_tool_label(entry.tool_title.as_deref().unwrap_or(&entry.label));
    let detail = entry.detail.as_deref().unwrap_or("").trim();
    let item_type = entry.item_type.as_deref().unwrap_or("");
    if label.is_empty() && detail.is_empty() && item_type.is_empty() {
        None
    } else {
        Some(format!("{item_type}\u{1f}{label}\u{1f}{detail}"))
    }
}

fn should_collapse_work_log_entries(previous: &WorkLogEntry, next: &WorkLogEntry) -> bool {
    matches!(previous.activity_kind.as_str(), "tool.updated")
        && matches!(
            next.activity_kind.as_str(),
            "tool.updated" | "tool.completed"
        )
        && work_log_collapse_key(previous).is_some()
        && work_log_collapse_key(previous) == work_log_collapse_key(next)
}

fn merge_work_log_entries(previous: WorkLogEntry, next: WorkLogEntry) -> WorkLogEntry {
    let mut changed_files = previous.changed_files;
    for file in next.changed_files {
        if !changed_files.iter().any(|existing| existing == &file) {
            changed_files.push(file);
        }
    }

    WorkLogEntry {
        detail: next.detail.or(previous.detail),
        command: next.command.or(previous.command),
        raw_command: next.raw_command.or(previous.raw_command),
        changed_files,
        tool_title: next.tool_title.or(previous.tool_title),
        item_type: next.item_type.or(previous.item_type),
        request_kind: next.request_kind.or(previous.request_kind),
        tool_call_id: next.tool_call_id.or(previous.tool_call_id),
        ..next
    }
}

fn thread_activity_to_work_log_entry(activity: ThreadActivity) -> WorkLogEntry {
    let is_task_activity = activity.kind == "task.progress" || activity.kind == "task.completed";
    let label = if is_task_activity {
        activity
            .payload
            .title
            .clone()
            .or_else(|| activity.payload.detail.clone())
            .unwrap_or_else(|| activity.summary.clone())
    } else {
        activity.summary.clone()
    };
    let tone = work_log_tone(&activity);
    let request_kind = activity.payload.request_kind.or_else(|| {
        activity
            .payload
            .request_type
            .as_deref()
            .and_then(ApprovalRequestKind::from_request_type)
    });

    WorkLogEntry {
        id: activity.id,
        activity_kind: activity.kind,
        created_at: activity.created_at,
        label,
        detail: activity.payload.detail,
        command: activity.payload.command,
        raw_command: activity.payload.raw_command,
        changed_files: activity.payload.changed_files,
        tone,
        tool_title: activity.payload.title,
        item_type: activity.payload.item_type,
        request_kind,
        tool_call_id: activity.payload.tool_call_id,
    }
}

pub fn derive_work_log_entries(
    activities: &[ThreadActivity],
    latest_turn_id: Option<&str>,
) -> Vec<WorkLogEntry> {
    let mut collapsed = Vec::<WorkLogEntry>::new();

    for activity in sorted_activities(activities) {
        if latest_turn_id
            .map(|turn_id| activity.turn_id.as_deref() != Some(turn_id))
            .unwrap_or(false)
        {
            continue;
        }
        if is_excluded_work_log_activity(&activity) {
            continue;
        }
        let entry = thread_activity_to_work_log_entry(activity);
        if let Some(previous) = collapsed.pop() {
            if should_collapse_work_log_entries(&previous, &entry) {
                collapsed.push(merge_work_log_entries(previous, entry));
            } else {
                collapsed.push(previous);
                collapsed.push(entry);
            }
        } else {
            collapsed.push(entry);
        }
    }

    collapsed
}

fn is_stale_pending_request_failure_detail(detail: Option<&str>) -> bool {
    let Some(detail) = detail else {
        return false;
    };
    let normalized = detail.to_ascii_lowercase();
    normalized.contains("stale pending approval request")
        || normalized.contains("stale pending user-input request")
        || normalized.contains("unknown pending approval request")
        || normalized.contains("unknown pending permission request")
        || normalized.contains("unknown pending user-input request")
}

pub fn derive_pending_approvals(activities: &[ThreadActivity]) -> Vec<PendingApproval> {
    let mut open_by_request_id = BTreeMap::<String, PendingApproval>::new();

    for activity in sorted_activities(activities) {
        let Some(request_id) = activity.payload.request_id.clone() else {
            continue;
        };
        let request_kind = activity.payload.request_kind.or_else(|| {
            activity
                .payload
                .request_type
                .as_deref()
                .and_then(ApprovalRequestKind::from_request_type)
        });

        match activity.kind.as_str() {
            "approval.requested" => {
                if let Some(request_kind) = request_kind {
                    open_by_request_id.insert(
                        request_id.clone(),
                        PendingApproval {
                            request_id,
                            request_kind,
                            created_at: activity.created_at,
                            detail: activity.payload.detail,
                        },
                    );
                }
            }
            "approval.resolved" => {
                open_by_request_id.remove(&request_id);
            }
            "provider.approval.respond.failed"
                if is_stale_pending_request_failure_detail(activity.payload.detail.as_deref()) =>
            {
                open_by_request_id.remove(&request_id);
            }
            _ => {}
        }
    }

    let mut pending = open_by_request_id.into_values().collect::<Vec<_>>();
    pending.sort_by(|left, right| left.created_at.cmp(&right.created_at));
    pending
}

pub fn derive_pending_user_inputs(activities: &[ThreadActivity]) -> Vec<PendingUserInput> {
    let mut open_by_request_id = BTreeMap::<String, PendingUserInput>::new();

    for activity in sorted_activities(activities) {
        let Some(request_id) = activity.payload.request_id.clone() else {
            continue;
        };

        match activity.kind.as_str() {
            "user-input.requested" => {
                if !activity.payload.questions.is_empty() {
                    open_by_request_id.insert(
                        request_id.clone(),
                        PendingUserInput {
                            request_id,
                            created_at: activity.created_at,
                            questions: activity.payload.questions,
                        },
                    );
                }
            }
            "user-input.resolved" => {
                open_by_request_id.remove(&request_id);
            }
            "provider.user-input.respond.failed"
                if is_stale_pending_request_failure_detail(activity.payload.detail.as_deref()) =>
            {
                open_by_request_id.remove(&request_id);
            }
            _ => {}
        }
    }

    let mut pending = open_by_request_id.into_values().collect::<Vec<_>>();
    pending.sort_by(|left, right| left.created_at.cmp(&right.created_at));
    pending
}

impl ChatMessage {
    pub fn user(
        id: impl Into<String>,
        text: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            role: MessageRole::User,
            text: text.into(),
            attachments: Vec::new(),
            turn_id: None,
            created_at: created_at.into(),
            completed_at: None,
            streaming: false,
        }
    }

    pub fn assistant(
        id: impl Into<String>,
        text: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            role: MessageRole::Assistant,
            text: text.into(),
            attachments: Vec::new(),
            turn_id: None,
            created_at: created_at.into(),
            completed_at: None,
            streaming: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedPlan {
    pub id: String,
    pub turn_id: Option<String>,
    pub plan_markdown: String,
    pub implemented_at: Option<String>,
    pub implementation_thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnDiffFileChange {
    pub path: String,
    pub kind: Option<String>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TurnDiffStat {
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnDiffTreeNode {
    Directory {
        name: String,
        path: String,
        stat: TurnDiffStat,
        children: Vec<TurnDiffTreeNode>,
    },
    File {
        name: String,
        path: String,
        stat: Option<TurnDiffStat>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffRouteSearch {
    pub diff: Option<String>,
    pub diff_turn_id: Option<String>,
    pub diff_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOpenValue {
    String(String),
    Number(i32),
    Bool(bool),
}

impl From<&str> for DiffOpenValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

fn is_diff_open_value(value: Option<&DiffOpenValue>) -> bool {
    matches!(
        value,
        Some(DiffOpenValue::String(value)) if value == "1"
    ) || matches!(
        value,
        Some(DiffOpenValue::Number(1)) | Some(DiffOpenValue::Bool(true))
    )
}

fn normalize_search_string(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn parse_diff_route_search(
    diff: Option<DiffOpenValue>,
    diff_turn_id: Option<&str>,
    diff_file_path: Option<&str>,
) -> DiffRouteSearch {
    if !is_diff_open_value(diff.as_ref()) {
        return DiffRouteSearch::default();
    }
    let diff_turn_id = normalize_search_string(diff_turn_id);
    let diff_file_path = diff_turn_id
        .as_ref()
        .and_then(|_| normalize_search_string(diff_file_path));

    DiffRouteSearch {
        diff: Some("1".to_string()),
        diff_turn_id,
        diff_file_path,
    }
}

pub fn summarize_turn_diff_stats(files: &[TurnDiffFileChange]) -> TurnDiffStat {
    files
        .iter()
        .fold(TurnDiffStat::default(), |mut stat, file| {
            if let (Some(additions), Some(deletions)) = (file.additions, file.deletions) {
                stat.additions += additions;
                stat.deletions += deletions;
            }
            stat
        })
}

pub fn has_non_zero_turn_diff_stat(stat: TurnDiffStat) -> bool {
    stat.additions > 0 || stat.deletions > 0
}

#[derive(Debug, Clone)]
struct MutableDiffDirectory {
    name: String,
    path: String,
    stat: TurnDiffStat,
    directories: BTreeMap<String, MutableDiffDirectory>,
    files: Vec<TurnDiffTreeNode>,
}

fn normalize_diff_path_segments(path: &str) -> Vec<String> {
    path.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn read_turn_diff_stat(file: &TurnDiffFileChange) -> Option<TurnDiffStat> {
    Some(TurnDiffStat {
        additions: file.additions?,
        deletions: file.deletions?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DiffSortToken {
    Text(String),
    Number(u128),
}

fn diff_sort_tokens(value: &str) -> Vec<DiffSortToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_digit = None;

    for character in value.chars() {
        let is_digit = character.is_ascii_digit();
        if current_is_digit == Some(is_digit) {
            current.push(character);
            continue;
        }
        if !current.is_empty() {
            if current_is_digit == Some(true) {
                tokens.push(DiffSortToken::Number(current.parse().unwrap_or(0)));
            } else {
                tokens.push(DiffSortToken::Text(current.to_ascii_lowercase()));
            }
        }
        current.clear();
        current.push(character);
        current_is_digit = Some(is_digit);
    }

    if !current.is_empty() {
        if current_is_digit == Some(true) {
            tokens.push(DiffSortToken::Number(current.parse().unwrap_or(0)));
        } else {
            tokens.push(DiffSortToken::Text(current.to_ascii_lowercase()));
        }
    }

    tokens
}

fn compare_diff_names(left: &str, right: &str) -> std::cmp::Ordering {
    diff_sort_tokens(left)
        .cmp(&diff_sort_tokens(right))
        .then_with(|| left.cmp(right))
}

fn compare_diff_node_name(left: &TurnDiffTreeNode, right: &TurnDiffTreeNode) -> std::cmp::Ordering {
    compare_diff_names(diff_node_name(left), diff_node_name(right))
}

fn diff_node_name(node: &TurnDiffTreeNode) -> &str {
    match node {
        TurnDiffTreeNode::Directory { name, .. } | TurnDiffTreeNode::File { name, .. } => name,
    }
}

fn compact_diff_directory_node(node: TurnDiffTreeNode) -> TurnDiffTreeNode {
    let TurnDiffTreeNode::Directory {
        mut name,
        mut path,
        mut stat,
        mut children,
    } = node
    else {
        return node;
    };

    children = children
        .into_iter()
        .map(|child| match child {
            TurnDiffTreeNode::Directory { .. } => compact_diff_directory_node(child),
            TurnDiffTreeNode::File { .. } => child,
        })
        .collect();

    loop {
        if children.len() != 1 {
            break;
        }
        match children.pop().expect("single child exists") {
            TurnDiffTreeNode::Directory {
                name: child_name,
                path: child_path,
                stat: child_stat,
                children: child_children,
            } => {
                name = format!("{name}/{child_name}");
                path = child_path;
                stat = child_stat;
                children = child_children;
            }
            child @ TurnDiffTreeNode::File { .. } => {
                children.push(child);
                break;
            }
        }
    }

    TurnDiffTreeNode::Directory {
        name,
        path,
        stat,
        children,
    }
}

fn to_turn_diff_tree_nodes(directory: MutableDiffDirectory) -> Vec<TurnDiffTreeNode> {
    let mut directories = directory
        .directories
        .into_values()
        .map(|directory| {
            compact_diff_directory_node(TurnDiffTreeNode::Directory {
                name: directory.name.clone(),
                path: directory.path.clone(),
                stat: directory.stat,
                children: to_turn_diff_tree_nodes(directory),
            })
        })
        .collect::<Vec<_>>();
    let mut files = directory.files;
    directories.sort_by(compare_diff_node_name);
    files.sort_by(compare_diff_node_name);
    directories.extend(files);
    directories
}

pub fn build_turn_diff_tree(files: &[TurnDiffFileChange]) -> Vec<TurnDiffTreeNode> {
    let mut root = MutableDiffDirectory {
        name: String::new(),
        path: String::new(),
        stat: TurnDiffStat::default(),
        directories: BTreeMap::new(),
        files: Vec::new(),
    };

    for file in files {
        let segments = normalize_diff_path_segments(&file.path);
        let Some(file_name) = segments.last().cloned() else {
            continue;
        };
        let file_path = segments.join("/");
        let stat = read_turn_diff_stat(file);
        let mut current = &mut root;
        if let Some(stat) = stat {
            current.stat.additions += stat.additions;
            current.stat.deletions += stat.deletions;
        }
        for segment in &segments[..segments.len().saturating_sub(1)] {
            let next_path = if current.path.is_empty() {
                segment.clone()
            } else {
                format!("{}/{}", current.path, segment)
            };
            current = current
                .directories
                .entry(segment.clone())
                .or_insert_with(|| MutableDiffDirectory {
                    name: segment.clone(),
                    path: next_path,
                    stat: TurnDiffStat::default(),
                    directories: BTreeMap::new(),
                    files: Vec::new(),
                });
            if let Some(stat) = stat {
                current.stat.additions += stat.additions;
                current.stat.deletions += stat.deletions;
            }
        }
        current.files.push(TurnDiffTreeNode::File {
            name: file_name,
            path: file_path,
            stat,
        });
    }

    to_turn_diff_tree_nodes(root)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnDiffSummary {
    pub turn_id: String,
    pub completed_at: String,
    pub status: Option<String>,
    pub files: Vec<TurnDiffFileChange>,
    pub checkpoint_ref: Option<String>,
    pub assistant_message_id: Option<String>,
    pub checkpoint_turn_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSession {
    pub provider: String,
    pub provider_instance_id: Option<String>,
    pub status: SessionPhase,
    pub active_turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_error: Option<String>,
    pub orchestration_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadShell {
    pub id: String,
    pub environment_id: String,
    pub codex_thread_id: Option<String>,
    pub project_id: String,
    pub title: String,
    pub runtime_mode: RuntimeMode,
    pub interaction_mode: ProviderInteractionMode,
    pub error: Option<String>,
    pub created_at: String,
    pub archived_at: Option<String>,
    pub updated_at: Option<String>,
    pub branch: Option<String>,
    pub worktree_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTurnState {
    pub latest_turn: Option<LatestTurn>,
    pub pending_source_proposed_plan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestTurn {
    pub turn_id: String,
    pub state: String,
    pub requested_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub assistant_message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thread {
    pub shell: ThreadShell,
    pub session: Option<ThreadSession>,
    pub latest_turn: Option<LatestTurn>,
    pub pending_source_proposed_plan: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub activities: Vec<ThreadActivity>,
    pub proposed_plans: Vec<ProposedPlan>,
    pub turn_diff_summaries: Vec<TurnDiffSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvironmentState {
    pub thread_shell_by_id: BTreeMap<String, ThreadShell>,
    pub thread_session_by_id: BTreeMap<String, ThreadSession>,
    pub thread_turn_state_by_id: BTreeMap<String, ThreadTurnState>,
    pub message_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub message_by_thread_id: BTreeMap<String, BTreeMap<String, ChatMessage>>,
    pub activity_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub activity_by_thread_id: BTreeMap<String, BTreeMap<String, ThreadActivity>>,
    pub proposed_plan_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub proposed_plan_by_thread_id: BTreeMap<String, BTreeMap<String, ProposedPlan>>,
    pub turn_diff_ids_by_thread_id: BTreeMap<String, Vec<String>>,
    pub turn_diff_summary_by_thread_id: BTreeMap<String, BTreeMap<String, TurnDiffSummary>>,
}

fn collect_by_ids<T: Clone>(
    ids_by_owner: &BTreeMap<String, Vec<String>>,
    records_by_owner: &BTreeMap<String, BTreeMap<String, T>>,
    owner_id: &str,
) -> Vec<T> {
    let Some(ids) = ids_by_owner.get(owner_id) else {
        return Vec::new();
    };
    let Some(records) = records_by_owner.get(owner_id) else {
        return Vec::new();
    };

    ids.iter()
        .filter_map(|id| records.get(id).cloned())
        .collect()
}

pub fn get_thread_from_environment_state(
    state: &EnvironmentState,
    thread_id: &str,
) -> Option<Thread> {
    let shell = state.thread_shell_by_id.get(thread_id)?.clone();
    let turn_state = state.thread_turn_state_by_id.get(thread_id);

    Some(Thread {
        shell,
        session: state.thread_session_by_id.get(thread_id).cloned(),
        latest_turn: turn_state.and_then(|state| state.latest_turn.clone()),
        pending_source_proposed_plan: turn_state
            .and_then(|state| state.pending_source_proposed_plan.clone()),
        messages: collect_by_ids(
            &state.message_ids_by_thread_id,
            &state.message_by_thread_id,
            thread_id,
        ),
        activities: collect_by_ids(
            &state.activity_ids_by_thread_id,
            &state.activity_by_thread_id,
            thread_id,
        ),
        proposed_plans: collect_by_ids(
            &state.proposed_plan_ids_by_thread_id,
            &state.proposed_plan_by_thread_id,
            thread_id,
        ),
        turn_diff_summaries: collect_by_ids(
            &state.turn_diff_ids_by_thread_id,
            &state.turn_diff_summary_by_thread_id,
            thread_id,
        ),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub route: ChatRoute,
    pub projects: Vec<ProjectSummary>,
    pub threads: Vec<ThreadSummary>,
    pub is_git_repo: bool,
    pub available_environments: Vec<BranchToolbarEnvironmentOption>,
    pub vcs_refs: Vec<VcsRef>,
    pub current_git_branch: Option<String>,
    pub primary_environment_id: Option<String>,
    pub available_editors: Vec<EditorId>,
    pub preferred_editor: Option<EditorId>,
    pub providers: Vec<ServerProvider>,
    pub selected_provider_instance_id: String,
    pub selected_model: String,
    pub model_favorites: Vec<ProviderModelFavorite>,
    pub messages: Vec<ChatMessage>,
    pub activities: Vec<ThreadActivity>,
    pub draft_sessions: Vec<DraftSessionState>,
    pub pending_approvals: Vec<PendingApproval>,
    pub pending_user_inputs: Vec<PendingUserInput>,
    pub pending_user_input_draft_answers: BTreeMap<String, PendingUserInputDraftAnswer>,
    pub active_pending_user_input_question_index: usize,
    pub responding_request_ids: Vec<String>,
    pub terminal_state: ThreadTerminalState,
    pub terminal_launch_context: Option<ThreadTerminalLaunchContext>,
    pub terminal_event_entries: Vec<TerminalEventEntry>,
    pub diff_route: DiffRouteSearch,
    pub turn_diff_summaries: Vec<TurnDiffSummary>,
}

impl AppSnapshot {
    fn reference_environments() -> Vec<BranchToolbarEnvironmentOption> {
        vec![
            BranchToolbarEnvironmentOption {
                environment_id: "local".to_string(),
                project_id: "project-r3code".to_string(),
                label: resolve_environment_option_label(
                    true,
                    "local",
                    Some("Local environment"),
                    Some("Local"),
                ),
                is_primary: true,
            },
            BranchToolbarEnvironmentOption {
                environment_id: "environment-build-box".to_string(),
                project_id: "project-r3code".to_string(),
                label: resolve_environment_option_label(
                    false,
                    "environment-build-box",
                    None,
                    Some("Build box"),
                ),
                is_primary: false,
            },
        ]
    }

    fn reference_vcs_refs() -> Vec<VcsRef> {
        dedupe_remote_branches_with_local_matches(&[
            VcsRef {
                name: "main".to_string(),
                current: true,
                is_default: true,
                is_remote: false,
                remote_name: None,
                worktree_path: None,
            },
            VcsRef {
                name: "feature/parity-branch-toolbar".to_string(),
                current: false,
                is_default: false,
                is_remote: false,
                remote_name: None,
                worktree_path: Some(
                    "C:\\Users\\bunny\\Downloads\\r3code\\.t3\\worktrees\\branch-toolbar"
                        .to_string(),
                ),
            },
            VcsRef {
                name: "origin/main".to_string(),
                current: false,
                is_default: true,
                is_remote: true,
                remote_name: Some("origin".to_string()),
                worktree_path: None,
            },
            VcsRef {
                name: "origin/feature/remote-only".to_string(),
                current: false,
                is_default: false,
                is_remote: true,
                remote_name: Some("origin".to_string()),
                worktree_path: None,
            },
        ])
    }

    fn reference_project_scripts() -> Vec<ProjectScript> {
        vec![
            ProjectScript {
                id: "test".to_string(),
                name: "Test".to_string(),
                command: "cargo test --workspace".to_string(),
                icon: ProjectScriptIcon::Test,
                run_on_worktree_create: false,
            },
            ProjectScript {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                command: "cargo fetch".to_string(),
                icon: ProjectScriptIcon::Configure,
                run_on_worktree_create: true,
            },
        ]
    }

    fn reference_providers() -> Vec<ServerProvider> {
        vec![
            ServerProvider {
                instance_id: "codex".to_string(),
                driver: "codex".to_string(),
                display_name: Some("Codex".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: Some("codex-default".to_string()),
                show_interaction_mode_toggle: true,
                enabled: true,
                installed: true,
                version: Some("0.49.0".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("codex".to_string()),
                    label: Some("Codex CLI".to_string()),
                    email: Some("dev@example.com".to_string()),
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "gpt-5.4".to_string(),
                        name: "GPT-5.4".to_string(),
                        short_name: Some("5.4".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "gpt-5.4-mini".to_string(),
                        name: "GPT-5.4 Mini".to_string(),
                        short_name: Some("5.4 Mini".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "gpt-5.3-codex".to_string(),
                        name: "GPT-5.3 Codex".to_string(),
                        short_name: Some("5.3".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                ],
                version_advisory: Some(ServerProviderVersionAdvisory {
                    status: ServerProviderVersionAdvisoryStatus::BehindLatest,
                    current_version: Some("0.49.0".to_string()),
                    latest_version: Some("0.50.0".to_string()),
                    update_command: Some("npm install -g @openai/codex@latest".to_string()),
                    can_update: true,
                    checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
                    message: None,
                }),
            },
            ServerProvider {
                instance_id: "codex_personal".to_string(),
                driver: "codex".to_string(),
                display_name: Some("Codex".to_string()),
                accent_color: Some("#2563EB".to_string()),
                badge_label: Some("Personal".to_string()),
                continuation_group_key: Some("codex-personal".to_string()),
                show_interaction_mode_toggle: true,
                enabled: true,
                installed: true,
                version: Some("0.50.0".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("codex".to_string()),
                    label: Some("Personal".to_string()),
                    email: Some("personal@example.com".to_string()),
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "gpt-5.4".to_string(),
                        name: "GPT-5.4".to_string(),
                        short_name: Some("5.4".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "internal-review".to_string(),
                        name: "internal-review".to_string(),
                        short_name: None,
                        sub_provider: Some("OpenAI".to_string()),
                        is_custom: true,
                    },
                ],
                version_advisory: Some(ServerProviderVersionAdvisory {
                    status: ServerProviderVersionAdvisoryStatus::Current,
                    current_version: Some("0.50.0".to_string()),
                    latest_version: Some("0.50.0".to_string()),
                    update_command: None,
                    can_update: false,
                    checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
                    message: None,
                }),
            },
            ServerProvider {
                instance_id: "claudeAgent".to_string(),
                driver: "claudeAgent".to_string(),
                display_name: Some("Claude".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: Some("claude-default".to_string()),
                show_interaction_mode_toggle: false,
                enabled: true,
                installed: true,
                version: Some("1.2.3".to_string()),
                status: ServerProviderState::Ready,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Authenticated,
                    kind: Some("oauth".to_string()),
                    label: Some("Claude Max".to_string()),
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: None,
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![
                    ServerProviderModel {
                        slug: "claude-sonnet-4-6".to_string(),
                        name: "Claude Sonnet 4.6".to_string(),
                        short_name: Some("Sonnet 4.6".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                    ServerProviderModel {
                        slug: "claude-haiku-4-5".to_string(),
                        name: "Claude Haiku 4.5".to_string(),
                        short_name: Some("Haiku 4.5".to_string()),
                        sub_provider: None,
                        is_custom: false,
                    },
                ],
                version_advisory: None,
            },
            ServerProvider {
                instance_id: "cursor".to_string(),
                driver: "cursor".to_string(),
                display_name: Some("Cursor".to_string()),
                accent_color: None,
                badge_label: None,
                continuation_group_key: None,
                show_interaction_mode_toggle: false,
                enabled: false,
                installed: false,
                version: None,
                status: ServerProviderState::Disabled,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Unknown,
                    kind: None,
                    label: None,
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: Some("Cursor CLI not detected on PATH.".to_string()),
                availability: ServerProviderAvailability::Unavailable,
                unavailable_reason: Some("Driver unavailable in this build.".to_string()),
                models: vec![ServerProviderModel {
                    slug: "composer-2".to_string(),
                    name: "Composer 2".to_string(),
                    short_name: Some("Composer".to_string()),
                    sub_provider: None,
                    is_custom: false,
                }],
                version_advisory: None,
            },
            ServerProvider {
                instance_id: "opencode".to_string(),
                driver: "opencode".to_string(),
                display_name: Some("OpenCode".to_string()),
                accent_color: None,
                badge_label: Some("Preview".to_string()),
                continuation_group_key: None,
                show_interaction_mode_toggle: false,
                enabled: true,
                installed: true,
                version: Some("0.8.1".to_string()),
                status: ServerProviderState::Warning,
                auth: ServerProviderAuth {
                    status: ServerProviderAuthStatus::Unknown,
                    kind: None,
                    label: None,
                    email: None,
                },
                checked_at: "2026-03-04T12:00:00.000Z".to_string(),
                message: Some("Server could not verify OpenCode authentication.".to_string()),
                availability: ServerProviderAvailability::Available,
                unavailable_reason: None,
                models: vec![ServerProviderModel {
                    slug: "openai/gpt-5".to_string(),
                    name: "OpenAI GPT-5".to_string(),
                    short_name: Some("GPT-5".to_string()),
                    sub_provider: Some("OpenAI".to_string()),
                    is_custom: false,
                }],
                version_advisory: None,
            },
        ]
    }

    fn reference_model_favorites() -> Vec<ProviderModelFavorite> {
        vec![
            ProviderModelFavorite {
                provider: "codex".to_string(),
                model: "gpt-5.4".to_string(),
            },
            ProviderModelFavorite {
                provider: "claudeAgent".to_string(),
                model: "claude-sonnet-4-6".to_string(),
            },
        ]
    }

    pub fn empty_reference_state() -> Self {
        Self {
            route: ChatRoute::Index,
            projects: Vec::new(),
            threads: Vec::new(),
            is_git_repo: false,
            available_environments: Vec::new(),
            vcs_refs: Vec::new(),
            current_git_branch: None,
            primary_environment_id: None,
            available_editors: Vec::new(),
            preferred_editor: None,
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            messages: Vec::new(),
            activities: Vec::new(),
            draft_sessions: Vec::new(),
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn draft_reference_state() -> Self {
        let draft_id = "draft-r3code-reference".to_string();
        let thread_ref = ScopedThreadRef::new("local", "thread-r3code-reference");
        let project_ref = ScopedProjectRef::new("local", "project-r3code");

        Self {
            route: ChatRoute::Thread(ThreadRouteTarget::Draft {
                draft_id: draft_id.clone(),
            }),
            projects: vec![ProjectSummary {
                id: "project-r3code".to_string(),
                environment_id: "local".to_string(),
                name: "server".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                scripts: Vec::new(),
            }],
            threads: Vec::new(),
            is_git_repo: false,
            available_environments: Self::reference_environments(),
            vcs_refs: Self::reference_vcs_refs(),
            current_git_branch: Some("main".to_string()),
            primary_environment_id: Some("local".to_string()),
            available_editors: vec![EditorId::VsCode, EditorId::FileManager],
            preferred_editor: Some(EditorId::VsCode),
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            messages: Vec::new(),
            activities: Vec::new(),
            draft_sessions: vec![DraftSessionState {
                draft_id,
                thread_ref,
                project_ref,
                logical_project_key: "local:project-r3code".to_string(),
                created_at: "2026-05-11T00:00:00.000Z".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Default,
                branch: None,
                worktree_path: None,
                env_mode: DraftThreadEnvMode::Local,
                promoted_to: None,
            }],
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn mock_reference_state() -> Self {
        Self {
            route: ChatRoute::Thread(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new("local", "thread-r3code-ui-shell"),
            }),
            projects: vec![ProjectSummary {
                id: "project-r3code".to_string(),
                environment_id: "local".to_string(),
                name: "r3code".to_string(),
                path: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                scripts: Self::reference_project_scripts(),
            }],
            is_git_repo: true,
            available_environments: Self::reference_environments(),
            vcs_refs: Self::reference_vcs_refs(),
            current_git_branch: Some("main".to_string()),
            primary_environment_id: Some("local".to_string()),
            available_editors: vec![EditorId::VsCode, EditorId::FileManager],
            preferred_editor: Some(EditorId::VsCode),
            providers: Self::reference_providers(),
            selected_provider_instance_id: "codex".to_string(),
            selected_model: DEFAULT_GIT_TEXT_GENERATION_MODEL.to_string(),
            model_favorites: Self::reference_model_favorites(),
            threads: vec![
                ThreadSummary {
                    id: "thread-r3code-ui-shell".to_string(),
                    environment_id: "local".to_string(),
                    project_id: "project-r3code".to_string(),
                    title: "Port R3Code UI shell".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Running,
                    created_at: "2026-03-04T11:59:00.000Z".to_string(),
                    updated_at: "2026-03-04T12:00:12.000Z".to_string(),
                    archived_at: None,
                    latest_user_message_at: Some("2026-03-04T12:00:09.000Z".to_string()),
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
                    branch: Some("main".to_string()),
                    worktree_path: None,
                },
                ThreadSummary {
                    id: "thread-visual-references".to_string(),
                    environment_id: "local".to_string(),
                    project_id: "project-r3code".to_string(),
                    title: "Capture visual references".to_string(),
                    project_name: "r3code".to_string(),
                    status: ThreadStatus::Idle,
                    created_at: "2026-03-03T14:12:00.000Z".to_string(),
                    updated_at: "2026-03-03T14:32:00.000Z".to_string(),
                    archived_at: None,
                    latest_user_message_at: None,
                    has_pending_approvals: false,
                    has_pending_user_input: false,
                    has_actionable_proposed_plan: false,
                    branch: Some("feature/parity-branch-toolbar".to_string()),
                    worktree_path: Some(
                        "C:\\Users\\bunny\\Downloads\\r3code\\.t3\\worktrees\\branch-toolbar"
                            .to_string(),
                    ),
                },
            ],
            messages: vec![
                ChatMessage::user(
                    "msg-user-r3code-ui-shell",
                    "Make the Rust port match the original UI exactly.",
                    "2026-03-04T12:00:09.000Z",
                ),
                ChatMessage::assistant(
                    "msg-assistant-r3code-ui-shell",
                    "Building a static GPUI shell first, then replacing mock data with Rust state.",
                    "2026-03-04T12:00:12.000Z",
                ),
            ],
            activities: Vec::new(),
            draft_sessions: Vec::new(),
            pending_approvals: Vec::new(),
            pending_user_inputs: Vec::new(),
            pending_user_input_draft_answers: BTreeMap::new(),
            active_pending_user_input_question_index: 0,
            responding_request_ids: Vec::new(),
            terminal_state: create_default_thread_terminal_state(),
            terminal_launch_context: None,
            terminal_event_entries: Vec::new(),
            diff_route: DiffRouteSearch::default(),
            turn_diff_summaries: Vec::new(),
        }
    }

    pub fn active_chat_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        snapshot.turn_diff_summaries = reference_turn_diff_summaries();
        snapshot
    }

    pub fn branch_toolbar_reference_state() -> Self {
        let mut snapshot = Self::draft_reference_state();
        snapshot.is_git_repo = true;
        if let Some(draft) = snapshot.draft_sessions.first_mut() {
            draft.env_mode = DraftThreadEnvMode::Worktree;
            draft.branch = None;
            draft.worktree_path = None;
        }
        snapshot
    }

    pub fn running_turn_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::Running;
        }
        snapshot.messages = vec![ChatMessage::user(
            "msg-user-running-turn",
            "Run the parity harness and fix any failures.",
            "2026-03-04T12:10:00.000Z",
        )];
        snapshot.activities = vec![
            ThreadActivity {
                id: "activity-thinking".to_string(),
                kind: "task.progress".to_string(),
                summary: "Inspecting changed surfaces".to_string(),
                tone: ActivityTone::Thinking,
                payload: ActivityPayload {
                    detail: Some("Reading upstream MessagesTimeline work log behavior".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(1),
                created_at: "2026-03-04T12:10:02.000Z".to_string(),
            },
            ThreadActivity {
                id: "activity-command".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Ran command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    command: Some("cargo test --workspace".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-run-tests".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(2),
                created_at: "2026-03-04T12:10:08.000Z".to_string(),
            },
            ThreadActivity {
                id: "activity-files".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Edited files".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    changed_files: vec![
                        "crates/r3_core/src/lib.rs".to_string(),
                        "crates/r3_ui/src/shell.rs".to_string(),
                    ],
                    title: Some("file change".to_string()),
                    item_type: Some("file_change".to_string()),
                    tool_call_id: Some("tool-edit-files".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-running-1".to_string()),
                sequence: Some(3),
                created_at: "2026-03-04T12:10:14.000Z".to_string(),
            },
        ];
        snapshot
    }

    pub fn pending_approval_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::NeedsInput;
            thread.has_pending_approvals = true;
        }
        snapshot.pending_approvals = vec![
            PendingApproval {
                request_id: "approval-command-run-tests".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-03-04T12:00:20.000Z".to_string(),
                detail: Some("cargo test --workspace".to_string()),
            },
            PendingApproval {
                request_id: "approval-file-change".to_string(),
                request_kind: ApprovalRequestKind::FileChange,
                created_at: "2026-03-04T12:00:23.000Z".to_string(),
                detail: Some("Allow editing crates/r3_ui/src/shell.rs".to_string()),
            },
        ];
        snapshot
    }

    pub fn pending_user_input_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        if let Some(thread) = snapshot.threads.first_mut() {
            thread.status = ThreadStatus::NeedsInput;
            thread.has_pending_user_input = true;
        }
        snapshot.pending_user_inputs = vec![PendingUserInput {
            request_id: "user-input-port-scope".to_string(),
            created_at: "2026-03-04T12:00:24.000Z".to_string(),
            questions: vec![
                UserInputQuestion {
                    id: "surface".to_string(),
                    header: "Surface".to_string(),
                    question: "Which surface should the Rust port match first?".to_string(),
                    options: vec![
                        UserInputQuestionOption {
                            label: "Composer".to_string(),
                            description: "Pending approval and user input states".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Terminal".to_string(),
                            description: "Drawer and command session state".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Diff".to_string(),
                            description: "Changed files and line review".to_string(),
                        },
                    ],
                    multi_select: false,
                },
                UserInputQuestion {
                    id: "coverage".to_string(),
                    header: "Coverage".to_string(),
                    question: "Select every state this parity pass should capture.".to_string(),
                    options: vec![
                        UserInputQuestionOption {
                            label: "Light".to_string(),
                            description: "Light theme".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Dark".to_string(),
                            description: "Dark theme".to_string(),
                        },
                        UserInputQuestionOption {
                            label: "Focused".to_string(),
                            description: "Composer focus state".to_string(),
                        },
                    ],
                    multi_select: true,
                },
            ],
        }];
        snapshot.pending_user_input_draft_answers = BTreeMap::from([(
            "surface".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Composer".to_string()],
                custom_answer: Some(String::new()),
            },
        )]);
        snapshot
    }

    pub fn terminal_drawer_reference_state() -> Self {
        let mut snapshot = Self::mock_reference_state();
        let thread_id = "thread-r3code-ui-shell".to_string();
        snapshot.terminal_state =
            split_thread_terminal(&create_default_thread_terminal_state(), "terminal-2");
        snapshot.terminal_launch_context = Some(ThreadTerminalLaunchContext {
            cwd: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
            worktree_path: None,
        });
        snapshot.terminal_event_entries = vec![
            TerminalEventEntry {
                id: 1,
                event: TerminalEvent::Started {
                    thread_id: thread_id.clone(),
                    terminal_id: "default".to_string(),
                    created_at: "2026-03-04T12:00:14.000Z".to_string(),
                    snapshot: TerminalSessionSnapshot {
                        thread_id: thread_id.clone(),
                        terminal_id: "default".to_string(),
                        cwd: "C:\\Users\\bunny\\Downloads\\r3code".to_string(),
                        worktree_path: None,
                        status: "running".to_string(),
                        pid: Some(24012),
                        history: String::new(),
                        exit_code: None,
                        exit_signal: None,
                        updated_at: "2026-03-04T12:00:14.000Z".to_string(),
                    },
                },
            },
            TerminalEventEntry {
                id: 2,
                event: TerminalEvent::Output {
                    thread_id: thread_id.clone(),
                    terminal_id: "default".to_string(),
                    created_at: "2026-03-04T12:00:15.000Z".to_string(),
                    data: "PS C:\\Users\\bunny\\Downloads\\r3code> cargo check --workspace\r\n"
                        .to_string(),
                },
            },
            TerminalEventEntry {
                id: 3,
                event: TerminalEvent::Activity {
                    thread_id: thread_id.clone(),
                    terminal_id: "terminal-2".to_string(),
                    created_at: "2026-03-04T12:00:16.000Z".to_string(),
                    has_running_subprocess: true,
                },
            },
            TerminalEventEntry {
                id: 4,
                event: TerminalEvent::Output {
                    thread_id,
                    terminal_id: "terminal-2".to_string(),
                    created_at: "2026-03-04T12:00:17.000Z".to_string(),
                    data: "Running upstream capture fixture...\r\n".to_string(),
                },
            },
        ];
        snapshot
    }

    pub fn diff_panel_reference_state() -> Self {
        let mut snapshot = Self::active_chat_reference_state();
        snapshot.diff_route = parse_diff_route_search(
            Some(DiffOpenValue::from("1")),
            Some("turn-r3code-ui-shell-2"),
            Some("crates/r3_ui/src/shell.rs"),
        );
        snapshot
    }

    pub fn renders_chat_view(&self) -> bool {
        self.route.renders_chat_view()
    }

    pub fn active_thread_summary(&self) -> Option<&ThreadSummary> {
        self.threads.first()
    }

    pub fn active_thread_title(&self) -> &str {
        self.active_thread_summary()
            .map(|thread| thread.title.as_str())
            .unwrap_or("New thread")
    }

    pub fn active_project_name(&self) -> Option<&str> {
        self.projects.first().map(|project| project.name.as_str())
    }

    pub fn active_project(&self) -> Option<&ProjectSummary> {
        self.projects.first()
    }

    pub fn active_environment_id(&self) -> Option<&str> {
        match &self.route {
            ChatRoute::Thread(ThreadRouteTarget::Server { thread_ref }) => {
                Some(thread_ref.environment_id.as_str())
            }
            ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) => self
                .draft_sessions
                .iter()
                .find(|draft| &draft.draft_id == draft_id)
                .map(|draft| draft.thread_ref.environment_id.as_str()),
            ChatRoute::Index => None,
        }
    }

    pub fn open_in_picker_visible(&self) -> bool {
        let Some(active_environment_id) = self.active_environment_id() else {
            return false;
        };
        should_show_open_in_picker(
            self.active_project_name(),
            active_environment_id,
            self.primary_environment_id.as_deref(),
        )
    }

    pub fn active_editor_option(&self, platform: &str) -> Option<EditorOption> {
        let options = resolve_editor_options(platform, &self.available_editors);
        self.preferred_editor
            .and_then(|preferred| {
                options
                    .iter()
                    .copied()
                    .find(|option| option.id == preferred)
            })
            .or_else(|| options.first().copied())
    }

    pub fn active_draft_session(&self) -> Option<&DraftSessionState> {
        match &self.route {
            ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) => self
                .draft_sessions
                .iter()
                .find(|draft| &draft.draft_id == draft_id),
            _ => None,
        }
    }

    pub fn active_thread_branch(&self) -> Option<&str> {
        self.active_draft_session()
            .and_then(|draft| draft.branch.as_deref())
            .or_else(|| {
                self.active_thread_summary()
                    .and_then(|thread| thread.branch.as_deref())
            })
    }

    pub fn active_worktree_path(&self) -> Option<&str> {
        self.active_draft_session()
            .and_then(|draft| draft.worktree_path.as_deref())
            .or_else(|| {
                self.active_thread_summary()
                    .and_then(|thread| thread.worktree_path.as_deref())
            })
    }

    pub fn active_branch_toolbar_state(&self) -> Option<BranchToolbarState> {
        if !self.renders_chat_view() || !self.is_git_repo {
            return None;
        }

        let environment = self
            .available_environments
            .first()
            .cloned()
            .unwrap_or_else(|| BranchToolbarEnvironmentOption {
                environment_id: "local".to_string(),
                project_id: "project-local".to_string(),
                label: "This device".to_string(),
                is_primary: true,
            });
        let active_worktree_path = self.active_worktree_path().map(str::to_string);
        let has_server_thread = matches!(
            self.route,
            ChatRoute::Thread(ThreadRouteTarget::Server { .. })
        ) && self.active_thread_summary().is_some();
        let draft_thread_env_mode = self.active_draft_session().map(|draft| draft.env_mode);
        let effective_env_mode = resolve_effective_env_mode(
            active_worktree_path.as_deref(),
            has_server_thread,
            draft_thread_env_mode,
        );
        let resolved_active_branch = resolve_branch_toolbar_value(
            effective_env_mode,
            active_worktree_path.as_deref(),
            self.active_thread_branch(),
            self.current_git_branch.as_deref(),
        );
        let env_mode_locked = has_server_thread && active_worktree_path.is_some();
        let workspace_label = if env_mode_locked {
            resolve_locked_workspace_label(active_worktree_path.as_deref())
        } else if effective_env_mode == DraftThreadEnvMode::Worktree {
            resolve_env_mode_label(DraftThreadEnvMode::Worktree)
        } else {
            resolve_current_workspace_label(active_worktree_path.as_deref())
        };
        let branch_label = branch_toolbar_trigger_label(
            active_worktree_path.as_deref(),
            effective_env_mode,
            resolved_active_branch.as_deref(),
        );

        Some(BranchToolbarState {
            environment_id: environment.environment_id,
            environment_label: environment.label,
            environment_is_primary: environment.is_primary,
            show_environment_picker: self.available_environments.len() > 1,
            effective_env_mode,
            env_locked: false,
            env_mode_locked,
            active_worktree_path,
            workspace_label,
            branch_label,
            resolved_active_branch,
        })
    }

    pub fn set_active_draft_env_mode(&mut self, mode: DraftThreadEnvMode) {
        let ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) = &self.route else {
            return;
        };
        if let Some(draft) = self
            .draft_sessions
            .iter_mut()
            .find(|draft| &draft.draft_id == draft_id)
        {
            draft.env_mode = mode;
            if mode == DraftThreadEnvMode::Worktree && draft.branch.is_none() {
                draft.branch = self.current_git_branch.clone();
            }
        }
    }

    pub fn select_branch_for_active_thread(&mut self, branch: impl Into<String>) {
        let branch = branch.into();
        if let Some(ref_name) = self
            .vcs_refs
            .iter()
            .find(|ref_name| ref_name.name == branch)
        {
            self.current_git_branch = Some(if ref_name.is_remote {
                derive_local_branch_name_from_remote_ref(&ref_name.name)
            } else {
                ref_name.name.clone()
            });
            if let ChatRoute::Thread(ThreadRouteTarget::Draft { draft_id }) = &self.route {
                if let Some(draft) = self
                    .draft_sessions
                    .iter_mut()
                    .find(|draft| &draft.draft_id == draft_id)
                {
                    let next_env_mode = resolve_draft_env_mode_after_branch_change(
                        ref_name.worktree_path.as_deref(),
                        draft.worktree_path.as_deref(),
                        draft.env_mode,
                    );
                    draft.branch = self.current_git_branch.clone();
                    draft.worktree_path = ref_name.worktree_path.clone();
                    draft.env_mode = next_env_mode;
                }
            } else if let Some(thread) = self.threads.first_mut() {
                thread.branch = self.current_git_branch.clone();
                thread.worktree_path = ref_name.worktree_path.clone();
            }
        }
    }

    pub fn active_pending_approval(&self) -> Option<&PendingApproval> {
        self.pending_approvals.first()
    }

    pub fn active_pending_user_input(&self) -> Option<&PendingUserInput> {
        self.pending_user_inputs.first()
    }

    pub fn active_pending_user_input_progress(&self) -> Option<PendingUserInputProgress> {
        let prompt = self.active_pending_user_input()?;
        Some(derive_pending_user_input_progress(
            &prompt.questions,
            &self.pending_user_input_draft_answers,
            self.active_pending_user_input_question_index,
        ))
    }

    pub fn is_responding_to_request(&self, request_id: &str) -> bool {
        self.responding_request_ids
            .iter()
            .any(|responding_id| responding_id == request_id)
    }

    pub fn terminal_open(&self) -> bool {
        self.terminal_state.terminal_open
    }

    pub fn work_log_entries(&self) -> Vec<WorkLogEntry> {
        let latest_turn_id = self
            .activities
            .iter()
            .rev()
            .find_map(|activity| activity.turn_id.as_deref());
        derive_work_log_entries(&self.activities, latest_turn_id)
    }

    pub fn diff_open(&self) -> bool {
        self.diff_route.diff.as_deref() == Some("1")
    }

    pub fn ordered_turn_diff_summaries(&self) -> Vec<&TurnDiffSummary> {
        let mut summaries = self.turn_diff_summaries.iter().collect::<Vec<_>>();
        summaries.sort_by(|left, right| {
            right
                .checkpoint_turn_count
                .unwrap_or(0)
                .cmp(&left.checkpoint_turn_count.unwrap_or(0))
                .then_with(|| right.completed_at.cmp(&left.completed_at))
        });
        summaries
    }

    pub fn selected_turn_diff_summary(&self) -> Option<&TurnDiffSummary> {
        let selected_turn_id = self.diff_route.diff_turn_id.as_ref()?;
        self.turn_diff_summaries
            .iter()
            .find(|summary| &summary.turn_id == selected_turn_id)
            .or_else(|| self.ordered_turn_diff_summaries().first().copied())
    }

    pub fn selected_diff_file_path(&self) -> Option<&str> {
        self.diff_route
            .diff_turn_id
            .as_ref()
            .and(self.diff_route.diff_file_path.as_deref())
    }
}

fn reference_turn_diff_summaries() -> Vec<TurnDiffSummary> {
    vec![
        TurnDiffSummary {
            turn_id: "turn-r3code-ui-shell-2".to_string(),
            completed_at: "2026-03-04T12:05:18.000Z".to_string(),
            status: Some("completed".to_string()),
            files: vec![
                TurnDiffFileChange {
                    path: "crates/r3_ui/src/shell.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(126),
                    deletions: Some(18),
                },
                TurnDiffFileChange {
                    path: "crates/r3_core/src/lib.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(74),
                    deletions: Some(4),
                },
                TurnDiffFileChange {
                    path: "docs/reference/PARITY_PLAN.md".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(8),
                    deletions: Some(0),
                },
            ],
            checkpoint_ref: Some("checkpoint-turn-2".to_string()),
            assistant_message_id: Some("msg-assistant-r3code-ui-shell".to_string()),
            checkpoint_turn_count: Some(2),
        },
        TurnDiffSummary {
            turn_id: "turn-r3code-ui-shell-1".to_string(),
            completed_at: "2026-03-04T12:01:42.000Z".to_string(),
            status: Some("completed".to_string()),
            files: vec![
                TurnDiffFileChange {
                    path: "crates/r3_ui/assets/icons/diff.svg".to_string(),
                    kind: Some("added".to_string()),
                    additions: Some(1),
                    deletions: Some(0),
                },
                TurnDiffFileChange {
                    path: "crates/r3_ui/src/assets.rs".to_string(),
                    kind: Some("modified".to_string()),
                    additions: Some(6),
                    deletions: Some(1),
                },
            ],
            checkpoint_ref: Some("checkpoint-turn-1".to_string()),
            assistant_message_id: Some("msg-assistant-r3code-ui-shell".to_string()),
            checkpoint_turn_count: Some(1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command_palette_project() -> ProjectSummary {
        ProjectSummary {
            id: "project-1".to_string(),
            environment_id: "environment-local".to_string(),
            name: "Project".to_string(),
            path: "/repo/project".to_string(),
            scripts: Vec::new(),
        }
    }

    fn make_command_palette_thread(
        id: &str,
        title: &str,
        created_at: &str,
        updated_at: &str,
    ) -> ThreadSummary {
        ThreadSummary {
            id: id.to_string(),
            environment_id: "environment-local".to_string(),
            project_id: "project-1".to_string(),
            title: title.to_string(),
            project_name: "Project".to_string(),
            status: ThreadStatus::Idle,
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
            archived_at: None,
            latest_user_message_at: None,
            has_pending_approvals: false,
            has_pending_user_input: false,
            has_actionable_proposed_plan: false,
            branch: None,
            worktree_path: None,
        }
    }

    fn make_server_provider(overrides: impl FnOnce(&mut ServerProvider)) -> ServerProvider {
        let mut provider = ServerProvider {
            instance_id: "codex".to_string(),
            driver: "codex".to_string(),
            display_name: Some("Codex".to_string()),
            accent_color: None,
            badge_label: None,
            continuation_group_key: None,
            show_interaction_mode_toggle: true,
            enabled: true,
            installed: true,
            version: Some("1.2.3".to_string()),
            status: ServerProviderState::Ready,
            auth: ServerProviderAuth {
                status: ServerProviderAuthStatus::Unknown,
                kind: None,
                label: None,
                email: None,
            },
            checked_at: "2026-03-04T12:00:00.000Z".to_string(),
            message: None,
            availability: ServerProviderAvailability::Available,
            unavailable_reason: None,
            models: Vec::new(),
            version_advisory: None,
        };
        overrides(&mut provider);
        provider
    }

    fn advertised_endpoint(
        id: &str,
        http_base_url: &str,
        reachability: &str,
        is_default: bool,
        status: AdvertisedEndpointStatus,
        hosted_https_app: HostedHttpsAppCompatibility,
    ) -> AdvertisedEndpoint {
        AdvertisedEndpoint {
            id: id.to_string(),
            provider_id: "desktop-core".to_string(),
            label: "Local network".to_string(),
            http_base_url: http_base_url.to_string(),
            reachability: reachability.to_string(),
            status,
            is_default,
            hosted_https_app,
        }
    }

    #[test]
    fn parses_manual_desktop_ssh_targets_like_upstream() {
        let target = parse_manual_desktop_ssh_target("alice@example.com:2222", "", "").unwrap();
        assert_eq!(
            target,
            DesktopSshEnvironmentTarget {
                alias: "example.com".to_string(),
                hostname: "example.com".to_string(),
                username: Some("alice".to_string()),
                port: Some(2222),
            }
        );
        assert_eq!(format_desktop_ssh_target(&target), "alice@example.com:2222");

        let explicit_username =
            parse_manual_desktop_ssh_target("alice@example.com", "root", "").unwrap();
        assert_eq!(explicit_username.username.as_deref(), Some("root"));

        let ipv6 = parse_manual_desktop_ssh_target("bob@[fe80::1]:2200", "", "").unwrap();
        assert_eq!(ipv6.hostname, "fe80::1");
        assert_eq!(ipv6.username.as_deref(), Some("bob"));
        assert_eq!(ipv6.port, Some(2200));
        assert_eq!(format_desktop_ssh_target(&ipv6), "bob@fe80::1:2200");

        let explicit_port_parse_int =
            parse_manual_desktop_ssh_target("devbox", "", "22abc").unwrap();
        assert_eq!(explicit_port_parse_int.port, Some(22));
    }

    #[test]
    fn rejects_manual_desktop_ssh_targets_with_upstream_messages() {
        assert_eq!(
            parse_manual_desktop_ssh_target("  ", "", "").unwrap_err(),
            "SSH host or alias is required."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("alice@:22", "", "").unwrap_err(),
            "SSH host or alias is required."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("example.com:70000", "", "").unwrap_err(),
            "SSH port must be between 1 and 65535."
        );
        assert_eq!(
            parse_manual_desktop_ssh_target("example.com", "", "nope").unwrap_err(),
            "SSH port must be between 1 and 65535."
        );
    }

    #[test]
    fn parses_remote_pairing_fields_from_urls_and_manual_fields() {
        assert_eq!(
            parse_remote_pairing_fields("https://remote.example.com/pair#token=pairing-token", "")
                .unwrap(),
            RemotePairingFields {
                host: "https://remote.example.com".to_string(),
                pairing_code: "pairing-token".to_string(),
            }
        );
        assert_eq!(
            parse_remote_pairing_fields(
                "https://app.t3.codes/pair?host=https%3A%2F%2Fdesktop.tailnet.ts.net%3A44342%2F#token=pairing-token",
                "",
            )
            .unwrap(),
            RemotePairingFields {
                host: "https://desktop.tailnet.ts.net:44342/".to_string(),
                pairing_code: "pairing-token".to_string(),
            }
        );
        assert_eq!(
            parse_remote_pairing_fields("backend.example.com", "PAIRCODE").unwrap(),
            RemotePairingFields {
                host: "backend.example.com".to_string(),
                pairing_code: "PAIRCODE".to_string(),
            }
        );

        assert_eq!(
            parse_remote_pairing_fields("", "PAIRCODE").unwrap_err(),
            "Enter a backend host."
        );
        assert_eq!(
            parse_remote_pairing_fields("backend.example.com", "").unwrap_err(),
            "Enter a pairing code."
        );
    }

    #[test]
    fn formats_desktop_ssh_connection_errors_like_upstream() {
        assert_eq!(
            format_desktop_ssh_connection_error(Some(
                "Error invoking remote method 'desktop:ensure-ssh-environment': SshConnectionError: bad host"
            )),
            "bad host"
        );
        assert_eq!(
            format_desktop_ssh_connection_error(Some("SshLaunchError: timed out")),
            "timed out"
        );
        assert_eq!(
            format_desktop_ssh_connection_error(Some("   ")),
            "Failed to connect SSH host."
        );
        assert_eq!(
            format_desktop_ssh_connection_error(None),
            "Failed to connect SSH host."
        );
    }

    #[test]
    fn selects_and_resolves_advertised_pairing_endpoints() {
        let loopback = advertised_endpoint(
            "desktop-loopback:127.0.0.1",
            "http://127.0.0.1:8765",
            "loopback",
            true,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Incompatible,
        );
        let lan = advertised_endpoint(
            "desktop-lan:192.168.1.44",
            "http://192.168.1.44:8765",
            "lan",
            false,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Incompatible,
        );
        let tailscale_https = advertised_endpoint(
            "tailscale-magicdns:desktop.tailnet.ts.net",
            "https://desktop.tailnet.ts.net:8765",
            "tailscale",
            false,
            AdvertisedEndpointStatus::Available,
            HostedHttpsAppCompatibility::Compatible,
        );
        let unavailable_preference = advertised_endpoint(
            "desktop-lan:stale",
            "http://stale.local:8765",
            "lan",
            false,
            AdvertisedEndpointStatus::Unavailable,
            HostedHttpsAppCompatibility::Incompatible,
        );

        assert_eq!(
            endpoint_default_preference_key(&loopback),
            "desktop-core:loopback:http"
        );
        assert_eq!(
            endpoint_default_preference_key(&tailscale_https),
            "tailscale:magicdns:https"
        );

        let endpoints = vec![
            unavailable_preference,
            loopback.clone(),
            lan.clone(),
            tailscale_https.clone(),
        ];
        assert_eq!(
            select_pairing_endpoint(&endpoints, Some("desktop-core:lan:http"))
                .unwrap()
                .id,
            lan.id
        );
        assert_eq!(
            resolve_advertised_endpoint_pairing_url(&lan, "PAIRCODE").unwrap(),
            "http://192.168.1.44:8765/pair#token=PAIRCODE"
        );
        assert_eq!(
            resolve_advertised_endpoint_pairing_url(&tailscale_https, "PAIRCODE").unwrap(),
            "https://app.t3.codes/pair?host=https%3A%2F%2Fdesktop.tailnet.ts.net%3A8765#token=PAIRCODE"
        );
    }

    #[test]
    fn sorts_and_upserts_access_records_like_upstream() {
        let old_link = ServerPairingLinkRecord {
            id: "old".to_string(),
            created_at: "2026-03-01T00:00:00.000Z".to_string(),
        };
        let new_link = ServerPairingLinkRecord {
            id: "new".to_string(),
            created_at: "2026-03-02T00:00:00.000Z".to_string(),
        };
        assert_eq!(
            sort_desktop_pairing_links(&[old_link.clone(), new_link.clone()])
                .into_iter()
                .map(|link| link.id)
                .collect::<Vec<_>>(),
            vec!["new", "old"]
        );
        assert_eq!(
            upsert_desktop_pairing_link(&[old_link], new_link)
                .into_iter()
                .map(|link| link.id)
                .collect::<Vec<_>>(),
            vec!["new", "old"]
        );

        let disconnected_current = ServerClientSessionRecord {
            session_id: "current".to_string(),
            issued_at: "2026-03-01T00:00:00.000Z".to_string(),
            current: true,
            connected: false,
        };
        let connected_other = ServerClientSessionRecord {
            session_id: "other".to_string(),
            issued_at: "2026-03-03T00:00:00.000Z".to_string(),
            current: false,
            connected: true,
        };
        assert_eq!(
            sort_desktop_client_sessions(&[connected_other, disconnected_current])
                .into_iter()
                .map(|session| session.session_id)
                .collect::<Vec<_>>(),
            vec!["current", "other"]
        );
    }

    #[test]
    fn formats_diagnostics_helpers_like_upstream() {
        assert_eq!(format_diagnostics_count(1234567), "1,234,567");
        assert_eq!(format_diagnostics_duration_ms(999.4), "999 ms");
        assert_eq!(format_diagnostics_duration_ms(1500.0), "1.50 s");
        assert_eq!(format_diagnostics_duration_ms(10_000.0), "10.0 s");
        assert_eq!(format_diagnostics_bytes(1023), "1023 B");
        assert_eq!(format_diagnostics_bytes(1536), "1.50 KB");
        assert_eq!(format_diagnostics_bytes(12 * 1024), "12.0 KB");
        assert_eq!(
            shorten_trace_id("0123456789abcdef0123456789abcdef0123456789"),
            "0123456789abcdef01...0123456789"
        );
        assert!(is_stale_process_signal_message(Some(
            "process is not a live descendant"
        )));
        assert!(!is_stale_process_signal_message(None));
    }

    #[test]
    fn formats_diagnostics_settings_description_like_upstream() {
        assert_eq!(
            collapse_otel_signals_url(
                "http://localhost:4318/v1/traces",
                "http://localhost:4318/v1/metrics",
            )
            .as_deref(),
            Some("http://localhost:4318/v1/{traces,metrics}")
        );
        assert_eq!(
            format_diagnostics_description(DiagnosticsDescriptionInput {
                local_tracing_enabled: true,
                otlp_traces_enabled: true,
                otlp_traces_url: Some("http://localhost:4318/v1/traces"),
                otlp_metrics_enabled: true,
                otlp_metrics_url: Some("http://localhost:4318/v1/metrics"),
            }),
            "Local trace file. Exporting OTEL to http://localhost:4318/v1/{traces,metrics}."
        );
        assert_eq!(
            format_diagnostics_description(DiagnosticsDescriptionInput {
                local_tracing_enabled: false,
                otlp_traces_enabled: false,
                otlp_traces_url: Some("http://localhost:4318/v1/traces"),
                otlp_metrics_enabled: false,
                otlp_metrics_url: None,
            }),
            "Terminal logs only."
        );
    }

    fn diagnostics_row(
        pid: u32,
        ppid: u32,
        pgid: i32,
        cpu_percent: f64,
        rss_bytes: u64,
        elapsed: &str,
        command: &str,
    ) -> ProcessDiagnosticsRow {
        ProcessDiagnosticsRow {
            pid,
            ppid,
            pgid: Some(pgid),
            status: "S".to_string(),
            cpu_percent,
            rss_bytes,
            elapsed: elapsed.to_string(),
            command: command.to_string(),
        }
    }

    #[test]
    fn parses_posix_process_rows_with_full_commands() {
        let rows = parse_posix_process_rows(
            &[
                "  10     1    10 Ss      0.0   1024   01:02.03 /usr/bin/node server.js",
                "  11    10    10 S+     12.5  20480      00:04 codex app-server --config /tmp/one two",
            ]
            .join("\n"),
        );

        assert_eq!(
            rows,
            vec![
                ProcessDiagnosticsRow {
                    pid: 10,
                    ppid: 1,
                    pgid: Some(10),
                    status: "Ss".to_string(),
                    cpu_percent: 0.0,
                    rss_bytes: 1024 * 1024,
                    elapsed: "01:02.03".to_string(),
                    command: "/usr/bin/node server.js".to_string(),
                },
                ProcessDiagnosticsRow {
                    pid: 11,
                    ppid: 10,
                    pgid: Some(10),
                    status: "S+".to_string(),
                    cpu_percent: 12.5,
                    rss_bytes: 20480 * 1024,
                    elapsed: "00:04".to_string(),
                    command: "codex app-server --config /tmp/one two".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_windows_process_rows_from_powershell_json() {
        let rows = parse_windows_process_rows(
            r#"[
                {
                    "ProcessId": 4242,
                    "ParentProcessId": 100,
                    "Name": "agent.exe",
                    "CommandLine": "codex app-server --config C:\\tmp\\one two",
                    "Status": "",
                    "WorkingSetSize": 1536.6,
                    "PercentProcessorTime": -3
                },
                {
                    "ProcessId": 4243,
                    "ParentProcessId": 4242,
                    "Name": "git.exe",
                    "CommandLine": "   ",
                    "WorkingSetSize": -10,
                    "PercentProcessorTime": 2.5
                },
                { "ProcessId": 0, "ParentProcessId": 1, "Name": "bad.exe" }
            ]"#,
        );

        assert_eq!(
            rows,
            vec![
                ProcessDiagnosticsRow {
                    pid: 4242,
                    ppid: 100,
                    pgid: None,
                    status: "Live".to_string(),
                    cpu_percent: 0.0,
                    rss_bytes: 1537,
                    elapsed: String::new(),
                    command: "codex app-server --config C:\\tmp\\one two".to_string(),
                },
                ProcessDiagnosticsRow {
                    pid: 4243,
                    ppid: 4242,
                    pgid: None,
                    status: "Live".to_string(),
                    cpu_percent: 2.5,
                    rss_bytes: 0,
                    elapsed: String::new(),
                    command: "git.exe".to_string(),
                },
            ]
        );
        assert!(parse_windows_process_rows("").is_empty());
        assert!(parse_windows_process_rows("not json").is_empty());
    }

    #[test]
    fn aggregates_only_descendants_of_the_server_process() {
        let diagnostics = aggregate_process_diagnostics(
            100,
            &[
                diagnostics_row(100, 1, 100, 0.0, 1_000, "01:00", "t3 server"),
                diagnostics_row(101, 100, 100, 1.5, 2_000, "00:20", "codex app-server"),
                diagnostics_row(102, 101, 100, 3.25, 4_000, "00:05", "git status"),
                diagnostics_row(200, 1, 200, 99.0, 8_000, "00:01", "unrelated"),
                diagnostics_row(
                    201,
                    100,
                    100,
                    9.0,
                    9_000,
                    "00:00",
                    "ps -axo pid=,ppid=,pgid=,stat=,pcpu=,rss=,etime=,command=",
                ),
            ],
            "2026-05-05T10:00:00.000Z",
        );

        assert_eq!(diagnostics.server_pid, 100);
        assert_eq!(diagnostics.read_at, "2026-05-05T10:00:00.000Z");
        assert_eq!(diagnostics.process_count, 2);
        assert_eq!(diagnostics.total_rss_bytes, 6_000);
        assert_eq!(diagnostics.total_cpu_percent, 4.75);
        assert_eq!(
            diagnostics
                .processes
                .iter()
                .map(|process| process.pid)
                .collect::<Vec<_>>(),
            vec![101, 102]
        );
        assert_eq!(
            diagnostics
                .processes
                .iter()
                .map(|process| process.depth)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(diagnostics.processes[0].pgid, Some(100));
        assert_eq!(diagnostics.processes[0].child_pids, vec![102]);
    }

    #[test]
    fn preserves_ascending_sibling_order_for_nested_descendants() {
        let diagnostics = aggregate_process_diagnostics(
            100,
            &[
                diagnostics_row(101, 100, 100, 0.0, 100, "00:10", "agent"),
                diagnostics_row(103, 101, 100, 0.0, 100, "00:10", "child-b"),
                diagnostics_row(102, 101, 100, 0.0, 100, "00:10", "child-a"),
            ],
            "2026-05-05T10:00:00.000Z",
        );

        assert_eq!(
            diagnostics
                .processes
                .iter()
                .map(|process| process.pid)
                .collect::<Vec<_>>(),
            vec![101, 102, 103]
        );
    }

    #[test]
    fn filters_diagnostics_query_processes_before_signaling_checks() {
        let ps_query = diagnostics_row(
            4242,
            100,
            100,
            1.5,
            2048,
            "00:00",
            "ps -axo pid=,ppid=,pgid=,stat=,pcpu=,rss=,etime=,command=",
        );
        let powershell_query = ProcessDiagnosticsRow {
            pid: 4243,
            ppid: 100,
            pgid: None,
            status: "Live".to_string(),
            cpu_percent: 1.0,
            rss_bytes: 2048,
            elapsed: String::new(),
            command: "powershell.exe -NoProfile Get-CimInstance Win32_Process".to_string(),
        };

        assert!(is_diagnostics_query_process(&ps_query, 100));
        assert!(is_diagnostics_query_process(&powershell_query, 100));
        assert_eq!(
            aggregate_process_diagnostics(
                100,
                &[ps_query, powershell_query],
                "2026-05-05T10:00:00.000Z",
            )
            .process_count,
            0
        );
    }

    fn ns(ms: i64) -> String {
        (ms as i128 * 1_000_000).to_string()
    }

    fn trace_record(
        name: &str,
        trace_id: &str,
        span_id: &str,
        start_ms: i64,
        duration_ms: f64,
        exit: Option<serde_json::Value>,
        events: Vec<serde_json::Value>,
    ) -> String {
        serde_json::json!({
            "type": "effect-span",
            "name": name,
            "traceId": trace_id,
            "spanId": span_id,
            "sampled": true,
            "kind": "internal",
            "startTimeUnixNano": ns(start_ms),
            "endTimeUnixNano": ns(start_ms + duration_ms as i64),
            "durationMs": duration_ms,
            "attributes": {},
            "events": events,
            "links": [],
            "exit": exit.unwrap_or_else(|| serde_json::json!({ "_tag": "Success" })),
        })
        .to_string()
    }

    #[test]
    fn aggregates_trace_failures_slow_spans_logs_and_parse_errors() {
        let diagnostics = aggregate_trace_diagnostics(TraceDiagnosticsInput {
            trace_file_path: "/tmp/server.trace.ndjson",
            read_at: "2026-05-05T10:00:00.000Z",
            slow_span_threshold_ms: Some(1_000.0),
            scanned_file_paths: None,
            error: None,
            partial_failure: false,
            files: &[
                TraceDiagnosticsFile {
                    path: "/tmp/server.trace.ndjson.1".to_string(),
                    text: [
                        trace_record(
                            "server.getConfig",
                            "trace-a",
                            "span-a",
                            1_000,
                            50.0,
                            None,
                            Vec::new(),
                        ),
                        "not-json".to_string(),
                    ]
                    .join("\n"),
                },
                TraceDiagnosticsFile {
                    path: "/tmp/server.trace.ndjson".to_string(),
                    text: [
                        trace_record(
                            "orchestration.dispatch",
                            "trace-b",
                            "span-b",
                            2_000,
                            1_500.0,
                            Some(serde_json::json!({
                                "_tag": "Failure",
                                "cause": "Provider crashed"
                            })),
                            vec![serde_json::json!({
                                "name": "provider failed",
                                "timeUnixNano": ns(3_400),
                                "attributes": { "effect.logLevel": "Error" }
                            })],
                        ),
                        trace_record(
                            "orchestration.dispatch",
                            "trace-c",
                            "span-c",
                            4_000,
                            250.0,
                            Some(serde_json::json!({
                                "_tag": "Failure",
                                "cause": "Provider crashed"
                            })),
                            Vec::new(),
                        ),
                        trace_record(
                            "git.status",
                            "trace-d",
                            "span-d",
                            5_000,
                            25.0,
                            Some(serde_json::json!({
                                "_tag": "Interrupted",
                                "cause": "Interrupted"
                            })),
                            vec![serde_json::json!({
                                "name": "status delayed",
                                "timeUnixNano": ns(5_010),
                                "attributes": { "effect.logLevel": "Warning" }
                            })],
                        ),
                    ]
                    .join("\n"),
                },
            ],
        });

        assert_eq!(diagnostics.record_count, 4);
        assert_eq!(diagnostics.read_at, "2026-05-05T10:00:00.000Z");
        assert_eq!(
            diagnostics.first_span_at.as_deref(),
            Some("1970-01-01T00:00:01.000Z")
        );
        assert_eq!(
            diagnostics.last_span_at.as_deref(),
            Some("1970-01-01T00:00:05.025Z")
        );
        assert_eq!(diagnostics.parse_error_count, 1);
        assert_eq!(diagnostics.failure_count, 2);
        assert_eq!(diagnostics.interruption_count, 1);
        assert_eq!(diagnostics.slow_span_count, 1);
        assert_eq!(diagnostics.log_level_counts.get("Error"), Some(&1));
        assert_eq!(diagnostics.log_level_counts.get("Warning"), Some(&1));
        assert_eq!(
            diagnostics.common_failures[0].name,
            "orchestration.dispatch"
        );
        assert_eq!(diagnostics.common_failures[0].count, 2);
        assert_eq!(diagnostics.latest_failures[0].trace_id, "trace-c");
        assert_eq!(diagnostics.slowest_spans[0].trace_id, "trace-b");
        assert_eq!(
            diagnostics.latest_warning_and_error_logs[0].message,
            "status delayed"
        );
        assert_eq!(
            diagnostics.top_spans_by_count[0].name,
            "orchestration.dispatch"
        );
    }

    #[test]
    fn returns_trace_not_found_diagnostic_when_no_files_are_available() {
        let diagnostics = aggregate_trace_diagnostics(TraceDiagnosticsInput {
            trace_file_path: "/tmp/missing.trace.ndjson",
            read_at: "2026-05-05T10:00:00.000Z",
            files: &[],
            scanned_file_paths: None,
            slow_span_threshold_ms: None,
            error: None,
            partial_failure: false,
        });

        assert_eq!(diagnostics.record_count, 0);
        assert_eq!(
            diagnostics.error.as_ref().map(|error| error.kind.as_str()),
            Some("trace-file-not-found")
        );
    }

    #[test]
    fn preserves_full_trace_failure_causes_and_log_messages() {
        let long_cause = format!("VcsProcessSpawnError: {}", "missing executable ".repeat(80))
            .trim()
            .to_string();
        let long_message = format!("provider warning: {}", "retrying command ".repeat(80))
            .trim()
            .to_string();
        let diagnostics = aggregate_trace_diagnostics(TraceDiagnosticsInput {
            trace_file_path: "/tmp/server.trace.ndjson",
            read_at: "2026-05-05T10:00:00.000Z",
            scanned_file_paths: None,
            slow_span_threshold_ms: None,
            error: None,
            partial_failure: false,
            files: &[TraceDiagnosticsFile {
                path: "/tmp/server.trace.ndjson".to_string(),
                text: trace_record(
                    "VcsProcess.run",
                    "trace-long",
                    "span-long",
                    1_000,
                    25.0,
                    Some(serde_json::json!({
                        "_tag": "Failure",
                        "cause": long_cause
                    })),
                    vec![serde_json::json!({
                        "name": long_message,
                        "timeUnixNano": ns(1_010),
                        "attributes": { "effect.logLevel": "Warning" }
                    })],
                ),
            }],
        });

        assert_eq!(diagnostics.latest_failures[0].cause, long_cause);
        assert_eq!(diagnostics.common_failures[0].cause, long_cause);
        assert_eq!(
            diagnostics.latest_warning_and_error_logs[0].message,
            long_message
        );
    }

    #[test]
    fn keeps_trace_partial_failure_metadata_with_loaded_data() {
        let diagnostics = aggregate_trace_diagnostics(TraceDiagnosticsInput {
            trace_file_path: "/tmp/server.trace.ndjson",
            read_at: "2026-05-05T10:00:00.000Z",
            scanned_file_paths: Some(vec![
                "/tmp/server.trace.ndjson.1".to_string(),
                "/tmp/server.trace.ndjson".to_string(),
            ]),
            slow_span_threshold_ms: None,
            error: Some(TraceDiagnosticsErrorSummary {
                kind: "trace-file-read-failed".to_string(),
                message: "permission denied".to_string(),
            }),
            partial_failure: true,
            files: &[TraceDiagnosticsFile {
                path: "/tmp/server.trace.ndjson".to_string(),
                text: trace_record(
                    "server.getConfig",
                    "trace-a",
                    "span-a",
                    1_000,
                    50.0,
                    None,
                    Vec::new(),
                ),
            }],
        });

        assert_eq!(diagnostics.record_count, 1);
        assert_eq!(diagnostics.partial_failure, Some(true));
        assert_eq!(
            diagnostics.error.as_ref().map(|error| error.kind.as_str()),
            Some("trace-file-read-failed")
        );
        assert_eq!(
            diagnostics.scanned_file_paths,
            vec!["/tmp/server.trace.ndjson.1", "/tmp/server.trace.ndjson"]
        );
    }

    #[test]
    fn keeps_only_the_slowest_trace_span_occurrences() {
        let text = (0..25)
            .map(|index| {
                trace_record(
                    &format!("span-{index}"),
                    &format!("trace-{index}"),
                    &format!("span-{index}"),
                    index * 1_000,
                    index as f64,
                    None,
                    Vec::new(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let diagnostics = aggregate_trace_diagnostics(TraceDiagnosticsInput {
            trace_file_path: "/tmp/server.trace.ndjson",
            read_at: "2026-05-05T10:00:00.000Z",
            files: &[TraceDiagnosticsFile {
                path: "/tmp/server.trace.ndjson".to_string(),
                text,
            }],
            scanned_file_paths: None,
            slow_span_threshold_ms: None,
            error: None,
            partial_failure: false,
        });

        assert_eq!(diagnostics.record_count, 25);
        assert_eq!(diagnostics.slowest_spans.len(), 10);
        assert_eq!(
            diagnostics
                .slowest_spans
                .iter()
                .map(|span| span.duration_ms as i64)
                .collect::<Vec<_>>(),
            vec![24, 23, 22, 21, 20, 19, 18, 17, 16, 15]
        );
    }

    #[test]
    fn builds_rotated_trace_paths_like_upstream() {
        assert_eq!(
            to_rotated_trace_paths("/tmp/server.trace.ndjson", 3),
            vec![
                "/tmp/server.trace.ndjson.3",
                "/tmp/server.trace.ndjson.2",
                "/tmp/server.trace.ndjson.1",
                "/tmp/server.trace.ndjson",
            ]
        );
        assert_eq!(
            to_rotated_trace_paths("/tmp/server.trace.ndjson", -1),
            vec!["/tmp/server.trace.ndjson"]
        );
    }

    #[test]
    fn resolves_server_route_before_draft_route() {
        let target = resolve_thread_route_target(Some("env-1"), Some("thread-1"), Some("draft-1"));

        assert_eq!(
            target,
            Some(ThreadRouteTarget::Server {
                thread_ref: ScopedThreadRef::new("env-1", "thread-1")
            })
        );
    }

    #[test]
    fn resolves_draft_route_when_no_server_thread_params_exist() {
        let target = resolve_thread_route_target(None, None, Some("draft-1"));

        assert_eq!(
            target,
            Some(ThreadRouteTarget::Draft {
                draft_id: "draft-1".to_string()
            })
        );
    }

    #[test]
    fn index_route_does_not_render_chat_view() {
        assert!(!AppSnapshot::empty_reference_state().renders_chat_view());
    }

    #[test]
    fn draft_reference_state_renders_chat_view_with_draft_session() {
        let snapshot = AppSnapshot::draft_reference_state();

        assert!(snapshot.renders_chat_view());
        assert_eq!(snapshot.draft_sessions.len(), 1);
        assert_eq!(snapshot.messages, Vec::new());
        assert_eq!(snapshot.active_thread_title(), "New thread");
        assert_eq!(snapshot.active_project_name(), Some("server"));
    }

    #[test]
    fn mock_reference_state_exposes_active_thread_header_data() {
        let snapshot = AppSnapshot::mock_reference_state();

        assert_eq!(snapshot.active_thread_title(), "Port R3Code UI shell");
        assert_eq!(snapshot.active_project_name(), Some("r3code"));
        assert!(snapshot.turn_diff_summaries.is_empty());
    }

    #[test]
    fn command_palette_builds_recent_threads_with_upstream_sort_and_timestamp_rules() {
        let projects = vec![make_command_palette_project()];
        let threads = vec![
            make_command_palette_thread(
                "thread-older",
                "Older thread",
                "2026-03-23T12:00:00.000Z",
                "2026-03-24T12:00:00.000Z",
            ),
            make_command_palette_thread(
                "thread-newer",
                "Newer thread",
                "2026-03-20T00:00:00.000Z",
                "2026-03-20T00:00:00.000Z",
            ),
        ];

        let items = build_thread_action_items(
            &threads,
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        assert_eq!(
            items
                .iter()
                .map(|item| item.value.as_str())
                .collect::<Vec<_>>(),
            vec!["thread:thread-older", "thread:thread-newer"]
        );
        assert_eq!(items[0].timestamp.as_deref(), Some("1d ago"));
        assert_eq!(items[1].timestamp.as_deref(), Some("5d ago"));
    }

    #[test]
    fn command_palette_search_ranks_titles_over_context_and_filters_archived_threads() {
        let projects = vec![make_command_palette_project()];
        let mut context_match = make_command_palette_thread(
            "thread-context-match",
            "Fix navbar spacing",
            "2026-03-02T00:00:00.000Z",
            "2026-03-20T00:00:00.000Z",
        );
        context_match.project_name = "Project".to_string();
        let title_match = make_command_palette_thread(
            "thread-title-match",
            "Project kickoff notes",
            "2026-03-02T00:00:00.000Z",
            "2026-03-19T00:00:00.000Z",
        );
        let mut archived_match = make_command_palette_thread(
            "thread-archived",
            "Archived project thread",
            "2026-03-02T00:00:00.000Z",
            "2026-03-21T00:00:00.000Z",
        );
        archived_match.archived_at = Some("2026-03-22T00:00:00.000Z".to_string());
        let thread_items = build_thread_action_items(
            &[context_match, title_match, archived_match],
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        let groups = filter_command_palette_groups(&[], "project", false, &[], &thread_items);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].value, "threads-search");
        assert_eq!(
            groups[0]
                .items
                .iter()
                .map(|item| item.value.as_str())
                .collect::<Vec<_>>(),
            vec!["thread:thread-title-match", "thread:thread-context-match"]
        );
    }

    #[test]
    fn command_palette_filters_action_only_queries_and_injects_projects_and_threads() {
        let action_items = vec![
            CommandPaletteItem::action(
                "action:add-project",
                vec!["add project".to_string(), "folder".to_string()],
                "Add project",
            ),
            CommandPaletteItem::action(
                "action:settings",
                vec!["settings".to_string(), "preferences".to_string()],
                "Open settings",
            ),
        ];
        let root_groups = build_root_command_palette_groups(action_items, Vec::new());
        let projects = vec![make_command_palette_project()];
        let project_items = build_project_action_items(&projects, "project");
        let thread_items = build_thread_action_items(
            &[make_command_palette_thread(
                "thread-1",
                "Project kickoff notes",
                "2026-03-02T00:00:00.000Z",
                "2026-03-19T00:00:00.000Z",
            )],
            None,
            &projects,
            SidebarThreadSortOrder::UpdatedAt,
            "2026-03-25T12:00:00.000Z",
            None,
        );

        let action_groups = filter_command_palette_groups(
            &root_groups,
            ">settings",
            false,
            &project_items,
            &thread_items,
        );
        assert_eq!(action_groups.len(), 1);
        assert_eq!(action_groups[0].value, "actions");
        assert_eq!(action_groups[0].items[0].value, "action:settings");

        let search_groups = filter_command_palette_groups(
            &root_groups,
            "project",
            false,
            &project_items,
            &thread_items,
        );
        assert_eq!(
            search_groups
                .iter()
                .map(|group| group.value.as_str())
                .collect::<Vec<_>>(),
            vec!["actions", "projects-search", "threads-search"]
        );
    }

    #[test]
    fn provider_status_summary_matches_upstream_precedence() {
        let missing = get_provider_summary(None);
        assert_eq!(missing.headline, "Checking provider status");
        assert_eq!(
            missing.detail.as_deref(),
            Some("Waiting for the server to report installation and authentication details.")
        );

        let disabled = make_server_provider(|provider| {
            provider.enabled = false;
            provider.message = None;
        });
        assert_eq!(get_provider_summary(Some(&disabled)).headline, "Disabled");

        let not_found = make_server_provider(|provider| {
            provider.installed = false;
            provider.message = Some("Binary missing.".to_string());
        });
        assert_eq!(get_provider_summary(Some(&not_found)).headline, "Not found");
        assert_eq!(
            get_provider_summary(Some(&not_found)).detail.as_deref(),
            Some("Binary missing.")
        );

        let authenticated = make_server_provider(|provider| {
            provider.auth = ServerProviderAuth {
                status: ServerProviderAuthStatus::Authenticated,
                kind: Some("oauth".to_string()),
                label: Some("Codex Pro".to_string()),
                email: None,
            };
        });
        assert_eq!(
            get_provider_summary(Some(&authenticated)).headline,
            "Authenticated · Codex Pro"
        );

        let unauthenticated = make_server_provider(|provider| {
            provider.auth.status = ServerProviderAuthStatus::Unauthenticated;
        });
        assert_eq!(
            get_provider_summary(Some(&unauthenticated)).headline,
            "Not authenticated"
        );

        let warning = make_server_provider(|provider| {
            provider.status = ServerProviderState::Warning;
            provider.auth.status = ServerProviderAuthStatus::Unknown;
        });
        assert_eq!(
            get_provider_summary(Some(&warning)).headline,
            "Needs attention"
        );

        let error = make_server_provider(|provider| {
            provider.status = ServerProviderState::Error;
            provider.auth.status = ServerProviderAuthStatus::Unknown;
        });
        assert_eq!(get_provider_summary(Some(&error)).headline, "Unavailable");
    }

    #[test]
    fn provider_version_labels_and_advisories_match_upstream_logic() {
        assert_eq!(get_provider_version_label(None), None);
        assert_eq!(
            get_provider_version_label(Some("1.2.3")),
            Some("v1.2.3".to_string())
        );
        assert_eq!(
            get_provider_version_label(Some("v1.2.3")),
            Some("v1.2.3".to_string())
        );

        let current = ServerProviderVersionAdvisory {
            status: ServerProviderVersionAdvisoryStatus::Current,
            current_version: Some("1.2.3".to_string()),
            latest_version: Some("1.2.3".to_string()),
            update_command: None,
            can_update: false,
            checked_at: None,
            message: None,
        };
        assert_eq!(
            get_provider_version_advisory_presentation(Some(&current)),
            None
        );

        let behind = ServerProviderVersionAdvisory {
            status: ServerProviderVersionAdvisoryStatus::BehindLatest,
            current_version: Some("1.2.3".to_string()),
            latest_version: Some("1.2.4".to_string()),
            update_command: Some("npm install -g provider@latest".to_string()),
            can_update: true,
            checked_at: Some("2026-03-04T12:00:00.000Z".to_string()),
            message: None,
        };
        let presentation = get_provider_version_advisory_presentation(Some(&behind)).unwrap();
        assert_eq!(presentation.detail, "Update available: install v1.2.4.");
        assert_eq!(
            presentation.update_command.as_deref(),
            Some("npm install -g provider@latest")
        );

        let custom_message = ServerProviderVersionAdvisory {
            message: Some("Use your package manager to update.".to_string()),
            ..behind
        };
        assert_eq!(
            get_provider_version_advisory_presentation(Some(&custom_message))
                .unwrap()
                .detail,
            "Use your package manager to update."
        );
    }

    #[test]
    fn active_chat_reference_state_links_diff_summary_to_assistant_message() {
        let snapshot = AppSnapshot::active_chat_reference_state();
        let assistant_message = snapshot
            .messages
            .iter()
            .find(|message| message.role == MessageRole::Assistant)
            .unwrap();

        assert_eq!(snapshot.turn_diff_summaries.len(), 2);
        assert!(
            snapshot
                .turn_diff_summaries
                .iter()
                .any(|summary| summary.assistant_message_id.as_deref()
                    == Some(assistant_message.id.as_str()))
        );
    }

    #[test]
    fn branch_toolbar_labels_match_upstream_logic() {
        assert_eq!(
            resolve_env_mode_label(DraftThreadEnvMode::Local),
            "Current checkout"
        );
        assert_eq!(
            resolve_env_mode_label(DraftThreadEnvMode::Worktree),
            "New worktree"
        );
        assert_eq!(resolve_current_workspace_label(None), "Current checkout");
        assert_eq!(
            resolve_current_workspace_label(Some("/repo/.t3/worktrees/feature-a")),
            "Current worktree"
        );
        assert_eq!(resolve_locked_workspace_label(None), "Local checkout");
        assert_eq!(
            resolve_locked_workspace_label(Some("/repo/.t3/worktrees/feature-a")),
            "Worktree"
        );
        assert_eq!(
            resolve_environment_option_label(
                true,
                "environment-local",
                Some("Local environment"),
                Some("Local")
            ),
            "This device"
        );
        assert_eq!(
            resolve_environment_option_label(false, "environment-remote", None, Some("Build box")),
            "Build box"
        );
    }

    #[test]
    fn branch_toolbar_env_mode_and_value_match_upstream_logic() {
        assert_eq!(
            resolve_effective_env_mode(
                Some("/repo/.t3/worktrees/feature-a"),
                false,
                Some(DraftThreadEnvMode::Worktree)
            ),
            DraftThreadEnvMode::Local
        );
        assert_eq!(
            resolve_effective_env_mode(None, false, Some(DraftThreadEnvMode::Worktree)),
            DraftThreadEnvMode::Worktree
        );
        assert_eq!(
            resolve_draft_env_mode_after_branch_change(
                None,
                Some("/repo/.t3/worktrees/feature-a"),
                DraftThreadEnvMode::Worktree
            ),
            DraftThreadEnvMode::Local
        );
        assert_eq!(
            resolve_draft_env_mode_after_branch_change(None, None, DraftThreadEnvMode::Worktree),
            DraftThreadEnvMode::Worktree
        );
        assert_eq!(
            resolve_branch_toolbar_value(DraftThreadEnvMode::Worktree, None, None, Some("main")),
            Some("main".to_string())
        );
        assert_eq!(
            resolve_branch_toolbar_value(
                DraftThreadEnvMode::Worktree,
                None,
                Some("feature/base"),
                Some("main")
            ),
            Some("feature/base".to_string())
        );
        assert_eq!(
            resolve_branch_toolbar_value(
                DraftThreadEnvMode::Local,
                None,
                Some("feature/base"),
                Some("main")
            ),
            Some("main".to_string())
        );
        assert_eq!(
            branch_toolbar_trigger_label(None, DraftThreadEnvMode::Worktree, Some("main")),
            "From main"
        );
    }

    #[test]
    fn branch_selection_target_matches_upstream_worktree_rules() {
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("feature-b", false, Some("/repo/.t3/worktrees/feature-b"))
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo/.t3/worktrees/feature-b".to_string(),
                next_worktree_path: Some("/repo/.t3/worktrees/feature-b".to_string()),
                reuse_existing_worktree: true,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("main", true, Some("/repo"))
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo".to_string(),
                next_worktree_path: None,
                reuse_existing_worktree: true,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("main", true, None)
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo".to_string(),
                next_worktree_path: None,
                reuse_existing_worktree: false,
            }
        );
        assert_eq!(
            resolve_branch_selection_target(
                "/repo",
                Some("/repo/.t3/worktrees/feature-a"),
                &vcs_ref("feature-a", false, None)
            ),
            BranchSelectionTarget {
                checkout_cwd: "/repo/.t3/worktrees/feature-a".to_string(),
                next_worktree_path: Some("/repo/.t3/worktrees/feature-a".to_string()),
                reuse_existing_worktree: false,
            }
        );
    }

    #[test]
    fn branch_picker_helpers_match_upstream_filtering() {
        assert_eq!(
            derive_local_branch_name_from_remote_ref("origin/feature/demo"),
            "feature/demo"
        );
        assert_eq!(
            derive_local_branch_name_from_remote_ref("my-org/upstream/feature/demo"),
            "upstream/feature/demo"
        );
        assert_eq!(
            derive_local_branch_name_from_remote_ref("origin/"),
            "origin/"
        );
        assert_eq!(
            dedupe_remote_branches_with_local_matches(&[
                vcs_ref("feature/demo", false, None),
                remote_vcs_ref("origin/feature/demo", "origin"),
                remote_vcs_ref("origin/feature/remote-only", "origin"),
            ])
            .iter()
            .map(|ref_name| ref_name.name.as_str())
            .collect::<Vec<_>>(),
            vec!["feature/demo", "origin/feature/remote-only"]
        );
        assert!(should_include_branch_picker_item(
            "__checkout_pull_request__:1359",
            "gh pr checkout 1359",
            Some("__create_new_branch__:gh pr checkout 1359"),
            Some("__checkout_pull_request__:1359")
        ));
        assert!(should_include_branch_picker_item(
            "__create_new_branch__:feature/demo",
            "feature/demo",
            Some("__create_new_branch__:feature/demo"),
            None
        ));
        assert!(!should_include_branch_picker_item(
            "main",
            "gh pr checkout 1359",
            Some("__create_new_branch__:gh pr checkout 1359"),
            Some("__checkout_pull_request__:1359")
        ));
    }

    #[test]
    fn branch_toolbar_reference_state_exposes_new_worktree_context() {
        let snapshot = AppSnapshot::branch_toolbar_reference_state();
        let toolbar = snapshot.active_branch_toolbar_state().unwrap();

        assert_eq!(toolbar.effective_env_mode, DraftThreadEnvMode::Worktree);
        assert_eq!(toolbar.workspace_label, "New worktree");
        assert_eq!(toolbar.branch_label, "From main");
        assert!(toolbar.show_environment_picker);
    }

    #[test]
    fn project_scripts_helpers_match_upstream_logic() {
        let command = command_for_project_script("lint");
        assert_eq!(command, "script.lint.run");
        assert_eq!(
            project_script_id_from_command(&command),
            Some("lint".to_string())
        );
        assert_eq!(project_script_id_from_command("terminal.toggle"), None);
        assert_eq!(
            next_project_script_id("Run Tests", [] as [&str; 0]),
            "run-tests"
        );
        assert_eq!(
            next_project_script_id("Run Tests", ["run-tests"]),
            "run-tests-2"
        );
        assert_eq!(next_project_script_id("!!!", [] as [&str; 0]), "script");

        let scripts = vec![
            ProjectScript {
                id: "setup".to_string(),
                name: "Setup".to_string(),
                command: "bun install".to_string(),
                icon: ProjectScriptIcon::Configure,
                run_on_worktree_create: true,
            },
            ProjectScript {
                id: "test".to_string(),
                name: "Test".to_string(),
                command: "bun test".to_string(),
                icon: ProjectScriptIcon::Test,
                run_on_worktree_create: false,
            },
        ];

        assert_eq!(primary_project_script(&scripts).unwrap().id, "test");
        assert_eq!(setup_project_script(&scripts).unwrap().id, "setup");
    }

    #[test]
    fn project_script_runtime_context_matches_upstream_logic() {
        let env = project_script_runtime_env("/repo", Some("/repo/worktree-a"), &[]);

        assert_eq!(
            env.get("T3CODE_PROJECT_ROOT").map(String::as_str),
            Some("/repo")
        );
        assert_eq!(
            env.get("T3CODE_WORKTREE_PATH").map(String::as_str),
            Some("/repo/worktree-a")
        );
        assert_eq!(
            project_script_cwd("/repo", Some("/repo/worktree-a")),
            "/repo/worktree-a"
        );
        assert_eq!(project_script_cwd("/repo", None), "/repo");

        let env = project_script_runtime_env(
            "/repo",
            None,
            &[
                ("T3CODE_PROJECT_ROOT", "/custom-root"),
                ("CUSTOM_FLAG", "1"),
            ],
        );
        assert_eq!(
            env.get("T3CODE_PROJECT_ROOT").map(String::as_str),
            Some("/custom-root")
        );
        assert_eq!(env.get("CUSTOM_FLAG").map(String::as_str), Some("1"));
        assert!(!env.contains_key("T3CODE_WORKTREE_PATH"));
    }

    #[test]
    fn open_in_picker_visibility_and_options_match_upstream_logic() {
        assert!(should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-primary",
            Some("environment-primary")
        ));
        assert!(!should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-remote",
            None
        ));
        assert!(!should_show_open_in_picker(
            Some("codething-mvp"),
            "environment-remote",
            Some("environment-primary")
        ));
        assert!(!should_show_open_in_picker(
            None,
            "environment-primary",
            Some("environment-primary")
        ));

        let options = resolve_editor_options(
            "Windows",
            &[
                EditorId::VsCodeInsiders,
                EditorId::VsCodium,
                EditorId::FileManager,
            ],
        );
        assert_eq!(
            options
                .iter()
                .map(|option| option.label)
                .collect::<Vec<_>>(),
            vec!["VS Code Insiders", "VSCodium", "Explorer"]
        );
    }

    #[test]
    fn provider_instance_projection_matches_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let entries = derive_provider_instance_entries(&snapshot.providers);
        let codex = entries
            .iter()
            .find(|entry| entry.instance_id == "codex")
            .unwrap();
        let personal = entries
            .iter()
            .find(|entry| entry.instance_id == "codex_personal")
            .unwrap();
        let cursor = entries
            .iter()
            .find(|entry| entry.instance_id == "cursor")
            .unwrap();

        assert_eq!(codex.display_name, "Codex");
        assert!(codex.is_default);
        assert_eq!(personal.display_name, "Codex Personal");
        assert_eq!(personal.accent_color.as_deref(), Some("#2563EB"));
        assert!(!personal.is_default);
        assert!(!cursor.is_available);
        assert_eq!(provider_instance_initials("Codex Personal"), "CP");
        assert_eq!(normalize_provider_accent_color(Some("not-a-color")), None);

        let sorted = sort_provider_instance_entries(&entries);
        let codex_index = sorted
            .iter()
            .position(|entry| entry.instance_id == "codex")
            .unwrap();
        let personal_index = sorted
            .iter()
            .position(|entry| entry.instance_id == "codex_personal")
            .unwrap();
        assert!(codex_index < personal_index);
    }

    #[test]
    fn model_picker_trigger_filtering_and_locking_match_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let state = resolve_model_picker_state(&snapshot, "", None, None, None);

        assert_eq!(state.trigger_title, "5.4 Mini");
        assert_eq!(state.trigger_label, "5.4 Mini");
        assert!(state.show_instance_badge);
        assert_eq!(
            state.selected_instance,
            ModelPickerSelectedInstance::Favorites
        );
        assert!(state.show_sidebar);
        assert_eq!(
            state
                .filtered_models
                .iter()
                .map(|model| provider_model_key(&model.instance_id, &model.slug))
                .collect::<Vec<_>>(),
            vec!["codex:gpt-5.4", "claudeAgent:claude-sonnet-4-6"]
        );

        let search = resolve_model_picker_state(&snapshot, "sonnet", None, None, None);
        assert!(!search.show_sidebar);
        assert_eq!(search.filtered_models[0].slug, "claude-sonnet-4-6");

        let locked = resolve_model_picker_state(
            &snapshot,
            "",
            Some(ModelPickerSelectedInstance::Instance("codex".to_string())),
            Some("codex"),
            Some("codex-default"),
        );
        assert!(locked.is_locked);
        assert!(!locked.show_locked_instance_sidebar);
        assert_eq!(locked.locked_header_label.as_deref(), Some("Codex"));
        assert!(
            locked
                .filtered_models
                .iter()
                .all(|model| model.instance_id == "codex")
        );
    }

    #[test]
    fn model_picker_search_sorting_and_selection_match_upstream_logic() {
        let snapshot = AppSnapshot::mock_reference_state();
        let (_, slug) = split_instance_model_key("codex:openai/custom:model");
        assert_eq!(slug, "openai/custom:model");

        let codex_models = &snapshot.providers[0].models;
        assert_eq!(
            resolve_selectable_model("codex", Some("5.4"), codex_models),
            Some("gpt-5.4".to_string())
        );
        assert_eq!(
            resolve_selectable_model("codex", Some("GPT-5.3 Codex"), codex_models),
            Some("gpt-5.3-codex".to_string())
        );
        assert_eq!(
            resolve_selectable_provider_instance(&snapshot.providers, Some("missing")),
            Some("codex".to_string())
        );

        let favorites = favorite_model_key_set(&snapshot.model_favorites);
        let state = resolve_model_picker_state(
            &snapshot,
            "",
            Some(ModelPickerSelectedInstance::Instance("codex".to_string())),
            None,
            None,
        );
        let sorted = sort_provider_model_items(&state.filtered_models, &favorites, true, &[]);
        assert_eq!(sorted[0].slug, "gpt-5.4");
        assert!(score_model_picker_search(&sorted[0], "5.4").unwrap() < 10);
        assert_eq!(
            build_model_picker_search_text(&sorted[0]),
            "gpt-5.4 5.4 codex codex"
        );
    }

    #[test]
    fn running_turn_reference_state_exposes_work_log_entries() {
        let snapshot = AppSnapshot::running_turn_reference_state();
        let entries = snapshot.work_log_entries();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::Running);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].tone, ActivityTone::Thinking);
        assert_eq!(
            entries[1].command.as_deref(),
            Some("cargo test --workspace")
        );
        assert_eq!(
            entries[2].changed_files,
            vec!["crates/r3_core/src/lib.rs", "crates/r3_ui/src/shell.rs"]
        );
    }

    #[test]
    fn derives_work_log_entries_with_upstream_filters_and_collapse() {
        let activities = vec![
            ThreadActivity {
                id: "started".to_string(),
                kind: "tool.started".to_string(),
                summary: "Started command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-1".to_string()),
                sequence: Some(1),
                created_at: "2026-03-04T12:00:01.000Z".to_string(),
            },
            ThreadActivity {
                id: "updated".to_string(),
                kind: "tool.updated".to_string(),
                summary: "Ran command".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    command: Some("cargo check".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-1".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-1".to_string()),
                sequence: Some(2),
                created_at: "2026-03-04T12:00:02.000Z".to_string(),
            },
            ThreadActivity {
                id: "completed".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Ran command completed".to_string(),
                tone: ActivityTone::Tool,
                payload: ActivityPayload {
                    detail: Some("Finished in 1s".to_string()),
                    title: Some("terminal".to_string()),
                    item_type: Some("command_execution".to_string()),
                    tool_call_id: Some("tool-1".to_string()),
                    ..ActivityPayload::default()
                },
                turn_id: Some("turn-1".to_string()),
                sequence: Some(3),
                created_at: "2026-03-04T12:00:03.000Z".to_string(),
            },
            ThreadActivity {
                id: "checkpoint".to_string(),
                kind: "tool.completed".to_string(),
                summary: "Checkpoint captured".to_string(),
                tone: ActivityTone::Info,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-1".to_string()),
                sequence: Some(4),
                created_at: "2026-03-04T12:00:04.000Z".to_string(),
            },
            ThreadActivity {
                id: "other-turn".to_string(),
                kind: "task.progress".to_string(),
                summary: "Other turn".to_string(),
                tone: ActivityTone::Thinking,
                payload: ActivityPayload::default(),
                turn_id: Some("turn-2".to_string()),
                sequence: Some(5),
                created_at: "2026-03-04T12:00:05.000Z".to_string(),
            },
        ];

        let entries = derive_work_log_entries(&activities, Some("turn-1"));

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "completed");
        assert_eq!(entries[0].command.as_deref(), Some("cargo check"));
        assert_eq!(entries[0].detail.as_deref(), Some("Finished in 1s"));
    }

    #[test]
    fn pending_approval_reference_state_exposes_first_approval() {
        let snapshot = AppSnapshot::pending_approval_reference_state();
        let approval = snapshot.active_pending_approval().unwrap();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::NeedsInput);
        assert!(snapshot.threads[0].has_pending_approvals);
        assert_eq!(approval.request_kind, ApprovalRequestKind::Command);
        assert_eq!(approval.request_id, "approval-command-run-tests");
        assert!(!snapshot.is_responding_to_request(&approval.request_id));
    }

    #[test]
    fn pending_user_input_reference_state_exposes_active_progress() {
        let snapshot = AppSnapshot::pending_user_input_reference_state();
        let progress = snapshot.active_pending_user_input_progress().unwrap();

        assert_eq!(snapshot.threads[0].status, ThreadStatus::NeedsInput);
        assert!(snapshot.threads[0].has_pending_user_input);
        assert_eq!(progress.question_index, 0);
        assert_eq!(progress.active_question.unwrap().id, "surface");
        assert_eq!(progress.selected_option_labels, vec!["Composer"]);
        assert_eq!(
            progress.resolved_answer,
            Some(PendingUserInputAnswer::Text("Composer".to_string()))
        );
        assert!(progress.can_advance);
        assert!(!progress.is_complete);
    }

    fn make_terminal_context(
        overrides: impl FnOnce(&mut TerminalContextDraft),
    ) -> TerminalContextDraft {
        let mut context = TerminalContextDraft {
            id: "context-1".to_string(),
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            terminal_label: "Terminal 1".to_string(),
            line_start: 12,
            line_end: 13,
            text: "git status\nOn branch main".to_string(),
            created_at: "2026-03-13T12:00:00.000Z".to_string(),
        };
        overrides(&mut context);
        context
    }

    #[test]
    fn terminal_context_formatting_matches_upstream_contract() {
        let context = make_terminal_context(|_| {});
        assert_eq!(
            format_terminal_context_label(&context.selection()),
            "Terminal 1 lines 12-13"
        );

        let single_line = make_terminal_context(|context| {
            context.line_start = 9;
            context.line_end = 9;
        });
        assert_eq!(
            format_terminal_context_label(&single_line.selection()),
            "Terminal 1 line 9"
        );

        assert_eq!(
            build_terminal_context_block(&[context.selection()]),
            [
                "<terminal_context>",
                "- Terminal 1 lines 12-13:",
                "  12 | git status",
                "  13 | On branch main",
                "</terminal_context>",
            ]
            .join("\n")
        );
    }

    #[test]
    fn terminal_context_prompt_materialization_matches_upstream_contract() {
        let context = make_terminal_context(|_| {});
        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;

        assert_eq!(
            append_terminal_contexts_to_prompt("Investigate this", &[context.selection()]),
            [
                "Investigate this",
                "",
                "<terminal_context>",
                "- Terminal 1 lines 12-13:",
                "  12 | git status",
                "  13 | On branch main",
                "</terminal_context>",
            ]
            .join("\n")
        );

        assert_eq!(
            append_terminal_contexts_to_prompt(
                &format!("Investigate {placeholder} carefully"),
                &[context.selection()]
            ),
            [
                "Investigate @terminal-1:12-13 carefully",
                "",
                "<terminal_context>",
                "- Terminal 1 lines 12-13:",
                "  12 | git status",
                "  13 | On branch main",
                "</terminal_context>",
            ]
            .join("\n")
        );
        assert_eq!(
            materialize_inline_terminal_context_prompt(
                &format!("Investigate {placeholder} carefully"),
                &[context.selection()]
            ),
            "Investigate @terminal-1:12-13 carefully"
        );
    }

    #[test]
    fn extracts_terminal_context_blocks_like_upstream() {
        let context = make_terminal_context(|_| {});
        let prompt = append_terminal_contexts_to_prompt("Investigate this", &[context.selection()]);

        assert_eq!(
            extract_trailing_terminal_contexts(&prompt),
            ExtractedTerminalContexts {
                prompt_text: "Investigate this".to_string(),
                context_count: 1,
                preview_title: Some(
                    "Terminal 1 lines 12-13\n12 | git status\n13 | On branch main".to_string()
                ),
                contexts: vec![ParsedTerminalContextEntry {
                    header: "Terminal 1 lines 12-13".to_string(),
                    body: "12 | git status\n13 | On branch main".to_string(),
                }],
            }
        );
        assert_eq!(
            derive_displayed_user_message_state(&prompt),
            DisplayedUserMessageState {
                visible_text: "Investigate this".to_string(),
                copy_text: prompt,
                context_count: 1,
                preview_title: Some(
                    "Terminal 1 lines 12-13\n12 | git status\n13 | On branch main".to_string()
                ),
                contexts: vec![ParsedTerminalContextEntry {
                    header: "Terminal 1 lines 12-13".to_string(),
                    body: "12 | git status\n13 | On branch main".to_string(),
                }],
            }
        );
        assert_eq!(
            extract_trailing_terminal_contexts("No attached context"),
            ExtractedTerminalContexts {
                prompt_text: "No attached context".to_string(),
                context_count: 0,
                preview_title: None,
                contexts: Vec::new(),
            }
        );
    }

    #[test]
    fn inline_terminal_context_placeholders_match_upstream_contract() {
        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;

        assert_eq!(
            count_inline_terminal_context_placeholders(&format!("a{placeholder}b{placeholder}")),
            2
        );
        assert_eq!(
            ensure_inline_terminal_context_placeholders("Investigate this", 2),
            format!("{placeholder}{placeholder}Investigate this")
        );
        assert_eq!(
            insert_inline_terminal_context_placeholder("abc", 1),
            InlineTerminalContextInsertion {
                prompt: format!("a {placeholder} bc"),
                cursor: 4,
                context_index: 0,
            }
        );
        assert_eq!(
            remove_inline_terminal_context_placeholder(
                &format!("a{placeholder}b{placeholder}c"),
                1
            ),
            InlineTerminalContextRemoval {
                prompt: format!("a{placeholder}bc"),
                cursor: 3,
            }
        );
        assert_eq!(
            strip_inline_terminal_context_placeholders(&format!("a{placeholder}b")),
            "ab"
        );
        assert_eq!(
            insert_inline_terminal_context_placeholder("Inspect @package.json ", 22),
            InlineTerminalContextInsertion {
                prompt: format!("Inspect @package.json {placeholder} "),
                cursor: 24,
                context_index: 0,
            }
        );
        assert_eq!(
            insert_inline_terminal_context_placeholder("yo whats", 3),
            InlineTerminalContextInsertion {
                prompt: format!("yo {placeholder} whats"),
                cursor: 5,
                context_index: 0,
            }
        );
    }

    #[test]
    fn terminal_context_expiry_and_preview_match_upstream_contract() {
        let live_context = make_terminal_context(|_| {});
        let expired_context = make_terminal_context(|context| {
            context.id = "context-2".to_string();
            context.text.clear();
        });
        let invalid_context = make_terminal_context(|context| {
            context.terminal_id = "   ".to_string();
        });
        let blank_context = make_terminal_context(|context| {
            context.id = "context-3".to_string();
            context.text = "\n\n".to_string();
        });

        assert!(has_terminal_context_text(&live_context.text));
        assert!(!is_terminal_context_expired(&live_context.text));
        assert!(!has_terminal_context_text(&expired_context.text));
        assert!(is_terminal_context_expired(&expired_context.text));
        assert_eq!(
            filter_terminal_contexts_with_text(&[expired_context.clone(), live_context.clone()]),
            vec![live_context.clone()]
        );
        assert_eq!(
            build_terminal_context_preview_title(&[
                invalid_context.selection(),
                blank_context.selection()
            ]),
            None
        );
        assert_eq!(
            format_inline_terminal_context_label(&live_context.selection()),
            "@terminal-1:12-13"
        );
    }

    #[test]
    fn composer_send_state_and_expired_terminal_copy_match_upstream() {
        let expired_context = make_terminal_context(|context| {
            context.id = "ctx-expired".to_string();
            context.text.clear();
            context.line_start = 4;
            context.line_end = 4;
        });
        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;

        let expired_only =
            derive_composer_send_state(&placeholder.to_string(), 0, &[expired_context.clone()]);
        assert_eq!(expired_only.trimmed_prompt, "");
        assert_eq!(expired_only.sendable_terminal_contexts, Vec::new());
        assert_eq!(expired_only.expired_terminal_context_count, 1);
        assert!(!expired_only.has_sendable_content);

        let with_text =
            derive_composer_send_state(&format!("yoo {placeholder} waddup"), 0, &[expired_context]);
        assert_eq!(with_text.trimmed_prompt, "yoo  waddup");
        assert_eq!(with_text.expired_terminal_context_count, 1);
        assert!(with_text.has_sendable_content);

        assert_eq!(
            build_expired_terminal_context_toast_copy(1, ExpiredTerminalContextToastVariant::Empty),
            ExpiredTerminalContextToastCopy {
                title: "Expired terminal context won't be sent".to_string(),
                description: "Remove it or re-add it to include terminal output.",
            }
        );
        assert_eq!(
            build_expired_terminal_context_toast_copy(
                2,
                ExpiredTerminalContextToastVariant::Omitted
            ),
            ExpiredTerminalContextToastCopy {
                title: "Expired terminal contexts omitted from message".to_string(),
                description: "Re-add it if you want that terminal output included.",
            }
        );
    }

    #[test]
    fn composer_segment_parser_matches_upstream_contract() {
        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;

        assert_eq!(
            split_prompt_into_composer_segments("Inspect @AGENTS.md please"),
            vec![
                ComposerPromptSegment::Text {
                    text: "Inspect ".to_string(),
                },
                ComposerPromptSegment::Mention {
                    path: "AGENTS.md".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " please".to_string(),
                },
            ]
        );
        assert_eq!(
            split_prompt_into_composer_segments("Inspect @AGENTS.md"),
            vec![ComposerPromptSegment::Text {
                text: "Inspect @AGENTS.md".to_string(),
            }]
        );
        assert_eq!(
            split_prompt_into_composer_segments("one\n@src/index.ts \ntwo"),
            vec![
                ComposerPromptSegment::Text {
                    text: "one\n".to_string(),
                },
                ComposerPromptSegment::Mention {
                    path: "src/index.ts".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " \ntwo".to_string(),
                },
            ]
        );
        assert_eq!(
            split_prompt_into_composer_segments("Use $review-follow-up please"),
            vec![
                ComposerPromptSegment::Text {
                    text: "Use ".to_string(),
                },
                ComposerPromptSegment::Skill {
                    name: "review-follow-up".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " please".to_string(),
                },
            ]
        );
        assert_eq!(
            split_prompt_into_composer_segments("Use $review-follow-up"),
            vec![ComposerPromptSegment::Text {
                text: "Use $review-follow-up".to_string(),
            }]
        );
        assert_eq!(
            split_prompt_into_composer_segments(&format!("Inspect {placeholder}@AGENTS.md please")),
            vec![
                ComposerPromptSegment::Text {
                    text: "Inspect ".to_string(),
                },
                ComposerPromptSegment::TerminalContext { context: None },
                ComposerPromptSegment::Mention {
                    path: "AGENTS.md".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " please".to_string(),
                },
            ]
        );
        assert_eq!(
            split_prompt_into_composer_segments(&format!("{placeholder}{placeholder}tail")),
            vec![
                ComposerPromptSegment::TerminalContext { context: None },
                ComposerPromptSegment::TerminalContext { context: None },
                ComposerPromptSegment::Text {
                    text: "tail".to_string(),
                },
            ]
        );
        assert_eq!(
            split_prompt_into_composer_segments(&format!(
                "Inspect {placeholder}$review-follow-up after @AGENTS.md "
            )),
            vec![
                ComposerPromptSegment::Text {
                    text: "Inspect ".to_string(),
                },
                ComposerPromptSegment::TerminalContext { context: None },
                ComposerPromptSegment::Skill {
                    name: "review-follow-up".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " after ".to_string(),
                },
                ComposerPromptSegment::Mention {
                    path: "AGENTS.md".to_string(),
                },
                ComposerPromptSegment::Text {
                    text: " ".to_string(),
                },
            ]
        );

        let context = make_terminal_context(|_| {});
        assert_eq!(
            split_prompt_into_composer_segments_for_terminal_contexts(
                &placeholder.to_string(),
                std::slice::from_ref(&context),
            ),
            vec![ComposerPromptSegment::TerminalContext {
                context: Some(context),
            }]
        );
    }

    #[test]
    fn composer_selection_boundary_detection_matches_upstream_contract() {
        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;

        assert!(selection_touches_mention_boundary(
            "hi @package.json there",
            "hi @package.json".chars().count(),
            "hi @package.json there".chars().count(),
        ));
        assert!(selection_touches_mention_boundary(
            "hi there @package.json later",
            "hi there".chars().count(),
            "hi there ".chars().count(),
        ));
        assert!(!selection_touches_mention_boundary(
            "hi @package.json there",
            "hi @package.json ".chars().count(),
            "hi @package.json there".chars().count(),
        ));

        let prompt = format!("{placeholder}@AGENTS.md there");
        assert!(selection_touches_mention_boundary(
            &prompt,
            format!("{placeholder}@AGENTS.md").chars().count(),
            prompt.chars().count(),
        ));
    }

    #[test]
    fn composer_trigger_detection_matches_upstream_contract() {
        let text = "Please check @src/com";
        assert_eq!(
            detect_composer_trigger(text, text.chars().count() as f64),
            Some(ComposerTrigger {
                kind: ComposerTriggerKind::Path,
                query: "src/com".to_string(),
                range_start: "Please check ".chars().count(),
                range_end: text.chars().count(),
            })
        );

        for (text, query) in [
            ("/mo", "mo"),
            ("/model", "model"),
            ("/pl", "pl"),
            ("/rev", "rev"),
        ] {
            assert_eq!(
                detect_composer_trigger(text, text.chars().count() as f64),
                Some(ComposerTrigger {
                    kind: ComposerTriggerKind::SlashCommand,
                    query: query.to_string(),
                    range_start: 0,
                    range_end: text.chars().count(),
                })
            );
        }

        assert_eq!(
            detect_composer_trigger("/model spark", "/model spark".chars().count() as f64),
            None
        );

        let text = "Use $gh-fi";
        assert_eq!(
            detect_composer_trigger(text, text.chars().count() as f64),
            Some(ComposerTrigger {
                kind: ComposerTriggerKind::Skill,
                query: "gh-fi".to_string(),
                range_start: "Use ".chars().count(),
                range_end: text.chars().count(),
            })
        );

        let text = "Please inspect @in this sentence";
        let cursor_after_at = "Please inspect @".chars().count();
        assert_eq!(
            detect_composer_trigger(text, cursor_after_at as f64),
            Some(ComposerTrigger {
                kind: ComposerTriggerKind::Path,
                query: String::new(),
                range_start: "Please inspect ".chars().count(),
                range_end: cursor_after_at,
            })
        );

        let text = "Please inspect @srin this sentence";
        let cursor_after_query = "Please inspect @sr".chars().count();
        assert_eq!(
            detect_composer_trigger(text, cursor_after_query as f64),
            Some(ComposerTrigger {
                kind: ComposerTriggerKind::Path,
                query: "sr".to_string(),
                range_start: "Please inspect ".chars().count(),
                range_end: cursor_after_query,
            })
        );
    }

    #[test]
    fn composer_cursor_transforms_match_upstream_contract() {
        assert_eq!(expand_collapsed_composer_cursor("plain text", 5.0), 5);
        assert_eq!(collapse_expanded_composer_cursor("plain text", 5.0), 5);

        let text = "what's in my @AGENTS.md fsfdas";
        let collapsed_cursor_after_mention = "what's in my ".chars().count() + 2;
        let expanded_cursor_after_mention = "what's in my @AGENTS.md ".chars().count();
        assert_eq!(
            expand_collapsed_composer_cursor(text, collapsed_cursor_after_mention as f64),
            expanded_cursor_after_mention
        );
        assert_eq!(
            collapse_expanded_composer_cursor(text, expanded_cursor_after_mention as f64),
            collapsed_cursor_after_mention
        );

        let text = "what's in my @AGENTS.md ";
        let expanded_cursor =
            expand_collapsed_composer_cursor(text, collapsed_cursor_after_mention as f64);
        assert_eq!(detect_composer_trigger(text, expanded_cursor as f64), None);

        let text = "run $review-follow-up then";
        let collapsed_cursor_after_skill = "run ".chars().count() + 2;
        let expanded_cursor_after_skill = "run $review-follow-up ".chars().count();
        assert_eq!(
            expand_collapsed_composer_cursor(text, collapsed_cursor_after_skill as f64),
            expanded_cursor_after_skill
        );
        assert_eq!(
            collapse_expanded_composer_cursor(text, expanded_cursor_after_skill as f64),
            collapsed_cursor_after_skill
        );

        let text = "open @AGENTS.md then @src/index.ts ";
        let expanded_cursor = text.chars().count();
        let collapsed_cursor = collapse_expanded_composer_cursor(text, expanded_cursor as f64);
        assert_eq!(
            collapsed_cursor,
            "open ".chars().count() + 1 + " then ".chars().count() + 2
        );
        assert_eq!(
            expand_collapsed_composer_cursor(text, collapsed_cursor as f64),
            expanded_cursor
        );

        let text = "open @AGENTS.md then ";
        assert_eq!(
            clamp_collapsed_composer_cursor(text, text.chars().count() as f64),
            "open ".chars().count() + 1 + " then ".chars().count()
        );
        assert_eq!(
            clamp_collapsed_composer_cursor(text, f64::INFINITY),
            "open ".chars().count() + 1 + " then ".chars().count()
        );
    }

    #[test]
    fn composer_inline_token_adjacency_matches_upstream_contract() {
        use ComposerCursorAdjacencyDirection::{Left, Right};

        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            "plain text",
            6.0,
            Left
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            "plain text",
            6.0,
            Right
        ));

        let text = "hello @pac";
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            text.chars().count() as f64,
            Left
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            text.chars().count() as f64,
            Right
        ));

        let text = "open @AGENTS.md next";
        let mention_start = "open ".chars().count();
        let mention_end = mention_start + 1;
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            text,
            mention_end as f64,
            Left
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            mention_start as f64,
            Left
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            (mention_end + 1) as f64,
            Left
        ));
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            text,
            mention_start as f64,
            Right
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            mention_end as f64,
            Right
        ));
        assert!(!is_collapsed_cursor_adjacent_to_inline_token(
            text,
            (mention_start - 1) as f64,
            Right
        ));

        let placeholder = INLINE_TERMINAL_CONTEXT_PLACEHOLDER;
        let text = format!("open {placeholder} next");
        let token_start = "open ".chars().count();
        let token_end = token_start + 1;
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            &text,
            token_end as f64,
            Left
        ));
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            &text,
            token_start as f64,
            Right
        ));

        let text = "run $review-follow-up next";
        let token_start = "run ".chars().count();
        let token_end = token_start + 1;
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            text,
            token_end as f64,
            Left
        ));
        assert!(is_collapsed_cursor_adjacent_to_inline_token(
            text,
            token_start as f64,
            Right
        ));
    }

    #[test]
    fn composer_slash_command_and_replacement_match_upstream_contract() {
        assert_eq!(
            replace_text_range("hello @src", 6.0, 10.0, ""),
            TextRangeReplacement {
                text: "hello ".to_string(),
                cursor: 6,
            }
        );

        let text = "and then @AG summarize";
        let range_start = "and then ".chars().count();
        let range_end = "and then @AG".chars().count();
        assert_eq!(
            replace_text_range(text, range_start as f64, range_end as f64, "@AGENTS.md ").text,
            "and then @AGENTS.md  summarize"
        );
        let extended_end = if char_at(text, range_end) == Some(' ') {
            range_end + 1
        } else {
            range_end
        };
        assert_eq!(
            replace_text_range(text, range_start as f64, extended_end as f64, "@AGENTS.md ").text,
            "and then @AGENTS.md summarize"
        );

        assert_eq!(
            parse_standalone_composer_slash_command(" /plan "),
            Some(ComposerSlashCommand::Plan)
        );
        assert_eq!(
            parse_standalone_composer_slash_command("/default"),
            Some(ComposerSlashCommand::Default)
        );
        assert_eq!(
            parse_standalone_composer_slash_command("/plan explain this"),
            None
        );
        assert_eq!(parse_standalone_composer_slash_command("/model"), None);
    }

    fn make_provider_skill(
        name: &str,
        overrides: impl FnOnce(&mut ServerProviderSkill),
    ) -> ServerProviderSkill {
        let mut skill = ServerProviderSkill {
            name: name.to_string(),
            description: None,
            path: format!("/tmp/{name}/SKILL.md"),
            scope: None,
            enabled: true,
            display_name: None,
            short_description: None,
        };
        overrides(&mut skill);
        skill
    }

    fn make_provider_slash_command(
        name: &str,
        description: Option<&str>,
    ) -> ServerProviderSlashCommand {
        ServerProviderSlashCommand {
            name: name.to_string(),
            description: description.map(str::to_string),
            input: None,
        }
    }

    #[test]
    fn shared_search_ranking_matches_upstream_contract() {
        assert_eq!(normalize_search_query("  UI  "), "ui");
        assert_eq!(normalize_search_query_trim_leading("  $ui", '$'), "ui");

        assert_eq!(
            score_query_match("ui", "ui", 0, Some(10), None, Some(20), None),
            Some(0)
        );
        assert!(
            score_query_match(
                "building native ui",
                "ui",
                0,
                Some(10),
                Some(20),
                Some(30),
                None
            )
            .unwrap()
                > 0
        );

        let boundary_score = score_query_match_with_boundary_markers(
            "gh-fix-ci",
            "fix",
            0,
            Some(10),
            Some(20),
            Some(30),
            None,
            &["-"],
        )
        .unwrap();
        let contains_score = score_query_match_with_boundary_markers(
            "highfixci",
            "fix",
            0,
            Some(10),
            Some(20),
            Some(30),
            None,
            &["-"],
        )
        .unwrap();
        assert!(boundary_score < contains_score);

        let compact = score_subsequence_match("ghfixci", "gfc").unwrap();
        let spread = score_subsequence_match("github-fix-ci", "gfc").unwrap();
        assert!(compact < spread);
    }

    #[test]
    fn provider_skill_presentation_and_search_match_upstream_contract() {
        let review = make_provider_skill("review-follow-up", |skill| {
            skill.display_name = Some("Review Follow-up".to_string());
        });
        assert_eq!(
            format_provider_skill_display_name(&review),
            "Review Follow-up"
        );
        assert_eq!(
            format_provider_skill_display_name(&make_provider_skill("review-follow-up", |_| {})),
            "Review Follow Up"
        );

        assert_eq!(
            format_provider_skill_install_source(&make_provider_skill("gh-fix-ci", |skill| {
                skill.path = "/Users/julius/.codex/plugins/cache/openai-curated/github/skills/gh-fix-ci/SKILL.md".to_string();
                skill.scope = Some("user".to_string());
            })),
            Some("App".to_string())
        );
        assert_eq!(
            format_provider_skill_install_source(&make_provider_skill("agent-browser", |skill| {
                skill.path = "/Users/julius/.agents/skills/agent-browser/SKILL.md".to_string();
                skill.scope = Some("user".to_string());
            })),
            Some("Personal".to_string())
        );
        assert_eq!(
            format_provider_skill_install_source(&make_provider_skill("imagegen", |skill| {
                skill.path = "/usr/local/share/codex/skills/imagegen/SKILL.md".to_string();
                skill.scope = Some("system".to_string());
            })),
            Some("System".to_string())
        );
        assert_eq!(
            format_provider_skill_install_source(&make_provider_skill(
                "review-follow-up",
                |skill| {
                    skill.path = "/workspace/.codex/skills/review-follow-up/SKILL.md".to_string();
                    skill.scope = Some("project".to_string());
                }
            )),
            Some("Project".to_string())
        );

        let skills = vec![
            make_provider_skill("agent-browser", |skill| {
                skill.display_name = Some("Agent Browser".to_string());
                skill.short_description = Some("Browser automation CLI for AI agents".to_string());
            }),
            make_provider_skill("building-native-ui", |skill| {
                skill.display_name = Some("Building Native Ui".to_string());
                skill.short_description =
                    Some("Complete guide for building beautiful apps with Expo Router".to_string());
            }),
            make_provider_skill("ui", |skill| {
                skill.display_name = Some("Ui".to_string());
                skill.short_description = Some("Explore, build, and refine UI.".to_string());
            }),
        ];
        assert_eq!(
            search_provider_skills(&skills, "ui")
                .iter()
                .map(|skill| skill.name.as_str())
                .collect::<Vec<_>>(),
            vec!["ui", "building-native-ui"]
        );

        let skills = vec![
            make_provider_skill("gh-fix-ci", |skill| {
                skill.display_name = Some("Gh Fix Ci".to_string());
            }),
            make_provider_skill("github", |skill| {
                skill.display_name = Some("Github".to_string());
            }),
            make_provider_skill("agent-browser", |skill| {
                skill.display_name = Some("Agent Browser".to_string());
            }),
        ];
        assert_eq!(
            search_provider_skills(&skills, "gfc")
                .iter()
                .map(|skill| skill.name.as_str())
                .collect::<Vec<_>>(),
            vec!["gh-fix-ci"]
        );

        let skills = vec![
            make_provider_skill("ui", |skill| {
                skill.display_name = Some("Ui".to_string());
                skill.enabled = false;
            }),
            make_provider_skill("frontend-design", |skill| {
                skill.display_name = Some("Frontend Design".to_string());
            }),
        ];
        assert!(search_provider_skills(&skills, "ui").is_empty());
    }

    #[test]
    fn composer_menu_item_derivation_matches_upstream_contract() {
        let provider = "claudeAgent";
        let slash_items = vec![
            make_provider_slash_command("ui", Some("Explore, build, and refine UI.")),
            make_provider_slash_command(
                "frontend-design",
                Some("Create distinctive, production-grade frontend interfaces"),
            ),
        ];
        let trigger = ComposerTrigger {
            kind: ComposerTriggerKind::SlashCommand,
            query: "ui".to_string(),
            range_start: 0,
            range_end: 3,
        };
        assert_eq!(
            build_composer_menu_items(Some(&trigger), &[], provider, &slash_items, &[])
                .iter()
                .map(ComposerCommandItem::id)
                .collect::<Vec<_>>(),
            vec!["provider-slash-command:claudeAgent:ui", "slash:default"]
        );

        let fuzzy_items = vec![
            make_provider_slash_command("gh-fix-ci", Some("Fix failing GitHub Actions")),
            make_provider_slash_command("github", Some("General GitHub help")),
        ];
        let trigger = ComposerTrigger {
            kind: ComposerTriggerKind::SlashCommand,
            query: "gfc".to_string(),
            range_start: 0,
            range_end: 4,
        };
        assert_eq!(
            build_composer_menu_items(Some(&trigger), &[], provider, &fuzzy_items, &[])
                .iter()
                .map(ComposerCommandItem::id)
                .collect::<Vec<_>>(),
            vec!["provider-slash-command:claudeAgent:gh-fix-ci"]
        );

        let trigger = ComposerTrigger {
            kind: ComposerTriggerKind::Path,
            query: "src".to_string(),
            range_start: 7,
            range_end: 11,
        };
        assert_eq!(
            build_composer_menu_items(
                Some(&trigger),
                &[ProjectEntry {
                    path: "src/main.rs".to_string(),
                    kind: ProjectEntryKind::File,
                    parent_path: Some("src".to_string()),
                }],
                provider,
                &[],
                &[],
            ),
            vec![ComposerCommandItem::Path {
                id: "path:file:src/main.rs".to_string(),
                path: "src/main.rs".to_string(),
                path_kind: ProjectEntryKind::File,
                label: "main.rs".to_string(),
                description: "src".to_string(),
            }]
        );

        let trigger = ComposerTrigger {
            kind: ComposerTriggerKind::Skill,
            query: "ui".to_string(),
            range_start: 4,
            range_end: 7,
        };
        let skills = vec![make_provider_skill("ui", |skill| {
            skill.display_name = Some("Ui".to_string());
            skill.short_description = Some("Explore, build, and refine UI.".to_string());
        })];
        assert_eq!(
            build_composer_menu_items(Some(&trigger), &[], provider, &[], &skills),
            vec![ComposerCommandItem::Skill {
                id: "skill:claudeAgent:ui".to_string(),
                provider: provider.to_string(),
                skill: skills[0].clone(),
                label: "Ui".to_string(),
                description: "Explore, build, and refine UI.".to_string(),
            }]
        );
    }

    #[test]
    fn composer_menu_grouping_highlight_and_selection_match_upstream_contract() {
        let built_in = builtin_composer_slash_command_items();
        let provider_item = ComposerCommandItem::ProviderSlashCommand {
            id: "provider-slash-command:claudeAgent:ui".to_string(),
            provider: "claudeAgent".to_string(),
            command: make_provider_slash_command("ui", Some("Explore, build, and refine UI.")),
            label: "/ui".to_string(),
            description: "Explore, build, and refine UI.".to_string(),
        };
        let mut items = built_in.clone();
        items.push(provider_item.clone());

        assert_eq!(
            group_composer_command_items(&items, Some(ComposerTriggerKind::SlashCommand), true),
            vec![
                ComposerCommandGroup {
                    id: "built-in".to_string(),
                    label: Some("Built-in".to_string()),
                    items: built_in,
                },
                ComposerCommandGroup {
                    id: "provider".to_string(),
                    label: Some("Provider".to_string()),
                    items: vec![provider_item],
                },
            ]
        );

        let skill_item = ComposerCommandItem::Skill {
            id: "skill:claudeAgent:ui".to_string(),
            provider: "claudeAgent".to_string(),
            skill: make_provider_skill("ui", |_| {}),
            label: "Ui".to_string(),
            description: "Run provider skill".to_string(),
        };
        assert_eq!(
            group_composer_command_items(
                std::slice::from_ref(&skill_item),
                Some(ComposerTriggerKind::Skill),
                true,
            ),
            vec![ComposerCommandGroup {
                id: "skills".to_string(),
                label: Some("Skills".to_string()),
                items: vec![skill_item],
            }]
        );

        let items = vec![
            ComposerCommandItem::SlashCommand {
                id: "top".to_string(),
                command: ComposerSlashCommand::Model,
                label: "/model".to_string(),
                description: "Switch response model for this thread".to_string(),
            },
            ComposerCommandItem::SlashCommand {
                id: "second".to_string(),
                command: ComposerSlashCommand::Plan,
                label: "/plan".to_string(),
                description: "Switch this thread into plan mode".to_string(),
            },
            ComposerCommandItem::SlashCommand {
                id: "third".to_string(),
                command: ComposerSlashCommand::Default,
                label: "/default".to_string(),
                description: "Switch this thread back to normal build mode".to_string(),
            },
        ];
        assert_eq!(
            resolve_composer_menu_active_item_id(&items, None, Some("skill:u"), None),
            Some("top".to_string())
        );
        assert_eq!(
            resolve_composer_menu_active_item_id(
                &items,
                Some("second"),
                Some("skill:u"),
                Some("skill:u"),
            ),
            Some("second".to_string())
        );
        assert_eq!(
            resolve_composer_menu_active_item_id(
                &items,
                Some("second"),
                Some("skill:ui"),
                Some("skill:u"),
            ),
            Some("top".to_string())
        );
        assert_eq!(
            resolve_composer_menu_active_item_id(
                &items,
                Some("missing"),
                Some("skill:ui"),
                Some("skill:ui"),
            ),
            Some("top".to_string())
        );
        assert_eq!(
            nudge_composer_menu_highlight(
                &items,
                Some("top"),
                ComposerMenuNudgeDirection::ArrowDown
            ),
            Some("second".to_string())
        );
        assert_eq!(
            nudge_composer_menu_highlight(&items, Some("top"), ComposerMenuNudgeDirection::ArrowUp),
            Some("third".to_string())
        );
    }

    #[test]
    fn composer_command_selection_matches_upstream_replacement_contract() {
        let trigger = ComposerTrigger {
            kind: ComposerTriggerKind::Path,
            query: "AG".to_string(),
            range_start: "and then ".chars().count(),
            range_end: "and then @AG".chars().count(),
        };
        let path_item = ComposerCommandItem::Path {
            id: "path:file:AGENTS.md".to_string(),
            path: "AGENTS.md".to_string(),
            path_kind: ProjectEntryKind::File,
            label: "AGENTS.md".to_string(),
            description: String::new(),
        };
        assert_eq!(
            resolve_composer_command_selection("and then @AG summarize", &trigger, &path_item),
            Some(ComposerCommandSelection {
                range_start: "and then ".chars().count(),
                range_end: "and then @AG ".chars().count(),
                replacement: "@AGENTS.md ".to_string(),
                interaction_mode: None,
                open_model_picker: false,
                focus_editor_after_replace: true,
            })
        );

        let model_item = ComposerCommandItem::SlashCommand {
            id: "slash:model".to_string(),
            command: ComposerSlashCommand::Model,
            label: "/model".to_string(),
            description: "Switch response model for this thread".to_string(),
        };
        let slash_trigger = ComposerTrigger {
            kind: ComposerTriggerKind::SlashCommand,
            query: "model".to_string(),
            range_start: 0,
            range_end: "/model".chars().count(),
        };
        assert_eq!(
            resolve_composer_command_selection("/model", &slash_trigger, &model_item),
            Some(ComposerCommandSelection {
                range_start: 0,
                range_end: "/model".chars().count(),
                replacement: String::new(),
                interaction_mode: None,
                open_model_picker: true,
                focus_editor_after_replace: false,
            })
        );

        let plan_item = ComposerCommandItem::SlashCommand {
            id: "slash:plan".to_string(),
            command: ComposerSlashCommand::Plan,
            label: "/plan".to_string(),
            description: "Switch this thread into plan mode".to_string(),
        };
        let plan_trigger = ComposerTrigger {
            kind: ComposerTriggerKind::SlashCommand,
            query: "plan".to_string(),
            range_start: 0,
            range_end: "/plan".chars().count(),
        };
        assert_eq!(
            resolve_composer_command_selection("/plan", &plan_trigger, &plan_item)
                .unwrap()
                .interaction_mode,
            Some(ComposerSlashCommand::Plan)
        );

        let provider_item = ComposerCommandItem::ProviderSlashCommand {
            id: "provider-slash-command:claudeAgent:ui".to_string(),
            provider: "claudeAgent".to_string(),
            command: make_provider_slash_command("ui", Some("Explore, build, and refine UI.")),
            label: "/ui".to_string(),
            description: "Explore, build, and refine UI.".to_string(),
        };
        assert_eq!(
            resolve_composer_command_selection("/u now", &slash_trigger, &provider_item)
                .unwrap()
                .replacement,
            "/ui ".to_string()
        );

        let skill_item = ComposerCommandItem::Skill {
            id: "skill:claudeAgent:review-follow-up".to_string(),
            provider: "claudeAgent".to_string(),
            skill: make_provider_skill("review-follow-up", |_| {}),
            label: "Review Follow Up".to_string(),
            description: "Run provider skill".to_string(),
        };
        let skill_trigger = ComposerTrigger {
            kind: ComposerTriggerKind::Skill,
            query: "review".to_string(),
            range_start: "run ".chars().count(),
            range_end: "run $review".chars().count(),
        };
        assert_eq!(
            resolve_composer_command_selection("run $review next", &skill_trigger, &skill_item)
                .unwrap()
                .replacement,
            "$review-follow-up ".to_string()
        );
    }

    #[test]
    fn terminal_drawer_reference_state_exposes_open_split_terminal() {
        let snapshot = AppSnapshot::terminal_drawer_reference_state();

        assert!(snapshot.terminal_open());
        assert_eq!(
            snapshot.terminal_state.terminal_ids,
            vec!["default", "terminal-2"]
        );
        assert_eq!(snapshot.terminal_state.active_terminal_id, "terminal-2");
        assert_eq!(
            snapshot.terminal_state.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );
        assert_eq!(snapshot.terminal_event_entries.len(), 4);
    }

    #[test]
    fn diff_route_search_matches_upstream_parser_contract() {
        assert_eq!(
            parse_diff_route_search(
                Some(DiffOpenValue::from("1")),
                Some("turn-1"),
                Some("src/app.ts")
            ),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: Some("src/app.ts".to_string()),
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::Number(1)), Some("turn-1"), None),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::Bool(true)), Some("turn-1"), None),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: Some("turn-1".to_string()),
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(
                Some(DiffOpenValue::from("0")),
                Some("turn-1"),
                Some("src/app.ts")
            ),
            DiffRouteSearch::default()
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::from("1")), None, Some("src/app.ts")),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: None,
                diff_file_path: None,
            }
        );
        assert_eq!(
            parse_diff_route_search(Some(DiffOpenValue::from("1")), Some("  "), Some("  ")),
            DiffRouteSearch {
                diff: Some("1".to_string()),
                diff_turn_id: None,
                diff_file_path: None,
            }
        );
    }

    #[test]
    fn turn_diff_stats_sum_only_files_with_numeric_values() {
        let stat = summarize_turn_diff_stats(&[
            diff_file("README.md", Some(3), Some(1)),
            diff_file("docs/notes.md", None, None),
            diff_file("src/index.ts", Some(5), Some(2)),
        ]);

        assert_eq!(
            stat,
            TurnDiffStat {
                additions: 8,
                deletions: 3,
            }
        );
        assert!(has_non_zero_turn_diff_stat(stat));
    }

    #[test]
    fn builds_turn_diff_tree_with_aggregated_directory_stats() {
        let tree = build_turn_diff_tree(&[
            diff_file("src/index.ts", Some(2), Some(1)),
            diff_file("src/components/Button.tsx", Some(4), Some(2)),
            diff_file("README.md", Some(1), Some(0)),
        ]);

        assert_eq!(
            tree,
            vec![
                TurnDiffTreeNode::Directory {
                    name: "src".to_string(),
                    path: "src".to_string(),
                    stat: TurnDiffStat {
                        additions: 6,
                        deletions: 3,
                    },
                    children: vec![
                        TurnDiffTreeNode::Directory {
                            name: "components".to_string(),
                            path: "src/components".to_string(),
                            stat: TurnDiffStat {
                                additions: 4,
                                deletions: 2,
                            },
                            children: vec![TurnDiffTreeNode::File {
                                name: "Button.tsx".to_string(),
                                path: "src/components/Button.tsx".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 4,
                                    deletions: 2,
                                }),
                            }],
                        },
                        TurnDiffTreeNode::File {
                            name: "index.ts".to_string(),
                            path: "src/index.ts".to_string(),
                            stat: Some(TurnDiffStat {
                                additions: 2,
                                deletions: 1,
                            }),
                        },
                    ],
                },
                TurnDiffTreeNode::File {
                    name: "README.md".to_string(),
                    path: "README.md".to_string(),
                    stat: Some(TurnDiffStat {
                        additions: 1,
                        deletions: 0,
                    }),
                },
            ]
        );
    }

    #[test]
    fn turn_diff_tree_keeps_missing_stats_and_normalizes_windows_paths() {
        let missing_stats = build_turn_diff_tree(&[
            diff_file("docs/notes.md", None, None),
            diff_file("docs/todo.md", Some(1), Some(1)),
        ]);
        assert_eq!(
            missing_stats,
            vec![TurnDiffTreeNode::Directory {
                name: "docs".to_string(),
                path: "docs".to_string(),
                stat: TurnDiffStat {
                    additions: 1,
                    deletions: 1,
                },
                children: vec![
                    TurnDiffTreeNode::File {
                        name: "notes.md".to_string(),
                        path: "docs/notes.md".to_string(),
                        stat: None,
                    },
                    TurnDiffTreeNode::File {
                        name: "todo.md".to_string(),
                        path: "docs/todo.md".to_string(),
                        stat: Some(TurnDiffStat {
                            additions: 1,
                            deletions: 1,
                        }),
                    },
                ],
            }]
        );

        assert_eq!(
            build_turn_diff_tree(&[diff_file("apps\\web\\src\\index.ts", Some(2), Some(1))]),
            vec![TurnDiffTreeNode::Directory {
                name: "apps/web/src".to_string(),
                path: "apps/web/src".to_string(),
                stat: TurnDiffStat {
                    additions: 2,
                    deletions: 1,
                },
                children: vec![TurnDiffTreeNode::File {
                    name: "index.ts".to_string(),
                    path: "apps/web/src/index.ts".to_string(),
                    stat: Some(TurnDiffStat {
                        additions: 2,
                        deletions: 1,
                    }),
                }],
            }]
        );
    }

    #[test]
    fn turn_diff_tree_compacts_directory_chains_and_sorts_numerically() {
        let tree = build_turn_diff_tree(&[
            diff_file("apps/server/src/file10.ts", Some(2), Some(1)),
            diff_file("apps/server/src/file2.ts", Some(4), Some(0)),
            diff_file("apps/server/main.ts", Some(1), Some(0)),
        ]);

        assert_eq!(
            tree,
            vec![TurnDiffTreeNode::Directory {
                name: "apps/server".to_string(),
                path: "apps/server".to_string(),
                stat: TurnDiffStat {
                    additions: 7,
                    deletions: 1,
                },
                children: vec![
                    TurnDiffTreeNode::Directory {
                        name: "src".to_string(),
                        path: "apps/server/src".to_string(),
                        stat: TurnDiffStat {
                            additions: 6,
                            deletions: 1,
                        },
                        children: vec![
                            TurnDiffTreeNode::File {
                                name: "file2.ts".to_string(),
                                path: "apps/server/src/file2.ts".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 4,
                                    deletions: 0,
                                }),
                            },
                            TurnDiffTreeNode::File {
                                name: "file10.ts".to_string(),
                                path: "apps/server/src/file10.ts".to_string(),
                                stat: Some(TurnDiffStat {
                                    additions: 2,
                                    deletions: 1,
                                }),
                            },
                        ],
                    },
                    TurnDiffTreeNode::File {
                        name: "main.ts".to_string(),
                        path: "apps/server/main.ts".to_string(),
                        stat: Some(TurnDiffStat {
                            additions: 1,
                            deletions: 0,
                        }),
                    },
                ],
            }]
        );
    }

    #[test]
    fn diff_panel_reference_state_exposes_selected_turn_and_file() {
        let snapshot = AppSnapshot::diff_panel_reference_state();
        let selected = snapshot.selected_turn_diff_summary().unwrap();

        assert!(snapshot.diff_open());
        assert_eq!(selected.turn_id, "turn-r3code-ui-shell-2");
        assert_eq!(
            snapshot.selected_diff_file_path(),
            Some("crates/r3_ui/src/shell.rs")
        );
        assert_eq!(
            snapshot.ordered_turn_diff_summaries()[0].turn_id,
            selected.turn_id
        );
        assert_eq!(
            summarize_turn_diff_stats(&selected.files),
            TurnDiffStat {
                additions: 208,
                deletions: 22,
            }
        );
    }

    #[test]
    fn default_terminal_state_matches_upstream_contract() {
        assert_eq!(
            create_default_thread_terminal_state(),
            ThreadTerminalState {
                terminal_open: false,
                terminal_height: DEFAULT_THREAD_TERMINAL_HEIGHT,
                terminal_ids: vec!["default".to_string()],
                running_terminal_ids: Vec::new(),
                active_terminal_id: "default".to_string(),
                terminal_groups: vec![ThreadTerminalGroup {
                    id: "group-default".to_string(),
                    terminal_ids: vec!["default".to_string()],
                }],
                active_terminal_group_id: "group-default".to_string(),
            }
        );
    }

    #[test]
    fn terminal_split_and_new_group_behaviors_match_upstream_store() {
        let state = create_default_thread_terminal_state();
        let split = split_thread_terminal(&state, "terminal-2");

        assert!(split.terminal_open);
        assert_eq!(split.terminal_ids, vec!["default", "terminal-2"]);
        assert_eq!(split.active_terminal_id, "terminal-2");
        assert_eq!(
            split.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );

        let separate = new_thread_terminal(&state, "terminal-2");
        assert_eq!(separate.active_terminal_id, "terminal-2");
        assert_eq!(separate.active_terminal_group_id, "group-terminal-2");
        assert_eq!(
            separate.terminal_groups,
            vec![
                ThreadTerminalGroup {
                    id: "group-default".to_string(),
                    terminal_ids: vec!["default".to_string()],
                },
                ThreadTerminalGroup {
                    id: "group-terminal-2".to_string(),
                    terminal_ids: vec!["terminal-2".to_string()],
                },
            ]
        );
    }

    #[test]
    fn terminal_split_caps_at_four_per_group() {
        let mut state = create_default_thread_terminal_state();
        for terminal_id in ["terminal-2", "terminal-3", "terminal-4", "terminal-5"] {
            state = split_thread_terminal(&state, terminal_id);
        }

        assert_eq!(
            state.terminal_ids,
            vec!["default", "terminal-2", "terminal-3", "terminal-4"]
        );
        assert_eq!(state.terminal_groups[0].terminal_ids.len(), 4);
    }

    #[test]
    fn terminal_close_keeps_valid_active_terminal() {
        let mut state = create_default_thread_terminal_state();
        state = split_thread_terminal(&state, "terminal-2");
        state = split_thread_terminal(&state, "terminal-3");
        state = close_thread_terminal(&state, "terminal-3");

        assert_eq!(state.active_terminal_id, "terminal-2");
        assert_eq!(state.terminal_ids, vec!["default", "terminal-2"]);
        assert_eq!(
            state.terminal_groups,
            vec![ThreadTerminalGroup {
                id: "group-default".to_string(),
                terminal_ids: vec!["default".to_string(), "terminal-2".to_string()],
            }]
        );
    }

    #[test]
    fn terminal_activity_and_event_filters_match_upstream_helpers() {
        let mut state =
            split_thread_terminal(&create_default_thread_terminal_state(), "terminal-2");
        state = set_thread_terminal_activity(&state, "terminal-2", true);
        assert_eq!(state.running_terminal_ids, vec!["terminal-2"]);
        state = set_thread_terminal_activity(&state, "terminal-2", false);
        assert_eq!(state.running_terminal_ids, Vec::<String>::new());

        let output = TerminalEvent::Output {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:00.000Z".to_string(),
            data: "before".to_string(),
        };
        let activity = TerminalEvent::Activity {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:01.000Z".to_string(),
            has_running_subprocess: true,
        };
        let exited = TerminalEvent::Exited {
            thread_id: "thread-1".to_string(),
            terminal_id: "default".to_string(),
            created_at: "2026-04-02T20:00:02.000Z".to_string(),
            exit_code: Some(0),
            exit_signal: None,
        };
        assert_eq!(terminal_running_subprocess_from_event(&output), None);
        assert_eq!(
            terminal_running_subprocess_from_event(&activity),
            Some(true)
        );
        assert_eq!(terminal_running_subprocess_from_event(&exited), Some(false));

        let entries = vec![
            TerminalEventEntry {
                id: 1,
                event: output,
            },
            TerminalEventEntry {
                id: 2,
                event: activity,
            },
            TerminalEventEntry {
                id: 3,
                event: exited,
            },
        ];
        assert_eq!(
            select_terminal_event_entries_after_snapshot(&entries, "2026-04-02T20:00:00.500Z")
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(
            select_pending_terminal_event_entries(&entries, 1)
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn derives_thread_from_environment_state_in_id_order() {
        let thread_id = "thread-browser-test";
        let mut state = EnvironmentState::default();
        state.thread_shell_by_id.insert(
            thread_id.to_string(),
            ThreadShell {
                id: thread_id.to_string(),
                environment_id: "environment-local".to_string(),
                codex_thread_id: None,
                project_id: "project-1".to_string(),
                title: "Browser test thread".to_string(),
                runtime_mode: RuntimeMode::FullAccess,
                interaction_mode: ProviderInteractionMode::Default,
                error: None,
                created_at: "2026-03-04T12:00:00.000Z".to_string(),
                archived_at: None,
                updated_at: Some("2026-03-04T12:00:03.000Z".to_string()),
                branch: Some("main".to_string()),
                worktree_path: None,
            },
        );
        state.thread_session_by_id.insert(
            thread_id.to_string(),
            ThreadSession {
                provider: "codex".to_string(),
                provider_instance_id: Some("codex".to_string()),
                status: SessionPhase::Ready,
                active_turn_id: None,
                created_at: "2026-03-04T12:00:00.000Z".to_string(),
                updated_at: "2026-03-04T12:00:03.000Z".to_string(),
                last_error: None,
                orchestration_status: "ready".to_string(),
            },
        );
        state.message_ids_by_thread_id.insert(
            thread_id.to_string(),
            vec![
                "msg-user".to_string(),
                "msg-missing".to_string(),
                "msg-assistant".to_string(),
            ],
        );
        state.message_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([
                (
                    "msg-assistant".to_string(),
                    ChatMessage::assistant(
                        "msg-assistant",
                        "assistant filler 0",
                        "2026-03-04T12:00:03.000Z",
                    ),
                ),
                (
                    "msg-user".to_string(),
                    ChatMessage::user("msg-user", "bootstrap", "2026-03-04T12:00:00.000Z"),
                ),
            ]),
        );
        state.activity_ids_by_thread_id.insert(
            thread_id.to_string(),
            vec!["activity-1".to_string(), "activity-2".to_string()],
        );
        state.activity_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([
                (
                    "activity-1".to_string(),
                    ThreadActivity {
                        id: "activity-1".to_string(),
                        kind: "tool.started".to_string(),
                        summary: "Read file".to_string(),
                        tone: ActivityTone::Tool,
                        payload: ActivityPayload::default(),
                        turn_id: Some("turn-1".to_string()),
                        sequence: Some(1),
                        created_at: "2026-03-04T12:00:01.000Z".to_string(),
                    },
                ),
                (
                    "activity-2".to_string(),
                    ThreadActivity {
                        id: "activity-2".to_string(),
                        kind: "tool.completed".to_string(),
                        summary: "Read file".to_string(),
                        tone: ActivityTone::Tool,
                        payload: ActivityPayload::default(),
                        turn_id: Some("turn-1".to_string()),
                        sequence: Some(2),
                        created_at: "2026-03-04T12:00:02.000Z".to_string(),
                    },
                ),
            ]),
        );
        state
            .turn_diff_ids_by_thread_id
            .insert(thread_id.to_string(), vec!["turn-1".to_string()]);
        state.turn_diff_summary_by_thread_id.insert(
            thread_id.to_string(),
            BTreeMap::from([(
                "turn-1".to_string(),
                TurnDiffSummary {
                    turn_id: "turn-1".to_string(),
                    completed_at: "2026-03-04T12:00:04.000Z".to_string(),
                    status: Some("completed".to_string()),
                    files: vec![TurnDiffFileChange {
                        path: "apps/web/src/components/chat/MessagesTimeline.tsx".to_string(),
                        kind: Some("modified".to_string()),
                        additions: Some(4),
                        deletions: Some(1),
                    }],
                    checkpoint_ref: None,
                    assistant_message_id: Some("msg-assistant".to_string()),
                    checkpoint_turn_count: Some(1),
                },
            )]),
        );

        let thread = get_thread_from_environment_state(&state, thread_id).unwrap();

        assert_eq!(thread.shell.title, "Browser test thread");
        assert_eq!(thread.session.unwrap().status, SessionPhase::Ready);
        assert_eq!(
            thread
                .messages
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["msg-user", "msg-assistant"]
        );
        assert_eq!(thread.messages[0].role, MessageRole::User);
        assert_eq!(thread.activities.len(), 2);
        assert_eq!(thread.turn_diff_summaries[0].files[0].additions, Some(4));
    }

    #[test]
    fn missing_thread_shell_returns_none() {
        let state = EnvironmentState::default();

        assert!(get_thread_from_environment_state(&state, "missing-thread").is_none());
    }

    fn vcs_ref(name: &str, is_default: bool, worktree_path: Option<&str>) -> VcsRef {
        VcsRef {
            name: name.to_string(),
            current: false,
            is_default,
            is_remote: false,
            remote_name: None,
            worktree_path: worktree_path.map(str::to_string),
        }
    }

    fn remote_vcs_ref(name: &str, remote_name: &str) -> VcsRef {
        VcsRef {
            name: name.to_string(),
            current: false,
            is_default: false,
            is_remote: true,
            remote_name: Some(remote_name.to_string()),
            worktree_path: None,
        }
    }

    fn diff_file(path: &str, additions: Option<u32>, deletions: Option<u32>) -> TurnDiffFileChange {
        TurnDiffFileChange {
            path: path.to_string(),
            kind: Some("modified".to_string()),
            additions,
            deletions,
        }
    }

    #[test]
    fn message_roles_expose_upstream_display_authors() {
        assert_eq!(MessageRole::User.display_author(), "You");
        assert_eq!(MessageRole::Assistant.display_author(), APP_NAME);
        assert_eq!(MessageRole::System.display_author(), "System");
    }

    fn activity(
        id: &str,
        kind: &str,
        created_at: &str,
        request_id: Option<&str>,
        payload: ActivityPayload,
    ) -> ThreadActivity {
        ThreadActivity {
            id: id.to_string(),
            kind: kind.to_string(),
            summary: kind.to_string(),
            tone: ActivityTone::Info,
            payload: ActivityPayload {
                request_id: request_id.map(str::to_string),
                ..payload
            },
            turn_id: None,
            sequence: None,
            created_at: created_at.to_string(),
        }
    }

    fn user_input_question(id: &str) -> UserInputQuestion {
        UserInputQuestion {
            id: id.to_string(),
            header: "Scope".to_string(),
            question: "What should this change cover?".to_string(),
            options: vec![UserInputQuestionOption {
                label: "Tight".to_string(),
                description: "Touch only the footer layout logic.".to_string(),
            }],
            multi_select: false,
        }
    }

    fn multi_select_question(id: &str) -> UserInputQuestion {
        UserInputQuestion {
            id: id.to_string(),
            header: "Areas".to_string(),
            question: "Which areas should this change cover?".to_string(),
            options: vec![
                UserInputQuestionOption {
                    label: "Server".to_string(),
                    description: "Server".to_string(),
                },
                UserInputQuestionOption {
                    label: "Web".to_string(),
                    description: "Web".to_string(),
                },
            ],
            multi_select: true,
        }
    }

    #[test]
    fn derives_pending_approvals_and_removes_resolved_requests() {
        let activities = vec![
            activity(
                "approval-open",
                "approval.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-1"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::Command),
                    detail: Some("bun run lint".to_string()),
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "approval-close",
                "approval.resolved",
                "2026-02-23T00:00:02.000Z",
                Some("req-2"),
                ActivityPayload::default(),
            ),
            activity(
                "approval-closed-request",
                "approval.requested",
                "2026-02-23T00:00:01.500Z",
                Some("req-2"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::FileChange),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(
            derive_pending_approvals(&activities),
            vec![PendingApproval {
                request_id: "req-1".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                detail: Some("bun run lint".to_string()),
            }]
        );
    }

    #[test]
    fn derives_pending_approvals_from_canonical_request_type() {
        let activities = vec![activity(
            "approval-open-request-type",
            "approval.requested",
            "2026-02-23T00:00:01.000Z",
            Some("req-request-type"),
            ActivityPayload {
                request_type: Some("command_execution_approval".to_string()),
                detail: Some("pwd".to_string()),
                ..ActivityPayload::default()
            },
        )];

        assert_eq!(
            derive_pending_approvals(&activities),
            vec![PendingApproval {
                request_id: "req-request-type".to_string(),
                request_kind: ApprovalRequestKind::Command,
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                detail: Some("pwd".to_string()),
            }]
        );
    }

    #[test]
    fn stale_provider_approval_failure_clears_pending_request() {
        let activities = vec![
            activity(
                "approval-open-stale",
                "approval.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-stale-1"),
                ActivityPayload {
                    request_kind: Some(ApprovalRequestKind::Command),
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "approval-failed-stale",
                "provider.approval.respond.failed",
                "2026-02-23T00:00:02.000Z",
                Some("req-stale-1"),
                ActivityPayload {
                    detail: Some("Unknown pending permission request: req-stale-1".to_string()),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(derive_pending_approvals(&activities), Vec::new());
    }

    #[test]
    fn derives_pending_user_inputs_and_removes_resolved_requests() {
        let activities = vec![
            activity(
                "user-input-open",
                "user-input.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-user-input-1"),
                ActivityPayload {
                    questions: vec![user_input_question("sandbox_mode")],
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "user-input-resolved",
                "user-input.resolved",
                "2026-02-23T00:00:02.000Z",
                Some("req-user-input-2"),
                ActivityPayload::default(),
            ),
            activity(
                "user-input-open-2",
                "user-input.requested",
                "2026-02-23T00:00:01.500Z",
                Some("req-user-input-2"),
                ActivityPayload {
                    questions: vec![user_input_question("approval")],
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(
            derive_pending_user_inputs(&activities),
            vec![PendingUserInput {
                request_id: "req-user-input-1".to_string(),
                created_at: "2026-02-23T00:00:01.000Z".to_string(),
                questions: vec![user_input_question("sandbox_mode")],
            }]
        );
    }

    #[test]
    fn stale_provider_user_input_failure_clears_pending_request() {
        let activities = vec![
            activity(
                "user-input-open-stale",
                "user-input.requested",
                "2026-02-23T00:00:01.000Z",
                Some("req-user-input-stale-1"),
                ActivityPayload {
                    questions: vec![user_input_question("sandbox_mode")],
                    ..ActivityPayload::default()
                },
            ),
            activity(
                "user-input-failed-stale",
                "provider.user-input.respond.failed",
                "2026-02-23T00:00:02.000Z",
                Some("req-user-input-stale-1"),
                ActivityPayload {
                    detail: Some(
                        "Stale pending user-input request: req-user-input-stale-1".to_string(),
                    ),
                    ..ActivityPayload::default()
                },
            ),
        ];

        assert_eq!(derive_pending_user_inputs(&activities), Vec::new());
    }

    #[test]
    fn pending_user_input_answer_prefers_custom_text() {
        let question = user_input_question("scope");
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Tight".to_string()],
            custom_answer: Some("Keep the existing envelope for one release".to_string()),
        };

        assert_eq!(
            resolve_pending_user_input_answer(&question, Some(&draft)),
            Some(PendingUserInputAnswer::Text(
                "Keep the existing envelope for one release".to_string()
            ))
        );
    }

    #[test]
    fn pending_user_input_answer_returns_multi_select_arrays() {
        let question = multi_select_question("areas");
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
            custom_answer: None,
        };

        assert_eq!(
            resolve_pending_user_input_answer(&question, Some(&draft)),
            Some(PendingUserInputAnswer::Multiple(vec![
                "Server".to_string(),
                "Web".to_string(),
            ]))
        );
    }

    #[test]
    fn setting_custom_answer_clears_selected_options_when_non_empty() {
        let draft = PendingUserInputDraftAnswer {
            selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
            custom_answer: None,
        };

        assert_eq!(
            set_pending_user_input_custom_answer(Some(&draft), "doesn't matter"),
            PendingUserInputDraftAnswer {
                selected_option_labels: Vec::new(),
                custom_answer: Some("doesn't matter".to_string()),
            }
        );
    }

    #[test]
    fn toggling_pending_user_input_options_matches_select_mode() {
        let multi = multi_select_question("areas");
        let selected = toggle_pending_user_input_option_selection(&multi, None, "Server");
        assert_eq!(
            selected,
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Server".to_string()],
                custom_answer: Some(String::new()),
            }
        );

        let removed = toggle_pending_user_input_option_selection(
            &multi,
            Some(&PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Server".to_string(), "Web".to_string()],
                custom_answer: None,
            }),
            "Server",
        );
        assert_eq!(
            removed,
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Web".to_string()],
                custom_answer: Some(String::new()),
            }
        );

        let single = user_input_question("scope");
        assert_eq!(
            toggle_pending_user_input_option_selection(&single, None, "Tight"),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: Some(String::new()),
            }
        );
    }

    #[test]
    fn builds_pending_user_input_answer_map_only_when_complete() {
        let scope = user_input_question("scope");
        let compat = user_input_question("compat");
        let mut answers = BTreeMap::new();
        answers.insert(
            "scope".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: None,
            },
        );

        assert_eq!(
            build_pending_user_input_answers(&[scope.clone(), compat.clone()], &answers),
            None
        );

        answers.insert(
            "compat".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: Vec::new(),
                custom_answer: Some("Keep the current envelope for one release window".to_string()),
            },
        );

        assert_eq!(
            build_pending_user_input_answers(&[scope, compat], &answers),
            Some(BTreeMap::from([
                (
                    "compat".to_string(),
                    PendingUserInputAnswer::Text(
                        "Keep the current envelope for one release window".to_string()
                    ),
                ),
                (
                    "scope".to_string(),
                    PendingUserInputAnswer::Text("Tight".to_string()),
                ),
            ]))
        );
    }

    #[test]
    fn derives_pending_user_input_question_progress() {
        let questions = vec![user_input_question("scope"), user_input_question("compat")];
        let draft_answers = BTreeMap::from([(
            "scope".to_string(),
            PendingUserInputDraftAnswer {
                selected_option_labels: vec!["Tight".to_string()],
                custom_answer: None,
            },
        )]);

        let progress = derive_pending_user_input_progress(&questions, &draft_answers, 0);

        assert_eq!(progress.question_index, 0);
        assert_eq!(progress.active_question, Some(questions[0].clone()));
        assert_eq!(progress.selected_option_labels, vec!["Tight"]);
        assert_eq!(
            progress.resolved_answer,
            Some(PendingUserInputAnswer::Text("Tight".to_string()))
        );
        assert_eq!(progress.answered_question_count, 1);
        assert!(!progress.is_last_question);
        assert!(!progress.is_complete);
        assert!(progress.can_advance);
        assert_eq!(
            find_first_unanswered_pending_user_input_question_index(&questions, &draft_answers),
            1
        );
    }

    #[test]
    fn completed_pending_user_input_progress_uses_last_question_index() {
        let questions = vec![user_input_question("scope"), user_input_question("compat")];
        let draft_answers = BTreeMap::from([
            (
                "scope".to_string(),
                PendingUserInputDraftAnswer {
                    selected_option_labels: vec!["Tight".to_string()],
                    custom_answer: None,
                },
            ),
            (
                "compat".to_string(),
                PendingUserInputDraftAnswer {
                    selected_option_labels: Vec::new(),
                    custom_answer: Some("Keep it for one release window".to_string()),
                },
            ),
        ]);

        assert_eq!(
            find_first_unanswered_pending_user_input_question_index(&questions, &draft_answers),
            1
        );
        assert_eq!(
            count_answered_pending_user_input_questions(&questions, &draft_answers),
            2
        );

        let progress = derive_pending_user_input_progress(&questions, &draft_answers, 9);

        assert_eq!(progress.question_index, 1);
        assert!(progress.is_last_question);
        assert!(progress.is_complete);
    }
}

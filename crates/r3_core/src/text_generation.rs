use crate::ChatAttachment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextGenerationPolicyKind {
    Default,
    ConventionalCommits,
    RepoConventions,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextGenerationPolicy {
    pub kind: TextGenerationPolicyKind,
    pub commit_instructions: Option<String>,
    pub change_request_instructions: Option<String>,
    pub branch_instructions: Option<String>,
    pub thread_title_instructions: Option<String>,
    pub infer_repository_conventions: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessagePromptInput {
    pub branch: Option<String>,
    pub staged_summary: String,
    pub staged_patch: String,
    pub include_branch: bool,
    pub policy: Option<TextGenerationPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrContentPromptInput {
    pub base_branch: String,
    pub head_branch: String,
    pub commit_summary: String,
    pub diff_summary: String,
    pub diff_patch: String,
    pub policy: Option<TextGenerationPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchNamePromptInput {
    pub message: String,
    pub attachments: Vec<ChatAttachment>,
    pub policy: Option<TextGenerationPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadTitlePromptInput {
    pub message: String,
    pub attachments: Vec<ChatAttachment>,
    pub policy: Option<TextGenerationPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextGenerationPrompt {
    pub prompt: String,
    pub output_keys: Vec<&'static str>,
}

pub fn default_text_generation_policy() -> TextGenerationPolicy {
    TextGenerationPolicy {
        kind: TextGenerationPolicyKind::Default,
        commit_instructions: None,
        change_request_instructions: None,
        branch_instructions: None,
        thread_title_instructions: None,
        infer_repository_conventions: false,
    }
}

pub fn conventional_commits_text_generation_policy() -> TextGenerationPolicy {
    TextGenerationPolicy {
        kind: TextGenerationPolicyKind::ConventionalCommits,
        commit_instructions: Some(
            "Use Conventional Commits when generating commit subjects. Prefer the narrowest accurate type and include a scope only when it is obvious from the diff.".to_string(),
        ),
        change_request_instructions: Some(
            "Keep the change request title concise. Do not force Conventional Commit syntax into the title unless the repository already uses it.".to_string(),
        ),
        branch_instructions: None,
        thread_title_instructions: None,
        infer_repository_conventions: false,
    }
}

pub fn repository_conventions_text_generation_policy() -> TextGenerationPolicy {
    TextGenerationPolicy {
        kind: TextGenerationPolicyKind::RepoConventions,
        commit_instructions: Some(
            "Follow the repository's established commit message style when examples are available."
                .to_string(),
        ),
        change_request_instructions: Some(
            "Follow the repository's established change request title and body style when examples are available.".to_string(),
        ),
        branch_instructions: None,
        thread_title_instructions: None,
        infer_repository_conventions: true,
    }
}

pub fn custom_text_generation_policy(overrides: TextGenerationPolicy) -> TextGenerationPolicy {
    TextGenerationPolicy {
        kind: TextGenerationPolicyKind::Custom,
        infer_repository_conventions: false,
        ..overrides
    }
}

pub fn build_commit_message_prompt(input: &CommitMessagePromptInput) -> TextGenerationPrompt {
    let mut lines = vec![
        "You write concise git commit messages.".to_string(),
        if input.include_branch {
            "Return a JSON object with keys: subject, body, branch.".to_string()
        } else {
            "Return a JSON object with keys: subject, body.".to_string()
        },
        "Rules:".to_string(),
        "- subject must be imperative, <= 72 chars, and no trailing period".to_string(),
        "- body can be empty string or short bullet points".to_string(),
    ];
    if input.include_branch {
        lines.push(
            "- branch must be a short semantic git branch fragment for this change".to_string(),
        );
    }
    lines.push("- capture the primary user-visible or developer-visible change".to_string());
    lines.extend(policy_instruction(
        input
            .policy
            .as_ref()
            .and_then(|policy| policy.commit_instructions.as_deref()),
    ));
    lines.extend([
        String::new(),
        format!(
            "Branch: {}",
            input.branch.as_deref().unwrap_or("(detached)")
        ),
        String::new(),
        "Staged files:".to_string(),
        limit_section(&input.staged_summary, 6_000),
        String::new(),
        "Staged patch:".to_string(),
        limit_section(&input.staged_patch, 40_000),
    ]);

    TextGenerationPrompt {
        prompt: lines.join("\n"),
        output_keys: if input.include_branch {
            vec!["subject", "body", "branch"]
        } else {
            vec!["subject", "body"]
        },
    }
}

pub fn build_pr_content_prompt(input: &PrContentPromptInput) -> TextGenerationPrompt {
    let mut lines = vec![
        "You write GitHub pull request content.".to_string(),
        "Return a JSON object with keys: title, body.".to_string(),
        "Rules:".to_string(),
        "- title should be concise and specific".to_string(),
        "- body must be markdown and include headings '## Summary' and '## Testing'".to_string(),
        "- under Summary, provide short bullet points".to_string(),
        "- under Testing, include bullet points with concrete checks or 'Not run' where appropriate".to_string(),
    ];
    lines.extend(policy_instruction(
        input
            .policy
            .as_ref()
            .and_then(|policy| policy.change_request_instructions.as_deref()),
    ));
    lines.extend([
        String::new(),
        format!("Base branch: {}", input.base_branch),
        format!("Head branch: {}", input.head_branch),
        String::new(),
        "Commits:".to_string(),
        limit_section(&input.commit_summary, 12_000),
        String::new(),
        "Diff stat:".to_string(),
        limit_section(&input.diff_summary, 12_000),
        String::new(),
        "Diff patch:".to_string(),
        limit_section(&input.diff_patch, 40_000),
    ]);

    TextGenerationPrompt {
        prompt: lines.join("\n"),
        output_keys: vec!["title", "body"],
    }
}

pub fn build_branch_name_prompt(input: &BranchNamePromptInput) -> TextGenerationPrompt {
    TextGenerationPrompt {
        prompt: build_prompt_from_message(PromptFromMessageInput {
            instruction: "You generate concise git branch names.",
            response_shape: "Return a JSON object with key: branch.",
            rules: &[
                "Branch should describe the requested work from the user message.",
                "Keep it short and specific (2-6 words).",
                "Use plain words only, no issue prefixes and no punctuation-heavy text.",
                "If images are attached, use them as primary context for visual/UI issues.",
            ],
            message: &input.message,
            attachments: &input.attachments,
            additional_instructions: input
                .policy
                .as_ref()
                .and_then(|policy| policy.branch_instructions.as_deref()),
        }),
        output_keys: vec!["branch"],
    }
}

pub fn build_thread_title_prompt(input: &ThreadTitlePromptInput) -> TextGenerationPrompt {
    TextGenerationPrompt {
        prompt: build_prompt_from_message(PromptFromMessageInput {
            instruction: "You write concise thread titles for coding conversations.",
            response_shape: "Return a JSON object with key: title.",
            rules: &[
                "Title should summarize the user's request, not restate it verbatim.",
                "Keep it short and specific (3-8 words).",
                "Avoid quotes, filler, prefixes, and trailing punctuation.",
                "If images are attached, use them as primary context for visual/UI issues.",
            ],
            message: &input.message,
            attachments: &input.attachments,
            additional_instructions: input
                .policy
                .as_ref()
                .and_then(|policy| policy.thread_title_instructions.as_deref()),
        }),
        output_keys: vec!["title"],
    }
}

pub fn limit_section(value: &str, max_chars: usize) -> String {
    if value.len() <= max_chars {
        return value.to_string();
    }
    format!("{}\n\n[truncated]", &value[..max_chars])
}

pub fn sanitize_commit_subject(raw: &str) -> String {
    let single_line = raw.trim().lines().next().unwrap_or("").trim();
    let without_trailing_period = single_line.trim_end_matches('.').trim();
    if without_trailing_period.is_empty() {
        return "Update project files".to_string();
    }
    if without_trailing_period.len() <= 72 {
        without_trailing_period.to_string()
    } else {
        without_trailing_period[..72].trim_end().to_string()
    }
}

pub fn sanitize_pr_title(raw: &str) -> String {
    let single_line = raw.trim().lines().next().unwrap_or("").trim();
    if single_line.is_empty() {
        "Update project changes".to_string()
    } else {
        single_line.to_string()
    }
}

pub fn sanitize_thread_title(raw: &str) -> String {
    let normalized = raw
        .trim()
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(['\'', '"', '`'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        return "New thread".to_string();
    }
    if normalized.len() <= 50 {
        normalized
    } else {
        format!("{}...", normalized[..47].trim_end())
    }
}

struct PromptFromMessageInput<'a> {
    instruction: &'a str,
    response_shape: &'a str,
    rules: &'a [&'a str],
    message: &'a str,
    attachments: &'a [ChatAttachment],
    additional_instructions: Option<&'a str>,
}

fn build_prompt_from_message(input: PromptFromMessageInput<'_>) -> String {
    let mut sections = vec![
        input.instruction.to_string(),
        input.response_shape.to_string(),
        "Rules:".to_string(),
    ];
    sections.extend(input.rules.iter().map(|rule| format!("- {rule}")));
    sections.extend([
        String::new(),
        "User message:".to_string(),
        limit_section(input.message, 8_000),
    ]);
    sections.extend(policy_instruction(input.additional_instructions));

    let attachment_lines = input
        .attachments
        .iter()
        .map(|attachment| match attachment {
            ChatAttachment::Image(image) => {
                format!(
                    "- {} ({}, {} bytes)",
                    image.name, image.mime_type, image.size_bytes
                )
            }
        })
        .collect::<Vec<_>>();
    if !attachment_lines.is_empty() {
        sections.extend([
            String::new(),
            "Attachment metadata:".to_string(),
            limit_section(&attachment_lines.join("\n"), 4_000),
        ]);
    }

    sections.join("\n")
}

fn policy_instruction(instruction: Option<&str>) -> Vec<String> {
    let Some(trimmed) = instruction.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };
    vec![
        String::new(),
        "Additional instructions:".to_string(),
        limit_section(trimmed, 4_000),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChatImageAttachment;

    #[test]
    fn ports_text_generation_policy_presets() {
        assert_eq!(
            default_text_generation_policy().kind,
            TextGenerationPolicyKind::Default
        );
        assert!(!default_text_generation_policy().infer_repository_conventions);
        assert!(
            conventional_commits_text_generation_policy()
                .commit_instructions
                .unwrap()
                .contains("Conventional Commits")
        );
        assert!(repository_conventions_text_generation_policy().infer_repository_conventions);
    }

    #[test]
    fn builds_commit_message_prompt_with_branch_schema_and_limits() {
        let prompt = build_commit_message_prompt(&CommitMessagePromptInput {
            branch: Some("main".to_string()),
            staged_summary: "src/main.rs | 2 +".to_string(),
            staged_patch: "x".repeat(40_010),
            include_branch: true,
            policy: Some(conventional_commits_text_generation_policy()),
        });

        assert_eq!(prompt.output_keys, vec!["subject", "body", "branch"]);
        assert!(
            prompt
                .prompt
                .contains("Return a JSON object with keys: subject, body, branch.")
        );
        assert!(prompt.prompt.contains("Branch: main"));
        assert!(prompt.prompt.contains("Additional instructions:"));
        assert!(prompt.prompt.contains("[truncated]"));
    }

    #[test]
    fn builds_pr_content_prompt_with_required_headings() {
        let prompt = build_pr_content_prompt(&PrContentPromptInput {
            base_branch: "main".to_string(),
            head_branch: "feature/work".to_string(),
            commit_summary: "abc Add thing".to_string(),
            diff_summary: "1 file changed".to_string(),
            diff_patch: "diff --git".to_string(),
            policy: None,
        });

        assert_eq!(prompt.output_keys, vec!["title", "body"]);
        assert!(prompt.prompt.contains("## Summary"));
        assert!(prompt.prompt.contains("Base branch: main"));
        assert!(prompt.prompt.contains("Head branch: feature/work"));
    }

    #[test]
    fn builds_message_prompts_with_attachment_metadata() {
        let attachment = ChatAttachment::Image(ChatImageAttachment {
            id: "img-1".to_string(),
            name: "screen.png".to_string(),
            mime_type: "image/png".to_string(),
            size_bytes: 42,
            preview_url: None,
        });

        let branch = build_branch_name_prompt(&BranchNamePromptInput {
            message: "Fix the layout".to_string(),
            attachments: vec![attachment.clone()],
            policy: None,
        });
        let title = build_thread_title_prompt(&ThreadTitlePromptInput {
            message: "Fix the layout".to_string(),
            attachments: vec![attachment],
            policy: None,
        });

        assert_eq!(branch.output_keys, vec!["branch"]);
        assert!(branch.prompt.contains("- screen.png (image/png, 42 bytes)"));
        assert_eq!(title.output_keys, vec!["title"]);
        assert!(title.prompt.contains("You write concise thread titles"));
    }

    #[test]
    fn sanitizes_generated_text_like_upstream_utils() {
        assert_eq!(sanitize_commit_subject(" Add thing.\nbody"), "Add thing");
        assert_eq!(sanitize_commit_subject(""), "Update project files");
        assert_eq!(sanitize_pr_title("\n"), "Update project changes");
        assert_eq!(
            sanitize_thread_title("  `Fix   the   settings panel`  \nmore"),
            "Fix the settings panel"
        );
        assert_eq!(
            sanitize_thread_title("a".repeat(80).as_str()),
            format!("{}...", "a".repeat(47))
        );
    }
}

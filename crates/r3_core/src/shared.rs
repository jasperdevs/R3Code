use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    net::{TcpListener, ToSocketAddrs},
};

use crate::VcsRef;
use qrcodegen::{
    Mask as NayukiQrMask, QrCode as NayukiQrCode, QrCodeEcc as NayukiQrCodeEcc,
    QrSegment as NayukiQrSegment, QrSegmentMode as NayukiQrSegmentMode, Version as NayukiQrVersion,
};
use serde_json::Value;

pub fn truncate_text(text: &str, max_length: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_length {
        return trimmed.to_string();
    }
    format!(
        "{}...",
        trimmed.chars().take(max_length).collect::<String>()
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCliArgs {
    pub flags: BTreeMap<String, Option<String>>,
    pub positionals: Vec<String>,
}

pub fn parse_cli_args_from_str(args: &str, boolean_flags: &[&str]) -> ParsedCliArgs {
    let tokens = args
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    parse_cli_args(tokens, boolean_flags)
}

pub fn parse_cli_args(
    tokens: impl IntoIterator<Item = impl Into<String>>,
    boolean_flags: &[&str],
) -> ParsedCliArgs {
    let tokens = tokens.into_iter().map(Into::into).collect::<Vec<_>>();
    let boolean_flags = boolean_flags.iter().copied().collect::<BTreeSet<_>>();
    let mut flags = BTreeMap::new();
    let mut positionals = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(rest) = token.strip_prefix("--") {
            if rest.is_empty() {
                index += 1;
                continue;
            }
            if let Some(eq_index) = rest.find('=') {
                flags.insert(
                    rest[..eq_index].to_string(),
                    Some(rest[eq_index + 1..].to_string()),
                );
                index += 1;
                continue;
            }
            if boolean_flags.contains(rest) {
                flags.insert(rest.to_string(), None);
                index += 1;
                continue;
            }
            if let Some(next) = tokens.get(index + 1) {
                if !next.starts_with("--") {
                    flags.insert(rest.to_string(), Some(next.clone()));
                    index += 2;
                    continue;
                }
            }
            flags.insert(rest.to_string(), None);
        } else {
            positionals.push(token.clone());
        }
        index += 1;
    }

    ParsedCliArgs { flags, positionals }
}

pub fn is_windows_drive_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2
        && bytes[1] == b':'
        && bytes[0].is_ascii_alphabetic()
        && (bytes.len() == 2 || matches!(bytes[2], b'/' | b'\\'))
}

pub fn is_unc_path(value: &str) -> bool {
    value.starts_with("\\\\")
}

pub fn is_windows_absolute_path(value: &str) -> bool {
    is_unc_path(value) || is_windows_drive_path(value)
}

pub fn is_explicit_relative_path(value: &str) -> bool {
    matches!(value, "." | "..")
        || value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with(".\\")
        || value.starts_with("..\\")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSemver {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Vec<String>,
}

pub fn normalize_semver_version(version: &str) -> String {
    let trimmed = version.trim();
    let (main, prerelease) = trimmed
        .split_once('-')
        .map(|(main, prerelease)| (main, Some(prerelease)))
        .unwrap_or((trimmed, None));
    let mut segments = main
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments.len() == 2 {
        segments.push("0".to_string());
    }
    let normalized = segments.join(".");
    match prerelease {
        Some(prerelease) => format!("{normalized}-{prerelease}"),
        None => normalized,
    }
}

pub fn parse_semver(value: &str) -> Option<ParsedSemver> {
    let normalized = normalize_semver_version(value)
        .strip_prefix('v')
        .map(str::to_string)
        .unwrap_or_else(|| normalize_semver_version(value));
    let (main, prerelease) = normalized
        .split_once('-')
        .map(|(main, prerelease)| (main, Some(prerelease)))
        .unwrap_or((normalized.as_str(), None));
    let segments = main.split('.').collect::<Vec<_>>();
    if segments.len() != 3 || segments.iter().any(|segment| !is_number_segment(segment)) {
        return None;
    }
    Some(ParsedSemver {
        major: segments[0].parse().ok()?,
        minor: segments[1].parse().ok()?,
        patch: segments[2].parse().ok()?,
        prerelease: prerelease
            .map(|value| {
                value
                    .split('.')
                    .map(str::trim)
                    .filter(|segment| !segment.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn compare_semver_versions(left: &str, right: &str) -> Ordering {
    let Some(parsed_left) = parse_semver(left) else {
        return left.cmp(right);
    };
    let Some(parsed_right) = parse_semver(right) else {
        return left.cmp(right);
    };

    parsed_left
        .major
        .cmp(&parsed_right.major)
        .then_with(|| parsed_left.minor.cmp(&parsed_right.minor))
        .then_with(|| parsed_left.patch.cmp(&parsed_right.patch))
        .then_with(|| compare_prerelease(&parsed_left.prerelease, &parsed_right.prerelease))
}

pub fn satisfies_semver_range(raw_version: &str, range: &str) -> bool {
    let Some(version) = parse_range_version(raw_version) else {
        return false;
    };
    range.split("||").any(|group| {
        let comparators = group.split_whitespace().collect::<Vec<_>>();
        !comparators.is_empty()
            && comparators
                .iter()
                .all(|comparator| satisfies_comparator(version, comparator))
    })
}

fn is_number_segment(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

fn compare_prerelease(left: &[String], right: &[String]) -> Ordering {
    if left.is_empty() && right.is_empty() {
        return Ordering::Equal;
    }
    if left.is_empty() {
        return Ordering::Greater;
    }
    if right.is_empty() {
        return Ordering::Less;
    }
    let length = left.len().max(right.len());
    for index in 0..length {
        match (left.get(index), right.get(index)) {
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(left), Some(right)) => {
                let comparison = compare_prerelease_identifier(left, right);
                if comparison != Ordering::Equal {
                    return comparison;
                }
            }
            (None, None) => return Ordering::Equal,
        }
    }
    Ordering::Equal
}

fn compare_prerelease_identifier(left: &str, right: &str) -> Ordering {
    match (is_number_segment(left), is_number_segment(right)) {
        (true, true) => left
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&right.parse::<u64>().unwrap_or(0)),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.cmp(right),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RangeVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

fn parse_range_version(value: &str) -> Option<RangeVersion> {
    let normalized = value.trim().trim_start_matches('v');
    let main = normalized
        .split_once('-')
        .map(|(main, _)| main)
        .unwrap_or(normalized);
    let segments = main.split('.').collect::<Vec<_>>();
    if segments.is_empty()
        || segments.len() > 3
        || segments.iter().any(|segment| !is_number_segment(segment))
    {
        return None;
    }
    Some(RangeVersion {
        major: segments[0].parse().ok()?,
        minor: segments.get(1).copied().unwrap_or("0").parse().ok()?,
        patch: segments.get(2).copied().unwrap_or("0").parse().ok()?,
    })
}

fn parse_comparator(comparator: &str) -> Option<(&str, RangeVersion)> {
    let trimmed = comparator.trim();
    let (operator, version) = ["^", ">=", ">", "<=", "<", "="]
        .iter()
        .find_map(|operator| {
            trimmed
                .strip_prefix(operator)
                .map(|rest| (*operator, rest.trim()))
        })
        .unwrap_or(("=", trimmed));
    let version = version.trim_start_matches('v');
    Some((operator, parse_range_version(version)?))
}

fn satisfies_comparator(version: RangeVersion, comparator: &str) -> bool {
    let Some((operator, target)) = parse_comparator(comparator) else {
        return false;
    };
    match operator {
        "^" => {
            version >= target
                && if target.major > 0 {
                    version.major == target.major
                } else if target.minor > 0 {
                    version.major == 0 && version.minor == target.minor
                } else {
                    version.major == 0 && version.minor == 0 && version.patch == target.patch
                }
        }
        ">=" => version >= target,
        ">" => version > target,
        "<=" => version <= target,
        "<" => version < target,
        "=" => version == target,
        _ => false,
    }
}

pub const WORKTREE_BRANCH_PREFIX: &str = "t3code";

pub fn sanitize_branch_fragment(raw: &str) -> String {
    let normalized = raw
        .trim()
        .to_ascii_lowercase()
        .replace(['\'', '"', '`'], "");
    let normalized = trim_ref_separators(&normalized);
    let mut fragment = String::new();
    let mut previous_dash = false;
    let mut previous_slash = false;

    for character in normalized.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            fragment.push(character);
            previous_dash = false;
            previous_slash = false;
        } else if character == '/' {
            if !previous_slash {
                fragment.push('/');
            }
            previous_dash = false;
            previous_slash = true;
        } else if character == '-' {
            if !previous_dash {
                fragment.push('-');
            }
            previous_dash = true;
            previous_slash = false;
        } else if !previous_dash {
            fragment.push('-');
            previous_dash = true;
            previous_slash = false;
        }
    }

    let fragment = trim_ref_separators(fragment.chars().take(64).collect::<String>().as_str());
    if fragment.is_empty() {
        "update".to_string()
    } else {
        fragment
    }
}

pub fn sanitize_feature_branch_name(raw: &str) -> String {
    let sanitized = sanitize_branch_fragment(raw);
    if sanitized.contains('/') {
        if sanitized.starts_with("feature/") {
            sanitized
        } else {
            format!("feature/{sanitized}")
        }
    } else {
        format!("feature/{sanitized}")
    }
}

pub fn resolve_auto_feature_branch_name(
    existing_branch_names: &[String],
    preferred_branch: Option<&str>,
) -> String {
    let preferred = preferred_branch
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
        .unwrap_or("feature/update");
    let base = sanitize_feature_branch_name(preferred);
    let existing = existing_branch_names
        .iter()
        .map(|branch| branch.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    if !existing.contains(&base) {
        return base;
    }
    let mut suffix = 2u32;
    while existing.contains(&format!("{base}-{suffix}")) {
        suffix += 1;
    }
    format!("{base}-{suffix}")
}

pub fn derive_local_branch_name_from_remote_ref(branch_name: &str) -> String {
    match branch_name.find('/') {
        Some(index) if index > 0 && index < branch_name.len() - 1 => {
            branch_name[index + 1..].to_string()
        }
        _ => branch_name.to_string(),
    }
}

pub fn build_temporary_worktree_branch_name(token: &str) -> String {
    let normalized = token
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase();
    format!("{WORKTREE_BRANCH_PREFIX}/{normalized:0<8}")
}

pub fn is_temporary_worktree_branch(ref_name: &str) -> bool {
    let trimmed = ref_name.trim().to_ascii_lowercase();
    let Some(token) = trimmed.strip_prefix(&format!("{WORKTREE_BRANCH_PREFIX}/")) else {
        return false;
    };
    token.len() == 8 && token.chars().all(|character| character.is_ascii_hexdigit())
}

pub fn normalize_git_remote_url(value: &str) -> String {
    let normalized = value
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_ascii_lowercase();
    if let Some((host, path)) = parse_url_remote(&normalized) {
        if path.contains('/') {
            return format!("{host}/{path}");
        }
    }
    if let Some((host, path)) = parse_scp_like_remote(&normalized) {
        if path.contains('/') {
            return format!("{host}/{path}");
        }
    }
    normalized
}

pub fn parse_github_repository_name_with_owner_from_remote_url(
    url: Option<&str>,
) -> Option<String> {
    let trimmed = url?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("git://github.com/") {
        rest
    } else {
        return None;
    };
    let repository = path.trim_end_matches('/').trim_end_matches(".git");
    (repository.split('/').count() == 2 && !repository.is_empty()).then(|| repository.to_string())
}

pub fn dedupe_remote_branches_with_local_matches(refs: &[VcsRef]) -> Vec<VcsRef> {
    let local_branch_names = refs
        .iter()
        .filter(|reference| !reference.is_remote)
        .map(|reference| reference.name.clone())
        .collect::<BTreeSet<_>>();
    refs.iter()
        .filter(|reference| {
            if !reference.is_remote || reference.remote_name.as_deref() != Some("origin") {
                return true;
            }
            let mut candidates = BTreeSet::new();
            candidates.insert(derive_local_branch_name_from_remote_ref(&reference.name));
            if let Some(remote_name) = &reference.remote_name {
                let prefix = format!("{remote_name}/");
                if let Some(candidate) = reference.name.strip_prefix(&prefix) {
                    if !candidate.is_empty() {
                        candidates.insert(candidate.to_string());
                    }
                }
            }
            !candidates
                .iter()
                .any(|candidate| local_branch_names.contains(candidate))
        })
        .cloned()
        .collect()
}

fn trim_ref_separators(value: &str) -> String {
    value
        .trim_matches(|character: char| {
            matches!(character, '.' | '/' | '_' | '-' | ' ' | '\t' | '\n' | '\r')
        })
        .to_string()
}

fn parse_url_remote(value: &str) -> Option<(String, String)> {
    let (_, rest) = value.split_once("://")?;
    let (authority, path) = rest.split_once('/')?;
    let host_with_port = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    let host = host_with_port
        .split_once(':')
        .map(|(host, _)| host)
        .unwrap_or(host_with_port);
    let repository_path = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    (!host.is_empty() && !repository_path.is_empty()).then(|| (host.to_string(), repository_path))
}

fn parse_scp_like_remote(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix("git@")?;
    let separator = rest.find([':', '/'])?;
    let host = &rest[..separator];
    let path = rest[separator + 1..]
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    (!host.is_empty() && !path.is_empty()).then(|| (host.to_string(), path))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedSourceControlProviderKind {
    Github,
    Gitlab,
    AzureDevops,
    Bitbucket,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSourceControlProviderInfo {
    pub kind: SharedSourceControlProviderKind,
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRequestPresentation {
    pub icon: &'static str,
    pub provider_name: &'static str,
    pub short_name: &'static str,
    pub long_name: &'static str,
    pub plural_long_name: &'static str,
    pub provider_long_name: &'static str,
    pub checkout_command_example: Option<&'static str>,
    pub url_example: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRequestTerminology {
    pub short_label: &'static str,
    pub singular: &'static str,
}

pub const DEFAULT_CHANGE_REQUEST_TERMINOLOGY: ChangeRequestTerminology = ChangeRequestTerminology {
    short_label: "PR",
    singular: "pull request",
};

pub fn resolve_change_request_presentation(
    provider: Option<&SharedSourceControlProviderInfo>,
) -> ChangeRequestPresentation {
    match provider.map(|provider| provider.kind) {
        Some(SharedSourceControlProviderKind::Gitlab) => ChangeRequestPresentation {
            icon: "gitlab",
            provider_name: "GitLab",
            short_name: "MR",
            long_name: "merge request",
            plural_long_name: "merge requests",
            provider_long_name: "GitLab merge request",
            checkout_command_example: Some("glab mr checkout 123"),
            url_example: "https://gitlab.com/group/project/-/merge_requests/42",
        },
        Some(SharedSourceControlProviderKind::AzureDevops) => ChangeRequestPresentation {
            icon: "azure-devops",
            provider_name: "Azure DevOps",
            short_name: "PR",
            long_name: "pull request",
            plural_long_name: "pull requests",
            provider_long_name: "Azure DevOps pull request",
            checkout_command_example: Some("az repos pr checkout --id 123"),
            url_example: "https://dev.azure.com/org/project/_git/repo/pullrequest/42",
        },
        Some(SharedSourceControlProviderKind::Bitbucket) => ChangeRequestPresentation {
            icon: "bitbucket",
            provider_name: "Bitbucket",
            short_name: "PR",
            long_name: "pull request",
            plural_long_name: "pull requests",
            provider_long_name: "Bitbucket pull request",
            checkout_command_example: None,
            url_example: "https://bitbucket.org/workspace/repo/pull-requests/42",
        },
        Some(SharedSourceControlProviderKind::Unknown) => ChangeRequestPresentation {
            icon: "change-request",
            provider_name: "source control",
            short_name: "change request",
            long_name: "change request",
            plural_long_name: "change requests",
            provider_long_name: "change request",
            checkout_command_example: None,
            url_example: "#42",
        },
        Some(SharedSourceControlProviderKind::Github) | None => ChangeRequestPresentation {
            icon: "github",
            provider_name: "GitHub",
            short_name: "PR",
            long_name: "pull request",
            plural_long_name: "pull requests",
            provider_long_name: "GitHub pull request",
            checkout_command_example: Some("gh pr checkout 123"),
            url_example: "https://github.com/owner/repo/pull/42",
        },
    }
}

pub fn resolve_change_request_presentation_for_kind(
    kind: SharedSourceControlProviderKind,
) -> ChangeRequestPresentation {
    resolve_change_request_presentation(Some(&SharedSourceControlProviderInfo {
        kind,
        name: String::new(),
        base_url: String::new(),
    }))
}

pub fn format_change_request_action(
    verb: &str,
    presentation: &ChangeRequestPresentation,
) -> String {
    format!("{verb} {}", presentation.short_name)
}

pub fn format_create_change_request_phrase(presentation: &ChangeRequestPresentation) -> String {
    format!("create {}", presentation.short_name)
}

pub fn get_change_request_terminology(
    provider: Option<&SharedSourceControlProviderInfo>,
) -> ChangeRequestTerminology {
    let presentation = resolve_change_request_presentation(provider);
    ChangeRequestTerminology {
        short_label: presentation.short_name,
        singular: presentation.long_name,
    }
}

pub fn get_change_request_terminology_for_kind(
    kind: SharedSourceControlProviderKind,
) -> ChangeRequestTerminology {
    let presentation = resolve_change_request_presentation_for_kind(kind);
    ChangeRequestTerminology {
        short_label: presentation.short_name,
        singular: presentation.long_name,
    }
}

pub fn detect_source_control_provider_from_remote_url(
    remote_url: &str,
) -> Option<SharedSourceControlProviderInfo> {
    let host = parse_remote_host(remote_url)?;
    let (kind, name) = if host == "github.com" || host.contains("github") {
        (
            SharedSourceControlProviderKind::Github,
            if host == "github.com" {
                "GitHub"
            } else {
                "GitHub Self-Hosted"
            },
        )
    } else if host == "gitlab.com" || host.contains("gitlab") {
        (
            SharedSourceControlProviderKind::Gitlab,
            if host == "gitlab.com" {
                "GitLab"
            } else {
                "GitLab Self-Hosted"
            },
        )
    } else if host == "dev.azure.com" || host.ends_with(".visualstudio.com") {
        (SharedSourceControlProviderKind::AzureDevops, "Azure DevOps")
    } else if host == "bitbucket.org" || host.contains("bitbucket") {
        (
            SharedSourceControlProviderKind::Bitbucket,
            if host == "bitbucket.org" {
                "Bitbucket"
            } else {
                "Bitbucket Self-Hosted"
            },
        )
    } else {
        (SharedSourceControlProviderKind::Unknown, host.as_str())
    };
    Some(SharedSourceControlProviderInfo {
        kind,
        name: name.to_string(),
        base_url: format!("https://{host}"),
    })
}

fn parse_remote_host(remote_url: &str) -> Option<String> {
    let trimmed = remote_url.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("git@") {
        let separator = rest.find([':', '/'])?;
        if separator == 0 {
            return None;
        }
        return Some(rest[..separator].to_ascii_lowercase());
    }
    parse_url_remote(&trimmed.to_ascii_lowercase()).map(|(host, _)| host)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVcsWorkingTreeFile {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVcsWorkingTreeSummary {
    pub files: Vec<SharedVcsWorkingTreeFile>,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVcsStatusLocalResult {
    pub is_repo: bool,
    pub source_control_provider: Option<SharedSourceControlProviderInfo>,
    pub has_primary_remote: bool,
    pub is_default_ref: bool,
    pub ref_name: Option<String>,
    pub has_working_tree_changes: bool,
    pub working_tree: SharedVcsWorkingTreeSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVcsStatusRemoteResult {
    pub has_upstream: bool,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub ahead_of_default_count: Option<u32>,
    pub pr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedVcsStatusResult {
    pub local: SharedVcsStatusLocalResult,
    pub remote: SharedVcsStatusRemoteResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedVcsStatusStreamEvent {
    Snapshot {
        local: SharedVcsStatusLocalResult,
        remote: SharedVcsStatusRemoteResult,
    },
    LocalUpdated {
        local: SharedVcsStatusLocalResult,
    },
    RemoteUpdated {
        remote: SharedVcsStatusRemoteResult,
    },
}

pub fn merge_git_status_parts(
    local: SharedVcsStatusLocalResult,
    remote: Option<SharedVcsStatusRemoteResult>,
) -> SharedVcsStatusResult {
    SharedVcsStatusResult {
        local,
        remote: remote.unwrap_or(SharedVcsStatusRemoteResult {
            has_upstream: false,
            ahead_count: 0,
            behind_count: 0,
            ahead_of_default_count: Some(0),
            pr: None,
        }),
    }
}

pub fn apply_git_status_stream_event(
    current: Option<&SharedVcsStatusResult>,
    event: SharedVcsStatusStreamEvent,
) -> SharedVcsStatusResult {
    match event {
        SharedVcsStatusStreamEvent::Snapshot { local, remote } => {
            merge_git_status_parts(local, Some(remote))
        }
        SharedVcsStatusStreamEvent::LocalUpdated { local } => {
            merge_git_status_parts(local, current.map(|status| status.remote.clone()))
        }
        SharedVcsStatusStreamEvent::RemoteUpdated { remote } => {
            let local =
                current
                    .map(|status| status.local.clone())
                    .unwrap_or(SharedVcsStatusLocalResult {
                        is_repo: true,
                        source_control_provider: None,
                        has_primary_remote: false,
                        is_default_ref: false,
                        ref_name: None,
                        has_working_tree_changes: false,
                        working_tree: SharedVcsWorkingTreeSummary {
                            files: Vec::new(),
                            insertions: 0,
                            deletions: 0,
                        },
                    });
            merge_git_status_parts(local, Some(remote))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetError {
    pub message: String,
    pub cause: Option<String>,
}

pub fn can_listen_on_host(port: u16, host: &str) -> bool {
    match TcpListener::bind((host, port)) {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(error) if error.kind() == io::ErrorKind::AddrNotAvailable => true,
        Err(_) => false,
    }
}

pub fn is_port_available_on_loopback(port: u16) -> bool {
    can_listen_on_host(port, "127.0.0.1") && can_listen_on_host(port, "::1")
}

pub fn reserve_loopback_port(host: Option<&str>) -> Result<u16, NetError> {
    let host = host.unwrap_or("127.0.0.1");
    let listener = TcpListener::bind((host, 0)).map_err(|cause| NetError {
        message: "Failed to reserve loopback port".to_string(),
        cause: Some(cause.to_string()),
    })?;
    let port = listener.local_addr().map_err(|cause| NetError {
        message: "Failed to reserve loopback port".to_string(),
        cause: Some(cause.to_string()),
    })?;
    drop(listener);
    Ok(port.port())
}

pub fn find_available_port(preferred: u16) -> Result<u16, NetError> {
    try_reserve_port(preferred).or_else(|_| try_reserve_port(0))
}

pub fn net_service_operation_names() -> Vec<&'static str> {
    vec![
        "canListenOnHost",
        "isPortAvailableOnLoopback",
        "reserveLoopbackPort",
        "findAvailablePort",
    ]
}

fn try_reserve_port(port: u16) -> Result<u16, NetError> {
    let addr = ("127.0.0.1", port)
        .to_socket_addrs()
        .map_err(|cause| NetError {
            message: "Could not find an available port.".to_string(),
            cause: Some(cause.to_string()),
        })?
        .next()
        .ok_or_else(|| NetError {
            message: "Could not find an available port.".to_string(),
            cause: None,
        })?;
    let listener = TcpListener::bind(addr).map_err(|cause| NetError {
        message: "Could not find an available port.".to_string(),
        cause: Some(cause.to_string()),
    })?;
    let resolved = listener.local_addr().map_err(|cause| NetError {
        message: "Could not find an available port.".to_string(),
        cause: Some(cause.to_string()),
    })?;
    drop(listener);
    Ok(resolved.port())
}

pub fn deep_merge_json(current: &Value, patch: &Value) -> Value {
    let (Some(current), Some(patch)) = (current.as_object(), patch.as_object()) else {
        return patch.clone();
    };
    let mut next = current.clone();
    for (key, value) in patch {
        let merged = match next.get(key) {
            Some(existing) if existing.is_object() && value.is_object() => {
                deep_merge_json(existing, value)
            }
            _ => value.clone(),
        };
        next.insert(key.clone(), merged);
    }
    Value::Object(next)
}

pub fn strip_lenient_json_comments_and_trailing_commas(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaping = false;

    while let Some(character) = chars.next() {
        if in_string {
            output.push(character);
            if escaping {
                escaping = false;
            } else if character == '\\' {
                escaping = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
            output.push(character);
            continue;
        }

        if character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }

        if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = '\0';
            for next in chars.by_ref() {
                if previous == '*' && next == '/' {
                    break;
                }
                previous = next;
            }
            continue;
        }

        output.push(character);
    }

    strip_trailing_json_commas(&output)
}

pub fn parse_lenient_json(input: &str) -> serde_json::Result<Value> {
    serde_json::from_str(&strip_lenient_json_comments_and_trailing_commas(input))
}

pub fn decode_json_result(input: &str) -> Result<Value, String> {
    serde_json::from_str(input).map_err(|error| error.to_string())
}

pub fn decode_unknown_json_result(input: &Value) -> Result<Value, String> {
    match input {
        Value::String(raw) => decode_json_result(raw),
        _ => Err("Expected JSON string input".to_string()),
    }
}

pub fn decode_lenient_json_result(input: &str) -> Result<Value, String> {
    parse_lenient_json(input).map_err(|error| error.to_string())
}

pub fn encode_json_pretty(value: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(&sort_json_object_keys(value)).map_err(|error| error.to_string())
}

pub fn pretty_json_string(input: &str) -> Result<String, String> {
    decode_json_result(input).and_then(|value| encode_json_pretty(&value))
}

fn sort_json_object_keys(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut sorted = serde_json::Map::new();
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                if let Some(value) = object.get(key) {
                    sorted.insert(key.clone(), sort_json_object_keys(value));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.iter().map(sort_json_object_keys).collect()),
        _ => value.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaJsonTransformationPlan {
    pub strict_decode: &'static str,
    pub unknown_decode: &'static str,
    pub lenient_decode: &'static str,
    pub lenient_encode: &'static str,
    pub pretty_decode: &'static str,
    pub pretty_encode: &'static str,
    pub error_formatter: &'static str,
}

pub fn schema_json_transformation_plan() -> SchemaJsonTransformationPlan {
    SchemaJsonTransformationPlan {
        strict_decode: "Schema.decodeExit(Schema.fromJsonString(schema))",
        unknown_decode: "Schema.decodeUnknownExit(Schema.fromJsonString(schema))",
        lenient_decode: "SchemaGetter.onSome parses JSON after stripping line comments, block comments, and trailing commas",
        lenient_encode: "SchemaGetter.stringifyJson()",
        pretty_decode: "SchemaGetter.parseJson<string>()",
        pretty_encode: "SchemaGetter.stringifyJson({ space: 2 })",
        error_formatter: "Cause.squash schema errors use SchemaIssue.makeFormatterDefault, otherwise Cause.pretty",
    }
}

pub fn extract_json_object(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }
    let Some(start) = trimmed.find('{') else {
        return trimmed.to_string();
    };

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaping = false;
    for (index, character) in trimmed[start..].char_indices() {
        if in_string {
            if escaping {
                escaping = false;
            } else if character == '\\' {
                escaping = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
            continue;
        }
        if character == '{' {
            depth += 1;
            continue;
        }
        if character == '}' {
            depth -= 1;
            if depth == 0 {
                let end = start + index + character.len_utf8();
                return trimmed[start..end].to_string();
            }
        }
    }

    trimmed[start..].to_string()
}

fn strip_trailing_json_commas(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaping = false;

    while let Some(character) = chars.next() {
        if in_string {
            output.push(character);
            if escaping {
                escaping = false;
            } else if character == '\\' {
                escaping = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
            output.push(character);
            continue;
        }

        if character == ',' {
            let mut lookahead = chars.clone();
            while matches!(lookahead.peek(), Some(next) if next.is_whitespace()) {
                lookahead.next();
            }
            if matches!(lookahead.peek(), Some('}' | ']')) {
                continue;
            }
        }

        output.push(character);
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedServerObservabilitySettings {
    pub otlp_traces_url: Option<String>,
    pub otlp_metrics_url: Option<String>,
}

pub fn normalize_persisted_server_setting_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn extract_persisted_server_observability_settings(
    input: &Value,
) -> PersistedServerObservabilitySettings {
    let observability = input.get("observability").and_then(Value::as_object);
    PersistedServerObservabilitySettings {
        otlp_traces_url: observability
            .and_then(|value| value.get("otlpTracesUrl"))
            .and_then(Value::as_str)
            .and_then(|value| normalize_persisted_server_setting_string(Some(value))),
        otlp_metrics_url: observability
            .and_then(|value| value.get("otlpMetricsUrl"))
            .and_then(Value::as_str)
            .and_then(|value| normalize_persisted_server_setting_string(Some(value))),
    }
}

pub fn parse_persisted_server_observability_settings(
    raw: &str,
) -> PersistedServerObservabilitySettings {
    parse_lenient_json(raw)
        .map(|value| extract_persisted_server_observability_settings(&value))
        .unwrap_or(PersistedServerObservabilitySettings {
            otlp_traces_url: None,
            otlp_metrics_url: None,
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedModelSelectionPatch {
    pub instance_id: Option<String>,
    pub model: Option<String>,
    pub options: Option<Vec<ProviderOptionSelection>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedProviderOptionValue {
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderOptionSelection {
    pub id: String,
    pub value: SharedProviderOptionValue,
}

pub fn should_replace_text_generation_model_selection(
    patch: Option<&SharedModelSelectionPatch>,
) -> bool {
    patch
        .map(|patch| patch.instance_id.is_some() || patch.model.is_some())
        .unwrap_or(false)
}

pub fn merge_model_selection_options_by_id(
    current: Option<&[ProviderOptionSelection]>,
    patch: Option<&[ProviderOptionSelection]>,
) -> Option<Vec<ProviderOptionSelection>> {
    let Some(patch) = patch else {
        return current.map(|current| current.to_vec());
    };
    if patch.is_empty() {
        return None;
    }
    let mut merged = current
        .unwrap_or(&[])
        .iter()
        .map(|selection| (selection.id.clone(), selection.value.clone()))
        .collect::<BTreeMap<_, _>>();
    for selection in patch {
        merged.insert(selection.id.clone(), selection.value.clone());
    }
    Some(
        merged
            .into_iter()
            .map(|(id, value)| ProviderOptionSelection { id, value })
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainableWorkerState<T> {
    queue: VecDeque<T>,
    outstanding: usize,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrainableWorkerRuntimePlan {
    pub queue_constructor: &'static str,
    pub scope_finalizer: &'static str,
    pub outstanding_ref: &'static str,
    pub worker_fiber: &'static str,
    pub enqueue_steps: Vec<&'static str>,
    pub process_finalizer: &'static str,
    pub drain_condition: &'static str,
}

impl<T> Default for DrainableWorkerState<T> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            outstanding: 0,
            active: false,
        }
    }
}

impl<T> DrainableWorkerState<T> {
    pub fn enqueue(&mut self, item: T) {
        self.queue.push_back(item);
        self.outstanding += 1;
    }

    pub fn take_next(&mut self) -> Option<T> {
        let item = self.queue.pop_front()?;
        self.active = true;
        Some(item)
    }

    pub fn finish_active(&mut self) {
        self.active = false;
        self.outstanding = self.outstanding.saturating_sub(1);
    }

    pub fn can_drain(&self) -> bool {
        self.outstanding == 0 && !self.active && self.queue.is_empty()
    }

    pub fn outstanding(&self) -> usize {
        self.outstanding
    }
}

pub fn drainable_worker_runtime_plan() -> DrainableWorkerRuntimePlan {
    DrainableWorkerRuntimePlan {
        queue_constructor: "TxQueue.unbounded",
        scope_finalizer: "TxQueue.shutdown",
        outstanding_ref: "TxRef.make(0)",
        worker_fiber: "TxQueue.take -> process(item) -> Effect.forever -> Effect.forkScoped",
        enqueue_steps: vec![
            "TxQueue.offer(queue, element)",
            "TxRef.update(outstanding, n => n + 1)",
            "Effect.tx",
        ],
        process_finalizer: "Effect.ensuring(process(item), TxRef.update(outstanding, n => n - 1))",
        drain_condition: "TxRef.get(outstanding) retries transaction while n > 0",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedCoalescingWorkerState<K, V> {
    latest_by_key: BTreeMap<K, V>,
    queued_keys: BTreeSet<K>,
    active_keys: BTreeSet<K>,
    queue: VecDeque<K>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedCoalescingWorkerRuntimePlan {
    pub queue_constructor: &'static str,
    pub scope_finalizer: &'static str,
    pub state_ref: &'static str,
    pub state_fields: Vec<&'static str>,
    pub enqueue_steps: Vec<&'static str>,
    pub take_steps: Vec<&'static str>,
    pub success_steps: Vec<&'static str>,
    pub failure_cleanup_steps: Vec<&'static str>,
    pub drain_condition: &'static str,
}

impl<K: Ord, V> Default for KeyedCoalescingWorkerState<K, V> {
    fn default() -> Self {
        Self {
            latest_by_key: BTreeMap::new(),
            queued_keys: BTreeSet::new(),
            active_keys: BTreeSet::new(),
            queue: VecDeque::new(),
        }
    }
}

impl<K: Ord + Clone, V: Clone> KeyedCoalescingWorkerState<K, V> {
    pub fn enqueue(&mut self, key: K, value: V, merge: impl FnOnce(&V, V) -> V) -> bool {
        let next_value = match self.latest_by_key.get(&key) {
            Some(current) => merge(current, value),
            None => value,
        };
        self.latest_by_key.insert(key.clone(), next_value);
        if self.queued_keys.contains(&key) || self.active_keys.contains(&key) {
            return false;
        }
        self.queued_keys.insert(key.clone());
        self.queue.push_back(key);
        true
    }

    pub fn take_next(&mut self) -> Option<(K, V)> {
        while let Some(key) = self.queue.pop_front() {
            self.queued_keys.remove(&key);
            let Some(value) = self.latest_by_key.remove(&key) else {
                continue;
            };
            self.active_keys.insert(key.clone());
            return Some((key, value));
        }
        None
    }

    pub fn finish_success(&mut self, key: &K) -> Option<V> {
        if let Some(next_value) = self.latest_by_key.remove(key) {
            return Some(next_value);
        }
        self.active_keys.remove(key);
        None
    }

    pub fn finish_failure(&mut self, key: &K) -> bool {
        self.active_keys.remove(key);
        if self.latest_by_key.contains_key(key) && !self.queued_keys.contains(key) {
            self.queued_keys.insert(key.clone());
            self.queue.push_back(key.clone());
            return true;
        }
        false
    }

    pub fn can_drain_key(&self, key: &K) -> bool {
        !self.latest_by_key.contains_key(key)
            && !self.queued_keys.contains(key)
            && !self.active_keys.contains(key)
    }
}

pub fn keyed_coalescing_worker_runtime_plan() -> KeyedCoalescingWorkerRuntimePlan {
    KeyedCoalescingWorkerRuntimePlan {
        queue_constructor: "TxQueue.unbounded",
        scope_finalizer: "TxQueue.shutdown",
        state_ref: "TxRef.make({ latestByKey, queuedKeys, activeKeys })",
        state_fields: vec!["latestByKey", "queuedKeys", "activeKeys"],
        enqueue_steps: vec![
            "merge existing latest value for key when present",
            "do not offer duplicate queue item while key is queued or active",
            "add key to queuedKeys and TxQueue.offer(queue, key) only for new idle keys",
            "Effect.tx then Effect.asVoid",
        ],
        take_steps: vec![
            "TxQueue.take(queue)",
            "remove key from queuedKeys",
            "skip stale queue item when latestByKey has no value",
            "move latest value to activeKeys before processing",
        ],
        success_steps: vec![
            "after process success, read next latestByKey value for active key",
            "when no pending value exists, remove key from activeKeys",
            "when pending value exists, delete it and process same key recursively",
        ],
        failure_cleanup_steps: vec![
            "remove key from activeKeys",
            "if latestByKey still has key and queuedKeys does not, add queuedKeys entry",
            "offer key back to queue when cleanup requests requeue",
        ],
        drain_condition: "drainKey retries transaction while latestByKey, queuedKeys, or activeKeys contain key",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotatingFileSinkOptions {
    pub file_path: String,
    pub max_bytes: u64,
    pub max_files: u16,
    pub throw_on_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotatingFileSinkWriteDecision {
    pub skip_empty_chunk: bool,
    pub rotate_before_append: bool,
    pub append_bytes: u64,
    pub rotate_after_append: bool,
    pub next_current_size: u64,
}

pub fn validate_rotating_file_sink_options(
    options: &RotatingFileSinkOptions,
) -> Result<(), String> {
    if options.max_bytes < 1 {
        return Err(format!(
            "maxBytes must be >= 1 (received {})",
            options.max_bytes
        ));
    }
    if options.max_files < 1 {
        return Err(format!(
            "maxFiles must be >= 1 (received {})",
            options.max_files
        ));
    }
    Ok(())
}

pub fn rotating_file_sink_write_decision(
    current_size: u64,
    max_bytes: u64,
    chunk_bytes: u64,
) -> Result<RotatingFileSinkWriteDecision, String> {
    if max_bytes < 1 {
        return Err(format!("maxBytes must be >= 1 (received {max_bytes})"));
    }
    if chunk_bytes == 0 {
        return Ok(RotatingFileSinkWriteDecision {
            skip_empty_chunk: true,
            rotate_before_append: false,
            append_bytes: 0,
            rotate_after_append: false,
            next_current_size: current_size,
        });
    }

    let rotate_before_append =
        current_size > 0 && current_size.saturating_add(chunk_bytes) > max_bytes;
    let size_before_append = if rotate_before_append {
        0
    } else {
        current_size
    };
    let size_after_append = size_before_append.saturating_add(chunk_bytes);
    let rotate_after_append = size_after_append > max_bytes;

    Ok(RotatingFileSinkWriteDecision {
        skip_empty_chunk: false,
        rotate_before_append,
        append_bytes: chunk_bytes,
        rotate_after_append,
        next_current_size: if rotate_after_append {
            0
        } else {
            size_after_append
        },
    })
}

pub fn rotating_file_sink_backup_path(file_path: &str, index: u16) -> String {
    format!("{file_path}.{index}")
}

pub fn rotating_file_sink_rotation_order(file_path: &str, max_files: u16) -> Vec<(String, String)> {
    if max_files == 0 {
        return Vec::new();
    }
    let mut operations = (1..max_files)
        .rev()
        .map(|index| {
            (
                rotating_file_sink_backup_path(file_path, index),
                rotating_file_sink_backup_path(file_path, index + 1),
            )
        })
        .collect::<Vec<_>>();
    operations.push((
        file_path.to_string(),
        rotating_file_sink_backup_path(file_path, 1),
    ));
    operations
}

pub fn rotating_file_sink_overflow_backup_names(
    file_path: &str,
    directory_entries: &[String],
    max_files: u16,
) -> Vec<String> {
    let Some(base_name) = file_path
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .map(str::to_string)
    else {
        return Vec::new();
    };
    let prefix = format!("{base_name}.");
    directory_entries
        .iter()
        .filter_map(|entry| {
            let suffix = entry.strip_prefix(&prefix)?;
            let index = suffix.parse::<u16>().ok()?;
            (index > max_files).then(|| entry.clone())
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedQrCodeEcc {
    Low,
    Medium,
    Quartile,
    High,
}

impl SharedQrCodeEcc {
    pub fn ordinal(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::Quartile => 2,
            Self::High => 3,
        }
    }

    pub fn format_bits(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 0,
            Self::Quartile => 3,
            Self::High => 2,
        }
    }

    fn to_nayuki(self) -> NayukiQrCodeEcc {
        match self {
            Self::Low => NayukiQrCodeEcc::Low,
            Self::Medium => NayukiQrCodeEcc::Medium,
            Self::Quartile => NayukiQrCodeEcc::Quartile,
            Self::High => NayukiQrCodeEcc::High,
        }
    }

    fn from_nayuki(ecc: NayukiQrCodeEcc) -> Self {
        match ecc {
            NayukiQrCodeEcc::Low => Self::Low,
            NayukiQrCodeEcc::Medium => Self::Medium,
            NayukiQrCodeEcc::Quartile => Self::Quartile,
            NayukiQrCodeEcc::High => Self::High,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedQrSegmentMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
    Eci,
}

impl SharedQrSegmentMode {
    pub fn mode_bits(self) -> u32 {
        match self {
            Self::Numeric => 0x1,
            Self::Alphanumeric => 0x2,
            Self::Byte => 0x4,
            Self::Kanji => 0x8,
            Self::Eci => 0x7,
        }
    }

    pub fn num_char_count_bits(self, version: u8) -> u8 {
        let index = ((version + 7) / 17) as usize;
        match self {
            Self::Numeric => [10, 12, 14][index],
            Self::Alphanumeric => [9, 11, 13][index],
            Self::Byte => [8, 16, 16][index],
            Self::Kanji => [8, 10, 12][index],
            Self::Eci => [0, 0, 0][index],
        }
    }

    fn to_nayuki(self) -> NayukiQrSegmentMode {
        match self {
            Self::Numeric => NayukiQrSegmentMode::Numeric,
            Self::Alphanumeric => NayukiQrSegmentMode::Alphanumeric,
            Self::Byte => NayukiQrSegmentMode::Byte,
            Self::Kanji => NayukiQrSegmentMode::Kanji,
            Self::Eci => NayukiQrSegmentMode::Eci,
        }
    }

    fn from_nayuki(mode: NayukiQrSegmentMode) -> Self {
        match mode {
            NayukiQrSegmentMode::Numeric => Self::Numeric,
            NayukiQrSegmentMode::Alphanumeric => Self::Alphanumeric,
            NayukiQrSegmentMode::Byte => Self::Byte,
            NayukiQrSegmentMode::Kanji => Self::Kanji,
            NayukiQrSegmentMode::Eci => Self::Eci,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedQrSegment {
    pub mode: SharedQrSegmentMode,
    pub num_chars: usize,
    data: Vec<bool>,
}

impl SharedQrSegment {
    pub fn new(mode: SharedQrSegmentMode, num_chars: usize, data: Vec<bool>) -> Self {
        Self {
            mode,
            num_chars,
            data,
        }
    }

    pub fn data(&self) -> Vec<bool> {
        self.data.clone()
    }

    fn to_nayuki(&self) -> NayukiQrSegment {
        NayukiQrSegment::new(self.mode.to_nayuki(), self.num_chars, self.data.clone())
    }
}

pub fn make_qr_bytes_segment(data: &[u8]) -> SharedQrSegment {
    shared_qr_segment_from_nayuki(NayukiQrSegment::make_bytes(data))
}

pub fn make_qr_numeric_segment(text: &str) -> Result<SharedQrSegment, String> {
    if !is_qr_numeric(text) {
        return Err("String contains non-numeric characters".to_string());
    }
    Ok(shared_qr_segment_from_nayuki(
        NayukiQrSegment::make_numeric(text),
    ))
}

pub fn make_qr_alphanumeric_segment(text: &str) -> Result<SharedQrSegment, String> {
    if !is_qr_alphanumeric(text) {
        return Err("String contains unencodable characters in alphanumeric mode".to_string());
    }
    Ok(shared_qr_segment_from_nayuki(
        NayukiQrSegment::make_alphanumeric(text),
    ))
}

pub fn make_qr_segments(text: &str) -> Vec<SharedQrSegment> {
    NayukiQrSegment::make_segments(text)
        .into_iter()
        .map(shared_qr_segment_from_nayuki)
        .collect()
}

pub fn make_qr_eci_segment(assign_value: u32) -> Result<SharedQrSegment, String> {
    if assign_value >= 1_000_000 {
        return Err("ECI assignment value out of range".to_string());
    }
    Ok(shared_qr_segment_from_nayuki(NayukiQrSegment::make_eci(
        assign_value,
    )))
}

pub fn is_qr_numeric(text: &str) -> bool {
    NayukiQrSegment::is_numeric(text)
}

pub fn is_qr_alphanumeric(text: &str) -> bool {
    NayukiQrSegment::is_alphanumeric(text)
}

pub fn qr_segments_total_bits(segments: &[SharedQrSegment], version: u8) -> Option<usize> {
    if !(1..=40).contains(&version) {
        return None;
    }
    let mut result = 0usize;
    for segment in segments {
        let char_count_bits = segment.mode.num_char_count_bits(version);
        if segment.num_chars >= (1usize).checked_shl(char_count_bits.into())? {
            return None;
        }
        result = result.checked_add(4 + usize::from(char_count_bits))?;
        result = result.checked_add(segment.data.len())?;
    }
    Some(result)
}

pub fn encode_qr_segments(
    segments: &[SharedQrSegment],
    ecc: SharedQrCodeEcc,
) -> Result<SharedQrCode, String> {
    let segments = segments
        .iter()
        .map(SharedQrSegment::to_nayuki)
        .collect::<Vec<_>>();
    NayukiQrCode::encode_segments(&segments, ecc.to_nayuki())
        .map(shared_qr_code_from_nayuki)
        .map_err(|error| error.to_string())
}

pub fn encode_qr_segments_advanced(
    segments: &[SharedQrSegment],
    ecc: SharedQrCodeEcc,
    min_version: u8,
    max_version: u8,
    mask: Option<u8>,
    boost_ecc: bool,
) -> Result<SharedQrCode, String> {
    let min_version = shared_qr_version(min_version)?;
    let max_version = shared_qr_version(max_version)?;
    if min_version > max_version {
        return Err("Invalid value".to_string());
    }
    let mask = shared_qr_mask(mask)?;
    let segments = segments
        .iter()
        .map(SharedQrSegment::to_nayuki)
        .collect::<Vec<_>>();
    NayukiQrCode::encode_segments_advanced(
        &segments,
        ecc.to_nayuki(),
        min_version,
        max_version,
        mask,
        boost_ecc,
    )
    .map(shared_qr_code_from_nayuki)
    .map_err(|error| error.to_string())
}

pub fn encode_qr_codewords(
    version: u8,
    ecc: SharedQrCodeEcc,
    data_codewords: &[u8],
    mask: Option<u8>,
) -> Result<SharedQrCode, String> {
    let version = shared_qr_version(version)?;
    let mask = shared_qr_mask(mask)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        NayukiQrCode::encode_codewords(version, ecc.to_nayuki(), data_codewords, mask)
    }))
    .map(shared_qr_code_from_nayuki)
    .map_err(|_| "Invalid argument".to_string())
}

fn shared_qr_version(version: u8) -> Result<NayukiQrVersion, String> {
    if (1..=40).contains(&version) {
        Ok(NayukiQrVersion::new(version))
    } else {
        Err("Version number out of range".to_string())
    }
}

fn shared_qr_mask(mask: Option<u8>) -> Result<Option<NayukiQrMask>, String> {
    match mask {
        Some(mask) if mask <= 7 => Ok(Some(NayukiQrMask::new(mask))),
        Some(_) => Err("Mask value out of range".to_string()),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedQrCode {
    pub version: u8,
    pub size: i32,
    pub error_correction_level: SharedQrCodeEcc,
    pub mask: u8,
    modules: Vec<bool>,
}

impl SharedQrCode {
    pub fn get_module(&self, x: i32, y: i32) -> bool {
        (0..self.size).contains(&x)
            && (0..self.size).contains(&y)
            && self.modules[(y * self.size + x) as usize]
    }

    pub fn rows(&self) -> Vec<Vec<bool>> {
        (0..self.size)
            .map(|y| (0..self.size).map(|x| self.get_module(x, y)).collect())
            .collect()
    }
}

pub fn encode_text_qr_code(text: &str, ecc: SharedQrCodeEcc) -> Result<SharedQrCode, String> {
    NayukiQrCode::encode_text(text, ecc.to_nayuki())
        .map(shared_qr_code_from_nayuki)
        .map_err(|_| "Data too long".to_string())
}

pub fn encode_binary_qr_code(data: &[u8], ecc: SharedQrCodeEcc) -> Result<SharedQrCode, String> {
    NayukiQrCode::encode_binary(data, ecc.to_nayuki())
        .map(shared_qr_code_from_nayuki)
        .map_err(|_| "Data too long".to_string())
}

pub fn render_terminal_qr_code(value: &str, margin: usize) -> Result<String, String> {
    let qr_code = encode_text_qr_code(value, SharedQrCodeEcc::Medium)?;
    Ok(render_terminal_qr_code_from_symbol(&qr_code, margin))
}

pub fn render_terminal_qr_code_from_symbol(qr_code: &SharedQrCode, margin: usize) -> String {
    let margin = margin as i32;
    let mut rows = Vec::new();
    let mut y = -margin;
    while y < qr_code.size + margin {
        let mut row = String::new();
        for x in -margin..qr_code.size + margin {
            let top_dark = qr_code.get_module(x, y);
            let bottom_dark = qr_code.get_module(x, y + 1);
            row.push(match (top_dark, bottom_dark) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        rows.push(row);
        y += 2;
    }
    rows.join("\n")
}

fn shared_qr_code_from_nayuki(qr_code: NayukiQrCode) -> SharedQrCode {
    let size = qr_code.size();
    let mut modules = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            modules.push(qr_code.get_module(x, y));
        }
    }
    SharedQrCode {
        version: qr_code.version().value(),
        size,
        error_correction_level: SharedQrCodeEcc::from_nayuki(qr_code.error_correction_level()),
        mask: qr_code.mask().value(),
        modules,
    }
}

fn shared_qr_segment_from_nayuki(segment: NayukiQrSegment) -> SharedQrSegment {
    SharedQrSegment {
        mode: SharedQrSegmentMode::from_nayuki(segment.mode()),
        num_chars: segment.num_chars(),
        data: segment.data().clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(value: Option<&str>) -> Option<String> {
        value.map(str::to_string)
    }

    #[test]
    fn ports_shared_string_cli_path_and_semver_helpers() {
        assert_eq!(truncate_text("   hello world   ", 50), "hello world");
        assert_eq!(truncate_text("abcdefghij", 5), "abcde...");

        assert_eq!(
            parse_cli_args_from_str("--chrome --effort high --debug", &[]),
            ParsedCliArgs {
                flags: BTreeMap::from([
                    ("chrome".to_string(), None),
                    ("debug".to_string(), None),
                    ("effort".to_string(), flag(Some("high"))),
                ]),
                positionals: Vec::new(),
            }
        );
        assert_eq!(
            parse_cli_args(["--github-output", "1.2.3"], &["github-output"]),
            ParsedCliArgs {
                flags: BTreeMap::from([("github-output".to_string(), None)]),
                positionals: vec!["1.2.3".to_string()],
            }
        );

        assert!(is_windows_drive_path("C:\\repo"));
        assert!(is_unc_path("\\\\server\\share\\repo"));
        assert!(is_windows_absolute_path("D:/repo"));
        assert!(is_explicit_relative_path("..\\repo"));
        assert!(!is_explicit_relative_path("~/repo"));

        assert_eq!(normalize_semver_version("2.1"), "2.1.0");
        assert!(compare_semver_versions("2.1.111-beta.1", "2.1.111").is_lt());
        assert!(compare_semver_versions("1.2.3abc", "1.2.10").is_gt());
        let range = "^22.16 || ^23.11 || >=24.10";
        assert!(satisfies_semver_range("22.16.0", range));
        assert!(satisfies_semver_range("23.11.1", range));
        assert!(satisfies_semver_range("24.10.0", range));
        assert!(!satisfies_semver_range("24.9.9", range));
        assert!(satisfies_semver_range("0.2.9", "^0.2.3"));
        assert!(!satisfies_semver_range("0.3.0", "^0.2.3"));
    }

    #[test]
    fn ports_shared_git_and_source_control_helpers() {
        assert_eq!(
            normalize_git_remote_url("git@github.com:T3Tools/T3Code.git"),
            "github.com/t3tools/t3code"
        );
        assert_eq!(
            normalize_git_remote_url("ssh://git@gitlab.company.com:2222/team/project.git"),
            "gitlab.company.com/team/project"
        );
        assert_eq!(
            parse_github_repository_name_with_owner_from_remote_url(Some(
                "https://github.com/T3Tools/T3Code.git"
            )),
            Some("T3Tools/T3Code".to_string())
        );

        assert_eq!(
            sanitize_feature_branch_name(" Demo branch! "),
            "feature/demo-branch"
        );
        assert_eq!(
            resolve_auto_feature_branch_name(
                &["feature/demo".to_string(), "feature/demo-2".to_string()],
                Some("demo")
            ),
            "feature/demo-3"
        );
        assert!(is_temporary_worktree_branch(&format!(
            " {WORKTREE_BRANCH_PREFIX}/DEADBEEF "
        )));
        assert!(!is_temporary_worktree_branch("main"));

        let refs = vec![
            VcsRef {
                name: "feature/demo".to_string(),
                current: false,
                is_default: false,
                is_remote: false,
                remote_name: None,
                worktree_path: None,
            },
            VcsRef {
                name: "origin/feature/demo".to_string(),
                current: false,
                is_default: false,
                is_remote: true,
                remote_name: Some("origin".to_string()),
                worktree_path: None,
            },
            VcsRef {
                name: "upstream/feature/demo".to_string(),
                current: false,
                is_default: false,
                is_remote: true,
                remote_name: Some("upstream".to_string()),
                worktree_path: None,
            },
        ];
        assert_eq!(
            dedupe_remote_branches_with_local_matches(&refs)
                .into_iter()
                .map(|reference| reference.name)
                .collect::<Vec<_>>(),
            vec!["feature/demo", "upstream/feature/demo"]
        );

        assert_eq!(
            get_change_request_terminology_for_kind(SharedSourceControlProviderKind::Gitlab),
            ChangeRequestTerminology {
                short_label: "MR",
                singular: "merge request",
            }
        );
        assert_eq!(
            detect_source_control_provider_from_remote_url("git@bitbucket.org:workspace/repo.git")
                .unwrap()
                .kind,
            SharedSourceControlProviderKind::Bitbucket
        );
        assert_eq!(
            resolve_change_request_presentation(Some(&SharedSourceControlProviderInfo {
                kind: SharedSourceControlProviderKind::Unknown,
                name: "forge".to_string(),
                base_url: String::new(),
            }))
            .short_name,
            "change request"
        );
    }

    #[test]
    fn ports_shared_git_status_stream_and_net_contracts() {
        let remote = SharedVcsStatusRemoteResult {
            has_upstream: true,
            ahead_count: 2,
            behind_count: 1,
            ahead_of_default_count: None,
            pr: None,
        };
        let status = apply_git_status_stream_event(
            None,
            SharedVcsStatusStreamEvent::RemoteUpdated {
                remote: remote.clone(),
            },
        );
        assert!(status.local.is_repo);
        assert!(!status.local.has_primary_remote);
        assert_eq!(status.remote, remote);

        let local = SharedVcsStatusLocalResult {
            is_repo: true,
            source_control_provider: Some(SharedSourceControlProviderInfo {
                kind: SharedSourceControlProviderKind::Github,
                name: "GitHub".to_string(),
                base_url: "https://github.com".to_string(),
            }),
            has_primary_remote: true,
            is_default_ref: false,
            ref_name: Some("feature/demo".to_string()),
            has_working_tree_changes: true,
            working_tree: SharedVcsWorkingTreeSummary {
                files: vec![SharedVcsWorkingTreeFile {
                    path: "src/demo.ts".to_string(),
                    insertions: 1,
                    deletions: 0,
                }],
                insertions: 1,
                deletions: 0,
            },
        };
        let updated = apply_git_status_stream_event(
            Some(&SharedVcsStatusResult {
                local: local.clone(),
                remote: SharedVcsStatusRemoteResult {
                    has_upstream: false,
                    ahead_count: 0,
                    behind_count: 0,
                    ahead_of_default_count: None,
                    pr: None,
                },
            }),
            SharedVcsStatusStreamEvent::RemoteUpdated {
                remote: remote.clone(),
            },
        );
        assert_eq!(updated.local, local);
        assert_eq!(updated.remote, remote);

        assert_eq!(
            net_service_operation_names(),
            vec![
                "canListenOnHost",
                "isPortAvailableOnLoopback",
                "reserveLoopbackPort",
                "findAvailablePort",
            ]
        );
        assert!(reserve_loopback_port(None).unwrap() > 0);
        assert!(find_available_port(0).unwrap() > 0);
    }

    #[test]
    fn ports_shared_struct_schema_json_and_server_settings_helpers() {
        let current = serde_json::json!({
            "observability": {
                "otlpTracesUrl": "http://old",
                "keep": true
            },
            "textGenerationModelSelection": {
                "instanceId": "codex",
                "model": "gpt-5.4-mini",
                "options": [{ "id": "reasoningEffort", "value": "high" }]
            }
        });
        let patch = serde_json::json!({
            "observability": {
                "otlpMetricsUrl": "http://metrics"
            },
            "providerInstances": []
        });
        assert_eq!(
            deep_merge_json(&current, &patch),
            serde_json::json!({
                "observability": {
                    "otlpTracesUrl": "http://old",
                    "otlpMetricsUrl": "http://metrics",
                    "keep": true
                },
                "textGenerationModelSelection": {
                    "instanceId": "codex",
                    "model": "gpt-5.4-mini",
                    "options": [{ "id": "reasoningEffort", "value": "high" }]
                },
                "providerInstances": []
            })
        );

        assert_eq!(
            extract_json_object(
                "prefix {\"message\":\"literal } brace\",\"nested\":{\"ok\":true}} suffix"
            ),
            "{\"message\":\"literal } brace\",\"nested\":{\"ok\":true}}"
        );
        assert_eq!(
            extract_json_object("  no structured output  "),
            "no structured output"
        );
        assert_eq!(
            parse_lenient_json(
                r#"{
                  // comment
                  "observability": {
                    "otlpTracesUrl": " http://localhost:4318/v1/traces ",
                    "otlpMetricsUrl": "http://localhost:4318/v1/metrics",
                  },
                }"#,
            )
            .unwrap()["observability"]["otlpMetricsUrl"],
            "http://localhost:4318/v1/metrics"
        );
        assert_eq!(
            decode_json_result(r#"{"ok":true}"#).unwrap(),
            serde_json::json!({ "ok": true })
        );
        assert!(decode_json_result("{").is_err());
        assert_eq!(
            decode_unknown_json_result(&Value::String(r#"{"ok":true}"#.to_string())).unwrap(),
            serde_json::json!({ "ok": true })
        );
        assert!(decode_unknown_json_result(&serde_json::json!({ "ok": true })).is_err());
        assert_eq!(
            decode_lenient_json_result(
                r#"{
                  /* block comment */
                  "items": [1, 2,],
                }"#,
            )
            .unwrap(),
            serde_json::json!({ "items": [1, 2] })
        );
        assert_eq!(
            pretty_json_string(r#"{"b":2,"a":{"nested":true}}"#).unwrap(),
            "{\n  \"a\": {\n    \"nested\": true\n  },\n  \"b\": 2\n}"
        );
        let schema_plan = schema_json_transformation_plan();
        assert_eq!(
            schema_plan.strict_decode,
            "Schema.decodeExit(Schema.fromJsonString(schema))"
        );
        assert!(schema_plan.lenient_decode.contains("trailing commas"));
        assert_eq!(
            schema_plan.pretty_encode,
            "SchemaGetter.stringifyJson({ space: 2 })"
        );
        assert!(
            schema_plan
                .error_formatter
                .contains("SchemaIssue.makeFormatterDefault")
        );

        assert_eq!(
            parse_persisted_server_observability_settings(
                r#"{"observability":{"otlpTracesUrl":" http://trace ","otlpMetricsUrl":" "}}"#
            ),
            PersistedServerObservabilitySettings {
                otlp_traces_url: Some("http://trace".to_string()),
                otlp_metrics_url: None,
            }
        );
        assert_eq!(
            parse_persisted_server_observability_settings("{"),
            PersistedServerObservabilitySettings {
                otlp_traces_url: None,
                otlp_metrics_url: None,
            }
        );

        let current_options = vec![
            ProviderOptionSelection {
                id: "reasoningEffort".to_string(),
                value: SharedProviderOptionValue::String("high".to_string()),
            },
            ProviderOptionSelection {
                id: "fastMode".to_string(),
                value: SharedProviderOptionValue::Boolean(true),
            },
        ];
        let patch_options = vec![ProviderOptionSelection {
            id: "fastMode".to_string(),
            value: SharedProviderOptionValue::Boolean(false),
        }];
        assert_eq!(
            merge_model_selection_options_by_id(Some(&current_options), Some(&patch_options)),
            Some(vec![
                ProviderOptionSelection {
                    id: "fastMode".to_string(),
                    value: SharedProviderOptionValue::Boolean(false),
                },
                ProviderOptionSelection {
                    id: "reasoningEffort".to_string(),
                    value: SharedProviderOptionValue::String("high".to_string()),
                },
            ])
        );
        assert!(should_replace_text_generation_model_selection(Some(
            &SharedModelSelectionPatch {
                instance_id: Some("opencode".to_string()),
                model: None,
                options: None,
            },
        )));
    }

    #[test]
    fn ports_shared_worker_state_and_rotating_file_sink_contracts() {
        let mut worker = DrainableWorkerState::default();
        worker.enqueue("first");
        assert_eq!(worker.outstanding(), 1);
        assert_eq!(worker.take_next(), Some("first"));
        worker.enqueue("second");
        assert!(!worker.can_drain());
        worker.finish_active();
        assert_eq!(worker.take_next(), Some("second"));
        worker.finish_active();
        assert!(worker.can_drain());

        let worker_plan = drainable_worker_runtime_plan();
        assert_eq!(worker_plan.queue_constructor, "TxQueue.unbounded");
        assert_eq!(worker_plan.scope_finalizer, "TxQueue.shutdown");
        assert_eq!(
            worker_plan.enqueue_steps,
            vec![
                "TxQueue.offer(queue, element)",
                "TxRef.update(outstanding, n => n + 1)",
                "Effect.tx",
            ]
        );
        assert!(
            worker_plan
                .process_finalizer
                .contains("TxRef.update(outstanding, n => n - 1)")
        );
        assert!(worker_plan.drain_condition.contains("retries transaction"));

        let mut keyed = KeyedCoalescingWorkerState::default();
        assert!(keyed.enqueue("terminal-1", "first", |_current, next| next));
        assert_eq!(keyed.take_next(), Some(("terminal-1", "first")));
        assert!(!keyed.enqueue("terminal-1", "second", |_current, next| next));
        assert_eq!(keyed.finish_success(&"terminal-1"), Some("second"));
        assert!(!keyed.can_drain_key(&"terminal-1"));
        assert_eq!(keyed.finish_success(&"terminal-1"), None);
        assert!(keyed.can_drain_key(&"terminal-1"));

        let mut failing = KeyedCoalescingWorkerState::default();
        failing.enqueue("terminal-1", "first", |_current, next| next);
        assert_eq!(failing.take_next(), Some(("terminal-1", "first")));
        failing.enqueue("terminal-1", "second", |_current, next| next);
        assert!(failing.finish_failure(&"terminal-1"));
        assert_eq!(failing.take_next(), Some(("terminal-1", "second")));

        let keyed_plan = keyed_coalescing_worker_runtime_plan();
        assert_eq!(
            keyed_plan.state_fields,
            vec!["latestByKey", "queuedKeys", "activeKeys"]
        );
        assert!(
            keyed_plan
                .enqueue_steps
                .contains(&"do not offer duplicate queue item while key is queued or active")
        );
        assert!(
            keyed_plan
                .success_steps
                .contains(&"when pending value exists, delete it and process same key recursively")
        );
        assert!(
            keyed_plan
                .failure_cleanup_steps
                .contains(&"offer key back to queue when cleanup requests requeue")
        );
        assert!(
            keyed_plan
                .drain_condition
                .contains("latestByKey, queuedKeys, or activeKeys")
        );

        assert_eq!(
            validate_rotating_file_sink_options(&RotatingFileSinkOptions {
                file_path: "server-child.log".to_string(),
                max_bytes: 0,
                max_files: 2,
                throw_on_error: false,
            }),
            Err("maxBytes must be >= 1 (received 0)".to_string())
        );
        assert_eq!(
            rotating_file_sink_write_decision(12, 100, 0).unwrap(),
            RotatingFileSinkWriteDecision {
                skip_empty_chunk: true,
                rotate_before_append: false,
                append_bytes: 0,
                rotate_after_append: false,
                next_current_size: 12,
            }
        );
        assert_eq!(
            rotating_file_sink_write_decision(90, 100, 20).unwrap(),
            RotatingFileSinkWriteDecision {
                skip_empty_chunk: false,
                rotate_before_append: true,
                append_bytes: 20,
                rotate_after_append: false,
                next_current_size: 20,
            }
        );
        assert_eq!(
            rotating_file_sink_write_decision(0, 100, 180).unwrap(),
            RotatingFileSinkWriteDecision {
                skip_empty_chunk: false,
                rotate_before_append: false,
                append_bytes: 180,
                rotate_after_append: true,
                next_current_size: 0,
            }
        );
        assert_eq!(
            rotating_file_sink_rotation_order("server-child.log", 3),
            vec![
                (
                    "server-child.log.2".to_string(),
                    "server-child.log.3".to_string()
                ),
                (
                    "server-child.log.1".to_string(),
                    "server-child.log.2".to_string()
                ),
                (
                    "server-child.log".to_string(),
                    "server-child.log.1".to_string()
                ),
            ]
        );
        assert_eq!(
            rotating_file_sink_overflow_backup_names(
                "C:/logs/server-child.log",
                &[
                    "server-child.log.1".to_string(),
                    "server-child.log.4".to_string(),
                    "other.log.9".to_string(),
                ],
                2,
            ),
            vec!["server-child.log.4".to_string()]
        );
    }

    #[test]
    fn ports_shared_nayuki_qr_code_contracts() {
        assert_eq!(SharedQrCodeEcc::Low.ordinal(), 0);
        assert_eq!(SharedQrCodeEcc::Low.format_bits(), 1);
        assert_eq!(SharedQrCodeEcc::Medium.ordinal(), 1);
        assert_eq!(SharedQrCodeEcc::Medium.format_bits(), 0);
        assert_eq!(SharedQrCodeEcc::Quartile.ordinal(), 2);
        assert_eq!(SharedQrCodeEcc::Quartile.format_bits(), 3);
        assert_eq!(SharedQrCodeEcc::High.ordinal(), 3);
        assert_eq!(SharedQrCodeEcc::High.format_bits(), 2);

        assert!(is_qr_numeric("01234567"));
        assert!(!is_qr_numeric("12A"));
        assert!(is_qr_alphanumeric("A-Z 09:$%*+./"));
        assert!(!is_qr_alphanumeric("lower"));

        assert_eq!(SharedQrSegmentMode::Numeric.mode_bits(), 0x1);
        assert_eq!(SharedQrSegmentMode::Alphanumeric.mode_bits(), 0x2);
        assert_eq!(SharedQrSegmentMode::Byte.mode_bits(), 0x4);
        assert_eq!(SharedQrSegmentMode::Kanji.mode_bits(), 0x8);
        assert_eq!(SharedQrSegmentMode::Eci.mode_bits(), 0x7);
        assert_eq!(SharedQrSegmentMode::Numeric.num_char_count_bits(1), 10);
        assert_eq!(SharedQrSegmentMode::Numeric.num_char_count_bits(10), 12);
        assert_eq!(SharedQrSegmentMode::Numeric.num_char_count_bits(27), 14);

        let numeric = make_qr_numeric_segment("01234567").unwrap();
        assert_eq!(numeric.mode, SharedQrSegmentMode::Numeric);
        assert_eq!(numeric.num_chars, 8);
        assert_eq!(numeric.data().len(), 27);
        assert_eq!(qr_segments_total_bits(&[numeric.clone()], 1), Some(41));
        assert!(make_qr_numeric_segment("12A").is_err());

        let alphanumeric = make_qr_alphanumeric_segment("AC-42").unwrap();
        assert_eq!(alphanumeric.mode, SharedQrSegmentMode::Alphanumeric);
        assert_eq!(alphanumeric.num_chars, 5);
        assert_eq!(alphanumeric.data().len(), 28);
        assert!(make_qr_alphanumeric_segment("lower").is_err());

        let bytes = make_qr_bytes_segment(b"Hi");
        assert_eq!(bytes.mode, SharedQrSegmentMode::Byte);
        assert_eq!(bytes.num_chars, 2);
        assert_eq!(bytes.data().len(), 16);

        let eci = make_qr_eci_segment(26).unwrap();
        assert_eq!(eci.mode, SharedQrSegmentMode::Eci);
        assert_eq!(eci.num_chars, 0);
        assert_eq!(eci.data().len(), 8);
        assert!(make_qr_eci_segment(1_000_000).is_err());

        let text_segments = make_qr_segments("HELLO WORLD");
        assert_eq!(text_segments.len(), 1);
        assert_eq!(text_segments[0].mode, SharedQrSegmentMode::Alphanumeric);
        assert_eq!(
            encode_qr_segments(&text_segments, SharedQrCodeEcc::Medium)
                .unwrap()
                .size,
            21
        );
        let forced = encode_qr_segments_advanced(
            &text_segments,
            SharedQrCodeEcc::High,
            5,
            5,
            Some(2),
            false,
        )
        .unwrap();
        assert_eq!(forced.version, 5);
        assert_eq!(forced.size, 37);
        assert_eq!(forced.error_correction_level, SharedQrCodeEcc::High);
        assert_eq!(forced.mask, 2);
        assert!(
            encode_qr_segments_advanced(
                &text_segments,
                SharedQrCodeEcc::High,
                6,
                5,
                Some(2),
                false,
            )
            .is_err()
        );
        assert!(
            encode_qr_segments_advanced(
                &text_segments,
                SharedQrCodeEcc::High,
                5,
                5,
                Some(8),
                false,
            )
            .is_err()
        );

        let codewords = encode_qr_codewords(1, SharedQrCodeEcc::Medium, &[0; 16], Some(0)).unwrap();
        assert_eq!(codewords.version, 1);
        assert_eq!(codewords.size, 21);
        assert_eq!(codewords.error_correction_level, SharedQrCodeEcc::Medium);
        assert_eq!(codewords.mask, 0);
        assert!(encode_qr_codewords(0, SharedQrCodeEcc::Medium, &[0; 16], Some(0)).is_err());

        let qr_code = encode_text_qr_code(
            "http://192.168.1.42:3773/pair#token=PAIRCODE",
            SharedQrCodeEcc::Medium,
        )
        .unwrap();
        assert_eq!(qr_code.version, 4);
        assert_eq!(qr_code.size, 33);
        assert!(qr_code.mask <= 7);
        assert_eq!(qr_code.rows().len(), 33);
        assert_eq!(qr_code.rows()[0].len(), 33);
        assert!(qr_code.get_module(0, 0));
        assert!(!qr_code.get_module(-1, 0));
        assert!(!qr_code.get_module(33, 33));

        let terminal = render_terminal_qr_code_from_symbol(&qr_code, 2);
        assert_eq!(terminal.lines().next().unwrap_or("").chars().count(), 37);
        assert_eq!(terminal.lines().count(), 19);
        assert!(terminal.contains('█') || terminal.contains('▀') || terminal.contains('▄'));

        let binary = encode_binary_qr_code(b"PAIRCODE", SharedQrCodeEcc::Medium).unwrap();
        assert_eq!(binary.size, 21);
        assert!(binary.get_module(0, 0));
    }
}

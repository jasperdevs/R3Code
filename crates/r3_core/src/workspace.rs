use std::{
    cmp::Ordering,
    collections::BTreeSet,
    env, fs, io,
    path::{Component, Path, PathBuf},
};

use crate::{ProjectEntry, ProjectEntryKind};

pub const PROJECT_SEARCH_ENTRIES_MAX_LIMIT: usize = 200;
pub const PROJECT_WRITE_FILE_PATH_MAX_LENGTH: usize = 512;
pub const FILESYSTEM_PATH_MAX_LENGTH: usize = 512;
const WORKSPACE_INDEX_MAX_ENTRIES: usize = 25_000;

const IGNORED_DIRECTORY_NAMES: &[&str] = &[
    ".git",
    ".convex",
    "node_modules",
    ".next",
    ".turbo",
    "dist",
    "build",
    "out",
    ".cache",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSearchEntriesInput {
    pub cwd: String,
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSearchEntriesResult {
    pub entries: Vec<ProjectEntry>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWriteFileInput {
    pub cwd: String,
    pub relative_path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWriteFileResult {
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemBrowseInput {
    pub partial_path: String,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemBrowseEntry {
    pub name: String,
    pub full_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemBrowseResult {
    pub parent_path: String,
    pub entries: Vec<FilesystemBrowseEntry>,
}

#[derive(Debug)]
pub enum WorkspaceError {
    RootDoesNotExist {
        workspace_root: String,
        normalized_workspace_root: String,
    },
    RootCreateFailed {
        workspace_root: String,
        normalized_workspace_root: String,
        source: io::Error,
    },
    RootNotDirectory {
        workspace_root: String,
        normalized_workspace_root: String,
    },
    PathOutsideRoot {
        workspace_root: String,
        relative_path: String,
    },
    InvalidInput {
        field: &'static str,
        detail: String,
    },
    Io {
        operation: &'static str,
        path: String,
        source: io::Error,
    },
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootDoesNotExist {
                normalized_workspace_root,
                ..
            } => write!(
                formatter,
                "Workspace root does not exist: {normalized_workspace_root}"
            ),
            Self::RootCreateFailed {
                normalized_workspace_root,
                source,
                ..
            } => write!(
                formatter,
                "Failed to create workspace root {normalized_workspace_root}: {source}"
            ),
            Self::RootNotDirectory {
                normalized_workspace_root,
                ..
            } => write!(
                formatter,
                "Workspace root is not a directory: {normalized_workspace_root}"
            ),
            Self::PathOutsideRoot { relative_path, .. } => {
                write!(
                    formatter,
                    "Workspace file path must be relative to the project root: {relative_path}"
                )
            }
            Self::InvalidInput { field, detail } => write!(formatter, "Invalid {field}: {detail}"),
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "{operation} failed for {path}: {source}"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedWorkspacePath {
    pub absolute_path: PathBuf,
    pub relative_path: String,
}

#[derive(Debug, Clone)]
struct SearchableWorkspaceEntry {
    entry: ProjectEntry,
    normalized_path: String,
    normalized_name: String,
}

#[derive(Debug, Clone)]
struct RankedWorkspaceEntry {
    entry: ProjectEntry,
    score: usize,
}

pub fn normalize_workspace_root(
    workspace_root: &str,
    create_if_missing: bool,
) -> Result<PathBuf, WorkspaceError> {
    let normalized = absolutize_path(&expand_home_path(workspace_root.trim()));
    let normalized_string = normalized.to_string_lossy().to_string();
    let metadata = match fs::metadata(&normalized) {
        Ok(metadata) => metadata,
        Err(_) if create_if_missing => {
            fs::create_dir_all(&normalized).map_err(|source| WorkspaceError::RootCreateFailed {
                workspace_root: workspace_root.to_string(),
                normalized_workspace_root: normalized_string.clone(),
                source,
            })?;
            fs::metadata(&normalized).map_err(|source| WorkspaceError::Io {
                operation: "workspace.metadata",
                path: normalized_string.clone(),
                source,
            })?
        }
        Err(_) => {
            return Err(WorkspaceError::RootDoesNotExist {
                workspace_root: workspace_root.to_string(),
                normalized_workspace_root: normalized_string,
            });
        }
    };
    if !metadata.is_dir() {
        return Err(WorkspaceError::RootNotDirectory {
            workspace_root: workspace_root.to_string(),
            normalized_workspace_root: normalized_string,
        });
    }
    Ok(normalized)
}

pub fn resolve_relative_path_within_root(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<ResolvedWorkspacePath, WorkspaceError> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() || trimmed.len() > PROJECT_WRITE_FILE_PATH_MAX_LENGTH {
        return Err(WorkspaceError::InvalidInput {
            field: "relativePath",
            detail: "path must be non-empty and at most 512 characters".to_string(),
        });
    }
    let normalized_input = trimmed.replace('\\', "/");
    let input_path = Path::new(&normalized_input);
    if input_path.is_absolute() {
        return Err(WorkspaceError::PathOutsideRoot {
            workspace_root: workspace_root.to_string_lossy().to_string(),
            relative_path: relative_path.to_string(),
        });
    }

    let mut clean = PathBuf::new();
    let mut clean_parts = Vec::new();
    for component in input_path.components() {
        match component {
            Component::Normal(part) => {
                clean.push(part);
                clean_parts.push(part.to_string_lossy().to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(WorkspaceError::PathOutsideRoot {
                    workspace_root: workspace_root.to_string_lossy().to_string(),
                    relative_path: relative_path.to_string(),
                });
            }
        }
    }

    if clean_parts.is_empty() {
        return Err(WorkspaceError::PathOutsideRoot {
            workspace_root: workspace_root.to_string_lossy().to_string(),
            relative_path: relative_path.to_string(),
        });
    }

    Ok(ResolvedWorkspacePath {
        absolute_path: workspace_root.join(clean),
        relative_path: clean_parts.join("/"),
    })
}

pub fn search_workspace_entries(
    input: &ProjectSearchEntriesInput,
) -> Result<ProjectSearchEntriesResult, WorkspaceError> {
    let cwd = normalize_workspace_root(&input.cwd, false)?;
    let query = normalize_search_query(&input.query);
    let limit = input.limit.min(PROJECT_SEARCH_ENTRIES_MAX_LIMIT);
    if limit == 0 {
        return Err(WorkspaceError::InvalidInput {
            field: "limit",
            detail: "limit must be positive".to_string(),
        });
    }

    let index = build_workspace_index(&cwd)?;
    let mut ranked = Vec::new();
    let mut matched_entry_count = 0;
    for entry in &index.entries {
        let Some(score) = score_entry(entry, &query) else {
            continue;
        };
        matched_entry_count += 1;
        insert_ranked_entry(
            &mut ranked,
            RankedWorkspaceEntry {
                entry: entry.entry.clone(),
                score,
            },
            limit,
        );
    }

    Ok(ProjectSearchEntriesResult {
        entries: ranked.into_iter().map(|ranked| ranked.entry).collect(),
        truncated: index.truncated || matched_entry_count > limit,
    })
}

pub fn browse_filesystem(
    input: &FilesystemBrowseInput,
) -> Result<FilesystemBrowseResult, WorkspaceError> {
    if input.partial_path.trim().is_empty() || input.partial_path.len() > FILESYSTEM_PATH_MAX_LENGTH
    {
        return Err(WorkspaceError::InvalidInput {
            field: "partialPath",
            detail: "path must be non-empty and at most 512 characters".to_string(),
        });
    }

    let resolved_input = resolve_browse_target(input)?;
    let ends_with_separator =
        input.partial_path.ends_with(['/', '\\']) || input.partial_path == "~";
    let parent_path = if ends_with_separator {
        resolved_input.clone()
    } else {
        resolved_input
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolved_input.clone())
    };
    let prefix = if ends_with_separator {
        String::new()
    } else {
        resolved_input
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    let lower_prefix = prefix.to_lowercase();
    let show_hidden = ends_with_separator || prefix.starts_with('.');

    let mut entries = Vec::new();
    for entry in fs::read_dir(&parent_path).map_err(|source| WorkspaceError::Io {
        operation: "workspace.browse.read_dir",
        path: parent_path.to_string_lossy().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| WorkspaceError::Io {
            operation: "workspace.browse.dir_entry",
            path: parent_path.to_string_lossy().to_string(),
            source,
        })?;
        let metadata = entry.metadata().map_err(|source| WorkspaceError::Io {
            operation: "workspace.browse.metadata",
            path: entry.path().to_string_lossy().to_string(),
            source,
        })?;
        if !metadata.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.to_lowercase().starts_with(&lower_prefix)
            || (!show_hidden && name.starts_with('.'))
        {
            continue;
        }
        entries.push(FilesystemBrowseEntry {
            name,
            full_path: entry.path().to_string_lossy().to_string(),
        });
    }

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(FilesystemBrowseResult {
        parent_path: parent_path.to_string_lossy().to_string(),
        entries,
    })
}

pub fn write_workspace_file(
    input: &ProjectWriteFileInput,
) -> Result<ProjectWriteFileResult, WorkspaceError> {
    let cwd = normalize_workspace_root(&input.cwd, false)?;
    let target = resolve_relative_path_within_root(&cwd, &input.relative_path)?;
    if let Some(parent) = target.absolute_path.parent() {
        fs::create_dir_all(parent).map_err(|source| WorkspaceError::Io {
            operation: "workspace.write.make_dir",
            path: parent.to_string_lossy().to_string(),
            source,
        })?;
    }
    fs::write(&target.absolute_path, input.contents.as_bytes()).map_err(|source| {
        WorkspaceError::Io {
            operation: "workspace.write.file",
            path: target.absolute_path.to_string_lossy().to_string(),
            source,
        }
    })?;
    Ok(ProjectWriteFileResult {
        relative_path: target.relative_path,
    })
}

struct WorkspaceIndex {
    entries: Vec<SearchableWorkspaceEntry>,
    truncated: bool,
}

fn build_workspace_index(cwd: &Path) -> Result<WorkspaceIndex, WorkspaceError> {
    let mut entries = Vec::new();
    let mut visited_dirs = BTreeSet::new();
    scan_workspace_directory(cwd, Path::new(""), &mut entries, &mut visited_dirs)?;
    let truncated = entries.len() > WORKSPACE_INDEX_MAX_ENTRIES;
    entries.truncate(WORKSPACE_INDEX_MAX_ENTRIES);
    Ok(WorkspaceIndex { entries, truncated })
}

fn scan_workspace_directory(
    root: &Path,
    relative_dir: &Path,
    entries: &mut Vec<SearchableWorkspaceEntry>,
    visited_dirs: &mut BTreeSet<String>,
) -> Result<(), WorkspaceError> {
    if entries.len() > WORKSPACE_INDEX_MAX_ENTRIES {
        return Ok(());
    }
    let relative_key = to_posix_path(relative_dir);
    if !visited_dirs.insert(relative_key) {
        return Ok(());
    }
    let absolute_dir = root.join(relative_dir);
    let mut children = fs::read_dir(&absolute_dir)
        .map_err(|source| WorkspaceError::Io {
            operation: "workspace.search.read_dir",
            path: absolute_dir.to_string_lossy().to_string(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| WorkspaceError::Io {
            operation: "workspace.search.dir_entry",
            path: absolute_dir.to_string_lossy().to_string(),
            source,
        })?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        if entries.len() > WORKSPACE_INDEX_MAX_ENTRIES {
            return Ok(());
        }
        let file_name = child.file_name().to_string_lossy().to_string();
        if file_name.is_empty() || file_name == "." || file_name == ".." {
            continue;
        }
        let metadata = child.metadata().map_err(|source| WorkspaceError::Io {
            operation: "workspace.search.metadata",
            path: child.path().to_string_lossy().to_string(),
            source,
        })?;
        if !metadata.is_dir() && !metadata.is_file() {
            continue;
        }
        if metadata.is_dir() && IGNORED_DIRECTORY_NAMES.contains(&file_name.as_str()) {
            continue;
        }

        let child_relative = relative_dir.join(&file_name);
        let path = to_posix_path(&child_relative);
        if is_path_in_ignored_directory(&path) {
            continue;
        }
        let entry = ProjectEntry {
            path: path.clone(),
            kind: if metadata.is_dir() {
                ProjectEntryKind::Directory
            } else {
                ProjectEntryKind::File
            },
            parent_path: parent_path_of(&path),
        };
        entries.push(to_searchable_workspace_entry(entry));

        if metadata.is_dir() {
            scan_workspace_directory(root, &child_relative, entries, visited_dirs)?;
        }
    }
    Ok(())
}

fn resolve_browse_target(input: &FilesystemBrowseInput) -> Result<PathBuf, WorkspaceError> {
    let partial = input.partial_path.trim();
    let partial_path = Path::new(partial);
    if is_explicit_relative_path(partial) {
        let Some(cwd) = input.cwd.as_deref() else {
            return Err(WorkspaceError::InvalidInput {
                field: "cwd",
                detail: "relative filesystem browse paths require a current project".to_string(),
            });
        };
        return Ok(absolutize_path(&expand_home_path(cwd)).join(partial_path));
    }
    Ok(absolutize_path(&expand_home_path(partial)))
}

fn to_searchable_workspace_entry(entry: ProjectEntry) -> SearchableWorkspaceEntry {
    let normalized_path = entry.path.to_lowercase();
    let normalized_name = basename_of(&normalized_path).to_string();
    SearchableWorkspaceEntry {
        entry,
        normalized_path,
        normalized_name,
    }
}

fn score_entry(entry: &SearchableWorkspaceEntry, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(match entry.entry.kind {
            ProjectEntryKind::Directory => 0,
            ProjectEntryKind::File => 1,
        });
    }
    let mut scores = Vec::new();
    if entry.normalized_name == query {
        scores.push(0);
    }
    if entry.normalized_path == query {
        scores.push(1);
    }
    if entry.normalized_name.starts_with(query) {
        scores.push(2);
    }
    if entry.normalized_path.starts_with(query) {
        scores.push(3);
    }
    if path_boundary_match(&entry.normalized_path, query) {
        scores.push(4);
    }
    if entry.normalized_name.contains(query) {
        scores.push(5);
    }
    if entry.normalized_path.contains(query) {
        scores.push(6);
    }
    if is_fuzzy_subsequence(&entry.normalized_name, query) {
        scores.push(100 + entry.normalized_name.len());
    }
    if is_fuzzy_subsequence(&entry.normalized_path, query) {
        scores.push(200 + entry.normalized_path.len());
    }
    scores.into_iter().min()
}

fn insert_ranked_entry(
    ranked: &mut Vec<RankedWorkspaceEntry>,
    candidate: RankedWorkspaceEntry,
    limit: usize,
) {
    let insert_at = ranked
        .binary_search_by(|existing| compare_ranked_entries(existing, &candidate))
        .unwrap_or_else(|index| index);
    ranked.insert(insert_at, candidate);
    ranked.truncate(limit);
}

fn compare_ranked_entries(left: &RankedWorkspaceEntry, right: &RankedWorkspaceEntry) -> Ordering {
    left.score
        .cmp(&right.score)
        .then_with(|| left.entry.path.cmp(&right.entry.path))
}

fn normalize_search_query(query: &str) -> String {
    query
        .trim()
        .trim_start_matches(['@', '.', '/'])
        .to_lowercase()
}

fn path_boundary_match(path: &str, query: &str) -> bool {
    path.split('/').any(|part| part.starts_with(query))
}

fn is_fuzzy_subsequence(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut query_chars = query.chars();
    let mut current = query_chars.next();
    for character in value.chars() {
        if Some(character) == current {
            current = query_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }
    false
}

fn basename_of(input: &str) -> &str {
    input
        .rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(input)
}

fn parent_path_of(input: &str) -> Option<String> {
    input
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .filter(|parent| !parent.is_empty())
}

fn is_path_in_ignored_directory(relative_path: &str) -> bool {
    relative_path
        .split('/')
        .next()
        .is_some_and(|segment| IGNORED_DIRECTORY_NAMES.contains(&segment))
}

fn is_explicit_relative_path(input: &str) -> bool {
    input == "."
        || input == ".."
        || input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with(".\\")
        || input.starts_with("..\\")
}

fn to_posix_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn expand_home_path(input: &str) -> PathBuf {
    if input == "~" || input.starts_with("~/") || input.starts_with("~\\") {
        if let Some(home) = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")) {
            let suffix = if input == "~" { "" } else { &input[2..] };
            return PathBuf::from(home).join(suffix);
        }
    }
    PathBuf::from(input)
}

fn absolutize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("r3code-workspace-test-{unique}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn resolves_workspace_relative_paths_inside_root_only() {
        let root = temp_workspace();

        let resolved = resolve_relative_path_within_root(&root, "src\\main.rs").unwrap();

        assert_eq!(resolved.relative_path, "src/main.rs");
        assert!(
            resolved
                .absolute_path
                .ends_with(Path::new("src").join("main.rs"))
        );
        assert!(matches!(
            resolve_relative_path_within_root(&root, "../outside.txt"),
            Err(WorkspaceError::PathOutsideRoot { .. })
        ));
        assert!(matches!(
            resolve_relative_path_within_root(&root, ""),
            Err(WorkspaceError::InvalidInput { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn searches_workspace_entries_with_upstream_limits_and_ignored_dirs() {
        let root = temp_workspace();
        fs::create_dir_all(root.join("src/components")).unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/components/Button.rs"), "button").unwrap();
        fs::write(root.join("node_modules/pkg/index.js"), "ignored").unwrap();

        let result = search_workspace_entries(&ProjectSearchEntriesInput {
            cwd: root.to_string_lossy().to_string(),
            query: "@button".to_string(),
            limit: 10,
        })
        .unwrap();

        assert_eq!(result.entries[0].path, "src/components/Button.rs");
        assert!(
            !result
                .entries
                .iter()
                .any(|entry| entry.path.starts_with("node_modules"))
        );

        let limited = search_workspace_entries(&ProjectSearchEntriesInput {
            cwd: root.to_string_lossy().to_string(),
            query: "src".to_string(),
            limit: 1,
        })
        .unwrap();
        assert_eq!(limited.entries.len(), 1);
        assert!(limited.truncated);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn browses_directory_prefixes_and_hides_dot_dirs_until_requested() {
        let root = temp_workspace();
        fs::create_dir_all(root.join("alpha")).unwrap();
        fs::create_dir_all(root.join("alpine")).unwrap();
        fs::create_dir_all(root.join(".agent")).unwrap();
        fs::write(root.join("alpha.txt"), "file").unwrap();

        let result = browse_filesystem(&FilesystemBrowseInput {
            partial_path: root.join("al").to_string_lossy().to_string(),
            cwd: None,
        })
        .unwrap();
        assert_eq!(
            result
                .entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "alpine"]
        );

        let hidden = browse_filesystem(&FilesystemBrowseInput {
            partial_path: root.join(".a").to_string_lossy().to_string(),
            cwd: None,
        })
        .unwrap();
        assert_eq!(hidden.entries[0].name, ".agent");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writes_workspace_files_and_rejects_traversal() {
        let root = temp_workspace();

        let result = write_workspace_file(&ProjectWriteFileInput {
            cwd: root.to_string_lossy().to_string(),
            relative_path: "notes/todo.txt".to_string(),
            contents: "ship it".to_string(),
        })
        .unwrap();

        assert_eq!(result.relative_path, "notes/todo.txt");
        assert_eq!(
            fs::read_to_string(root.join("notes/todo.txt")).unwrap(),
            "ship it"
        );
        assert!(matches!(
            write_workspace_file(&ProjectWriteFileInput {
                cwd: root.to_string_lossy().to_string(),
                relative_path: "../escape.txt".to_string(),
                contents: "no".to_string(),
            }),
            Err(WorkspaceError::PathOutsideRoot { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }
}

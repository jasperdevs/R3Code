use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

#[derive(Debug, Clone, Copy)]
pub struct R3Assets;

const ICON_ASSETS: &[(&str, &[u8])] = &[
    (
        "icons/archive.svg",
        include_bytes!("../assets/icons/archive.svg"),
    ),
    (
        "icons/arrow-left.svg",
        include_bytes!("../assets/icons/arrow-left.svg"),
    ),
    (
        "icons/arrow-up-down.svg",
        include_bytes!("../assets/icons/arrow-up-down.svg"),
    ),
    (
        "icons/arrow-up.svg",
        include_bytes!("../assets/icons/arrow-up.svg"),
    ),
    ("icons/bot.svg", include_bytes!("../assets/icons/bot.svg")),
    ("icons/bug.svg", include_bytes!("../assets/icons/bug.svg")),
    (
        "icons/check.svg",
        include_bytes!("../assets/icons/check.svg"),
    ),
    (
        "icons/chevron-down.svg",
        include_bytes!("../assets/icons/chevron-down.svg"),
    ),
    (
        "icons/chevron-left.svg",
        include_bytes!("../assets/icons/chevron-left.svg"),
    ),
    (
        "icons/chevron-right.svg",
        include_bytes!("../assets/icons/chevron-right.svg"),
    ),
    (
        "icons/claude-ai.svg",
        include_bytes!("../assets/icons/claude-ai.svg"),
    ),
    (
        "icons/clock-3.svg",
        include_bytes!("../assets/icons/clock-3.svg"),
    ),
    (
        "icons/cloud-upload.svg",
        include_bytes!("../assets/icons/cloud-upload.svg"),
    ),
    (
        "icons/cloud.svg",
        include_bytes!("../assets/icons/cloud.svg"),
    ),
    (
        "icons/columns-2.svg",
        include_bytes!("../assets/icons/columns-2.svg"),
    ),
    ("icons/copy.svg", include_bytes!("../assets/icons/copy.svg")),
    (
        "icons/cursor.svg",
        include_bytes!("../assets/icons/cursor.svg"),
    ),
    ("icons/diff.svg", include_bytes!("../assets/icons/diff.svg")),
    (
        "icons/ellipsis.svg",
        include_bytes!("../assets/icons/ellipsis.svg"),
    ),
    (
        "icons/file-json.svg",
        include_bytes!("../assets/icons/file-json.svg"),
    ),
    (
        "icons/file-type-light-agents.svg",
        include_bytes!("../assets/icons/file-type-light-agents.svg"),
    ),
    (
        "icons/flask-conical.svg",
        include_bytes!("../assets/icons/flask-conical.svg"),
    ),
    (
        "icons/folder-closed.svg",
        include_bytes!("../assets/icons/folder-closed.svg"),
    ),
    (
        "icons/folder-git-2.svg",
        include_bytes!("../assets/icons/folder-git-2.svg"),
    ),
    (
        "icons/folder-git.svg",
        include_bytes!("../assets/icons/folder-git.svg"),
    ),
    (
        "icons/folder-plus.svg",
        include_bytes!("../assets/icons/folder-plus.svg"),
    ),
    (
        "icons/folder.svg",
        include_bytes!("../assets/icons/folder.svg"),
    ),
    (
        "icons/git-branch.svg",
        include_bytes!("../assets/icons/git-branch.svg"),
    ),
    (
        "icons/git-commit.svg",
        include_bytes!("../assets/icons/git-commit.svg"),
    ),
    (
        "icons/git-pull-request.svg",
        include_bytes!("../assets/icons/git-pull-request.svg"),
    ),
    (
        "icons/hammer.svg",
        include_bytes!("../assets/icons/hammer.svg"),
    ),
    ("icons/info.svg", include_bytes!("../assets/icons/info.svg")),
    (
        "icons/keyboard.svg",
        include_bytes!("../assets/icons/keyboard.svg"),
    ),
    (
        "icons/link-2.svg",
        include_bytes!("../assets/icons/link-2.svg"),
    ),
    (
        "icons/list-checks.svg",
        include_bytes!("../assets/icons/list-checks.svg"),
    ),
    (
        "icons/lock-open.svg",
        include_bytes!("../assets/icons/lock-open.svg"),
    ),
    ("icons/lock.svg", include_bytes!("../assets/icons/lock.svg")),
    (
        "icons/message-square.svg",
        include_bytes!("../assets/icons/message-square.svg"),
    ),
    (
        "icons/minus.svg",
        include_bytes!("../assets/icons/minus.svg"),
    ),
    (
        "icons/monitor.svg",
        include_bytes!("../assets/icons/monitor.svg"),
    ),
    (
        "icons/openai.svg",
        include_bytes!("../assets/icons/openai.svg"),
    ),
    (
        "icons/opencode.svg",
        include_bytes!("../assets/icons/opencode.svg"),
    ),
    (
        "icons/pen-line.svg",
        include_bytes!("../assets/icons/pen-line.svg"),
    ),
    (
        "icons/pilcrow.svg",
        include_bytes!("../assets/icons/pilcrow.svg"),
    ),
    ("icons/play.svg", include_bytes!("../assets/icons/play.svg")),
    (
        "icons/plus-square.svg",
        include_bytes!("../assets/icons/plus-square.svg"),
    ),
    ("icons/plus.svg", include_bytes!("../assets/icons/plus.svg")),
    (
        "icons/refresh-cw.svg",
        include_bytes!("../assets/icons/refresh-cw.svg"),
    ),
    (
        "icons/rotate-ccw.svg",
        include_bytes!("../assets/icons/rotate-ccw.svg"),
    ),
    (
        "icons/rows-3.svg",
        include_bytes!("../assets/icons/rows-3.svg"),
    ),
    (
        "icons/search.svg",
        include_bytes!("../assets/icons/search.svg"),
    ),
    (
        "icons/settings-2.svg",
        include_bytes!("../assets/icons/settings-2.svg"),
    ),
    (
        "icons/skill-chip.svg",
        include_bytes!("../assets/icons/skill-chip.svg"),
    ),
    (
        "icons/sparkles.svg",
        include_bytes!("../assets/icons/sparkles.svg"),
    ),
    (
        "icons/square-pen.svg",
        include_bytes!("../assets/icons/square-pen.svg"),
    ),
    (
        "icons/square-split-horizontal.svg",
        include_bytes!("../assets/icons/square-split-horizontal.svg"),
    ),
    (
        "icons/square-terminal.svg",
        include_bytes!("../assets/icons/square-terminal.svg"),
    ),
    ("icons/star.svg", include_bytes!("../assets/icons/star.svg")),
    (
        "icons/terminal.svg",
        include_bytes!("../assets/icons/terminal.svg"),
    ),
    (
        "icons/text-wrap.svg",
        include_bytes!("../assets/icons/text-wrap.svg"),
    ),
    (
        "icons/trash-2.svg",
        include_bytes!("../assets/icons/trash-2.svg"),
    ),
    (
        "icons/triangle-alert.svg",
        include_bytes!("../assets/icons/triangle-alert.svg"),
    ),
    (
        "icons/visual-studio-code.svg",
        include_bytes!("../assets/icons/visual-studio-code.svg"),
    ),
    (
        "icons/wrench.svg",
        include_bytes!("../assets/icons/wrench.svg"),
    ),
    ("icons/zed.svg", include_bytes!("../assets/icons/zed.svg")),
];

impl AssetSource for R3Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(ICON_ASSETS
            .iter()
            .find(|(asset_path, _)| *asset_path == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path != "icons" && path != "icons/" {
            return Ok(Vec::new());
        }

        Ok(ICON_ASSETS
            .iter()
            .filter_map(|(asset_path, _)| asset_path.strip_prefix("icons/"))
            .map(SharedString::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listed_icons_are_loadable() {
        let assets = R3Assets;
        for icon in assets.list("icons").expect("list icons") {
            let path = format!("icons/{icon}");
            assert!(
                assets.load(&path).expect("load icon").is_some(),
                "listed icon was not loadable: {path}"
            );
        }
    }

    #[test]
    fn icon_registry_is_sorted_and_unique() {
        let mut previous = "";
        for (path, _) in ICON_ASSETS {
            assert!(
                path.starts_with("icons/"),
                "icon path must be scoped: {path}"
            );
            assert!(
                *path > previous,
                "icon registry must be sorted and unique: {path}"
            );
            previous = path;
        }
    }

    #[test]
    fn upstream_lucide_icons_used_by_native_surfaces_are_listed() {
        let icons = assets_list();
        for icon in [
            "check.svg",
            "bug.svg",
            "chevron-left.svg",
            "claude-ai.svg",
            "clock-3.svg",
            "cloud.svg",
            "cloud-upload.svg",
            "columns-2.svg",
            "cursor.svg",
            "diff.svg",
            "ellipsis.svg",
            "file-type-light-agents.svg",
            "flask-conical.svg",
            "folder-closed.svg",
            "folder-plus.svg",
            "folder.svg",
            "folder-git-2.svg",
            "folder-git.svg",
            "git-commit.svg",
            "git-pull-request.svg",
            "hammer.svg",
            "info.svg",
            "list-checks.svg",
            "lock.svg",
            "lock-open.svg",
            "message-square.svg",
            "monitor.svg",
            "openai.svg",
            "opencode.svg",
            "pen-line.svg",
            "pilcrow.svg",
            "play.svg",
            "rotate-ccw.svg",
            "rows-3.svg",
            "skill-chip.svg",
            "square-pen.svg",
            "square-split-horizontal.svg",
            "square-terminal.svg",
            "sparkles.svg",
            "star.svg",
            "text-wrap.svg",
            "triangle-alert.svg",
            "trash-2.svg",
            "visual-studio-code.svg",
            "wrench.svg",
            "zed.svg",
        ] {
            assert!(icons.iter().any(|listed| listed == icon), "{icon}");
        }
    }

    fn assets_list() -> Vec<String> {
        R3Assets
            .list("icons")
            .expect("list icons")
            .into_iter()
            .map(String::from)
            .collect()
    }
}

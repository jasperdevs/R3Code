use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

#[derive(Debug, Clone, Copy)]
pub struct R3Assets;

impl AssetSource for R3Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes = match path {
            "icons/archive.svg" => include_bytes!("../assets/icons/archive.svg").as_slice(),
            "icons/arrow-left.svg" => include_bytes!("../assets/icons/arrow-left.svg").as_slice(),
            "icons/arrow-up-down.svg" => {
                include_bytes!("../assets/icons/arrow-up-down.svg").as_slice()
            }
            "icons/bot.svg" => include_bytes!("../assets/icons/bot.svg").as_slice(),
            "icons/git-branch.svg" => include_bytes!("../assets/icons/git-branch.svg").as_slice(),
            "icons/keyboard.svg" => include_bytes!("../assets/icons/keyboard.svg").as_slice(),
            "icons/link-2.svg" => include_bytes!("../assets/icons/link-2.svg").as_slice(),
            "icons/plus-square.svg" => include_bytes!("../assets/icons/plus-square.svg").as_slice(),
            "icons/settings-2.svg" => include_bytes!("../assets/icons/settings-2.svg").as_slice(),
            _ => return Ok(None),
        };

        Ok(Some(Cow::Borrowed(bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path != "icons" && path != "icons/" {
            return Ok(Vec::new());
        }

        Ok([
            "archive.svg",
            "arrow-left.svg",
            "arrow-up-down.svg",
            "bot.svg",
            "git-branch.svg",
            "keyboard.svg",
            "link-2.svg",
            "plus-square.svg",
            "settings-2.svg",
        ]
        .into_iter()
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
    fn project_header_icons_are_listed() {
        let icons = assets_list();
        assert!(icons.iter().any(|icon| icon == "arrow-up-down.svg"));
        assert!(icons.iter().any(|icon| icon == "plus-square.svg"));
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

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

#[derive(Debug, Clone, Copy)]
pub struct R3Assets;

impl AssetSource for R3Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes = match path {
            "icons/archive.svg" => include_bytes!("../assets/icons/archive.svg").as_slice(),
            "icons/arrow-left.svg" => include_bytes!("../assets/icons/arrow-left.svg").as_slice(),
            "icons/bot.svg" => include_bytes!("../assets/icons/bot.svg").as_slice(),
            "icons/git-branch.svg" => include_bytes!("../assets/icons/git-branch.svg").as_slice(),
            "icons/keyboard.svg" => include_bytes!("../assets/icons/keyboard.svg").as_slice(),
            "icons/link-2.svg" => include_bytes!("../assets/icons/link-2.svg").as_slice(),
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
            "bot.svg",
            "git-branch.svg",
            "keyboard.svg",
            "link-2.svg",
            "settings-2.svg",
        ]
        .into_iter()
        .map(SharedString::from)
        .collect())
    }
}

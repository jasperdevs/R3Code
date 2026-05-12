use gpui::{Hsla, Window, WindowAppearance, hsla};

pub const SIDEBAR_MIN_WIDTH: f32 = 255.0;
pub const MAIN_MIN_WIDTH: f32 = 640.0;
pub const RADIUS: f32 = 10.0;
pub const SCROLLBAR_WIDTH: f32 = 6.0;

pub const FONT_FAMILY: &str = "Segoe UI";
pub const MONO_FONT_FAMILY: &str = "Consolas";

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Hsla,
    pub app_chrome_background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub border: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub primary: Hsla,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn from_env() -> Self {
        match std::env::var("R3CODE_THEME")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    pub fn resolve(self, window: &Window) -> Theme {
        match self {
            Self::Light => Theme::light(),
            Self::Dark => Theme::dark(),
            Self::System => Theme::for_window(window),
        }
    }
}

impl Theme {
    pub fn for_window(window: &Window) -> Self {
        match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: hsla(0.0, 0.0, 0.10, 1.0),
            app_chrome_background: hsla(0.0, 0.0, 0.10, 1.0),
            foreground: hsla(0.0, 0.0, 0.96, 1.0),
            card: hsla(0.0, 0.0, 0.11, 1.0),
            border: hsla(0.0, 0.0, 1.0, 0.06),
            muted_foreground: hsla(0.0, 0.0, 0.62, 1.0),
            accent: hsla(0.0, 0.0, 1.0, 0.04),
            primary: hsla(247.0 / 360.0, 0.82, 0.58, 1.0),
        }
    }

    pub fn light() -> Self {
        Self {
            background: hsla(0.0, 0.0, 0.995, 1.0),
            app_chrome_background: hsla(0.0, 0.0, 0.995, 1.0),
            foreground: hsla(0.0, 0.0, 38.0 / 255.0, 1.0),
            card: hsla(0.0, 0.0, 0.98, 1.0),
            border: hsla(0.0, 0.0, 0.0, 0.08),
            muted_foreground: hsla(0.0, 0.0, 0.58, 1.0),
            accent: hsla(0.0, 0.0, 0.0, 0.04),
            primary: hsla(247.0 / 360.0, 0.82, 0.48, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn light_foreground_matches_upstream_neutral_800_token() {
        let foreground = Theme::light().foreground;

        assert_eq!(foreground.h, 0.0);
        assert_eq!(foreground.s, 0.0);
        assert_eq!(foreground.l, 38.0 / 255.0);
        assert_eq!(foreground.a, 1.0);
    }
}

use gpui::{Hsla, hsla};

pub const SIDEBAR_MIN_WIDTH: f32 = 255.0;
pub const MAIN_MIN_WIDTH: f32 = 640.0;
pub const RADIUS: f32 = 10.0;
pub const SCROLLBAR_WIDTH: f32 = 6.0;

pub const FONT_FAMILY: &str = "Segoe UI";
pub const MONO_FONT_FAMILY: &str = "SF Mono";

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

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: hsla(0.0, 0.0, 0.08, 1.0),
            app_chrome_background: hsla(0.0, 0.0, 0.08, 1.0),
            foreground: hsla(0.0, 0.0, 0.96, 1.0),
            card: hsla(0.0, 0.0, 0.09, 1.0),
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
            foreground: hsla(0.0, 0.0, 0.09, 1.0),
            card: hsla(0.0, 0.0, 0.98, 1.0),
            border: hsla(0.0, 0.0, 0.0, 0.08),
            muted_foreground: hsla(0.0, 0.0, 0.58, 1.0),
            accent: hsla(0.0, 0.0, 0.0, 0.04),
            primary: hsla(247.0 / 360.0, 0.82, 0.48, 1.0),
        }
    }
}

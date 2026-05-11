use gpui::Application;
use r3_ui::assets::R3Assets;

fn main() {
    Application::new().with_assets(R3Assets).run(|cx| {
        r3_ui::shell::open_main_window(cx);
    });
}

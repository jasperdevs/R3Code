use gpui::Application;

fn main() {
    Application::new().run(|cx| {
        r3_ui::shell::open_main_window(cx);
    });
}

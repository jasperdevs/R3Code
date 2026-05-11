use gpui::{
    App, AppContext, Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, px,
};
use r3_core::{APP_NAME, AppSnapshot, MessageAuthor, ThreadStatus};

use crate::theme::{SIDEBAR_MIN_WIDTH, Theme};

pub struct R3Shell {
    snapshot: AppSnapshot,
    theme: Theme,
}

impl R3Shell {
    pub fn new(snapshot: AppSnapshot) -> Self {
        Self {
            snapshot,
            theme: Theme::light(),
        }
    }
}

impl Render for R3Shell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .h_full()
            .w_full()
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .font_family(SharedString::from("DM Sans"))
            .child(self.sidebar())
            .child(self.main_panel())
    }
}

impl R3Shell {
    fn sidebar(&self) -> impl IntoElement {
        let mut sidebar = div()
            .flex()
            .flex_col()
            .h_full()
            .border_r_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .w(px(SIDEBAR_MIN_WIDTH));

        sidebar = sidebar.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .pt_4()
                .pb_5()
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight(700.0))
                        .child(APP_NAME),
                )
                .child(
                    div()
                        .rounded(px(5.0))
                        .bg(self.theme.accent)
                        .px_1()
                        .py_0p5()
                        .text_size(px(9.0))
                        .text_color(self.theme.muted_foreground)
                        .child("DEV"),
                ),
        );

        sidebar = sidebar.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .pb_6()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_size(px(12.0))
                        .text_color(self.theme.muted_foreground)
                        .child("⌕")
                        .child("Search"),
                )
                .child(
                    div()
                        .rounded(px(5.0))
                        .bg(self.theme.accent)
                        .px_1p5()
                        .py_0p5()
                        .text_size(px(10.0))
                        .text_color(self.theme.muted_foreground)
                        .child("Ctrl+K"),
                ),
        );

        sidebar = sidebar.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .pb_6()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(self.theme.muted_foreground)
                        .child("PROJECTS"),
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .text_size(px(12.0))
                        .text_color(self.theme.muted_foreground)
                        .child("↕")
                        .child("+"),
                ),
        );

        if self.snapshot.projects.is_empty() {
            sidebar = sidebar.child(
                div()
                    .flex()
                    .justify_center()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .child("No projects yet"),
            );
        }

        for project in &self.snapshot.projects {
            sidebar = sidebar.child(
                div()
                    .rounded(px(8.0))
                    .bg(self.theme.accent)
                    .p_2()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight(600.0))
                            .child(project.name.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(self.theme.muted_foreground)
                            .child(project.path.clone()),
                    ),
            );
        }

        for thread in &self.snapshot.threads {
            let status = match thread.status {
                ThreadStatus::Idle => "",
                ThreadStatus::Running => "Running",
                ThreadStatus::NeedsInput => "Needs input",
                ThreadStatus::Failed => "Failed",
            };
            sidebar = sidebar.child(
                div()
                    .rounded(px(8.0))
                    .p_2()
                    .child(div().text_size(px(13.0)).child(thread.title.clone()))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(self.theme.muted_foreground)
                            .child(format!("{} {}", thread.project_name, status)),
                    ),
            );
        }

        sidebar.child(
            div().flex_1().child("").child(
                div()
                    .absolute()
                    .bottom_4()
                    .left_4()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .child("⚙  Settings"),
            ),
        )
    }

    fn main_panel(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(self.toolbar())
            .child(self.timeline())
            .child(self.composer())
    }

    fn toolbar(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .h(px(41.0))
            .px_5()
            .border_b_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(self.theme.muted_foreground)
                    .child("No active thread"),
            )
    }

    fn timeline(&self) -> impl IntoElement {
        let mut timeline = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .p_4();

        if self.snapshot.messages.is_empty() {
            return timeline
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .rounded(px(18.0))
                        .border_1()
                        .border_color(self.theme.border)
                        .bg(self.theme.background)
                        .w(px(512.0))
                        .h(px(151.0))
                        .child(
                            div()
                                .text_size(px(20.0))
                                .font_weight(FontWeight(700.0))
                                .child("Pick a thread to continue"),
                        )
                        .child(
                            div()
                                .mt_3()
                                .text_size(px(14.0))
                                .text_color(self.theme.muted_foreground)
                                .child(
                                    "Select an existing thread or create a new one to get started.",
                                ),
                        ),
                )
                .into_any_element();
        }

        for message in &self.snapshot.messages {
            let author = match message.author {
                MessageAuthor::User => "You",
                MessageAuthor::Agent => APP_NAME,
            };
            timeline = timeline.child(
                div()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .p_3()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground)
                            .child(author),
                    )
                    .child(div().text_size(px(14.0)).child(message.body.clone())),
            );
        }

        timeline.into_any_element()
    }

    fn composer(&self) -> impl IntoElement {
        div().h(px(0.0))
    }
}

pub fn open_main_window(cx: &mut App) {
    let bounds = gpui::Bounds::centered(None, gpui::size(px(1280.0), px(800.0)), cx);
    cx.open_window(
        gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: None,
            ..Default::default()
        },
        |_, cx| cx.new(|_| R3Shell::new(AppSnapshot::empty_reference_state())),
    )
    .expect("failed to open R3Code window");
}

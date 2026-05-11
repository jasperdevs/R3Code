use gpui::{
    App, AppContext, Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, px, rgb,
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
            theme: Theme::dark(),
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
        let mut projects = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .border_r_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .w(px(SIDEBAR_MIN_WIDTH));

        projects = projects.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .mb_2()
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight(600.0))
                        .child(APP_NAME),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(self.theme.muted_foreground)
                        .child("Alpha"),
                ),
        );

        for project in &self.snapshot.projects {
            projects = projects.child(
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
            projects = projects.child(
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

        projects
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
            .justify_between()
            .h(px(48.0))
            .px_4()
            .border_b_1()
            .border_color(self.theme.border)
            .child(div().text_size(px(13.0)).child("main"))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .child("Rust / GPUI parity shell"),
            )
    }

    fn timeline(&self) -> impl IntoElement {
        let mut timeline = div().flex().flex_col().gap_3().flex_1().p_4();

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

        timeline
    }

    fn composer(&self) -> impl IntoElement {
        div()
            .p_4()
            .border_t_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(rgb(0x151515))
                    .p_3()
                    .text_size(px(14.0))
                    .text_color(self.theme.muted_foreground)
                    .child("Ask R3Code to work on this repo..."),
            )
    }
}

pub fn open_main_window(cx: &mut App) {
    let bounds = gpui::Bounds::centered(None, gpui::size(px(1280.0), px(800.0)), cx);
    cx.open_window(
        gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(SharedString::from(APP_NAME)),
                ..Default::default()
            }),
            ..Default::default()
        },
        |_, cx| cx.new(|_| R3Shell::new(AppSnapshot::mock_reference_state())),
    )
    .expect("failed to open R3Code window");
}

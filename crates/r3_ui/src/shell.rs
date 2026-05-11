use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, FontWeight, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, px,
};
use r3_core::{APP_NAME, AppSnapshot, MessageAuthor, ThreadStatus};

use crate::theme::{FONT_FAMILY, SIDEBAR_MIN_WIDTH, Theme, ThemeMode};

pub struct R3Shell {
    snapshot: AppSnapshot,
    theme: Theme,
    theme_mode: ThemeMode,
    screen: R3Screen,
}

impl R3Shell {
    pub fn new(snapshot: AppSnapshot, screen: R3Screen, theme_mode: ThemeMode) -> Self {
        Self {
            snapshot,
            theme: Theme::light(),
            theme_mode,
            screen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R3Screen {
    Empty,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsControl {
    Select(&'static str),
    Toggle(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SettingsRow {
    label: &'static str,
    description: &'static str,
    control: SettingsControl,
}

impl Render for R3Shell {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.theme = self.theme_mode.resolve(window);

        div()
            .flex()
            .h_full()
            .w_full()
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .font_family(SharedString::from(FONT_FAMILY))
            .when(self.screen == R3Screen::Empty, |element| {
                element.child(self.sidebar()).child(self.main_panel())
            })
            .when(self.screen == R3Screen::Settings, |element| {
                element
                    .child(self.settings_sidebar())
                    .child(self.settings_panel())
            })
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
                        .child("Sort")
                        .child("Add"),
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
                    .child("Settings"),
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

    fn settings_sidebar(&self) -> impl IntoElement {
        let nav_items = [
            ("General", true),
            ("Keybindings", false),
            ("Providers", false),
            ("Source Control", false),
            ("Connections", false),
            ("Archive", false),
        ];

        let mut sidebar = div()
            .flex()
            .flex_col()
            .h_full()
            .border_r_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .w(px(SIDEBAR_MIN_WIDTH))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_5()
                    .pt_4()
                    .pb_6()
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

        for (label, active) in nav_items {
            sidebar = sidebar.child(
                div()
                    .flex()
                    .items_center()
                    .px_5()
                    .pb_4()
                    .text_size(px(14.0))
                    .text_color(if active {
                        self.theme.foreground
                    } else {
                        self.theme.muted_foreground
                    })
                    .child(label),
            );
        }

        sidebar.child(
            div().flex_1().child("").child(
                div()
                    .absolute()
                    .bottom_4()
                    .left_5()
                    .flex()
                    .gap_2()
                    .text_size(px(13.0))
                    .text_color(self.theme.muted_foreground)
                    .child("Back"),
            ),
        )
    }

    fn settings_panel(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(41.0))
                    .px_5()
                    .border_b_1()
                    .border_color(self.theme.border)
                    .child(div().text_size(px(14.0)).child("Settings"))
                    .child(
                        div()
                            .rounded(px(7.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .px_2()
                            .py_1()
                            .text_size(px(13.0))
                            .text_color(self.theme.muted_foreground)
                            .child("Restore defaults"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .pt_8()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .w(px(768.0))
                            .pb_3()
                            .child(div().h(px(1.0)).w(px(12.0)).bg(self.theme.border))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child("GENERAL"),
                            ),
                    )
                    .child(self.settings_card()),
            )
    }

    fn settings_card(&self) -> impl IntoElement {
        let rows = [
            SettingsRow {
                label: "Theme",
                description: "Choose how R3Code looks across the app.",
                control: SettingsControl::Select("System"),
            },
            SettingsRow {
                label: "Time format",
                description: "System default follows your browser or OS clock preference.",
                control: SettingsControl::Select("System default"),
            },
            SettingsRow {
                label: "Diff line wrapping",
                description: "Set the default wrap state when the diff panel opens.",
                control: SettingsControl::Toggle(false),
            },
            SettingsRow {
                label: "Hide whitespace changes",
                description: "Set whether the diff panel ignores whitespace-only edits by default.",
                control: SettingsControl::Toggle(true),
            },
            SettingsRow {
                label: "Assistant output",
                description: "Show token-by-token output while a response is in progress.",
                control: SettingsControl::Toggle(false),
            },
            SettingsRow {
                label: "Auto-open task panel",
                description: "Open the right-side plan and task panel automatically when steps appear.",
                control: SettingsControl::Toggle(true),
            },
            SettingsRow {
                label: "New threads",
                description: "Pick the default workspace mode for newly created draft threads.",
                control: SettingsControl::Select("Local"),
            },
            SettingsRow {
                label: "Add project starts in",
                description: "Leave empty to use \"~/\" when the Add Project browser opens.",
                control: SettingsControl::Select("~/"),
            },
            SettingsRow {
                label: "Archive confirmation",
                description: "Require a second click on the inline archive action before a thread is archived.",
                control: SettingsControl::Toggle(false),
            },
            SettingsRow {
                label: "Delete confirmation",
                description: "Ask before deleting a thread and its chat history.",
                control: SettingsControl::Toggle(true),
            },
        ];

        let mut card = div()
            .flex()
            .flex_col()
            .w(px(768.0))
            .rounded(px(14.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background);

        for row in rows {
            card = card.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .min_h(px(69.0))
                    .px_5()
                    .border_b_1()
                    .border_color(self.theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight(650.0))
                                    .child(row.label),
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child(row.description),
                            ),
                    )
                    .child(self.settings_value(row.control)),
            );
        }

        card
    }

    fn settings_value(&self, control: SettingsControl) -> impl IntoElement {
        match control {
            SettingsControl::Toggle(is_on) => div()
                .w(px(30.0))
                .h(px(18.0))
                .rounded(px(9.0))
                .bg(if is_on {
                    self.theme.primary
                } else {
                    self.theme.accent
                })
                .child(
                    div()
                        .absolute()
                        .top(px(1.0))
                        .left(if is_on { px(13.0) } else { px(1.0) })
                        .w(px(16.0))
                        .h(px(16.0))
                        .rounded(px(8.0))
                        .bg(self.theme.background),
                )
                .into_any_element(),
            SettingsControl::Select(value) => div()
                .min_w(px(160.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(self.theme.border)
                .px_3()
                .py_2()
                .text_size(px(14.0))
                .child(value)
                .into_any_element(),
        }
    }
}

pub fn open_main_window(cx: &mut App) {
    let screen = match std::env::var("R3CODE_SCREEN").as_deref() {
        Ok("settings") => R3Screen::Settings,
        _ => R3Screen::Empty,
    };
    let theme_mode = ThemeMode::from_env();
    let bounds = gpui::Bounds::centered(None, gpui::size(px(1280.0), px(800.0)), cx);
    cx.open_window(
        gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: None,
            ..Default::default()
        },
        move |window, cx| {
            cx.new(|cx| {
                cx.observe_window_appearance(window, |_, window, _| {
                    window.refresh();
                })
                .detach();
                R3Shell::new(AppSnapshot::empty_reference_state(), screen, theme_mode)
            })
        },
    )
    .expect("failed to open R3Code window");
}

use gpui::prelude::{InteractiveElement, StatefulInteractiveElement};
use gpui::{
    App, AppContext, BoxShadow, Context, CursorStyle, FocusHandle, Focusable, FontWeight,
    IntoElement, KeyDownEvent, ParentElement, Render, SharedString, Styled, TextAlign, Window, div,
    hsla, point, px, svg,
};
use r3_core::{APP_NAME, AppSnapshot, MessageAuthor, ThreadStatus};

use crate::theme::{FONT_FAMILY, SIDEBAR_MIN_WIDTH, Theme, ThemeMode};

pub struct R3Shell {
    snapshot: AppSnapshot,
    project_sort_ascending: bool,
    theme: Theme,
    theme_mode: ThemeMode,
    screen: R3Screen,
    command_palette_open: bool,
    command_palette_query: String,
    command_palette_highlighted_index: usize,
    settings_select_open: Option<SettingsSelect>,
    settings_section: SettingsSection,
    source_control_scan_requested: bool,
    providers_refresh_requested: bool,
    providers_add_dialog_open: bool,
    expanded_provider_index: Option<usize>,
    provider_enabled: [bool; 4],
    settings_theme_highlighted_index: usize,
    shell_focus_handle: FocusHandle,
    command_palette_focus_handle: FocusHandle,
    settings_theme_select_focus_handle: FocusHandle,
}

impl R3Shell {
    pub fn new(
        snapshot: AppSnapshot,
        screen: R3Screen,
        theme_mode: ThemeMode,
        command_palette_open: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            snapshot,
            project_sort_ascending: true,
            theme: Theme::light(),
            theme_mode,
            screen,
            command_palette_open,
            command_palette_query: String::new(),
            command_palette_highlighted_index: 0,
            settings_select_open: None,
            settings_section: SettingsSection::General,
            source_control_scan_requested: false,
            providers_refresh_requested: false,
            providers_add_dialog_open: false,
            expanded_provider_index: Some(0),
            provider_enabled: [true, true, false, true],
            settings_theme_highlighted_index: 0,
            shell_focus_handle: cx.focus_handle(),
            command_palette_focus_handle: cx.focus_handle(),
            settings_theme_select_focus_handle: cx.focus_handle(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R3Screen {
    Empty,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandPaletteAction {
    AddProject,
    OpenSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSelect {
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    General,
    Keybindings,
    Providers,
    SourceControl,
    Connections,
    Archive,
}

impl SettingsSection {
    fn id(self) -> &'static str {
        match self {
            Self::General => "settings-nav-general",
            Self::Keybindings => "settings-nav-keybindings",
            Self::Providers => "settings-nav-providers",
            Self::SourceControl => "settings-nav-source-control",
            Self::Connections => "settings-nav-connections",
            Self::Archive => "settings-nav-archive",
        }
    }

    fn from_shortcut_key(key: &str) -> Option<Self> {
        match key {
            "1" => Some(Self::General),
            "2" => Some(Self::Keybindings),
            "3" => Some(Self::Providers),
            "4" => Some(Self::SourceControl),
            "5" => Some(Self::Connections),
            "6" => Some(Self::Archive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsNavIcon {
    Settings,
    Keyboard,
    Bot,
    GitBranch,
    Link,
    Archive,
}

impl SettingsNavIcon {
    fn path(self) -> &'static str {
        match self {
            Self::Settings => "icons/settings-2.svg",
            Self::Keyboard => "icons/keyboard.svg",
            Self::Bot => "icons/bot.svg",
            Self::GitBranch => "icons/git-branch.svg",
            Self::Link => "icons/link-2.svg",
            Self::Archive => "icons/archive.svg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SettingsNavItem {
    label: &'static str,
    icon: SettingsNavIcon,
    section: SettingsSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeybindingRow {
    command: &'static str,
    key: &'static str,
    when: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProviderInstanceRow {
    label: &'static str,
    id: &'static str,
    driver: &'static str,
    status: ProviderStatus,
    badge: Option<&'static str>,
    description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderStatus {
    Ready,
    NotConfigured,
    EarlyAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CommandPaletteCommand {
    title: &'static str,
    search_terms: &'static [&'static str],
    action: CommandPaletteAction,
}

const COMMAND_PALETTE_COMMANDS: &[CommandPaletteCommand] = &[
    CommandPaletteCommand {
        title: "Add project",
        search_terms: &[
            "add project",
            "folder",
            "directory",
            "browse",
            "clone",
            "repository",
            "repo",
            "git",
            "github",
        ],
        action: CommandPaletteAction::AddProject,
    },
    CommandPaletteCommand {
        title: "Open settings",
        search_terms: &["settings", "preferences", "configuration", "keybindings"],
        action: CommandPaletteAction::OpenSettings,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsControl {
    Theme,
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.theme = self.theme_mode.resolve(window);

        let mut root = div()
            .flex()
            .relative()
            .key_context("R3Shell")
            .track_focus(&self.shell_focus_handle)
            .on_key_down(cx.listener(Self::on_shell_key_down))
            .h_full()
            .w_full()
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .font_family(SharedString::from(FONT_FAMILY));

        root = match self.screen {
            R3Screen::Empty => root.child(self.sidebar(cx)).child(self.main_panel()),
            R3Screen::Settings => root
                .child(self.settings_sidebar(cx))
                .child(self.settings_panel(cx)),
        };

        if self.command_palette_open {
            root = root.child(self.command_palette_overlay(cx));
        }

        root
    }
}

impl R3Shell {
    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                .id("command-palette-trigger")
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .pb_6()
                .cursor_pointer()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.open_command_palette(window, cx);
                }))
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
                        .text_color(self.theme.muted_foreground)
                        .child(self.sidebar_icon_button(
                            "project-sort",
                            "icons/arrow-up-down.svg",
                            cx,
                        ))
                        .child(self.sidebar_icon_button(
                            "project-add",
                            "icons/plus-square.svg",
                            cx,
                        )),
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

        let mut projects: Vec<_> = self.snapshot.projects.iter().collect();
        projects.sort_by(|left, right| {
            if self.project_sort_ascending {
                left.name.cmp(&right.name)
            } else {
                right.name.cmp(&left.name)
            }
        });

        for project in projects {
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

    fn settings_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_items = [
            SettingsNavItem {
                label: "General",
                icon: SettingsNavIcon::Settings,
                section: SettingsSection::General,
            },
            SettingsNavItem {
                label: "Keybindings",
                icon: SettingsNavIcon::Keyboard,
                section: SettingsSection::Keybindings,
            },
            SettingsNavItem {
                label: "Providers",
                icon: SettingsNavIcon::Bot,
                section: SettingsSection::Providers,
            },
            SettingsNavItem {
                label: "Source Control",
                icon: SettingsNavIcon::GitBranch,
                section: SettingsSection::SourceControl,
            },
            SettingsNavItem {
                label: "Connections",
                icon: SettingsNavIcon::Link,
                section: SettingsSection::Connections,
            },
            SettingsNavItem {
                label: "Archive",
                icon: SettingsNavIcon::Archive,
                section: SettingsSection::Archive,
            },
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
                    .px_4()
                    .py_3()
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

        let mut nav = div().flex().flex_col().px_2().py_1();
        for item in nav_items {
            let active = self.settings_section == item.section;
            nav = nav.child(
                div()
                    .id(item.section.id())
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .rounded(px(6.0))
                    .px_2p5()
                    .py_1p5()
                    .text_size(px(13.0))
                    .font_weight(if active {
                        FontWeight(500.0)
                    } else {
                        FontWeight(400.0)
                    })
                    .text_color(if active {
                        self.theme.foreground
                    } else {
                        self.theme.muted_foreground
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_settings_section(item.section, cx);
                    }))
                    .child(self.settings_nav_icon(item.icon, active))
                    .child(div().min_w_0().overflow_hidden().child(item.label)),
            );
        }
        sidebar = sidebar.child(nav);

        sidebar.child(
            div().flex_1().child("").child(
                div()
                    .id("settings-back")
                    .absolute()
                    .bottom_2()
                    .left_2()
                    .right_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(6.0))
                    .px_2()
                    .py_2()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.close_settings(window, cx);
                    }))
                    .child(
                        svg()
                            .path("icons/arrow-left.svg")
                            .size_4()
                            .flex_shrink_0()
                            .text_color(self.theme.muted_foreground),
                    )
                    .child("Back"),
            ),
        )
    }

    fn settings_nav_icon(&self, icon: SettingsNavIcon, active: bool) -> impl IntoElement {
        svg()
            .path(icon.path())
            .size_4()
            .flex_shrink_0()
            .text_color(if active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground
            })
    }

    fn settings_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(match self.settings_section {
                SettingsSection::General => self.settings_general_panel(cx).into_any_element(),
                SettingsSection::Keybindings => {
                    self.settings_keybindings_panel().into_any_element()
                }
                SettingsSection::Providers => self.settings_providers_panel(cx).into_any_element(),
                SettingsSection::SourceControl => {
                    self.settings_source_control_panel(cx).into_any_element()
                }
                SettingsSection::Archive => self.settings_archive_panel().into_any_element(),
                section => self.settings_placeholder_panel(section).into_any_element(),
            })
    }

    fn settings_general_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(self.settings_card(cx))
    }

    fn settings_keybindings_panel(&self) -> impl IntoElement {
        let rows = keybinding_rows();
        let mut table = div()
            .flex()
            .flex_col()
            .min_w(px(680.0))
            .child(self.keybindings_table_header());

        for (index, row) in rows.iter().enumerate() {
            table = table.child(self.keybindings_table_row(*row, index));
        }

        div()
            .id("settings-keybindings-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .overflow_y_scroll()
            .pt(px(32.0))
            .px_8()
            .pb_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(960.0))
                    .gap_2p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight(600.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child(div().h(px(1.0)).w(px(12.0)).bg(self.theme.border))
                                    .child("KEYBINDINGS"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_size(px(11.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child(format!("{} bindings", rows.len()))
                                    .child(self.header_icon_button("+"))
                                    .child(self.header_icon_button("{}")),
                            ),
                    )
                    .child(
                        div()
                            .relative()
                            .overflow_hidden()
                            .rounded(px(16.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .bg(self.theme.card)
                            .child(self.keybindings_warning_banner())
                            .child(table),
                    ),
            )
    }

    fn settings_providers_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = provider_instance_rows();
        let mut card = div()
            .relative()
            .overflow_hidden()
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card);

        for (index, row) in rows.iter().enumerate() {
            card = card.child(self.provider_instance_row(index, *row, cx));
            if self.expanded_provider_index == Some(index) {
                card = card.child(self.provider_instance_details(*row));
            }
        }

        let mut content = div()
            .id("settings-providers-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .overflow_y_scroll()
            .p_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(768.0))
                    .gap_2p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_1()
                            .child(self.settings_section_header("PROVIDERS"))
                            .child(self.providers_header_actions(cx)),
                    )
                    .child(card),
            );

        if self.providers_add_dialog_open {
            content = content.child(self.provider_add_panel(cx));
        }

        content
    }

    fn providers_header_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut actions = div().flex().items_center().gap_1p5();

        if self.providers_refresh_requested {
            actions = actions.child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground.opacity(0.72))
                    .child("Checked just now"),
            );
        }

        actions
            .child(
                div()
                    .id("providers-add-instance")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.providers_add_dialog_open = !this.providers_add_dialog_open;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/plus-square.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground),
                    ),
            )
            .child(
                div()
                    .id("providers-refresh")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.providers_refresh_requested = true;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/refresh-cw.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground),
                    ),
            )
    }

    fn provider_instance_row(
        &self,
        index: usize,
        row: ProviderInstanceRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let expanded = self.expanded_provider_index == Some(index);
        div()
            .id(row.id)
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(72.0))
            .border_t_1()
            .border_color(if index == 0 {
                self.theme.card
            } else {
                self.theme.border
            })
            .px_5()
            .py_3p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .min_w_0()
                    .child(self.provider_instance_icon(row))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .min_w_0()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight(650.0))
                                            .child(row.label),
                                    )
                                    .child(self.provider_status_badge(row.status))
                                    .child(match row.badge {
                                        Some(label) => {
                                            self.provider_warning_badge(label).into_any_element()
                                        }
                                        None => div().into_any_element(),
                                    }),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(self.theme.muted_foreground.opacity(0.82))
                                    .child(row.description),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.provider_enabled_switch(index, cx))
                    .child(
                        div()
                            .id(match index {
                                0 => "provider-expand-codex",
                                1 => "provider-expand-claude",
                                2 => "provider-expand-cursor",
                                _ => "provider-expand-opencode",
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(28.0))
                            .h(px(28.0))
                            .rounded(px(7.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.expanded_provider_index =
                                    if this.expanded_provider_index == Some(index) {
                                        None
                                    } else {
                                        Some(index)
                                    };
                                cx.notify();
                            }))
                            .child(
                                svg()
                                    .path(if expanded {
                                        "icons/chevron-down.svg"
                                    } else {
                                        "icons/chevron-right.svg"
                                    })
                                    .size_4()
                                    .text_color(self.theme.muted_foreground),
                            ),
                    ),
            )
    }

    fn provider_instance_details(&self, row: ProviderInstanceRow) -> impl IntoElement {
        div()
            .border_t_1()
            .border_color(self.theme.border)
            .bg(self.theme.background.opacity(0.34))
            .px_5()
            .py_4()
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(self.provider_detail_cell("Driver", row.driver))
                    .child(self.provider_detail_cell("Models", "Default list"))
                    .child(self.provider_detail_cell("Environment", "No overrides"))
                    .child(self.provider_detail_cell("Accent color", "Default")),
            )
    }

    fn provider_detail_cell(&self, label: &'static str, value: &'static str) -> impl IntoElement {
        div()
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .px_3()
            .py_2p5()
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground)
                    .child(label),
            )
            .child(
                div()
                    .mt_1()
                    .text_size(px(13.0))
                    .font_weight(FontWeight(550.0))
                    .child(value),
            )
    }

    fn provider_instance_icon(&self, row: ProviderInstanceRow) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(34.0))
            .h(px(34.0))
            .rounded(px(9.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .child(
                svg()
                    .path("icons/bot.svg")
                    .size_4()
                    .text_color(match row.status {
                        ProviderStatus::Ready => self.theme.foreground,
                        ProviderStatus::NotConfigured => self.theme.muted_foreground,
                        ProviderStatus::EarlyAccess => self.theme.muted_foreground,
                    }),
            )
    }

    fn provider_status_badge(&self, status: ProviderStatus) -> impl IntoElement {
        let (label, color) = match status {
            ProviderStatus::Ready => ("Ready", self.theme.primary.opacity(0.88)),
            ProviderStatus::NotConfigured => ("Not configured", self.theme.muted_foreground),
            ProviderStatus::EarlyAccess => ("Preview", self.theme.muted_foreground),
        };

        div()
            .rounded(px(9.0))
            .border_1()
            .border_color(self.theme.border)
            .px_2()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(color)
            .child(label)
    }

    fn provider_warning_badge(&self, label: &'static str) -> impl IntoElement {
        div()
            .rounded(px(9.0))
            .border_1()
            .border_color(hsla(36.0 / 360.0, 1.0, 0.50, 0.38))
            .bg(hsla(36.0 / 360.0, 1.0, 0.58, 0.06))
            .px_2()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(hsla(36.0 / 360.0, 1.0, 0.42, 1.0))
            .child(label)
    }

    fn provider_enabled_switch(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let enabled = self.provider_enabled[index];
        div()
            .id(match index {
                0 => "provider-toggle-codex",
                1 => "provider-toggle-claude",
                2 => "provider-toggle-cursor",
                _ => "provider-toggle-opencode",
            })
            .relative()
            .w(px(30.0))
            .h(px(18.0))
            .rounded(px(9.0))
            .cursor_pointer()
            .bg(if enabled {
                self.theme.primary
            } else {
                self.theme.accent
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(value) = this.provider_enabled.get_mut(index) {
                    *value = !*value;
                    cx.notify();
                }
            }))
            .child(
                div()
                    .absolute()
                    .top(px(1.0))
                    .left(if enabled { px(13.0) } else { px(1.0) })
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded(px(8.0))
                    .bg(self.theme.background),
            )
    }

    fn provider_add_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top(px(88.0))
            .right(px(48.0))
            .w(px(280.0))
            .rounded(px(12.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .p_2()
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.10),
                offset: point(px(0.0), px(12.0)),
                blur_radius: px(22.0),
                spread_radius: px(-8.0),
            }])
            .child(
                div()
                    .px_2()
                    .py_2()
                    .text_size(px(12.0))
                    .font_weight(FontWeight(650.0))
                    .child("Add provider instance"),
            )
            .child(self.provider_add_option("Codex", 0, cx))
            .child(self.provider_add_option("Claude", 1, cx))
            .child(self.provider_add_option("OpenCode", 3, cx))
    }

    fn provider_add_option(
        &self,
        label: &'static str,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(match index {
                0 => "provider-add-codex",
                1 => "provider-add-claude",
                2 => "provider-add-cursor",
                _ => "provider-add-opencode",
            })
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(7.0))
            .px_2()
            .py_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.providers_add_dialog_open = false;
                this.expanded_provider_index = Some(index);
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/bot.svg")
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
            .child(div().text_size(px(13.0)).child(label))
    }

    fn keybindings_warning_banner(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(hsla(36.0 / 360.0, 0.95, 0.58, 0.24))
            .bg(hsla(36.0 / 360.0, 1.0, 0.58, 0.05))
            .px_4()
            .py_2p5()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(hsla(36.0 / 360.0, 1.0, 0.50, 1.0))
                    .text_size(px(11.0))
                    .text_color(hsla(36.0 / 360.0, 1.0, 0.50, 1.0))
                    .child("!"),
            )
            .child(
                "Some shortcuts may be claimed by the browser before R3Code sees them. Use the desktop app for better keybinding support.",
            )
    }

    fn keybindings_table_header(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(self.theme.border)
            .bg(hsla(0.0, 0.0, 0.0, 0.025))
            .px_4()
            .py_2()
            .text_size(px(11.0))
            .font_weight(FontWeight(650.0))
            .text_color(self.theme.muted_foreground)
            .child(div().w(px(322.0)).child("Command"))
            .child(div().w(px(250.0)).child("Keybinding"))
            .child(div().w(px(294.0)).child("When"))
            .child(div().w(px(60.0)).child("Status"))
    }

    fn keybindings_table_row(&self, row: KeybindingRow, index: usize) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .min_h(px(40.0))
            .border_b_1()
            .border_color(self.theme.border)
            .bg(if index % 2 == 1 {
                hsla(0.0, 0.0, 0.0, 0.015)
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            })
            .px_4()
            .py_1p5()
            .child(
                div()
                    .w(px(322.0))
                    .pr_4()
                    .text_size(px(13.0))
                    .font_weight(FontWeight(500.0))
                    .child(row.command),
            )
            .child(
                div()
                    .w(px(250.0))
                    .pr_4()
                    .child(self.keybinding_pill(row.key)),
            )
            .child(div().w(px(300.0)).pr_4().child(self.when_pill(row.when)))
            .child(
                div()
                    .w(px(60.0))
                    .flex()
                    .justify_end()
                    .child(if row.when == "modelPickerOpen" {
                        self.keybinding_warning_mark().into_any_element()
                    } else {
                        div().into_any_element()
                    }),
            )
    }

    fn keybinding_pill(&self, value: &'static str) -> impl IntoElement {
        let mut group = div().flex().items_center().gap_1();
        for part in value.split('+') {
            let label = match part {
                "mod" => "Ctrl".to_string(),
                "shift" => "^".to_string(),
                "alt" => "Alt".to_string(),
                "ctrl" => "Ctrl".to_string(),
                value if value.len() == 1 => value.to_ascii_uppercase(),
                value => value.to_string(),
            };
            group = group.child(
                div()
                    .min_w(px(24.0))
                    .rounded(px(5.0))
                    .bg(hsla(0.0, 0.0, 0.0, 0.05))
                    .px_1p5()
                    .py_0p5()
                    .text_align(TextAlign::Center)
                    .text_size(px(11.0))
                    .font_weight(FontWeight(600.0))
                    .text_color(self.theme.muted_foreground)
                    .child(label),
            );
        }
        group
    }

    fn when_pill(&self, value: &'static str) -> impl IntoElement {
        div()
            .h(px(28.0))
            .w(px(278.0))
            .flex()
            .items_center()
            .justify_between()
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_2p5()
            .text_size(px(12.0))
            .text_color(if value.is_empty() {
                self.theme.muted_foreground
            } else {
                self.theme.foreground
            })
            .child(if value.is_empty() { "Always" } else { value })
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground)
                    .child("v"),
            )
    }

    fn keybinding_warning_mark(&self) -> impl IntoElement {
        div()
            .text_size(px(13.0))
            .text_color(hsla(36.0 / 360.0, 1.0, 0.50, 1.0))
            .child("!")
    }

    fn header_icon_button(&self, label: &'static str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .rounded(px(4.0))
            .text_size(px(11.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn settings_archive_panel(&self) -> impl IntoElement {
        div()
            .id("settings-archive-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .overflow_y_scroll()
            .p_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(768.0))
                    .gap_2p5()
                    .child(self.settings_section_header("ARCHIVED THREADS"))
                    .child(
                        div()
                            .relative()
                            .overflow_hidden()
                            .rounded(px(16.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .bg(self.theme.card)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .min_h(px(76.0))
                                    .px_5()
                                    .py_3p5()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .text_size(px(13.0))
                                                    .font_weight(FontWeight(650.0))
                                                    .child(
                                                        svg()
                                                            .path("icons/archive.svg")
                                                            .size_4()
                                                            .flex_shrink_0()
                                                            .text_color(
                                                                self.theme.muted_foreground,
                                                            ),
                                                    )
                                                    .child("No archived threads"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(12.0))
                                                    .text_color(self.theme.muted_foreground)
                                                    .child("Archived threads will appear here."),
                                            ),
                                    ),
                            ),
                    ),
            )
    }

    fn settings_source_control_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.source_control_scan_requested {
            return self.source_control_empty_panel(cx).into_any_element();
        }

        div()
            .id("settings-source-control-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .overflow_y_scroll()
            .p_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(768.0))
                    .gap_8()
                    .child(self.source_control_skeleton_section("VERSION CONTROL", true, cx))
                    .child(self.source_control_skeleton_section(
                        "SOURCE CONTROL PROVIDERS",
                        false,
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn source_control_empty_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let description = if self.source_control_scan_requested {
            "No Git or hosting integrations were detected. Install Git or configure credentials, then scan again."
        } else {
            "Install Git on the server, add optional hosting integrations or credentials your workspace needs, then rescan."
        };

        div()
            .id("settings-source-control-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .items_center()
            .overflow_y_scroll()
            .p_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(768.0))
                    .gap_2p5()
                    .child(self.settings_section_header("SERVER ENVIRONMENT"))
                    .child(
                        div()
                            .relative()
                            .overflow_hidden()
                            .rounded(px(16.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .bg(self.theme.card)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .min_h(px(352.0))
                                    .gap_6()
                                    .p_12()
                                    .text_align(TextAlign::Center)
                                    .child(self.empty_source_control_media())
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .w(px(384.0))
                                            .child(
                                                div()
                                                    .font_weight(FontWeight(650.0))
                                                    .text_size(px(20.0))
                                                    .child("Nothing detected yet"),
                                            )
                                            .child(
                                                div()
                                                    .mt_1()
                                                    .text_size(px(14.0))
                                                    .text_color(self.theme.muted_foreground)
                                                    .child(description),
                                            ),
                                    )
                                    .child(self.source_control_scan_button(cx)),
                            ),
                    ),
            )
    }

    fn source_control_skeleton_section(
        &self,
        title: &'static str,
        header_action: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .child(self.settings_section_header(title))
                    .child(if header_action {
                        self.source_control_scan_icon_button(cx).into_any_element()
                    } else {
                        div().w(px(20.0)).h(px(20.0)).into_any_element()
                    }),
            )
            .child(
                div()
                    .relative()
                    .overflow_hidden()
                    .rounded(px(16.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.card)
                    .child(self.source_control_skeleton_row(true))
                    .child(self.source_control_skeleton_row(false)),
            )
    }

    fn source_control_skeleton_row(&self, first: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(68.0))
            .border_t_1()
            .border_color(if first {
                self.theme.card
            } else {
                self.theme.border
            })
            .px_5()
            .py_3p5()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.skeleton_block(px(18.0), px(18.0), px(9.0)))
                            .child(self.skeleton_block(px(112.0), px(16.0), px(8.0)))
                            .child(self.skeleton_block(px(56.0), px(18.0), px(9.0))),
                    )
                    .child(self.skeleton_block(px(320.0), px(10.0), px(5.0))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.skeleton_block(px(28.0), px(28.0), px(7.0)))
                    .child(self.skeleton_block(px(36.0), px(18.0), px(9.0))),
            )
    }

    fn skeleton_block(
        &self,
        width: gpui::Pixels,
        height: gpui::Pixels,
        radius: gpui::Pixels,
    ) -> impl IntoElement {
        div()
            .w(width)
            .h(height)
            .rounded(radius)
            .bg(self.theme.accent)
    }

    fn empty_source_control_media(&self) -> impl IntoElement {
        div()
            .relative()
            .mb_6()
            .w(px(42.0))
            .h(px(38.0))
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .bottom(px(1.0))
                    .w(px(36.0))
                    .h(px(36.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.card)
                    .opacity(0.78),
            )
            .child(
                div()
                    .absolute()
                    .right(px(0.0))
                    .bottom(px(1.0))
                    .w(px(36.0))
                    .h(px(36.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.card)
                    .opacity(0.78),
            )
            .child(
                div()
                    .absolute()
                    .left(px(3.0))
                    .top(px(0.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(36.0))
                    .h(px(36.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.card)
                    .child(
                        svg()
                            .path("icons/git-pull-request.svg")
                            .size_4()
                            .text_color(self.theme.foreground),
                    ),
            )
    }

    fn source_control_scan_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("source-control-scan")
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(32.0))
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .text_size(px(12.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.source_control_scan_requested = true;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/refresh-cw.svg")
                    .size_4()
                    .text_color(self.theme.foreground),
            )
            .child("Scan")
    }

    fn source_control_scan_icon_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("source-control-rescan")
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.source_control_scan_requested = true;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/refresh-cw.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn settings_placeholder_panel(&self, section: SettingsSection) -> impl IntoElement {
        div().flex().flex_col().items_center().p_8().child(
            div()
                .flex()
                .flex_col()
                .w(px(768.0))
                .gap_2p5()
                .child(
                    self.settings_section_header(
                        settings_section_label(section).to_ascii_uppercase(),
                    ),
                )
                .child(
                    div()
                        .rounded(px(16.0))
                        .border_1()
                        .border_color(self.theme.border)
                        .bg(self.theme.card)
                        .h(px(72.0)),
                ),
        )
    }

    fn settings_section_header(&self, label: impl Into<SharedString>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_1()
            .text_size(px(11.0))
            .font_weight(FontWeight(600.0))
            .text_color(self.theme.muted_foreground)
            .child(div().h(px(1.0)).w(px(12.0)).bg(self.theme.border))
            .child(label.into())
    }

    fn settings_card(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = [
            SettingsRow {
                label: "Theme",
                description: "Choose how R3Code looks across the app.",
                control: SettingsControl::Theme,
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
            .relative()
            .flex()
            .flex_col()
            .w(px(768.0))
            .rounded(px(14.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card);

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
                    .child(self.settings_value(row.control, cx)),
            );
        }

        if self.settings_select_open == Some(SettingsSelect::Theme) {
            card = card.child(self.theme_select_popup(cx));
        }

        card
    }

    fn settings_value(&self, control: SettingsControl, cx: &mut Context<Self>) -> impl IntoElement {
        match control {
            SettingsControl::Theme => self.theme_select(cx).into_any_element(),
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

    fn theme_select(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-theme-select")
            .relative()
            .min_w(px(160.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_2()
            .text_size(px(14.0))
            .track_focus(&self.settings_theme_select_focus_handle)
            .tab_index(0)
            .key_context("SettingsThemeSelect")
            .on_key_down(cx.listener(Self::on_theme_select_key_down))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.toggle_settings_select(SettingsSelect::Theme, cx);
                cx.stop_propagation();
            }))
            .child(self.theme_mode_label())
    }

    fn theme_select_popup(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut popup = div()
            .absolute()
            .top(px(54.0))
            .right(px(20.0))
            .flex()
            .flex_col()
            .w(px(160.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.05),
                offset: point(px(0.0), px(10.0)),
                blur_radius: px(15.0),
                spread_radius: px(-3.0),
            }]);

        for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
            popup = popup.child(self.theme_select_item(mode, cx));
        }

        popup
    }

    fn theme_select_item(&self, mode: ThemeMode, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.theme_mode == mode;
        let highlighted = self.settings_theme_highlighted_index == theme_mode_index(mode);
        div()
            .id(match mode {
                ThemeMode::System => "settings-theme-option-system",
                ThemeMode::Light => "settings-theme-option-light",
                ThemeMode::Dark => "settings-theme-option-dark",
            })
            .flex()
            .items_center()
            .min_h(px(30.0))
            .rounded(px(4.0))
            .px_2()
            .text_size(px(14.0))
            .cursor_pointer()
            .bg(if active || highlighted {
                self.theme.accent
            } else {
                self.theme.background.alpha(0.0)
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_theme_mode(mode, cx);
                cx.stop_propagation();
            }))
            .child(self.theme_mode_label_for(mode))
    }

    fn command_palette_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0))
            .left(px(0.0))
            .flex()
            .flex_col()
            .items_center()
            .px_4()
            .pt(px(80.0))
            .bg(self.theme.background.alpha(0.60))
            .child(self.command_palette_popup(cx))
    }

    fn command_palette_popup(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .overflow_hidden()
            .w(px(576.0))
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.05),
                offset: point(px(0.0), px(10.0)),
                blur_radius: px(15.0),
                spread_radius: px(-3.0),
            }])
            .child(self.command_palette_input(cx))
            .child(self.command_palette_results(cx))
            .child(self.command_palette_footer())
    }

    fn command_palette_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let input_text = if self.command_palette_query.is_empty() {
            "Search commands, projects, and threads...".to_string()
        } else {
            self.command_palette_query.clone()
        };

        div()
            .id("command-palette-input")
            .relative()
            .flex()
            .items_center()
            .key_context("CommandPalette")
            .track_focus(&self.command_palette_focus_handle)
            .on_key_down(cx.listener(Self::on_palette_key_down))
            .cursor(CursorStyle::IBeam)
            .h(px(52.0))
            .px_3()
            .on_click(cx.listener(|this, _, window, _| {
                window.focus(&this.command_palette_focus_handle);
            }))
            .child(
                div()
                    .absolute()
                    .left(px(18.0))
                    .top(px(18.0))
                    .child(self.search_icon()),
            )
            .child(
                div()
                    .pl(px(34.0))
                    .text_size(px(15.0))
                    .text_color(if self.command_palette_query.is_empty() {
                        self.theme.muted_foreground
                    } else {
                        self.theme.foreground
                    })
                    .child(input_text),
            )
    }

    fn command_palette_results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let commands = self.filtered_command_items();
        let mut panel = div()
            .flex()
            .flex_col()
            .mx(px(-1.0))
            .border_1()
            .border_b_0()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_2()
            .child(self.command_group_label("Actions"));

        if commands.is_empty() {
            return panel
                .child(
                    div()
                        .py_10()
                        .text_align(TextAlign::Center)
                        .text_size(px(14.0))
                        .text_color(self.theme.muted_foreground)
                        .child("No matching commands, projects, or threads."),
                )
                .into_any_element();
        }

        for (index, command) in commands.into_iter().enumerate() {
            panel = panel.child(self.command_palette_row(
                command,
                index == self.command_palette_highlighted_index,
                cx,
            ));
        }

        panel.into_any_element()
    }

    fn command_group_label(&self, label: &'static str) -> impl IntoElement {
        div()
            .px_2()
            .py_1p5()
            .text_size(px(12.0))
            .font_weight(FontWeight(600.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn command_palette_row(
        &self,
        command: &'static CommandPaletteCommand,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let row_id = match command.action {
            CommandPaletteAction::AddProject => "command-palette-row-add-project",
            CommandPaletteAction::OpenSettings => "command-palette-row-open-settings",
        };

        div()
            .id(row_id)
            .flex()
            .items_center()
            .gap_2()
            .min_h(px(30.0))
            .rounded(px(3.0))
            .px_2()
            .py_1p5()
            .cursor_pointer()
            .bg(if active {
                self.theme.accent
            } else {
                self.theme.background.alpha(0.0)
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.execute_palette_action(command.action, window, cx);
            }))
            .child(self.palette_item_icon(active))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(14.0))
                    .text_color(self.theme.foreground)
                    .child(command.title),
            )
    }

    fn command_palette_footer(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_3()
            .border_t_1()
            .border_color(self.theme.border)
            .px_5()
            .py_3()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .child(self.footer_shortcut(&["Up", "Down"], "Navigate"))
            .child(self.footer_shortcut(&["Enter"], "Select"))
            .child(self.footer_shortcut(&["Esc"], "Close"))
    }

    fn footer_shortcut(&self, keys: &[&'static str], label: &'static str) -> impl IntoElement {
        let mut group = div().flex().items_center().gap_1p5();
        for key in keys {
            group = group.child(self.kbd(key));
        }
        group.child(div().text_color(self.theme.muted_foreground).child(label))
    }

    fn kbd(&self, label: &'static str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .min_w(px(24.0))
            .h(px(20.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.accent)
            .px_1p5()
            .text_size(px(11.0))
            .font_weight(FontWeight(600.0))
            .child(label)
    }

    fn search_icon(&self) -> impl IntoElement {
        div()
            .relative()
            .w(px(16.0))
            .h(px(16.0))
            .child(
                div()
                    .absolute()
                    .left(px(2.0))
                    .top(px(2.0))
                    .w(px(9.0))
                    .h(px(9.0))
                    .rounded(px(5.0))
                    .border_1()
                    .border_color(self.theme.muted_foreground.opacity(0.55)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(10.0))
                    .top(px(11.0))
                    .w(px(5.0))
                    .h(px(1.0))
                    .bg(self.theme.muted_foreground.opacity(0.55)),
            )
    }

    fn sidebar_icon_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(16.0))
            .h(px(16.0))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| {
                match id {
                    "project-add" => {
                        this.open_command_palette(window, cx);
                        this.execute_palette_action(CommandPaletteAction::AddProject, window, cx);
                    }
                    "project-sort" => {
                        this.project_sort_ascending = !this.project_sort_ascending;
                        cx.notify();
                    }
                    _ => {}
                }
                cx.stop_propagation();
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn palette_item_icon(&self, active: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(16.0))
            .h(px(16.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(self.theme.muted_foreground.opacity(0.32))
            .bg(if active {
                self.theme.background.alpha(0.40)
            } else {
                self.theme.accent
            })
    }

    fn theme_mode_label(&self) -> &'static str {
        self.theme_mode_label_for(self.theme_mode)
    }

    fn theme_mode_label_for(&self, mode: ThemeMode) -> &'static str {
        match mode {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    fn toggle_settings_select(&mut self, select: SettingsSelect, cx: &mut Context<Self>) {
        self.settings_select_open = if self.settings_select_open == Some(select) {
            None
        } else {
            if select == SettingsSelect::Theme {
                self.settings_theme_highlighted_index = theme_mode_index(self.theme_mode);
            }
            Some(select)
        };
        cx.notify();
    }

    fn set_theme_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.theme_mode = mode;
        self.settings_theme_highlighted_index = theme_mode_index(mode);
        self.settings_select_open = None;
        cx.notify();
    }

    fn on_theme_select_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.handle_theme_select_key(event.keystroke.key.as_str(), window, cx);
    }

    fn handle_theme_select_key(&mut self, key: &str, _: &mut Window, cx: &mut Context<Self>) {
        match key {
            "enter" | "space" if self.settings_select_open == Some(SettingsSelect::Theme) => {
                self.set_theme_mode(
                    theme_mode_for_index(self.settings_theme_highlighted_index),
                    cx,
                );
                cx.stop_propagation();
            }
            "enter" | "space" => {
                self.toggle_settings_select(SettingsSelect::Theme, cx);
                cx.stop_propagation();
            }
            "down" if self.settings_select_open == Some(SettingsSelect::Theme) => {
                self.settings_theme_highlighted_index =
                    (self.settings_theme_highlighted_index + 1) % 3;
                cx.notify();
                cx.stop_propagation();
            }
            "up" if self.settings_select_open == Some(SettingsSelect::Theme) => {
                self.settings_theme_highlighted_index = self
                    .settings_theme_highlighted_index
                    .checked_sub(1)
                    .unwrap_or(2);
                cx.notify();
                cx.stop_propagation();
            }
            "escape" if self.settings_select_open == Some(SettingsSelect::Theme) => {
                self.settings_select_open = None;
                cx.notify();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn on_shell_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key.as_str() == "escape" && self.settings_select_open.is_some() {
            self.settings_select_open = None;
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if self.screen == R3Screen::Settings && event.keystroke.key.as_str() == "escape" {
            self.close_settings(window, cx);
            cx.stop_propagation();
            return;
        }

        if self.settings_select_open == Some(SettingsSelect::Theme) {
            self.handle_theme_select_key(event.keystroke.key.as_str(), window, cx);
            return;
        }

        if let Some(section) = self.settings_section_shortcut(event) {
            self.set_settings_section(section, cx);
            cx.stop_propagation();
            return;
        }

        if self.screen == R3Screen::Settings
            && event.keystroke.key.as_str() == "tab"
            && self.settings_select_open.is_none()
        {
            window.focus(&self.settings_theme_select_focus_handle);
            cx.stop_propagation();
            return;
        }

        if self.is_settings_theme_shortcut(event) {
            self.toggle_settings_select(SettingsSelect::Theme, cx);
            cx.stop_propagation();
            return;
        }

        if self.is_command_palette_shortcut(event) {
            self.open_command_palette(window, cx);
            cx.stop_propagation();
        }
    }

    fn on_palette_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_command_palette_shortcut(event) {
            self.close_command_palette(window, cx);
            cx.stop_propagation();
            return;
        }

        match event.keystroke.key.as_str() {
            "escape" => self.close_command_palette(window, cx),
            "up" => self.move_palette_highlight(-1, cx),
            "down" => self.move_palette_highlight(1, cx),
            "enter" => self.execute_highlighted_palette_action(window, cx),
            "backspace" => {
                self.command_palette_query.pop();
                self.command_palette_highlighted_index = 0;
                cx.notify();
            }
            _ => {
                let modifiers = event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
                    return;
                }
                if let Some(text) = event.keystroke.key_char.as_deref()
                    && text != "\n"
                    && text != "\t"
                {
                    self.command_palette_query.push_str(text);
                    self.command_palette_highlighted_index = 0;
                    cx.notify();
                }
            }
        }

        cx.stop_propagation();
    }

    fn open_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = true;
        self.command_palette_highlighted_index = 0;
        self.settings_select_open = None;
        window.focus(&self.command_palette_focus_handle);
        cx.notify();
    }

    fn close_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_highlighted_index = 0;
        window.focus(&self.shell_focus_handle);
        cx.notify();
    }

    fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = R3Screen::Empty;
        self.settings_select_open = None;
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_highlighted_index = 0;
        window.focus(&self.shell_focus_handle);
        cx.notify();
    }

    fn set_settings_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.settings_section = section;
        self.settings_select_open = None;
        cx.notify();
    }

    fn is_command_palette_shortcut(&self, event: &KeyDownEvent) -> bool {
        event.keystroke.modifiers.secondary() && event.keystroke.key.eq_ignore_ascii_case("k")
    }

    fn settings_section_shortcut(&self, event: &KeyDownEvent) -> Option<SettingsSection> {
        if self.screen != R3Screen::Settings || !event.keystroke.modifiers.secondary() {
            return None;
        }

        SettingsSection::from_shortcut_key(event.keystroke.key.as_str())
    }

    fn is_settings_theme_shortcut(&self, event: &KeyDownEvent) -> bool {
        self.screen == R3Screen::Settings
            && event.keystroke.modifiers.secondary()
            && event.keystroke.key.eq_ignore_ascii_case("t")
    }

    fn filtered_command_items(&self) -> Vec<&'static CommandPaletteCommand> {
        let query = self.command_palette_query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return COMMAND_PALETTE_COMMANDS.iter().collect();
        }

        COMMAND_PALETTE_COMMANDS
            .iter()
            .filter(|command| {
                command.title.to_ascii_lowercase().contains(&query)
                    || command
                        .search_terms
                        .iter()
                        .any(|term| term.contains(query.as_str()))
            })
            .collect()
    }

    fn move_palette_highlight(&mut self, direction: isize, cx: &mut Context<Self>) {
        let item_count = self.filtered_command_items().len();
        if item_count == 0 {
            self.command_palette_highlighted_index = 0;
            cx.notify();
            return;
        }

        self.command_palette_highlighted_index = if direction < 0 {
            self.command_palette_highlighted_index
                .checked_sub(1)
                .unwrap_or(item_count - 1)
        } else {
            (self.command_palette_highlighted_index + 1) % item_count
        };
        cx.notify();
    }

    fn execute_highlighted_palette_action(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let commands = self.filtered_command_items();
        let Some(command) = commands.get(self.command_palette_highlighted_index) else {
            return;
        };
        self.execute_palette_action(command.action, window, cx);
    }

    fn execute_palette_action(
        &mut self,
        action: CommandPaletteAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            CommandPaletteAction::AddProject => {
                self.command_palette_query = "~/".to_string();
                self.command_palette_highlighted_index = 0;
                window.focus(&self.command_palette_focus_handle);
                cx.notify();
            }
            CommandPaletteAction::OpenSettings => {
                self.screen = R3Screen::Settings;
                self.settings_section = SettingsSection::General;
                self.close_command_palette(window, cx);
            }
        }
    }
}

fn theme_mode_index(mode: ThemeMode) -> usize {
    match mode {
        ThemeMode::System => 0,
        ThemeMode::Light => 1,
        ThemeMode::Dark => 2,
    }
}

fn theme_mode_for_index(index: usize) -> ThemeMode {
    match index {
        1 => ThemeMode::Light,
        2 => ThemeMode::Dark,
        _ => ThemeMode::System,
    }
}

fn settings_section_label(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::General => "General",
        SettingsSection::Keybindings => "Keybindings",
        SettingsSection::Providers => "Providers",
        SettingsSection::SourceControl => "Source Control",
        SettingsSection::Connections => "Connections",
        SettingsSection::Archive => "Archive",
    }
}

fn provider_instance_rows() -> &'static [ProviderInstanceRow] {
    &[
        ProviderInstanceRow {
            label: "Codex",
            id: "provider-row-codex",
            driver: "codex",
            status: ProviderStatus::Ready,
            badge: None,
            description: "Default code agent provider.",
        },
        ProviderInstanceRow {
            label: "Claude",
            id: "provider-row-claude",
            driver: "claudeAgent",
            status: ProviderStatus::NotConfigured,
            badge: None,
            description: "Claude agent bridge.",
        },
        ProviderInstanceRow {
            label: "Cursor",
            id: "provider-row-cursor",
            driver: "cursor",
            status: ProviderStatus::EarlyAccess,
            badge: Some("Early Access"),
            description: "Cursor integration for existing desktop sessions.",
        },
        ProviderInstanceRow {
            label: "OpenCode",
            id: "provider-row-opencode",
            driver: "opencode",
            status: ProviderStatus::Ready,
            badge: None,
            description: "OpenCode CLI provider.",
        },
    ]
}

fn keybinding_rows() -> &'static [KeybindingRow] {
    &[
        KeybindingRow {
            command: "Chat: New",
            key: "mod+n",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Chat: New",
            key: "mod+shift+o",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Chat: New Local",
            key: "mod+shift+n",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Command Palette: Toggle",
            key: "mod+k",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Diff: Toggle",
            key: "mod+d",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Editor: Open Favorite",
            key: "mod+o",
            when: "",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 1",
            key: "mod+1",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 2",
            key: "mod+2",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 3",
            key: "mod+3",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 4",
            key: "mod+4",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 5",
            key: "mod+5",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 6",
            key: "mod+6",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 7",
            key: "mod+7",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 8",
            key: "mod+8",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Jump: 9",
            key: "mod+9",
            when: "modelPickerOpen",
        },
        KeybindingRow {
            command: "Model Picker: Toggle",
            key: "mod+shift+m",
            when: "!terminalFocus",
        },
        KeybindingRow {
            command: "Terminal: Close",
            key: "mod+w",
            when: "terminalFocus",
        },
        KeybindingRow {
            command: "Terminal: New",
            key: "mod+n",
            when: "terminalFocus",
        },
        KeybindingRow {
            command: "Terminal: Split",
            key: "mod+d",
            when: "terminalFocus",
        },
        KeybindingRow {
            command: "Terminal: Toggle",
            key: "mod+j",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 1",
            key: "mod+1",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 2",
            key: "mod+2",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 3",
            key: "mod+3",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 4",
            key: "mod+4",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 5",
            key: "mod+5",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 6",
            key: "mod+6",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 7",
            key: "mod+7",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 8",
            key: "mod+8",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Jump: 9",
            key: "mod+9",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Next",
            key: "mod+shift+]",
            when: "",
        },
        KeybindingRow {
            command: "Thread: Previous",
            key: "mod+shift+[",
            when: "",
        },
    ]
}

impl Focusable for R3Shell {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.shell_focus_handle.clone()
    }
}

pub fn open_main_window(cx: &mut App) {
    let (screen, command_palette_open) = match std::env::var("R3CODE_SCREEN").as_deref() {
        Ok("command-palette") => (R3Screen::Empty, true),
        Ok("settings") => (R3Screen::Settings, false),
        _ => (R3Screen::Empty, false),
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
            let shell = cx.new(|cx| {
                cx.observe_window_appearance(window, |_, window, _| {
                    window.refresh();
                })
                .detach();
                R3Shell::new(
                    AppSnapshot::empty_reference_state(),
                    screen,
                    theme_mode,
                    command_palette_open,
                    cx,
                )
            });
            if command_palette_open {
                let focus_handle = shell.read(cx).command_palette_focus_handle.clone();
                window.focus(&focus_handle);
                window.defer(cx, move |window, _| {
                    window.focus(&focus_handle);
                });
            } else {
                let focus_handle = shell.read(cx).focus_handle(cx);
                window.focus(&focus_handle);
                window.defer(cx, move |window, _| {
                    window.focus(&focus_handle);
                });
            }
            shell
        },
    )
    .expect("failed to open R3Code window");
}

use gpui::prelude::{FluentBuilder, InteractiveElement, StatefulInteractiveElement};
use gpui::{
    AnyElement, App, AppContext, BoxShadow, Context, CursorStyle, FocusHandle, Focusable,
    FontWeight, IntoElement, KeyDownEvent, ParentElement, Render, SharedString, Styled, TextAlign,
    Window, div, hsla, point, px, svg,
};
use r3_core::{
    APP_NAME, AppSnapshot, ChatMessage, DiffOpenValue, DiffRouteSearch, MAX_TERMINALS_PER_GROUP,
    PendingApproval, PendingUserInputProgress, ProjectSummary, TerminalEvent, ThreadStatus,
    TurnDiffFileChange, TurnDiffStat, TurnDiffTreeNode, build_turn_diff_tree,
    close_thread_terminal, new_thread_terminal, parse_diff_route_search,
    set_pending_user_input_custom_answer, set_thread_active_terminal, set_thread_terminal_open,
    split_thread_terminal, summarize_turn_diff_stats, toggle_pending_user_input_option_selection,
};

use crate::theme::{FONT_FAMILY, MONO_FONT_FAMILY, SIDEBAR_MIN_WIDTH, Theme, ThemeMode};

pub struct R3Shell {
    snapshot: AppSnapshot,
    project_sort_ascending: bool,
    theme: Theme,
    theme_mode: ThemeMode,
    screen: R3Screen,
    command_palette_open: bool,
    command_palette_query: String,
    command_palette_highlighted_index: usize,
    keybindings_search_open: bool,
    keybindings_add_dialog_open: bool,
    keybindings_file_opened: bool,
    settings_select_open: Option<SettingsSelect>,
    settings_section: SettingsSection,
    settings_defaults_restored: bool,
    settings_update_checked: bool,
    settings_diagnostics_opened: bool,
    settings_toggle_values: [bool; 6],
    settings_select_values: [usize; 4],
    source_control_scan_requested: bool,
    source_control_git_enabled: bool,
    source_control_git_details_open: bool,
    source_control_fetch_interval_seconds: u32,
    source_control_account_revealed: bool,
    source_control_provider_enabled: [bool; 3],
    providers_refresh_requested: bool,
    providers_add_dialog_open: bool,
    expanded_provider_index: Option<usize>,
    provider_enabled: [bool; 4],
    connections_network_accessible: bool,
    connections_add_dialog_open: bool,
    connections_mode: ConnectionMode,
    connections_saved_environment: bool,
    connections_saved_environment_connected: bool,
    connections_endpoint_copied: bool,
    connections_refresh_requested: bool,
    settings_theme_highlighted_index: usize,
    composer_prompt: String,
    composer_prompt_focused: bool,
    composer_model_index: usize,
    composer_runtime_index: usize,
    composer_plan_mode: bool,
    composer_submitted_count: usize,
    diff_render_split: bool,
    diff_word_wrap: bool,
    diff_ignore_whitespace: bool,
    shell_focus_handle: FocusHandle,
    composer_focus_handle: FocusHandle,
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
            keybindings_search_open: false,
            keybindings_add_dialog_open: false,
            keybindings_file_opened: false,
            settings_select_open: None,
            settings_section: SettingsSection::General,
            settings_defaults_restored: false,
            settings_update_checked: false,
            settings_diagnostics_opened: false,
            settings_toggle_values: [false, true, false, true, false, true],
            settings_select_values: [0, 0, 0, 0],
            source_control_scan_requested: false,
            source_control_git_enabled: true,
            source_control_git_details_open: false,
            source_control_fetch_interval_seconds: SOURCE_CONTROL_DEFAULT_FETCH_INTERVAL_SECONDS,
            source_control_account_revealed: false,
            source_control_provider_enabled: [true, false, false],
            providers_refresh_requested: false,
            providers_add_dialog_open: false,
            expanded_provider_index: Some(0),
            provider_enabled: [true, true, false, true],
            connections_network_accessible: false,
            connections_add_dialog_open: false,
            connections_mode: ConnectionMode::Remote,
            connections_saved_environment: false,
            connections_saved_environment_connected: false,
            connections_endpoint_copied: false,
            connections_refresh_requested: false,
            settings_theme_highlighted_index: 0,
            composer_prompt: String::new(),
            composer_prompt_focused: false,
            composer_model_index: 0,
            composer_runtime_index: 2,
            composer_plan_mode: false,
            composer_submitted_count: 0,
            diff_render_split: false,
            diff_word_wrap: false,
            diff_ignore_whitespace: true,
            shell_focus_handle: cx.focus_handle(),
            composer_focus_handle: cx.focus_handle(),
            command_palette_focus_handle: cx.focus_handle(),
            settings_theme_select_focus_handle: cx.focus_handle(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R3Screen {
    Empty,
    Draft,
    ActiveChat,
    PendingApproval,
    PendingUserInput,
    TerminalDrawer,
    DiffPanel,
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
enum ConnectionMode {
    Remote,
    Ssh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceControlSwitch {
    Git,
    Provider(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceControlFetchAction {
    Decrease,
    Increase,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingApprovalActionKind {
    Cancel,
    Decline,
    AcceptForSession,
    Accept,
}

impl PendingApprovalActionKind {
    fn id(self) -> &'static str {
        match self {
            Self::Cancel => "chat-composer-pending-approval-cancel",
            Self::Decline => "chat-composer-pending-approval-decline",
            Self::AcceptForSession => "chat-composer-pending-approval-accept-session",
            Self::Accept => "chat-composer-pending-approval-accept",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeybindingsHeaderAction {
    ToggleSearch,
    ToggleAdd,
    MarkFileOpened,
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
    Select(SettingsSelectKind),
    Toggle(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSelectKind {
    TimeFormat,
    NewThreads,
    ProjectBase,
    TextGenerationModel,
}

impl SettingsSelectKind {
    fn index(self) -> usize {
        match self {
            Self::TimeFormat => 0,
            Self::NewThreads => 1,
            Self::ProjectBase => 2,
            Self::TextGenerationModel => 3,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::TimeFormat => "settings-select-time-format",
            Self::NewThreads => "settings-select-new-threads",
            Self::ProjectBase => "settings-select-project-base",
            Self::TextGenerationModel => "settings-select-text-generation-model",
        }
    }

    fn options(self) -> &'static [&'static str] {
        match self {
            Self::TimeFormat => &["System default", "12-hour", "24-hour"],
            Self::NewThreads => &["Local", "New worktree"],
            Self::ProjectBase => &["~/", "C:\\Users\\bunny\\Downloads", "Custom"],
            Self::TextGenerationModel => {
                &["Codex / gpt-5", "Claude / Sonnet", "OpenCode / default"]
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SettingsRow {
    label: &'static str,
    description: &'static str,
    control: SettingsControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComposerModel {
    provider: &'static str,
    model: &'static str,
    icon: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComposerRuntimeMode {
    label: &'static str,
    icon: &'static str,
}

const COMPOSER_MODELS: &[ComposerModel] = &[
    ComposerModel {
        provider: "GPT-5.4",
        model: "",
        icon: "icons/bot.svg",
    },
    ComposerModel {
        provider: "Claude",
        model: "Sonnet",
        icon: "icons/bot.svg",
    },
    ComposerModel {
        provider: "OpenCode",
        model: "default",
        icon: "icons/terminal.svg",
    },
];

const COMPOSER_RUNTIME_MODES: &[ComposerRuntimeMode] = &[
    ComposerRuntimeMode {
        label: "Supervised",
        icon: "icons/lock.svg",
    },
    ComposerRuntimeMode {
        label: "Auto-accept edits",
        icon: "icons/pen-line.svg",
    },
    ComposerRuntimeMode {
        label: "Full access",
        icon: "icons/lock-open.svg",
    },
];

const SOURCE_CONTROL_DEFAULT_FETCH_INTERVAL_SECONDS: u32 = 30;
const SOURCE_CONTROL_FETCH_INTERVAL_STEP_SECONDS: u32 = 5;

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
            R3Screen::Empty
            | R3Screen::Draft
            | R3Screen::ActiveChat
            | R3Screen::PendingApproval
            | R3Screen::PendingUserInput
            | R3Screen::TerminalDrawer
            | R3Screen::DiffPanel => root.child(self.sidebar(cx)).child(self.main_panel(cx)),
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
                        .child(
                            svg()
                                .path("icons/search.svg")
                                .size_4()
                                .text_color(self.theme.muted_foreground),
                        )
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
            sidebar = if self.snapshot.renders_chat_view() {
                sidebar.child(self.sidebar_active_project_group(project))
            } else {
                sidebar.child(
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
                )
            };
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
                    .id("sidebar-settings")
                    .absolute()
                    .bottom_4()
                    .left_4()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.screen = R3Screen::Settings;
                        this.settings_section = SettingsSection::General;
                        this.command_palette_open = false;
                        window.focus(&this.shell_focus_handle);
                        cx.notify();
                    }))
                    .child("Settings"),
            ),
        )
    }

    fn main_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(self.toolbar(cx))
            .child(self.timeline());

        if self.snapshot.terminal_open() {
            panel = panel.child(self.terminal_drawer(cx));
        }

        if self.snapshot.renders_chat_view() {
            panel = panel.child(self.composer(cx));
        }

        if self.snapshot.diff_open() {
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .child(panel)
                .child(self.diff_panel(cx))
                .into_any_element()
        } else {
            panel.into_any_element()
        }
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let renders_chat_view = self.snapshot.renders_chat_view();
        let toolbar_title = if renders_chat_view {
            self.snapshot.active_thread_title().to_string()
        } else {
            "No active thread".to_string()
        };
        let project_name = self.snapshot.active_project_name().map(str::to_string);
        let mut toolbar = div()
            .flex()
            .items_center()
            .justify_between()
            .h(if renders_chat_view {
                px(49.0)
            } else {
                px(41.0)
            })
            .px_5()
            .border_b_1()
            .border_color(self.theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .text_size(px(14.0))
                    .font_weight(if renders_chat_view {
                        FontWeight(500.0)
                    } else {
                        FontWeight(400.0)
                    })
                    .text_color(if renders_chat_view {
                        self.theme.foreground
                    } else {
                        self.theme.muted_foreground
                    })
                    .child(toolbar_title)
                    .when_some(project_name, |header, project_name| {
                        header.child(
                            div()
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(self.theme.border)
                                .px_2()
                                .py_0p5()
                                .text_size(px(12.0))
                                .text_color(self.theme.muted_foreground)
                                .child(project_name),
                        )
                    }),
            );

        if renders_chat_view {
            toolbar = toolbar.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.toolbar_icon_button(
                        "thread-terminal",
                        "icons/square-terminal.svg",
                        cx,
                    ))
                    .child(self.toolbar_icon_button("thread-diff", "icons/diff.svg", cx)),
            );
        }

        toolbar
    }

    fn timeline(&self) -> impl IntoElement {
        if !self.snapshot.messages.is_empty() {
            return self.messages_timeline().into_any_element();
        }

        let mut timeline = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .flex_1()
            .p_4();

        if self.snapshot.messages.is_empty() {
            if self.snapshot.renders_chat_view() {
                return timeline
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(self.theme.muted_foreground.opacity(0.30))
                            .child("Send a message to start the conversation."),
                    )
                    .into_any_element();
            }

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
                            .child(message.role.display_author()),
                    )
                    .child(div().text_size(px(14.0)).child(message.text.clone())),
            );
        }

        timeline.into_any_element()
    }

    fn messages_timeline(&self) -> impl IntoElement {
        let content_width = if self.snapshot.diff_open() {
            460.0
        } else {
            760.0
        };
        let mut content = div().flex().flex_col().gap_4().w(px(content_width));

        for message in &self.snapshot.messages {
            content = content.child(self.timeline_message(message));
        }

        div()
            .id("messages-timeline-scroll")
            .flex()
            .flex_col()
            .items_center()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px_5()
            .pt_4()
            .pb_4()
            .child(content)
    }

    fn timeline_message(&self, message: &ChatMessage) -> impl IntoElement {
        match message.role {
            r3_core::MessageRole::User => self.user_timeline_message(message).into_any_element(),
            r3_core::MessageRole::Assistant | r3_core::MessageRole::System => {
                self.assistant_timeline_message(message).into_any_element()
            }
        }
    }

    fn user_timeline_message(&self, message: &ChatMessage) -> impl IntoElement {
        let bubble_width = if self.snapshot.diff_open() {
            420.0
        } else {
            520.0
        };
        div().flex().justify_end().child(
            div()
                .rounded(px(16.0))
                .border_1()
                .border_color(self.theme.border)
                .bg(self.theme.accent)
                .px_4()
                .py_3()
                .w(px(bubble_width))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(self.theme.foreground)
                        .child(message.text.clone()),
                )
                .child(
                    div()
                        .mt_2()
                        .flex()
                        .justify_end()
                        .text_size(px(11.0))
                        .text_color(self.theme.muted_foreground.opacity(0.50))
                        .child("12:00 PM"),
                ),
        )
    }

    fn assistant_timeline_message(&self, message: &ChatMessage) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .px_1()
            .py_0p5()
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(self.theme.foreground)
                    .child(message.text.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(10.0))
                    .text_color(self.theme.muted_foreground.opacity(0.30))
                    .child("12:00 PM"),
            )
    }

    fn terminal_drawer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = &self.snapshot.terminal_state;
        let active_group = state
            .terminal_groups
            .iter()
            .find(|group| group.id == state.active_terminal_group_id)
            .or_else(|| {
                state.terminal_groups.iter().find(|group| {
                    group
                        .terminal_ids
                        .iter()
                        .any(|terminal_id| terminal_id == &state.active_terminal_id)
                })
            })
            .or_else(|| state.terminal_groups.first());
        let visible_terminal_ids = active_group
            .map(|group| group.terminal_ids.clone())
            .unwrap_or_else(|| vec![state.active_terminal_id.clone()]);
        let has_terminal_sidebar = state.terminal_ids.len() > 1;

        div()
            .id("thread-terminal-drawer")
            .relative()
            .flex()
            .flex_col()
            .min_w_0()
            .flex_shrink_0()
            .overflow_hidden()
            .border_t_1()
            .border_color(self.theme.border.opacity(0.80))
            .bg(self.theme.background)
            .h(px(state.terminal_height as f32))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(6.0))
                    .cursor(CursorStyle::ResizeUpDown),
            )
            .when(!has_terminal_sidebar, |drawer| {
                drawer.child(self.terminal_floating_actions(cx))
            })
            .child(
                div().min_h_0().w_full().flex_1().child(
                    div()
                        .flex()
                        .h_full()
                        .min_h_0()
                        .w_full()
                        .gap(if has_terminal_sidebar {
                            px(6.0)
                        } else {
                            px(0.0)
                        })
                        .child(self.terminal_split_view(&visible_terminal_ids, cx))
                        .when(has_terminal_sidebar, |row| {
                            row.child(self.terminal_sidebar(cx))
                        }),
                ),
            )
    }

    fn terminal_split_view(
        &self,
        visible_terminal_ids: &[String],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut row = div().flex().h_full().min_w_0().flex_1().overflow_hidden();
        for (index, terminal_id) in visible_terminal_ids.iter().enumerate() {
            let terminal_id_for_click = terminal_id.clone();
            row = row.child(
                div()
                    .id(SharedString::from(format!("terminal-pane-{index}")))
                    .min_h_0()
                    .min_w_0()
                    .flex_1()
                    .border_l_1()
                    .border_color(
                        if terminal_id == &self.snapshot.terminal_state.active_terminal_id {
                            self.theme.border
                        } else {
                            self.theme.border.opacity(0.70)
                        },
                    )
                    .when(index == 0, |pane| pane.border_l_0())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.snapshot.terminal_state = set_thread_active_terminal(
                            &this.snapshot.terminal_state,
                            &terminal_id_for_click,
                        );
                        cx.notify();
                    }))
                    .child(
                        div()
                            .h_full()
                            .p_1()
                            .child(self.terminal_viewport(terminal_id, index)),
                    ),
            );
        }
        row
    }

    fn terminal_viewport(&self, terminal_id: &str, index: usize) -> impl IntoElement {
        let active = terminal_id == self.snapshot.terminal_state.active_terminal_id;
        let mut body = div()
            .h_full()
            .rounded(px(4.0))
            .border_1()
            .border_color(if active {
                self.theme.border
            } else {
                self.theme.border.opacity(0.70)
            })
            .bg(self.theme.card)
            .p_3()
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(12.0))
            .text_color(self.theme.foreground)
            .overflow_hidden()
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground)
                    .child(
                        svg()
                            .path("icons/square-terminal.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground),
                    )
                    .child(format!("Terminal {}", index + 1)),
            );

        let lines = self.terminal_lines_for(terminal_id);
        for line in lines {
            body = body.child(div().mb_1().child(line));
        }

        body.child(
            div()
                .mt_1()
                .flex()
                .items_center()
                .gap_1()
                .child(">")
                .child(div().w(px(7.0)).h(px(14.0)).bg(self.theme.foreground)),
        )
    }

    fn terminal_lines_for(&self, terminal_id: &str) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(context) = self.snapshot.terminal_launch_context.as_ref() {
            lines.push(format!("cwd {}", context.cwd));
        }
        for entry in &self.snapshot.terminal_event_entries {
            if entry.event.terminal_id() != terminal_id {
                continue;
            }
            match &entry.event {
                TerminalEvent::Started { snapshot, .. }
                | TerminalEvent::Restarted { snapshot, .. } => {
                    lines.push(format!(
                        "[terminal] started pid {}",
                        snapshot.pid.unwrap_or(0)
                    ));
                }
                TerminalEvent::Output { data, .. } => {
                    for line in data.lines() {
                        if !line.trim().is_empty() {
                            lines.push(line.to_string());
                        }
                    }
                }
                TerminalEvent::Activity {
                    has_running_subprocess,
                    ..
                } => {
                    if *has_running_subprocess {
                        lines.push("[terminal] process running".to_string());
                    }
                }
                TerminalEvent::Error { message, .. } => {
                    lines.push(format!("[terminal] {message}"));
                }
                TerminalEvent::Cleared { .. } => {
                    lines.clear();
                }
                TerminalEvent::Exited {
                    exit_code,
                    exit_signal,
                    ..
                } => {
                    lines.push(format!(
                        "[terminal] exited {}",
                        exit_signal
                            .clone()
                            .or_else(|| exit_code.map(|code| code.to_string()))
                            .unwrap_or_else(|| "unknown".to_string())
                    ));
                }
            }
        }
        if lines.is_empty() {
            lines.push("Terminal ready.".to_string());
        }
        lines
    }

    fn terminal_floating_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().absolute().right_2().top_2().child(
            div()
                .flex()
                .items_center()
                .overflow_hidden()
                .rounded(px(6.0))
                .border_1()
                .border_color(self.theme.border.opacity(0.80))
                .bg(self.theme.background.opacity(0.70))
                .child(self.terminal_action_button(
                    "terminal-split",
                    "icons/square-split-horizontal.svg",
                    cx,
                ))
                .child(div().h(px(16.0)).w(px(1.0)).bg(self.theme.border))
                .child(self.terminal_action_button("terminal-new", "icons/plus.svg", cx))
                .child(div().h(px(16.0)).w(px(1.0)).bg(self.theme.border))
                .child(self.terminal_action_button("terminal-close", "icons/trash-2.svg", cx)),
        )
    }

    fn terminal_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut sidebar = div()
            .flex()
            .flex_col()
            .w(px(144.0))
            .min_w(px(144.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.70))
            .bg(self.theme.accent.opacity(0.10))
            .child(
                div()
                    .flex()
                    .h(px(22.0))
                    .items_center()
                    .justify_end()
                    .border_b_1()
                    .border_color(self.theme.border.opacity(0.70))
                    .child(self.terminal_action_button(
                        "terminal-sidebar-split",
                        "icons/square-split-horizontal.svg",
                        cx,
                    ))
                    .child(self.terminal_action_button(
                        "terminal-sidebar-new",
                        "icons/plus.svg",
                        cx,
                    ))
                    .child(self.terminal_action_button(
                        "terminal-sidebar-close",
                        "icons/trash-2.svg",
                        cx,
                    )),
            );

        let show_group_headers = self.snapshot.terminal_state.terminal_groups.len() > 1
            || self
                .snapshot
                .terminal_state
                .terminal_groups
                .iter()
                .any(|group| group.terminal_ids.len() > 1);

        let mut groups = div()
            .id("terminal-sidebar-groups")
            .flex()
            .flex_col()
            .min_h_0()
            .flex_1()
            .overflow_y_scroll()
            .px_1()
            .py_1();
        for (group_index, group) in self
            .snapshot
            .terminal_state
            .terminal_groups
            .iter()
            .enumerate()
        {
            let group_active = group
                .terminal_ids
                .iter()
                .any(|id| id == &self.snapshot.terminal_state.active_terminal_id);
            let group_active_terminal_id = if group_active {
                self.snapshot.terminal_state.active_terminal_id.clone()
            } else {
                group.terminal_ids[0].clone()
            };
            let mut group_node = div().pb_0p5();
            if show_group_headers {
                let group_active_terminal_id = group_active_terminal_id.clone();
                group_node = group_node.child(
                    div()
                        .id(SharedString::from(format!("terminal-group-{group_index}")))
                        .flex()
                        .w_full()
                        .rounded(px(4.0))
                        .px_1()
                        .py_0p5()
                        .text_size(px(10.0))
                        .text_color(if group_active {
                            self.theme.foreground
                        } else {
                            self.theme.muted_foreground
                        })
                        .bg(if group_active {
                            self.theme.accent.opacity(0.70)
                        } else {
                            self.theme.background.alpha(0.0)
                        })
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.snapshot.terminal_state = set_thread_active_terminal(
                                &this.snapshot.terminal_state,
                                &group_active_terminal_id,
                            );
                            cx.notify();
                        }))
                        .child(if group.terminal_ids.len() > 1 {
                            format!("SPLIT {}", group_index + 1)
                        } else {
                            format!("TERMINAL {}", group_index + 1)
                        }),
                );
            }

            let mut terminals = div().ml_1().border_l_1().border_color(self.theme.border);
            for terminal_id in &group.terminal_ids {
                terminals = terminals.child(self.terminal_sidebar_row(terminal_id, cx));
            }
            groups = groups.child(group_node.child(terminals));
        }
        sidebar = sidebar.child(groups);
        sidebar
    }

    fn terminal_sidebar_row(&self, terminal_id: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let terminal_id_for_click = terminal_id.to_string();
        let active = terminal_id == self.snapshot.terminal_state.active_terminal_id;
        let label = self.terminal_label(terminal_id);
        div()
            .id(SharedString::from(format!(
                "terminal-sidebar-row-{terminal_id}"
            )))
            .group("terminal-sidebar-row")
            .flex()
            .items_center()
            .gap_1()
            .rounded(px(4.0))
            .px_1()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(if active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground
            })
            .bg(if active {
                self.theme.accent
            } else {
                self.theme.background.alpha(0.0)
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.snapshot.terminal_state = set_thread_active_terminal(
                    &this.snapshot.terminal_state,
                    &terminal_id_for_click,
                );
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/square-terminal.svg")
                    .size_3()
                    .flex_shrink_0()
                    .text_color(if active {
                        self.theme.foreground
                    } else {
                        self.theme.muted_foreground
                    }),
            )
            .child(div().min_w_0().flex_1().overflow_hidden().child(label))
    }

    fn terminal_action_button(
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
            .w(px(24.0))
            .h(px(22.0))
            .text_color(self.theme.foreground.opacity(0.90))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.handle_terminal_action(id, cx);
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_3()
                    .text_color(self.theme.foreground.opacity(0.90)),
            )
    }

    fn terminal_label(&self, terminal_id: &str) -> String {
        let index = self
            .snapshot
            .terminal_state
            .terminal_ids
            .iter()
            .position(|id| id == terminal_id)
            .unwrap_or(0);
        format!("Terminal {}", index + 1)
    }

    fn next_terminal_id(&self) -> String {
        let mut index = self.snapshot.terminal_state.terminal_ids.len() + 1;
        loop {
            let candidate = format!("terminal-{index}");
            if !self
                .snapshot
                .terminal_state
                .terminal_ids
                .iter()
                .any(|id| id == &candidate)
            {
                return candidate;
            }
            index += 1;
        }
    }

    fn diff_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("thread-diff-panel")
            .flex()
            .flex_col()
            .h_full()
            .min_w(px(360.0))
            .w(px(520.0))
            .flex_shrink_0()
            .border_l_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .child(self.diff_panel_header(cx))
            .child(self.diff_panel_body(cx))
    }

    fn diff_panel_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let summaries = self.snapshot.ordered_turn_diff_summaries();
        let selected_turn_id = self.snapshot.diff_route.diff_turn_id.clone();
        let mut turn_strip = div()
            .flex()
            .items_center()
            .gap_1()
            .min_w_0()
            .flex_1()
            .overflow_hidden()
            .px_1();

        turn_strip = turn_strip.child(self.diff_turn_chip(
            "All turns",
            None,
            selected_turn_id.is_none(),
            cx,
        ));
        for summary in summaries {
            let label = format!(
                "Turn {}  {}",
                summary
                    .checkpoint_turn_count
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "?".to_string()),
                short_timestamp_label(&summary.completed_at),
            );
            turn_strip = turn_strip.child(self.diff_turn_chip(
                label,
                Some(summary.turn_id.clone()),
                Some(&summary.turn_id) == selected_turn_id.as_ref(),
                cx,
            ));
        }

        div().border_b_1().border_color(self.theme.border).child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .h(px(48.0))
                .px_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .min_w_0()
                        .flex_1()
                        .child(self.diff_header_scroll_button(
                            "diff-turn-scroll-left",
                            "icons/chevron-left.svg",
                            false,
                        ))
                        .child(turn_strip)
                        .child(self.diff_header_scroll_button(
                            "diff-turn-scroll-right",
                            "icons/chevron-right.svg",
                            false,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .flex_shrink_0()
                        .child(self.diff_toggle_button(
                            "diff-view-stacked",
                            "icons/rows-3.svg",
                            !self.diff_render_split,
                            cx,
                        ))
                        .child(self.diff_toggle_button(
                            "diff-view-split",
                            "icons/columns-2.svg",
                            self.diff_render_split,
                            cx,
                        ))
                        .child(self.diff_toggle_button(
                            "diff-word-wrap",
                            "icons/text-wrap.svg",
                            self.diff_word_wrap,
                            cx,
                        ))
                        .child(self.diff_toggle_button(
                            "diff-ignore-whitespace",
                            "icons/pilcrow.svg",
                            self.diff_ignore_whitespace,
                            cx,
                        )),
                ),
        )
    }

    fn diff_header_scroll_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        enabled: bool,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(24.0))
            .h(px(24.0))
            .flex_shrink_0()
            .rounded(px(6.0))
            .border_1()
            .border_color(if enabled {
                self.theme.border.opacity(0.70)
            } else {
                self.theme.border.opacity(0.40)
            })
            .bg(self.theme.background.opacity(0.90))
            .text_color(if enabled {
                self.theme.muted_foreground
            } else {
                self.theme.muted_foreground.opacity(0.40)
            })
            .child(svg().path(icon_path).size_3().text_color(if enabled {
                self.theme.muted_foreground
            } else {
                self.theme.muted_foreground.opacity(0.40)
            }))
    }

    fn diff_turn_chip(
        &self,
        label: impl Into<SharedString>,
        turn_id: Option<String>,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let turn_id_for_click = turn_id.clone();
        div()
            .id(SharedString::from(match turn_id.as_deref() {
                Some(turn_id) => format!("diff-turn-chip-{turn_id}"),
                None => "diff-turn-chip-all".to_string(),
            }))
            .flex()
            .items_center()
            .h(px(28.0))
            .flex_shrink_0()
            .rounded(px(6.0))
            .border_1()
            .border_color(if selected {
                self.theme.border
            } else {
                self.theme.border.opacity(0.70)
            })
            .bg(if selected {
                self.theme.accent
            } else {
                self.theme.background.opacity(0.70)
            })
            .px_2()
            .text_size(px(10.0))
            .font_weight(FontWeight(500.0))
            .text_color(if selected {
                self.theme.foreground
            } else {
                self.theme.muted_foreground.opacity(0.85)
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.snapshot.diff_route = parse_diff_route_search(
                    Some(DiffOpenValue::from("1")),
                    turn_id_for_click.as_deref(),
                    None,
                );
                cx.notify();
            }))
            .child(label.into())
    }

    fn diff_toggle_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(if active {
                self.theme.border
            } else {
                self.theme.border.opacity(0.70)
            })
            .bg(if active {
                self.theme.accent
            } else {
                self.theme.background.opacity(0.70)
            })
            .text_color(if active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match id {
                    "diff-view-stacked" => this.diff_render_split = false,
                    "diff-view-split" => this.diff_render_split = true,
                    "diff-word-wrap" => this.diff_word_wrap = !this.diff_word_wrap,
                    "diff-ignore-whitespace" => {
                        this.diff_ignore_whitespace = !this.diff_ignore_whitespace
                    }
                    _ => {}
                }
                cx.notify();
            }))
            .child(svg().path(icon_path).size_3().text_color(if active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground
            }))
    }

    fn diff_panel_body(&self, cx: &mut Context<Self>) -> AnyElement {
        if !self.snapshot.renders_chat_view() {
            return self
                .diff_panel_center_state("Select a thread to inspect turn diffs.")
                .into_any_element();
        }
        if self.snapshot.turn_diff_summaries.is_empty() {
            return self
                .diff_panel_center_state("No completed turns yet.")
                .into_any_element();
        }

        let files = self.diff_panel_files();
        if files.is_empty() {
            return self
                .diff_panel_center_state("No patch available for this selection.")
                .into_any_element();
        }

        let total_stat = summarize_turn_diff_stats(&files);
        let tree = build_turn_diff_tree(&files);
        let mut patch_surface = div().flex().flex_col().gap_2().p_2();
        for (index, file) in files.iter().enumerate() {
            patch_surface = patch_surface.child(self.diff_file_card(file, index));
        }

        div()
            .id("diff-panel-viewport")
            .min_h_0()
            .min_w_0()
            .flex_1()
            .overflow_y_scroll()
            .p_2()
            .child(
                div()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(self.theme.border.opacity(0.60))
                    .bg(self.theme.card.opacity(0.25))
                    .mb_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .border_b_1()
                            .border_color(self.theme.border.opacity(0.50))
                            .px_3()
                            .py_2()
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight(650.0))
                                    .text_color(self.theme.muted_foreground.opacity(0.65))
                                    .child(
                                        format!("Changed files ({})", files.len()).to_uppercase(),
                                    ),
                            )
                            .child(self.diff_stat_label(total_stat, false)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .p_2()
                            .child(self.diff_tree_nodes(&tree, 0, cx)),
                    ),
            )
            .child(patch_surface)
            .into_any_element()
    }

    fn diff_panel_center_state(&self, label: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .px_5()
            .text_align(TextAlign::Center)
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground.opacity(0.70))
            .child(label)
    }

    fn diff_panel_files(&self) -> Vec<TurnDiffFileChange> {
        if let Some(summary) = self.snapshot.selected_turn_diff_summary() {
            return summary.files.clone();
        }

        self.snapshot
            .turn_diff_summaries
            .iter()
            .flat_map(|summary| summary.files.clone())
            .collect()
    }

    fn diff_tree_nodes(
        &self,
        nodes: &[TurnDiffTreeNode],
        depth: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut list = div().flex().flex_col().gap(px(2.0));
        for node in nodes {
            list = list.child(self.diff_tree_node(node, depth, cx));
        }
        list
    }

    fn diff_tree_node(
        &self,
        node: &TurnDiffTreeNode,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let left_padding = 8.0 + depth as f32 * 14.0;
        match node {
            TurnDiffTreeNode::Directory {
                name,
                path,
                stat,
                children,
            } => div()
                .id(SharedString::from(format!("diff-tree-dir-{path}")))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .rounded(px(6.0))
                        .py_1()
                        .pr_2()
                        .pl(px(left_padding))
                        .text_size(px(11.0))
                        .text_color(self.theme.muted_foreground.opacity(0.90))
                        .child(
                            svg()
                                .path("icons/chevron-down.svg")
                                .size_3()
                                .text_color(self.theme.muted_foreground.opacity(0.70)),
                        )
                        .child(
                            svg()
                                .path("icons/folder.svg")
                                .size_3()
                                .text_color(self.theme.muted_foreground.opacity(0.75)),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .flex_1()
                                .font_family(SharedString::from(MONO_FONT_FAMILY))
                                .child(name.clone()),
                        )
                        .when(stat.additions > 0 || stat.deletions > 0, |row| {
                            row.child(self.diff_stat_label(*stat, false))
                        }),
                )
                .child(self.diff_tree_nodes(children, depth + 1, cx))
                .into_any_element(),
            TurnDiffTreeNode::File { name, path, stat } => {
                let turn_id = self
                    .snapshot
                    .selected_turn_diff_summary()
                    .map(|summary| summary.turn_id.clone());
                let path_for_click = path.clone();
                let selected = self.snapshot.selected_diff_file_path() == Some(path.as_str());
                div()
                    .id(SharedString::from(format!("diff-tree-file-{path}")))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .rounded(px(6.0))
                    .py_1()
                    .pr_2()
                    .pl(px(left_padding))
                    .bg(if selected {
                        self.theme.accent.opacity(0.70)
                    } else {
                        self.theme.background.alpha(0.0)
                    })
                    .text_size(px(11.0))
                    .text_color(if selected {
                        self.theme.foreground.opacity(0.90)
                    } else {
                        self.theme.muted_foreground.opacity(0.80)
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.snapshot.diff_route = parse_diff_route_search(
                            Some(DiffOpenValue::from("1")),
                            turn_id.as_deref(),
                            Some(&path_for_click),
                        );
                        cx.notify();
                    }))
                    .child(div().w(px(14.0)).h(px(14.0)).flex_shrink_0())
                    .child(
                        svg()
                            .path("icons/file-json.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground.opacity(0.70)),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .font_family(SharedString::from(MONO_FONT_FAMILY))
                            .child(name.clone()),
                    )
                    .when_some(*stat, |row, stat| {
                        row.child(self.diff_stat_label(stat, false))
                    })
                    .into_any_element()
            }
        }
    }

    fn diff_file_card(&self, file: &TurnDiffFileChange, index: usize) -> impl IntoElement {
        let stat = TurnDiffStat {
            additions: file.additions.unwrap_or(0),
            deletions: file.deletions.unwrap_or(0),
        };
        let selected = self.snapshot.selected_diff_file_path() == Some(file.path.as_str());
        div()
            .id(SharedString::from(format!("diff-render-file-{index}")))
            .rounded(px(6.0))
            .border_1()
            .border_color(if selected {
                self.theme.border
            } else {
                self.theme.border.opacity(0.70)
            })
            .bg(self.theme.card.opacity(0.90))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .border_b_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.card.opacity(0.94))
                    .px_3()
                    .py_2()
                    .text_size(px(11.0))
                    .child(
                        svg()
                            .path("icons/chevron-down.svg")
                            .size_4()
                            .text_color(self.diff_kind_color(file.kind.as_deref())),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .font_family(SharedString::from(MONO_FONT_FAMILY))
                            .text_color(self.theme.foreground.opacity(0.90))
                            .child(file.path.clone()),
                    )
                    .child(self.diff_file_kind_badge(file.kind.as_deref()))
                    .child(self.diff_stat_label(stat, false)),
            )
            .child(self.diff_line_row(" ", "context", "checkpoint summary", self.theme.background))
            .when(file.deletions.unwrap_or(0) > 0, |card| {
                card.child(self.diff_line_row(
                    "-",
                    "old",
                    "removed lines represented by this turn diff",
                    diff_destructive_color().opacity(0.10),
                ))
            })
            .when(file.additions.unwrap_or(0) > 0, |card| {
                card.child(self.diff_line_row(
                    "+",
                    "new",
                    "added lines represented by this turn diff",
                    diff_success_color().opacity(0.10),
                ))
            })
    }

    fn diff_file_kind_badge(&self, kind: Option<&str>) -> impl IntoElement {
        let label = match kind {
            Some("added") | Some("new") => "added",
            Some("deleted") => "deleted",
            Some("renamed") => "renamed",
            _ => "modified",
        };
        div()
            .rounded(px(999.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.70))
            .px_1p5()
            .py_0p5()
            .text_size(px(10.0))
            .text_color(self.theme.muted_foreground.opacity(0.80))
            .child(label)
    }

    fn diff_line_row(
        &self,
        marker: &'static str,
        gutter: &'static str,
        text: &'static str,
        bg: gpui::Hsla,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .min_h(px(24.0))
            .bg(bg)
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(11.0))
            .child(
                div()
                    .w(px(34.0))
                    .flex_shrink_0()
                    .text_align(TextAlign::Center)
                    .text_color(self.theme.muted_foreground.opacity(0.45))
                    .child(gutter),
            )
            .child(
                div()
                    .w(px(18.0))
                    .flex_shrink_0()
                    .text_color(match marker {
                        "+" => diff_success_color(),
                        "-" => diff_destructive_color(),
                        _ => self.theme.muted_foreground.opacity(0.45),
                    })
                    .child(marker),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .text_color(self.theme.foreground.opacity(0.76))
                    .child(text),
            )
    }

    fn diff_kind_color(&self, kind: Option<&str>) -> gpui::Hsla {
        match kind {
            Some("added") | Some("new") => diff_success_color(),
            Some("deleted") => diff_destructive_color(),
            _ => self.theme.muted_foreground.opacity(0.80),
        }
    }

    fn diff_stat_label(&self, stat: TurnDiffStat, show_parentheses: bool) -> impl IntoElement {
        let mut label = div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(10.0));

        if show_parentheses {
            label = label.child(
                div()
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child("("),
            );
        }

        label = label
            .child(
                div()
                    .text_color(diff_success_color())
                    .child(format!("+{}", stat.additions)),
            )
            .child(
                div()
                    .mx_0p5()
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child("/"),
            )
            .child(
                div()
                    .text_color(diff_destructive_color())
                    .child(format!("-{}", stat.deletions)),
            );

        if show_parentheses {
            label = label.child(
                div()
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child(")"),
            );
        }

        label
    }

    fn composer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active_pending_approval = self.snapshot.active_pending_approval();
        let active_pending_user_input_progress = self.snapshot.active_pending_user_input_progress();
        let composer_width = if self.snapshot.diff_open() {
            440.0
        } else {
            832.0
        };
        let mut surface = div()
            .rounded(px(20.0))
            .border_1()
            .border_color(if self.composer_prompt_focused {
                self.theme.primary.opacity(0.45)
            } else {
                self.theme.border
            })
            .bg(self.theme.card);

        if let Some(approval) = active_pending_approval {
            surface = surface.child(self.composer_pending_approval_panel(approval));
        } else if let Some(progress) = active_pending_user_input_progress.as_ref() {
            surface = surface.child(self.composer_pending_user_input_panel(progress, cx));
        }

        surface = surface.child(self.composer_prompt_editor(cx));

        surface = if let Some(approval) = active_pending_approval {
            surface.child(self.composer_pending_approval_actions(approval, cx))
        } else {
            surface.child(self.composer_footer(cx))
        };

        div()
            .flex()
            .items_center()
            .justify_center()
            .px_8()
            .pb_1()
            .child(
                div()
                    .id("chat-composer-form")
                    .w(px(composer_width))
                    .rounded(px(22.0))
                    .bg(self.theme.primary.opacity(0.10))
                    .p(px(1.0))
                    .child(surface),
            )
    }

    fn composer_pending_approval_panel(&self, approval: &PendingApproval) -> impl IntoElement {
        let pending_count = self.snapshot.pending_approvals.len();
        div()
            .rounded_t(px(19.0))
            .border_b_1()
            .border_color(self.theme.border.opacity(0.65))
            .bg(self.theme.accent.opacity(0.20))
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
                            .text_color(self.theme.foreground)
                            .child("PENDING APPROVAL"),
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight(500.0))
                            .child(approval.request_kind.summary()),
                    )
                    .when(pending_count > 1, |panel| {
                        panel.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(self.theme.muted_foreground)
                                .child(format!("1/{pending_count}")),
                        )
                    }),
            )
    }

    fn composer_pending_user_input_panel(
        &self,
        progress: &PendingUserInputProgress,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(active_question) = progress.active_question.as_ref() else {
            return div().into_any_element();
        };
        let question_count = self
            .snapshot
            .active_pending_user_input()
            .map(|prompt| prompt.questions.len())
            .unwrap_or(0);

        let mut options = div().mt_3().flex().flex_col().gap_1();
        for (index, option) in active_question.options.iter().enumerate() {
            let is_selected = progress
                .selected_option_labels
                .iter()
                .any(|label| label == &option.label);
            let question_id = active_question.id.clone();
            let option_label = option.label.clone();
            let shortcut_key = (index < 9).then_some(index + 1);
            let description =
                if !option.description.is_empty() && option.description != option.label {
                    Some(option.description.clone())
                } else {
                    None
                };
            options = options.child(
                div()
                    .id(SharedString::from(format!(
                        "chat-composer-pending-user-input-option-{index}"
                    )))
                    .flex()
                    .items_center()
                    .gap_3()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(if is_selected {
                        pending_blue().opacity(0.40)
                    } else {
                        self.theme.background.alpha(0.0)
                    })
                    .bg(if is_selected {
                        pending_blue().opacity(0.08)
                    } else {
                        self.theme.accent.opacity(0.20)
                    })
                    .px_3()
                    .py_2()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_pending_user_input_option(&question_id, &option_label, cx);
                    }))
                    .when_some(shortcut_key, |row, key| {
                        row.child(self.pending_input_shortcut_key(key, is_selected))
                    })
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight(500.0))
                                    .child(option.label.clone()),
                            )
                            .when_some(description, |row, description| {
                                row.child(
                                    div()
                                        .ml_2()
                                        .text_size(px(12.0))
                                        .text_color(self.theme.muted_foreground.opacity(0.50))
                                        .child(description),
                                )
                            }),
                    )
                    .when(is_selected, |row| {
                        row.child(
                            svg()
                                .path("icons/check.svg")
                                .w(px(14.0))
                                .h(px(14.0))
                                .flex_shrink_0()
                                .text_color(pending_blue_icon()),
                        )
                    }),
            );
        }

        div()
            .rounded_t(px(19.0))
            .border_b_1()
            .border_color(self.theme.border.opacity(0.65))
            .bg(self.theme.accent.opacity(0.20))
            .child(
                div()
                    .px_4()
                    .py_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(if question_count > 1 {
                                div()
                                    .flex()
                                    .items_center()
                                    .rounded(px(6.0))
                                    .bg(self.theme.accent.opacity(0.60))
                                    .px_1p5()
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight(500.0))
                                    .text_color(self.theme.muted_foreground.opacity(0.60))
                                    .child(format!(
                                        "{}/{}",
                                        progress.question_index + 1,
                                        question_count
                                    ))
                            } else {
                                div()
                            })
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight(650.0))
                                    .text_color(self.theme.muted_foreground.opacity(0.50))
                                    .child(active_question.header.to_ascii_uppercase()),
                            ),
                    )
                    .child(
                        div()
                            .mt_1p5()
                            .text_size(px(14.0))
                            .text_color(self.theme.foreground.opacity(0.90))
                            .child(active_question.question.clone()),
                    )
                    .when(active_question.multi_select, |panel| {
                        panel.child(
                            div()
                                .mt_1()
                                .text_size(px(12.0))
                                .text_color(self.theme.muted_foreground.opacity(0.65))
                                .child("Select one or more options."),
                        )
                    })
                    .child(options),
            )
            .into_any_element()
    }

    fn pending_input_shortcut_key(&self, key: usize, selected: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .flex_shrink_0()
            .rounded(px(4.0))
            .text_size(px(11.0))
            .font_weight(FontWeight(500.0))
            .text_color(if selected {
                pending_blue_icon()
            } else {
                self.theme.muted_foreground.opacity(0.50)
            })
            .bg(if selected {
                pending_blue().opacity(0.20)
            } else {
                self.theme.accent.opacity(0.40)
            })
            .child(key.to_string())
    }

    fn composer_prompt_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active_pending_approval = self.snapshot.active_pending_approval();
        let active_pending_user_input_progress = self.snapshot.active_pending_user_input_progress();
        let text = if let Some(approval) = active_pending_approval {
            approval
                .detail
                .clone()
                .unwrap_or_else(|| "Resolve this approval request to continue".to_string())
        } else if let Some(progress) = active_pending_user_input_progress.as_ref() {
            if progress.custom_answer.trim().is_empty() {
                "Type your own answer, or leave this blank to use the selected option".to_string()
            } else {
                progress.custom_answer.clone()
            }
        } else if self.composer_prompt.is_empty() {
            if self.composer_submitted_count > 0 {
                "Message queued. Type another prompt.".to_string()
            } else {
                "Ask for follow-up changes or attach images".to_string()
            }
        } else {
            self.composer_prompt.clone()
        };
        let placeholder = self.composer_prompt.is_empty()
            && active_pending_approval.is_none()
            && active_pending_user_input_progress
                .as_ref()
                .map(|progress| progress.custom_answer.trim().is_empty())
                .unwrap_or(true);

        div()
            .id("chat-composer-input")
            .relative()
            .track_focus(&self.composer_focus_handle)
            .key_context("ChatComposer")
            .on_key_down(cx.listener(Self::on_composer_key_down))
            .tab_index(0)
            .cursor(CursorStyle::IBeam)
            .min_h(px(96.0))
            .px_4()
            .pt_4()
            .pb_3()
            .on_click(cx.listener(|this, _, window, cx| {
                if this.snapshot.active_pending_approval().is_none() {
                    this.composer_prompt_focused = true;
                    window.focus(&this.composer_focus_handle);
                }
                cx.notify();
            }))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(if active_pending_approval.is_some() {
                        self.theme.muted_foreground.opacity(0.72)
                    } else if placeholder {
                        self.theme
                            .muted_foreground
                            .opacity(if self.composer_prompt_focused {
                                0.72
                            } else {
                                0.50
                            })
                    } else {
                        self.theme.foreground
                    })
                    .child(text),
            )
    }

    fn composer_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        if let Some(progress) = self.snapshot.active_pending_user_input_progress() {
            return self
                .composer_pending_user_input_footer(&progress, cx)
                .into_any_element();
        }

        let compact = self.snapshot.diff_open();
        let effort_label = if compact {
            "Medium"
        } else {
            "Medium · Normal"
        };
        let runtime_label = if compact {
            "Full"
        } else {
            self.runtime_mode().label
        };

        div()
            .id("chat-composer-footer")
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .pb_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w_0()
                    .child(self.composer_model_picker(cx))
                    .child(self.composer_footer_separator())
                    .child(self.composer_footer_text_control(
                        "chat-composer-effort-mode",
                        effort_label,
                        cx,
                    ))
                    .child(self.composer_footer_separator())
                    .child(self.composer_footer_control(
                        "chat-composer-plan-toggle",
                        "icons/bot.svg",
                        if self.composer_plan_mode {
                            "Plan"
                        } else {
                            "Build"
                        },
                        cx,
                    ))
                    .child(self.composer_footer_separator())
                    .child(self.composer_footer_control(
                        "chat-composer-runtime-mode",
                        self.runtime_mode().icon,
                        runtime_label,
                        cx,
                    )),
            )
            .child(self.composer_send_button(cx))
            .into_any_element()
    }

    fn composer_pending_user_input_footer(
        &self,
        progress: &PendingUserInputProgress,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("chat-composer-pending-user-input-footer")
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .pb_3()
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .when(progress.question_index > 0, |actions| {
                        actions.child(self.composer_pending_user_input_previous_button(cx))
                    })
                    .child(self.composer_pending_user_input_submit_button(progress, cx)),
            )
    }

    fn composer_pending_user_input_previous_button(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("chat-composer-pending-user-input-previous")
            .rounded(px(999.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_4()
            .py_1p5()
            .text_size(px(13.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.previous_pending_user_input_question(cx);
            }))
            .child("Previous")
    }

    fn composer_pending_user_input_submit_button(
        &self,
        progress: &PendingUserInputProgress,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let disabled = if progress.is_last_question {
            !progress.is_complete
        } else {
            !progress.can_advance
        };
        let is_responding = self
            .snapshot
            .active_pending_user_input()
            .map(|prompt| self.snapshot.is_responding_to_request(&prompt.request_id))
            .unwrap_or(false);
        let label = if is_responding {
            "Submitting..."
        } else if !progress.is_last_question {
            "Next question"
        } else if progress.question_index > 0 {
            "Submit answers"
        } else {
            "Submit answer"
        };

        div()
            .id("chat-composer-pending-user-input-submit")
            .rounded(px(999.0))
            .bg(if disabled || is_responding {
                self.theme.primary.opacity(0.30)
            } else {
                self.theme.primary.opacity(0.92)
            })
            .px_4()
            .py_1p5()
            .text_size(px(13.0))
            .text_color(hsla(0.0, 0.0, 1.0, 1.0))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                if !disabled && !is_responding {
                    this.advance_pending_user_input(cx);
                }
            }))
            .child(label)
    }

    fn composer_pending_approval_actions(
        &self,
        approval: &PendingApproval,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_responding = self.snapshot.is_responding_to_request(&approval.request_id);

        div()
            .id("chat-composer-pending-approval-actions")
            .flex()
            .items_center()
            .justify_end()
            .gap_2()
            .px_3()
            .pb_3()
            .child(self.composer_pending_approval_action_button(
                "Cancel turn",
                PendingApprovalActionKind::Cancel,
                approval,
                is_responding,
                cx,
            ))
            .child(self.composer_pending_approval_action_button(
                "Decline",
                PendingApprovalActionKind::Decline,
                approval,
                is_responding,
                cx,
            ))
            .child(self.composer_pending_approval_action_button(
                "Always allow this session",
                PendingApprovalActionKind::AcceptForSession,
                approval,
                is_responding,
                cx,
            ))
            .child(self.composer_pending_approval_action_button(
                "Approve once",
                PendingApprovalActionKind::Accept,
                approval,
                is_responding,
                cx,
            ))
    }

    fn composer_pending_approval_action_button(
        &self,
        label: &'static str,
        action: PendingApprovalActionKind,
        approval: &PendingApproval,
        is_responding: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let request_id = approval.request_id.clone();
        let (border, background, foreground) = match action {
            PendingApprovalActionKind::Decline => (
                hsla(0.0, 0.72, 0.48, 0.32),
                hsla(0.0, 0.72, 0.48, 0.05),
                hsla(0.0, 0.72, 0.48, 1.0),
            ),
            PendingApprovalActionKind::Accept => (
                self.theme.primary.opacity(0.92),
                self.theme.primary.opacity(0.92),
                hsla(0.0, 0.0, 1.0, 1.0),
            ),
            PendingApprovalActionKind::Cancel | PendingApprovalActionKind::AcceptForSession => (
                self.theme.border,
                self.theme.background,
                self.theme.foreground,
            ),
        };

        div()
            .id(action.id())
            .rounded(px(7.0))
            .border_1()
            .border_color(if is_responding {
                border.opacity(0.50)
            } else {
                border
            })
            .bg(if is_responding {
                background.opacity(0.50)
            } else {
                background
            })
            .px_3()
            .py_1p5()
            .text_size(px(13.0))
            .text_color(if is_responding {
                foreground.opacity(0.50)
            } else {
                foreground
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                if !is_responding {
                    this.respond_to_pending_approval(&request_id, cx);
                }
            }))
            .child(label)
    }

    fn composer_model_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.composer_model();

        div()
            .id("chat-composer-model-picker")
            .flex()
            .items_center()
            .gap_2()
            .h(px(32.0))
            .rounded(px(8.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.composer_model_index = (this.composer_model_index + 1) % COMPOSER_MODELS.len();
                cx.notify();
            }))
            .child(
                svg()
                    .path(model.icon)
                    .size_4()
                    .text_color(self.theme.foreground),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w_0()
                    .child(if model.model.is_empty() {
                        div()
                            .text_color(self.theme.foreground)
                            .font_weight(FontWeight(550.0))
                            .child(model.provider)
                    } else {
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_color(self.theme.foreground)
                                    .font_weight(FontWeight(550.0))
                                    .child(model.provider),
                            )
                            .child(div().text_color(self.theme.muted_foreground).child("/"))
                            .child(div().child(model.model))
                    }),
            )
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground.opacity(0.72)),
            )
    }

    fn composer_footer_separator(&self) -> impl IntoElement {
        div().w(px(1.0)).h(px(16.0)).bg(self.theme.border)
    }

    fn composer_footer_text_control(
        &self,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(32.0))
            .rounded(px(8.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(|_, _, _, cx| {
                cx.notify();
            }))
            .child(label)
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground.opacity(0.72)),
            )
    }

    fn composer_footer_control(
        &self,
        id: &'static str,
        icon: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(32.0))
            .rounded(px(8.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match id {
                    "chat-composer-runtime-mode" => {
                        this.composer_runtime_index =
                            (this.composer_runtime_index + 1) % COMPOSER_RUNTIME_MODES.len();
                    }
                    "chat-composer-plan-toggle" => {
                        this.composer_plan_mode = !this.composer_plan_mode;
                    }
                    _ => {}
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path(icon)
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
            .child(label)
    }

    fn composer_send_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_prompt = !self.composer_prompt.trim().is_empty();

        div()
            .id("chat-composer-send")
            .flex()
            .items_center()
            .justify_center()
            .w(px(32.0))
            .h(px(32.0))
            .rounded(px(16.0))
            .bg(if has_prompt {
                self.theme.primary.opacity(0.92)
            } else {
                self.theme.primary.opacity(0.30)
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| {
                if this.composer_prompt.trim().is_empty() {
                    this.composer_prompt_focused = true;
                    window.focus(&this.composer_focus_handle);
                } else {
                    this.submit_composer(cx);
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/arrow-up.svg")
                    .size_4()
                    .text_color(hsla(0.0, 0.0, 1.0, 1.0)),
            )
    }

    fn composer_model(&self) -> ComposerModel {
        COMPOSER_MODELS[self.composer_model_index % COMPOSER_MODELS.len()]
    }

    fn runtime_mode(&self) -> ComposerRuntimeMode {
        COMPOSER_RUNTIME_MODES[self.composer_runtime_index % COMPOSER_RUNTIME_MODES.len()]
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
        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(41.0))
            .px_5()
            .border_b_1()
            .border_color(self.theme.border)
            .child(div().text_size(px(14.0)).child("Settings"));

        if self.settings_section == SettingsSection::General {
            header = header.child(
                div()
                    .id("settings-restore-defaults")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .rounded(px(7.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .px_2()
                    .py_1()
                    .text_size(px(13.0))
                    .text_color(self.theme.muted_foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings_defaults_restored = true;
                        this.settings_select_open = None;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/rotate-ccw.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(self.theme.muted_foreground),
                    )
                    .child(if self.settings_defaults_restored {
                        "Defaults restored"
                    } else {
                        "Restore defaults"
                    }),
            );
        }

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(header)
            .child(match self.settings_section {
                SettingsSection::General => self.settings_general_panel(cx).into_any_element(),
                SettingsSection::Keybindings => {
                    self.settings_keybindings_panel(cx).into_any_element()
                }
                SettingsSection::Providers => self.settings_providers_panel(cx).into_any_element(),
                SettingsSection::SourceControl => {
                    self.settings_source_control_panel(cx).into_any_element()
                }
                SettingsSection::Connections => {
                    self.settings_connections_panel(cx).into_any_element()
                }
                SettingsSection::Archive => self.settings_archive_panel().into_any_element(),
            })
    }

    fn settings_general_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-general-scroll")
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
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("GENERAL"))
                            .child(self.settings_card(cx)),
                    )
                    .child(self.settings_about_section(cx)),
            )
    }

    fn settings_keybindings_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child(self.keybindings_header_actions(rows.len(), cx)),
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
                    )
                    .child(if self.keybindings_add_dialog_open {
                        self.keybinding_add_panel(cx).into_any_element()
                    } else {
                        div().into_any_element()
                    }),
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
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_color(if value.is_empty() {
                self.theme.muted_foreground
            } else {
                self.theme.foreground
            })
            .child(if value.is_empty() { "Always" } else { value })
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn keybinding_warning_mark(&self) -> impl IntoElement {
        svg()
            .path("icons/triangle-alert.svg")
            .w(px(14.0))
            .h(px(14.0))
            .text_color(hsla(36.0 / 360.0, 1.0, 0.50, 1.0))
    }

    fn keybindings_header_actions(&self, count: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let mut actions = div()
            .flex()
            .items_center()
            .gap_1p5()
            .text_size(px(11.0))
            .text_color(self.theme.muted_foreground);

        if self.keybindings_search_open {
            actions = actions.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .h(px(24.0))
                    .w(px(176.0))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.background)
                    .px_2()
                    .child(
                        svg()
                            .path("icons/search.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(self.theme.muted_foreground.opacity(0.72))
                            .child("Search keybindings"),
                    ),
            );
        } else {
            actions = actions.child(format!("{count} bindings"));
        }

        if self.keybindings_file_opened {
            actions = actions.child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground.opacity(0.72))
                    .child("keybindings.json"),
            );
        }

        actions
            .child(self.keybindings_header_icon_button(
                "keybindings-search",
                "icons/search.svg",
                KeybindingsHeaderAction::ToggleSearch,
                cx,
            ))
            .child(self.keybindings_header_icon_button(
                "keybindings-add",
                "icons/plus.svg",
                KeybindingsHeaderAction::ToggleAdd,
                cx,
            ))
            .child(self.keybindings_header_icon_button(
                "keybindings-open-json",
                "icons/file-json.svg",
                KeybindingsHeaderAction::MarkFileOpened,
                cx,
            ))
    }

    fn keybindings_header_icon_button(
        &self,
        id: &'static str,
        icon: &'static str,
        action: KeybindingsHeaderAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match action {
                    KeybindingsHeaderAction::ToggleSearch => {
                        this.keybindings_search_open = !this.keybindings_search_open;
                    }
                    KeybindingsHeaderAction::ToggleAdd => {
                        this.keybindings_add_dialog_open = !this.keybindings_add_dialog_open;
                    }
                    KeybindingsHeaderAction::MarkFileOpened => {
                        this.keybindings_file_opened = true;
                    }
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path(icon)
                    .size_3()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn keybinding_add_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .rounded(px(12.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .p_4()
            .child(
                div()
                    .mb_3()
                    .text_size(px(13.0))
                    .font_weight(FontWeight(650.0))
                    .child("Keybinding draft"),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_3()
                    .child(self.keybinding_draft_cell("Command", "Command Palette: Toggle"))
                    .child(self.keybinding_draft_cell("Shortcut", "mod+k"))
                    .child(self.keybinding_draft_cell("When", "!terminalFocus")),
            )
            .child(
                div()
                    .id("keybindings-save-draft")
                    .mt_3()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .h(px(32.0))
                    .rounded(px(7.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.background)
                    .text_size(px(12.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.keybindings_add_dialog_open = false;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/plus.svg")
                            .size_4()
                            .text_color(self.theme.foreground),
                    )
                    .child("Save draft"),
            )
    }

    fn keybinding_draft_cell(&self, label: &'static str, value: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight(600.0))
                    .text_color(self.theme.muted_foreground)
                    .child(label),
            )
            .child(
                div()
                    .h(px(32.0))
                    .rounded(px(7.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.background)
                    .px_3()
                    .py_2()
                    .text_size(px(12.0))
                    .child(value),
            )
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
                    .child(self.source_control_version_control_section(cx))
                    .child(self.source_control_provider_section(cx)),
            )
            .into_any_element()
    }

    fn source_control_version_control_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut card = self
            .source_control_card()
            .child(self.source_control_git_row(cx));

        if self.source_control_git_details_open {
            card = card.child(self.source_control_git_fetch_settings(cx));
        }

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
                    .child(self.settings_section_header("VERSION CONTROL"))
                    .child(self.source_control_scan_icon_button(cx)),
            )
            .child(card)
    }

    fn source_control_provider_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2p5()
            .child(self.settings_section_header("SOURCE CONTROL PROVIDERS"))
            .child(
                self.source_control_card()
                    .child(self.source_control_provider_row(
                        0,
                        "GitHub",
                        "Authenticated",
                        "jasperdevs",
                        "icons/git-pull-request.svg",
                        true,
                        cx,
                    ))
                    .child(self.source_control_provider_row(
                        1,
                        "GitLab",
                        "Not authenticated",
                        "glab auth login",
                        "icons/git-branch.svg",
                        false,
                        cx,
                    ))
                    .child(self.source_control_provider_row(
                        2,
                        "Bitbucket",
                        "Not available on this server",
                        "Install the Bitbucket CLI on the server host.",
                        "icons/link-2.svg",
                        false,
                        cx,
                    )),
            )
    }

    fn source_control_card(&self) -> gpui::Div {
        div()
            .relative()
            .overflow_hidden()
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
    }

    fn source_control_git_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = if self.source_control_scan_requested {
            "Available. Server environment checked just now."
        } else {
            "Available. Refresh remote branch status in the background."
        };

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
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.source_control_item_mark(
                                "icons/git-branch.svg",
                                self.theme.primary,
                            ))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight(650.0))
                                    .child("Git"),
                            )
                            .child(self.source_control_code_badge("2.x")),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground)
                            .child(summary),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.source_control_details_button(cx))
                    .child(self.source_control_switch(
                        "source-control-git-switch",
                        SourceControlSwitch::Git,
                        self.source_control_git_enabled,
                        cx,
                    )),
            )
    }

    fn source_control_provider_row(
        &self,
        index: usize,
        label: &'static str,
        status: &'static str,
        detail: &'static str,
        icon: &'static str,
        authenticated: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let enabled = self.source_control_provider_enabled[index];
        let indicator = if authenticated {
            self.theme.primary
        } else {
            hsla(36.0 / 360.0, 1.0, 0.48, 1.0)
        };
        let summary = if authenticated {
            if self.source_control_account_revealed {
                format!("{status} as {detail}")
            } else {
                format!("{status} as hidden account")
            }
        } else {
            detail.to_string()
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(76.0))
            .border_t_1()
            .border_color(self.theme.border)
            .px_5()
            .py_3p5()
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
                            .child(self.source_control_item_mark(icon, indicator))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight(650.0))
                                    .child(label),
                            )
                            .child(if authenticated {
                                div().into_any_element()
                            } else {
                                self.source_control_warning_badge(status).into_any_element()
                            }),
                    )
                    .child(
                        div()
                            .id(match index {
                                0 => "source-control-github-account",
                                1 => "source-control-gitlab-summary",
                                _ => "source-control-bitbucket-summary",
                            })
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground)
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if index == 0 {
                                    this.source_control_account_revealed =
                                        !this.source_control_account_revealed;
                                    cx.notify();
                                }
                            }))
                            .child(summary),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.source_control_switch(
                        match index {
                            0 => "source-control-github-switch",
                            1 => "source-control-gitlab-switch",
                            _ => "source-control-bitbucket-switch",
                        },
                        SourceControlSwitch::Provider(index),
                        enabled,
                        cx,
                    )),
            )
    }

    fn source_control_git_fetch_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .border_t_1()
            .border_color(self.theme.border)
            .px_5()
            .py_3p5()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight(600.0))
                                    .child("Fetch interval")
                                    .child(self.source_control_fetch_button(
                                        "source-control-fetch-reset",
                                        "icons/refresh-cw.svg",
                                        SourceControlFetchAction::Reset,
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child(
                                        "Set this to 0 seconds if credentials should only be prompted by explicit Git actions.",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.source_control_fetch_button(
                                "source-control-fetch-decrease",
                                "icons/minus.svg",
                                SourceControlFetchAction::Decrease,
                                cx,
                            ))
                            .child(
                                div()
                                    .min_w(px(44.0))
                                    .rounded(px(7.0))
                                    .border_1()
                                    .border_color(self.theme.border)
                                    .bg(self.theme.background)
                                    .px_2()
                                    .py_1p5()
                                    .text_align(TextAlign::Center)
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight(600.0))
                                    .child(self.source_control_fetch_interval_seconds.to_string()),
                            )
                            .child(self.source_control_fetch_button(
                                "source-control-fetch-increase",
                                "icons/plus.svg",
                                SourceControlFetchAction::Increase,
                                cx,
                            ))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child("seconds"),
                            ),
                    ),
            )
    }

    fn source_control_item_mark(
        &self,
        icon: &'static str,
        indicator: gpui::Hsla,
    ) -> impl IntoElement {
        div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .child(svg().path(icon).size_4().text_color(self.theme.foreground))
            .child(
                div()
                    .absolute()
                    .left(px(-2.0))
                    .top(px(-2.0))
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded(px(4.0))
                    .bg(indicator)
                    .border_1()
                    .border_color(self.theme.background),
            )
    }

    fn source_control_code_badge(&self, label: &'static str) -> impl IntoElement {
        div()
            .rounded(px(5.0))
            .bg(self.theme.accent)
            .px_1p5()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn source_control_warning_badge(&self, label: &'static str) -> impl IntoElement {
        div()
            .rounded(px(5.0))
            .border_1()
            .border_color(hsla(36.0 / 360.0, 1.0, 0.48, 0.28))
            .bg(hsla(36.0 / 360.0, 1.0, 0.48, 0.08))
            .px_1p5()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(hsla(36.0 / 360.0, 1.0, 0.42, 1.0))
            .child(label)
    }

    fn source_control_details_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("source-control-git-details")
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded(px(7.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.source_control_git_details_open = !this.source_control_git_details_open;
                cx.notify();
            }))
            .child(
                svg()
                    .path(if self.source_control_git_details_open {
                        "icons/chevron-down.svg"
                    } else {
                        "icons/chevron-right.svg"
                    })
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn source_control_switch(
        &self,
        id: &'static str,
        target: SourceControlSwitch,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .relative()
            .w(px(34.0))
            .h(px(20.0))
            .rounded(px(10.0))
            .cursor_pointer()
            .bg(if enabled {
                self.theme.primary
            } else {
                self.theme.accent
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                match target {
                    SourceControlSwitch::Git => {
                        this.source_control_git_enabled = !this.source_control_git_enabled;
                    }
                    SourceControlSwitch::Provider(index) => {
                        if let Some(value) = this.source_control_provider_enabled.get_mut(index) {
                            *value = !*value;
                        }
                    }
                }
                cx.notify();
            }))
            .child(
                div()
                    .absolute()
                    .top(px(2.0))
                    .left(if enabled { px(16.0) } else { px(2.0) })
                    .w(px(16.0))
                    .h(px(16.0))
                    .rounded(px(8.0))
                    .bg(self.theme.background),
            )
    }

    fn source_control_fetch_button(
        &self,
        id: &'static str,
        icon: &'static str,
        action: SourceControlFetchAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(24.0))
            .h(px(24.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match action {
                    SourceControlFetchAction::Decrease => {
                        this.source_control_fetch_interval_seconds = this
                            .source_control_fetch_interval_seconds
                            .saturating_sub(SOURCE_CONTROL_FETCH_INTERVAL_STEP_SECONDS);
                    }
                    SourceControlFetchAction::Increase => {
                        this.source_control_fetch_interval_seconds = this
                            .source_control_fetch_interval_seconds
                            .saturating_add(SOURCE_CONTROL_FETCH_INTERVAL_STEP_SECONDS);
                    }
                    SourceControlFetchAction::Reset => {
                        this.source_control_fetch_interval_seconds =
                            SOURCE_CONTROL_DEFAULT_FETCH_INTERVAL_SECONDS;
                    }
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path(icon)
                    .size_3()
                    .text_color(self.theme.muted_foreground),
            )
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

    fn settings_connections_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = div()
            .id("settings-connections-scroll")
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
                    .child(self.connections_local_backend_section(cx))
                    .child(self.connections_remote_environments_section(cx)),
            );

        if self.connections_add_dialog_open {
            content = content.child(self.connections_add_panel(cx));
        }

        content
    }

    fn connections_local_backend_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let network_description = if self.connections_network_accessible {
            "Exposed on all interfaces. Pairing links use the advertised endpoint."
        } else {
            "Only this machine can reach the local backend."
        };
        let tailscale_description = if self.connections_refresh_requested {
            "Checked just now. No MagicDNS endpoint was detected."
        } else {
            "No MagicDNS endpoint detected. Refresh after enabling Tailscale Serve."
        };

        div()
            .flex()
            .flex_col()
            .gap_2p5()
            .child(self.settings_section_header("MANAGE LOCAL BACKEND"))
            .child(
                self.connections_card()
                    .child(self.connection_settings_row(
                        "Network access",
                        network_description,
                        self.connections_network_switch(cx),
                        true,
                    ))
                    .child(if self.connections_network_accessible {
                        self.connections_endpoint_row(cx).into_any_element()
                    } else {
                        self.connection_settings_row(
                            "Advertised endpoints",
                            "Enable network access to publish local pairing endpoints.",
                            self.connections_muted_badge("Local only"),
                            false,
                        )
                        .into_any_element()
                    })
                    .child(self.connection_settings_row(
                        "Tailscale HTTPS",
                        tailscale_description,
                        self.connections_refresh_button(cx),
                        false,
                    )),
            )
    }

    fn connections_remote_environments_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let body = if self.connections_saved_environment {
            self.connections_saved_environment_row(cx)
                .into_any_element()
        } else {
            self.connections_empty_environment_row().into_any_element()
        };

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
                    .child(self.settings_section_header("REMOTE ENVIRONMENTS"))
                    .child(self.connections_add_button(cx)),
            )
            .child(self.connections_card().child(body))
    }

    fn connections_card(&self) -> gpui::Div {
        div()
            .relative()
            .overflow_hidden()
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
    }

    fn connection_settings_row(
        &self,
        title: &'static str,
        description: &'static str,
        control: impl IntoElement,
        first: bool,
    ) -> impl IntoElement {
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
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight(650.0))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground.opacity(0.82))
                            .child(description),
                    ),
            )
            .child(control)
    }

    fn connections_network_switch(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let enabled = self.connections_network_accessible;
        div()
            .id("connections-network-access-toggle")
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
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_network_accessible = !this.connections_network_accessible;
                this.connections_endpoint_copied = false;
                cx.notify();
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

    fn connections_endpoint_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let description = if self.connections_endpoint_copied {
            "Pairing URL copied."
        } else {
            "Default local network endpoint for pairing links."
        };

        self.connection_settings_row(
            "http://127.0.0.1:8765",
            description,
            div()
                .id("connections-copy-endpoint")
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.0))
                .h(px(28.0))
                .rounded(px(7.0))
                .border_1()
                .border_color(self.theme.border)
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.connections_endpoint_copied = true;
                    cx.notify();
                }))
                .child(
                    svg()
                        .path("icons/copy.svg")
                        .size_4()
                        .text_color(self.theme.muted_foreground),
                ),
            false,
        )
    }

    fn connections_refresh_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("connections-refresh-tailscale")
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_refresh_requested = true;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/refresh-cw.svg")
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn connections_muted_badge(&self, label: &'static str) -> impl IntoElement {
        div()
            .rounded(px(9.0))
            .border_1()
            .border_color(self.theme.border)
            .px_2()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn connections_add_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("connections-add-environment")
            .flex()
            .items_center()
            .justify_center()
            .w(px(20.0))
            .h(px(20.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_add_dialog_open = !this.connections_add_dialog_open;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/plus-square.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn connections_empty_environment_row(&self) -> impl IntoElement {
        div()
            .min_h(px(62.0))
            .px_5()
            .py_3p5()
            .text_size(px(12.0))
            .text_color(self.theme.muted_foreground)
            .child("No remote environments yet. Use the plus control to pair another environment.")
    }

    fn connections_saved_environment_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let connected = self.connections_saved_environment_connected;
        div()
            .id("connections-saved-environment-row")
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
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(self.connection_status_dot(connected))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight(650.0))
                                    .child(if connected {
                                        "Remote environment"
                                    } else {
                                        "Saved environment"
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground.opacity(0.82))
                            .child(if connected {
                                "Client · Last connected just now"
                            } else {
                                "Disconnected · Pairing saved"
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.connections_connect_button(connected, cx))
                    .child(self.connections_remove_button(cx)),
            )
    }

    fn connection_status_dot(&self, connected: bool) -> impl IntoElement {
        div()
            .w(px(8.0))
            .h(px(8.0))
            .rounded(px(4.0))
            .bg(if connected {
                self.theme.primary
            } else {
                self.theme.muted_foreground.opacity(0.40)
            })
    }

    fn connections_connect_button(
        &self,
        connected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("connections-environment-connect")
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_1p5()
            .text_size(px(12.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_saved_environment_connected =
                    !this.connections_saved_environment_connected;
                cx.notify();
            }))
            .child(if connected { "Disconnect" } else { "Connect" })
    }

    fn connections_remove_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("connections-environment-remove")
            .rounded(px(7.0))
            .border_1()
            .border_color(hsla(0.0, 0.72, 0.48, 0.32))
            .bg(hsla(0.0, 0.72, 0.48, 0.05))
            .px_3()
            .py_1p5()
            .text_size(px(12.0))
            .text_color(hsla(0.0, 0.72, 0.48, 1.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_saved_environment = false;
                this.connections_saved_environment_connected = false;
                cx.notify();
            }))
            .child("Remove")
    }

    fn connections_add_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mode_body = match self.connections_mode {
            ConnectionMode::Remote => self.connections_remote_form(cx).into_any_element(),
            ConnectionMode::Ssh => self.connections_ssh_form(cx).into_any_element(),
        };

        div()
            .absolute()
            .top(px(88.0))
            .right(px(48.0))
            .w(px(560.0))
            .rounded(px(14.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
            .p_4()
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.12),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(28.0),
                spread_radius: px(-10.0),
            }])
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight(650.0))
                                    .child("Pair Environment"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(self.theme.muted_foreground)
                                    .child("Pair another environment to this client."),
                            ),
                    )
                    .child(
                        div()
                            .id("connections-close-environment-dialog")
                            .rounded(px(7.0))
                            .border_1()
                            .border_color(self.theme.border)
                            .px_2()
                            .py_1()
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.connections_add_dialog_open = false;
                                cx.notify();
                            }))
                            .child("Esc"),
                    ),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(self.connections_mode_card(
                        ConnectionMode::Remote,
                        "Remote link",
                        "Enter a backend host and pairing code.",
                        "icons/link-2.svg",
                        cx,
                    ))
                    .child(self.connections_mode_card(
                        ConnectionMode::Ssh,
                        "SSH",
                        "Use local SSH config, agent, and tunnels for the backend.",
                        "icons/terminal.svg",
                        cx,
                    )),
            )
            .child(div().mt_4().child(mode_body))
    }

    fn connections_mode_card(
        &self,
        mode: ConnectionMode,
        title: &'static str,
        description: &'static str,
        icon: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.connections_mode == mode;
        div()
            .id(match mode {
                ConnectionMode::Remote => "connections-mode-remote",
                ConnectionMode::Ssh => "connections-mode-ssh",
            })
            .flex()
            .items_center()
            .gap_3()
            .min_h(px(96.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(if selected {
                self.theme.primary.opacity(0.50)
            } else {
                self.theme.border
            })
            .bg(if selected {
                self.theme.primary.opacity(0.06)
            } else {
                self.theme.background
            })
            .p_4()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.connections_mode = mode;
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(32.0))
                    .h(px(32.0))
                    .rounded(px(7.0))
                    .border_1()
                    .border_color(if selected {
                        self.theme.primary.opacity(0.30)
                    } else {
                        self.theme.border
                    })
                    .bg(if selected {
                        self.theme.primary.opacity(0.10)
                    } else {
                        self.theme.card
                    })
                    .child(svg().path(icon).size_4().text_color(if selected {
                        self.theme.primary
                    } else {
                        self.theme.muted_foreground
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight(650.0))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground)
                            .child(description),
                    ),
            )
    }

    fn connections_remote_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(self.connection_field_box("Host", "backend.example.com"))
                    .child(self.connection_field_box("Pairing code", "PAIRCODE")),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground)
                    .child("Paste a full pairing URL here to fill both fields automatically."),
            )
            .child(self.connections_create_environment_button(cx))
    }

    fn connections_ssh_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.connection_field_box("SSH host or alias", "Search hosts or type devbox"))
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(self.connection_field_box("Username", "root"))
                    .child(self.connection_field_box("Port", "22")),
            )
            .child(
                div()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.background)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .border_b_1()
                            .border_color(self.theme.border)
                            .px_3()
                            .py_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .font_weight(FontWeight(650.0))
                                            .child("Suggested hosts"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(self.theme.muted_foreground)
                                            .child("From SSH config and known hosts"),
                                    ),
                            )
                            .child(self.connections_refresh_button(cx)),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_3()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground)
                            .child("No new SSH hosts were discovered."),
                    ),
            )
            .child(self.connections_create_environment_button(cx))
    }

    fn connection_field_box(&self, label: &'static str, value: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight(600.0))
                    .child(label),
            )
            .child(
                div()
                    .h(px(34.0))
                    .rounded(px(7.0))
                    .border_1()
                    .border_color(self.theme.border)
                    .bg(self.theme.background)
                    .px_3()
                    .py_2()
                    .text_size(px(13.0))
                    .text_color(self.theme.muted_foreground)
                    .child(value),
            )
    }

    fn connections_create_environment_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("connections-create-environment")
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .h(px(34.0))
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .text_size(px(13.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.connections_saved_environment = true;
                this.connections_saved_environment_connected = true;
                this.connections_add_dialog_open = false;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/plus-square.svg")
                    .size_4()
                    .text_color(self.theme.foreground),
            )
            .child("Create environment")
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

    fn settings_about_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let version_description = if self.settings_update_checked {
            "Current version of the application. No update is queued."
        } else {
            "Current version of the application."
        };
        let diagnostics_description = if self.settings_diagnostics_opened {
            "Diagnostics view selected."
        } else {
            "View runtime, environment, and integration diagnostics."
        };

        div()
            .flex()
            .flex_col()
            .gap_2p5()
            .child(self.settings_section_header("ABOUT"))
            .child(
                self.settings_about_card()
                    .child(self.settings_about_row(
                        "Version",
                        version_description,
                        self.settings_update_button(cx),
                        true,
                    ))
                    .child(self.settings_about_row(
                        "Update track",
                        "Stable follows full releases. Nightly follows the desktop channel.",
                        self.settings_about_badge("Stable"),
                        false,
                    ))
                    .child(self.settings_about_row(
                        "Diagnostics",
                        diagnostics_description,
                        self.settings_diagnostics_button(cx),
                        false,
                    )),
            )
    }

    fn settings_about_card(&self) -> gpui::Div {
        div()
            .relative()
            .overflow_hidden()
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card)
    }

    fn settings_about_row(
        &self,
        title: &'static str,
        description: &'static str,
        control: impl IntoElement,
        first: bool,
    ) -> impl IntoElement {
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
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight(650.0))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground.opacity(0.82))
                            .child(description),
                    ),
            )
            .child(control)
    }

    fn settings_update_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-check-updates")
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_1p5()
            .text_size(px(12.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.settings_update_checked = true;
                cx.notify();
            }))
            .child(if self.settings_update_checked {
                "Up to Date"
            } else {
                "Check for Updates"
            })
    }

    fn settings_diagnostics_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-view-diagnostics")
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_1p5()
            .text_size(px(12.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.settings_diagnostics_opened = true;
                cx.notify();
            }))
            .child(if self.settings_diagnostics_opened {
                "Diagnostics ready"
            } else {
                "View diagnostics"
            })
    }

    fn settings_about_badge(&self, label: &'static str) -> impl IntoElement {
        div()
            .rounded(px(7.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_1p5()
            .text_size(px(12.0))
            .text_color(self.theme.foreground)
            .child(label)
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
                control: SettingsControl::Select(SettingsSelectKind::TimeFormat),
            },
            SettingsRow {
                label: "Diff line wrapping",
                description: "Set the default wrap state when the diff panel opens.",
                control: SettingsControl::Toggle(0),
            },
            SettingsRow {
                label: "Hide whitespace changes",
                description: "Set whether the diff panel ignores whitespace-only edits by default.",
                control: SettingsControl::Toggle(1),
            },
            SettingsRow {
                label: "Assistant output",
                description: "Show token-by-token output while a response is in progress.",
                control: SettingsControl::Toggle(2),
            },
            SettingsRow {
                label: "Auto-open task panel",
                description: "Open the right-side plan and task panel automatically when steps appear.",
                control: SettingsControl::Toggle(3),
            },
            SettingsRow {
                label: "New threads",
                description: "Pick the default workspace mode for newly created draft threads.",
                control: SettingsControl::Select(SettingsSelectKind::NewThreads),
            },
            SettingsRow {
                label: "Add project starts in",
                description: "Leave empty to use \"~/\" when the Add Project browser opens.",
                control: SettingsControl::Select(SettingsSelectKind::ProjectBase),
            },
            SettingsRow {
                label: "Archive confirmation",
                description: "Require a second click on the inline archive action before a thread is archived.",
                control: SettingsControl::Toggle(4),
            },
            SettingsRow {
                label: "Delete confirmation",
                description: "Ask before deleting a thread and its chat history.",
                control: SettingsControl::Toggle(5),
            },
            SettingsRow {
                label: "Text generation model",
                description: "Configure the model used for generated commit messages and PR text.",
                control: SettingsControl::Select(SettingsSelectKind::TextGenerationModel),
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
                                    .font_weight(FontWeight(600.0))
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
            SettingsControl::Toggle(index) => self.settings_toggle(index, cx).into_any_element(),
            SettingsControl::Select(kind) => {
                self.settings_cycling_select(kind, cx).into_any_element()
            }
        }
    }

    fn settings_cycling_select(
        &self,
        kind: SettingsSelectKind,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let values = kind.options();
        let value_index = self.settings_select_values[kind.index()] % values.len();
        div()
            .id(kind.id())
            .flex()
            .items_center()
            .justify_between()
            .min_w(px(160.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_3()
            .py_2()
            .text_size(px(14.0))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                let index = kind.index();
                let option_count = kind.options().len();
                if let Some(value) = this.settings_select_values.get_mut(index) {
                    *value = (*value + 1) % option_count;
                    cx.notify();
                }
            }))
            .child(values[value_index])
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground)
                    .child("v"),
            )
    }

    fn settings_toggle(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let is_on = self.settings_toggle_values[index];
        div()
            .id(match index {
                0 => "settings-toggle-diff-wrap",
                1 => "settings-toggle-hide-whitespace",
                2 => "settings-toggle-assistant-output",
                3 => "settings-toggle-auto-open-task-panel",
                4 => "settings-toggle-archive-confirmation",
                _ => "settings-toggle-delete-confirmation",
            })
            .relative()
            .w(px(30.0))
            .h(px(18.0))
            .rounded(px(9.0))
            .cursor_pointer()
            .bg(if is_on {
                self.theme.primary
            } else {
                self.theme.accent
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(value) = this.settings_toggle_values.get_mut(index) {
                    *value = !*value;
                    cx.notify();
                }
            }))
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

    fn toolbar_icon_button(
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
            .w(px(28.0))
            .h(px(28.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                if id == "thread-terminal" {
                    let open = !this.snapshot.terminal_open();
                    this.snapshot.terminal_state =
                        set_thread_terminal_open(&this.snapshot.terminal_state, open);
                    cx.notify();
                } else if id == "thread-diff" {
                    this.snapshot.diff_route = if this.snapshot.diff_open() {
                        DiffRouteSearch::default()
                    } else {
                        let selected_turn_id = this
                            .snapshot
                            .ordered_turn_diff_summaries()
                            .first()
                            .map(|summary| summary.turn_id.clone());
                        parse_diff_route_search(
                            Some(DiffOpenValue::from("1")),
                            selected_turn_id.as_deref(),
                            None,
                        )
                    };
                    cx.notify();
                }
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn sidebar_active_project_group(&self, project: &ProjectSummary) -> impl IntoElement {
        div()
            .px_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .h(px(22.0))
                    .text_size(px(13.0))
                    .text_color(self.theme.foreground)
                    .child(
                        svg()
                            .path("icons/chevron-down.svg")
                            .size_3()
                            .text_color(self.theme.muted_foreground),
                    )
                    .child(
                        svg()
                            .path("icons/folder.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(self.theme.muted_foreground.opacity(0.50)),
                    )
                    .child(project.name.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mt_2()
                    .ml_3()
                    .pl_3()
                    .border_l_1()
                    .border_color(self.theme.border)
                    .h(px(24.0))
                    .text_size(px(13.0))
                    .child(div().child("New thread"))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(self.theme.muted_foreground.opacity(0.70))
                            .child("just now"),
                    ),
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

    fn respond_to_pending_approval(&mut self, request_id: &str, cx: &mut Context<Self>) {
        if !self.snapshot.is_responding_to_request(request_id) {
            self.snapshot
                .responding_request_ids
                .push(request_id.to_string());
        }
        cx.notify();
    }

    fn select_pending_user_input_option(
        &mut self,
        question_id: &str,
        option_label: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(question) = self
            .snapshot
            .active_pending_user_input()
            .and_then(|prompt| {
                prompt
                    .questions
                    .iter()
                    .find(|question| question.id == question_id)
            })
            .cloned()
        else {
            return;
        };
        let draft = self
            .snapshot
            .pending_user_input_draft_answers
            .get(question_id)
            .cloned();
        let next_draft =
            toggle_pending_user_input_option_selection(&question, draft.as_ref(), option_label);
        self.snapshot
            .pending_user_input_draft_answers
            .insert(question_id.to_string(), next_draft);

        if !question.multi_select {
            self.advance_pending_user_input(cx);
        } else {
            cx.notify();
        }
    }

    fn set_active_pending_user_input_custom_answer(
        &mut self,
        custom_answer: String,
        cx: &mut Context<Self>,
    ) {
        let Some(question_id) = self
            .snapshot
            .active_pending_user_input_progress()
            .and_then(|progress| progress.active_question.map(|question| question.id))
        else {
            return;
        };
        let draft = self
            .snapshot
            .pending_user_input_draft_answers
            .get(&question_id)
            .cloned();
        let next_draft = set_pending_user_input_custom_answer(draft.as_ref(), custom_answer);
        self.snapshot
            .pending_user_input_draft_answers
            .insert(question_id, next_draft);
        cx.notify();
    }

    fn advance_pending_user_input(&mut self, cx: &mut Context<Self>) {
        let Some(progress) = self.snapshot.active_pending_user_input_progress() else {
            return;
        };
        if progress.is_last_question {
            if !progress.is_complete {
                return;
            }
            let request_id = self
                .snapshot
                .active_pending_user_input()
                .map(|prompt| prompt.request_id.clone());
            if let Some(request_id) = request_id {
                if !self.snapshot.is_responding_to_request(&request_id) {
                    self.snapshot.responding_request_ids.push(request_id);
                }
            }
        } else if progress.can_advance {
            self.snapshot.active_pending_user_input_question_index =
                progress.question_index.saturating_add(1);
        }
        cx.notify();
    }

    fn previous_pending_user_input_question(&mut self, cx: &mut Context<Self>) {
        self.snapshot.active_pending_user_input_question_index = self
            .snapshot
            .active_pending_user_input_question_index
            .saturating_sub(1);
        cx.notify();
    }

    fn handle_terminal_action(&mut self, id: &str, cx: &mut Context<Self>) {
        match id {
            "terminal-split" | "terminal-sidebar-split" => {
                let active_group_size = self
                    .snapshot
                    .terminal_state
                    .terminal_groups
                    .iter()
                    .find(|group| group.id == self.snapshot.terminal_state.active_terminal_group_id)
                    .map(|group| group.terminal_ids.len())
                    .unwrap_or(1);
                if active_group_size < MAX_TERMINALS_PER_GROUP {
                    let terminal_id = self.next_terminal_id();
                    self.snapshot.terminal_state =
                        split_thread_terminal(&self.snapshot.terminal_state, &terminal_id);
                }
            }
            "terminal-new" | "terminal-sidebar-new" => {
                let terminal_id = self.next_terminal_id();
                self.snapshot.terminal_state =
                    new_thread_terminal(&self.snapshot.terminal_state, &terminal_id);
            }
            "terminal-close" | "terminal-sidebar-close" => {
                let terminal_id = self.snapshot.terminal_state.active_terminal_id.clone();
                self.snapshot.terminal_state =
                    close_thread_terminal(&self.snapshot.terminal_state, &terminal_id);
            }
            _ => {}
        }
        cx.notify();
    }

    fn submit_composer(&mut self, cx: &mut Context<Self>) {
        if self.snapshot.active_pending_approval().is_some() {
            return;
        }
        if self.snapshot.active_pending_user_input().is_some() {
            self.advance_pending_user_input(cx);
            return;
        }
        if self.composer_prompt.trim().is_empty() {
            return;
        }

        self.composer_prompt.clear();
        self.composer_prompt_focused = true;
        self.composer_submitted_count = self.composer_submitted_count.saturating_add(1);
        cx.notify();
    }

    fn on_composer_key_down(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let handled = match event.keystroke.key.as_str() {
            "enter" => {
                self.submit_composer(cx);
                true
            }
            "backspace" => {
                if self.snapshot.active_pending_approval().is_none() {
                    if let Some(progress) = self.snapshot.active_pending_user_input_progress() {
                        let mut custom_answer = progress.custom_answer;
                        custom_answer.pop();
                        self.set_active_pending_user_input_custom_answer(custom_answer, cx);
                    } else {
                        self.composer_prompt.pop();
                        cx.notify();
                    }
                }
                true
            }
            "escape" => {
                self.composer_prompt_focused = false;
                cx.notify();
                true
            }
            _ => {
                let modifiers = event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
                    false
                } else if let Some(text) = event.keystroke.key_char.as_deref() {
                    if text != "\n" && text != "\t" {
                        if self.snapshot.active_pending_approval().is_none() {
                            if let Some(progress) =
                                self.snapshot.active_pending_user_input_progress()
                            {
                                let mut custom_answer = progress.custom_answer;
                                custom_answer.push_str(text);
                                self.set_active_pending_user_input_custom_answer(custom_answer, cx);
                            } else {
                                self.composer_prompt.push_str(text);
                                self.composer_prompt_focused = true;
                                cx.notify();
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };

        if handled {
            cx.stop_propagation();
        }
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

fn pending_blue() -> gpui::Hsla {
    hsla(217.0 / 360.0, 0.91, 0.60, 1.0)
}

fn pending_blue_icon() -> gpui::Hsla {
    hsla(213.0 / 360.0, 0.94, 0.68, 1.0)
}

fn diff_success_color() -> gpui::Hsla {
    hsla(142.0 / 360.0, 0.71, 0.38, 1.0)
}

fn diff_destructive_color() -> gpui::Hsla {
    hsla(0.0, 0.72, 0.48, 1.0)
}

fn short_timestamp_label(timestamp: &str) -> String {
    let Some(time) = timestamp.split('T').nth(1) else {
        return timestamp.to_string();
    };
    let mut parts = time.split(':');
    let Some(hour) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
        return timestamp.to_string();
    };
    let Some(minute) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
        return timestamp.to_string();
    };
    let suffix = if hour >= 12 { "PM" } else { "AM" };
    let hour = match hour % 12 {
        0 => 12,
        value => value,
    };
    format!("{hour}:{minute:02} {suffix}")
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
        Ok("draft") | Ok("chat-composer") => (R3Screen::Draft, false),
        Ok("active-chat") | Ok("chat") => (R3Screen::ActiveChat, false),
        Ok("pending-approval") | Ok("approval") => (R3Screen::PendingApproval, false),
        Ok("pending-user-input") | Ok("user-input") => (R3Screen::PendingUserInput, false),
        Ok("terminal-drawer") | Ok("terminal") => (R3Screen::TerminalDrawer, false),
        Ok("diff-panel") | Ok("diff") => (R3Screen::DiffPanel, false),
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
            let snapshot = match screen {
                R3Screen::Draft => AppSnapshot::draft_reference_state(),
                R3Screen::ActiveChat => AppSnapshot::mock_reference_state(),
                R3Screen::PendingApproval => AppSnapshot::pending_approval_reference_state(),
                R3Screen::PendingUserInput => AppSnapshot::pending_user_input_reference_state(),
                R3Screen::TerminalDrawer => AppSnapshot::terminal_drawer_reference_state(),
                R3Screen::DiffPanel => AppSnapshot::diff_panel_reference_state(),
                R3Screen::Empty | R3Screen::Settings => AppSnapshot::empty_reference_state(),
            };
            let shell = cx.new(|cx| {
                cx.observe_window_appearance(window, |_, window, _| {
                    window.refresh();
                })
                .detach();
                R3Shell::new(snapshot, screen, theme_mode, command_palette_open, cx)
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

use std::collections::BTreeMap;

use gpui::prelude::{FluentBuilder, InteractiveElement, StatefulInteractiveElement};
use gpui::{
    AnyElement, App, AppContext, BoxShadow, Context, CursorStyle, FocusHandle, Focusable,
    FontWeight, IntoElement, KeyDownEvent, ParentElement, Pixels, Render, SharedString, Styled,
    TextAlign, Window, div, hsla, point, px, svg,
};
use r3_core::{
    APP_NAME, ActivityTone, AppSnapshot, ChatMessage, CommandPaletteGroup, CommandPaletteItem,
    CommandPaletteItemKind, ComposerCommandItem, ComposerMenuNudgeDirection, ComposerPromptSegment,
    ComposerSlashCommand, ComposerTrigger, DiagnosticsDescriptionInput, DiffOpenValue,
    DiffRouteSearch, DraftThreadEnvMode, EditorOption, GitActionIconName, GitActionMenuItem,
    GitStatusSnapshot, KeybindingSettingsRow, MAX_TERMINALS_PER_GROUP,
    MAX_VISIBLE_WORK_LOG_ENTRIES, ModelPickerItem, ModelPickerSelectedInstance, ModelPickerState,
    PendingApproval, PendingUserInputProgress, ProcessDiagnosticsEntry, ProcessDiagnosticsResult,
    ProjectEntry, ProjectEntryKind, ProjectScript, ProjectScriptIcon, ProjectSummary,
    ProviderInstanceEntry, RECENT_COMMAND_PALETTE_THREAD_LIMIT, ServerProviderModel,
    ServerProviderSkill, ServerProviderSlashCommand, SidebarOptionsState,
    SidebarProjectGroupingMode, SidebarProjectSortOrder, SidebarThreadSortOrder, TerminalEvent,
    ThreadStatus, TraceDiagnosticsFailureSummary, TraceDiagnosticsLogEvent,
    TraceDiagnosticsRecentFailure, TraceDiagnosticsResult, TraceDiagnosticsSpanOccurrence,
    TraceDiagnosticsSpanSummary, TurnDiffFileChange, TurnDiffStat, TurnDiffSummary,
    TurnDiffTreeNode, WorkLogEntry, build_composer_menu_items, build_default_keybinding_rows,
    build_git_action_menu_items, build_project_action_items, build_root_command_palette_groups,
    build_thread_action_items, build_turn_diff_tree, close_thread_terminal,
    composer_menu_search_key, default_sidebar_options_state, detect_composer_trigger,
    filter_command_palette_groups, format_diagnostics_bytes, format_diagnostics_count,
    format_diagnostics_description, format_diagnostics_duration_ms,
    format_provider_skill_display_name, format_working_timer_at, get_display_model_name,
    get_provider_summary, get_provider_version_advisory_presentation, get_provider_version_label,
    group_composer_command_items, new_thread_terminal, nudge_composer_menu_highlight,
    ordered_turn_diff_files, parse_diff_route_search, primary_project_script,
    provider_instance_initials, replace_text_range, resolve_composer_command_selection,
    resolve_composer_menu_active_item_id, resolve_editor_options, resolve_model_picker_state,
    resolve_selectable_model, set_pending_user_input_custom_answer, set_thread_active_terminal,
    set_thread_terminal_open, shorten_trace_id, sidebar_project_grouping_label,
    sidebar_project_grouping_options, sidebar_project_sort_label, sidebar_project_sort_options,
    sidebar_thread_sort_label, sidebar_thread_sort_options, split_prompt_into_composer_segments,
    split_thread_terminal, summarize_turn_diff_stats, toggle_pending_user_input_option_selection,
};

use crate::theme::{FONT_FAMILY, MONO_FONT_FAMILY, SIDEBAR_MIN_WIDTH, Theme, ThemeMode};

const COMMAND_PALETTE_REFERENCE_NOW_ISO: &str = "2026-03-25T12:00:00.000Z";
const REFERENCE_WORKING_TIMER_NOW: &str = "2026-05-12T09:35:10.000Z";

#[derive(Debug, Clone, Copy)]
struct DiffPatchRow {
    old_line: Option<u32>,
    new_line: Option<u32>,
    text: &'static str,
    kind: DiffPatchRowKind,
}

#[derive(Debug, Clone, Copy)]
enum DiffPatchRowKind {
    Context,
    Meta,
    Addition,
    Deletion,
}

pub struct R3Shell {
    snapshot: AppSnapshot,
    project_sort_ascending: bool,
    theme: Theme,
    theme_mode: ThemeMode,
    screen: R3Screen,
    command_palette_open: bool,
    command_palette_query: String,
    command_palette_highlighted_index: usize,
    command_palette_submenu: Option<CommandPaletteSubmenu>,
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
    sidebar_options_menu_open: bool,
    sidebar_options_state: SidebarOptionsState,
    project_script_run_count: usize,
    project_script_menu_open: bool,
    opened_editor_count: usize,
    open_in_menu_open: bool,
    git_actions_menu_open: bool,
    providers_refresh_requested: bool,
    providers_add_dialog_open: bool,
    expanded_provider_index: Option<usize>,
    provider_enabled: [bool; 5],
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
    composer_highlighted_item_id: Option<String>,
    composer_highlighted_search_key: Option<String>,
    model_picker_open: bool,
    model_picker_query: String,
    model_picker_selected_instance: ModelPickerSelectedInstance,
    composer_compact_controls_menu_open: bool,
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
            command_palette_submenu: None,
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
            sidebar_options_menu_open: screen == R3Screen::SidebarOptionsMenu,
            sidebar_options_state: default_sidebar_options_state(),
            project_script_run_count: 0,
            project_script_menu_open: screen == R3Screen::ProjectScriptsMenu,
            opened_editor_count: 0,
            open_in_menu_open: screen == R3Screen::OpenInMenu,
            git_actions_menu_open: screen == R3Screen::GitActionsMenu,
            providers_refresh_requested: false,
            providers_add_dialog_open: false,
            expanded_provider_index: Some(0),
            provider_enabled: [true, true, true, false, true],
            connections_network_accessible: false,
            connections_add_dialog_open: false,
            connections_mode: ConnectionMode::Remote,
            connections_saved_environment: false,
            connections_saved_environment_connected: false,
            connections_endpoint_copied: false,
            connections_refresh_requested: false,
            settings_theme_highlighted_index: 0,
            composer_prompt: match screen {
                R3Screen::ComposerCommandMenu => "/".to_string(),
                R3Screen::ComposerInlineTokens => "use @AGENTS.md and $agent-browser ".to_string(),
                _ => String::new(),
            },
            composer_prompt_focused: matches!(
                screen,
                R3Screen::ComposerFocused
                    | R3Screen::ComposerCommandMenu
                    | R3Screen::ComposerInlineTokens
            ),
            composer_highlighted_item_id: None,
            composer_highlighted_search_key: None,
            model_picker_open: screen == R3Screen::ProviderModelPicker,
            model_picker_query: String::new(),
            model_picker_selected_instance: if screen == R3Screen::ProviderModelPicker {
                ModelPickerSelectedInstance::Instance("codex".to_string())
            } else {
                ModelPickerSelectedInstance::Favorites
            },
            composer_compact_controls_menu_open: false,
            composer_runtime_index: 2,
            composer_plan_mode: screen == R3Screen::PendingUserInput,
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
    ComposerFocused,
    ActiveChat,
    RunningTurn,
    PendingApproval,
    PendingUserInput,
    ComposerCommandMenu,
    ComposerInlineTokens,
    TerminalDrawer,
    DiffPanel,
    BranchToolbar,
    SidebarOptionsMenu,
    ProjectScriptsMenu,
    OpenInMenu,
    GitActionsMenu,
    ProviderModelPicker,
    Settings,
    SettingsDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandPaletteAction {
    NewThread,
    NewThreadInProject,
    AddProject,
    OpenSettings,
    OpenProject,
    OpenThread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandPaletteSubmenu {
    NewThreadInProject,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderInstanceRow {
    label: String,
    id: String,
    driver: String,
    status: ProviderStatus,
    badge: Option<String>,
    description: String,
    enabled: bool,
    version_label: Option<String>,
    advisory_detail: Option<String>,
    model_count: usize,
    accent_color: Option<String>,
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
struct ComposerRuntimeMode {
    label: &'static str,
    icon: &'static str,
}

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
            | R3Screen::ComposerFocused
            | R3Screen::ActiveChat
            | R3Screen::RunningTurn
            | R3Screen::PendingApproval
            | R3Screen::PendingUserInput
            | R3Screen::ComposerCommandMenu
            | R3Screen::ComposerInlineTokens
            | R3Screen::TerminalDrawer
            | R3Screen::DiffPanel
            | R3Screen::BranchToolbar
            | R3Screen::SidebarOptionsMenu
            | R3Screen::ProjectScriptsMenu
            | R3Screen::OpenInMenu
            | R3Screen::GitActionsMenu
            | R3Screen::ProviderModelPicker => {
                root.child(self.sidebar(cx)).child(self.main_panel(cx))
            }
            R3Screen::Settings | R3Screen::SettingsDiagnostics => root
                .child(self.settings_sidebar(cx))
                .child(self.settings_panel(cx)),
        };

        if self.command_palette_open {
            root = root.child(self.command_palette_overlay(cx));
        }

        if self.model_picker_open && self.snapshot.renders_chat_view() {
            root = root.child(self.model_picker_popup(cx));
        }

        if self.composer_compact_controls_menu_open && self.snapshot.renders_chat_view() {
            root = root.child(self.composer_compact_controls_menu_popup(cx));
        }

        if self.project_script_menu_open && self.snapshot.renders_chat_view() {
            root = root.child(self.project_scripts_menu_popup(cx));
        }

        if self.open_in_menu_open && self.snapshot.open_in_picker_visible() {
            root = root.child(self.open_in_menu_popup(cx));
        }

        if self.git_actions_menu_open && self.snapshot.is_git_repo {
            root = root.child(self.git_actions_menu_popup(cx));
        }

        if self.sidebar_options_menu_open {
            root = root.child(self.sidebar_options_menu_popup(cx));
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
                        .py_1()
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
                .pb_4()
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
                .pb_3()
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
            .child(self.timeline(cx));

        if self.snapshot.renders_chat_view() {
            panel = panel.child(self.composer(cx));
            if self.snapshot.active_branch_toolbar_state().is_some() {
                panel = panel.child(self.branch_toolbar(cx));
            }
        }

        if self.snapshot.terminal_open() {
            panel = panel.child(self.terminal_drawer(cx));
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
                    .child(self.project_scripts_control(cx))
                    .when(self.snapshot.open_in_picker_visible(), |actions| {
                        actions.child(self.open_in_picker(cx))
                    })
                    .when(self.snapshot.is_git_repo, |actions| {
                        actions.child(self.git_actions_control(cx))
                    })
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

    fn timeline(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.snapshot.messages.is_empty() {
            return self.messages_timeline(cx).into_any_element();
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

    fn messages_timeline(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content_width = if self.snapshot.diff_open() {
            460.0
        } else {
            760.0
        };
        let mut content = div().flex().flex_col().gap_4().w(px(content_width));

        for message in &self.snapshot.messages {
            content = content.child(self.timeline_message(message, cx));
        }
        let work_log_entries = self.snapshot.work_log_entries();
        if !work_log_entries.is_empty() {
            content = content.child(self.work_log_group(&work_log_entries));
        }
        if let Some(started_at) = self.snapshot.active_work_started_at.as_deref() {
            content = content.child(self.working_timeline_row(started_at));
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

    fn timeline_message(&self, message: &ChatMessage, cx: &mut Context<Self>) -> impl IntoElement {
        match message.role {
            r3_core::MessageRole::User => self.user_timeline_message(message).into_any_element(),
            r3_core::MessageRole::Assistant | r3_core::MessageRole::System => self
                .assistant_timeline_message(message, cx)
                .into_any_element(),
        }
    }

    fn user_timeline_message(&self, message: &ChatMessage) -> impl IntoElement {
        let bubble_max_width = if self.snapshot.diff_open() {
            368.0
        } else {
            608.0
        };
        div().flex().justify_end().child(
            div()
                .rounded(px(16.0))
                .rounded_br(px(2.0))
                .border_1()
                .border_color(self.theme.border)
                .bg(self.theme.accent)
                .px_4()
                .py_3()
                .max_w(px(bubble_max_width))
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

    fn assistant_timeline_message(
        &self,
        message: &ChatMessage,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
            .when_some(
                self.turn_diff_summary_for_message(&message.id),
                |message_node, summary| {
                    message_node.child(self.assistant_changed_files_section(summary, cx))
                },
            )
            .child(
                div()
                    .mt_1p5()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(10.0))
                    .text_color(self.theme.muted_foreground.opacity(0.30))
                    .child("12:00 PM"),
            )
    }

    fn turn_diff_summary_for_message(&self, message_id: &str) -> Option<&TurnDiffSummary> {
        self.snapshot
            .turn_diff_summaries
            .iter()
            .find(|summary| summary.assistant_message_id.as_deref() == Some(message_id))
    }

    fn assistant_changed_files_section(
        &self,
        summary: &TurnDiffSummary,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let files = &summary.files;
        let stat = summarize_turn_diff_stats(files);
        let tree = build_turn_diff_tree(files);
        let turn_id = summary.turn_id.clone();
        let first_file_path = files.first().map(|file| file.path.clone());

        div()
            .id(SharedString::from(format!(
                "assistant-changed-files-{}",
                summary.turn_id
            )))
            .mt_2()
            .mx(px(-4.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.80))
            .bg(self.theme.card.opacity(0.45))
            .p_2()
            .child(
                div()
                    .mb_1p5()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_size(px(10.0))
                            .font_weight(FontWeight(650.0))
                            .text_color(self.theme.muted_foreground.opacity(0.65))
                            .child(format!("Changed files ({})", files.len()).to_uppercase())
                            .when(stat.additions > 0 || stat.deletions > 0, |label| {
                                label
                                    .child(
                                        div()
                                            .mx_1()
                                            .text_color(self.theme.muted_foreground.opacity(0.70))
                                            .child("•"),
                                    )
                                    .child(self.diff_stat_label(stat, false))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(self.changed_files_header_button("Collapse all", None, cx))
                            .child(self.changed_files_header_button(
                                "View diff",
                                Some((turn_id, first_file_path)),
                                cx,
                            )),
                    ),
            )
            .child(self.changed_files_tree_nodes(&tree, &summary.turn_id, 0, cx))
    }

    fn changed_files_header_button(
        &self,
        label: &'static str,
        open_diff: Option<(String, Option<String>)>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!(
                "changed-files-button-{}",
                label.to_ascii_lowercase().replace(' ', "-")
            )))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .px_2()
            .py_1()
            .text_size(px(11.0))
            .text_color(self.theme.foreground.opacity(0.90))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some((turn_id, file_path)) = open_diff.as_ref() {
                    this.snapshot.diff_route = parse_diff_route_search(
                        Some(DiffOpenValue::from("1")),
                        Some(turn_id),
                        file_path.as_deref(),
                    );
                    cx.notify();
                }
            }))
            .child(label)
    }

    fn changed_files_tree_nodes(
        &self,
        nodes: &[TurnDiffTreeNode],
        turn_id: &str,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut list = div().flex().flex_col();
        for node in nodes {
            list = list.child(self.changed_files_tree_node(node, turn_id, depth, cx));
        }
        list
    }

    fn changed_files_tree_node(
        &self,
        node: &TurnDiffTreeNode,
        turn_id: &str,
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
                .id(SharedString::from(format!("changed-files-dir-{path}")))
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .rounded(px(6.0))
                        .py_0p5()
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
                .child(self.changed_files_tree_nodes(children, turn_id, depth + 1, cx))
                .into_any_element(),
            TurnDiffTreeNode::File { name, path, stat } => {
                let turn_id_for_click = turn_id.to_string();
                let path_for_click = path.clone();
                let icon_path = if name == "diff.svg" {
                    "icons/diff.svg"
                } else {
                    "icons/file-json.svg"
                };
                div()
                    .id(SharedString::from(format!("changed-files-file-{path}")))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .rounded(px(6.0))
                    .py_1()
                    .pr_2()
                    .pl(px(left_padding))
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground.opacity(0.82))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.snapshot.diff_route = parse_diff_route_search(
                            Some(DiffOpenValue::from("1")),
                            Some(&turn_id_for_click),
                            Some(&path_for_click),
                        );
                        cx.notify();
                    }))
                    .child(div().w(px(14.0)).h(px(14.0)).flex_shrink_0())
                    .child(
                        svg()
                            .path(icon_path)
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

    fn work_log_group(&self, entries: &[WorkLogEntry]) -> impl IntoElement {
        let has_overflow = entries.len() > MAX_VISIBLE_WORK_LOG_ENTRIES;
        let visible_start = if has_overflow {
            entries.len() - MAX_VISIBLE_WORK_LOG_ENTRIES
        } else {
            0
        };
        let visible_entries = &entries[visible_start..];
        let only_tool_entries = entries.iter().all(|entry| entry.tone == ActivityTone::Tool);
        let show_header = has_overflow || !only_tool_entries;
        let group_label = if only_tool_entries {
            "Tool calls"
        } else {
            "Work log"
        };

        let mut rows = div().flex().flex_col().gap(px(2.0));
        for entry in visible_entries {
            rows = rows.child(self.work_log_entry_row(entry));
        }

        div()
            .id("timeline-work-log")
            .rounded(px(12.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.45))
            .bg(self.theme.card.opacity(0.25))
            .px_2()
            .py_1p5()
            .when(show_header, |group| {
                group.child(
                    div()
                        .mb_1p5()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .px_0p5()
                        .child(
                            div()
                                .text_size(px(9.0))
                                .font_weight(FontWeight(650.0))
                                .text_color(self.theme.muted_foreground.opacity(0.55))
                                .child(format!("{group_label} ({})", entries.len()).to_uppercase()),
                        )
                        .when(has_overflow, |header| {
                            header.child(
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight(650.0))
                                    .text_color(self.theme.muted_foreground.opacity(0.55))
                                    .child(format!(
                                        "SHOW {} MORE",
                                        entries.len() - visible_entries.len()
                                    )),
                            )
                        }),
                )
            })
            .child(rows)
    }

    fn work_log_entry_row(&self, entry: &WorkLogEntry) -> impl IntoElement {
        let heading = work_log_heading(entry);
        let preview = work_log_preview(entry).filter(|preview| {
            normalize_work_log_label(preview).to_ascii_lowercase()
                != normalize_work_log_label(&heading).to_ascii_lowercase()
        });
        let display_text = preview
            .as_ref()
            .map(|preview| format!("{heading} - {preview}"))
            .unwrap_or(heading);
        let icon_path = work_log_icon_path(entry);
        div()
            .id(SharedString::from(format!("work-log-entry-{}", entry.id)))
            .rounded(px(8.0))
            .px_1()
            .py_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(20.0))
                            .h(px(20.0))
                            .flex_shrink_0()
                            .child(
                                svg()
                                    .path(icon_path)
                                    .size_3()
                                    .text_color(work_log_tone_color(entry.tone, self.theme)),
                            ),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .overflow_hidden()
                            .text_size(px(12.0))
                            .text_color(work_log_tone_color(entry.tone, self.theme))
                            .child(display_text),
                    ),
            )
    }

    fn working_timeline_row(&self, started_at: &str) -> impl IntoElement {
        let label = format!(
            "Working for {}",
            working_timer_label(started_at).unwrap_or_else(|| "0s".to_string())
        );
        div()
            .id("timeline-working-indicator")
            .py_0p5()
            .pl_1p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pt_1()
                    .text_size(px(11.0))
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .w(px(4.0))
                                    .h(px(4.0))
                                    .rounded(px(2.0))
                                    .bg(self.theme.muted_foreground.opacity(0.30)),
                            )
                            .child(
                                div()
                                    .w(px(4.0))
                                    .h(px(4.0))
                                    .rounded(px(2.0))
                                    .bg(self.theme.muted_foreground.opacity(0.30)),
                            )
                            .child(
                                div()
                                    .w(px(4.0))
                                    .h(px(4.0))
                                    .rounded(px(2.0))
                                    .bg(self.theme.muted_foreground.opacity(0.30)),
                            ),
                    )
                    .child(label),
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

    fn terminal_viewport(&self, terminal_id: &str, _index: usize) -> impl IntoElement {
        let mut body = div()
            .h_full()
            .rounded(px(4.0))
            .bg(self.theme.background)
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(12.0))
            .line_height(px(14.4))
            .text_color(self.theme.foreground)
            .overflow_hidden();

        let lines = self.terminal_lines_for(terminal_id);
        for line in lines {
            body = body.child(div().child(line));
        }

        body
    }

    fn terminal_lines_for(&self, terminal_id: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for entry in &self.snapshot.terminal_event_entries {
            if entry.event.terminal_id() != terminal_id {
                continue;
            }
            match &entry.event {
                TerminalEvent::Started { snapshot, .. }
                | TerminalEvent::Restarted { snapshot, .. } => {
                    lines.clear();
                    for line in snapshot.history.lines() {
                        if !line.trim().is_empty() {
                            lines.push(line.to_string());
                        }
                    }
                }
                TerminalEvent::Output { data, .. } => {
                    for line in data.lines() {
                        if !line.trim().is_empty() {
                            lines.push(line.to_string());
                        }
                    }
                }
                TerminalEvent::Activity { .. } => {}
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
                    let details = exit_signal
                        .clone()
                        .or_else(|| exit_code.map(|code| format!("code {code}")));
                    lines.push(match details {
                        Some(details) => format!("[terminal] Process exited ({details})"),
                        None => "[terminal] Process exited".to_string(),
                    });
                }
            }
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
            .w(px(435.0))
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

        let mut patch_surface = div().flex().flex_col().gap_2().p_2();
        for (index, file) in files.iter().enumerate() {
            patch_surface = patch_surface.child(self.diff_file_card(file, index, cx));
        }

        div()
            .id("diff-panel-viewport")
            .min_h_0()
            .min_w_0()
            .flex_1()
            .overflow_y_scroll()
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
            return ordered_turn_diff_files(&summary.files);
        }

        let files = self
            .snapshot
            .turn_diff_summaries
            .iter()
            .flat_map(|summary| summary.files.clone())
            .collect::<Vec<_>>();
        ordered_turn_diff_files(&files)
    }

    fn diff_file_card(
        &self,
        file: &TurnDiffFileChange,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let stat = TurnDiffStat {
            additions: file.additions.unwrap_or(0),
            deletions: file.deletions.unwrap_or(0),
        };
        let header_icon_color = self.diff_kind_color(file.kind.as_deref());
        let file_path_for_selection = file.path.clone();
        let mut card = div()
            .id(SharedString::from(format!("diff-render-file-{index}")))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.60))
            .bg(self.theme.card.opacity(0.90))
            .overflow_hidden()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.snapshot
                    .select_diff_file_path(file_path_for_selection.clone());
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .h(px(44.0))
                    .border_b_1()
                    .border_color(self.theme.border.opacity(0.70))
                    .bg(self.theme.card.opacity(0.94))
                    .px_4()
                    .text_size(px(13.0))
                    .child(
                        svg()
                            .path("icons/chevron-down.svg")
                            .size_4()
                            .text_color(header_icon_color),
                    )
                    .child(
                        svg()
                            .path("icons/square-pen.svg")
                            .size_4()
                            .text_color(header_icon_color),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .font_weight(FontWeight(700.0))
                            .text_color(self.theme.foreground.opacity(0.90))
                            .child(file.path.clone()),
                    )
                    .child(self.diff_panel_file_stat_label(stat)),
            );

        for row in self.diff_file_rows(file.path.as_str()) {
            let row_is_meta = matches!(row.kind, DiffPatchRowKind::Meta);
            card = card.child(self.diff_patch_row(row));
            if row_is_meta {
                card = card.child(div().h(px(6.0)).bg(self.theme.background));
            }
        }

        card = card.child(div().h(px(4.0)).bg(self.theme.background));

        card
    }

    fn diff_file_rows(&self, path: &str) -> Vec<DiffPatchRow> {
        match path {
            "crates/r3_core/src/lib.rs" => vec![
                DiffPatchRow {
                    old_line: None,
                    new_line: None,
                    text: "9 unmodified lines",
                    kind: DiffPatchRowKind::Meta,
                },
                DiffPatchRow {
                    old_line: Some(10),
                    new_line: Some(10),
                    text: "    pub terminal_open: bool,",
                    kind: DiffPatchRowKind::Context,
                },
                DiffPatchRow {
                    old_line: None,
                    new_line: Some(11),
                    text: "    pub active_terminal_group_id: String,",
                    kind: DiffPatchRowKind::Addition,
                },
                DiffPatchRow {
                    old_line: Some(12),
                    new_line: Some(12),
                    text: "}",
                    kind: DiffPatchRowKind::Context,
                },
            ],
            "crates/r3_ui/src/shell.rs" => vec![
                DiffPatchRow {
                    old_line: None,
                    new_line: None,
                    text: "41 unmodified lines",
                    kind: DiffPatchRowKind::Meta,
                },
                DiffPatchRow {
                    old_line: Some(42),
                    new_line: None,
                    text: "    draw_static_terminal();",
                    kind: DiffPatchRowKind::Deletion,
                },
                DiffPatchRow {
                    old_line: None,
                    new_line: Some(42),
                    text: "    draw_split_terminal();",
                    kind: DiffPatchRowKind::Addition,
                },
                DiffPatchRow {
                    old_line: None,
                    new_line: Some(43),
                    text: "    draw_terminal_sidebar();",
                    kind: DiffPatchRowKind::Addition,
                },
                DiffPatchRow {
                    old_line: Some(44),
                    new_line: Some(44),
                    text: "}",
                    kind: DiffPatchRowKind::Context,
                },
            ],
            _ => vec![DiffPatchRow {
                old_line: None,
                new_line: None,
                text: "No patch available for this file.",
                kind: DiffPatchRowKind::Meta,
            }],
        }
    }

    fn diff_patch_row(&self, row: DiffPatchRow) -> AnyElement {
        if matches!(row.kind, DiffPatchRowKind::Meta) {
            return div()
                .flex()
                .items_center()
                .min_h(px(32.0))
                .bg(self.theme.accent.opacity(0.46))
                .px_4()
                .font_family(SharedString::from(MONO_FONT_FAMILY))
                .text_size(px(13.0))
                .line_height(px(20.0))
                .text_color(self.theme.foreground.opacity(0.74))
                .child(row.text)
                .into_any_element();
        }

        let background = match row.kind {
            DiffPatchRowKind::Addition => diff_success_color().opacity(0.08),
            DiffPatchRowKind::Deletion => diff_destructive_color().opacity(0.08),
            DiffPatchRowKind::Meta => self.theme.accent.opacity(0.46),
            DiffPatchRowKind::Context => self.theme.background,
        };
        let marker_color = match row.kind {
            DiffPatchRowKind::Addition => diff_success_color(),
            DiffPatchRowKind::Deletion => diff_destructive_color(),
            DiffPatchRowKind::Meta | DiffPatchRowKind::Context => {
                self.theme.muted_foreground.opacity(0.45)
            }
        };
        let text_color = match row.kind {
            DiffPatchRowKind::Addition | DiffPatchRowKind::Deletion => diff_syntax_plain_color(),
            DiffPatchRowKind::Meta => self.theme.foreground.opacity(0.74),
            DiffPatchRowKind::Context => diff_syntax_plain_color(),
        };
        let line_number = match row.kind {
            DiffPatchRowKind::Deletion => row.old_line,
            _ => row.new_line.or(row.old_line),
        };
        let line_number_color = match row.kind {
            DiffPatchRowKind::Addition | DiffPatchRowKind::Deletion => marker_color,
            DiffPatchRowKind::Context => self.theme.foreground.opacity(0.58),
            DiffPatchRowKind::Meta => self.theme.muted_foreground.opacity(0.60),
        };
        let change_bar_color = match row.kind {
            DiffPatchRowKind::Addition | DiffPatchRowKind::Deletion => marker_color,
            DiffPatchRowKind::Meta | DiffPatchRowKind::Context => self.theme.background,
        };
        let row_height = if matches!(row.kind, DiffPatchRowKind::Meta) {
            32.0
        } else {
            20.0
        };

        div()
            .flex()
            .items_center()
            .min_h(px(row_height))
            .bg(background)
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(13.0))
            .line_height(px(20.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(row_height))
                    .flex_shrink_0()
                    .bg(change_bar_color),
            )
            .child(
                div()
                    .w(px(40.0))
                    .flex_shrink_0()
                    .text_align(TextAlign::Right)
                    .pr_2()
                    .text_color(line_number_color)
                    .child(line_number.map(|line| line.to_string()).unwrap_or_default()),
            )
            .child(self.diff_patch_text(row.text, text_color))
            .into_any_element()
    }

    fn diff_patch_text(&self, text: &'static str, default_color: gpui::Hsla) -> impl IntoElement {
        let mut line = div().min_w_0().flex().flex_1().text_color(default_color);
        for (segment, color) in diff_patch_text_segments(text, default_color) {
            line = line.child(div().text_color(color).child(segment));
        }
        line
    }

    fn diff_kind_color(&self, kind: Option<&str>) -> gpui::Hsla {
        match kind {
            Some("added") | Some("new") => diff_success_color(),
            Some("deleted") => diff_destructive_color(),
            Some("modified") | Some("change") | Some("rename-pure") | Some("rename-changed") => {
                diff_modified_color()
            }
            _ => self.theme.muted_foreground.opacity(0.80),
        }
    }

    fn diff_panel_file_stat_label(&self, stat: TurnDiffStat) -> impl IntoElement {
        let mut label = div()
            .flex()
            .items_center()
            .gap_1()
            .flex_shrink_0()
            .font_family(SharedString::from(MONO_FONT_FAMILY))
            .text_size(px(13.0));

        if stat.deletions > 0 {
            label = label.child(
                div()
                    .text_color(diff_destructive_color())
                    .child(format!("-{}", stat.deletions)),
            );
        }
        if stat.additions > 0 {
            label = label.child(
                div()
                    .text_color(diff_success_color())
                    .child(format!("+{}", stat.additions)),
            );
        }
        if stat.additions == 0 && stat.deletions == 0 {
            label = label.child(
                div()
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child("0"),
            );
        }

        label
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

    fn branch_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self
            .snapshot
            .active_branch_toolbar_state()
            .expect("branch toolbar should only render when state exists");
        let toolbar_width = if self.snapshot.diff_open() {
            440.0
        } else {
            832.0
        };

        let mut context_controls = div()
            .flex()
            .items_center()
            .gap_1()
            .min_w_0()
            .flex_shrink_0();

        if state.show_environment_picker {
            let environment_icon = if state.environment_is_primary {
                "icons/monitor.svg"
            } else {
                "icons/cloud.svg"
            };
            context_controls = context_controls
                .child(self.branch_toolbar_context_button(
                    "branch-toolbar-environment",
                    environment_icon,
                    state.environment_label.clone(),
                    state.env_locked,
                    cx,
                ))
                .child(div().mx_0p5().h(px(14.0)).w(px(1.0)).bg(self.theme.border));
        }

        context_controls = context_controls.child(self.branch_toolbar_context_button(
            "branch-toolbar-env-mode",
            branch_toolbar_workspace_icon(
                state.effective_env_mode,
                state.active_worktree_path.as_deref(),
            ),
            state.workspace_label,
            state.env_mode_locked,
            cx,
        ));

        div()
            .id("branch-toolbar")
            .flex()
            .items_center()
            .justify_center()
            .px_8()
            .pb_3()
            .pt_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .w(px(toolbar_width))
                    .px_3()
                    .child(context_controls)
                    .child(div().flex_1())
                    .child(self.branch_toolbar_branch_button(state.branch_label, cx)),
            )
    }

    fn branch_toolbar_context_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        label: impl Into<SharedString>,
        locked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_1()
            .h(px(24.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.background.alpha(0.0))
            .px_2()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(self.theme.muted_foreground.opacity(0.70))
            .when(!locked, |button| button.cursor_pointer())
            .on_click(cx.listener(move |this, _, _, cx| {
                if id == "branch-toolbar-env-mode" && !locked {
                    if let Some(toolbar) = this.snapshot.active_branch_toolbar_state() {
                        this.snapshot
                            .set_active_draft_env_mode(toolbar.effective_env_mode.toggled());
                        cx.notify();
                    }
                }
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_3()
                    .text_color(self.theme.muted_foreground.opacity(0.70)),
            )
            .child(div().min_w_0().child(label.into()))
    }

    fn branch_toolbar_branch_button(
        &self,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("branch-toolbar-branch")
            .flex()
            .items_center()
            .gap_1()
            .h(px(24.0))
            .max_w(px(240.0))
            .rounded(px(6.0))
            .px_2()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(self.theme.muted_foreground.opacity(0.70))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                if let Some(next_ref_name) = this.next_branch_toolbar_ref_name() {
                    this.snapshot.select_branch_for_active_thread(next_ref_name);
                    cx.notify();
                }
            }))
            .child(div().min_w_0().overflow_hidden().child(label.into()))
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_3()
                    .flex_shrink_0()
                    .text_color(self.theme.muted_foreground.opacity(0.50)),
            )
    }

    fn next_branch_toolbar_ref_name(&self) -> Option<String> {
        let toolbar = self.snapshot.active_branch_toolbar_state()?;
        let refs = &self.snapshot.vcs_refs;
        if refs.is_empty() {
            return None;
        }
        let active_index = toolbar
            .resolved_active_branch
            .as_deref()
            .and_then(|branch| refs.iter().position(|ref_name| ref_name.name == branch))
            .unwrap_or(0);
        refs.get((active_index + 1) % refs.len())
            .map(|ref_name| ref_name.name.clone())
    }

    fn project_scripts_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(project) = self.snapshot.active_project() else {
            return div().into_any_element();
        };

        let primary_script = primary_project_script(&project.scripts);
        let mut group = div()
            .id("project-scripts-control")
            .flex()
            .items_center()
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .overflow_hidden();

        if let Some(script) = primary_script {
            group = group
                .child(self.project_script_primary_button(script, cx))
                .child(div().h(px(18.0)).w(px(1.0)).bg(self.theme.border))
                .child(self.project_script_menu_button(cx));
        } else {
            group = group.child(self.project_script_add_button(cx));
        }

        group.into_any_element()
    }

    fn project_script_primary_button(
        &self,
        script: &ProjectScript,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let script_name = script.name.clone();
        let command = script.command.clone();
        div()
            .id("project-script-primary")
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(28.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.project_script_run_count = this.project_script_run_count.saturating_add(1);
                this.composer_prompt = command.clone();
                this.project_script_menu_open = false;
                this.open_in_menu_open = false;
                this.git_actions_menu_open = false;
                cx.notify();
            }))
            .child(
                svg()
                    .path(project_script_icon_path(script.icon))
                    .size_3p5()
                    .text_color(self.theme.foreground),
            )
            .child(div().child(script_name))
    }

    fn project_script_menu_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("project-script-menu")
            .flex()
            .items_center()
            .justify_center()
            .h(px(28.0))
            .w(px(28.0))
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.project_script_menu_open = !this.project_script_menu_open;
                this.open_in_menu_open = false;
                this.git_actions_menu_open = false;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn project_script_add_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("project-script-add")
            .flex()
            .items_center()
            .gap_1p5()
            .h(px(28.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.composer_prompt = "Add action".to_string();
                this.project_script_menu_open = false;
                this.open_in_menu_open = false;
                this.git_actions_menu_open = false;
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/plus.svg")
                    .size_3p5()
                    .text_color(self.theme.foreground),
            )
            .child("Add action")
    }

    fn open_in_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let option = self
            .snapshot
            .active_editor_option("Windows")
            .unwrap_or(EditorOption {
                label: "Explorer",
                id: r3_core::EditorId::FileManager,
            });

        div()
            .id("open-in-picker")
            .flex()
            .items_center()
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .overflow_hidden()
            .child(
                div()
                    .id("open-in-primary")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .h(px(28.0))
                    .px_2()
                    .text_size(px(12.0))
                    .text_color(self.theme.foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.opened_editor_count = this.opened_editor_count.saturating_add(1);
                        this.project_script_menu_open = false;
                        this.open_in_menu_open = false;
                        this.git_actions_menu_open = false;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path(editor_option_icon_path(option))
                            .size_3p5()
                            .text_color(self.theme.foreground),
                    )
                    .child("Open"),
            )
            .child(div().h(px(18.0)).w(px(1.0)).bg(self.theme.border))
            .child(
                div()
                    .id("open-in-menu")
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(28.0))
                    .w(px(28.0))
                    .text_color(self.theme.muted_foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_in_menu_open = !this.open_in_menu_open;
                        this.project_script_menu_open = false;
                        this.git_actions_menu_open = false;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/chevron-down.svg")
                            .size_4()
                            .text_color(self.theme.muted_foreground),
                    ),
            )
    }

    fn git_actions_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("git-actions-control")
            .flex()
            .items_center()
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .overflow_hidden()
            .child(
                div()
                    .id("git-action-primary")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .h(px(28.0))
                    .px_2()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground)
                    .opacity(0.64)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.composer_prompt = "Commit".to_string();
                        this.project_script_menu_open = false;
                        this.open_in_menu_open = false;
                        this.git_actions_menu_open = false;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/git-commit.svg")
                            .size_3p5()
                            .text_color(self.theme.muted_foreground),
                    )
                    .child("Commit"),
            )
            .child(div().h(px(18.0)).w(px(1.0)).bg(self.theme.border))
            .child(
                div()
                    .id("git-action-menu")
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(28.0))
                    .w(px(28.0))
                    .text_color(self.theme.muted_foreground)
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_actions_menu_open = !this.git_actions_menu_open;
                        this.project_script_menu_open = false;
                        this.open_in_menu_open = false;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("icons/chevron-down.svg")
                            .size_4()
                            .text_color(self.theme.muted_foreground),
                    ),
            )
    }

    fn project_scripts_menu_popup(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(project) = self.snapshot.active_project() else {
            return div().into_any_element();
        };

        let mut popup = div()
            .id("project-script-menu-popup")
            .absolute()
            .top(px(42.0))
            .right(px(206.0))
            .flex()
            .flex_col()
            .w(px(126.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .text_color(self.theme.foreground)
            .shadow(self.header_menu_shadow());

        for script in &project.scripts {
            popup = popup.child(self.project_scripts_menu_item(script, cx));
        }

        popup
            .child(
                self.header_menu_row(
                    "project-script-menu-add".to_string(),
                    "icons/plus.svg",
                    "Add action".to_string(),
                    None,
                    None,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.composer_prompt = "Add action".to_string();
                    this.project_script_menu_open = false;
                    this.open_in_menu_open = false;
                    this.git_actions_menu_open = false;
                    cx.notify();
                })),
            )
            .into_any_element()
    }

    fn project_scripts_menu_item(
        &self,
        script: &ProjectScript,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let icon = project_script_icon_path(script.icon);
        let label = if script.run_on_worktree_create {
            format!("{} (setup)", script.name)
        } else {
            script.name.clone()
        };
        let command = script.command.clone();

        self.header_menu_row(
            format!("project-script-menu-item-{}", script.id),
            icon,
            label,
            Some("icons/settings-2.svg"),
            None,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.project_script_run_count = this.project_script_run_count.saturating_add(1);
            this.composer_prompt = command.clone();
            this.project_script_menu_open = false;
            this.open_in_menu_open = false;
            this.git_actions_menu_open = false;
            cx.notify();
        }))
    }

    fn git_actions_menu_popup(&self, cx: &mut Context<Self>) -> AnyElement {
        let status = self.reference_git_status_snapshot();
        let mut popup = div()
            .id("git-actions-menu-popup")
            .absolute()
            .top(px(40.0))
            .right(px(92.0))
            .flex()
            .flex_col()
            .w(px(522.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .text_color(self.theme.foreground)
            .shadow(self.header_menu_shadow());

        for item in build_git_action_menu_items(Some(&status), false, true) {
            popup = popup.child(self.git_action_menu_item(&item, cx));
        }

        if status.ref_name.is_none() {
            popup = popup.child(
                div()
                    .px_2()
                    .py_1p5()
                    .text_size(px(12.0))
                    .text_color(git_action_warning_color())
                    .child(
                        "Detached HEAD: create and checkout a refName to enable push and pull request actions.",
                    ),
            );
        }

        popup.into_any_element()
    }

    fn git_action_menu_item(
        &self,
        item: &GitActionMenuItem,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let label = item.label.clone();
        let row = self
            .header_menu_row(
                format!(
                    "git-actions-menu-item-{}",
                    label.to_ascii_lowercase().replace(' ', "-")
                ),
                git_action_icon_path(item.icon),
                label.clone(),
                None,
                None,
            )
            .h(px(28.0))
            .rounded(px(2.0))
            .text_size(px(14.0))
            .opacity(if item.disabled { 0.64 } else { 1.0 });

        if item.disabled {
            row
        } else {
            row.on_click(cx.listener(move |this, _, _, cx| {
                this.composer_prompt = label.clone();
                this.project_script_menu_open = false;
                this.open_in_menu_open = false;
                this.git_actions_menu_open = false;
                cx.notify();
            }))
        }
    }

    fn reference_git_status_snapshot(&self) -> GitStatusSnapshot {
        GitStatusSnapshot {
            // The pinned upstream seed renders GitActionsControl with detached VCS status
            // while the branch toolbar still resolves "From main".
            ref_name: None,
            has_working_tree_changes: false,
            has_upstream: true,
            ahead_count: 0,
            behind_count: 0,
            ahead_of_default_count: Some(0),
            has_open_pr: false,
        }
    }

    fn open_in_menu_popup(&self, cx: &mut Context<Self>) -> AnyElement {
        let options = resolve_editor_options("Windows", &self.snapshot.available_editors);
        let preferred_editor = self.snapshot.preferred_editor;
        let mut popup = div()
            .id("open-in-menu-popup")
            .absolute()
            .top(px(42.0))
            .right(px(206.0))
            .flex()
            .flex_col()
            .w(px(154.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .text_color(self.theme.foreground)
            .shadow(self.header_menu_shadow());

        if options.is_empty() {
            popup = popup.child(
                div()
                    .flex()
                    .items_center()
                    .h(px(32.0))
                    .rounded(px(6.0))
                    .px_2()
                    .text_size(px(13.0))
                    .text_color(self.theme.muted_foreground)
                    .child("No installed editors found"),
            );
        } else {
            for option in options {
                let label = option.label.to_string();
                popup = popup.child(
                    self.header_menu_row(
                        format!(
                            "open-in-menu-item-{}",
                            label.to_ascii_lowercase().replace(' ', "-")
                        ),
                        editor_option_icon_path(option),
                        label.clone(),
                        None,
                        if Some(option.id) == preferred_editor {
                            Some("Ctrl+O")
                        } else {
                            None
                        },
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.opened_editor_count = this.opened_editor_count.saturating_add(1);
                        this.composer_prompt = format!("Open in {label}");
                        this.project_script_menu_open = false;
                        this.open_in_menu_open = false;
                        this.git_actions_menu_open = false;
                        cx.notify();
                    })),
                );
            }
        }

        popup.into_any_element()
    }

    fn header_menu_row(
        &self,
        id: String,
        icon: &'static str,
        label: String,
        trailing_icon: Option<&'static str>,
        trailing_text: Option<&'static str>,
    ) -> gpui::Stateful<gpui::Div> {
        let mut row = div()
            .id(SharedString::from(id))
            .flex()
            .items_center()
            .gap_2()
            .h(px(32.0))
            .rounded(px(6.0))
            .px_2()
            .text_size(px(13.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .child(
                svg()
                    .path(icon)
                    .size_4()
                    .text_color(self.theme.muted_foreground),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(label),
            );

        row = if let Some(text) = trailing_text {
            row.child(
                div()
                    .ml_auto()
                    .h(px(24.0))
                    .min_w(px(44.0))
                    .text_align(TextAlign::Right)
                    .text_size(px(12.0))
                    .font_weight(FontWeight(500.0))
                    .text_color(self.theme.muted_foreground.opacity(0.72))
                    .child(text),
            )
        } else if let Some(icon) = trailing_icon {
            row.child(
                div()
                    .ml_auto()
                    .flex()
                    .h(px(24.0))
                    .min_w(px(24.0))
                    .items_center()
                    .justify_end()
                    .opacity(0.0)
                    .child(
                        svg()
                            .path(icon)
                            .size_3p5()
                            .text_color(self.theme.muted_foreground),
                    ),
            )
        } else {
            row
        };

        row
    }

    fn header_menu_shadow(&self) -> Vec<BoxShadow> {
        vec![BoxShadow {
            color: hsla(0.0, 0.0, 0.0, 0.05),
            offset: point(px(0.0), px(10.0)),
            blur_radius: px(15.0),
            spread_radius: px(-3.0),
        }]
    }

    fn composer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active_pending_approval = self.snapshot.active_pending_approval();
        let active_pending_user_input_progress = self.snapshot.active_pending_user_input_progress();
        let composer_width = if self.snapshot.diff_open() {
            548.0
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
            .flex_col()
            .items_center()
            .justify_center()
            .px_8()
            .pb_1()
            .when(self.should_show_composer_command_menu(), |container| {
                container.child(self.composer_command_menu(cx))
            })
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
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .px_5()
                    .py_4()
                    .child(self.composer_pending_approval_label())
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

    fn composer_pending_approval_label(&self) -> impl IntoElement {
        let mut label = div()
            .flex()
            .items_center()
            .gap(px(2.8))
            .whitespace_nowrap()
            .text_size(px(14.0))
            .text_color(self.theme.foreground);

        for character in "PENDING APPROVAL".chars() {
            label = if character == ' ' {
                label.child(div().w(px(1.2)).h(px(1.0)).flex_shrink_0())
            } else {
                label.child(div().child(character.to_string()))
            };
        }

        label
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

    fn active_composer_trigger(&self) -> Option<ComposerTrigger> {
        if self.snapshot.active_pending_approval().is_some() {
            return None;
        }
        detect_composer_trigger(
            &self.composer_prompt,
            self.composer_prompt.chars().count() as f64,
        )
    }

    fn composer_workspace_entries(&self) -> Vec<ProjectEntry> {
        vec![
            ProjectEntry {
                path: "src/main.rs".to_string(),
                kind: ProjectEntryKind::File,
                parent_path: Some("src".to_string()),
            },
            ProjectEntry {
                path: "crates/r3_ui/src/shell.rs".to_string(),
                kind: ProjectEntryKind::File,
                parent_path: Some("crates/r3_ui/src".to_string()),
            },
            ProjectEntry {
                path: "docs/reference".to_string(),
                kind: ProjectEntryKind::Directory,
                parent_path: Some("docs".to_string()),
            },
        ]
    }

    fn composer_provider_slash_commands(&self) -> Vec<ServerProviderSlashCommand> {
        Vec::new()
    }

    fn composer_provider_skills(&self) -> Vec<ServerProviderSkill> {
        Vec::new()
    }

    fn composer_menu_items(&self) -> Vec<ComposerCommandItem> {
        let trigger = self.active_composer_trigger();
        build_composer_menu_items(
            trigger.as_ref(),
            &self.composer_workspace_entries(),
            "claudeAgent",
            &self.composer_provider_slash_commands(),
            &self.composer_provider_skills(),
        )
    }

    fn should_show_composer_command_menu(&self) -> bool {
        self.active_composer_trigger().is_some()
            && !self.composer_menu_items().is_empty()
            && self.snapshot.active_pending_user_input_progress().is_none()
    }

    fn active_composer_menu_item_id(&self, items: &[ComposerCommandItem]) -> Option<String> {
        resolve_composer_menu_active_item_id(
            items,
            self.composer_highlighted_item_id.as_deref(),
            composer_menu_search_key(self.active_composer_trigger().as_ref()).as_deref(),
            self.composer_highlighted_search_key.as_deref(),
        )
    }

    fn composer_command_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.composer_menu_items();
        let active_item_id = self.active_composer_menu_item_id(&items);
        let trigger_kind = self.active_composer_trigger().map(|trigger| trigger.kind);
        let groups = group_composer_command_items(&items, trigger_kind, true);
        let mut menu = div()
            .id("chat-composer-command-menu")
            .w(px(if self.snapshot.diff_open() {
                440.0
            } else {
                832.0
            }))
            .mb_2()
            .overflow_hidden()
            .rounded(px(12.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.80))
            .bg(self.theme.card.opacity(0.96))
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.08),
                offset: point(px(0.0), px(10.0)),
                blur_radius: px(18.0),
                spread_radius: px(-6.0),
            }]);

        for (group_index, group) in groups.into_iter().enumerate() {
            let mut section = div().flex().flex_col();
            if group_index > 0 {
                section = section.child(
                    div()
                        .mx_2()
                        .my_0p5()
                        .h(px(1.0))
                        .bg(self.theme.border.opacity(0.70)),
                );
            }
            if let Some(label) = group.label {
                section = section.child(
                    div()
                        .px_3()
                        .pt_2()
                        .pb_1()
                        .text_size(px(10.0))
                        .font_weight(FontWeight(650.0))
                        .text_color(self.theme.muted_foreground.opacity(0.55))
                        .child(label.to_ascii_uppercase()),
                );
            }
            for item in group.items {
                let active = active_item_id
                    .as_deref()
                    .map(|id| id == item.id())
                    .unwrap_or(false);
                section = section.child(self.composer_command_menu_item(item, active, cx));
            }
            menu = menu.child(section);
        }

        menu
    }

    fn composer_command_menu_item(
        &self,
        item: ComposerCommandItem,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (id, label, description, icon_path) = match &item {
            ComposerCommandItem::Path {
                id,
                label,
                description,
                path_kind,
                ..
            } => (
                id.clone(),
                label.clone(),
                description.clone(),
                match path_kind {
                    ProjectEntryKind::File => "icons/file-json.svg",
                    ProjectEntryKind::Directory => "icons/folder.svg",
                },
            ),
            ComposerCommandItem::SlashCommand {
                id,
                label,
                description,
                ..
            } => (
                id.clone(),
                label.clone(),
                description.clone(),
                "icons/bot.svg",
            ),
            ComposerCommandItem::ProviderSlashCommand {
                id,
                label,
                description,
                ..
            }
            | ComposerCommandItem::Skill {
                id,
                label,
                description,
                ..
            } => (
                id.clone(),
                label.clone(),
                description.clone(),
                "icons/sparkles.svg",
            ),
        };
        let item_for_click = item.clone();
        div()
            .id(SharedString::from(format!(
                "composer-command-menu-item-{id}"
            )))
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(if active {
                self.theme.accent
            } else {
                self.theme.background.alpha(0.0)
            })
            .text_color(if active {
                self.theme.foreground
            } else {
                self.theme.foreground.opacity(0.90)
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.apply_composer_command_item(&item_for_click, cx);
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_4()
                    .flex_shrink_0()
                    .text_color(self.theme.muted_foreground.opacity(0.82)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .flex_1()
                    .child(div().flex_shrink_0().text_size(px(14.0)).child(label))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(12.0))
                            .text_color(self.theme.muted_foreground.opacity(0.70))
                            .child(description),
                    ),
            )
    }

    fn composer_path_basename<'a>(&self, path: &'a str) -> &'a str {
        path.rsplit_once('/')
            .map(|(_, basename)| basename)
            .unwrap_or(path)
    }

    fn composer_inline_chip(&self, label: String, icon_path: &'static str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .max_w_full()
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.border.opacity(0.70))
            .bg(self.theme.accent.opacity(0.40))
            .px_1p5()
            .py_0p5()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(self.theme.foreground)
            .child(
                svg()
                    .path(icon_path)
                    .size_3p5()
                    .flex_shrink_0()
                    .text_color(self.theme.muted_foreground.opacity(0.85)),
            )
            .child(div().overflow_hidden().text_ellipsis().child(label))
    }

    fn composer_skill_chip(&self, label: String) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .max_w_full()
            .rounded(px(6.0))
            .border_1()
            .border_color(hsla(322.0 / 360.0, 0.72, 0.50, 0.25))
            .bg(hsla(322.0 / 360.0, 0.72, 0.50, 0.12))
            .px_1p5()
            .py_0p5()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(hsla(322.0 / 360.0, 0.72, 0.36, 1.0))
            .child(
                svg()
                    .path("icons/skill-chip.svg")
                    .size_3p5()
                    .flex_shrink_0()
                    .text_color(hsla(322.0 / 360.0, 0.72, 0.36, 0.85)),
            )
            .child(div().overflow_hidden().text_ellipsis().child(label))
    }

    fn composer_prompt_content(&self, text: &str, tokenize: bool) -> AnyElement {
        if !tokenize {
            return div().child(text.to_string()).into_any_element();
        }

        let segments = split_prompt_into_composer_segments(text);
        if segments
            .iter()
            .all(|segment| matches!(segment, ComposerPromptSegment::Text { .. }))
        {
            return div().child(text.to_string()).into_any_element();
        }

        let mut content = div().flex().items_center().gap_1().min_w_0();
        for segment in segments {
            match segment {
                ComposerPromptSegment::Text { text } => {
                    if !text.is_empty() {
                        content = content.child(div().child(text));
                    }
                }
                ComposerPromptSegment::Mention { path } => {
                    let label = self.composer_path_basename(&path).to_string();
                    content = content.child(
                        self.composer_inline_chip(label, "icons/file-type-light-agents.svg"),
                    );
                }
                ComposerPromptSegment::Skill { name } => {
                    let skill = ServerProviderSkill {
                        name,
                        description: None,
                        path: String::new(),
                        scope: None,
                        enabled: true,
                        display_name: None,
                        short_description: None,
                    };
                    content = content.child(
                        self.composer_skill_chip(format_provider_skill_display_name(&skill)),
                    );
                }
                ComposerPromptSegment::TerminalContext { .. } => {
                    content = content.child(self.composer_inline_chip(
                        "Terminal context".to_string(),
                        "icons/terminal.svg",
                    ));
                }
            }
        }
        content.into_any_element()
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
        let tokenize_prompt = !placeholder
            && active_pending_approval.is_none()
            && active_pending_user_input_progress.is_none();
        let input_min_height = if active_pending_approval.is_some() {
            79.0
        } else if active_pending_user_input_progress.is_some() {
            85.0
        } else {
            96.0
        };
        let input_top_padding =
            if active_pending_approval.is_some() || active_pending_user_input_progress.is_some() {
                12.0
            } else {
                16.0
            };
        let input_bottom_padding =
            if active_pending_approval.is_some() || active_pending_user_input_progress.is_some() {
                8.0
            } else {
                12.0
            };

        div()
            .id("chat-composer-input")
            .relative()
            .track_focus(&self.composer_focus_handle)
            .key_context("ChatComposer")
            .on_key_down(cx.listener(Self::on_composer_key_down))
            .tab_index(0)
            .cursor(CursorStyle::IBeam)
            .min_h(px(input_min_height))
            .px_4()
            .pt(px(input_top_padding))
            .pb(px(input_bottom_padding))
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
                    .child(self.composer_prompt_content(&text, tokenize_prompt)),
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

        if compact {
            return div()
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
                        .gap_1p5()
                        .min_w_0()
                        .child(self.composer_model_picker(cx))
                        .child(self.composer_compact_controls_button(cx)),
                )
                .child(self.composer_send_button(cx))
                .into_any_element();
        }

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

    fn composer_compact_controls_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("chat-composer-compact-controls")
            .flex()
            .items_center()
            .justify_center()
            .h(px(32.0))
            .rounded(px(8.0))
            .px_2()
            .text_color(self.theme.muted_foreground.opacity(0.70))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.composer_compact_controls_menu_open =
                    !this.composer_compact_controls_menu_open;
                if this.composer_compact_controls_menu_open {
                    this.model_picker_open = false;
                    this.project_script_menu_open = false;
                    this.open_in_menu_open = false;
                    this.git_actions_menu_open = false;
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/ellipsis.svg")
                    .size_4()
                    .text_color(self.theme.muted_foreground.opacity(0.70)),
            )
    }

    fn composer_compact_controls_menu_popup(&self, cx: &mut Context<Self>) -> AnyElement {
        let show_interaction_mode_toggle = self
            .snapshot
            .providers
            .iter()
            .find(|provider| provider.instance_id == self.snapshot.selected_provider_instance_id)
            .map(|provider| provider.show_interaction_mode_toggle)
            .unwrap_or(false);

        let mut popup = div()
            .id("chat-composer-compact-controls-menu")
            .absolute()
            .bottom(px(88.0))
            .right(px(332.0))
            .flex()
            .flex_col()
            .w(px(208.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .text_color(self.theme.foreground)
            .shadow(self.header_menu_shadow());

        if show_interaction_mode_toggle {
            popup = popup
                .child(self.composer_compact_menu_heading("Mode"))
                .child(self.composer_compact_mode_item("Chat", !self.composer_plan_mode, cx))
                .child(self.composer_compact_mode_item("Plan", self.composer_plan_mode, cx))
                .child(self.menu_divider());
        }

        popup = popup.child(self.composer_compact_menu_heading("Access"));
        for (index, mode) in COMPOSER_RUNTIME_MODES.iter().enumerate() {
            popup = popup.child(self.composer_compact_access_item(index, *mode, cx));
        }

        popup.into_any_element()
    }

    fn composer_compact_menu_heading(&self, label: &'static str) -> impl IntoElement {
        div()
            .px_2()
            .py_1p5()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn menu_divider(&self) -> impl IntoElement {
        div().my_1().h(px(1.0)).bg(self.theme.border.opacity(0.72))
    }

    fn composer_compact_mode_item(
        &self,
        label: &'static str,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let plan_mode = label == "Plan";
        self.composer_compact_radio_item_base(
            format!("chat-composer-compact-mode-{}", label.to_ascii_lowercase()),
            label,
            selected,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            if this.composer_plan_mode != plan_mode {
                this.composer_plan_mode = plan_mode;
            }
            this.composer_compact_controls_menu_open = false;
            cx.notify();
        }))
    }

    fn composer_compact_access_item(
        &self,
        index: usize,
        mode: ComposerRuntimeMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.composer_compact_radio_item_base(
            format!(
                "chat-composer-compact-access-{}",
                mode.label.to_ascii_lowercase().replace(' ', "-")
            ),
            mode.label,
            self.composer_runtime_index % COMPOSER_RUNTIME_MODES.len() == index,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.composer_runtime_index = index;
            this.composer_compact_controls_menu_open = false;
            cx.notify();
        }))
    }

    fn composer_compact_radio_item_base(
        &self,
        id: String,
        label: &'static str,
        selected: bool,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(SharedString::from(id))
            .flex()
            .items_center()
            .gap_2()
            .h(px(30.0))
            .rounded(px(6.0))
            .px_2()
            .text_size(px(13.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(16.0))
                    .h(px(16.0))
                    .child(if selected {
                        svg()
                            .path("icons/check.svg")
                            .size_3p5()
                            .text_color(self.theme.foreground)
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    }),
            )
            .child(label)
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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w_0()
                    .child(self.composer_model_picker(cx))
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
                        self.runtime_mode().label,
                        cx,
                    )),
            )
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
        let state = self.current_model_picker_state();
        let active_entry = state.active_entry.as_ref();

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
                this.model_picker_open = !this.model_picker_open;
                this.composer_compact_controls_menu_open = false;
                if this.model_picker_open {
                    this.model_picker_query.clear();
                }
                cx.notify();
            }))
            .when_some(active_entry, |picker, entry| {
                picker.child(self.model_provider_instance_icon(
                    entry,
                    state.show_instance_badge,
                    px(20.0),
                    px(16.0),
                ))
            })
            .child(div().flex().items_center().gap_1p5().min_w_0().child(
                if let Some(subtitle) = state.trigger_subtitle.as_deref() {
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .min_w_0()
                        .child(
                            div()
                                .max_w(px(82.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(subtitle.to_string()),
                        )
                        .child(
                            div()
                                .text_color(self.theme.muted_foreground.opacity(0.60))
                                .child("·"),
                        )
                        .child(
                            div()
                                .max_w(px(82.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .text_color(self.theme.foreground)
                                .font_weight(FontWeight(550.0))
                                .child(state.trigger_title.clone()),
                        )
                } else {
                    div()
                        .max_w(px(142.0))
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_color(self.theme.foreground)
                        .font_weight(FontWeight(550.0))
                        .child(state.trigger_title.clone())
                },
            ))
            .child(
                svg()
                    .path("icons/chevron-down.svg")
                    .size_3()
                    .text_color(self.theme.muted_foreground.opacity(0.72)),
            )
    }

    fn current_model_picker_state(&self) -> ModelPickerState {
        resolve_model_picker_state(
            &self.snapshot,
            &self.model_picker_query,
            Some(self.model_picker_selected_instance.clone()),
            None,
            None,
        )
    }

    fn model_picker_popup(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.current_model_picker_state();
        let mut popup = div()
            .id("model-picker-content")
            .absolute()
            .left(px(SIDEBAR_MIN_WIDTH + 24.0))
            .bottom(px(76.0))
            .flex()
            .w(px(400.0))
            .h(px(384.0))
            .overflow_hidden()
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .shadow(vec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.05),
                offset: point(px(0.0), px(10.0)),
                blur_radius: px(15.0),
                spread_radius: px(-3.0),
            }]);

        if state.show_sidebar {
            popup = popup.child(self.model_picker_sidebar(&state, cx));
        }

        popup.child(self.model_picker_content(&state, cx))
    }

    fn model_picker_sidebar(
        &self,
        state: &ModelPickerState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut rail = div()
            .id("model-picker-sidebar")
            .flex()
            .flex_col()
            .gap_1()
            .w(px(48.0))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(self.theme.border)
            .bg(self.theme.accent.opacity(0.30))
            .p_1();

        if !state.is_locked {
            rail = rail.child(self.model_picker_favorites_button(state, cx));
        }

        for entry in &state.sidebar_entries {
            rail = rail.child(self.model_picker_instance_button(entry, state, cx));
        }

        if !state.is_locked {
            rail = rail
                .child(self.model_picker_coming_soon_button("Gemini", "G", cx))
                .child(self.model_picker_coming_soon_button("Github Copilot", "GH", cx));
        }

        rail
    }

    fn model_picker_favorites_button(
        &self,
        state: &ModelPickerState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = state.selected_instance == ModelPickerSelectedInstance::Favorites;
        div()
            .relative()
            .w_full()
            .pb_1()
            .mb_1()
            .border_b_1()
            .border_color(self.theme.border)
            .child(
                self.model_picker_rail_button(
                    "model-picker-favorites",
                    selected,
                    false,
                    cx,
                    |this, cx| {
                        this.model_picker_selected_instance =
                            ModelPickerSelectedInstance::Favorites;
                        this.model_picker_query.clear();
                        cx.notify();
                    },
                    div().child(
                        svg()
                            .path("icons/star.svg")
                            .size_5()
                            .text_color(self.theme.foreground),
                    ),
                ),
            )
    }

    fn model_picker_instance_button(
        &self,
        entry: &ProviderInstanceEntry,
        state: &ModelPickerState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let instance_id = entry.instance_id.clone();
        let selected =
            state.selected_instance == ModelPickerSelectedInstance::Instance(instance_id.clone());
        let disabled = !entry.is_available || entry.status != r3_core::ServerProviderState::Ready;
        let duplicate_driver_count = state
            .sidebar_entries
            .iter()
            .filter(|candidate| candidate.driver_kind == entry.driver_kind)
            .count();
        let show_badge = entry.accent_color.is_some() || duplicate_driver_count > 1;

        self.model_picker_rail_button(
            "model-picker-provider",
            selected,
            disabled,
            cx,
            move |this, cx| {
                if !disabled {
                    this.model_picker_selected_instance =
                        ModelPickerSelectedInstance::Instance(instance_id.clone());
                    this.model_picker_query.clear();
                    cx.notify();
                }
            },
            self.model_provider_instance_icon(entry, show_badge, px(24.0), px(20.0)),
        )
    }

    fn model_picker_coming_soon_button(
        &self,
        label: &'static str,
        initials: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.model_picker_rail_button(
            "model-picker-coming-soon",
            false,
            true,
            cx,
            |_, _| {},
            div()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .w(px(24.0))
                .h(px(24.0))
                .text_size(px(9.0))
                .font_weight(FontWeight(650.0))
                .text_color(self.theme.muted_foreground.opacity(0.80))
                .child(initials)
                .child(
                    div()
                        .absolute()
                        .top(px(0.0))
                        .right(px(-2.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(14.0))
                        .h(px(14.0))
                        .rounded(px(7.0))
                        .bg(self.theme.background)
                        .child(
                            svg()
                                .path("icons/clock-3.svg")
                                .size(px(8.0))
                                .text_color(self.theme.muted_foreground),
                        ),
                )
                .child(div().w(px(0.0)).h(px(0.0)).child(label)),
        )
    }

    fn model_picker_rail_button<F>(
        &self,
        id: &'static str,
        selected: bool,
        disabled: bool,
        cx: &mut Context<Self>,
        on_click: F,
        child: impl IntoElement,
    ) -> impl IntoElement
    where
        F: Fn(&mut R3Shell, &mut Context<Self>) + 'static,
    {
        div()
            .id(id)
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .h(px(40.0))
            .rounded(px(4.0))
            .bg(if selected {
                self.theme.background
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            })
            .text_color(if disabled {
                self.theme.muted_foreground.opacity(0.45)
            } else {
                self.theme.foreground
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                on_click(this, cx);
            }))
            .when(selected, |button| {
                button.child(
                    div()
                        .absolute()
                        .right(px(-4.0))
                        .top(px(10.0))
                        .w(px(2.0))
                        .h(px(20.0))
                        .rounded(px(2.0))
                        .bg(self.theme.primary),
                )
            })
            .child(child)
    }

    fn model_picker_content(
        &self,
        state: &ModelPickerState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut list = div()
            .id("model-picker-list")
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .bg(self.theme.accent.opacity(0.22))
            .px_2()
            .py_1();

        for (index, model) in state.filtered_models.iter().enumerate() {
            list = list.child(self.model_picker_row(index, model, state, cx));
        }

        div()
            .id("model-picker-main")
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .id("model-picker-search")
                    .flex()
                    .items_center()
                    .gap_2()
                    .border_b_1()
                    .border_color(self.theme.border)
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .w_full()
                            .h(px(32.0))
                            .rounded(px(6.0))
                            .bg(self.theme.background)
                            .px_2()
                            .child(
                                svg()
                                    .path("icons/search.svg")
                                    .size_4()
                                    .text_color(self.theme.muted_foreground.opacity(0.50)),
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(if self.model_picker_query.is_empty() {
                                        self.theme.muted_foreground.opacity(0.55)
                                    } else {
                                        self.theme.foreground
                                    })
                                    .child(if self.model_picker_query.is_empty() {
                                        "Search models...".to_string()
                                    } else {
                                        self.model_picker_query.clone()
                                    }),
                            ),
                    ),
            )
            .child(list)
            .when(state.filtered_models.is_empty(), |content| {
                content.child(
                    div()
                        .absolute()
                        .bottom(px(18.0))
                        .left(px(72.0))
                        .text_size(px(12.0))
                        .text_color(self.theme.muted_foreground)
                        .child("No models found"),
                )
            })
    }

    fn model_picker_row(
        &self,
        index: usize,
        model: &ModelPickerItem,
        state: &ModelPickerState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.snapshot.selected_provider_instance_id == model.instance_id
            && self.snapshot.selected_model == model.slug;
        let show_provider = !state.is_locked || state.show_locked_instance_sidebar;
        let instance_id = model.instance_id.clone();
        let slug = model.slug.clone();

        div()
            .id("model-picker-row")
            .flex()
            .items_start()
            .gap_2()
            .w_full()
            .rounded(px(4.0))
            .px_3()
            .py_2()
            .text_size(px(12.0))
            .bg(if selected {
                self.theme.accent
            } else if index == 0 {
                self.theme.accent.opacity(0.60)
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_provider_model(&instance_id, &slug);
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/star.svg")
                    .size_4()
                    .text_color(if model.is_favorite {
                        hsla(0.12, 0.92, 0.48, 1.0)
                    } else {
                        self.theme.muted_foreground.opacity(0.40)
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .min_w_0()
                    .flex_1()
                    .child(
                        div().flex().items_center().justify_between().gap_2().child(
                            div()
                                .min_w_0()
                                .overflow_hidden()
                                .text_ellipsis()
                                .font_weight(FontWeight(500.0))
                                .child(get_display_model_name(
                                    &ServerProviderModel {
                                        slug: model.slug.clone(),
                                        name: model.name.clone(),
                                        short_name: model.short_name.clone(),
                                        sub_provider: model.sub_provider.clone(),
                                        is_custom: false,
                                    },
                                    !state.is_locked,
                                )),
                        ),
                    )
                    .when(show_provider, |row| {
                        row.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_size(px(12.0))
                                .text_color(self.theme.muted_foreground.opacity(0.70))
                                .child(
                                    svg()
                                        .path(provider_driver_icon_path(&model.driver_kind))
                                        .size(px(12.0))
                                        .text_color(self.theme.muted_foreground.opacity(0.80)),
                                )
                                .when_some(model.instance_accent_color.as_deref(), |line, color| {
                                    line.child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(
                                        provider_accent_color(color).unwrap_or(self.theme.primary),
                                    ))
                                })
                                .child(div().min_w_0().overflow_hidden().text_ellipsis().child(
                                    if let Some(sub_provider) = model.sub_provider.as_deref() {
                                        format!(
                                            "{} · {}",
                                            model.instance_display_name, sub_provider
                                        )
                                    } else {
                                        model.instance_display_name.clone()
                                    },
                                )),
                        )
                    }),
            )
    }

    fn model_provider_instance_icon(
        &self,
        entry: &ProviderInstanceEntry,
        show_badge: bool,
        container_size: Pixels,
        icon_size: Pixels,
    ) -> impl IntoElement {
        let initials = provider_instance_initials(&entry.display_name);
        let badge_initials = initials.clone();
        let icon_path = provider_driver_icon_path(&entry.driver_kind);
        div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .w(container_size)
            .h(container_size)
            .flex_shrink_0()
            .text_color(self.theme.foreground)
            .child(if icon_path.is_empty() {
                div()
                    .text_size(px(10.0))
                    .font_weight(FontWeight(650.0))
                    .child(initials)
                    .into_any_element()
            } else {
                svg()
                    .path(icon_path)
                    .size(icon_size)
                    .text_color(provider_icon_color(
                        &entry.driver_kind,
                        self.theme.foreground,
                    ))
                    .into_any_element()
            })
            .when(show_badge, |icon| {
                icon.child(
                    div()
                        .absolute()
                        .right(px(-2.0))
                        .bottom(px(-2.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .min_w(px(12.0))
                        .h(px(12.0))
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(self.theme.background)
                        .bg(entry
                            .accent_color
                            .as_deref()
                            .and_then(provider_accent_color)
                            .unwrap_or(self.theme.accent))
                        .px_0p5()
                        .text_size(px(7.0))
                        .font_weight(FontWeight(650.0))
                        .text_color(if entry.accent_color.is_some() {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        } else {
                            self.theme.muted_foreground
                        })
                        .child(badge_initials),
                )
            })
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
            let active = self.screen == R3Screen::Settings && self.settings_section == item.section;
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

        if self.screen == R3Screen::Settings && self.settings_section == SettingsSection::General {
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
            .child(if self.screen == R3Screen::SettingsDiagnostics {
                self.settings_diagnostics_panel().into_any_element()
            } else {
                match self.settings_section {
                    SettingsSection::General => self.settings_general_panel(cx).into_any_element(),
                    SettingsSection::Keybindings => {
                        self.settings_keybindings_panel(cx).into_any_element()
                    }
                    SettingsSection::Providers => {
                        self.settings_providers_panel(cx).into_any_element()
                    }
                    SettingsSection::SourceControl => {
                        self.settings_source_control_panel(cx).into_any_element()
                    }
                    SettingsSection::Connections => {
                        self.settings_connections_panel(cx).into_any_element()
                    }
                    SettingsSection::Archive => self.settings_archive_panel().into_any_element(),
                }
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

    fn settings_diagnostics_panel(&self) -> impl IntoElement {
        let process_diagnostics = reference_process_diagnostics_result();
        let trace_diagnostics = reference_trace_diagnostics_result();

        div()
            .id("settings-diagnostics-scroll")
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
                    .gap_7()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("LIVE PROCESSES"))
                            .child(self.diagnostics_stats_card(vec![
                                (
                                    "Child Processes",
                                    format_diagnostics_count(
                                        process_diagnostics.process_count as u64,
                                    ),
                                ),
                                (
                                    "CPU",
                                    format!("{:.1}%", process_diagnostics.total_cpu_percent),
                                ),
                                (
                                    "Memory",
                                    format_diagnostics_bytes(process_diagnostics.total_rss_bytes),
                                ),
                                ("Server PID", process_diagnostics.server_pid.to_string()),
                            ]))
                            .child(self.diagnostics_process_card(&process_diagnostics.processes)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("TRACE DIAGNOSTICS"))
                            .child(self.diagnostics_stats_card(vec![
                                (
                                    "Spans",
                                    format_diagnostics_count(trace_diagnostics.record_count),
                                ),
                                (
                                    "Failures",
                                    format_diagnostics_count(trace_diagnostics.failure_count),
                                ),
                                (
                                    "Slow Spans",
                                    format_diagnostics_count(trace_diagnostics.slow_span_count),
                                ),
                                (
                                    "Parse Errors",
                                    format_diagnostics_count(trace_diagnostics.parse_error_count),
                                ),
                            ])),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("LATEST FAILURES"))
                            .child(
                                self.diagnostics_failures_card(&trace_diagnostics.latest_failures),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("MOST COMMON FAILURES"))
                            .child(self.diagnostics_common_failures_card(
                                &trace_diagnostics.common_failures,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("SLOWEST SPANS"))
                            .child(
                                self.diagnostics_slowest_spans_card(
                                    &trace_diagnostics.slowest_spans,
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("SPAN LOGS"))
                            .child(self.diagnostics_logs_card(
                                &trace_diagnostics.latest_warning_and_error_logs,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .child(self.settings_section_header("TOP SPAN NAMES"))
                            .child(
                                self.diagnostics_top_spans_card(
                                    &trace_diagnostics.top_spans_by_count,
                                ),
                            ),
                    ),
            )
    }

    fn diagnostics_stats_card(&self, stats: Vec<(&'static str, String)>) -> gpui::Div {
        let mut grid = div().grid().grid_cols(4);
        for (label, value) in stats {
            grid = grid.child(
                div()
                    .min_w_0()
                    .border_l_1()
                    .border_color(self.theme.border)
                    .px_4()
                    .py_3()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight(650.0))
                            .text_color(self.theme.muted_foreground.opacity(0.72))
                            .child(label),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_size(px(18.0))
                            .font_weight(FontWeight(650.0))
                            .font_family(SharedString::from(MONO_FONT_FAMILY))
                            .child(value),
                    ),
            );
        }
        self.settings_about_card().child(grid)
    }

    fn diagnostics_process_card(&self, processes: &[ProcessDiagnosticsEntry]) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, process) in processes.iter().enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                process.command.clone(),
                format!(
                    "PID {} · {} · depth {}",
                    process.pid, process.status, process.depth
                ),
                format!(
                    "{} · {:.1}%",
                    format_diagnostics_bytes(process.rss_bytes),
                    process.cpu_percent
                ),
            ));
        }
        card
    }

    fn diagnostics_failures_card(&self, failures: &[TraceDiagnosticsRecentFailure]) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, failure) in failures.iter().take(4).enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                failure.name.clone(),
                failure.cause.clone(),
                format_diagnostics_duration_ms(failure.duration_ms),
            ));
        }
        card
    }

    fn diagnostics_common_failures_card(
        &self,
        failures: &[TraceDiagnosticsFailureSummary],
    ) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, failure) in failures.iter().take(4).enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                failure.name.clone(),
                failure.cause.clone(),
                format!("{}x", format_diagnostics_count(failure.count)),
            ));
        }
        card
    }

    fn diagnostics_slowest_spans_card(
        &self,
        spans: &[TraceDiagnosticsSpanOccurrence],
    ) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, span) in spans.iter().take(5).enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                span.name.clone(),
                format!("Trace {}", shorten_trace_id(&span.trace_id)),
                format_diagnostics_duration_ms(span.duration_ms),
            ));
        }
        card
    }

    fn diagnostics_logs_card(&self, logs: &[TraceDiagnosticsLogEvent]) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, event) in logs.iter().take(4).enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                event.message.clone(),
                format!("{} · {}", event.level, event.span_name),
                shorten_trace_id(&event.trace_id),
            ));
        }
        card
    }

    fn diagnostics_top_spans_card(&self, spans: &[TraceDiagnosticsSpanSummary]) -> gpui::Div {
        let mut card = self.settings_about_card();
        for (index, span) in spans.iter().take(5).enumerate() {
            card = card.child(self.diagnostics_row(
                index == 0,
                span.name.clone(),
                format!(
                    "{} failures · avg {}",
                    format_diagnostics_count(span.failure_count),
                    format_diagnostics_duration_ms(span.average_duration_ms)
                ),
                format!("{}x", format_diagnostics_count(span.count)),
            ));
        }
        card
    }

    fn diagnostics_row(
        &self,
        first: bool,
        title: impl Into<SharedString>,
        detail: impl Into<SharedString>,
        metric: impl Into<SharedString>,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(58.0))
            .border_t_1()
            .border_color(if first {
                self.theme.card
            } else {
                self.theme.border
            })
            .px_5()
            .py_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight(650.0))
                            .overflow_hidden()
                            .child(title.into()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(self.theme.muted_foreground.opacity(0.82))
                            .overflow_hidden()
                            .child(detail.into()),
                    ),
            )
            .child(
                div()
                    .ml_4()
                    .text_size(px(12.0))
                    .font_family(SharedString::from(MONO_FONT_FAMILY))
                    .text_color(self.theme.muted_foreground)
                    .child(metric.into()),
            )
    }

    fn settings_keybindings_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = build_default_keybinding_rows();
        let mut table = div()
            .flex()
            .flex_col()
            .min_w(px(680.0))
            .child(self.keybindings_table_header());

        for (index, row) in rows.iter().enumerate() {
            table = table.child(self.keybindings_table_row(row, index));
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
                            .bg(self.theme.background)
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
        let rows = self.provider_instance_rows();
        let mut card = div()
            .relative()
            .overflow_hidden()
            .rounded(px(16.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.card);

        for (index, row) in rows.iter().enumerate() {
            card = card.child(self.provider_instance_row(index, row, cx));
            if self.expanded_provider_index == Some(index) {
                card = card.child(self.provider_instance_details(row));
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

    fn provider_instance_rows(&self) -> Vec<ProviderInstanceRow> {
        self.snapshot
            .providers
            .iter()
            .map(|provider| {
                let summary = get_provider_summary(Some(provider));
                let advisory =
                    get_provider_version_advisory_presentation(provider.version_advisory.as_ref());
                let status = if !provider.enabled || !provider.installed {
                    ProviderStatus::NotConfigured
                } else if provider.status == r3_core::ServerProviderState::Warning {
                    ProviderStatus::EarlyAccess
                } else {
                    ProviderStatus::Ready
                };
                ProviderInstanceRow {
                    label: provider
                        .display_name
                        .clone()
                        .unwrap_or_else(|| provider.driver.clone()),
                    id: format!("provider-row-{}", provider.instance_id),
                    driver: provider.driver.clone(),
                    status,
                    badge: provider
                        .badge_label
                        .clone()
                        .or_else(|| advisory.as_ref().map(|_| "Update".to_string())),
                    description: summary.detail.unwrap_or(summary.headline),
                    enabled: provider.enabled,
                    version_label: get_provider_version_label(provider.version.as_deref()),
                    advisory_detail: advisory.map(|advisory| advisory.detail),
                    model_count: provider.models.len(),
                    accent_color: provider.accent_color.clone(),
                }
            })
            .collect()
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
        row: &ProviderInstanceRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let expanded = self.expanded_provider_index == Some(index);
        div()
            .id(SharedString::from(row.id.clone()))
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
                                            .child(row.label.clone()),
                                    )
                                    .child(self.provider_status_badge(row.status))
                                    .child(match row.badge.as_deref() {
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
                                    .child(row.description.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.provider_enabled_switch(index, row, cx))
                    .child(
                        div()
                            .id(SharedString::from(
                                row.id.replace("provider-row", "provider-expand"),
                            ))
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

    fn provider_instance_details(&self, row: &ProviderInstanceRow) -> impl IntoElement {
        let models_label = if row.model_count == 1 {
            "1 model".to_string()
        } else {
            format!("{} models", row.model_count)
        };
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
                    .child(self.provider_detail_cell("Driver", row.driver.clone()))
                    .child(self.provider_detail_cell("Models", models_label))
                    .child(self.provider_detail_cell("Environment", "No overrides"))
                    .child(
                        self.provider_detail_cell(
                            "Version",
                            row.version_label
                                .clone()
                                .unwrap_or_else(|| "Unknown".to_string()),
                        ),
                    )
                    .when_some(row.accent_color.clone(), |grid, color| {
                        grid.child(self.provider_detail_cell("Accent color", color))
                    })
                    .when_some(row.advisory_detail.clone(), |grid, advisory| {
                        grid.child(self.provider_detail_cell("Update", advisory))
                    }),
            )
    }

    fn provider_detail_cell(
        &self,
        label: &'static str,
        value: impl Into<String>,
    ) -> impl IntoElement {
        let value = value.into();
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
                    .line_height(px(18.0))
                    .child(value),
            )
    }

    fn provider_instance_icon(&self, row: &ProviderInstanceRow) -> impl IntoElement {
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

    fn provider_warning_badge(&self, label: &str) -> impl IntoElement {
        div()
            .rounded(px(9.0))
            .border_1()
            .border_color(hsla(36.0 / 360.0, 1.0, 0.50, 0.38))
            .bg(hsla(36.0 / 360.0, 1.0, 0.58, 0.06))
            .px_2()
            .py_0p5()
            .text_size(px(11.0))
            .text_color(hsla(36.0 / 360.0, 1.0, 0.42, 1.0))
            .child(label.to_string())
    }

    fn provider_enabled_switch(
        &self,
        index: usize,
        row: &ProviderInstanceRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let enabled = self
            .provider_enabled
            .get(index)
            .copied()
            .unwrap_or(row.enabled);
        div()
            .id(SharedString::from(
                row.id.replace("provider-row", "provider-toggle"),
            ))
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
            .items_start()
            .gap_2()
            .border_b_1()
            .border_color(hsla(38.0 / 360.0, 0.92, 0.50, 0.20))
            .bg(hsla(36.0 / 360.0, 1.0, 0.58, 0.05))
            .px_3()
            .py_2p5()
            .text_size(px(12.0))
            .line_height(px(18.0))
            .text_color(self.theme.muted_foreground)
            .child(
                svg()
                    .path("icons/info.svg")
                    .mt(px(2.0))
                    .w(px(14.0))
                    .h(px(14.0))
                    .flex_shrink_0()
                    .text_color(hsla(38.0 / 360.0, 0.92, 0.50, 1.0)),
            )
            .child(
                "Some shortcuts may be claimed by the browser before T3 Code sees them. Use the desktop app for better keybinding support.",
            )
    }

    fn keybindings_table_header(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(self.theme.border.opacity(0.70))
            .bg(hsla(0.0, 0.0, 0.0, 0.010))
            .px_4()
            .py_2()
            .text_size(px(11.0))
            .font_weight(FontWeight(650.0))
            .text_color(self.theme.muted_foreground)
            .child(div().w(px(282.0)).child("COMMAND"))
            .child(div().w(px(292.0)).child("KEYBINDING"))
            .child(div().w(px(294.0)).child("WHEN"))
            .child(div().w(px(60.0)).child("STATUS"))
    }

    fn keybindings_table_row(&self, row: &KeybindingSettingsRow, index: usize) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .min_h(px(40.0))
            .border_b_1()
            .border_color(self.theme.border.opacity(0.60))
            .bg(if index % 2 == 1 {
                hsla(0.0, 0.0, 0.0, 0.006)
            } else {
                hsla(0.0, 0.0, 0.0, 0.0)
            })
            .px_4()
            .py_1p5()
            .child(
                div()
                    .w(px(282.0))
                    .pr_4()
                    .text_size(px(13.0))
                    .font_weight(FontWeight(500.0))
                    .child(row.command_label.clone()),
            )
            .child(
                div()
                    .w(px(292.0))
                    .pr_4()
                    .child(self.keybinding_pill(row.key)),
            )
            .child(div().w(px(294.0)).pr_4().child(self.when_pill(row.when)))
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
        let mut group = div()
            .flex()
            .items_center()
            .gap_1()
            .h(px(28.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.0))
            .px_1p5();
        for part in value.split('+') {
            let label = match part {
                "mod" => "Ctrl".to_string(),
                "shift" => "⇧".to_string(),
                "alt" => "Alt".to_string(),
                "ctrl" => "⌃".to_string(),
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
        let diagnostics_description = format_diagnostics_description(DiagnosticsDescriptionInput {
            local_tracing_enabled: false,
            otlp_traces_enabled: false,
            otlp_traces_url: None,
            otlp_metrics_enabled: false,
            otlp_metrics_url: None,
        });

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
        description: impl Into<SharedString>,
        control: impl IntoElement,
        first: bool,
    ) -> impl IntoElement {
        let description = description.into();
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
                this.screen = R3Screen::SettingsDiagnostics;
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

    fn command_palette_action_items(&self) -> Vec<CommandPaletteItem> {
        let mut items = Vec::new();

        if let Some(project) = self.snapshot.active_project() {
            items.push(CommandPaletteItem::action(
                "action:new-thread",
                vec![
                    "new thread".to_string(),
                    "chat".to_string(),
                    "create".to_string(),
                    "draft".to_string(),
                ],
                format!("New thread in {}", project.name),
            ));
            items.push(CommandPaletteItem::submenu(
                "action:new-thread-in",
                vec![
                    "new thread".to_string(),
                    "project".to_string(),
                    "pick".to_string(),
                    "choose".to_string(),
                    "select".to_string(),
                ],
                "New thread in...",
            ));
        }

        items.push(CommandPaletteItem::action(
            "action:add-project",
            vec![
                "add project".to_string(),
                "folder".to_string(),
                "directory".to_string(),
                "browse".to_string(),
                "clone".to_string(),
                "remote".to_string(),
                "repository".to_string(),
                "repo".to_string(),
                "git".to_string(),
                "github".to_string(),
                "gitlab".to_string(),
                "bitbucket".to_string(),
                "azure".to_string(),
                "devops".to_string(),
                "url".to_string(),
                "environment".to_string(),
            ],
            "Add project",
        ));
        items.push(CommandPaletteItem::action(
            "action:settings",
            vec![
                "settings".to_string(),
                "preferences".to_string(),
                "configuration".to_string(),
                "keybindings".to_string(),
            ],
            "Open settings",
        ));

        items
    }

    fn command_palette_groups(&self) -> Vec<CommandPaletteGroup> {
        let project_search_items = build_project_action_items(&self.snapshot.projects, "project");
        let all_thread_items = build_thread_action_items(
            &self.snapshot.threads,
            self.snapshot
                .active_thread_summary()
                .map(|thread| thread.id.as_str()),
            &self.snapshot.projects,
            SidebarThreadSortOrder::UpdatedAt,
            COMMAND_PALETTE_REFERENCE_NOW_ISO,
            None,
        );

        if self.command_palette_submenu == Some(CommandPaletteSubmenu::NewThreadInProject) {
            let active_groups = vec![CommandPaletteGroup {
                value: "projects".to_string(),
                label: "Projects".to_string(),
                items: build_project_action_items(&self.snapshot.projects, "new-thread-in"),
            }];
            return filter_command_palette_groups(
                &active_groups,
                &self.command_palette_query,
                true,
                &[],
                &[],
            );
        }

        let recent_thread_items = build_thread_action_items(
            &self.snapshot.threads,
            self.snapshot
                .active_thread_summary()
                .map(|thread| thread.id.as_str()),
            &self.snapshot.projects,
            SidebarThreadSortOrder::UpdatedAt,
            COMMAND_PALETTE_REFERENCE_NOW_ISO,
            Some(RECENT_COMMAND_PALETTE_THREAD_LIMIT),
        );
        let active_groups = build_root_command_palette_groups(
            self.command_palette_action_items(),
            recent_thread_items,
        );

        filter_command_palette_groups(
            &active_groups,
            &self.command_palette_query,
            false,
            &project_search_items,
            &all_thread_items,
        )
    }

    fn flattened_command_palette_items(&self) -> Vec<CommandPaletteItem> {
        self.command_palette_groups()
            .into_iter()
            .flat_map(|group| group.items)
            .collect()
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
        let placeholder = if self.command_palette_submenu.is_some() {
            "Search..."
        } else {
            "Search commands, projects, and threads..."
        };
        let input_text = if self.command_palette_query.is_empty() {
            placeholder.to_string()
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
        let groups = self.command_palette_groups();
        let mut panel = div()
            .flex()
            .flex_col()
            .mx(px(-1.0))
            .border_1()
            .border_b_0()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_2();

        if groups.is_empty() {
            let empty_message = if self.command_palette_query.starts_with('>') {
                "No matching actions."
            } else {
                "No matching commands, projects, or threads."
            };
            return panel
                .child(
                    div()
                        .py_10()
                        .text_align(TextAlign::Center)
                        .text_size(px(14.0))
                        .text_color(self.theme.muted_foreground)
                        .child(empty_message),
                )
                .into_any_element();
        }

        let mut item_index = 0usize;
        for group in groups {
            panel = panel.child(self.command_group_label(group.label.as_str()));
            for item in group.items {
                let active = item_index == self.command_palette_highlighted_index;
                panel = panel.child(self.command_palette_row(item, active, cx));
                item_index = item_index.saturating_add(1);
            }
        }

        panel.into_any_element()
    }

    fn command_group_label(&self, label: &str) -> impl IntoElement {
        div()
            .px_2()
            .py_1p5()
            .text_size(px(12.0))
            .font_weight(FontWeight(600.0))
            .text_color(self.theme.muted_foreground)
            .child(label.to_string())
    }

    fn command_palette_row(
        &self,
        item: CommandPaletteItem,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let item_value = item.value.clone();
        let row_id = match item.value.as_str() {
            "action:add-project" => "command-palette-row-add-project".to_string(),
            "action:settings" => "command-palette-row-open-settings".to_string(),
            "action:new-thread" => "command-palette-row-new-thread".to_string(),
            "action:new-thread-in" => "command-palette-row-new-thread-in".to_string(),
            value => format!(
                "command-palette-row-{}",
                value
                    .chars()
                    .map(|character| {
                        if character.is_ascii_alphanumeric() {
                            character
                        } else {
                            '-'
                        }
                    })
                    .collect::<String>()
            ),
        };
        let icon_path = self.command_palette_item_icon_path(&item);
        let mut title = div()
            .flex()
            .min_w_0()
            .items_center()
            .gap_1p5()
            .text_size(px(14.0))
            .text_color(self.theme.foreground)
            .child(
                div()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(item.title.clone()),
            );
        if let Some(description) = item.description.clone() {
            title = div().flex().min_w_0().flex_col().child(title).child(
                div()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(px(12.0))
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child(description),
            );
        }

        let mut row = div()
            .id(SharedString::from(row_id))
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
                this.execute_palette_item_value(&item_value, window, cx);
            }))
            .child(self.command_palette_item_icon(icon_path, active))
            .child(div().flex_1().min_w_0().child(title));

        if let Some(timestamp) = item.timestamp.clone() {
            row = row.child(
                div()
                    .min_w(px(48.0))
                    .text_align(TextAlign::Right)
                    .text_size(px(10.0))
                    .text_color(self.theme.muted_foreground.opacity(0.70))
                    .child(timestamp),
            );
        }

        if item.kind == CommandPaletteItemKind::Submenu {
            row = row.child(
                svg()
                    .path("icons/chevron-right.svg")
                    .size_4()
                    .text_color(self.theme.muted_foreground.opacity(0.50)),
            );
        }

        row.into_any_element()
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
                        this.sidebar_options_menu_open = false;
                        this.open_command_palette(window, cx);
                        this.execute_palette_action(CommandPaletteAction::AddProject, window, cx);
                    }
                    "project-sort" => {
                        this.sidebar_options_menu_open = !this.sidebar_options_menu_open;
                        this.project_script_menu_open = false;
                        this.open_in_menu_open = false;
                        this.git_actions_menu_open = false;
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

    fn sidebar_options_menu_popup(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut popup = div()
            .id("sidebar-options-menu-popup")
            .absolute()
            .top(px(116.0))
            .left(px(17.0))
            .flex()
            .flex_col()
            .w(px(200.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(self.theme.border)
            .bg(self.theme.background)
            .p_1()
            .text_color(self.theme.foreground)
            .shadow(self.header_menu_shadow());

        popup = popup.child(self.sidebar_options_group_label("Sort projects", false));
        for option in sidebar_project_sort_options() {
            popup = popup.child(self.sidebar_project_sort_option_row(option, cx));
        }

        popup = popup.child(self.sidebar_options_group_label("Sort threads", true));
        for option in sidebar_thread_sort_options() {
            popup = popup.child(self.sidebar_thread_sort_option_row(option, cx));
        }

        popup = popup
            .child(self.sidebar_options_group_label("Visible threads", true))
            .child(self.sidebar_thread_preview_count_control(cx))
            .child(self.sidebar_options_separator())
            .child(self.sidebar_options_group_label("Group projects", true));

        for option in sidebar_project_grouping_options() {
            popup = popup.child(self.sidebar_project_grouping_option_row(option, cx));
        }

        popup.into_any_element()
    }

    fn sidebar_options_group_label(
        &self,
        label: &'static str,
        extra_top: bool,
    ) -> impl IntoElement {
        div()
            .px_2()
            .pt(if extra_top { px(8.0) } else { px(4.0) })
            .pb_1()
            .text_size(px(12.0))
            .font_weight(FontWeight(500.0))
            .text_color(self.theme.muted_foreground)
            .child(label)
    }

    fn sidebar_options_radio_row(
        &self,
        id: String,
        label: &'static str,
        selected: bool,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(SharedString::from(id))
            .flex()
            .items_center()
            .gap_2()
            .h(px(28.0))
            .rounded(px(2.0))
            .pl_2()
            .pr_4()
            .text_size(px(12.0))
            .text_color(self.theme.foreground)
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .w(px(16.0))
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("icons/check.svg")
                            .size_4()
                            .text_color(self.theme.foreground)
                            .opacity(if selected { 1.0 } else { 0.0 }),
                    ),
            )
            .child(div().min_w_0().overflow_hidden().child(label))
    }

    fn sidebar_project_sort_option_row(
        &self,
        option: SidebarProjectSortOrder,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        self.sidebar_options_radio_row(
            format!("sidebar-project-sort-{option:?}"),
            sidebar_project_sort_label(option),
            self.sidebar_options_state.project_sort_order == option,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.sidebar_options_state.project_sort_order = option;
            cx.notify();
        }))
    }

    fn sidebar_thread_sort_option_row(
        &self,
        option: SidebarThreadSortOrder,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        self.sidebar_options_radio_row(
            format!("sidebar-thread-sort-{option:?}"),
            sidebar_thread_sort_label(option),
            self.sidebar_options_state.thread_sort_order == option,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.sidebar_options_state.thread_sort_order = option;
            cx.notify();
        }))
    }

    fn sidebar_project_grouping_option_row(
        &self,
        option: SidebarProjectGroupingMode,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        self.sidebar_options_radio_row(
            format!("sidebar-project-grouping-{option:?}"),
            sidebar_project_grouping_label(option),
            self.sidebar_options_state.project_grouping_mode == option,
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.sidebar_options_state.project_grouping_mode = option;
            cx.notify();
        }))
    }

    fn sidebar_thread_preview_count_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().px_2().py_1().child(
            div()
                .id("sidebar-thread-preview-count")
                .flex()
                .items_center()
                .h(px(26.0))
                .w(px(112.0))
                .rounded(px(6.0))
                .border_1()
                .border_color(self.theme.primary)
                .overflow_hidden()
                .text_size(px(12.0))
                .child(self.sidebar_thread_preview_count_button("decrement", "icons/minus.svg", cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .h_full()
                        .w(px(36.0))
                        .border_l_1()
                        .border_r_1()
                        .border_color(self.theme.border)
                        .text_color(self.theme.foreground)
                        .child(self.sidebar_options_state.thread_preview_count.to_string()),
                )
                .child(self.sidebar_thread_preview_count_button("increment", "icons/plus.svg", cx)),
        )
    }

    fn sidebar_thread_preview_count_button(
        &self,
        id: &'static str,
        icon_path: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("sidebar-thread-preview-{id}")))
            .flex()
            .items_center()
            .justify_center()
            .h_full()
            .w(px(37.0))
            .text_color(self.theme.muted_foreground)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                match id {
                    "decrement" => {
                        this.sidebar_options_state.thread_preview_count = this
                            .sidebar_options_state
                            .thread_preview_count
                            .saturating_sub(1)
                            .max(r3_core::MIN_SIDEBAR_THREAD_PREVIEW_COUNT);
                    }
                    "increment" => {
                        this.sidebar_options_state.thread_preview_count = this
                            .sidebar_options_state
                            .thread_preview_count
                            .saturating_add(1)
                            .min(r3_core::MAX_SIDEBAR_THREAD_PREVIEW_COUNT);
                    }
                    _ => {}
                }
                cx.notify();
            }))
            .child(
                svg()
                    .path(icon_path)
                    .size_3p5()
                    .text_color(self.theme.muted_foreground),
            )
    }

    fn sidebar_options_separator(&self) -> impl IntoElement {
        div().mx_2().my_1().h(px(1.0)).bg(self.theme.border)
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
        let mut group = div().px_3().child(
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
        );

        let project_threads = self
            .snapshot
            .threads
            .iter()
            .filter(|thread| {
                thread.environment_id == project.environment_id
                    && thread.project_id == project.id
                    && thread.archived_at.is_none()
            })
            .collect::<Vec<_>>();

        if project_threads.is_empty() {
            group = group.child(self.sidebar_draft_thread_row());
        } else {
            let active_thread_id = self
                .snapshot
                .active_thread_summary()
                .map(|thread| thread.id.as_str());
            let mut thread_list = div()
                .mt_2()
                .ml_3()
                .pl_1p5()
                .border_l_1()
                .border_color(self.theme.border)
                .flex()
                .flex_col()
                .gap_0p5();

            for thread in project_threads {
                thread_list =
                    thread_list.child(self.sidebar_project_thread_row(thread, active_thread_id));
            }
            group = group.child(thread_list);
        }

        group
    }

    fn sidebar_draft_thread_row(&self) -> impl IntoElement {
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
            )
    }

    fn sidebar_project_thread_row(
        &self,
        thread: &r3_core::ThreadSummary,
        active_thread_id: Option<&str>,
    ) -> impl IntoElement {
        let is_active = active_thread_id == Some(thread.id.as_str());
        let status = sidebar_thread_status_label(thread);

        div()
            .id(SharedString::from(format!("sidebar-thread-{}", thread.id)))
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .h(px(26.0))
            .rounded(px(6.0))
            .px_2()
            .text_size(px(12.0))
            .text_color(if is_active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground
            })
            .bg(if is_active {
                self.theme.accent.opacity(0.85)
            } else {
                self.theme.card
            })
            .child(
                div()
                    .flex()
                    .min_w_0()
                    .flex_1()
                    .items_center()
                    .gap_1p5()
                    .when_some(status, |row, status| {
                        row.child(self.sidebar_thread_status_pill(status))
                    })
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(thread.title.clone()),
                    ),
            )
            .child(
                div()
                    .ml_auto()
                    .flex_shrink_0()
                    .text_size(px(10.0))
                    .text_color(if is_active {
                        self.theme.foreground.opacity(0.72)
                    } else {
                        self.theme.muted_foreground.opacity(0.45)
                    })
                    .child(sidebar_thread_relative_time(thread)),
            )
    }

    fn sidebar_thread_status_pill(&self, status: SidebarThreadStatus) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(10.0))
            .text_color(status.color)
            .child(
                div()
                    .w(px(6.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(status.dot_color),
            )
            .child(status.label)
    }

    fn command_palette_item_icon(&self, icon_path: &'static str, active: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(16.0))
            .h(px(16.0))
            .text_color(if active {
                self.theme.foreground
            } else {
                self.theme.muted_foreground.opacity(0.80)
            })
            .child(svg().path(icon_path).size_4())
    }

    fn command_palette_item_icon_path(&self, item: &CommandPaletteItem) -> &'static str {
        if item.value == "action:add-project" {
            return "icons/folder-plus.svg";
        }
        if item.value == "action:settings" {
            return "icons/settings-2.svg";
        }
        if item.value == "action:new-thread"
            || item.value == "action:new-thread-in"
            || item.value.starts_with("new-thread-in:")
        {
            return "icons/square-pen.svg";
        }
        if item.value.starts_with("project:") {
            return "icons/folder.svg";
        }
        if item.value.starts_with("thread:") {
            return "icons/message-square.svg";
        }
        "icons/pilcrow.svg"
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
        if self.model_picker_open {
            self.handle_model_picker_key(event, cx);
            return;
        }

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
                if self.command_palette_query.is_empty() && self.command_palette_submenu.is_some() {
                    self.command_palette_submenu = None;
                } else {
                    self.command_palette_query.pop();
                }
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
        self.command_palette_submenu = None;
        self.settings_select_open = None;
        window.focus(&self.command_palette_focus_handle);
        cx.notify();
    }

    fn close_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_highlighted_index = 0;
        self.command_palette_submenu = None;
        window.focus(&self.shell_focus_handle);
        cx.notify();
    }

    fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = R3Screen::Empty;
        self.settings_select_open = None;
        self.command_palette_open = false;
        self.command_palette_query.clear();
        self.command_palette_highlighted_index = 0;
        self.command_palette_submenu = None;
        window.focus(&self.shell_focus_handle);
        cx.notify();
    }

    fn set_settings_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.screen = R3Screen::Settings;
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

    fn active_composer_menu_item(&self) -> Option<ComposerCommandItem> {
        let items = self.composer_menu_items();
        let active_id = self.active_composer_menu_item_id(&items)?;
        items.into_iter().find(|item| item.id() == active_id)
    }

    fn nudge_active_composer_menu(
        &mut self,
        direction: ComposerMenuNudgeDirection,
        cx: &mut Context<Self>,
    ) {
        let items = self.composer_menu_items();
        let current_active = self.active_composer_menu_item_id(&items);
        self.composer_highlighted_item_id =
            nudge_composer_menu_highlight(&items, current_active.as_deref(), direction);
        self.composer_highlighted_search_key =
            composer_menu_search_key(self.active_composer_trigger().as_ref());
        cx.notify();
    }

    fn apply_composer_command_item(&mut self, item: &ComposerCommandItem, cx: &mut Context<Self>) {
        let Some(trigger) = self.active_composer_trigger() else {
            return;
        };
        let Some(selection) =
            resolve_composer_command_selection(&self.composer_prompt, &trigger, item)
        else {
            return;
        };
        let next = replace_text_range(
            &self.composer_prompt,
            selection.range_start as f64,
            selection.range_end as f64,
            &selection.replacement,
        );
        self.composer_prompt = next.text;
        if let Some(interaction_mode) = selection.interaction_mode {
            self.composer_plan_mode = interaction_mode == ComposerSlashCommand::Plan;
        }
        if selection.open_model_picker {
            self.model_picker_open = true;
            self.model_picker_query.clear();
        }
        self.composer_highlighted_item_id = None;
        self.composer_highlighted_search_key = None;
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
        self.composer_highlighted_item_id = None;
        self.composer_highlighted_search_key = None;
        self.composer_submitted_count = self.composer_submitted_count.saturating_add(1);
        cx.notify();
    }

    fn select_provider_model(&mut self, instance_id: &str, slug: &str) {
        let Some(provider) = self
            .snapshot
            .providers
            .iter()
            .find(|provider| provider.instance_id == instance_id)
        else {
            return;
        };
        let selected_model =
            resolve_selectable_model(&provider.driver, Some(slug), &provider.models)
                .unwrap_or_else(|| slug.to_string());
        self.snapshot.selected_provider_instance_id = instance_id.to_string();
        self.snapshot.selected_model = selected_model;
        self.model_picker_selected_instance =
            ModelPickerSelectedInstance::Instance(instance_id.to_string());
        self.model_picker_open = false;
        self.model_picker_query.clear();
    }

    fn handle_model_picker_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" => {
                self.model_picker_open = false;
                self.model_picker_query.clear();
                cx.notify();
            }
            "backspace" => {
                self.model_picker_query.pop();
                cx.notify();
            }
            "enter" => {
                let state = self.current_model_picker_state();
                if let Some(model) = state.filtered_models.first() {
                    let instance_id = model.instance_id.clone();
                    let slug = model.slug.clone();
                    self.select_provider_model(&instance_id, &slug);
                    cx.notify();
                }
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
                    self.model_picker_query.push_str(text);
                    cx.notify();
                }
            }
        }
        cx.stop_propagation();
    }

    fn on_composer_key_down(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.snapshot.active_pending_approval().is_none()
            && self.snapshot.active_pending_user_input_progress().is_none()
            && self.active_composer_trigger().is_some()
        {
            match event.keystroke.key.as_str() {
                "down" => {
                    self.nudge_active_composer_menu(ComposerMenuNudgeDirection::ArrowDown, cx);
                    cx.stop_propagation();
                    return;
                }
                "up" => {
                    self.nudge_active_composer_menu(ComposerMenuNudgeDirection::ArrowUp, cx);
                    cx.stop_propagation();
                    return;
                }
                "enter" => {
                    if let Some(item) = self.active_composer_menu_item() {
                        self.apply_composer_command_item(&item, cx);
                        cx.stop_propagation();
                        return;
                    }
                }
                _ => {}
            }
        }

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
                        self.composer_highlighted_item_id = None;
                        self.composer_highlighted_search_key = None;
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
                                self.composer_highlighted_item_id = None;
                                self.composer_highlighted_search_key = None;
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

    fn move_palette_highlight(&mut self, direction: isize, cx: &mut Context<Self>) {
        let item_count = self.flattened_command_palette_items().len();
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
        let items = self.flattened_command_palette_items();
        let Some(item) = items.get(self.command_palette_highlighted_index) else {
            return;
        };
        self.execute_palette_item_value(&item.value, window, cx);
    }

    fn execute_palette_item_value(
        &mut self,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if value == "action:new-thread" {
            self.execute_palette_action(CommandPaletteAction::NewThread, window, cx);
        } else if value == "action:new-thread-in" {
            self.execute_palette_action(CommandPaletteAction::NewThreadInProject, window, cx);
        } else if value == "action:add-project" {
            self.execute_palette_action(CommandPaletteAction::AddProject, window, cx);
        } else if value == "action:settings" {
            self.execute_palette_action(CommandPaletteAction::OpenSettings, window, cx);
        } else if value.starts_with("project:") {
            self.execute_palette_action(CommandPaletteAction::OpenProject, window, cx);
        } else if value.starts_with("new-thread-in:") {
            self.execute_palette_action(CommandPaletteAction::NewThread, window, cx);
        } else if value.starts_with("thread:") {
            self.execute_palette_action(CommandPaletteAction::OpenThread, window, cx);
        }
    }

    fn execute_palette_action(
        &mut self,
        action: CommandPaletteAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            CommandPaletteAction::NewThread => {
                self.snapshot = AppSnapshot::draft_reference_state();
                self.screen = R3Screen::Draft;
                self.close_command_palette(window, cx);
            }
            CommandPaletteAction::NewThreadInProject => {
                self.command_palette_submenu = Some(CommandPaletteSubmenu::NewThreadInProject);
                self.command_palette_query.clear();
                self.command_palette_highlighted_index = 0;
                window.focus(&self.command_palette_focus_handle);
                cx.notify();
            }
            CommandPaletteAction::AddProject => {
                self.command_palette_submenu = None;
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
            CommandPaletteAction::OpenProject | CommandPaletteAction::OpenThread => {
                self.screen = R3Screen::ActiveChat;
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

fn branch_toolbar_workspace_icon(
    mode: DraftThreadEnvMode,
    active_worktree_path: Option<&str>,
) -> &'static str {
    if mode == DraftThreadEnvMode::Worktree {
        "icons/folder-git-2.svg"
    } else if active_worktree_path.is_some() {
        "icons/folder-git.svg"
    } else {
        "icons/folder.svg"
    }
}

fn project_script_icon_path(icon: ProjectScriptIcon) -> &'static str {
    match icon {
        ProjectScriptIcon::Play => "icons/play.svg",
        ProjectScriptIcon::Test => "icons/flask-conical.svg",
        ProjectScriptIcon::Lint => "icons/list-checks.svg",
        ProjectScriptIcon::Configure => "icons/wrench.svg",
        ProjectScriptIcon::Build => "icons/hammer.svg",
        ProjectScriptIcon::Debug => "icons/bug.svg",
    }
}

fn editor_option_icon_path(option: EditorOption) -> &'static str {
    match option.id {
        r3_core::EditorId::Cursor => "icons/cursor.svg",
        r3_core::EditorId::FileManager => "icons/folder-closed.svg",
        r3_core::EditorId::VsCode => "icons/visual-studio-code.svg",
        r3_core::EditorId::Zed => "icons/zed.svg",
        _ => "icons/terminal.svg",
    }
}

fn git_action_icon_path(icon: GitActionIconName) -> &'static str {
    match icon {
        GitActionIconName::Commit => "icons/git-commit.svg",
        GitActionIconName::Push => "icons/cloud-upload.svg",
        GitActionIconName::Pr => "icons/git-pull-request.svg",
    }
}

fn git_action_warning_color() -> gpui::Hsla {
    hsla(38.0 / 360.0, 0.92, 0.50, 1.0)
}

#[derive(Debug, Clone, Copy)]
struct SidebarThreadStatus {
    label: &'static str,
    color: gpui::Hsla,
    dot_color: gpui::Hsla,
}

fn sidebar_thread_status_label(thread: &r3_core::ThreadSummary) -> Option<SidebarThreadStatus> {
    if thread.has_pending_approvals {
        return Some(SidebarThreadStatus {
            label: "Pending Approval",
            color: hsla(35.0 / 360.0, 0.74, 0.42, 1.0),
            dot_color: hsla(38.0 / 360.0, 0.92, 0.50, 1.0),
        });
    }
    if thread.has_pending_user_input {
        return Some(SidebarThreadStatus {
            label: "Awaiting Input",
            color: hsla(243.0 / 360.0, 0.72, 0.58, 1.0),
            dot_color: hsla(239.0 / 360.0, 0.84, 0.67, 1.0),
        });
    }
    if thread.status == ThreadStatus::Running {
        return Some(SidebarThreadStatus {
            label: "Working",
            color: hsla(201.0 / 360.0, 0.80, 0.42, 1.0),
            dot_color: hsla(199.0 / 360.0, 0.89, 0.48, 1.0),
        });
    }
    if thread.has_actionable_proposed_plan {
        return Some(SidebarThreadStatus {
            label: "Plan Ready",
            color: hsla(262.0 / 360.0, 0.72, 0.52, 1.0),
            dot_color: hsla(258.0 / 360.0, 0.90, 0.66, 1.0),
        });
    }
    if thread.status == ThreadStatus::Failed {
        return Some(SidebarThreadStatus {
            label: "Failed",
            color: diff_destructive_color(),
            dot_color: diff_destructive_color(),
        });
    }
    None
}

fn sidebar_thread_relative_time(thread: &r3_core::ThreadSummary) -> String {
    let timestamp = thread
        .latest_user_message_at
        .as_deref()
        .unwrap_or(thread.updated_at.as_str());
    if timestamp.starts_with("2026-03-04") {
        "68d ago".to_string()
    } else if timestamp.starts_with("2026-03-03") {
        "69d ago".to_string()
    } else {
        "just now".to_string()
    }
}

fn working_timer_label(started_at: &str) -> Option<String> {
    format_working_timer_at(started_at, REFERENCE_WORKING_TIMER_NOW)
}

fn provider_driver_icon_path(driver: &str) -> &'static str {
    match driver {
        "codex" => "icons/openai.svg",
        "claudeAgent" => "icons/claude-ai.svg",
        "cursor" => "icons/cursor.svg",
        "opencode" => "icons/opencode.svg",
        _ => "icons/bot.svg",
    }
}

fn provider_icon_color(driver: &str, fallback: gpui::Hsla) -> gpui::Hsla {
    match driver {
        "claudeAgent" => hsla(14.0 / 360.0, 0.62, 0.60, 1.0),
        _ => fallback,
    }
}

fn provider_accent_color(value: &str) -> Option<gpui::Hsla> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some(rgb_to_hsla(red, green, blue))
}

fn rgb_to_hsla(red: f32, green: f32, blue: f32) -> gpui::Hsla {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return hsla(0.0, 0.0, lightness, 1.0);
    }
    let delta = max - min;
    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if (max - red).abs() < f32::EPSILON {
        ((green - blue) / delta + if green < blue { 6.0 } else { 0.0 }) / 6.0
    } else if (max - green).abs() < f32::EPSILON {
        ((blue - red) / delta + 2.0) / 6.0
    } else {
        ((red - green) / delta + 4.0) / 6.0
    };
    hsla(hue, saturation, lightness, 1.0)
}

fn pending_blue() -> gpui::Hsla {
    hsla(217.0 / 360.0, 0.91, 0.60, 1.0)
}

fn pending_blue_icon() -> gpui::Hsla {
    hsla(213.0 / 360.0, 0.94, 0.68, 1.0)
}

fn diff_patch_text_segments(
    text: &'static str,
    default_color: gpui::Hsla,
) -> Vec<(&'static str, gpui::Hsla)> {
    let mut segments = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let next_char = text[index..].chars().next().unwrap();
        if next_char.is_ascii_alphanumeric() || next_char == '_' {
            let start = index;
            index += next_char.len_utf8();
            while index < text.len() {
                let current = text[index..].chars().next().unwrap();
                if current.is_ascii_alphanumeric() || current == '_' {
                    index += current.len_utf8();
                } else {
                    break;
                }
            }
            let word = &text[start..index];
            segments.push((
                word,
                diff_patch_word_color(text, index, word, default_color),
            ));
        } else {
            let start = index;
            index += next_char.len_utf8();
            while index < text.len() {
                let current = text[index..].chars().next().unwrap();
                if current.is_ascii_alphanumeric() || current == '_' {
                    break;
                }
                index += current.len_utf8();
            }
            segments.push((&text[start..index], default_color));
        }
    }
    segments
}

fn diff_patch_word_is_function(text: &str, word_end: usize) -> bool {
    text[word_end..]
        .chars()
        .find(|character| !character.is_whitespace())
        == Some('(')
}

fn diff_patch_word_color(
    text: &str,
    word_end: usize,
    word: &str,
    default_color: gpui::Hsla,
) -> gpui::Hsla {
    if matches!(
        word,
        "pub" | "fn" | "struct" | "impl" | "let" | "mut" | "match" | "return"
    ) {
        return diff_syntax_keyword_color();
    }
    if matches!(
        word,
        "bool" | "String" | "usize" | "u32" | "i32" | "Option" | "Vec"
    ) {
        return diff_syntax_type_color();
    }
    if diff_patch_word_is_function(text, word_end) {
        return diff_syntax_function_color();
    }
    default_color
}

fn diff_syntax_keyword_color() -> gpui::Hsla {
    rgb_to_hsla(255.0 / 255.0, 91.0 / 255.0, 158.0 / 255.0)
}

fn diff_syntax_type_color() -> gpui::Hsla {
    rgb_to_hsla(198.0 / 255.0, 53.0 / 255.0, 228.0 / 255.0)
}

fn diff_syntax_function_color() -> gpui::Hsla {
    rgb_to_hsla(139.0 / 255.0, 67.0 / 255.0, 244.0 / 255.0)
}

fn diff_syntax_plain_color() -> gpui::Hsla {
    rgb_to_hsla(208.0 / 255.0, 120.0 / 255.0, 40.0 / 255.0)
}

fn diff_success_color() -> gpui::Hsla {
    hsla(142.0 / 360.0, 0.71, 0.38, 1.0)
}

fn diff_modified_color() -> gpui::Hsla {
    hsla(203.0 / 360.0, 1.0, 0.50, 1.0)
}

fn diff_destructive_color() -> gpui::Hsla {
    hsla(0.0, 0.72, 0.48, 1.0)
}

fn reference_process_diagnostics_result() -> ProcessDiagnosticsResult {
    ProcessDiagnosticsResult {
        server_pid: 100,
        read_at: "2026-05-05T10:00:00.000Z".to_string(),
        process_count: 2,
        total_rss_bytes: 6_000,
        total_cpu_percent: 4.75,
        processes: vec![
            ProcessDiagnosticsEntry {
                pid: 101,
                ppid: 100,
                pgid: Some(100),
                status: "S".to_string(),
                cpu_percent: 1.5,
                rss_bytes: 2_000,
                elapsed: "00:20".to_string(),
                command: "codex app-server".to_string(),
                depth: 0,
                child_pids: vec![102],
            },
            ProcessDiagnosticsEntry {
                pid: 102,
                ppid: 101,
                pgid: Some(100),
                status: "R".to_string(),
                cpu_percent: 3.25,
                rss_bytes: 4_000,
                elapsed: "00:05".to_string(),
                command: "git status".to_string(),
                depth: 1,
                child_pids: Vec::new(),
            },
        ],
        error: None,
    }
}

fn reference_trace_diagnostics_result() -> TraceDiagnosticsResult {
    TraceDiagnosticsResult {
        trace_file_path: "/tmp/server.trace.ndjson".to_string(),
        scanned_file_paths: vec![
            "/tmp/server.trace.ndjson.1".to_string(),
            "/tmp/server.trace.ndjson".to_string(),
        ],
        read_at: "2026-05-05T10:00:00.000Z".to_string(),
        record_count: 4,
        parse_error_count: 1,
        first_span_at: Some("1970-01-01T00:00:01.000Z".to_string()),
        last_span_at: Some("1970-01-01T00:00:05.025Z".to_string()),
        failure_count: 2,
        interruption_count: 1,
        slow_span_threshold_ms: 1_000.0,
        slow_span_count: 1,
        log_level_counts: BTreeMap::from([("Error".to_string(), 1), ("Warning".to_string(), 1)]),
        top_spans_by_count: vec![
            TraceDiagnosticsSpanSummary {
                name: "orchestration.dispatch".to_string(),
                count: 2,
                failure_count: 2,
                total_duration_ms: 1_750.0,
                average_duration_ms: 875.0,
                max_duration_ms: 1_500.0,
            },
            TraceDiagnosticsSpanSummary {
                name: "server.getConfig".to_string(),
                count: 1,
                failure_count: 0,
                total_duration_ms: 50.0,
                average_duration_ms: 50.0,
                max_duration_ms: 50.0,
            },
        ],
        slowest_spans: vec![
            TraceDiagnosticsSpanOccurrence {
                name: "orchestration.dispatch".to_string(),
                duration_ms: 1_500.0,
                ended_at: "1970-01-01T00:00:03.500Z".to_string(),
                trace_id: "trace-b".to_string(),
                span_id: "span-b".to_string(),
            },
            TraceDiagnosticsSpanOccurrence {
                name: "orchestration.dispatch".to_string(),
                duration_ms: 250.0,
                ended_at: "1970-01-01T00:00:04.250Z".to_string(),
                trace_id: "trace-c".to_string(),
                span_id: "span-c".to_string(),
            },
        ],
        common_failures: vec![TraceDiagnosticsFailureSummary {
            name: "orchestration.dispatch".to_string(),
            cause: "Provider crashed".to_string(),
            count: 2,
            last_seen_at: "1970-01-01T00:00:04.250Z".to_string(),
            trace_id: "trace-c".to_string(),
            span_id: "span-c".to_string(),
        }],
        latest_failures: vec![
            TraceDiagnosticsRecentFailure {
                name: "orchestration.dispatch".to_string(),
                cause: "Provider crashed".to_string(),
                duration_ms: 250.0,
                ended_at: "1970-01-01T00:00:04.250Z".to_string(),
                trace_id: "trace-c".to_string(),
                span_id: "span-c".to_string(),
            },
            TraceDiagnosticsRecentFailure {
                name: "orchestration.dispatch".to_string(),
                cause: "Provider crashed".to_string(),
                duration_ms: 1_500.0,
                ended_at: "1970-01-01T00:00:03.500Z".to_string(),
                trace_id: "trace-b".to_string(),
                span_id: "span-b".to_string(),
            },
        ],
        latest_warning_and_error_logs: vec![
            TraceDiagnosticsLogEvent {
                span_name: "git.status".to_string(),
                level: "Warning".to_string(),
                message: "status delayed".to_string(),
                seen_at: "1970-01-01T00:00:05.010Z".to_string(),
                trace_id: "trace-d".to_string(),
                span_id: "span-d".to_string(),
            },
            TraceDiagnosticsLogEvent {
                span_name: "orchestration.dispatch".to_string(),
                level: "Error".to_string(),
                message: "provider failed".to_string(),
                seen_at: "1970-01-01T00:00:03.400Z".to_string(),
                trace_id: "trace-b".to_string(),
                span_id: "span-b".to_string(),
            },
        ],
        partial_failure: None,
        error: None,
    }
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

fn normalize_work_log_label(value: &str) -> String {
    let trimmed = value.trim();
    for suffix in [" complete", " completed"] {
        if trimmed.to_ascii_lowercase().ends_with(suffix) {
            return trimmed[..trimmed.len() - suffix.len()].trim().to_string();
        }
    }
    trimmed.to_string()
}

fn work_log_heading(entry: &WorkLogEntry) -> String {
    normalize_work_log_label(entry.tool_title.as_deref().unwrap_or(&entry.label))
}

fn work_log_preview(entry: &WorkLogEntry) -> Option<String> {
    if let Some(command) = entry.command.as_ref().filter(|command| !command.is_empty()) {
        return Some(command.clone());
    }
    if let Some(detail) = entry.detail.as_ref().filter(|detail| !detail.is_empty()) {
        return Some(detail.clone());
    }
    let first_path = entry.changed_files.first()?;
    if entry.changed_files.len() == 1 {
        Some(first_path.clone())
    } else {
        Some(format!(
            "{} +{} more",
            first_path,
            entry.changed_files.len() - 1
        ))
    }
}

fn work_log_icon_path(entry: &WorkLogEntry) -> &'static str {
    if entry.command.is_some() || entry.item_type.as_deref() == Some("command_execution") {
        return "icons/terminal.svg";
    }
    if !entry.changed_files.is_empty() || entry.item_type.as_deref() == Some("file_change") {
        return "icons/pen-line.svg";
    }
    match entry.tone {
        ActivityTone::Error => "icons/triangle-alert.svg",
        ActivityTone::Thinking => "icons/bot.svg",
        ActivityTone::Info | ActivityTone::Approval => "icons/check.svg",
        ActivityTone::Tool => "icons/terminal.svg",
    }
}

fn work_log_tone_color(tone: ActivityTone, theme: Theme) -> gpui::Hsla {
    match tone {
        ActivityTone::Error => diff_destructive_color().opacity(0.72),
        ActivityTone::Tool => theme.muted_foreground.opacity(0.70),
        ActivityTone::Thinking => theme.muted_foreground.opacity(0.50),
        ActivityTone::Info | ActivityTone::Approval => theme.muted_foreground.opacity(0.55),
    }
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
        Ok("composer-focused") | Ok("focused-composer") => (R3Screen::ComposerFocused, false),
        Ok("composer-menu") | Ok("slash-menu") => (R3Screen::ComposerCommandMenu, false),
        Ok("composer-inline-tokens") | Ok("inline-tokens") => {
            (R3Screen::ComposerInlineTokens, false)
        }
        Ok("active-chat") | Ok("chat") => (R3Screen::ActiveChat, false),
        Ok("running-turn") | Ok("running") => (R3Screen::RunningTurn, false),
        Ok("pending-approval") | Ok("approval") => (R3Screen::PendingApproval, false),
        Ok("pending-user-input") | Ok("user-input") => (R3Screen::PendingUserInput, false),
        Ok("terminal-drawer") | Ok("terminal") => (R3Screen::TerminalDrawer, false),
        Ok("diff-panel") | Ok("diff") => (R3Screen::DiffPanel, false),
        Ok("branch-toolbar") | Ok("branch") => (R3Screen::BranchToolbar, false),
        Ok("sidebar-options-menu") | Ok("sidebar-options") => (R3Screen::SidebarOptionsMenu, false),
        Ok("project-scripts-menu") | Ok("scripts-menu") => (R3Screen::ProjectScriptsMenu, false),
        Ok("open-in-menu") | Ok("editor-menu") => (R3Screen::OpenInMenu, false),
        Ok("git-actions-menu") | Ok("git-menu") => (R3Screen::GitActionsMenu, false),
        Ok("provider-model-picker") | Ok("model-picker") => (R3Screen::ProviderModelPicker, false),
        Ok("settings-diagnostics") | Ok("diagnostics") => (R3Screen::SettingsDiagnostics, false),
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
                R3Screen::Draft | R3Screen::ComposerFocused => AppSnapshot::draft_reference_state(),
                R3Screen::ActiveChat => AppSnapshot::active_chat_reference_state(),
                R3Screen::RunningTurn => AppSnapshot::running_turn_reference_state(),
                R3Screen::PendingApproval => AppSnapshot::pending_approval_reference_state(),
                R3Screen::PendingUserInput => AppSnapshot::pending_user_input_reference_state(),
                R3Screen::ComposerCommandMenu | R3Screen::ComposerInlineTokens => {
                    AppSnapshot::draft_reference_state()
                }
                R3Screen::TerminalDrawer => AppSnapshot::terminal_drawer_reference_state(),
                R3Screen::DiffPanel => AppSnapshot::diff_panel_reference_state(),
                R3Screen::BranchToolbar | R3Screen::SidebarOptionsMenu => {
                    AppSnapshot::branch_toolbar_reference_state()
                }
                R3Screen::ProjectScriptsMenu => AppSnapshot::active_chat_reference_state(),
                R3Screen::OpenInMenu | R3Screen::GitActionsMenu => {
                    AppSnapshot::branch_toolbar_reference_state()
                }
                R3Screen::ProviderModelPicker => {
                    AppSnapshot::provider_model_picker_reference_state()
                }
                R3Screen::Empty | R3Screen::Settings | R3Screen::SettingsDiagnostics => {
                    AppSnapshot::empty_reference_state()
                }
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

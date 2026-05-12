# UI Parity Plan

R3Code should be judged against frozen reference screenshots, not against taste or memory.

## Required Reference Screens

- Empty project/no active thread
- Sidebar with projects and threads
- Active chat with user and agent messages
- Running agent turn
- Pending approval or user input
- Composer focused and unfocused
- Command palette
- Settings
- Terminal drawer
- Diff panel
- Light theme
- Dark theme

## Refreshing References

Use the Rust xtask to launch the frozen upstream checkout with an isolated reference home, capture the currently automated reference screens, and stop the watcher process tree:

```text
cargo run -p xtask -- capture-reference-browser
```

The task currently captures:

- `reference/screenshots/upstream-empty-reference.png`
- `reference/screenshots/upstream-empty-dark-reference.png`
- `reference/screenshots/upstream-command-palette-reference.png`
- `reference/screenshots/upstream-draft-reference.png`
- `reference/screenshots/upstream-composer-focused-reference.png`
- `reference/screenshots/upstream-composer-menu-reference.png`
- `reference/screenshots/upstream-composer-inline-tokens-reference.png`
- `reference/screenshots/upstream-provider-model-picker-reference.png`
- `reference/screenshots/upstream-branch-toolbar-reference.png`
- `reference/screenshots/upstream-active-chat-reference.png`
- `reference/screenshots/upstream-running-turn-reference.png`
- `reference/screenshots/upstream-terminal-drawer-reference.png`
- `reference/screenshots/upstream-diff-panel-reference.png`
- `reference/screenshots/upstream-pending-user-input-reference.png`
- `reference/screenshots/upstream-pending-approval-reference.png`
- `reference/screenshots/upstream-settings-reference.png`
- `reference/screenshots/upstream-settings-keybindings-reference.png`
- `reference/screenshots/upstream-settings-providers-reference.png`
- `reference/screenshots/upstream-settings-source-control-reference.png`
- `reference/screenshots/upstream-settings-connections-reference.png`
- `reference/screenshots/upstream-settings-diagnostics-reference.png`
- `reference/screenshots/upstream-settings-archive-reference.png`
- `reference/screenshots/upstream-settings-theme-menu-reference.png`
- `reference/screenshots/upstream-settings-dark-reference.png`

Do not use screenshots from a different upstream commit unless `docs/reference/UPSTREAM_REFERENCE.md` is intentionally updated.

## Parity Gates

Each implemented GPUI screen needs:

- Same viewport size as the reference capture
- Same shell structure
- Same sidebar width and resize limits
- Same background, card, border, text, muted text, and accent colors
- Same font stack and font sizes
- Same row heights, padding, border radius, and icon sizing
- Same empty, hover, active, disabled, loading, and error states where implemented
- Screenshot comparison saved before commit

Run the current implemented-screen gate:

```text
cargo run -p xtask -- check-parity --allow-window-capture
```

The app defaults to `R3CODE_THEME=system`, which resolves from GPUI's OS window appearance. The parity gate forces `light` for screenshots that compare against the current light reference captures and forces `dark` for the dark empty-shell reference.
Native R3Code captures move the GPUI window off-screen immediately, drive clickable controls with window messages, and capture the GPU surface through Windows Graphics Capture so parity runs do not steal the foreground cursor or cover the active desktop.
The explicit `--allow-window-capture` flag is required so normal xtask usage cannot launch capture windows accidentally.

Run it with a fresh upstream reference capture:

```text
cargo run -p xtask -- check-parity --allow-window-capture --refresh-reference
```

## Current Empty-State Baseline

Reference: `reference/screenshots/upstream-empty-reference.png`

R3Code capture: `reference/screenshots/r3code-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --output reference\screenshots\r3code-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-empty-reference.png --actual reference\screenshots\r3code-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.571%`.

## Current Command Palette Baseline

Reference: `reference/screenshots/upstream-command-palette-reference.png`

R3Code capture: `reference/screenshots/r3code-command-palette-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen command-palette --output reference\screenshots\r3code-command-palette-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-command-palette-reference.png --actual reference\screenshots\r3code-command-palette-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `3.247%`.

The R3Code command palette capture launches the normal empty shell and opens the palette through the native sidebar click target before taking the screenshot. The Rust palette now ports the upstream `CommandPalette.logic.ts` search ranking, `>` action filter, root action groups, project/thread search item injection, recent-thread limit, archived-thread filtering, and the FolderPlus/Settings/SquarePen/MessageSquare icon set. Add-project source selection, filesystem browsing, remote clone flow, and full submenu chrome are still incomplete.

## Current Draft Chat Baseline

Reference: `reference/screenshots/upstream-draft-reference.png`

R3Code capture: `reference/screenshots/r3code-draft-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen draft --output reference\screenshots\r3code-draft-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-draft-reference.png --actual reference\screenshots\r3code-draft-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 1.75
```

Last measured result: `1.692%`.

The upstream draft reference is produced by the real T3 command-palette add-project flow, which creates a `/draft/$draftId` route and captures the active empty chat surface after dismissing unrelated provider-update toast chrome.

## Current Composer Focus Baseline

Reference: `reference/screenshots/upstream-composer-focused-reference.png`

R3Code capture: `reference/screenshots/r3code-composer-focused-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-focused --output reference\screenshots\r3code-composer-focused-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-composer-focused-reference.png --actual reference\screenshots\r3code-composer-focused-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 1.9
```

Last measured result: `1.790%`.

## Current Active Chat Baseline

Reference: `reference/screenshots/upstream-active-chat-reference.png`

R3Code capture: `reference/screenshots/r3code-active-chat-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen active-chat --output reference\screenshots\r3code-active-chat-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-active-chat-reference.png --actual reference\screenshots\r3code-active-chat-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4.1
```

Last measured result: `4.014%`.

The upstream active-chat reference seeds the real T3 Code app store with a deterministic server thread, then captures the actual `/$environmentId/$threadId` route from pinned commit `8fc317939f5c8bbef4afbe309ae897abbc221631`.
The R3Code sidebar now renders the seeded active thread as a project-scoped row under the matching project, with source-backed status pill priority and deterministic reference relative-time text. The message timeline now ports upstream content-sized user bubbles, assistant metadata ordering after changed-files cards, deterministic working-row timer formatting for seeded running states, and the active reference composer model defaults to upstream `gpt-5.4`. Multi-select, hover archive controls, context menus, show-more/show-less, live sidebar grouping, and live timer updates remain incomplete.

## Current Native Captures

R3Code composer command menu capture: `reference/screenshots/r3code-composer-menu-window.png`
R3Code composer inline-token capture: `reference/screenshots/r3code-composer-inline-tokens-window.png`
R3Code terminal drawer capture: `reference/screenshots/r3code-terminal-drawer-window.png`
R3Code diff panel capture: `reference/screenshots/r3code-diff-panel-window.png`
R3Code branch toolbar capture: `reference/screenshots/r3code-branch-toolbar-window.png`
R3Code sidebar options menu capture: `reference/screenshots/r3code-sidebar-options-menu-window.png`
R3Code project scripts menu capture: `reference/screenshots/r3code-project-scripts-menu-window.png`
R3Code Open In menu capture: `reference/screenshots/r3code-open-in-menu-window.png`
R3Code Git actions menu capture: `reference/screenshots/r3code-git-actions-menu-window.png`
R3Code provider/model picker capture: `reference/screenshots/r3code-provider-model-picker-window.png`

Command:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen active-chat --output reference\screenshots\r3code-active-chat-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-menu --output reference\screenshots\r3code-composer-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-inline-tokens --output reference\screenshots\r3code-composer-inline-tokens-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen running-turn --output reference\screenshots\r3code-running-turn-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-approval --output reference\screenshots\r3code-pending-approval-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-user-input --output reference\screenshots\r3code-pending-user-input-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen terminal-drawer --output reference\screenshots\r3code-terminal-drawer-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen diff-panel --output reference\screenshots\r3code-diff-panel-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen branch-toolbar --output reference\screenshots\r3code-branch-toolbar-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen sidebar-options-menu --output reference\screenshots\r3code-sidebar-options-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen project-scripts-menu --output reference\screenshots\r3code-project-scripts-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen open-in-menu --output reference\screenshots\r3code-open-in-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen git-actions-menu --output reference\screenshots\r3code-git-actions-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen provider-model-picker --output reference\screenshots\r3code-provider-model-picker-window.png
```

These are native Rust/GPUI captures for seeded chat states. Active chat, running turn, pending approval, pending user input, terminal drawer, diff panel, branch toolbar, sidebar options menu, project scripts menu, Open In menu, Git actions menu, provider/model picker, composer command menu, composer inline tokens, and focused composer are now covered by source-backed upstream reference comparisons.
The running turn capture ports the upstream work-log derivation shape first: activity ordering, ignored lifecycle rows, checkpoint filtering, simple tool update/completion collapse, command previews, changed-file previews, and the `WorkGroupSection` surface. It is not a live provider stream yet.
The Rust core also ports upstream `ChatView.logic.ts`, `terminalContext.ts`, `composer-editor-mentions.ts`, `composer-logic.ts`, `ComposerPromptEditor.tsx`, `composerInlineChip.ts`, `vscode-icons.ts`, `composerSlashCommandSearch.ts`, `composerMenuHighlight.ts`, `providerSkillSearch.ts`, and `providerSkillPresentation.ts` composer contracts: inline terminal-context placeholders, terminal-context block append/extract/display state, expired terminal-context filtering, expired-context toast copy, completed `@path` and `$skill` segment parsing, inline mention/skill chip rendering, terminal-context prompt segments, mention-boundary selection detection, collapsed/expanded cursor mapping, active `/`, `@`, and `$` trigger detection, standalone `/plan` and `/default` command parsing, text-range replacement, built-in and provider slash command search, provider skill search/presentation, composer menu grouping, active-item highlight reset, keyboard nudging, and command selection replacement behavior. The `composer-menu` GPUI capture renders the slash-command menu from those Rust contracts and is now compared against `upstream-composer-menu-reference.png` at a 4.5% threshold. The `composer-inline-tokens` GPUI capture renders completed `@AGENTS.md` and `$agent-browser` chips against a pinned upstream controlled-draft browser capture at a 2.2% threshold.

## Current Pending Approval Baseline

Reference: `reference/screenshots/upstream-pending-approval-reference.png`

R3Code capture: `reference/screenshots/r3code-pending-approval-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-approval --output reference\screenshots\r3code-pending-approval-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-pending-approval-reference.png --actual reference\screenshots\r3code-pending-approval-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `4.888%`.

The upstream pending-approval reference reuses the seeded active thread and injects two `approval.requested` activities matching the R3 command/file-change approval fixture, then captures the real upstream composer approval card and actions. R3 now keeps the active-chat changed-files card and working indicator visible in the pending-approval fixture before rendering the approval composer.

## Current Running Turn Baseline

Reference: `reference/screenshots/upstream-running-turn-reference.png`

R3Code capture: `reference/screenshots/r3code-running-turn-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen running-turn --output reference\screenshots\r3code-running-turn-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-running-turn-reference.png --actual reference\screenshots\r3code-running-turn-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4
```

Last measured result: `3.682%`.

The upstream running-turn reference reuses the seeded active thread, replaces it with a running `latestTurn`, one user message, and three source-backed work-log activities, then captures the real upstream timeline/work-log state.

## Current Pending User Input Baseline

Reference: `reference/screenshots/upstream-pending-user-input-reference.png`

R3Code capture: `reference/screenshots/r3code-pending-user-input-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-user-input --output reference\screenshots\r3code-pending-user-input-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-pending-user-input-reference.png --actual reference\screenshots\r3code-pending-user-input-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5.1
```

Last measured result: `4.963%`.

The upstream pending-user-input reference reuses the seeded active thread and injects the browser-test `user-input.requested` activity from `ChatView.browser.tsx`, then captures the real upstream composer question card. R3 now keeps the active-chat changed-files card and working indicator visible in the pending-user-input fixture before rendering the user-input composer. The Rust fixture also mirrors the upstream Plan + Full access pending footer, uses the same default `gpt-5.4` composer model for this capture, and uses a pending-input-specific editor height so the composer dividers align with the upstream reference.

## Current Composer Command Menu Baseline

Reference: `reference/screenshots/upstream-composer-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-composer-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-menu --output reference\screenshots\r3code-composer-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-composer-menu-reference.png --actual reference\screenshots\r3code-composer-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4.5
```

Last measured result: `4.346%`.

## Current Composer Inline Tokens Baseline

Reference: `reference/screenshots/upstream-composer-inline-tokens-reference.png`

R3Code capture: `reference/screenshots/r3code-composer-inline-tokens-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-inline-tokens --output reference\screenshots\r3code-composer-inline-tokens-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-composer-inline-tokens-reference.png --actual reference\screenshots\r3code-composer-inline-tokens-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2.2
```

Last measured result: `2.070%`.

## Current Terminal Drawer Baseline

Reference: `reference/screenshots/upstream-terminal-drawer-reference.png`

R3Code capture: `reference/screenshots/r3code-terminal-drawer-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen terminal-drawer --output reference\screenshots\r3code-terminal-drawer-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-terminal-drawer-reference.png --actual reference\screenshots\r3code-terminal-drawer-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.692%`.

The terminal drawer capture ports the upstream terminal UI state contract into Rust first: default drawer height, terminal IDs, active terminal/group, split/new/close behavior, running activity, event replay filtering, terminal-context prompt materialization, split pane/sidebar layout, and xterm-style snapshot history rendering. The GPUI surface is now compared against a deterministic upstream `ThreadTerminalDrawer` capture, but the native runtime/xterm layer is still incomplete.
## Current Diff Panel Baseline

Reference: `reference/screenshots/upstream-diff-panel-reference.png`

R3Code capture: `reference/screenshots/r3code-diff-panel-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen diff-panel --output reference\screenshots\r3code-diff-panel-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-diff-panel-reference.png --actual reference\screenshots\r3code-diff-panel-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 8.8
```

Last measured result: `8.669%`.

The diff panel capture ports the upstream diff route parser, turn-diff summary ordering, changed-file tree/stat contracts, inline `DiffPanelShell` header controls, selected-turn/selected-file route state, path-normalized file ordering, `@pierre/diffs`-style panel width, compact composer footer/form width, compact Mode/Access menu, file header height, modified-file icon color, metadata stat shape/order, hunk separator spacing, unified single-gutter patch rows, and a deterministic parsed-patch-style syntax palette. It is now compared against a seeded upstream `DiffPanel` reference, but the Rust side still lacks the real checkpoint-diff query, compact footer traits/plan-sidebar behavior, and the full `@pierre/diffs` renderer. The 8.8% threshold is a provisional gate for the simplified Rust patch renderer and should tighten after the renderer is replaced with a closer `@pierre/diffs` equivalent.
## Current Branch Toolbar Baseline

Reference: `reference/screenshots/upstream-branch-toolbar-reference.png`

R3Code capture: `reference/screenshots/r3code-branch-toolbar-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen branch-toolbar --output reference\screenshots\r3code-branch-toolbar-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-branch-toolbar-reference.png --actual reference\screenshots\r3code-branch-toolbar-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 3
```

Last measured result: `2.614%`.

The branch toolbar capture ports upstream `BranchToolbar.tsx`, `BranchToolbar.logic.ts`, `BranchToolbarEnvModeSelector.tsx`, `BranchToolbarBranchSelector.tsx`, the shared remote-branch dedupe helpers, environment/worktree labels, current/new-worktree mode resolution, branch trigger text, worktree selection target rules, and lucide `Monitor`, `Cloud`, `FolderGit`, and `FolderGit2` assets. It is now compared against a seeded upstream draft-route worktree toolbar reference, but the full combobox, async git ref query, create-ref, PR checkout, and real environment switching paths remain missing.
The branch/header chrome also now includes a source-backed static `GitActionsControl` quick action group so the surrounding action order matches the pinned upstream references; dialogs, publish flow, progress toasts, and live git mutations are still missing.

## Current Sidebar Options Menu Baseline

Reference: `reference/screenshots/upstream-sidebar-options-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-sidebar-options-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen sidebar-options-menu --output reference\screenshots\r3code-sidebar-options-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-sidebar-options-menu-reference.png --actual reference\screenshots\r3code-sidebar-options-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 3.7
```

Last measured result: `3.533%`.

The sidebar options menu capture ports upstream `Sidebar.tsx` and `packages/contracts/src/settings.ts` defaults for project sort order, thread sort order, visible thread preview count, and project grouping labels. It now compares the native GPUI menu against the pinned upstream menu opened from the Projects header, including default selected radio rows and the focused visible-thread count control. Real project reordering, grouped-project override dialogs, drag sorting, and persisted settings are still missing.

## Current Git Actions Menu Baseline

Reference: `reference/screenshots/upstream-git-actions-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-git-actions-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen git-actions-menu --output reference\screenshots\r3code-git-actions-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-git-actions-menu-reference.png --actual reference\screenshots\r3code-git-actions-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 3
```

Last measured result: `2.894%`.

The Git actions menu capture ports upstream `GitActionsControl.tsx`, `GitActionsControl.logic.ts`, and `sourceControlPresentation.ts` for the seeded detached-HEAD menu state: disabled Commit/Push/Create PR rows, the `CloudUpload` push icon, change-request PR icon mapping, popup width/alignment, and the detached refName warning. It is now compared against a pinned upstream draft-route menu reference, but the real commit/push/PR dialogs, publish repository path, disabled-reason tooltips, source-control refresh, and live git mutations are still missing.

## Current Project Scripts And Open In Menu Baselines

Project scripts reference: `reference/screenshots/upstream-project-scripts-menu-reference.png`

Project scripts R3Code capture: `reference/screenshots/r3code-project-scripts-menu-window.png`

Open In reference: `reference/screenshots/upstream-open-in-menu-reference.png`

Open In R3Code capture: `reference/screenshots/r3code-open-in-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diffs:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen project-scripts-menu --output reference\screenshots\r3code-project-scripts-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-project-scripts-menu-reference.png --actual reference\screenshots\r3code-project-scripts-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4.2

cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen open-in-menu --output reference\screenshots\r3code-open-in-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-open-in-menu-reference.png --actual reference\screenshots\r3code-open-in-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 3
```

Last measured results: project scripts menu `4.039%`; Open In menu `2.575%`.

The project script/open-in header controls port upstream `projectScripts.ts`, `ProjectScriptsControl.tsx` primary/add button behavior, `OpenInPicker.tsx` visibility rules, editor option labels, preferred-editor shortcut display, script runtime cwd/env helpers, and the script/editor icon assets visible in the seeded references. The project scripts menu is now compared against a pinned active-chat menu reference and the Open In menu is compared against a pinned draft-route editor menu reference. The add/edit/delete dialogs, keybinding capture UI, full custom editor icon set, real shell open-in calls, and real shell/project-script process execution are still missing.
The provider/model picker capture ports upstream `ProviderModelPicker.tsx`, `ModelPickerContent.tsx`, `ModelPickerSidebar.tsx`, `providerInstances.ts`, `modelOrdering.ts`, `modelPickerSearch.ts`, provider trigger labels, duplicate-instance badges, active-instance sidebar selection, locked-provider filtering, selectable-model aliases, the seeded provider rail order, and the provider/lucide/coming-soon icon set used by the picker. It is now compared against a pinned upstream draft-route picker capture. Live provider snapshots, editable provider settings, real favorites persistence, and full combobox/input behavior are still incomplete.

## Current Provider Model Picker Baseline

Reference: `reference/screenshots/upstream-provider-model-picker-reference.png`

R3Code capture: `reference/screenshots/r3code-provider-model-picker-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen provider-model-picker --output reference\screenshots\r3code-provider-model-picker-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-provider-model-picker-reference.png --actual reference\screenshots\r3code-provider-model-picker-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4.45
```

Last measured result: `4.369%`.

## Current Settings Baseline

Reference: `reference/screenshots/upstream-settings-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings --output reference\screenshots\r3code-settings-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-reference.png --actual reference\screenshots\r3code-settings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.792%`.

The settings sidebar renders the upstream settings nav icon set through GPUI SVG assets:
`Settings2`, `Keyboard`, `Bot`, `GitBranch`, `Link2`, `Archive`, and the footer `ArrowLeft`.
The footer Back affordance is a native GPUI click target, and the parity gate follows that click path for settings-to-empty verification.
The settings nav rows are native GPUI click targets; the parity gate opens Keybindings through the native settings nav click path.
The parity gate also compares provider, connections, and diagnostics settings screens against pinned upstream references at `reference/screenshots/upstream-settings-providers-reference.png`, `reference/screenshots/upstream-settings-connections-reference.png`, and `reference/screenshots/upstream-settings-diagnostics-reference.png`.
The Rust core now ports the deterministic settings helpers from upstream `ConnectionsSettings.tsx`, `pairingUrls.ts`, `pairingUrl.ts`, `hostedPairing.ts`, `DiagnosticsSettings.tsx`, `SettingsPanels.logic.ts`, server `ProcessDiagnostics.ts`, and server `TraceDiagnostics.ts`: manual SSH target parsing, remote pairing URL/field parsing, desktop SSH error normalization, advertised endpoint preference/selection and pairing URL resolution, pairing/client-session sorting, diagnostics count/duration/byte/trace formatting, stale process signal detection, diagnostics description collapse rules, POSIX process row parsing, Windows process JSON normalization, descendant process aggregation, trace NDJSON aggregation, trace parse errors, slow-span/common-failure/latest-log buckets, rotated trace path ordering, totals, and diagnostics-query filtering. The general settings diagnostics row consumes the Rust diagnostics description helper; the connections/diagnostics UI still needs live saved-environment state, owner pairing links, client sessions, discovered SSH hosts, Tailscale Serve controls, process action wiring, trace file read wiring, and deeper live-state upstream fixture captures.

## Current Settings Providers Baseline

Reference: `reference/screenshots/upstream-settings-providers-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-providers-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-providers --output reference\screenshots\r3code-settings-providers-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-providers-reference.png --actual reference\screenshots\r3code-settings-providers-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `4.228%`.

## Current Settings Keybindings Baseline

Reference: `reference/screenshots/upstream-settings-keybindings-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-keybindings-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-keybindings --output reference\screenshots\r3code-settings-keybindings-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-keybindings-reference.png --actual reference\screenshots\r3code-settings-keybindings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6.5
```

Last measured result: `6.350%`.

Current ported parity scope: native keybindings table now follows the upstream `SettingsSection` table proportions more closely: upstream `InfoIcon` warning banner, T3 Code warning copy, `muted/25` and `muted/15` table backgrounds, `border/70` and `border/60` dividers, upstream grid column math, uppercase table headers, upstream Windows shortcut glyph labels, upstream `Kbd` chip sizing/color/weight, upstream `border-input` when triggers, and the default keybinding rows now come from a Rust core projection of upstream command ids/default rules instead of a UI-only display string table.

## Current Settings Source Control Baseline

Reference: `reference/screenshots/upstream-settings-source-control-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-source-control-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-source-control --output reference\screenshots\r3code-settings-source-control-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-source-control-reference.png --actual reference\screenshots\r3code-settings-source-control-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `3.441%`.

## Current Settings Connections Baseline

Reference: `reference/screenshots/upstream-settings-connections-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-connections-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-connections --output reference\screenshots\r3code-settings-connections-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-connections-reference.png --actual reference\screenshots\r3code-settings-connections-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 4
```

Last measured result: `2.854%`.

## Current Settings Diagnostics Baseline

Reference: `reference/screenshots/upstream-settings-diagnostics-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-diagnostics-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-diagnostics --output reference\screenshots\r3code-settings-diagnostics-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-diagnostics-reference.png --actual reference\screenshots\r3code-settings-diagnostics-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `4.235%`.

## Current Settings Archive Baseline

Reference: `reference/screenshots/upstream-settings-archive-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-archive-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-archive --output reference\screenshots\r3code-settings-archive-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-archive-reference.png --actual reference\screenshots\r3code-settings-archive-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `1.181%`.

## Current Settings Back Baseline

Reference: `reference/screenshots/upstream-empty-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-back-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-back --output reference\screenshots\r3code-settings-back-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-empty-reference.png --actual reference\screenshots\r3code-settings-back-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.571%`.

## Current Settings Theme Menu Baseline

Reference: `reference/screenshots/upstream-settings-theme-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-theme-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-theme-menu --output reference\screenshots\r3code-settings-theme-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-theme-menu-reference.png --actual reference\screenshots\r3code-settings-theme-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.798%`.

The R3Code capture opens the settings route in forced light mode, opens the native GPUI theme select with the settings keyboard path, and then screenshots the open `System / Light / Dark` popup. The reference selects `Light` before opening the menu so both screenshots compare the same selected value.

## Current Settings Dark Selection Baseline

Reference: `reference/screenshots/upstream-settings-dark-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-dark-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-dark --output reference\screenshots\r3code-settings-dark-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-dark-reference.png --actual reference\screenshots\r3code-settings-dark-window.png --channel-tolerance 11 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.275%`.

The dark settings comparison uses a slightly higher channel tolerance because the Chromium reference and GPUI render the same dark text with different subpixel antialiasing, while the layout and pixel-percent gate remain unchanged.

The R3Code capture opens settings from forced light mode, opens the native theme select with the settings keyboard path, moves from `Light` to `Dark` with one Down arrow press, selects it with `Enter`, and screenshots the dark settings surface.

## Current Dark Empty-State Baseline

Reference: `reference/screenshots/upstream-empty-dark-reference.png`

R3Code capture: `reference/screenshots/r3code-dark-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme dark --output reference\screenshots\r3code-dark-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-empty-dark-reference.png --actual reference\screenshots\r3code-dark-window.png --channel-tolerance 11 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.313%`.

## Implementation Rule

Build static GPUI screens from mock state before wiring real providers, git, terminal, or persistence. If the static screen does not match, functionality work waits.

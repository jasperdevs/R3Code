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
- `reference/screenshots/upstream-command-palette-reference.png`
- `reference/screenshots/upstream-draft-reference.png`
- `reference/screenshots/upstream-composer-menu-reference.png`
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

The app defaults to `R3CODE_THEME=system`, which resolves from GPUI's OS window appearance. The parity gate forces `light` for screenshots that compare against the current light reference captures and also captures a dark R3Code smoke screenshot.
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
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-draft-reference.png --actual reference\screenshots\r3code-draft-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.814%`.

The upstream draft reference is produced by the real T3 command-palette add-project flow, which creates a `/draft/$draftId` route and captures the active empty chat surface after dismissing unrelated provider-update toast chrome.

## Current Active Chat Smoke Baseline

R3Code capture: `reference/screenshots/r3code-active-chat-window.png`
R3Code composer command menu capture: `reference/screenshots/r3code-composer-menu-window.png`
R3Code running turn capture: `reference/screenshots/r3code-running-turn-window.png`
R3Code pending approval capture: `reference/screenshots/r3code-pending-approval-window.png`
R3Code pending user input capture: `reference/screenshots/r3code-pending-user-input-window.png`
R3Code terminal drawer capture: `reference/screenshots/r3code-terminal-drawer-window.png`
R3Code diff panel capture: `reference/screenshots/r3code-diff-panel-window.png`
R3Code branch toolbar capture: `reference/screenshots/r3code-branch-toolbar-window.png`
R3Code provider/model picker capture: `reference/screenshots/r3code-provider-model-picker-window.png`

Command:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen active-chat --output reference\screenshots\r3code-active-chat-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-menu --output reference\screenshots\r3code-composer-menu-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen running-turn --output reference\screenshots\r3code-running-turn-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-approval --output reference\screenshots\r3code-pending-approval-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen pending-user-input --output reference\screenshots\r3code-pending-user-input-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen terminal-drawer --output reference\screenshots\r3code-terminal-drawer-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen diff-panel --output reference\screenshots\r3code-diff-panel-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen branch-toolbar --output reference\screenshots\r3code-branch-toolbar-window.png
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen provider-model-picker --output reference\screenshots\r3code-provider-model-picker-window.png
```

This is a native Rust/GPUI smoke capture for the first server-thread state: active header title/project badge, project script and open-in controls, sidebar thread row, user/assistant timeline rows, changed-files summary tree, and composer chrome. It is intentionally not a parity comparison yet because the current upstream reference harness has deterministic captures for empty/draft/settings, but not a full active server-thread route with persisted messages. The next parity step is to add a source-backed upstream fixture capture for the same `ChatView.browser.tsx` message state.
The running turn capture ports the upstream work-log derivation shape first: activity ordering, ignored lifecycle rows, checkpoint filtering, simple tool update/completion collapse, command previews, changed-file previews, and the `WorkGroupSection` smoke surface. It is not a live provider stream yet.
The pending approval and pending user input captures use the same active-thread shell with upstream-shaped `ChatComposer` state: approval summary/actions, user-input question progress, option shortcut chips, selected `CheckIcon`, and pending primary actions. They are also smoke captures until the upstream browser harness can seed the matching server-thread pending request fixtures.
The Rust core also ports upstream `ChatView.logic.ts`, `terminalContext.ts`, `composer-editor-mentions.ts`, `composer-logic.ts`, `composerSlashCommandSearch.ts`, `composerMenuHighlight.ts`, `providerSkillSearch.ts`, and `providerSkillPresentation.ts` composer contracts: inline terminal-context placeholders, terminal-context block append/extract/display state, expired terminal-context filtering, expired-context toast copy, completed `@path` and `$skill` segment parsing, terminal-context prompt segments, mention-boundary selection detection, collapsed/expanded cursor mapping, active `/`, `@`, and `$` trigger detection, standalone `/plan` and `/default` command parsing, text-range replacement, built-in and provider slash command search, provider skill search/presentation, composer menu grouping, active-item highlight reset, keyboard nudging, and command selection replacement behavior. The `composer-menu` GPUI capture renders the slash-command menu from those Rust contracts and is now compared against `upstream-composer-menu-reference.png` at a 5% threshold.

## Current Composer Command Menu Baseline

Reference: `reference/screenshots/upstream-composer-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-composer-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen composer-menu --output reference\screenshots\r3code-composer-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-composer-menu-reference.png --actual reference\screenshots\r3code-composer-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `4.468%`.
The terminal drawer capture ports the upstream terminal UI state contract into Rust first: default drawer height, terminal IDs, active terminal/group, split/new/close behavior, running activity, event replay filtering, and terminal-context prompt materialization. The GPUI surface is a static terminal drawer smoke screen until the native runtime/xterm layer exists.
The diff panel capture ports the upstream diff route parser, turn-diff summary ordering, changed-file tree/stat contracts, and the inline `DiffPanelShell` header controls. It remains a smoke screen until the Rust side has the real checkpoint-diff query and patch renderer equivalent to `@pierre/diffs`.
The branch toolbar capture ports upstream `BranchToolbar.logic.ts`, the shared remote-branch dedupe helpers, the environment/worktree labels, current/new-worktree mode resolution, branch trigger text, worktree selection target rules, and lucide `Monitor`, `Cloud`, `FolderGit`, and `FolderGit2` assets. It is still a GPUI smoke screen: the full combobox, async git ref query, create-ref, PR checkout, and real environment switching paths remain missing.
The project script/open-in header controls port upstream `projectScripts.ts`, `ProjectScriptsControl.tsx` primary/add button behavior, `OpenInPicker.tsx` visibility rules, editor option labels, script runtime cwd/env helpers, and the lucide script icon set. The add/edit/delete dialogs, custom editor SVG icon set, keybinding capture UI, and real shell/project-script process execution are still missing.
The provider/model picker capture ports upstream `ProviderModelPicker.tsx`, `ModelPickerContent.tsx`, `providerInstances.ts`, `modelOrdering.ts`, `modelPickerSearch.ts`, provider trigger labels, duplicate-instance badges, favorites/default sidebar selection, locked-provider filtering, selectable-model aliases, and the provider/lucide icon set used by the picker. It remains a GPUI smoke screen until the Rust app has live provider snapshots, editable provider settings, real favorites persistence, and a full combobox/input implementation.

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
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-keybindings-reference.png --actual reference\screenshots\r3code-settings-keybindings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 9
```

Last measured result: `8.739%`.

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

## Implementation Rule

Build static GPUI screens from mock state before wiring real providers, git, terminal, or persistence. If the static screen does not match, functionality work waits.

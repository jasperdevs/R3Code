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
- `reference/screenshots/upstream-settings-reference.png`
- `reference/screenshots/upstream-settings-keybindings-reference.png`
- `reference/screenshots/upstream-settings-source-control-reference.png`
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

Last measured result: `1.565%`.

## Current Command Palette Baseline

Reference: `reference/screenshots/upstream-command-palette-reference.png`

R3Code capture: `reference/screenshots/r3code-command-palette-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen command-palette --output reference\screenshots\r3code-command-palette-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-command-palette-reference.png --actual reference\screenshots\r3code-command-palette-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 5
```

Last measured result: `3.267%`.

The R3Code command palette capture launches the normal empty shell and opens the palette through the native sidebar click target before taking the screenshot.

## Current Draft Chat Baseline

Reference: `reference/screenshots/upstream-draft-reference.png`

R3Code capture: `reference/screenshots/r3code-draft-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen draft --output reference\screenshots\r3code-draft-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-draft-reference.png --actual reference\screenshots\r3code-draft-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 3
```

Last measured result: `2.241%`.

The upstream draft reference is produced by the real T3 command-palette add-project flow, which creates a `/draft/$draftId` route and captures the active empty chat surface after dismissing unrelated provider-update toast chrome.

## Current Settings Baseline

Reference: `reference/screenshots/upstream-settings-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings --output reference\screenshots\r3code-settings-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-reference.png --actual reference\screenshots\r3code-settings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.781%`.

The settings sidebar renders the upstream settings nav icon set through GPUI SVG assets:
`Settings2`, `Keyboard`, `Bot`, `GitBranch`, `Link2`, `Archive`, and the footer `ArrowLeft`.
The footer Back affordance is a native GPUI click target, and the parity gate follows that click path for settings-to-empty verification.
The settings nav rows are native GPUI click targets; the parity gate opens Keybindings through the native settings nav click path.

## Current Settings Keybindings Baseline

Reference: `reference/screenshots/upstream-settings-keybindings-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-keybindings-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-keybindings --output reference\screenshots\r3code-settings-keybindings-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-keybindings-reference.png --actual reference\screenshots\r3code-settings-keybindings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 10
```

Last measured result: `8.794%`.

## Current Settings Source Control Baseline

Reference: `reference/screenshots/upstream-settings-source-control-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-source-control-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-source-control --output reference\screenshots\r3code-settings-source-control-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-source-control-reference.png --actual reference\screenshots\r3code-settings-source-control-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `3.507%`.

## Current Settings Archive Baseline

Reference: `reference/screenshots/upstream-settings-archive-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-archive-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-archive --output reference\screenshots\r3code-settings-archive-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-archive-reference.png --actual reference\screenshots\r3code-settings-archive-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `1.247%`.

## Current Settings Back Baseline

Reference: `reference/screenshots/upstream-empty-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-back-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-back --output reference\screenshots\r3code-settings-back-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-empty-reference.png --actual reference\screenshots\r3code-settings-back-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.565%`.

## Current Settings Theme Menu Baseline

Reference: `reference/screenshots/upstream-settings-theme-menu-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-theme-menu-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --allow-window-capture --theme light --screen settings-theme-menu --output reference\screenshots\r3code-settings-theme-menu-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\upstream-settings-theme-menu-reference.png --actual reference\screenshots\r3code-settings-theme-menu-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.787%`.

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

Last measured result: `5.264%`.

The dark settings comparison uses a slightly higher channel tolerance because the Chromium reference and GPUI render the same dark text with different subpixel antialiasing, while the layout and pixel-percent gate remain unchanged.

The R3Code capture opens settings from forced light mode, opens the native theme select with the settings keyboard path, moves from `Light` to `Dark` with one Down arrow press, selects it with `Enter`, and screenshots the dark settings surface.

## Implementation Rule

Build static GPUI screens from mock state before wiring real providers, git, terminal, or persistence. If the static screen does not match, functionality work waits.

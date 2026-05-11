# UI Parity Plan

R3Code should be judged against frozen T3Code reference screenshots, not against taste or memory.

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

Use the Rust xtask to launch the frozen upstream T3Code checkout with an isolated `T3CODE_HOME`, capture the currently automated reference screens, and stop the watcher process tree:

```text
cargo run -p xtask -- capture-t3code-browser
```

The task currently captures:

- `reference/screenshots/t3code-empty-reference.png`
- `reference/screenshots/t3code-settings-reference.png`

Do not use screenshots from a different upstream commit unless `docs/reference/T3CODE_VERSION.md` is intentionally updated.

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
cargo run -p xtask -- check-parity
```

The app defaults to `R3CODE_THEME=system`, which resolves from GPUI's OS window appearance. The parity gate forces `light` for screenshots that compare against the current light T3Code references and also captures a dark R3Code smoke screenshot.

Run it with a fresh upstream T3Code capture:

```text
cargo run -p xtask -- check-parity --refresh-t3code-reference
```

## Current Empty-State Baseline

Reference: `reference/screenshots/t3code-empty-reference.png`

R3Code capture: `reference/screenshots/r3code-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --theme light --output reference\screenshots\r3code-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\t3code-empty-reference.png --actual reference\screenshots\r3code-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 2
```

Last measured result: `1.557%`.

## Current Settings Baseline

Reference: `reference/screenshots/t3code-settings-reference.png`

R3Code capture: `reference/screenshots/r3code-settings-window.png`

Allowed brand-copy difference: `--ignore-rect 0,0,120,45`

Current measured diff:

```text
cargo run -p xtask -- capture-r3code-window --theme light --screen settings --output reference\screenshots\r3code-settings-window.png
cargo run -p xtask -- compare-screenshots --expected reference\screenshots\t3code-settings-reference.png --actual reference\screenshots\r3code-settings-window.png --channel-tolerance 8 --ignore-rect 0,0,120,45 --max-different-pixels-percent 6
```

Last measured result: `5.108%`.

## Implementation Rule

Build static GPUI screens from mock state before wiring real providers, git, terminal, or persistence. If the static screen does not match, functionality work waits.

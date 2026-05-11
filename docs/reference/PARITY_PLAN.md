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

## Current Empty-State Baseline

Reference: `reference/screenshots/t3code-pair-reference.png`

R3Code capture: `reference/screenshots/r3code-window.png`

Allowed brand-copy difference: `-IgnoreRect 0,0,120,45`

Current measured diff:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\compare-screenshots.ps1 -Expected reference\screenshots\t3code-pair-reference.png -Actual reference\screenshots\r3code-window.png -ChannelTolerance 8 -IgnoreRect 0,0,120,45 -MaxDifferentPixelsPercent 2
```

Last measured result: `1.557%`.

## Implementation Rule

Build static GPUI screens from mock state before wiring real providers, git, terminal, or persistence. If the static screen does not match, functionality work waits.

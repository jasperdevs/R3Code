# R3Code Port Gap Audit

R3Code is still an early static Rust/GPUI shell, not a full Rust port of T3 Code.

Reference commit: `8fc317939f5c8bbef4afbe309ae897abbc221631`

Current local baseline: R3Code `main` after the keybindings logic-contract projection, provider/model picker rail, pending-user-input composer footer, and diff-panel compact-composer/renderer parity slices.

## Size Check

Measured source inventory:

| Surface | Source files |
| --- | ---: |
| R3Code Rust crates (`crates/**/*.rs`) | 7 |
| T3 Code apps/packages (`apps`, `packages`, TS/TSX only) | 964 |
| T3 Code web app | 374 |
| T3 Code server app | 397 |
| T3 Code desktop app | 71 |
| T3 Code packages | 121 |

This is not a line-for-line progress metric, but it is a useful scope warning: most of the T3 application is still not represented in Rust.

## What Is Ported Enough To Gate

These screens have automated reference captures and native GPUI comparisons:

| Screen | Current gate |
| --- | ---: |
| Empty/no active thread | 2% |
| Command palette | 5% |
| Draft empty chat | 1.75% |
| Composer focused empty state | 1.9% |
| Active chat with user and assistant messages | 4.1% |
| Running turn with work-log rows | 4% |
| Terminal drawer split view | 6% |
| Diff panel selected turn patch view | 8.8% |
| Branch toolbar draft worktree state | 3% |
| Sidebar options menu | 3.7% |
| Project scripts action menu | 4.2% |
| Open In editor picker menu | 3% |
| Git actions options menu | 3% |
| Composer slash-command menu | 4.5% |
| Composer inline mention/skill chips | 2.2% |
| Provider/model picker | 4.4% |
| Pending approval composer state | 5% |
| Pending user input composer state | 5.1% |
| Settings general | 6% |
| Settings keybindings | 6.35% |
| Settings providers | 5% |
| Settings source control | 6% |
| Settings connections | 4% |
| Settings diagnostics | 5% |
| Settings archive | 6% |
| Settings back navigation | 2% |
| Settings theme menu | 6% |
| Settings dark selection | 6% |
| Empty/no active thread, dark theme | 2% |

The strongest parity areas are the empty shell, draft chat chrome, archive empty state, and settings back path. The weakest implemented area is now the diff panel renderer, followed by Keybindings; the diff panel gate is tightened to 8.8% after source-backed row geometry, stat-label, compact composer, and syntax-palette improvements, while the simplified renderer still carries browser-vs-GPUI and missing `@pierre/diffs` differences. The active-chat gate is tightened to 4.1% after the seeded active reference model matched upstream `gpt-5.4`. The draft-family gates now hide project-only header chrome in draft references, lowering draft to 1.75%, focused composer to 1.9%, composer menu to 4.5%, inline composer tokens to 2.2%, and provider/model picker to 4.4%. Keybindings now ports upstream `Kbd` chip sizing/color/weight, `border-input` when triggers, upstream light `foreground`, and the Rust equivalents of `KeybindingsSettings.logic.ts` shortcut/when/source/conflict/edit-draft contracts, lowering the gate to 6.35%. The pending-user-input composer now restores the upstream footer controls and tighter editor height, with the gate reduced to 5.1%.

## Major Missing Product Surfaces

These T3 surfaces have no complete Rust implementation yet:

| Upstream surface | Main source area | R3 state |
| --- | --- | --- |
| Active chat with real user/assistant turns | `apps/web/src/components/ChatView.tsx`, `MessagesTimeline.tsx` | Seeded upstream reference gate for the message shell, content-sized user bubbles, assistant changed-files metadata ordering, and project-scoped sidebar row; live thread/runtime behavior still missing |
| Running agent turn, work log, tool output | `apps/web/src/session-logic.ts`, `MessagesTimeline.tsx` | Seeded upstream reference gate for running session/work-log shell plus deterministic working indicator; live provider stream and tool output runtime still missing |
| Pending approval and pending user input panels | `ChatComposer.tsx`, `ComposerPendingApproval*`, `pendingUserInput.ts` | Seeded upstream reference gates for approval and user-input composer states with active-thread changed-files and working indicator carried through. Core pending-input, composer send-state, trigger/cursor, command-menu, slash-menu, and inline-token contracts are partial |
| Terminal drawer and xterm integration | `ThreadTerminalDrawer.tsx`, `terminalStateStore.ts`, `terminalContext.ts` | Seeded upstream reference gate for the split drawer shell plus terminal state, event replay, terminal-context, and composer inline-token contracts; live terminal runtime/xterm backend still missing |
| Diff panel and changed-file browsing | `DiffPanel.tsx`, `DiffPanelShell.tsx`, `diffRouteSearch.ts`, `turnDiffTree.ts` | Seeded upstream reference gate for selected-turn patch view with source-backed shell width, compact composer form/footer/menu, file header, stat shape/order, hunk spacing, syntax palette, and unified-row renderer parity; route/tree/header contracts are partial and real checkpoint-diff query plus full `@pierre/diffs` rendering are still missing |
| Branch/worktree toolbar | `BranchToolbar.tsx`, `BranchToolbar.logic.ts`, `BranchToolbarEnvModeSelector.tsx`, `BranchToolbarBranchSelector.tsx` | Seeded upstream reference gate for draft worktree toolbar state; full combobox, async git ref query, create-ref, PR checkout, and real environment switching paths remain missing |
| Provider/model picker behavior | `ProviderModelPicker.tsx`, `ModelPickerContent.tsx`, `providerModels.ts` | Partial core logic + GPUI picker comparison gate |
| Project scripts, open-in-editor picker, and Git header actions | `ProjectScriptsControl.tsx`, `projectScripts.ts`, `OpenInPicker.tsx`, `GitActionsControl.tsx`, `GitActionsControl.logic.ts` | Seeded upstream menu gates plus partial core/header logic; dialogs, disabled-reason tooltips, full editor/icon set, real process/editor execution, and live git mutations are still missing |
| Settings providers/connections diagnostics depth | `settings/*`, `Provider*`, `ConnectionsSettings.tsx`, `DiagnosticsSettings.tsx`, `ProcessDiagnostics.ts`, `TraceDiagnostics.ts` | Partial provider, pairing, endpoint, process/trace diagnostics, diagnostics-format contracts, and top-level provider/connections/diagnostics comparison gates |
| Command palette real actions/search | `CommandPalette.tsx`, `CommandPaletteResults.tsx`, `CommandPalette.logic.ts` | Partial core logic + dynamic GPUI groups |
| Sidebar real grouping, selection, archival actions | `Sidebar.tsx`, `uiStateStore.ts`, `threadSelectionStore.ts` | Seeded sidebar options menu gate plus source-backed project-scoped thread rows/status pills for deterministic references; real grouping snapshots, multi-select, context menus, hover archive controls, show-more/show-less, drag sorting, archive/delete actions, and persisted settings are still missing |

## Major Missing Runtime/Backend Layers

These upstream backend/runtime areas have no Rust equivalent yet:

| Upstream area | Files/modules | R3 state |
| --- | --- | --- |
| Server HTTP/WebSocket API | `apps/server/src/http.ts`, `ws.ts`, generated contracts | Missing |
| Provider orchestration | `apps/server/src/provider`, `orchestration`, `textGeneration` | Missing |
| Persistence and migrations | `apps/server/src/persistence` | Missing |
| Project discovery/setup/scripts | `apps/server/src/project`, `workspace` | Missing |
| Git/source control/PR workflow | `apps/server/src/sourceControl`, `git`, `vcs` | Header/menu state and seeded detached-HEAD gate only; runtime git status refresh, commit/push/PR dialogs, publish repository flow, and source-control backend are still missing |
| Terminal process management | `apps/server/src/terminal`, `processRunner.ts` | Missing runtime, UI state contracts only |
| Auth/pairing/saved environments | `apps/server/src/auth`, `apps/web/src/environments` | Missing |
| Desktop IPC/menu/bootstrap | `apps/desktop/src` | Missing |
| Shared contracts and generated schemas | `packages/contracts`, `effect-codex-app-server` | Small hand-written subset only |

## Immediate Port Order

1. Keep expanding deterministic visual references before porting behavior for each surface.
2. Port the Rust data contracts that unblock those surfaces: scoped refs, thread/project/session/provider state, keybindings, terminal state, diff route state.
3. Replace static GPUI panels with state-driven panels one screen at a time.
4. Add real runtime layers after the matching static surface exists and is gated.

The current port is visually promising for the gated static shell, but it is far from a full T3 Code port. The next meaningful milestones should deepen the seeded reference gates and add the missing runtime-backed behavior, especially real provider streams, checkpoint diff retrieval/rendering, git/project actions, sidebar state, and backend contracts.

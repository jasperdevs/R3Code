# R3Code Port Gap Audit

R3Code is still an early static Rust/GPUI shell, not a full Rust port of T3 Code.

Reference commit: `8fc317939f5c8bbef4afbe309ae897abbc221631`

Current local baseline: R3Code `main` after the branch/worktree toolbar state slice.

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
| Draft empty chat | 2% |
| Composer focused empty state | 2% |
| Active chat with user and assistant messages | 6% |
| Running turn with work-log rows | 6% |
| Composer slash-command menu | 5% |
| Composer inline mention/skill chips | 5% |
| Provider/model picker | 6% |
| Pending approval composer state | 6% |
| Pending user input composer state | 7% |
| Settings general | 6% |
| Settings keybindings | 9% |
| Settings providers | 5% |
| Settings source control | 6% |
| Settings connections | 4% |
| Settings diagnostics | 5% |
| Settings archive | 6% |
| Settings back navigation | 2% |
| Settings theme menu | 6% |
| Settings dark selection | 6% |
| Empty/no active thread, dark theme | 2% |

The strongest parity areas are the empty shell, draft chat chrome, archive empty state, and settings back path. The weakest implemented area is still Keybindings because it is a dense editable table with browser-vs-GPUI font, border, and control rendering differences.

## Major Missing Product Surfaces

These T3 surfaces have no complete Rust implementation yet:

| Upstream surface | Main source area | R3 state |
| --- | --- | --- |
| Active chat with real user/assistant turns | `apps/web/src/components/ChatView.tsx`, `MessagesTimeline.tsx` | Seeded upstream reference gate for the message shell; live thread/runtime behavior still missing |
| Running agent turn, work log, tool output | `apps/web/src/session-logic.ts`, `MessagesTimeline.tsx` | Seeded upstream reference gate for running session/work-log shell; live provider stream and tool output runtime still missing |
| Pending approval and pending user input panels | `ChatComposer.tsx`, `ComposerPendingApproval*`, `pendingUserInput.ts` | Seeded upstream reference gates for approval and user-input composer states. Core pending-input, composer send-state, trigger/cursor, command-menu, slash-menu, and inline-token contracts are partial |
| Terminal drawer and xterm integration | `ThreadTerminalDrawer.tsx`, `terminalStateStore.ts`, `terminalContext.ts` | Partial static GPUI smoke plus terminal drawer, terminal-context, and composer inline-token contracts |
| Diff panel and changed-file browsing | `DiffPanel.tsx`, `diffRouteSearch.ts`, `turnDiffTree.ts` | Partial static GPUI smoke |
| Branch/worktree toolbar | `BranchToolbar.tsx`, `BranchToolbar.logic.ts`, `BranchToolbarBranchSelector.tsx` | Partial core logic + GPUI smoke |
| Provider/model picker behavior | `ProviderModelPicker.tsx`, `ModelPickerContent.tsx`, `providerModels.ts` | Partial core logic + GPUI picker comparison gate |
| Project scripts and open-in-editor picker | `ProjectScriptsControl.tsx`, `projectScripts.ts`, `OpenInPicker.tsx` | Partial core logic + header smoke |
| Settings providers/connections diagnostics depth | `settings/*`, `Provider*`, `ConnectionsSettings.tsx`, `DiagnosticsSettings.tsx`, `ProcessDiagnostics.ts`, `TraceDiagnostics.ts` | Partial provider, pairing, endpoint, process/trace diagnostics, diagnostics-format contracts, and top-level provider/connections/diagnostics comparison gates |
| Command palette real actions/search | `CommandPalette.tsx`, `CommandPaletteResults.tsx`, `CommandPalette.logic.ts` | Partial core logic + dynamic GPUI groups |
| Sidebar real grouping, selection, archival actions | `Sidebar.tsx`, `uiStateStore.ts`, `threadSelectionStore.ts` | Partial static |

## Major Missing Runtime/Backend Layers

These upstream backend/runtime areas have no Rust equivalent yet:

| Upstream area | Files/modules | R3 state |
| --- | --- | --- |
| Server HTTP/WebSocket API | `apps/server/src/http.ts`, `ws.ts`, generated contracts | Missing |
| Provider orchestration | `apps/server/src/provider`, `orchestration`, `textGeneration` | Missing |
| Persistence and migrations | `apps/server/src/persistence` | Missing |
| Project discovery/setup/scripts | `apps/server/src/project`, `workspace` | Missing |
| Git/source control/PR workflow | `apps/server/src/sourceControl`, `git`, `vcs` | Missing |
| Terminal process management | `apps/server/src/terminal`, `processRunner.ts` | Missing runtime, UI state contracts only |
| Auth/pairing/saved environments | `apps/server/src/auth`, `apps/web/src/environments` | Missing |
| Desktop IPC/menu/bootstrap | `apps/desktop/src` | Missing |
| Shared contracts and generated schemas | `packages/contracts`, `effect-codex-app-server` | Small hand-written subset only |

## Immediate Port Order

1. Keep expanding deterministic visual references before porting behavior for each surface.
2. Port the Rust data contracts that unblock those surfaces: scoped refs, thread/project/session/provider state, keybindings, terminal state, diff route state.
3. Replace static GPUI panels with state-driven panels one screen at a time.
4. Add real runtime layers after the matching static surface exists and is gated.

The current port is visually promising for the gated static shell, but it is far from a full T3 Code port. The next meaningful milestones should add reference captures for active chat, running turns, approvals, terminal, and diff panel, then port the backing Rust state for each.

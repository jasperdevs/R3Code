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
| Settings keybindings add row | 6.45% |
| Settings providers | 5% |
| Settings source control | 6% |
| Settings connections | 4% |
| Settings diagnostics | 5% |
| Settings archive | 6% |
| Settings back navigation | 2% |
| Settings theme menu | 6% |
| Settings dark selection | 6% |
| Empty/no active thread, dark theme | 2% |

The strongest parity areas are the empty shell, draft chat chrome, archive empty state, and settings back path. The weakest implemented area is now the diff panel renderer, followed by Keybindings; the diff panel gate is tightened to 8.8% after source-backed row geometry, stat-label, compact composer, and syntax-palette improvements, while the simplified renderer still carries browser-vs-GPUI and missing `@pierre/diffs` differences. The active-chat gate is tightened to 4.1% after the seeded active reference model matched upstream `gpt-5.4`. The draft-family gates now hide project-only header chrome in draft references, lowering draft to 1.75%, focused composer to 1.9%, composer menu to 4.5%, inline composer tokens to 2.2%, and provider/model picker to 4.4%. Keybindings now ports upstream `Kbd` chip sizing/color/weight, `border-input` when triggers, upstream light `foreground`, resolved-row rendering, in-table add-draft shell, and the Rust equivalents of `KeybindingsSettings.logic.ts` shortcut/when/source/conflict/edit-draft contracts, lowering the gate to 6.35%. The pending-user-input composer now restores the upstream footer controls and tighter editor height, with the gate reduced to 5.1%.

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
| Project scripts, open-in-editor picker, and Git header actions | `ProjectScriptsControl.tsx`, `projectScripts.ts`, `OpenInPicker.tsx`, `GitActionsControl.tsx`, `GitActionsControl.logic.ts` | Seeded upstream menu gates plus partial core/header logic and Rust editor-launch contract parity; dialogs, disabled-reason tooltips, full editor/icon set, RPC wiring, detached process execution, and live git mutations are still missing |
| Settings providers/source-control/connections diagnostics depth | `settings/*`, `Provider*`, `SourceControlSettings.tsx`, `ConnectionsSettings.tsx`, `DiagnosticsSettings.tsx`, `ProcessDiagnostics.ts`, `TraceDiagnostics.ts` | Partial provider, source-control discovery/auth/fetch presentation, pairing, endpoint, process/trace diagnostics, diagnostics-format contracts, and top-level provider/source-control/connections/diagnostics comparison gates |
| Keybindings server runtime | `apps/server/src/keybindings.ts`, `apps/server/src/keybindings.test.ts` | Partial shortcut/when parsing, resolved-rule compilation, default merge/backfill, conflict skip, upsert/remove, and settings-row contracts in `r3_core`; file-backed keybindings.json IO, lenient JSON/schema diagnostics, atomic writes, cache invalidation, filesystem watch/debounce, PubSub streams, startup Deferreds, and RPC handlers remain missing |
| Command palette real actions/search | `CommandPalette.tsx`, `CommandPaletteResults.tsx`, `CommandPalette.logic.ts` | Partial core logic + dynamic GPUI groups |
| Sidebar real grouping, selection, archival actions | `Sidebar.tsx`, `uiStateStore.ts`, `threadSelectionStore.ts` | Seeded sidebar options menu gate plus source-backed project-scoped thread rows/status pills for deterministic references; real grouping snapshots, multi-select, context menus, hover archive controls, show-more/show-less, drag sorting, archive/delete actions, and persisted settings are still missing |

## Major Missing Runtime/Backend Layers

These upstream backend/runtime areas have no Rust equivalent yet:

| Upstream area | Files/modules | R3 state |
| --- | --- | --- |
| Server HTTP/WebSocket API | `apps/server/src/http.ts`, `auth/http.ts`, `orchestration/http.ts`, `ws.ts`, generated contracts | Partial route/method contract table in `r3_core::rpc`, including upstream WS method names, stream classification, aggregate routing, `WsRpcGroup` schema registration, server handler map, unary/stream dispatch planning, HTTP route/auth levels, RPC envelope parsing, attachment path guard and route decisions, static/dev redirect/path-guard/fallback/content-type helpers with real static file/index read response helper, browser API CORS constants and preflight/header decision contracts, project favicon route decisions plus file/fallback response helper, OTLP traces proxy route decisions, and orchestration snapshot/dispatch route contracts in `r3_core::orchestration`; live server transport/handler execution/runtime schema decoding/subscriptions/actual CORS middleware integration/attachment file and OTLP decode/export responses are still missing |
| Server startup utilities | `apps/server/src/pathExpansion.ts`, `apps/server/src/startupAccess.ts`, `atomicWrite.ts`, `stream/collectUint8StreamText.ts`, `cliAuthFormat.ts`, `serverSettings.ts`, `os-jank.ts`, `environment/Layers/ServerEnvironment*.ts`, `environment/Services/ServerEnvironment.ts` | Partial path/base-dir expansion, environment label parsing/fallback, environment-id plan, execution-environment descriptor, headless host/port resolution, static/dev loopback redirect helpers, pairing URL, Nayuki medium-ECC terminal QR rendering, terminal output, stream text collection, atomic-write plan, CLI auth formatting, and provider environment secret/redaction contracts in `r3_core::server`; live login-shell PATH hydration, Windows environment repair, launchctl fallback, macOS scutil, Linux machine-info reads, hostnamectl process execution, filesystem environment-id persistence, Effect service layer, package metadata sourcing, ServerConfig/HttpServer/ServerAuth access issuance, Effect Stream decoder flushing, filesystem temp-directory/rename side effects, exact DateTime types, CLI command integration, settings schema normalization, sparse default stripping, file watch/cache/pubsub runtime, and secret-store materialization/persistence remain missing |
| Server config/bootstrap | `apps/server/src/server.ts`, `config.ts`, `bootstrap.ts`, `serverRuntimeStartup.ts`, `serverLifecycleEvents.ts`, `serverRuntimeState.ts`, `serverLogger.ts`, `cli/config.ts`, `bin.ts`, `cli/*` | Partial server layer composition, route membership, WebSocket RPC route plan, Bun/Node HTTP/PTY/platform adapter selection, runtime dependency groups, runtime mode/defaults, derived-path, bootstrap fd-path/envelope decode, startup model/welcome, command-gate, lifecycle snapshot/sequence, persisted runtime-state, logger layer plan, CLI precedence, duration shorthand, command topology, serve run-plan, and auth/project command descriptor contracts in `r3_core::server`; live Effect layer graph, HTTP server launch, route handlers, real WebSocket execution, Tailscale Serve side effects, browser OTLP tests, bootstrap worktree dispatch, live effect/unstable CLI parsing, Node runtime layer, auth/project command execution, TTL schema errors, live/offline project mutation dispatch, runtime-state probing, live Effect logger layer, live Effect config layers, directory creation, static-dir lookup, fd readiness probing, stream duplication/fallback, readline timeout cleanup, command queue workers, PubSub streams, Ref state, runtime-state file persistence/read/clear, ConfigProvider/env reading, persisted observability loading, full ServerConfig assembly, reactors, heartbeat telemetry, auto-bootstrap dispatch, auth pairing URL, browser/headless side effects, CLI entrypoints, and project launch remain missing |
| Provider orchestration | `apps/server/src/provider`, `orchestration`, `textGeneration` | Partial command decider, composite reactor order, provider runtime ingestion command planner plus queue/drain and ordered batch bridges, guarded helper/activity/session/assistant-delta/assistant-buffer-flush/assistant-complete/fallback/pause-finalize/turn-complete-finalize/proposed-plan/proposed-plan-finalize/diff/thread-metadata command mapping contracts, runtime receipt bus contracts, checkpoint reactor filters/status/cwd/baseline/completion/revert planning, normalizer attachment plans, schema alias table, runtime layer composition, event-store execution, command receipts, projection replay, planned-event reactor intent mapping, persisted-event reactor intent extraction/batching, provider service request contracts, start-session resolution, start-session execution/completion planning, empty send-turn planning, routable-session recovery planning, session/send/stop binding payload planning, listSessions merge/mismatch rules, rollback no-op/recovery planning, stale-session stop planning, capabilities/instance-info reads, runtime event instance correlation/fan-out planning, adapter-call execution planning, shutdown reconciliation planning, and text-generation policy/prompt/sanitizer contracts exist; provider drivers, live reactor workers, subscriptions, real checkpoint git side effects, attachment file writes, full runtime ingestion, concrete adapter execution, session directory execution, and provider-backed text generation are still missing |
| Persistence and migrations | `apps/server/src/persistence` | Partial SQLite store for orchestration events, command receipts, provider_session_runtime rows, session-directory binding helpers, auth session/pairing-link rows, projection state, projection indexes, event-projected project/thread/message/session/turn/activity/plan/checkpoint/pending-approval rows, persistence error tags/messages, sqlite runtime/client config, setup pragmas, node sqlite compatibility gate, and 30-entry migration ordering/filtering, including turn-start, interrupt, revert, and stale-approval cleanup; live Effect SQL layer execution, full migration SQL/backfill semantics, live auth service integration, live provider runtime integration, persisted attachment side effects, and remaining edge events are still missing |
| Project discovery/setup/scripts | `apps/server/src/project`, `workspace` | Partial project script helpers plus workspace root normalization, safe relative-path resolution, filesystem search/browse, ignored-directory filtering, and write-file behavior in `r3_core::workspace`; project registry add/list/remove, VCS-backed indexing, cache invalidation, favicon resolver, setup runner, repository identity, and live RPC wiring remain missing |
| Git/source control/PR workflow | `apps/server/src/sourceControl`, `git`, `vcs` | Header/menu state and seeded detached-HEAD gate only; runtime git status refresh, commit/push/PR dialogs, publish repository flow, and source-control backend are still missing |
| Shared package utilities | `packages/shared/*` | Partial Rust contracts now cover string truncation, CLI arg parsing, Windows/relative path detection, semver/range comparison, git remote/branch/status helpers, source-control PR/MR terminology/provider detection, search ranking, TCP port helpers, Struct deep merge, schemaJson object extraction plus strict/unknown/lenient/pretty transformation contracts, server settings patch helpers, deterministic worker state/runtime-plan contracts, rotating-log write/rotation/prune plans, trace sink buffering plus Effect/OTLP trace record conversion contracts, shell command availability, process launch planning, model helper subset, project-script env/cwd helpers, Nayuki QR text/binary/segment/advanced-codeword/module and terminal rendering, and keybinding logic | Actual live Effect worker fibers, Effect Schema runtime integration, logging/observability runtime layers with filesystem append/rename IO, native package export generation, and live service wiring remain missing |
| Terminal process management | `apps/server/src/terminal`, `processRunner.ts` | Partial process runner output-limit/error contract in `r3_core::process`; terminal runtime, PTY management, live process-tree signaling, and UI state wiring are still missing |
| Auth/pairing/saved environments | `apps/server/src/auth`, `packages/client-runtime`, `apps/web/src/environments` | Partial auth descriptor, HTTP/websocket credential selection, owner access-control/error mapping, HTTP route plans, auth HTTP error/success/session-state response and CORS/cookie contracts, AuthControlPlane CLI session defaults plus pairing/session listing rules, bootstrap credential issue/consume decision contracts, pairing-token request body detection, secret-store filesystem contracts plus filesystem get/set/get-or-create/remove helpers, cookie, client metadata, pairing-token, HMAC-signed session/websocket token issue/verify helpers with default TTLs and claim decode/expiry checks, persisted token-to-session repository verification decisions, verified-session credential assembly, session-claim, access-stream change fan-in/current-session marking, session-credential change, connected-session count, auth session/pairing-link persistence, pairing URL helper contracts, client-runtime advertised endpoint, known-environment, scoped-ref, and source-control discovery state helpers; full live secret-store service permissions/concurrency layer, live repository-backed auth service execution, concrete HTTP exchange execution, atomic live bootstrap consume/emit behavior, websocket upgrade execution, live auth PubSub streams, browser AtomRegistry/reactivity runtime, async discovery refresh dedupe with live RPC clients, and saved-environment runtime remain missing |
| Observability/metrics | `apps/server/src/observability/Attributes.ts`, `Metrics.ts`, `RpcInstrumentation.ts`, `Layers/Observability.ts`, `Services/BrowserTraceCollector.ts` | Partial metric attribute compaction, outcome/model-family labels, metric specs/update plans, provider attributes, RPC span/metric/disabled-trace contracts, layer assembly, OTLP exporter metadata, and browser trace collector push contracts in `r3_core::observability`; live Effect Metric snapshots, Clock/TestClock durations, span/tracer runtime, Stream onExit instrumentation, disabled-tracer service behavior, local trace sink rotation, OTLP exporters, and Effect service layers remain missing |
| Telemetry analytics | `apps/server/src/telemetry/Identify.ts`, `Layers/AnalyticsService.ts`, `Services/AnalyticsService.ts` | Partial identifier priority, anonymous-id persistence plan, analytics buffer/flush/payload, service-tag, and default PostHog config contracts in `r3_core::telemetry`; live SHA-256 hashing, Codex/Claude file reads, filesystem writes, Effect ConfigProvider, Ref buffer, periodic scoped flush, HttpClient PostHog submission, and finalizer behavior remain missing |
| Server package scripts and probes | `apps/server/package.json`, `scripts/cli.ts`, `scripts/acp-mock-agent.ts`, `scripts/cursor-acp-model-mismatch-probe.ts`, build/test configs | Partial package metadata, scripts, dependency role lists, tsdown/vitest options, build/publish plans, ACP mock-agent state/config/env contracts, and Cursor ACP probe request plan; live npm publishing, catalog resolution, icon override IO, Bun/Node script execution, stdio JSON-RPC, request/exit log IO, callback behavior, and real Cursor probing remain missing |
| Desktop IPC/menu/bootstrap | `apps/desktop/src` | Partial desktop app bootstrap/lifecycle/assets/identity/config/environment/state/observability/update-channel/update-runtime/backend-start decision logic plus Electron app/dialog/menu/protocol/safe-storage/shell/theme/updater/window, preload bridge, desktop shell environment, IPC channel/handler-order, server-exposure runtime, Tailscale advertised endpoint/serve-command, backend-manager readiness/backoff, window-option, and menu contracts in `r3_core::desktop`; live GPUI desktop lifecycle, Effect layers, filesystem-backed asset lookup, live shell probing/process.env mutation, shutdown deferreds, event listeners, fatal-startup UI side effects, rotating log file IO, trace sink/tracer wiring, backend process supervision, IPC bridge/preload exposure, menus, protocols, settings IO, updater runtime side effects, and packaging remain missing |
| Chat markdown rendering | `apps/web/src/components/ChatMarkdown.tsx`, `ChatMarkdown.browser.tsx` | Partial file URI rewrite, line/column anchors, duplicate basename labels, web links, code fence language fallback, headings, bullet/ordered/task lists, blockquotes, strikethrough, GFM-style tables, and GPUI assistant markdown block rendering; full remark-gfm edge cases, Shiki/diff highlighter cache, clipboard timers, preferred-editor open actions, context menu, tooltips, skill inline text, browser interaction tests, and exact CSS remain missing |
| Package-level app surfaces | `apps/desktop/package.json`, desktop scripts/configs, `apps/marketing/*`, `packages/contracts/package.json`, `packages/contracts/tsconfig.json`, `packages/shared/package.json`, `packages/shared/tsconfig.json` | Partial desktop package/script/build/launcher/smoke/wait-resource contracts, marketing package/release URL/cache contracts, contracts package export/script/tsconfig contracts, and shared package export/script/dependency/tsconfig contracts in `r3_core::package_surfaces`; live native packaging, dev restart loop, macOS bundle/icon IO, smoke process execution, Astro pages/assets/runtime, generated Rust schema package outputs, and native shared package output generation remain missing |
| Shared contracts and generated schemas | `packages/contracts`, `effect-codex-app-server`, `effect-acp` | Small hand-written contract subset plus ACP/Codex app-server protocol method tables, wire routing, request/error builders, terminal plans, and package export/build-entrypoint contracts; generated schema/type parity, stdio transports, Effect RPC clients, mock peers, probe examples, package wiring, and full schema generation remain missing |

## Immediate Port Order

1. Keep expanding deterministic visual references before porting behavior for each surface.
2. Port the Rust data contracts that unblock those surfaces: scoped refs, thread/project/session/provider state, keybindings, terminal state, diff route state.
3. Replace static GPUI panels with state-driven panels one screen at a time.
4. Add real runtime layers after the matching static surface exists and is gated.

The current port is visually promising for the gated static shell, but it is far from a full T3 Code port. The next meaningful milestones should deepen the seeded reference gates and add the missing runtime-backed behavior, especially real provider streams, checkpoint diff retrieval/rendering, git/project actions, sidebar state, and backend contracts.

# Exact Port Goal

R3Code is a Rust/GPUI port of T3 Code. The goal is exact parity with the chosen upstream T3 Code commit, except visible product branding changes from T3 Code to R3Code.

## Non-Negotiable Target

- The upstream source commit is the truth.
- The upstream screenshots are the visual truth.
- The upstream behavior, data contracts, persistence, runtime state, keyboard handling, desktop behavior, and release shape are the product truth.
- R3Code may differ only where the rename requires it: app name, visible brand text, package/repo identifiers, and any generated artifact names that must say R3Code.

## Definition Of Done

The port is done only when all of these are true:

1. Every shipped upstream screen has an R3Code GPUI equivalent with screenshot parity.
2. Every user workflow in upstream works in R3Code with the same states, ordering, disabled states, errors, toasts, dialogs, keyboard shortcuts, and persistence.
3. Every upstream runtime layer has a Rust equivalent: provider sessions, orchestration, persistence, terminal, git/source control, project setup, auth/pairing, remote environments, desktop IPC, diagnostics, updates, and release packaging.
4. Every upstream contract has Rust tests or parity fixtures proving equivalent behavior.
5. Pixel gates are strict enough that differences are intentional and documented. Temporary loose gates are tracked as debt and tightened.
6. Latest upstream drift is either intentionally ignored by freezing the target commit or pulled into the target and ported.

## Easiest Path

Do not hand-port random UI pieces. Move in this order:

1. Freeze target: choose the exact upstream commit and keep `UPSTREAM_REFERENCE.md` current.
2. Inventory upstream: map every upstream file/module to one of `ported`, `partial`, `missing`, or `intentionally different`.
3. Lock visuals first: add screenshot references for every screen/state before changing behavior.
4. Port contracts: move TypeScript logic into Rust tests and pure Rust data functions.
5. Replace fixtures: connect GPUI screens to real Rust state one subsystem at a time.
6. Build runtime layers: provider sessions, orchestration, SQLite, terminal PTY, git/VCS, source-control providers, project scripts, auth, remote environments, desktop IPC.
7. Tighten pixels: replace approximate renderers and loose thresholds until the only ignored regions are approved branding differences.
8. Ship like upstream: package, launch, update, and release with the same practical surface.

## First Milestone

Create a full parity matrix from the pinned upstream commit:

- Web UI screens and components
- Server/runtime modules
- Shared contracts/packages
- Desktop app/IPC/update/SSH surfaces
- Tests and browser fixtures
- Release/build scripts

Each row must include upstream path, Rust target path, status, proof command, and remaining gap. No subsystem should be marked complete without passing tests or screenshot parity.

Tracking artifact: [PARITY_MATRIX.md](PARITY_MATRIX.md).
Generated file inventory: [PARITY_FILE_INVENTORY.md](PARITY_FILE_INVENTORY.md).

## Working Rule

If a change cannot be proven against upstream source, upstream screenshots, or upstream tests, it is not parity. Read upstream first, port the smallest matching slice, verify it, then move to the next slice.

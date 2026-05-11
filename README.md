# R3Code

R3Code is a Rust and GPUI coding-agent desktop experience.

The port target is visual and workflow parity with the frozen reference, with R3Code branding and a fully Rust implementation.

## Development

```text
cargo check --workspace
cargo run -p r3_app
```

Theme follows the OS by default. For screenshots or manual checks, set `R3CODE_THEME` to `light`, `dark`, or `system`.

## Parity

The UI is built against a frozen reference. See [docs/reference/UPSTREAM_REFERENCE.md](docs/reference/UPSTREAM_REFERENCE.md) and [docs/reference/PARITY_PLAN.md](docs/reference/PARITY_PLAN.md).

Useful local checks:

```text
cargo run -p xtask -- capture-reference-browser

cargo run -p xtask -- check-parity
```

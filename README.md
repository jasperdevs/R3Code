# R3Code

R3Code is a Rust and GPUI port of T3Code's coding-agent desktop experience.

The port target is visual and workflow parity with T3Code, with R3Code branding and a fully Rust implementation.

## Development

```text
cargo check --workspace
cargo run -p r3_app
```

Theme follows the OS by default. For screenshots or manual checks, set `R3CODE_THEME` to `light`, `dark`, or `system`.

## Parity

The UI is built against a frozen T3Code reference. See [docs/reference/T3CODE_VERSION.md](docs/reference/T3CODE_VERSION.md) and [docs/reference/PARITY_PLAN.md](docs/reference/PARITY_PLAN.md).

Useful local checks:

```text
cargo run -p xtask -- capture-t3code-browser

cargo run -p xtask -- check-parity
```

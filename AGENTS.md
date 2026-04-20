# Rimu

Rimu is a friendly template language for structured data and functional expressions.

## Principles

- Premature optimization is the root of all evil
- Do not second guess or make asumptions
- Prefer robustness over performance
- Achieve performance with simple fit-for-purpose abstractions, not clever hacks

### Complexity check

Before adding significant amounts of code, verify:

1. The approach is solid — not just the first thing that came to mind.
2. No simpler alternative achieves the same goal.
3. Compare to industry-standard tools and specifications if relevant.
4. Check if a good Rust crate already handles the task.

Complexity is fine when warranted - this is a genuinely complex project. The point is to be deliberate.

## Develop

### Rust

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

### JS

npm workspaces live at the repo root (`docs`, `play`, `syntax/lezer`). Top-level scripts:

```sh
npm run docs          # run docs site (Nextra) in dev mode
npm run play          # run playground (Next.js) in dev mode
npm run lezer:build   # build the Lezer grammar for Rimu
npm run lezer:test    # run Lezer grammar tests
```

The playground pulls in the Rust `play/wasm` crate via `wasm-pack`.

## Conventions

- Uses `thiserror` for error types, with a blank line between each variant
- Import order: std, external crates, internal crates (`rimu_*`), within crate (`crate::`/`self::`/`super::`), with a blank line between each group
- After code is changed, ensure any relevant documentation (e.g. crate `README.md`) is up-to-date.
- Think about module exports like the intimacy gradient of a home, the first thing you see should be the entryway (public exports), then as you go deeper into the module you should see more private bedrooms (helper functions)

### Reviews

- Think about the long-term maintenance of the project
- Check all algorithms are correct
  - Look at relevant specifications where possible
- Check there's not a simpler way to do (or say) what is needed
- Imagine alternative abstractions, compare with current abstractions
- Add debug_assert to validate any assumptions
- Add more tests, but only if useful
- Make sure all affected crates have up-to-date README.md's
- For any observations that don't lead to a change now:
  - Make a comment "Note(cc): xxx" to document for future readers,
  - Or "TODO(cc): xxx" if we should make a change in the future

### Testing

- Don't assume the current code is correct
  - Don't ever fix a test in order to pass, unless you are absolutely certain this is correct
- Before adding tests, think about specific edge cases that should be tested
  - Don't add tests just for the sake of adding tests
- If a test is redundant, remove it

## Structure

Rust workspace (see root `Cargo.toml`) laid out as a pipeline from source text to evaluated value:

- `meta/` — `rimu-meta`: source metadata (spans, errors, positions)
- `ast/` — `rimu-ast`: abstract syntax tree (blocks and expressions)
- `parse/` — `rimu-parse`: tokenizer + parser (source → AST)
- `value/` — `rimu-value`: runtime value types, environment, eval errors
- `eval/` — `rimu-eval`: AST evaluator (AST + environment → value)
- `stdlib/` — `rimu-stdlib`: built-in functions available to Rimu programs
- `rimu/` — `rimu`: top-level library crate that re-exports the pipeline; integration tests live in `rimu/tests`
- `cli/` — `rimu-cli`: command-line interface
- `repl/` — `rimu-repl`: interactive REPL
- `play/wasm/` — `rimu-playground-wasm`: wasm bindings consumed by the playground

JS / web side (npm workspaces):

- `docs/` — Nextra/Next.js docs site (deployed at `rimu.dev`)
- `play/` — Next.js playground (deployed at `play.rimu.dev`), uses `play/wasm` via `wasm-pack`
- `syntax/lezer/` — Lezer grammar package (`rimu-lezer`), consumed by `docs` and `play`
- `syntax/textmate/` — TextMate grammar (`rimu.tmLanguage.json`)

Other top-level directories:

- `examples/` — example `.rimu` programs

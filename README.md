<div align="center">
  <img
    src="./docs/public/rimu-tree-768x1024.jpg"
    alt="An 800-year-old giant rimu tree stretching high into the canopy at Ōtari-Wilton's Bush in Te Whanganui-a-Tara (Wellington), Aotearoa (New Zealand)."
    height="512px"
  />
</div>

<h1 align="center">
  <a href="https://rimu.dev">
    Rimu
  </a>
  🌱
</h1>

<div align="center">
  <strong>
    A data structure template language.
  </strong>
</div>

<br />

<div align="center">

[![release version](https://img.shields.io/github/v/release/ahdinosaur/rimu?style=flat-square&display_name=tag&include_prereleases)](https://github.com/ahdinosaur/rimu/releases/latest)
[![crates.io version](https://img.shields.io/crates/v/rimu.svg?style=flat-square)](https://crates.io/crates/rimu)
[![ci status](https://img.shields.io/github/checks-status/ahdinosaur/rimu/main?style=flat-square)](https://github.com/ahdinosaur/rimu/actions/workflows/ci.yml?query=branch%3Amain)
[![chat](https://img.shields.io/matrix/rimu:matrix.org?style=flat-square&label=chat)](https://matrix.to/#/#rimu:matrix.org)

</div>

Rimu is a friendly template language for structured data and functional expressions.

Create parametric data using the best of Yaml structures and Lisp functions.

Learn more: [rimu.dev](https://rimu.dev)

## Example

[![Screenshot of a "Hello world" Rimu example](./screenshot.png)](https://play.rimu.dev/?i=bNcpBCoAgFATQqwweQ3DXohtE4EbxV8LPD31bdPs0aTMD88acpBp2sr4ATLUXcIokC2-Uriy3etPnXAbm7XM41x-Z-RkO1INK29YQY8A0UuRXYqWmMzELFrk4dTEv)

Playground: [play.rimu.dev](https://play.rimu.dev/?i=bNcpBCoAgFATQqwweQ3DXohtE4EbxV8LPD31bdPs0aTMD88acpBp2sr4ATLUXcIokC2-Uriy3etPnXAbm7XM41x-Z-RkO1INK29YQY8A0UuRXYqWmMzELFrk4dTEv)

## Modules

- [`rimu`](./rimu/) : [![crates.io version](https://img.shields.io/crates/v/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![download](https://img.shields.io/crates/d/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu)
- [`rimu-cli`](./rimu-cli) : [![crates.io version](https://img.shields.io/crates/v/rimu-cli.svg?style=flat-square)](https://crates.io/crates/rimu-cli) [![download](https://img.shields.io/crates/d/rimu-cli.svg?style=flat-square)](https://crates.io/crates/rimu-cli) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-cli)
- [`rimu-repl`](./rimu-repl) : [![crates.io version](https://img.shields.io/crates/v/rimu-repl.svg?style=flat-square)](https://crates.io/crates/rimu-repl) [![download](https://img.shields.io/crates/d/rimu-repl.svg?style=flat-square)](https://crates.io/crates/rimu-repl) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-repl)
- [`rimu-stdlib`](./rimu-stdlib) : [![crates.io version](https://img.shields.io/crates/v/rimu-stdlib.svg?style=flat-square)](https://crates.io/crates/rimu-stdlib) [![download](https://img.shields.io/crates/d/rimu-stdlib.svg?style=flat-square)](https://crates.io/crates/rimu-stdlib) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-stdlib)
- [`rimu-eval`](./rimu-eval) : [![crates.io version](https://img.shields.io/crates/v/rimu-eval.svg?style=flat-square)](https://crates.io/crates/rimu-eval) [![download](https://img.shields.io/crates/d/rimu-eval.svg?style=flat-square)](https://crates.io/crates/rimu-eval) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-eval)
- [`rimu-value`](./rimu-value) : [![crates.io version](https://img.shields.io/crates/v/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![download](https://img.shields.io/crates/d/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-value)
- [`rimu-parse`](./rimu-parse) : [![crates.io version](https://img.shields.io/crates/v/rimu-parse.svg?style=flat-square)](https://crates.io/crates/rimu-parse) [![download](https://img.shields.io/crates/d/rimu-parse.svg?style=flat-square)](https://crates.io/crates/rimu-parse) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-parse)
- [`rimu-ast`](./rimu-ast) : [![crates.io version](https://img.shields.io/crates/v/rimu-ast.svg?style=flat-square)](https://crates.io/crates/rimu-ast) [![download](https://img.shields.io/crates/d/rimu-ast.svg?style=flat-square)](https://crates.io/crates/rimu-ast) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-ast)
- [`rimu-meta`](./rimu-meta) : [![crates.io version](https://img.shields.io/crates/v/rimu-meta.svg?style=flat-square)](https://crates.io/crates/rimu-meta) [![download](https://img.shields.io/crates/d/rimu-meta.svg?style=flat-square)](https://crates.io/crates/rimu-meta) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-meta)

## Sponsors

### [Village Kit](https://villagekit.com)

<a href="https://villagekit.com">
  <img
    src="https://villagekit.com/icon.svg"
    alt="Village Kit icon"
    height="256px"
  />
</a>

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Rimu by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>

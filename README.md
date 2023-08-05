<div align="center">
  <img
    alt="Tree"
    src="https://i.imgur.com/edQ8A2am.png"
  />
</div>

<h1 align="center">Rimu 🌲</h1>

<div align="center">
  <strong>
    Templates for data strutures, not text!
  </strong>
</div>

_Work in progress..._

Rimu is a structured template system.

Unlike other template systems, Rimu operates on data structures, not text.

We use a data structure as a template, then using another data structure as context, produce an output data structure.

## Modules

- `rimu` : [![crates.io version](https://img.shields.io/crates/v/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![download](https://img.shields.io/crates/d/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu/actions/workflows/rust.yml)
- `rimu-value` : [![crates.io version](https://img.shields.io/crates/v/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![download](https://img.shields.io/crates/d/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-value) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu-value/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu-value/actions/workflows/rust.yml)


## Install

Add `rimu` to your Cargo.toml

```toml
rimu = "*"
```

## Usage

### Interpolation

Template:

```yaml
message: hello {{key}}
```

Context:

```yaml
key: world
```

Output:

```yaml
message: hello world
```

### Expressions

To evaluate expressions, we use [Rhai](https://rhai.rs/).

### Operations

#### Eval

Template:

```yaml
- a
- b
- $eval: a + b
```

Context:

```yaml
a: 1
b: 2
```

Output:

```yaml
- a
- b
- 3
```

#### Let
#### If
#### Match
#### Switch
#### Map
#### MapObject
#### Flatten
#### FlattenDeep
#### Merge
#### MergeDeep
#### Sort
#### Reverse

## Inspiration

- [JSON-e](https://json-e.js.org/)

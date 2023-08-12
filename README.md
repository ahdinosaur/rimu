<div align="center">
  <img
    alt="Tree"
    src="https://i.imgur.com/edQ8A2am.png"
  />
</div>

<h1 align="center">Rimu 🌲</h1>

<div align="center">
  <strong>
    Templates for data strutures (not text)
  </strong>
</div>

_Work in progress..._

Rimu is a structured template system.

Unlike other template systems, Rimu operates on data structures, not text.

Write your templates in your favorite data format such as YAML, TOML, or JSON. No alternate template language, just data.

```txt
(template, context) => output
```

## Modules

- [`rimu`](./rimu/) : [![crates.io version](https://img.shields.io/crates/v/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![download](https://img.shields.io/crates/d/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu/actions/workflows/rust.yml)
- [`rimu-value`](./rimu-value) : [![crates.io version](https://img.shields.io/crates/v/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![download](https://img.shields.io/crates/d/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-value) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu-value/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu-value/actions/workflows/rust.yml)

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

### Blocks

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

Template:

```yaml
foo:
  $let:
    bar: 200
    baz:
      $eval: ten
  in:
    - $eval: bar + baz
    - $eval: bar - baz
    - $eval: bar * baz
```

Context:

```yaml
ten: 10
```

Output:

```yaml
foo:
  - 210
  - 190
  - 2000
```

#### If

Template:

```yaml
k1:
  $if: 'cond'
  then: 1
  else: 2
k2: 3
```

Context:

```yaml
cond: true
```

Output:

```yaml
k1: 1
k2: 3
```

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

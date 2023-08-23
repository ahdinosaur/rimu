<div align="center">
  <img
    alt="Tree"
    src="https://i.imgur.com/edQ8A2am.png"
  />
</div>

<h1 align="center">Rimu 🌱</h1>

<div align="center">
  <strong>
    Data structure template language.
  </strong>
</div>

Rimu is a template language for structured data and functional expressions.

Unlike other template languages, Rimu operates on data structures, not text.

Learn more at [rimu.dev](https://rimu.dev)

## Modules

- [`rimu`](./rimu/) : [![crates.io version](https://img.shields.io/crates/v/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![download](https://img.shields.io/crates/d/rimu.svg?style=flat-square)](https://crates.io/crates/rimu) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu/actions/workflows/rust.yml)
- [`rimu-value`](./rimu-value) : [![crates.io version](https://img.shields.io/crates/v/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![download](https://img.shields.io/crates/d/rimu-value.svg?style=flat-square)](https://crates.io/crates/rimu-value) [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/rimu-value) [![ci status](https://img.shields.io/github/actions/workflow/status/ahdinosaur/rimu-value/rust.yml?branch=main&style=flat-square)](https://github.com/ahdinosaur/rimu-value/actions/workflows/rust.yml)

## Example

Let's say we wanted to make a mini [Salt Stack](https://docs.saltproject.io/en/getstarted/), similar to [Comtrya](https://github.com/comtrya/comtrya).

Here's a _contrived_ example,

The template:

```yaml
$let:
  pkgs:
    - "zsh"
    - "vim"
in:
  $map: pkgs
  item: pkg
  each:
    $switch:
      - case: os.name == "macos"
        then:
          $type: brew_install
          pkg: pkg
      - case: os.name == "linux" && os.distribution == "debian"
        then:
          $type: apt_install
          pkg: pkg
```

With an environment (context):

```yaml
os:
  name: "linux"
  distribution: "debian"
```

Becomes:

```yaml
- type: "apt_install"
  pkg: "zsh"
- type: "apt_install"
  pkg: "vim"
```

## Usage

### Interpolation

Template:

```yaml
message: hello {{key}}
```

Environment:

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

Environment:

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

Environment:

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
  $if: "cond"
  then: 1
  else: 2
k2: 3
```

Environment:

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

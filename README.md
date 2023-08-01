# rimu

_Work in progress..._

Rimu is a structured template system. Unlike other template systems, Rimu operates on data structures, not text.

We use a data structure as a template, then using another data structure as context, produce an output data structure.

## Install

TODO

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
- type: ops.eval
  expr: a + b
```

Context:

```yaml
a: 1
b: 2
```

Output:

```yaml
- 1
- 2
- 3
```

#### Let

TODO

#### If

TODO

#### Match

TODO

#### Switch

TODO

#### Map

TODO

## Inspiration

- [JSON-e](https://json-e.js.org/)

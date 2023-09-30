# Releasing Rimu

## Pre-requisities

- `cargo-workspaces`

## Release new version

```shell
cargo workspace publish --all --force '*' <version>
```

Then our [`cargo-dist`](https://github.com/axodotdev/cargo-dist)-powered release action will build a new release on GitHub.

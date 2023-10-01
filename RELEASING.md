# Releasing Rimu

## Pre-requisities

- `cargo-workspaces`

## Release new version

```shell
cargo workspaces version --all --force '*' --no-individual-tags
```

Then our [`cargo-dist`](https://github.com/axodotdev/cargo-dist)-powered release action will build a new release on GitHub.

```shell
cargo workspaces publish --from-git
```

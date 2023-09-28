# Releasing Rimu

## Pre-requisities

- `cargo-workspaces`

## Release new version

```shell
cargo workspace version --all --force '*' <major|minor|patch>
```

```shell
git commit -am "release: 0.2.0"
git tag "v0.2.0"
git push
git push --tags
```

Then our [`cargo-dist`](https://github.com/axodotdev/cargo-dist)-powered release action will build a new release.

# Contributing to Rimu

Thanks for your interest in contributing to Rimu! 🌱

This project is open commons that anyone can improve, stewarded by the Rimu team. 😺

We welcome all contributions, such as but not limited to:

- designs
- specifications
- documentation
- tests
- websites
- artwork
- code

#### Table Of Contents

- [Code of Conduct](#code-of-conduct)
- [What should I know before I get started?](#what-should-i-know-before-i-get-started)
  - [Rimu ecosystem](#rimu-ecosystem)
  - [Rimu design decisions](#design-decisions)
- [How can I contribute?](#how-can-i-contribute)
  - [Bug reports](#bug-reports)
  - [Feature requests](#feature-requests)
  - [Code contributions](#code-contributions)
- [Style guides](#style-guides)
  - [Git Commit Messages](#git-commit-messages)
  - [Rust style guide](#rust-style-guide)
  - [JavaScript style guide](#javascript-style-guide)
  - [Specs style guide](#specs-style-guide)
  - [Documentation style guide](#documentation-style-guide)
- [Additional Notes](#additional-notes)

## Code of Conduct

Anyone who interacts with Rimu in any space, including but not limited to our GitHub repositories, must follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## What should I know before I get started?

TODO

### Rimu ecosystem

TODO

### Rimu design decisions

TODO

## How can I contribute?

### Bug reports

Have a look at the issue tracker [here](https://github.com/ahdinosaur/rimu/issues). If you can't find an issue (open or closed) describing your problem (or a very similar one) there, please open a new issue with the following details:

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:

1. ...
2. ...
3. ...

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Device (please complete the following information):**

- OS: [e.g. Debian, macOS]
- OS version: [e.g. Bullseye, 13]
- If web,
  - Browser [e.g. Firefox, Chrome]
- If smartphone,
  - Device [e.g. OnePlus 9, iPhone 6]

**Additional context**
Add any other context about the problem here.

### Feature requests

If you can't find an issue (open or closed) describing your idea, open an issue. Adding answers to the following questions in your description is helpful:

**Is your feature request related to a problem? Please describe.**

A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**

A clear and concise description of what you want to happen.

**How might this be added to Rimu?**

How might be involved in making this change to Rimu?

What effects would this have? Any possible disadvantages?

**What are possible alternatives?**

A clear and concise description of any alternative solutions or features you've considered.

**Additional context**

Add any other context or screenshots about the feature request here.

### Licensing

We prefer to license under the Apache-2.0 license.

### Code contributions

Please be sure to follow the relevant [Style Guides](#style-guide).

## Style Guide

### Git style guide

Write your commit messages however you want, at the end we will squash the commits as the pull request is merged.

When you submit a pull request, include clear information about how your pull request:

- What problem are you trying to solve? (Include references to relevant links)
- What is changed by this pull request:
  - Any breaking changes?
  - Any feature additions?
  - Any bug fixes?
- Document any and all technical decisions you had to make along the way.

If and when your pull request is merged, we will squash the commits into a single commit, where the commit message is your pull request text.

### Rust style guide

We follow the [Rust Style Guide](https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md), enforced using [rustfmt](https://github.com/rust-lang-nursery/rustfmt).

To run rustfmt tests locally:

1. Use rustup to set rust toolchain.

2. Install the rustfmt and clippy by running

```shell
rustup component add rustfmt
rustup component add clippy
```

3. Run clippy using cargo from the root of your repo.

```shell
cargo clippy
```

Each Pull Request needs to compile without warning.

4. Run rustfmt using cargo from the root of your repo

To see changes that need to be made, run

```shell
cargo fmt --all -- --check
```

If all code is properly formatted (e.g. if you have not made any changes), this should run without error or output.

If your code needs to be reformatted, you will see a diff between your code and properly formatted code. If you see code here that you didn't make any changes to then you are probably running the wrong version of rustfmt.

Once you are ready to apply the formatting changes, run

```shell
cargo fmt --all
```

You won't see any output, but all your files will be corrected.

### JavaScript style guide

We follow the a JavaScript style similar to [Standard Style](https://standardjs.com/), enforced using [`prettier`](https://www.npmjs.com/package/prettier). See `prettierrc.yml` for the Prettier configuration.

### Specs style guide

We use [Markdown](https://daringfireball.net/projects/markdown) for our specifications.

TODO

### Documentation style guide

We use [Markdown](https://daringfireball.net/projects/markdown) for our documentation.

TODO

## Additional Notes

TODO

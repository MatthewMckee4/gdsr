# Contributing

## Finding ways to help

We label issues that would be good for a first time contributor as
[`good first issue`](https://github.com/MatthewMckee4/gdsr/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
These usually do not require significant experience with code base.

We label issues that we think are a good opportunity for subsequent contributions as
[`help wanted`](https://github.com/MatthewMckee4/gdsr/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22).
These require varying levels of experience.

## Setup

[Rust](https://rustup.rs/) is required to build and work on the project.

## Testing

For running tests, we recommend [nextest](https://nexte.st/).

## Documentation

To prepare and run the documentation locally, run:

```shell
uv run -s scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical serve
```

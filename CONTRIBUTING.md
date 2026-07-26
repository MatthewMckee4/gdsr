# Contributing to GDSR

Thanks for your interest in contributing to GDSR. Contributions of all kinds are
welcome, and we try to keep the development process as smooth as possible.

If you hit a bug, have a feature idea, or want to suggest an improvement to the
contributing docs themselves, please
[open an issue](https://github.com/MatthewMckee4/gdsr/issues/new).

For small changes like bug fixes, feel free to jump straight to a pull request.
For anything larger, it is usually worth opening an issue first to discuss the
approach.

If you want to tackle an issue that is already open, please leave a comment
letting me know you would like to work on it. There is only one person working
on this project right now, so issues may be out of date and I do not want you to
work on something that does not align with the goals for the project.

## Finding Ways To Help

We label issues that would be good for a first-time contributor as
[`good first issue`](https://github.com/MatthewMckee4/gdsr/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
These usually do not require significant experience with the codebase.

We label issues that we think are a good opportunity for subsequent
contributions as
[`help wanted`](https://github.com/MatthewMckee4/gdsr/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22).
These require varying levels of experience.

## Architecture

GDSR is a Rust workspace with a library crate, a viewer crate, docs, and
benchmark tooling.

The main crates:

- `gdsr` - the library crate. Contains the GDSII data model, binary reader and
  writer, units, element types, and geometry helpers.
- `gdsr-viewer` - the Bevy-rendered viewer binary with an `egui` interface.
  Loads GDSII libraries, renders cells, and provides the interactive inspection
  workflow.
- `gdsr-benchmark` - the CodSpeed and Criterion benchmark suite for parser and
  writer performance.

Infrastructure and tooling:

- `docs/` - zensical documentation.
- `scripts/prepare_docs.py` - prepares generated docs pages before serving or
  building the docs.
- `scripts/benchmark/` - benchmark plotting and benchmark result assets.

## Prerequisites

GDSR is written in Rust. Install the
[Rust toolchain](https://www.rust-lang.org/tools/install) to get started. The
repository includes `rust-toolchain.toml`, so `rustup` will select the expected
toolchain automatically.

You can optionally install prek hooks to automatically run validation checks
when making a commit:

```bash
uv tool install prek
prek install
```

We recommend [nextest](https://nexte.st/) for running the Rust test suite:

```bash
cargo install cargo-nextest --locked
```

## Development

To run the tests:

```bash
cargo nextest run
```

Pass test arguments directly for focused iteration:

```bash
cargo nextest run -p gdsr
```

Before opening a pull request, run the relevant tests plus the full validation
sweep:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
uvx prek run -a
```

Run the GDS parser and writer benchmarks locally with Criterion:

```bash
cargo bench -p gdsr-benchmark --bench io
```

Check the CodSpeed integration without uploading benchmark results:

```bash
cargo codspeed build -p gdsr-benchmark --bench io -m simulation
cargo codspeed run -p gdsr-benchmark --bench io -m simulation
```

## Documentation

We use zensical to build the documentation.

```bash
uv run -s scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical build
```

To serve the docs locally:

```bash
uv run -s scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical serve
```

## Release Process

Releases are automated through the release workflow and `seal`.

First, install [seal](https://github.com/MatthewMckee4/seal), then bump the
version with:

```bash
seal bump alpha
```

or:

```bash
seal bump <version>
```

This creates a branch and commit, so you just need to open a pull request.

## GitHub Actions

If you update GitHub Actions, run `pinact` to pin action versions:

```bash
pinact run
```

Audit the active default-branch ruleset with:

```bash
gh api repos/MatthewMckee4/gdsr/rulesets/1435762
gh api repos/MatthewMckee4/gdsr/rulesets/1435762 \
  --jq '.rules as $rules | [$rules[] | select(.type == "required_status_checks")] as $status_rules | [$rules[] | select(.type == "pull_request")] as $pull_request_rules | ($status_rules[0].parameters) as $checks | ($pull_request_rules[0].parameters) as $pull_request | if (.target == "branch" and ($status_rules | length) == 1 and ($pull_request_rules | length) == 1 and .enforcement == "active" and .conditions.ref_name == {"exclude":[],"include":["~DEFAULT_BRANCH"]} and .bypass_actors == [] and $checks.strict_required_status_checks_policy == true and $checks.do_not_enforce_on_create == true and ([$checks.required_status_checks[].context] | sort) == (["CI gate","codecov/patch","codecov/project"] | sort) and $pull_request == {"required_approving_review_count":0,"dismiss_stale_reviews_on_push":true,"require_code_owner_review":false,"require_last_push_approval":false,"required_review_thread_resolution":false,"allowed_merge_methods":["merge","squash","rebase"]}) then {target, enforcement, conditions, bypass_actors, required_status_checks: $checks, pull_request: $pull_request} else error("ruleset mismatch") end'
```

Documentation-only pull requests still run coverage so `codecov/project` and
`codecov/patch` resolve. Rust platform tests and benchmarks may skip only when
`determine changes` reports `code=false`; `CI gate` validates that combination.

GitHub code-owner review is disabled because the sole code owner cannot approve
their own pull request. Independent review by a non-author agent remains
mandatory under `AGENTS.md`.

The required status contexts are `CI gate`, `codecov/project`, and
`codecov/patch`.

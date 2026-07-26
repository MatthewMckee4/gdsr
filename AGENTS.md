# GDSR Repository

GDSR is a Rust workspace for reading, writing, and viewing GDSII layout files.
Read `CONTRIBUTING.md` before changing files; it has the current crate map,
setup commands, docs commands, and release process.

## Code Review Rules

When reviewing a branch or pull request, be deliberately nitpicky. Report bugs,
regressions, architectural and maintenance risks, weak test coverage, unclear
code, unnecessary complexity, and meaningful consistency issues. Order
findings by severity, cite files and lines, distinguish blockers from
non-blocking improvements, and number each finding for follow-up discussion.

## Running Tests

Run the test suite with nextest:

```sh
cargo nextest run
```

Run `cargo nextest run` for code changes before finishing. Prefer focused
tests during iteration, then run the full suite when behavior changed.

Pass package or test filters directly when a narrower run is enough during
iteration:

```sh
cargo nextest run -p gdsr
```

If `cargo-nextest` is not installed, use `cargo test`.

Run Clippy with the same strictness as CI:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Run docs locally with:

```sh
uv run -s scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical serve
```

Run `uvx prek run -a` at the end of every task. During iteration, use
`uvx prek run --files <path1> <path2>` with every changed file to keep hook runs
independent of staged state.

## Snapshots And Generated Files

Prefer property tests under `src/property_tests/` or integration tests when
behavior crosses parser, writer, geometry, or viewer boundaries. Keep UI
changes tested through pure logic such as coordinate transforms, selection
state, or data preparation instead of rendering.

Use snapshot tests for command-output integration tests. Prefer inline
snapshots for small expected values instead of chains of individual
`assert_eq!` calls. Use external snapshot files only when the output is too
large to read inline. After updating snapshots, review the diff before
accepting them and check for pending snapshot files.

Do not edit snapshot files or inline snapshot bodies manually. Regenerate them
with the relevant test, then review the generated diff. Do not accept unrelated
snapshot changes.

`scripts/prepare_docs.py` generates `docs/index.md` from `README.md`; update the
source content rather than hand-editing the generated file.

## Development Guidelines

- All behavior changes must be tested. If you did not run the relevant tests,
  the change is not done.
- Look for an existing test file before creating a new one.
- Look for existing utilities and local patterns before writing new code.
- Prefer fixing the underlying architectural problem over adding a local
  workaround. Ask for guidance when that requires a larger change.
- Keep visibility narrow by default, but make an item public when another
  workspace crate needs it and that is the cleaner implementation.
- Keep Rust imports at the top of the file, not locally inside functions.
- Avoid `panic!`, `unreachable!`, `.unwrap()`, unsafe code, and Clippy ignores.
  Encode constraints in the type system instead.
- Prefer `if let` for fallibility, and prefer let chains over nested `if let`
  when it improves readability.
- Prefer `#[expect()]` over `#[allow()]` when a Clippy lint must be suppressed.
- Delete unused code instead of suppressing dead-code warnings.
- Prefer short imports over fully qualified paths for readability.
- Use comments to explain invariants or unusual decisions, not to narrate code.
- Avoid redundant comments and section separators in tests.
- Prefer function comments over inline comments.
- Add new dependencies to `[workspace.dependencies]` in the root `Cargo.toml`
  and reference them with `{ workspace = true }` in crate manifests.
- Do not commit directly to `main`; use a feature branch and open a pull
  request.
- Pin GitHub Actions to full commit SHAs, except the documented
  `dtolnay/rust-toolchain` tag, and run `pinact run` after changing a workflow.
- Consider whether a change needs docs under `docs/`. New public APIs, CLI
  changes, changed defaults, and release behavior changes usually need docs.

## Pull Requests

Keep pull requests minimal and focused. Use the pull request template and add
labels based on user-facing impact. If a pull request has no user-facing
change, add only the `internal` label. CI performance changes use only the `ci`
label; reserve `performance` for user-facing performance improvements.

Keep the summary and test plan concise. Write descriptions as prose, not bullet
lists or checklists. Explain what changed and why; include implementation
details only when reviewers need them.

Use a descriptive one-line commit subject by default. Do not add AI tooling as
an author or co-author.

Every pull request must receive independent review from an agent that did not
author its changes. Resolve all findings before merge.

Never merge a pull request unless every required check for the exact current
head commit has completed successfully. `main` must also be green. The only
exception is a narrowly scoped pull request whose sole purpose is restoring
`main`; that repair still requires exact-head CI green and independent review.
The head must include current `main`: after another pull request merges, update
the next branch with latest `main`, rerun every required check, and inspect the
exact head SHA and check conclusions immediately before merging. A failing,
pending, skipped, cancelled, stale, or missing required check blocks merging;
never bypass, acknowledge, or ignore it.

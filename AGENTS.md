# GDSR Repository

GDSR is a Rust workspace for reading, writing, and viewing GDSII layout files.
Read `CONTRIBUTING.md` before changing files; it has the current crate map,
setup commands, docs commands, and release process.

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

`scripts/prepare_docs.py` generates `docs/index.md` from `README.md`; update the
source content rather than hand-editing the generated file.

## Development Guidelines

- All behavior changes must be tested. If you did not run the relevant tests,
  the change is not done.
- Look for existing utilities and local patterns before writing new code.
- Keep visibility narrow by default, but make an item public when another
  workspace crate needs it and that is the cleaner implementation.
- Keep Rust imports at the top of the file, not locally inside functions.
- Avoid `panic!`, `unreachable!`, `.unwrap()`, unsafe code, and Clippy ignores.
  Encode constraints in the type system instead.
- Prefer `if let` for fallibility, and prefer let chains over nested `if let`
  when it improves readability.
- Prefer `#[expect()]` over `#[allow()]` when a Clippy lint must be suppressed.
- Prefer short imports over fully qualified paths for readability.
- Use comments to explain invariants or unusual decisions, not to narrate code.
- Avoid redundant comments and section separators in tests.
- Prefer function comments over inline comments.
- Add new dependencies to `[workspace.dependencies]` in the root `Cargo.toml`
  and reference them with `{ workspace = true }` in crate manifests.
- Do not commit directly to `main`; use a feature branch and open a pull
  request.
- Consider whether a change needs docs under `docs/`. New public APIs, CLI
  changes, changed defaults, and release behavior changes usually need docs.

## Pull Requests

Always use the pull request template and add labels. Write the description in
concise prose paragraphs, with code examples only when they help the reviewer.
Do not use checkboxes. Do not add AI tooling as an author or co-author.

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

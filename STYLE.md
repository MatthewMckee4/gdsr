# Style Guide

This guide covers user-facing text in GDSR documentation, CLI output, issue
templates, and release notes.

## General

- Write `GDSR` for the project, `gdsr` for the crate, and `gdsr-viewer` for the
  viewer binary.
- Write `GDSII` for the file format.
- Use direct, concrete language. Prefer "run `gdsr-viewer path/to/file.gds`"
  over "it is possible to run the viewer".
- Use backticks for commands, flags, environment variables, file paths, crate
  names, and code expressions.
- Avoid bare URLs in prose. Prefer descriptive links.
- Wrap Markdown at 100 characters unless the file is generated.

## Documentation

- Start from the user's task, then add details.
- Put common workflows before edge cases.
- Keep generated pages generated. Update the source and regenerate the docs
  instead of hand-editing generated output.
- Use `console` fences when showing commands and their output. Use `bash` only
  for shell scripts.
- Include command output only when the exact output matters.
- Link to reference pages from guides when the reader needs the complete option
  list.

## CLI Output

- Error messages should say what failed and include the relevant path, layer,
  cell, or element when available.
- Hints should be actionable and formatted as `hint: <message>`.
- Output must still make sense without color.
- Write machine-readable data to stdout. Write diagnostics, progress, and
  warnings to stderr.

## Terminology

- Use "library" for a complete GDSII library.
- Use "cell" for structures unless quoting GDSII record names.
- Use "element" for boundaries, paths, text, references, nodes, and boxes.
- Use "layer" and "datatype" for the GDSII layer selectors.
- Use "snapshot", not "golden file", for snapshot-testing output.

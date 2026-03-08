# Changelog

## 0.0.1-alpha.3

This release is mainly for fixing the release process of gdsr-viewer.

### Documentation

- Add installation methods to README.md ([#218](https://github.com/MatthewMckee4/gdsr/pull/218))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.2

### Bug Fixes

- Fix integer overflow panics in spatial grid and add viewer stress tests ([#205](https://github.com/MatthewMckee4/gdsr/pull/205))
- Fix viewer crash on overflow and improve rendering performance ([#164](https://github.com/MatthewMckee4/gdsr/pull/164))

### CLI

- Add command line argument parser & open viewer with .gds file preloaded. ([#166](https://github.com/MatthewMckee4/gdsr/pull/166))

### GDS Spec

- Add first-class Node element type ([#182](https://github.com/MatthewMckee4/gdsr/pull/182))
- Add first-class GdsBox element type ([#178](https://github.com/MatthewMckee4/gdsr/pull/178))
- Add path extension support (BgnExtn/EndExtn) ([#177](https://github.com/MatthewMckee4/gdsr/pull/177))

### IO

- Change ToGds to return Vec<u8> and parallelize within cells ([#109](https://github.com/MatthewMckee4/gdsr/pull/109))

### Library

- Add SVG export for cells and libraries ([#192](https://github.com/MatthewMckee4/gdsr/pull/192))
- Expand paths to true width for rendering ([#188](https://github.com/MatthewMckee4/gdsr/pull/188))
- Add iter_elements / iter_elements_mut to Cell ([#185](https://github.com/MatthewMckee4/gdsr/pull/185))
- Add library and cell statistics ([#183](https://github.com/MatthewMckee4/gdsr/pull/183))
- Add layer remapping support ([#179](https://github.com/MatthewMckee4/gdsr/pull/179))
- Add dangling reference detection to Library ([#175](https://github.com/MatthewMckee4/gdsr/pull/175))

### Performance

- Refactor viewer: split viewport module, group app fields, add property tests ([#171](https://github.com/MatthewMckee4/gdsr/pull/171))

### Viewer

- Fix path extension rendering in viewer ([#209](https://github.com/MatthewMckee4/gdsr/pull/209))
- Make QuickPick generic and improve recent projects display ([#208](https://github.com/MatthewMckee4/gdsr/pull/208))
- Add selectable display units with coordinate display in viewer ([#207](https://github.com/MatthewMckee4/gdsr/pull/207))
- Redesign viewer UI with Zed-like panels and command palette ([#203](https://github.com/MatthewMckee4/gdsr/pull/203))
- Add color picker & fix render cache bug. ([#201](https://github.com/MatthewMckee4/gdsr/pull/201))
- Improve viewer hover highlighting and fix text rendering ([#199](https://github.com/MatthewMckee4/gdsr/pull/199))
- Add adaptive grid overlay to viewer ([#190](https://github.com/MatthewMckee4/gdsr/pull/190))
- Add ruler/measurement tool to viewer ([#189](https://github.com/MatthewMckee4/gdsr/pull/189))
- Highlight element on hover in viewer ([#187](https://github.com/MatthewMckee4/gdsr/pull/187))
- Add cell search/filter to viewer side panel ([#186](https://github.com/MatthewMckee4/gdsr/pull/186))
- Add keyboard shortcuts for pan/zoom in viewer ([#181](https://github.com/MatthewMckee4/gdsr/pull/181))
- Show cell hierarchy as a tree in viewer side panel ([#180](https://github.com/MatthewMckee4/gdsr/pull/180))
- Add spatial grid with cell-level LOAD for large element counts ([#169](https://github.com/MatthewMckee4/gdsr/pull/169))
- Fix zoom to anchor on cursor position ([#168](https://github.com/MatthewMckee4/gdsr/pull/168))
- Add streaming element expansion for progressive rendering ([#167](https://github.com/MatthewMckee4/gdsr/pull/167))
- Add interactive GDS viewer and sample example ([#142](https://github.com/MatthewMckee4/gdsr/pull/142))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)
- [@limonfort](https://github.com/limonfort)

## 0.0.1-alpha.1

### Bug Fixes

- Write SREF for single-instance cell references ([#87](https://github.com/MatthewMckee4/gdsr/pull/87))
- Fix record_size mismatch in write_points_to_file (#73) ([#86](https://github.com/MatthewMckee4/gdsr/pull/86))
- Return error for oversized polygons (#74) ([#85](https://github.com/MatthewMckee4/gdsr/pull/85))
- Fix Text.transform_impl() missing reflection and incomplete scale ([#61](https://github.com/MatthewMckee4/gdsr/pull/61))
- Fix Reflection.from_line() absolute value bug ([#58](https://github.com/MatthewMckee4/gdsr/pull/58))
- Fix Polygon.move_to() collapsing all points ([#57](https://github.com/MatthewMckee4/gdsr/pull/57))

### Documentation

- Add documentation to public types and methods ([#70](https://github.com/MatthewMckee4/gdsr/pull/70))
- Add codecov badge to README ([#41](https://github.com/MatthewMckee4/gdsr/pull/41))
- Add crate README and crates.io link ([#40](https://github.com/MatthewMckee4/gdsr/pull/40))

### Geometry

- Add quickcheck property tests for Polygon ([#96](https://github.com/MatthewMckee4/gdsr/pull/96))
- Add quickcheck property tests for Point ([#90](https://github.com/MatthewMckee4/gdsr/pull/90))
- Add Dimensions trait for Path, Text, Element, and Cell ([#72](https://github.com/MatthewMckee4/gdsr/pull/72))
- Add transformation composition tests ([#67](https://github.com/MatthewMckee4/gdsr/pull/67))
- Add edge case tests for Grid calculations ([#66](https://github.com/MatthewMckee4/gdsr/pull/66))
- Add tests for geometry utility functions ([#65](https://github.com/MatthewMckee4/gdsr/pull/65))

### IO

- Add fuzz-style tests for malformed GDS input ([#99](https://github.com/MatthewMckee4/gdsr/pull/99))
- Add GDS2 spec validation for element fields ([#92](https://github.com/MatthewMckee4/gdsr/pull/92))
- Add unit tests for utils/io.rs ([#89](https://github.com/MatthewMckee4/gdsr/pull/89))
- Error on invalid data types and non-ASCII strings ([#88](https://github.com/MatthewMckee4/gdsr/pull/88))
- Add I/O edge case tests ([#69](https://github.com/MatthewMckee4/gdsr/pull/69))

### New Features

- Introduce GdsError custom error type ([#91](https://github.com/MatthewMckee4/gdsr/pull/91))
- Add to_integer and to_float conversion methods for all types ([#71](https://github.com/MatthewMckee4/gdsr/pull/71))
- Add Display implementations for transformation types ([#60](https://github.com/MatthewMckee4/gdsr/pull/60))

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

## 0.0.1-alpha.0

First alpha release of gdsr.

### Contributors

- [@MatthewMckee4](https://github.com/MatthewMckee4)

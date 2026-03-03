# Changelog

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

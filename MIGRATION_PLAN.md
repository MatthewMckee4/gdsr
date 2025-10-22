# GDSR Migration Plan: Generic Point to Non-Generic Point

## Overview
This document tracks the migration from a generic `Point<DatabaseUnitT>` to a non-generic `Point` that uses the `Unit` enum internally.

## Status: In Progress

### Phase 1: Core Types ✅ COMPLETED
- [x] Implement `Unit` enum with Integer and Float variants
- [x] Add custom `PartialEq` for Unit (compares real-world values)
- [x] Add `Display` and `Debug` implementations for Unit
- [x] Implement `Point` struct using Unit internally
- [x] Add `Display` and `Debug` implementations for Point
- [x] Add `From` implementations for common types (arrays, tuples)
- [x] Add rotation method to Point
- [x] Add conversion methods (to_integer_unit, to_float_unit)
- [x] Add getter/setter methods and mutable accessors
- [x] Make Point::new accept `impl Into<Unit>`

### Phase 2: Element Types ✅ COMPLETED
- [x] Update Path struct to use non-generic Point
- [x] Update Polygon struct to use non-generic Point
- [x] Update Text struct to use non-generic Point
- [x] Update Grid struct to use non-generic Point
- [x] Simplify tests for all element types
- [x] Export elements module in lib.rs

### Phase 3: Core Structures ✅ COMPLETED
- [x] Update Cell struct to work with new Point
  - Location: `crates/gdsr_core/src/cell/mod.rs`
  - Removed generic `<DatabaseUnitT: CoordNum>` parameter
  - Updated all collections to use non-generic types
  - Temporarily removed references field (will be re-added after Reference is updated)
  - Added Display implementation
  - Commented out Transformable trait implementation
  - Commented out add() and get_elements() methods (depend on Element enum and Library)
  - Added simplified add_polygon(), add_path(), add_text() methods
  - Created 7 basic tests (all passing)

- [x] Re-create Element enum without generics
  - Location: `crates/gdsr_core/src/elements/element.rs`
  - Created new Element enum with Path, Polygon, Text variants
  - Added Display implementation
  - Added From implementations for Path, Polygon, Text
  - Placeholder for Reference variant (to be added later)
  - Created 4 basic tests (all passing)
  - Exported from elements module

### Phase 4: Reference Structure ✅ COMPLETED
- [x] Update Instance enum to work with new Point
  - Location: `crates/gdsr_core/src/elements/reference/instance/mod.rs`
  - Removed generic `<DatabaseUnitT: CoordNum>` parameter
  - Updated to use non-generic Element enum
  - Updated From implementations for Path, Polygon, Text
  - Added Display implementation
  - Created 6 tests (all passing)
  - Note: Reference From impl not added due to circular dependency

- [x] Update Reference struct to work with new Point
  - Location: `crates/gdsr_core/src/elements/reference/mod.rs`
  - Removed generic parameter
  - Updated Instance and Grid fields to non-generic types
  - Commented out get_elements_in_grid() and flatten() methods (depend on transformation/Library)
  - Commented out Transformable and Movable trait implementations
  - Added Display implementation
  - Created 5 basic tests (all passing)
  - Commented out IO module

- [x] Updated Element enum to include Reference variant
  - Added Reference to Element enum
  - Added From<Reference> for Element implementation
  - All Element tests still passing

### Phase 5: Traits Module ⏸️ POSTPONED
- [x] Temporarily comment out Transformable and Movable traits
  - Updated commented code to use non-generic Point syntax (for reference)
  - **Decision**: Leave commented out for now, will tackle later

- [x] Update Dimensions trait
  - Removed generic `<T: CoordNum>` parameter
  - Updated bounding_box() to return `(Point, Point)`
  - Can be used once needed

**Note**: Traits module work is postponed. Focus on other modules first.

### Phase 6: Transformation Module ⏸️ POSTPONED
**Decision**: Skip transformation module for now, tackle later.

- [ ] Update Translation struct/module (later)
- [ ] Update Rotation struct/module (later)
- [ ] Update Scale struct/module (later)
- [ ] Update Reflection struct/module (later)
- [ ] Update Transformation struct to combine all transforms (later)

### Phase 7: Other Modules 🔜 NEXT PRIORITY
- [ ] Identify all remaining modules that use Point/Unit
  - Search for Point<T>, CoordNum, DatabaseUnitT patterns
  - Check config module
  - Check library module
  - Check utility modules

- [ ] Update Library module if needed
  - Location: TBD
  - May need to work with non-generic Cell/Reference

- [ ] Update Config module if needed
  - Update configuration types to work with new Point
  - Update database unit handling
  - Update user unit handling

### Phase 8: Re-implement Traits for Elements ⏸️ POSTPONED
**Will tackle after traits and transformation modules are updated:**
- [ ] Re-implement Transformable for Path
- [ ] Re-implement Movable for Path
- [ ] Re-implement Dimensions for Path
- [ ] Re-implement Transformable for Polygon
- [ ] Re-implement Movable for Polygon
- [ ] Re-implement Dimensions for Polygon
- [ ] Re-implement Transformable for Text
- [ ] Re-implement Movable for Text
- [ ] Re-implement Transformable for Grid
- [ ] Re-implement Movable for Grid
- [ ] Re-implement Transformable for Reference (after it's updated)
- [ ] Re-implement Movable for Reference (after it's updated)
- [ ] Re-implement Transformable for Cell (after it's updated)
- [ ] Re-implement Movable for Cell (after it's updated)

### Phase 9: Re-implement IO Modules ⏸️ POSTPONED
- [ ] Update Path IO module
  - Location: `crates/gdsr_core/src/elements/path/io/mod.rs`
  - Remove generic parameters
  - Update serialization/deserialization

- [ ] Update Polygon IO module
  - Location: `crates/gdsr_core/src/elements/polygon/io/mod.rs`
  - Remove generic parameters
  - Update serialization/deserialization

- [ ] Update Text IO module
  - Location: `crates/gdsr_core/src/elements/text/io/mod.rs`
  - Remove generic parameters
  - Update serialization/deserialization

### Phase 10: Re-implement Polygon Utilities ⏸️ POSTPONED
- [ ] Implement area calculation
- [ ] Implement perimeter calculation
- [ ] Implement is_point_inside method
- [ ] Implement is_point_on_edge method
- [ ] Implement bounding_box method

### Phase 11: Testing Improvements 🔜 TODO
- [ ] Add `insta` crate to dependencies
- [ ] Add snapshot tests for Display implementations
- [ ] Add snapshot tests for Debug implementations
- [ ] Restore comprehensive transformation tests after traits are re-implemented
- [ ] Add integration tests for end-to-end workflows

## Files Modified So Far

### Core Files
- `crates/gdsr_core/src/units.rs` - Added Unit enum, Display impl
- `crates/gdsr_core/src/point/mod.rs` - Updated Point struct, Display impl
- `crates/gdsr_core/src/lib.rs` - Added exports for Point, Unit, Grid, elements

### Element Files
- `crates/gdsr_core/src/elements/mod.rs` - Updated exports
- `crates/gdsr_core/src/elements/path/mod.rs` - Removed generics
- `crates/gdsr_core/src/elements/polygon/mod.rs` - Removed generics
- `crates/gdsr_core/src/elements/polygon/utils.rs` - Removed generics
- `crates/gdsr_core/src/elements/text/mod.rs` - Removed generics
- `crates/gdsr_core/src/grid/mod.rs` - Removed generics

## Key Technical Decisions

### Unit Comparison
- Custom PartialEq compares Units by real-world values (value * scale)
- Uses epsilon tolerance of 1e-15 for floating-point comparison
- Allows comparison between different unit scales

### Default Units
- Integer default: db_unit = 1e-9
- Float default: user_unit = 1e-6, db_unit = 1e-9

### Point API
- Fields are private (x, y)
- Getters: x(), y()
- Setters: set_x(), set_y()
- Mutable getters: x_mut(), y_mut()
- new() accepts impl Into<Unit> for ergonomics

### Incremental Migration Strategy
1. Update core types first (Unit, Point)
2. Update element types one by one
3. Comment out trait implementations temporarily
4. Simplify tests to basic functionality
5. Update traits and transformations
6. Re-implement trait methods for all types
7. Restore comprehensive tests
8. Re-enable IO modules

## Test Status
- Total tests passing: 173
- Grid tests: 5/5 passing
- Cell tests: 7/7 passing
- Element tests: 4/4 passing
- Instance tests: 6/6 passing
- Reference tests: 5/5 passing
- Point tests: 80+ passing
- Unit tests: 60+ passing
- All element type tests: Passing

## Next Steps
1. **SKIP traits and transformation modules for now** - Leave commented out
2. Identify and update other modules that depend on Point/Unit (Phase 7)
   - Config module
   - Library module
   - Any utility modules
3. Add comprehensive tests for existing updated structures (Phase 11)
4. Add snapshot tests using insta crate (Phase 11)
5. **LATER**: Update transformation module (Phase 6)
6. **LATER**: Re-enable traits module (Phase 5)
7. **LATER**: Re-implement trait methods for all elements (Phase 8)
8. **LATER**: Re-implement IO modules (Phase 9)

## Notes for Next Session
- All basic structures compile and test successfully (173 tests passing!)
- All core element types updated: Path, Polygon, Text, Grid, Cell, Reference, Instance
- Element enum complete with all 4 variants
- Display implementations added to: Unit, Point, Grid, Cell, Element, Instance, Reference
- All struct migrations follow same pattern: remove generics, simplify, add Display, add basic tests
- **Traits and transformation modules temporarily commented out** - will tackle later
- Focus on other modules that use Point/Unit: config, library, utilities
- Consider adding snapshot tests for Display/Debug implementations
- Look for any other structs/modules that reference Point generics

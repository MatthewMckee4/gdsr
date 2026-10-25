# Hierarchy bounds

Use `Library::hierarchy_bounds` to calculate axis-aligned bounds for a cell and
the hierarchy reachable from it:

```rs
let report = library.hierarchy_bounds("top")?;

if let Some(bounds) = report.root_bounds() {
    println!(
        "({}, {}) to ({}, {})",
        bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y
    );
}
```

Coordinates are physical world values. Cell bounds use each cell's local
coordinate space. Reference bounds use the local coordinate space of the cell
containing that reference.

The report includes bounds for every reachable cell and one aggregate bound for
each reference element. An array reference has one entry covering its whole
grid; it is not expanded into individual members. Empty cells and references
without geometry have `None` bounds.

Dangling references and hierarchy cycles do not fail the query. Recursive
traversal stops at repeated cells. Bounds for cyclic cells conservatively cover
one finite traversal through the strongly connected component and remain
independent of the requested root and traversal order. One cycle diagnostic
lists the sorted component cells and a representative cycle edge. Valid sibling
geometry remains in the result.

Elements with non-finite coordinates or reference transforms are also omitted
and diagnosed, so a report never contains `NaN` or infinite bounds. Path widths
use their physical magnitude. Positive path extensions conservatively expand
the axis-aligned bounds; negative extensions never shrink them.

Requesting an unknown root is the only fatal error.

Each call creates a detached snapshot with a query-local memo. Call the method
again after mutating the library; no cache invalidation is required.

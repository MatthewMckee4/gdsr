use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use gdsr::Library;

/// A node in the cell hierarchy tree.
#[derive(Clone, Debug, PartialEq)]
pub struct CellTreeNode {
    pub name: String,
    pub children: Vec<Self>,
}

/// Builds a cell hierarchy tree from a library.
///
/// Root nodes are cells that are not referenced by any other cell.
/// Children are cells that are directly referenced by the parent cell.
/// Cells that appear as children of multiple parents are duplicated in the tree.
/// Cycles are broken by not revisiting cells already on the current path.
pub fn build_cell_tree(library: &Library) -> Vec<CellTreeNode> {
    let cells = library.cells();

    // Build parent -> direct children mapping
    let mut children_of: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut referenced: HashSet<&str> = HashSet::new();

    for (name, cell) in cells {
        let entry = children_of.entry(name.as_str()).or_default();
        for ref_name in cell.referenced_cell_names() {
            if cells.contains_key(ref_name) {
                entry.insert(ref_name);
                referenced.insert(ref_name);
            }
        }
    }

    // Root cells: exist in library but are not referenced by any other cell
    let roots: BTreeSet<&str> = cells
        .keys()
        .map(String::as_str)
        .filter(|name| !referenced.contains(name))
        .collect();

    roots
        .into_iter()
        .map(|name| build_node(name, &children_of, &mut HashSet::new()))
        .collect()
}

fn build_node<'a>(
    name: &'a str,
    children_of: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    ancestors: &mut HashSet<&'a str>,
) -> CellTreeNode {
    ancestors.insert(name);
    let children = if let Some(kids) = children_of.get(name) {
        let valid_kids: Vec<&str> = kids
            .iter()
            .filter(|kid| !ancestors.contains(**kid))
            .copied()
            .collect();
        valid_kids
            .into_iter()
            .map(|kid| build_node(kid, children_of, ancestors))
            .collect()
    } else {
        Vec::new()
    };
    ancestors.remove(name);

    CellTreeNode {
        name: name.to_string(),
        children,
    }
}

/// Builds a flat cell list where every cell in the library is a top-level node,
/// each with its direct children (references) as subtree entries.
pub fn build_flat_cell_tree(library: &Library) -> Vec<CellTreeNode> {
    let cells = library.cells();

    let mut children_of: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (name, cell) in cells {
        let entry = children_of.entry(name.as_str()).or_default();
        for ref_name in cell.referenced_cell_names() {
            if cells.contains_key(ref_name) {
                entry.insert(ref_name);
            }
        }
    }

    let mut names: Vec<&str> = cells.keys().map(String::as_str).collect();
    names.sort_unstable();

    names
        .into_iter()
        .map(|name| build_node(name, &children_of, &mut HashSet::new()))
        .collect()
}

/// Returns a filtered copy of the tree containing only nodes whose name matches
/// the query (case-insensitive substring) or that have a matching descendant.
/// An empty query returns a clone of the full tree.
#[cfg(test)]
pub fn filter_tree(tree: &[CellTreeNode], query: &str) -> Vec<CellTreeNode> {
    if query.is_empty() {
        return tree.to_vec();
    }
    let query_lower = query.to_lowercase();
    tree.iter()
        .filter_map(|node| filter_node(node, &query_lower))
        .collect()
}

#[cfg(test)]
fn filter_node(node: &CellTreeNode, query_lower: &str) -> Option<CellTreeNode> {
    let name_matches = node.name.to_lowercase().contains(query_lower);
    let filtered_children: Vec<CellTreeNode> = node
        .children
        .iter()
        .filter_map(|child| filter_node(child, query_lower))
        .collect();

    if name_matches || !filtered_children.is_empty() {
        Some(CellTreeNode {
            name: node.name.clone(),
            children: filtered_children,
        })
    } else {
        None
    }
}

/// Tracks which nodes in the hierarchy tree are expanded.
#[derive(Clone, Debug, Default)]
pub struct ExpandState {
    expanded: HashMap<String, bool>,
}

impl ExpandState {
    pub fn is_expanded(&self, name: &str) -> bool {
        self.expanded.get(name).copied().unwrap_or(false)
    }

    #[cfg(test)]
    fn toggle(&mut self, name: &str) {
        let entry = self.expanded.entry(name.to_string()).or_insert(false);
        *entry = !*entry;
    }

    pub fn set_expanded(&mut self, name: &str, expanded: bool) {
        self.expanded.insert(name.to_string(), expanded);
    }

    #[cfg(test)]
    pub fn expand_all(&mut self, tree: &[CellTreeNode]) {
        for node in tree {
            self.set_expanded(&node.name, true);
            self.expand_all(&node.children);
        }
    }

    #[cfg(test)]
    pub fn collapse_all(&mut self, tree: &[CellTreeNode]) {
        for node in tree {
            self.set_expanded(&node.name, false);
            self.collapse_all(&node.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use gdsr::{Cell, DataType, Layer, Library, Point, Polygon, Reference};

    use super::*;

    fn visible_names(tree: &[CellTreeNode], expand_state: &ExpandState) -> Vec<String> {
        let mut result = Vec::new();
        for node in tree {
            collect_visible(node, expand_state, &mut result);
        }
        result
    }

    fn collect_visible(node: &CellTreeNode, expand_state: &ExpandState, result: &mut Vec<String>) {
        result.push(node.name.clone());
        if expand_state.is_expanded(&node.name) {
            for child in &node.children {
                collect_visible(child, expand_state, result);
            }
        }
    }

    fn node_depth(tree: &[CellTreeNode], name: &str) -> Option<usize> {
        for node in tree {
            if let Some(depth) = find_depth(node, name, 0) {
                return Some(depth);
            }
        }
        None
    }

    fn find_depth(node: &CellTreeNode, name: &str, depth: usize) -> Option<usize> {
        if node.name == name {
            return Some(depth);
        }
        for child in &node.children {
            if let Some(d) = find_depth(child, name, depth + 1) {
                return Some(d);
            }
        }
        None
    }

    fn make_library(cells: Vec<(&str, Vec<&str>)>) -> Library {
        let mut library = Library::new("test");
        for (name, refs) in cells {
            let mut cell = Cell::new(name);
            for r in refs {
                cell.add(Reference::new(r.to_string()));
            }
            library.add_cell(cell);
        }
        library
    }

    #[test]
    fn empty_library_produces_empty_tree() {
        let library = Library::new("empty");
        let tree = build_cell_tree(&library);
        assert!(tree.is_empty());
    }

    #[test]
    fn single_cell_is_root() {
        let library = make_library(vec![("top", vec![])]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [],
            },
        ]
        "#);
    }

    #[test]
    fn linear_chain() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [
                    CellTreeNode {
                        name: "mid",
                        children: [
                            CellTreeNode {
                                name: "bot",
                                children: [],
                            },
                        ],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn multiple_roots() {
        let library = make_library(vec![
            ("a", vec!["shared"]),
            ("b", vec!["shared"]),
            ("shared", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "a",
                children: [
                    CellTreeNode {
                        name: "shared",
                        children: [],
                    },
                ],
            },
            CellTreeNode {
                name: "b",
                children: [
                    CellTreeNode {
                        name: "shared",
                        children: [],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn diamond_hierarchy() {
        let library = make_library(vec![
            ("top", vec!["left", "right"]),
            ("left", vec!["bottom"]),
            ("right", vec!["bottom"]),
            ("bottom", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [
                    CellTreeNode {
                        name: "left",
                        children: [
                            CellTreeNode {
                                name: "bottom",
                                children: [],
                            },
                        ],
                    },
                    CellTreeNode {
                        name: "right",
                        children: [
                            CellTreeNode {
                                name: "bottom",
                                children: [],
                            },
                        ],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn cycle_is_broken() {
        let library = make_library(vec![("a", vec!["b"]), ("b", vec!["a"])]);
        let tree = build_cell_tree(&library);
        // Both reference each other so neither is purely a leaf — both appear as roots
        // since both are referenced. Actually: a is referenced by b, b is referenced by a,
        // so neither is a root... but both are referenced, meaning no roots exist.
        // In that case, the tree is empty. Let's verify.
        insta::assert_debug_snapshot!(tree, @"[]");
    }

    #[test]
    fn self_referencing_cycle() {
        let library = make_library(vec![("loop", vec!["loop"])]);
        let tree = build_cell_tree(&library);
        // "loop" references itself, so it's in the referenced set → not a root
        insta::assert_debug_snapshot!(tree, @"[]");
    }

    #[test]
    fn dangling_references_are_ignored() {
        let library = make_library(vec![("top", vec!["missing"])]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [],
            },
        ]
        "#);
    }

    #[test]
    fn reference_with_inline_element_not_counted_as_child() {
        let units = 1e-9;
        let mut library = Library::new("test");
        let mut cell = Cell::new("top");
        cell.add(Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        )));
        library.add_cell(cell);

        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [],
            },
        ]
        "#);
    }

    #[test]
    fn expand_state_defaults_to_collapsed() {
        let state = ExpandState::default();
        assert!(!state.is_expanded("anything"));
    }

    #[test]
    fn toggle_expand_state() {
        let mut state = ExpandState::default();
        state.toggle("cell_a");
        assert!(state.is_expanded("cell_a"));
        state.toggle("cell_a");
        assert!(!state.is_expanded("cell_a"));
    }

    #[test]
    fn expand_all_and_collapse_all() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let mut state = ExpandState::default();

        state.expand_all(&tree);
        assert!(state.is_expanded("top"));
        assert!(state.is_expanded("mid"));
        assert!(state.is_expanded("bot"));

        state.collapse_all(&tree);
        assert!(!state.is_expanded("top"));
        assert!(!state.is_expanded("mid"));
        assert!(!state.is_expanded("bot"));
    }

    #[test]
    fn visible_names_collapsed() {
        let library = make_library(vec![("top", vec!["child"]), ("child", vec![])]);
        let tree = build_cell_tree(&library);
        let state = ExpandState::default();

        let names = visible_names(&tree, &state);
        insta::assert_debug_snapshot!(names, @r#"
        [
            "top",
        ]
        "#);
    }

    #[test]
    fn visible_names_expanded() {
        let library = make_library(vec![
            ("top", vec!["child"]),
            ("child", vec!["grandchild"]),
            ("grandchild", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let mut state = ExpandState::default();
        state.expand_all(&tree);

        let names = visible_names(&tree, &state);
        insta::assert_debug_snapshot!(names, @r#"
        [
            "top",
            "child",
            "grandchild",
        ]
        "#);
    }

    #[test]
    fn visible_names_partially_expanded() {
        let library = make_library(vec![
            ("top", vec!["a", "b"]),
            ("a", vec!["a_child"]),
            ("a_child", vec![]),
            ("b", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let mut state = ExpandState::default();
        state.set_expanded("top", true);

        let names = visible_names(&tree, &state);
        insta::assert_debug_snapshot!(names, @r#"
        [
            "top",
            "a",
            "b",
        ]
        "#);
    }

    #[test]
    fn node_depth_roots_are_zero() {
        let library = make_library(vec![("top", vec!["child"]), ("child", vec![])]);
        let tree = build_cell_tree(&library);
        assert_eq!(node_depth(&tree, "top"), Some(0));
    }

    #[test]
    fn node_depth_children() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        assert_eq!(node_depth(&tree, "top"), Some(0));
        assert_eq!(node_depth(&tree, "mid"), Some(1));
        assert_eq!(node_depth(&tree, "bot"), Some(2));
    }

    #[test]
    fn node_depth_missing_returns_none() {
        let library = make_library(vec![("top", vec![])]);
        let tree = build_cell_tree(&library);
        assert_eq!(node_depth(&tree, "nonexistent"), None);
    }

    #[test]
    fn all_cells_without_references_are_roots() {
        let library = make_library(vec![("a", vec![]), ("b", vec![]), ("c", vec![])]);
        let tree = build_cell_tree(&library);
        let root_names: Vec<&str> = tree.iter().map(|n| n.name.as_str()).collect();
        insta::assert_debug_snapshot!(root_names, @r#"
        [
            "a",
            "b",
            "c",
        ]
        "#);
    }

    #[test]
    fn tree_with_fan_out() {
        let library = make_library(vec![
            ("top", vec!["a", "b", "c"]),
            ("a", vec![]),
            ("b", vec![]),
            ("c", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 3);
        let child_names: Vec<&str> = tree[0].children.iter().map(|n| n.name.as_str()).collect();
        insta::assert_debug_snapshot!(child_names, @r#"
        [
            "a",
            "b",
            "c",
        ]
        "#);
    }

    #[test]
    fn longer_cycle_is_broken() {
        let library = make_library(vec![
            ("root", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["a"]),
        ]);
        let tree = build_cell_tree(&library);
        insta::assert_debug_snapshot!(tree, @r#"
        [
            CellTreeNode {
                name: "root",
                children: [
                    CellTreeNode {
                        name: "a",
                        children: [
                            CellTreeNode {
                                name: "b",
                                children: [
                                    CellTreeNode {
                                        name: "c",
                                        children: [],
                                    },
                                ],
                            },
                        ],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn set_expanded_directly() {
        let mut state = ExpandState::default();
        state.set_expanded("x", true);
        assert!(state.is_expanded("x"));
        state.set_expanded("x", false);
        assert!(!state.is_expanded("x"));
    }

    #[test]
    fn visible_names_empty_tree() {
        let tree: Vec<CellTreeNode> = vec![];
        let state = ExpandState::default();
        assert!(visible_names(&tree, &state).is_empty());
    }

    #[test]
    fn node_depth_empty_tree() {
        let tree: Vec<CellTreeNode> = vec![];
        assert_eq!(node_depth(&tree, "anything"), None);
    }

    #[test]
    fn filter_tree_empty_query_returns_full_tree() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let filtered = filter_tree(&tree, "");
        assert_eq!(filtered, tree);
    }

    #[test]
    fn filter_tree_partial_match() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let filtered = filter_tree(&tree, "mi");
        insta::assert_debug_snapshot!(filtered, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [
                    CellTreeNode {
                        name: "mid",
                        children: [],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn filter_tree_no_match_returns_empty() {
        let library = make_library(vec![("top", vec!["child"]), ("child", vec![])]);
        let tree = build_cell_tree(&library);
        let filtered = filter_tree(&tree, "xyz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_tree_deep_match_keeps_ancestors() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["deep_target"]),
            ("deep_target", vec![]),
        ]);
        let tree = build_cell_tree(&library);
        let filtered = filter_tree(&tree, "deep");
        insta::assert_debug_snapshot!(filtered, @r#"
        [
            CellTreeNode {
                name: "top",
                children: [
                    CellTreeNode {
                        name: "mid",
                        children: [
                            CellTreeNode {
                                name: "deep_target",
                                children: [],
                            },
                        ],
                    },
                ],
            },
        ]
        "#);
    }

    #[test]
    fn filter_tree_case_insensitive() {
        let library = make_library(vec![("MyCell", vec![])]);
        let tree = build_cell_tree(&library);
        let filtered = filter_tree(&tree, "mycell");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "MyCell");
    }

    #[test]
    fn duplicate_references_produce_single_child() {
        let mut library = Library::new("test");
        let mut cell = Cell::new("top");
        cell.add(Reference::new("child".to_string()));
        cell.add(Reference::new("child".to_string()));
        library.add_cell(cell);
        library.add_cell(Cell::new("child"));

        let tree = build_cell_tree(&library);
        assert_eq!(tree[0].children.len(), 1);
    }

    #[test]
    fn flat_tree_lists_all_cells_as_roots() {
        let library = make_library(vec![
            ("top", vec!["mid"]),
            ("mid", vec!["bot"]),
            ("bot", vec![]),
        ]);
        let flat = build_flat_cell_tree(&library);
        let root_names: Vec<&str> = flat.iter().map(|n| n.name.as_str()).collect();
        insta::assert_debug_snapshot!(root_names, @r#"
        [
            "bot",
            "mid",
            "top",
        ]
        "#);
        assert!(flat[0].children.is_empty());
        assert_eq!(flat[1].children.len(), 1);
        assert_eq!(flat[2].children.len(), 1);
    }
}

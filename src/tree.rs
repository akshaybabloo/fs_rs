use std::collections::BTreeMap;
use std::path::Path;

use colored::Colorize;
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;

use crate::utils;

/// A node in the tree structure
#[derive(Debug, Default)]
struct TreeNode {
    size: u64,
    is_dir: bool,
    children: BTreeMap<String, TreeNode>,
}

/// Recursively collect all files and directories with their sizes
fn collect_entries(
    path: &Path,
    base_path: &Path,
    depth: usize,
    max_depth: usize,
) -> Vec<(String, u64, bool)> {
    if depth > max_depth {
        return vec![];
    }

    let mut entries = vec![];

    if let Ok(read_dir) = std::fs::read_dir(path) {
        let dir_entries: Vec<_> = read_dir.filter_map(Result::ok).collect();

        let results: Vec<_> = dir_entries
            .par_iter()
            .flat_map(|entry| {
                let entry_path = entry.path();
                let relative_path = entry_path
                    .strip_prefix(base_path)
                    .unwrap_or(&entry_path)
                    .to_string_lossy()
                    .to_string();

                let mut local_entries = vec![];

                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        local_entries.push((relative_path, metadata.len(), false));
                    } else if metadata.is_dir() {
                        let dir_size = utils::calculate_dir_size(&entry_path);
                        local_entries.push((relative_path.clone(), dir_size, true));

                        if depth < max_depth {
                            let sub_entries =
                                collect_entries(&entry_path, base_path, depth + 1, max_depth);
                            local_entries.extend(sub_entries);
                        }
                    }
                }
                local_entries
            })
            .collect();

        entries.extend(results);
    }

    entries
}

/// Build a tree structure from flat paths
fn build_tree(entries: Vec<(String, u64, bool)>) -> TreeNode {
    let mut root = TreeNode::default();

    for (path, size, is_dir) in entries {
        let parts: Vec<&str> = path.split(std::path::MAIN_SEPARATOR).collect();
        let mut current = &mut root;

        for (i, part) in parts.iter().enumerate() {
            current = current.children.entry(part.to_string()).or_default();

            if i == parts.len() - 1 {
                current.size = size;
                current.is_dir = is_dir;
            }
        }
    }

    root
}

/// Render a tree node recursively
fn render_tree(node: &TreeNode, prefix: &str, ascii: bool) -> String {
    let mut output = String::new();

    let mut children: Vec<_> = node.children.iter().collect();
    children.sort_by(|a, b| a.0.cmp(b.0));

    let (branch_last, branch_mid, pipe) = if ascii {
        ("`-- ", "+-- ", "|   ")
    } else {
        ("└── ", "├── ", "│   ")
    };

    for (i, (name, child)) in children.iter().enumerate() {
        let is_last_child = i == children.len() - 1;
        let branch = if is_last_child {
            branch_last
        } else {
            branch_mid
        };
        let size_str = format_size(child.size, DECIMAL);

        let formatted_line = if child.is_dir {
            format!(
                "{}{}{}/  ({})\n",
                prefix,
                branch,
                name.blue(),
                size_str.blue()
            )
        } else {
            format!(
                "{}{}{}{}  ({})\n",
                prefix,
                branch,
                name.green(),
                "*".green(),
                size_str.green()
            )
        };

        output.push_str(&formatted_line);

        if !child.children.is_empty() {
            let new_prefix = if is_last_child {
                format!("{}    ", prefix)
            } else {
                format!("{}{}", prefix, pipe)
            };
            output.push_str(&render_tree(child, &new_prefix, ascii));
        }
    }

    output
}

/// Generates a tree representation of a given path.
///
/// # Arguments
///
/// * `path` - The path to generate tree for.
/// * `depth` - An optional depth limit for the tree representation.
/// * `ascii` - Whether to use ASCII characters instead of Unicode.
///
/// # Returns
///
/// * A String representing the tree structure.
pub fn generate_tree(path: &Path, depth: Option<usize>, ascii: bool) -> String {
    let max_depth = depth.unwrap_or(usize::MAX);
    let entries = collect_entries(path, path, 1, max_depth);
    let tree = build_tree(entries);
    render_tree(&tree, "", ascii)
}

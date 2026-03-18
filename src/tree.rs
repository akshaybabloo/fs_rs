use std::collections::BTreeMap;
use std::path::Path;

use colored::Colorize;
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;

use crate::utils;

/// A flat entry: (relative_path, size_in_bytes, is_directory)
type Entry = (String, u64, bool);

/// A node in the tree structure
#[derive(Debug, Default)]
struct TreeNode {
    size: u64,
    is_dir: bool,
    children: BTreeMap<String, TreeNode>,
}

/// Recursively collect all files and directories with their sizes.
///
/// Returns `(entries, total_size)` where entries is a flat list of
/// `(relative_path, size, is_dir)` tuples and total_size is the sum of
/// all content under `path`. This avoids double-traversal by computing
/// directory sizes from the recursive results rather than calling
/// `calculate_dir_size` separately.
fn collect_entries(
    path: &Path,
    base_path: &Path,
    depth: usize,
    max_depth: usize,
) -> (Vec<Entry>, u64) {
    if depth > max_depth {
        return (vec![], 0);
    }

    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return (vec![], 0),
    };

    let dir_entries: Vec<_> = read_dir.filter_map(Result::ok).collect();

    let results: Vec<(Vec<Entry>, u64)> = dir_entries
        .par_iter()
        .map(|entry| {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => return (vec![], 0),
            };

            if file_type.is_symlink() {
                return (vec![], 0);
            }

            let entry_path = entry.path();
            let relative_path = entry_path
                .strip_prefix(base_path)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .to_string();

            if file_type.is_file() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                (vec![(relative_path, size, false)], size)
            } else if file_type.is_dir() {
                if depth < max_depth {
                    // Recurse: collect children and derive size from them
                    let (sub_entries, dir_size) =
                        collect_entries(&entry_path, base_path, depth + 1, max_depth);
                    let mut entries = vec![(relative_path, dir_size, true)];
                    entries.extend(sub_entries);
                    (entries, dir_size)
                } else {
                    // At max depth: compute size without expanding children
                    let dir_size = utils::calculate_dir_size(&entry_path);
                    (vec![(relative_path, dir_size, true)], dir_size)
                }
            } else {
                (vec![], 0)
            }
        })
        .collect();

    let mut all_entries = Vec::new();
    let mut total_size = 0u64;
    for (entries, size) in results {
        all_entries.extend(entries);
        total_size += size;
    }

    (all_entries, total_size)
}

/// Build a tree structure from flat paths
fn build_tree(entries: &[Entry]) -> TreeNode {
    let mut root = TreeNode::default();

    for (path, size, is_dir) in entries {
        let parts: Vec<&str> = path.split(std::path::MAIN_SEPARATOR).collect();
        let mut current = &mut root;

        for (i, part) in parts.iter().enumerate() {
            current = current.children.entry(part.to_string()).or_default();

            if i == parts.len() - 1 {
                current.size = *size;
                current.is_dir = *is_dir;
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
    let (entries, _) = collect_entries(path, path, 1, max_depth);
    let tree = build_tree(&entries);
    render_tree(&tree, "", ascii)
}

use colored::Colorize;
use comfy_table::{Cell, Table};
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;
use std::fs;
use std::path::Path;

const MAX_FILENAME_LENGTH: usize = 45;

/// Struct to hold sizes of files/directories
#[derive(Clone, Debug, PartialEq)]
pub struct Sizes {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// Returns true if `name` matches any of the provided ignore patterns.
///
/// Matching is plain basename equality — a pattern like `node_modules`
/// will match any file or directory whose name is `node_modules`.
///
/// # Examples
///
/// ```
/// let patterns = vec!["target".to_string(), ".git".to_string()];
/// assert!(fs_rs::utils::is_ignored("target", &patterns));
/// assert!(!fs_rs::utils::is_ignored("src", &patterns));
/// ```
pub fn is_ignored(name: &str, ignore: &[String]) -> bool {
    ignore.iter().any(|p| p == name)
}

/// Calculate directory size in parallel, skipping symlinks.
///
/// Equivalent to [`calculate_dir_size_with_ignore`] with an empty ignore list.
pub fn calculate_dir_size(dir_path: &Path) -> u64 {
    calculate_dir_size_with_ignore(dir_path, &[])
}

/// Calculate directory size in parallel, skipping symlinks and any entry
/// whose basename matches one of the `ignore` patterns.
///
/// # Arguments
///
/// * `dir_path`: Path to the directory
/// * `ignore`: List of file/directory names to skip
///
/// returns: u64 - The size of the directory in bytes
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let dir_path = Path::new("/some/directory");
/// let ignore = vec!["node_modules".to_string()];
/// let size = fs_rs::utils::calculate_dir_size_with_ignore(dir_path, &ignore);
/// ```
pub fn calculate_dir_size_with_ignore(dir_path: &Path, ignore: &[String]) -> u64 {
    fs::read_dir(dir_path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .par_bridge()
                .map(|entry| {
                    let file_type = match entry.file_type() {
                        Ok(ft) => ft,
                        Err(_) => return 0,
                    };
                    if file_type.is_symlink() {
                        return 0;
                    }
                    if let Some(name) = entry.file_name().to_str()
                        && is_ignored(name, ignore)
                    {
                        return 0;
                    }
                    if file_type.is_dir() {
                        calculate_dir_size_with_ignore(&entry.path(), ignore)
                    } else {
                        entry.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Sort sizes by size in descending order (in-place)
///
/// # Arguments
///
/// * `sizes`: A mutable slice of Sizes
///
/// # Examples
///
/// ```
/// let mut sizes = vec![
///     fs_rs::utils::Sizes{name: "file1.txt".to_string(), size: 100, is_dir: false},
///     fs_rs::utils::Sizes{name: "file2.txt".to_string(), size: 200, is_dir: false},
/// ];
///
/// fs_rs::utils::sort_by_size(&mut sizes);
/// assert_eq!(sizes[0].size, 200);
/// ```
pub fn sort_by_size(sizes: &mut [Sizes]) {
    sizes.sort_by(|a, b| b.size.cmp(&a.size));
}

/// Sort sizes by name in ascending order (in-place)
///
/// # Arguments
///
/// * `sizes`: A mutable slice of Sizes
///
/// # Examples
///
/// ```
/// let mut sizes = vec![
///     fs_rs::utils::Sizes{name: "file2.txt".to_string(), size: 200, is_dir: false},
///     fs_rs::utils::Sizes{name: "file1.txt".to_string(), size: 100, is_dir: false},
/// ];
///
/// fs_rs::utils::sort_by_name(&mut sizes);
/// assert_eq!(sizes[0].name, "file1.txt");
/// ```
pub fn sort_by_name(sizes: &mut [Sizes]) {
    sizes.sort_by(|a, b| a.name.cmp(&b.name));
}

/// Truncate a filename so the stem fits within `MAX_FILENAME_LENGTH` visible
/// characters by removing the middle, keeping both the start and end intact.
///
/// # Arguments
///
/// * `path`: Path of the file
///
/// returns: String
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let path = Path::new("this_is_a_long_filename.txt");
/// let truncated = fs_rs::utils::truncate_filename(path);
/// ```
pub fn truncate_filename(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let truncated_stem = if stem.len() > MAX_FILENAME_LENGTH {
        let prefix_len = MAX_FILENAME_LENGTH / 2;
        let suffix_len = MAX_FILENAME_LENGTH - prefix_len;
        let prefix = &stem[..prefix_len];
        let suffix = &stem[stem.len() - suffix_len..];
        format!("{}...{}", prefix, suffix)
    } else {
        stem.to_string()
    };

    if !extension.is_empty() {
        format!("{}.{}", truncated_stem, extension)
    } else {
        truncated_stem
    }
}

/// Add rows to a table from a slice of Sizes
pub fn add_row(table: &mut Table, values: &[Sizes]) {
    for s in values {
        let sz = format_size(s.size, DECIMAL);

        let (name_cell, size_cell) = if s.is_dir {
            (
                Cell::new(format!("{}/", s.name.blue())),
                Cell::new(sz.blue().to_string()),
            )
        } else {
            (
                Cell::new(format!("{}*", s.name.green())),
                Cell::new(sz.green().to_string()),
            )
        };

        table.add_row(vec![name_cell, size_cell]);
    }
}

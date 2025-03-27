use colored::Colorize;
use comfy_table::{Cell, Table};
use humansize::{DECIMAL, format_size};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Get the size of a directory
///
/// # Arguments
///
/// * `path`: Absolute path to the directory
///
/// returns: u64 - The size of the directory in bytes
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let size = fs_rs::utils::dir_size(Path::new("/some/path"));
/// ```
pub fn dir_size(path: &Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += dir_size(&path);
                }
            }
        }
    }
    total_size
}

/// Sort a HashMap by value
///
/// # Arguments
///
/// * `sizes`: A HashMap of file/directory names and their sizes
///
/// returns: Vec<(String, u64), Global>
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// let mut sizes: HashMap<String, u64> = HashMap::new();
/// sizes.insert("file1.txt".to_string(), 100);
/// sizes.insert("file2.txt".to_string(), 200);
///
/// let sorted_vec = fs_rs::utils::sort_by_size(&sizes);
/// ```
pub fn sort_by_size(sizes: &HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut sorted_vec: Vec<_> = sizes.iter().collect();
    sorted_vec.sort_by(|a, b| b.1.cmp(a.1));
    sorted_vec
        .into_iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

/// Sort a HashMap by key
///
/// # Arguments
///
/// * `sizes`: A HashMap of file/directory names and their sizes
///
/// returns: Vec<(String, u64), Global>
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// let mut sizes: HashMap<String, u64> = HashMap::new();
/// sizes.insert("file1.txt".to_string(), 100);
/// sizes.insert("file2.txt".to_string(), 200);
///
/// let sorted_vec = fs_rs::utils::sort_by_name(&sizes);
/// ```
pub fn sort_by_name(sizes: &HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut sorted_vec: Vec<_> = sizes.iter().collect();
    sorted_vec.sort_by(|a, b| a.0.cmp(b.0));
    sorted_vec
        .into_iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

/// Truncate a filename to 15 characters
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
    // Extract the file stem (name without extension) and extension
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Check if the stem length exceeds 15 characters
    let truncated_stem = if stem.len() > 15 {
        format!("{}...", &stem[..15])
    } else {
        stem.to_string()
    };

    // Append the extension if present
    if !extension.is_empty() {
        format!("{}.{}", truncated_stem, extension)
    } else {
        truncated_stem
    }
}

/// Add a row to a table
pub fn add_row(table: &mut Table, sorted_sizes: Vec<(String, u64)>) {
    for (root, size) in sorted_sizes {
        let sz = format_size(size, DECIMAL);

        let (name_cell, size_cell) = if root.ends_with("/") {
            (
                Cell::new(root.yellow().to_string()),
                Cell::new(sz.yellow().to_string()),
            )
        } else {
            (
                Cell::new(root.bright_blue().to_string()),
                Cell::new(sz.bright_blue().to_string()),
            )
        };

        table.add_row(vec![name_cell, size_cell]);
    }
}

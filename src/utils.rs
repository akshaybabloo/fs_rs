use colored::Colorize;
use comfy_table::{Cell, Table};
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;
use std::fs;
use std::path::Path;

const MAX_FILENAME_LENGTH: usize = 25;

/// Struct to hold sizes of files/directories
#[derive(Clone, Debug, PartialEq)]
pub struct Sizes {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// Calculate directory size in parallel
///
/// # Arguments
///
/// * `dir_path`: Path to the directory
///
/// returns: u64 - The size of the directory in bytes
pub fn calculate_dir_size(dir_path: &Path) -> u64 {
    fs::read_dir(dir_path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .par_bridge()
                .map(|entry| {
                    let path = entry.path();
                    if path.is_dir() {
                        calculate_dir_size(&path)
                    } else {
                        entry.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
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
/// let mut sizes: Vec<fs_rs::utils::Sizes> = Vec::new();
/// sizes.push(fs_rs::utils::Sizes{name: "file1.txt".to_string(), size: 100, is_dir: false});
/// sizes.push(fs_rs::utils::Sizes{name: "file2.txt".to_string(), size: 200, is_dir: false});
///
/// let sorted_vec = fs_rs::utils::sort_by_size(&sizes);
/// ```
pub fn sort_by_size(sizes: &[Sizes]) -> Vec<Sizes> {
    let mut sorted_vec: Vec<_> = sizes.iter().collect();
    sorted_vec.sort_by(|a, b| b.size.cmp(&a.size));
    sorted_vec.into_iter().cloned().collect()
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
/// let mut sizes: Vec<fs_rs::utils::Sizes> = Vec::new();
/// sizes.push(fs_rs::utils::Sizes{name: "file1.txt".to_string(), size: 100, is_dir: false});
/// sizes.push(fs_rs::utils::Sizes{name: "file2.txt".to_string(), size: 200, is_dir: false});
///
/// let sorted_vec = fs_rs::utils::sort_by_name(&sizes);
/// ```
pub fn sort_by_name(sizes: &[Sizes]) -> Vec<Sizes> {
    let mut sorted_vec: Vec<_> = sizes.iter().collect();
    sorted_vec.sort_by(|a, b| a.name.cmp(&b.name));
    sorted_vec.into_iter().cloned().collect()
}

/// Truncate a filename to `MAX_FILENAME_LENGTH` characters
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

    // Check if the stem length exceeds the maximum allowed length
    let truncated_stem = if stem.len() > MAX_FILENAME_LENGTH {
        format!("{}...", &stem[..MAX_FILENAME_LENGTH])
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
pub fn add_row(table: &mut Table, values: Vec<Sizes>) {
    for Sizes { name, size, is_dir } in values {
        let sz = format_size(size, DECIMAL);

        let (name_cell, size_cell) = if is_dir {
            (
                Cell::new(format!("{}/", name.blue().to_string())),
                Cell::new(sz.blue().to_string()),
            )
        } else {
            (
                Cell::new(format!("{}*", name.green().to_string())),
                Cell::new(sz.green().to_string()),
            )
        };

        table.add_row(vec![name_cell, size_cell]);
    }
}

use std::path::Path;
use walkdir::WalkDir;

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
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

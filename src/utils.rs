use std::path::{Path, PathBuf};

/// Get the root of a path
///
/// # Arguments
///
/// * `path`: String - The path to get the root of
///
/// returns: PathBuf
///
/// # Examples
///
/// ```
/// let root = fs_rs::utils::get_root("/some/path", Some(true));
/// ```
pub fn get_root(path: &str, get_first: Option<bool>) -> PathBuf {
    let first = get_first.unwrap_or(true);
    let path = Path::new(path);

    if first {
        let components: Vec<_> = path.components().take(2).collect();
        components.iter().collect()
    } else {
        if path.is_file() {
            path.parent().map_or(PathBuf::new(), ToOwned::to_owned)
        } else {
            path.components()
                .last()
                .map_or(PathBuf::new(), |comp| PathBuf::from(comp.as_os_str()))
        }
    }
}

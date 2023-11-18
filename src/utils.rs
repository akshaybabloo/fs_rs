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
/// let root = get_root(String::from("some/path"));
/// ```
pub fn get_root(path: &str, get_first: Option<bool>) -> PathBuf {
    let first = get_first.unwrap_or(true);

    let components: Vec<_> = Path::new(path).components().collect();

    if first {
        // Take the first two components (if available) and add them to root
        let mut root = PathBuf::new();
        if let Some(first_component) = components.get(0) {
            root.push(first_component.as_os_str());
            if let Some(second_component) = components.get(1) {
                root.push(second_component.as_os_str());
            }
        }
        root
    } else {
        // Take the last component (if available) and add it to root
        if let Some(last_component) = components.last() {
            PathBuf::from(last_component.as_os_str())
        } else {
            PathBuf::new()
        }
    }
}

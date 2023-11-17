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
pub(crate) fn get_root(path: &str) -> PathBuf {
    let mut components = Path::new(path).components();

    // Initialize an empty PathBuf
    let mut root = PathBuf::new();

    // Take the first two components and add them to root
    if let Some(first_component) = components.next() {
        root.push(first_component.as_os_str());
        if let Some(second_component) = components.next() {
            root.push(second_component.as_os_str());
        }
    }

    root
}

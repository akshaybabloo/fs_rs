use fs_rs::utils;
use std::path::PathBuf;

#[test]
fn test_get_root_first_component() {
    let path = "/some/path";
    let root = utils::get_root(path, Some(true));
    assert_eq!(root, PathBuf::from("/some"));
}

#[test]
fn test_get_root_last_component() {
    let path = "some/path";
    let root = utils::get_root(path, Some(false));
    assert_eq!(root, PathBuf::from("path"));
}

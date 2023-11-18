use fs_rs::utils;
use std::path::PathBuf;

use std::env;
use std::fs::File;
use std::io::Write;

fn create_temp_file() -> std::io::Result<PathBuf> {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("test_get_root_last_component_with_file.txt");

    let mut file = File::create(&temp_dir)?;
    writeln!(file, "Temporary file content")?;

    Ok(temp_dir)
}

#[test]
fn test_get_root_first_component() {
    let path = "/some/path";
    let root = utils::get_root(path, Some(true));
    assert_eq!(root, PathBuf::from("/some"));
}

#[test]
fn test_get_root_last_component() {
    let path = "/some/path";
    let root = utils::get_root(path, Some(false));
    assert_eq!(
        root,
        PathBuf::from("path"),
        "Expected 'path', got {:?}",
        root
    );
}

#[test]
fn test_get_root_last_component_with_file() {
    let temp_file_path = create_temp_file().expect("Failed to create temporary file");
    let root = utils::get_root(temp_file_path.to_str().unwrap(), Some(false));

    // The expected result is the parent directory of the temporary file
    let expected = temp_file_path.parent().unwrap();
    assert_eq!(root, expected, "Expected {:?}, got {:?}", expected, root);

    // Clean up the temporary file
    std::fs::remove_file(temp_file_path).expect("Failed to delete temporary file");
}

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

use fs_rs::utils::dir_size;

#[test]
fn test_dir_size() {
    let dir = tempdir().expect("Failed to create a temporary directory");
    let file_path1 = dir.path().join("file1.txt");
    let file_path2 = dir.path().join("file2.txt");

    let mut file1 = File::create(file_path1).expect("Failed to create file1");
    let mut file2 = File::create(file_path2).expect("Failed to create file2");

    writeln!(file1, "Hello").expect("Failed to write to file1");
    writeln!(file2, "Hello, Rust!").expect("Failed to write to file2");

    let size = dir_size(dir.path());

    // The size may vary
    assert!(
        size > 10,
        "Expected size to be greater than 10, but got {}",
        size
    );

    dir.close()
        .expect("Failed to delete the temporary directory");
}

#[test]
fn test_sort_by_size() {
    let left: HashMap<String, u64> = HashMap::from([
        ("file1.txt".to_string(), 100),
        ("file2.txt".to_string(), 200),
    ]);

    let right: Vec<(String, u64)> = vec![
        ("file2.txt".to_string(), 200),
        ("file1.txt".to_string(), 100),
    ];

    let sorted_vec = fs_rs::utils::sort_by_size(&left);

    assert_eq!(
        sorted_vec, right,
        "Expected {:?}, but got {:?}",
        right, sorted_vec
    );
}

#[test]
fn test_sort_by_name() {
    let left: HashMap<String, u64> = HashMap::from([
        ("file2.txt".to_string(), 200),
        ("file1.txt".to_string(), 100),
    ]);

    let right: Vec<(String, u64)> = vec![
        ("file1.txt".to_string(), 100),
        ("file2.txt".to_string(), 200),
    ];

    let sorted_vec = fs_rs::utils::sort_by_name(&left);

    assert_eq!(
        sorted_vec, right,
        "Expected {:?}, but got {:?}",
        right, sorted_vec
    );
}

#[test]
fn test_truncate_filename() {
    let path = Path::new("this_is_a_long_filename.txt");
    let truncated = fs_rs::utils::truncate_filename(path);
    let right = "this_is_a_long_....txt";

    assert_eq!(
        truncated, right,
        "Expected {:?}, but got {:?}",
        right, truncated
    );
}

use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

use fs_rs::utils::{Sizes, calculate_dir_size};

#[test]
fn test_calculate_dir_size() {
    let dir = tempdir().expect("Failed to create a temporary directory");
    let file_path1 = dir.path().join("file1.txt");
    let file_path2 = dir.path().join("file2.txt");

    let mut file1 = File::create(file_path1).expect("Failed to create file1");
    let mut file2 = File::create(file_path2).expect("Failed to create file2");

    writeln!(file1, "Hello").expect("Failed to write to file1");
    writeln!(file2, "Hello, Rust!").expect("Failed to write to file2");

    // Close file handles before calculating size (required on Windows)
    drop(file1);
    drop(file2);

    let size = calculate_dir_size(dir.path());

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
    let left = vec![
        Sizes {
            name: "file1.txt".to_string(),
            size: 100,
            is_dir: false,
        },
        Sizes {
            name: "file2.txt".to_string(),
            size: 200,
            is_dir: false,
        },
    ];

    let sorted_vec = fs_rs::utils::sort_by_size(&left);

    assert_eq!(sorted_vec.len(), 2);
    assert_eq!(sorted_vec[0].name, "file2.txt");
    assert_eq!(sorted_vec[0].size, 200);
    assert_eq!(sorted_vec[1].name, "file1.txt");
    assert_eq!(sorted_vec[1].size, 100);
}

#[test]
fn test_sort_by_name() {
    let left = vec![
        Sizes {
            name: "file2.txt".to_string(),
            size: 200,
            is_dir: false,
        },
        Sizes {
            name: "file1.txt".to_string(),
            size: 100,
            is_dir: false,
        },
    ];

    let sorted_vec = fs_rs::utils::sort_by_name(&left);

    assert_eq!(sorted_vec.len(), 2);
    assert_eq!(sorted_vec[0].name, "file1.txt");
    assert_eq!(sorted_vec[0].size, 100);
    assert_eq!(sorted_vec[1].name, "file2.txt");
    assert_eq!(sorted_vec[1].size, 200);
}

#[test]
fn test_truncate_filename() {
    let path = Path::new("this_is_a_long_filename_and_some_more_text_to_make_it_even_longer.txt");
    let truncated = fs_rs::utils::truncate_filename(path);
    let right = "this_is_a_long_filename_a....txt";

    assert_eq!(
        truncated, right,
        "Expected {:?}, but got {:?}",
        right, truncated
    );
}


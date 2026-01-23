use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use fs_rs::tree::generate_tree;

#[test]
fn test_generate_tree() {
    let dir = tempdir().expect("Failed to create a temporary directory");
    // Create structure: subdir/file1.txt and file2.txt
    // subdir comes before file2.txt alphabetically, so subdir is not last
    // This ensures the pipe character is used
    let subdir = dir.path().join("aaa_subdir");
    fs::create_dir(&subdir).expect("Failed to create subdir");
    let file_path1 = subdir.join("file1.txt");
    let file_path2 = dir.path().join("file2.txt");

    let mut file1 = File::create(&file_path1).expect("Failed to create file1");
    let mut file2 = File::create(&file_path2).expect("Failed to create file2");

    writeln!(file1, "Hello").expect("Failed to write to file1");
    writeln!(file2, "Hello, Rust!").expect("Failed to write to file2");

    // Close file handles before generating tree (required on Windows)
    drop(file1);
    drop(file2);

    let tree = generate_tree(dir.path(), None, false);

    assert!(tree.contains("file1.txt"));
    assert!(tree.contains("file2.txt"));

    let tree_ascii = generate_tree(dir.path(), None, true);

    assert!(tree_ascii.contains("file1.txt"));
    assert!(tree_ascii.contains("file2.txt"));
    assert!(tree_ascii.contains("|"));

    dir.close()
        .expect("Failed to delete the temporary directory");
}
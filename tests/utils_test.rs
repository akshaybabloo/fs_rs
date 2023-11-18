use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use fs_rs::utils::dir_size;

#[test]
fn test_dir_size() {
    let dir = tempdir().expect("Failed to create a temporary directory");
    let file_path1 = dir.path().join("file1.txt");
    let file_path2 = dir.path().join("file2.txt");

    let mut file1 = File::create(&file_path1).expect("Failed to create file1");
    let mut file2 = File::create(&file_path2).expect("Failed to create file2");

    writeln!(file1, "Hello").expect("Failed to write to file1");
    writeln!(file2, "Hello, Rust!").expect("Failed to write to file2");

    let size = dir_size(dir.path());

    // The size may vary
    assert_eq!(size, 19);

    dir.close()
        .expect("Failed to delete the temporary directory");
}

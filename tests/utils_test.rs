use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

use fs_rs::utils::{Sizes, calculate_dir_size, calculate_dir_size_with_ignore, is_ignored};

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
    let mut sizes = vec![
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

    fs_rs::utils::sort_by_size(&mut sizes);

    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0].name, "file2.txt");
    assert_eq!(sizes[0].size, 200);
    assert_eq!(sizes[1].name, "file1.txt");
    assert_eq!(sizes[1].size, 100);
}

#[test]
fn test_sort_by_name() {
    let mut sizes = vec![
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

    fs_rs::utils::sort_by_name(&mut sizes);

    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0].name, "file1.txt");
    assert_eq!(sizes[0].size, 100);
    assert_eq!(sizes[1].name, "file2.txt");
    assert_eq!(sizes[1].size, 200);
}

#[test]
fn test_is_ignored() {
    let patterns = vec!["target".to_string(), ".git".to_string()];
    assert!(is_ignored("target", &patterns));
    assert!(is_ignored(".git", &patterns));
    assert!(!is_ignored("src", &patterns));
    assert!(!is_ignored("", &patterns));
    assert!(!is_ignored("target", &[]));
}

#[test]
fn test_calculate_dir_size_with_ignore_skips_subdir() {
    let dir = tempdir().expect("Failed to create a temporary directory");
    let keep = dir.path().join("keep.txt");
    let mut keep_file = File::create(&keep).expect("Failed to create keep.txt");
    writeln!(keep_file, "keep").expect("write");
    drop(keep_file);

    let ignored_dir = dir.path().join("node_modules");
    std::fs::create_dir(&ignored_dir).expect("create ignored dir");
    let ignored_file = ignored_dir.join("big.bin");
    let mut big = File::create(&ignored_file).expect("create big");
    big.write_all(&[0u8; 4096]).expect("write big");
    drop(big);

    let with_ignore = calculate_dir_size_with_ignore(dir.path(), &["node_modules".to_string()]);
    let without_ignore = calculate_dir_size(dir.path());

    assert!(without_ignore >= with_ignore + 4096);
    assert!(with_ignore < 100, "only keep.txt should be counted");
}

#[test]
fn test_truncate_filename() {
    let path = Path::new("this_is_a_long_filename_and_some_more_text_to_make_it_even_longer.txt");
    let truncated = fs_rs::utils::truncate_filename(path);
    let right = "this_is_a_long_filena...o_make_it_even_longer.txt";

    assert_eq!(
        truncated, right,
        "Expected {:?}, but got {:?}",
        right, truncated
    );
}

#[test]
fn test_truncate_filename_short() {
    let path = Path::new("short.txt");
    assert_eq!(fs_rs::utils::truncate_filename(path), "short.txt");
}

#[test]
fn test_truncate_filename_no_extension() {
    // Short, no extension: returned verbatim.
    let path = Path::new("just_a_filename");
    assert_eq!(fs_rs::utils::truncate_filename(path), "just_a_filename");

    // Long, no extension: stem still truncated, no trailing dot.
    let long = "a".repeat(60);
    let truncated = fs_rs::utils::truncate_filename(Path::new(&long));
    // 21 + "..." + 21 = 45 visible chars, with exactly 3 dots from the separator.
    assert_eq!(truncated.chars().count(), 45);
    assert_eq!(truncated.matches('.').count(), 3);
    assert_eq!(
        truncated,
        format!("{}...{}", "a".repeat(21), "a".repeat(21))
    );
}

#[test]
fn test_truncate_filename_exact_boundary() {
    // Stem exactly MAX_FILENAME_LENGTH (45) chars — must NOT be truncated.
    let stem = "a".repeat(45);
    let name = format!("{}.txt", stem);
    let truncated = fs_rs::utils::truncate_filename(Path::new(&name));
    assert_eq!(truncated, name);
    assert!(!truncated.contains("..."));
}

#[test]
fn test_truncate_filename_utf8() {
    // CJK: each `日` is 3 bytes; 50 chars × 3 = 150 bytes. Byte-slicing
    // would have panicked on a non-boundary index; char-slicing must not.
    let cjk = "日".repeat(50);
    let name = format!("{}.txt", cjk);
    let truncated = fs_rs::utils::truncate_filename(Path::new(&name));
    let expected_stem = format!("{}...{}", "日".repeat(21), "日".repeat(21));
    assert_eq!(truncated, format!("{}.txt", expected_stem));
    assert_eq!(truncated.chars().count(), 45 + 4); // stem + ".txt"

    // Emoji: 4 bytes per char.
    let emoji = "😀".repeat(50);
    let name2 = format!("{}.png", emoji);
    let truncated2 = fs_rs::utils::truncate_filename(Path::new(&name2));
    let expected2 = format!("{}...{}", "😀".repeat(21), "😀".repeat(21));
    assert_eq!(truncated2, format!("{}.png", expected2));
    assert_eq!(truncated2.chars().count(), 45 + 4);

    // Accented (precomposed NFC é = 2 bytes).
    let acc = "é".repeat(50);
    let name3 = format!("{}.md", acc);
    let truncated3 = fs_rs::utils::truncate_filename(Path::new(&name3));
    let expected3 = format!("{}...{}", "é".repeat(21), "é".repeat(21));
    assert_eq!(truncated3, format!("{}.md", expected3));
    assert_eq!(truncated3.chars().count(), 45 + 3);
}

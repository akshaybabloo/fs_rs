use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

fn fs_rs() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fs_rs"))
}

#[test]
fn test_default_output_contains_filename_and_total() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.txt");
    File::create(&file)
        .unwrap()
        .write_all(b"hello")
        .unwrap();

    let output = fs_rs()
        .arg(dir.path())
        .arg("--no-color")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test.txt"), "should list the file");
    assert!(stdout.contains("Total size:"), "should show total");
    assert!(
        stdout.contains("Number of files:"),
        "should show file count"
    );
}

#[test]
fn test_json_output_is_valid_and_structured() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("data.txt");
    File::create(&file)
        .unwrap()
        .write_all(b"hello world")
        .unwrap();

    let output = fs_rs()
        .arg(dir.path())
        .arg("--json")
        .arg("--no-color")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed.is_array(), "JSON output should be an array");

    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "data.txt");
    assert!(arr[0]["size_bytes"].is_number());
    assert!(arr[0]["size_human"].is_string());
    assert_eq!(arr[0]["is_dir"], false);
}

#[test]
fn test_sort_by_size_orders_largest_first() {
    let dir = tempdir().unwrap();
    let small = dir.path().join("small.txt");
    let large = dir.path().join("large.txt");
    File::create(&small).unwrap().write_all(b"a").unwrap();
    File::create(&large)
        .unwrap()
        .write_all(&[b'x'; 1000])
        .unwrap();

    let output = fs_rs()
        .arg(dir.path())
        .arg("--sort-by-size")
        .arg("--no-color")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let large_pos = stdout.find("large.txt").expect("should contain large.txt");
    let small_pos = stdout.find("small.txt").expect("should contain small.txt");
    assert!(
        large_pos < small_pos,
        "large.txt should appear before small.txt"
    );
}

#[test]
fn test_tree_output_shows_hierarchy() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    File::create(subdir.join("nested.txt"))
        .unwrap()
        .write_all(b"test")
        .unwrap();

    let output = fs_rs()
        .arg(dir.path())
        .arg("--tree")
        .arg("--no-color")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("subdir"), "should show directory");
    assert!(stdout.contains("nested.txt"), "should show nested file");
}

#[test]
fn test_nonexistent_path_shows_error() {
    let output = fs_rs()
        .arg("/nonexistent/path/xyz_abc_123")
        .arg("--no-color")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("does not exist"));
}

#[test]
fn test_tree_conflicts_with_json() {
    let output = fs_rs()
        .arg("--tree")
        .arg("--json")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "--tree and --json should conflict"
    );
}

#[test]
fn test_tree_conflicts_with_sort_by_size() {
    let output = fs_rs()
        .arg("--tree")
        .arg("--sort-by-size")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "--tree and --sort-by-size should conflict"
    );
}

#[test]
fn test_json_respects_sort_by_size() {
    let dir = tempdir().unwrap();
    let small = dir.path().join("small.txt");
    let large = dir.path().join("large.txt");
    File::create(&small).unwrap().write_all(b"a").unwrap();
    File::create(&large)
        .unwrap()
        .write_all(&[b'x'; 1000])
        .unwrap();

    let output = fs_rs()
        .arg(dir.path())
        .arg("--json")
        .arg("--sort-by-size")
        .arg("--no-color")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["name"], "large.txt", "largest should be first");
    assert_eq!(parsed[1]["name"], "small.txt", "smallest should be last");
}

# FS

Get a quick rundown of your storage usage using `fs_rs` CLI

## Install

```sh
cargo install fs_rs
```

## Usage

```sh
# For current folder
fs_rs

# For a specific folder
fs_rs /path/to/folder

# For multiple folders
fs_rs /path/to/folder1 /path/to/folder2
```
![fs_rs](https://raw.githubusercontent.com/akshaybabloo/fs_rs/main/assets/screencast.gif)

### Options

- `-h` or `--help`: Get help
- `-s` or `--sort-by-size`: Sort by size
- `--disk-usages`: Get disk usages
- `--json`: Get output in JSON format, prints to stdout
- `--version`: Get version

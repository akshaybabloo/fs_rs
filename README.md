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

- `-s` or `--sort-by-size`: Sort the output by size
- `--disk`: Show disk usage
- `--json`: Show as JSON output
- `-t` or `--tree`: Show tree representation
- `--by-files`: Show only files (omit directories from the listing)
- `--by-folder`: Show only folders (omit files from the listing)
- `-d <DEPTH>` or `--depth <DEPTH>`: Depth of the tree representation. Only applicable if `--tree` is set. Defaults to unlimited depth
- `--ascii`: Use ASCII characters for tree representation instead of Unicode
- `--no-color`: Disable colored output
- `-i <NAME>` or `--ignore <NAME>`: Ignore a file or folder by name when calculating sizes. Repeat to ignore multiple names
- `--cpu <N>`: Maximum number of CPU threads used for scanning. Capped at the number of available cores. Defaults to all available cores
- `-h` or `--help`: Print help
- `-V` or `--version`: Print version

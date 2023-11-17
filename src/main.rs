use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::process::exit;

use clap::Parser;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use humansize::{format_size, DECIMAL};
use walkdir::WalkDir;

mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional path to the files or folders
    #[arg(default_values_t = [".".to_string()])]
    path: Vec<String>,
}

fn main() {
    let cli = Args::parse();
    let mut sizes: HashMap<String, u64> = HashMap::new();
    let mut table = Table::new();

    for input_path in cli.path {
        for entry in WalkDir::new(&input_path) {
            match entry {
                Ok(entry) => match entry.metadata() {
                    Ok(metadata) => {
                        let get_first = input_path == ".";
                        let entry_path = entry.path().display().to_string();
                        let root_name = utils::get_root(&entry_path, Some(get_first))
                            .display()
                            .to_string();
                        if metadata.is_dir() {
                            sizes.entry(root_name).or_insert(0);
                        } else if metadata.is_file() {
                            sizes
                                .entry(root_name)
                                .and_modify(|root_name| *root_name += metadata.len())
                                .or_insert(0);
                        }
                    }
                    Err(err) => {
                        println!("failed to get metadata: {}", err);
                        exit(1)
                    }
                },
                Err(err) => {
                    let path = err.path().unwrap_or(Path::new("")).display();
                    println!("failed to access entry {}", path);
                    if let Some(inner) = err.io_error() {
                        match inner.kind() {
                            io::ErrorKind::InvalidData => {
                                println!("entry contains invalid data: {}", inner);
                                exit(1)
                            }
                            io::ErrorKind::PermissionDenied => {
                                println!("Missing permission to read entry: {}", inner);
                                exit(1)
                            }
                            _ => {
                                println!("Unexpected error occurred: {}", inner);
                                exit(1)
                            }
                        }
                    }
                }
            }
        }
    }

    table.load_preset(UTF8_FULL).set_width(80);
    // table.set_header(vec!["Header1", "Header2", "Header3"]);
    // Print the sizes values
    for (root, size) in &sizes {
        if root != "." {
            let sz = format_size(*size, DECIMAL);
            table.add_row(vec![root, &String::from(sz)]);
        }
    }
    println!("{table}")
}

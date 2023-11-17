use std::collections::HashMap;
use std::io;
use std::path::Path;

use clap::Parser;
use humansize::{DECIMAL, format_size};
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

    for path in cli.path {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    match entry.metadata() {
                        Ok(metadata) => {
                            let root_name = utils::get_root(&entry.path().display().to_string()).display().to_string();
                            if metadata.is_dir() {
                                sizes.entry(root_name).or_insert(0);
                            } else if metadata.is_file() {
                                sizes.entry(root_name).and_modify(|root_name| *root_name += metadata.len()).or_insert(0);
                            } else {
                                println!("{} is unknown", entry.path().display());
                            }
                        }
                        Err(err) => println!("failed to get metadata: {}", err)
                    }
                }
                Err(err) => {
                    let path = err.path().unwrap_or(Path::new("")).display();
                    println!("failed to access entry {}", path);
                    if let Some(inner) = err.io_error() {
                        match inner.kind() {
                            io::ErrorKind::InvalidData => {
                                println!(
                                    "entry contains invalid data: {}",
                                    inner)
                            }
                            io::ErrorKind::PermissionDenied => {
                                println!(
                                    "Missing permission to read entry: {}",
                                    inner)
                            }
                            _ => {
                                println!(
                                    "Unexpected error occurred: {}",
                                    inner)
                            }
                        }
                    }
                }
            }
        }
    }

    // Print the sizes values
    for (root, size) in &sizes {
        let sz = format_size(*size, DECIMAL);
        println!("{root} has {sz} hp");
    }
}

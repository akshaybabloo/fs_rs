use std::collections::HashMap;
use std::path::Path;

use clap::Parser;
use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::Table;
use humansize::{format_size, DECIMAL};
use spinoff::{spinners, Color, Spinner, Streams};
use walkdir::WalkDir;

use crate::utils;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional path to the files or folders
    #[arg(default_values_t = [".".to_string()])]
    path: Vec<String>,

    /// Sort the output by size
    #[clap(long, short, action)]
    sort_by_size: bool,
}

/// Run the CLI
pub fn run() {
    let cli = Args::parse();
    let mut sizes: HashMap<String, u64> = HashMap::new();
    let mut sp = Spinner::new_with_stream(
        spinners::Dots,
        "Computing...",
        Color::Yellow,
        Streams::Stderr,
    );

    for (index, input_path) in cli.path.iter().enumerate() {
        let path = Path::new(&input_path);

        if !path.exists() {
            // Stop the spinner if it's the first index
            if index == 0 {
                sp.stop_with_message("");
            }

            println!("{} {}", input_path.red().bold(), "does not exist".red());

            // If on the last index of the path vector, return
            if index == cli.path.len() - 1 {
                return;
            }
            continue;
        }

        if path.is_file() {
            let file_size = path.metadata().unwrap().len();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            sizes.insert(file_name, file_size);
            continue;
        }

        WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .for_each(|entry| {
                let entry_path = entry.path();

                if entry.file_type().is_file() {
                    let file_size = entry.metadata().unwrap().len();
                    let file_name = entry_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    sizes.insert(file_name, file_size);
                } else if entry.file_type().is_dir() {
                    let dir_size = utils::dir_size(entry_path);
                    let dir_name = entry_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    sizes.insert(dir_name, dir_size);
                }
            });
    }

    if sizes.is_empty() {
        sp.stop_with_message("No files or folders found");
        return;
    }

    let mut table = Table::new();
    table.load_preset(NOTHING).set_width(80);

    // Sort the sizes values
    if cli.sort_by_size {
        let sorted_sizes = utils::sort_by_size(&sizes);
        for (root, size) in sorted_sizes {
            let sz = format_size(size, DECIMAL);
            table.add_row(vec![root, sz]);
        }
    } else {
        // Print the sizes values
        for (root, size) in &sizes {
            let sz = format_size(*size, DECIMAL);
            table.add_row(vec![root, &sz]);
        }
    }
    sp.stop_with_message("");
    println!("{table}");

    if cli.path.len() == 1 {
        let total_size = sizes.values().sum::<u64>();
        let sz = format_size(total_size, DECIMAL);
        println!("\n{} {}", "Total size:".green(), sz.green().bold());
    }
}

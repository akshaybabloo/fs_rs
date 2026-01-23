use std::collections::HashMap;
use std::path::Path;

use clap::{ArgAction, Parser};
use colored::Colorize;
use comfy_table::Table;
use comfy_table::presets::{ASCII_MARKDOWN, NOTHING};
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;
use spinoff::{Color, Spinner, spinners};
use sysinfo::{Disks, System};

use crate::utils;
use crate::tree;

/// CLI arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional path to the files or folders
    #[arg(default_values_t = [".".to_string()])]
    path: Vec<String>,

    /// Sort the output by size
    #[arg(long, short, action = ArgAction::SetTrue)]
    sort_by_size: bool,

    /// Show disk usage
    #[arg(long, action = ArgAction::SetTrue)]
    disk_usage: bool,

    /// Show as JSON output
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// Show tree representation
    #[arg(long, short, action = ArgAction::SetTrue)]
    tree: bool,

    /// Depth of the tree representation. Only applicable if --tree is set. Defaults to unlimited depth.
    #[arg(long, short, action = ArgAction::Set)]
    depth: Option<usize>,

    /// Use ASCII characters for tree representation instead of Unicode
    #[arg(long, action = ArgAction::SetTrue)]
    ascii: bool,
}

/// Calculate the size of a directory in parallel
fn calculate_dir_size(dir_path: &Path) -> u64 {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        entries
            .filter_map(Result::ok)
            .collect::<Vec<_>>() // Needed to collect before par_iter
            .par_iter()
            .map(|entry| {
                let path = entry.path();
                match entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            metadata.len()
                        } else if metadata.is_dir() {
                            calculate_dir_size(&path)
                        } else {
                            0
                        }
                    }
                    Err(_) => 0,
                }
            })
            .sum()
    } else {
        0
    }
}

/// Run the CLI
pub fn run() {
    let cli = Args::parse();
    let mut sp = Spinner::new(spinners::Dots, "Computing...", Color::Yellow);

    // Handle tree mode separately
    if cli.tree {
        for input_path in cli.path.iter() {
            let path = Path::new(&input_path);
            if !path.exists() {
                sp.stop_with_message("");
                println!("{} {}", input_path.red().bold(), "does not exist".red());
                continue;
            }
            sp.stop_with_message("");
            print!("{}", tree::generate_tree(path, cli.depth, cli.ascii));
        }
        return;
    }

    let mut sizes: Vec<utils::Sizes> = Vec::new();

    for (index, input_path) in cli.path.iter().enumerate() {
        let path = Path::new(&input_path);

        if !path.exists() {
            if index == 0 {
                sp.stop_with_message("");
            }
            println!("{} {}", input_path.red().bold(), "does not exist".red());
            if index == cli.path.len() - 1 {
                return;
            }
            continue;
        }

        if path.is_file() {
            // File handling remains the same
            match path.metadata() {
                Ok(metadata) => {
                    let file_size = metadata.len();
                    if let Some(file_name) = path.file_name() {
                        let file_name = utils::truncate_filename(Path::new(file_name));
                        sizes.push(utils::Sizes {
                            name: file_name.to_string(),
                            size: file_size,
                            is_dir: false,
                        });
                    }
                }
                Err(e) => println!("Failed to get metadata for {}: {}", path.display(), e),
            }
            continue;
        }

        // Process directory contents
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    let file_name = utils::truncate_filename(Path::new(file_name));
                    match entry.file_type() {
                        Ok(file_type) => {
                            if file_type.is_file() {
                                if let Ok(metadata) = entry.metadata() {
                                    sizes.push(utils::Sizes {
                                        name: file_name.to_string(),
                                        size: metadata.len(),
                                        is_dir: false,
                                    });
                                }
                            } else if file_type.is_dir() {
                                let dir_size = calculate_dir_size(&entry_path);
                                sizes.push(utils::Sizes {
                                    name: file_name.to_string(),
                                    size: dir_size,
                                    is_dir: true,
                                });
                            }
                        }
                        Err(e) => println!("Error getting file type: {}", e),
                    }
                }
            }
        }
    }

    if sizes.is_empty() {
        sp.stop_with_message("No files or folders found");
        return;
    }

    // Output as JSON
    if cli.json {
        // Format values to decimal
        let sizes: HashMap<String, String> = sizes
            .into_iter()
            .map(|s| (s.name, format_size(s.size, DECIMAL)))
            .collect();

        let json_output = serde_json::to_string(&sizes).unwrap();

        sp.stop_with_message(&json_output);
        return;
    }

    let mut table = Table::new();
    table.load_preset(NOTHING).set_width(80);

    // Sort the sizes values
    if cli.sort_by_size {
        let sorted_sizes = utils::sort_by_size(&sizes);
        utils::add_row(&mut table, sorted_sizes);
    } else {
        let sorted_names = utils::sort_by_name(&sizes);
        utils::add_row(&mut table, sorted_names);
    }
    sp.stop_with_message("");
    println!("{table}");

    let total_size = sizes.iter().map(|s| s.size).sum::<u64>();
    let sz = format_size(total_size, DECIMAL);
    println!("\n{} {}", "Total size:".green(), sz.green().bold());
    println!(
        "{} {}\n",
        "Number of files:".green(),
        sizes.len().to_string().green().bold()
    );

    if cli.disk_usage {
        let mut disk_table = Table::new();
        let mut sys = System::new_all();
        // First we update all information of our `System` struct.
        sys.refresh_all();
        disk_table
            .load_preset(ASCII_MARKDOWN)
            .set_header(vec!["Name", "Total", "Available"]);
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let disk_name = disk.name().to_str().unwrap_or("Invalid Disk Name");
            disk_table.add_row(vec![
                disk_name.to_string(),
                format_size(disk.total_space(), DECIMAL),
                format_size(disk.available_space(), DECIMAL),
            ]);
        }
        println!("{disk_table}");
    }
}

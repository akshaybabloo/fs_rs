use std::collections::HashMap;
use std::path::Path;

use clap::{ArgAction, Parser};
use colored::Colorize;
use comfy_table::presets::{ASCII_MARKDOWN, NOTHING};
use comfy_table::Table;
use humansize::{format_size, DECIMAL};
use spinoff::{spinners, Color, Spinner};
use sysinfo::{Disks, System};

use crate::utils;

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
}

/// Calculate the size of a directory
fn calculate_dir_size(dir_path: &Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += calculate_dir_size(&entry.path());
                }
            }
        }
    }
    total_size
}

/// Run the CLI
pub fn run() {
    let cli = Args::parse();
    let mut sizes: HashMap<String, u64> = HashMap::new();
    let mut sp = Spinner::new(spinners::Dots, "Computing...", Color::Yellow);

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
                        sizes.insert(file_name, file_size);
                    }
                }
                Err(e) => println!("Failed to get metadata for {}: {}", path.display(), e),
            }
            continue;
        }

        // Process directory contents
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let entry_path = entry.path();
                    if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        let file_name = utils::truncate_filename(Path::new(file_name));
                        match entry.file_type() {
                            Ok(file_type) => {
                                if file_type.is_file() {
                                    if let Ok(metadata) = entry.metadata() {
                                        sizes.insert(file_name.to_string(), metadata.len());
                                    }
                                } else if file_type.is_dir() {
                                    let dir_size = calculate_dir_size(&entry_path);
                                    sizes.insert(file_name.to_string() + "/", dir_size);
                                }
                            }
                            Err(e) => println!("Error getting file type: {}", e),
                        }
                    }
                }
            }
        }
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
        let sorted_names = utils::sort_by_name(&sizes);
        // Print the sizes values
        for (root, size) in sorted_names {
            let sz = format_size(size, DECIMAL);

            if root.ends_with("/") {
                table.add_row(vec![root.yellow(), sz.yellow()]);
            } else {
                table.add_row(vec![root.bright_blue(), sz.bright_blue()]);
            }
        }
    }
    sp.stop_with_message("");
    println!("{table}");

    let total_size = sizes.values().sum::<u64>();
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

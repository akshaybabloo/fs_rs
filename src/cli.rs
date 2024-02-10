use std::collections::HashMap;
use std::path::Path;

use clap::{Parser, ArgAction, Subcommand};
use colored::Colorize;
use comfy_table::presets::{ASCII_MARKDOWN, NOTHING};
use comfy_table::Table;
use humansize::{format_size, DECIMAL};
use spinoff::{spinners, Color, Spinner, Streams};
use sysinfo::{Disks, System};
use walkdir::WalkDir;

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
    #[arg(long, short, action = ArgAction::SetTrue)]
    disk_usage: bool,
    
    #[command(subcommand)]
    commands: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    Rm {name: Option<String>}
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
            match path.metadata() {
                Ok(metadata) => {
                    let file_size = metadata.len();
                    match path.file_name() {
                        Some(file_name) => {
                            let file_name = utils::truncate_filename(Path::new(file_name));
                            sizes.insert(file_name, file_size);
                        },
                        None => println!("Invalid file name encountered at {}", path.display()),
                    }
                },
                Err(e) => println!("Failed to get metadata for {}: {}", path.display(), e),
            }
            continue;
        }

        WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .for_each(|entry_result| {
                match entry_result {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        match entry_path.file_name().and_then(|n| n.to_str()) {
                            Some(file_name) => {
                                let file_name = utils::truncate_filename(Path::new(file_name));
                                if entry.file_type().is_file() {
                                    match entry.metadata() {
                                        Ok(metadata) => {
                                            let file_size = metadata.len();
                                            sizes.insert(file_name, file_size);
                                        }
                                        Err(e) => println!("Failed to get metadata for {}: {}", entry_path.display(), e),
                                    }
                                } else if entry.file_type().is_dir() {
                                    let dir_size = utils::dir_size(entry_path);
                                    sizes.insert(file_name + "/", dir_size);
                                }
                            }
                            None => println!("Invalid file name encountered at {}", entry_path.display()),
                        }
                    }
                    Err(e) => println!("Error walking directory: {}", e),
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
        let sorted_names = utils::sort_by_name(&sizes);
        // Print the sizes values
        for (root, size) in &sorted_names {
            let sz = format_size(*size, DECIMAL);
            table.add_row(vec![root, &sz]);
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

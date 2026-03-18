use std::path::Path;

use clap::{ArgAction, Parser};
use colored::Colorize;
use colored::control::set_override;
use comfy_table::Table;
use comfy_table::presets::{ASCII_MARKDOWN, NOTHING};
use humansize::{DECIMAL, format_size};
use spinoff::{Color, Spinner, spinners};
use sysinfo::Disks;

use crate::tree;
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

    /// Show as JSON output
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// Show tree representation
    #[arg(long, short, action = ArgAction::SetTrue, conflicts_with_all = ["sort_by_size", "disk_usage", "json"])]
    tree: bool,

    /// Depth of the tree representation. Only applicable if --tree is set. Defaults to unlimited depth.
    #[arg(long, short, action = ArgAction::Set, requires = "tree")]
    depth: Option<usize>,

    /// Use ASCII characters for tree representation instead of Unicode
    #[arg(long, action = ArgAction::SetTrue, requires = "tree")]
    ascii: bool,

    /// Disable colored output
    #[arg(long, action = ArgAction::SetTrue)]
    no_color: bool,
}

/// Run the CLI
pub fn run() {
    let cli = Args::parse();

    if cli.no_color {
        set_override(false);
    }

    // Skip spinner for JSON mode — it writes control characters to stdout
    let mut sp = if cli.json {
        None
    } else {
        Some(Spinner::new(spinners::Dots, "Computing...", Color::Yellow))
    };

    let stop_spinner = |sp: &mut Option<Spinner>| {
        if let Some(mut spinner) = sp.take() {
            spinner.stop_with_message("");
        }
    };

    // Handle tree mode separately
    if cli.tree {
        for input_path in cli.path.iter() {
            let path = Path::new(&input_path);
            if !path.exists() {
                stop_spinner(&mut sp);
                println!("{} {}", input_path.red().bold(), "does not exist".red());
                continue;
            }
            stop_spinner(&mut sp);
            print!("{}", tree::generate_tree(path, cli.depth, cli.ascii));
        }
        return;
    }

    let mut sizes: Vec<utils::Sizes> = Vec::new();

    for (index, input_path) in cli.path.iter().enumerate() {
        let path = Path::new(&input_path);

        if !path.exists() {
            if index == 0 {
                stop_spinner(&mut sp);
            }
            println!("{} {}", input_path.red().bold(), "does not exist".red());
            if index == cli.path.len() - 1 {
                return;
            }
            continue;
        }

        if path.is_file() {
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
                                let dir_size = utils::calculate_dir_size(&entry_path);
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
        stop_spinner(&mut sp);
        eprintln!("No files or folders found");
        return;
    }

    // Sort
    if cli.sort_by_size {
        utils::sort_by_size(&mut sizes);
    } else {
        utils::sort_by_name(&mut sizes);
    }

    // Output as JSON
    if cli.json {
        let json_entries: Vec<serde_json::Value> = sizes
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "size_bytes": s.size,
                    "size_human": format_size(s.size, DECIMAL),
                    "is_dir": s.is_dir,
                })
            })
            .collect();

        match serde_json::to_string(&json_entries) {
            Ok(json_output) => println!("{json_output}"),
            Err(e) => eprintln!("Failed to serialize JSON: {e}"),
        }
        return;
    }

    let mut table = Table::new();
    table.load_preset(NOTHING).set_width(80);

    utils::add_row(&mut table, &sizes);
    stop_spinner(&mut sp);
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
        disk_table
            .load_preset(ASCII_MARKDOWN)
            .set_header(vec!["Name", "Total", "Available"]);
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let disk_name = disk.name().to_str().unwrap_or("Unknown");
            disk_table.add_row(vec![
                disk_name.to_string(),
                format_size(disk.total_space(), DECIMAL),
                format_size(disk.available_space(), DECIMAL),
            ]);
        }
        println!("{disk_table}");
    }
}

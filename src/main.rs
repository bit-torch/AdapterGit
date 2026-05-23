#![allow(dead_code)]

mod ai;
mod cli;
mod commands;
mod config;
mod core;
mod output;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    ai::set_ai_mode(cli.ai);
    output::set_json_mode(cli.json);
    output::set_yaml_mode(cli.yaml);
    output::set_no_color(cli.no_color);

    let result = match &cli.command {
        None => {
            println!("agit - AI-native Git tool (Pure Rust)");
            println!("Run 'agit --help' for usage information.");
            Ok(())
        }
        Some(Commands::Init) => commands::init::run(),
        Some(Commands::Add { files }) => commands::add::run(files),
        Some(Commands::Commit { message, ai }) => commands::commit::run(message.clone(), *ai),
        Some(Commands::Status) => commands::status::run(),
        Some(Commands::Log) => commands::log::run(),
        Some(Commands::Clone { url }) => {
            println!("not implemented yet: clone {}", url);
            Ok(())
        }
        Some(Commands::CatFile {
            show_type,
            pretty_print,
            object,
        }) => cat_file(object, *show_type, *pretty_print),
        Some(Commands::LsTree { tree_sha1 }) => commands::ls_tree::run(tree_sha1),
        Some(Commands::Show { object }) => commands::show::run(object),
        Some(Commands::Diff) => commands::diff::run(),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn cat_file(
    object: &str,
    show_type: bool,
    pretty_print: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = core::repo::find_repo_root()?;
    let (obj_type, content) = core::storage::read_object(&repo_root, object)?;

    if show_type {
        println!("{}", obj_type);
        return Ok(());
    }

    if pretty_print {
        match obj_type.as_str() {
            "blob" => {
                print!("{}", String::from_utf8_lossy(&content));
            }
            "tree" => {
                let tree_data = core::objects::format_object_data("tree", &content);
                let tree = core::objects::tree::Tree::deserialize(&tree_data)?;
                for entry in &tree.entries {
                    let type_str = if entry.mode == "40000" {
                        "tree"
                    } else {
                        "blob"
                    };
                    println!(
                        "{} {} {}\t{}",
                        entry.mode, type_str, entry.sha1, entry.name
                    );
                }
            }
            "commit" => {
                print!("{}", String::from_utf8_lossy(&content));
            }
            _ => {
                print!("{}", String::from_utf8_lossy(&content));
            }
        }
        return Ok(());
    }

    print!("{}", String::from_utf8_lossy(&content));
    Ok(())
}

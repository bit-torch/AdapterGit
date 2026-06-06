mod ai;
mod cli;
mod commands;
mod config;
mod core;
mod output;
mod utils;

use clap::Parser;
use cli::{Cli, Commands, RemoteAction};

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
        Some(Commands::Clone { url }) => commands::clone::run(url),
        Some(Commands::CatFile {
            show_type,
            pretty_print,
            object,
        }) => commands::cat_file::run(object, *show_type, *pretty_print),
        Some(Commands::LsTree { tree_sha1 }) => commands::ls_tree::run(tree_sha1),
        Some(Commands::Show { object }) => commands::show::run(object),
        Some(Commands::Diff) => commands::diff::run(),
        Some(Commands::Fetch { url }) => commands::fetch::run(url.as_deref()),
        Some(Commands::Push { remote, branch }) => {
            commands::push::run(remote.as_deref(), branch.as_deref())
        }
        Some(Commands::Pull) => commands::pull::run(),
        Some(Commands::Remote { action }) => match action {
            RemoteAction::Add { name, url } => commands::remote::run_add(name, url),
            RemoteAction::List => commands::remote::run_list(),
        },
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

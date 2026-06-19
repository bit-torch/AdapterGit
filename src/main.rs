mod ai;
mod cli;
mod commands;
mod config;
mod core;
mod output;
mod utils;

use clap::Parser;
#[cfg(feature = "tag")]
use cli::TagAction;
use cli::{Cli, Commands, RemoteAction, StashAction};
use std::env;

fn main() {
    // 加载配置（含别名）
    let cfg = config::load();

    // 别名解析：将别名替换为实际命令
    let args = resolve_aliases(env::args().collect(), &cfg.aliases);
    let cli = Cli::parse_from(args);

    ai::set_ai_mode(cli.ai);
    output::set_json_mode(cli.json);
    output::set_yaml_mode(cli.yaml);
    output::set_no_color(cli.no_color);

    // AI 模式下检查危险命令
    if let Some(ref cmd) = cli.command {
        if let Err(e) = ai::check_dangerous_command(&command_name(cmd)) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }

    let result = match &cli.command {
        None => {
            println!("agit - AI-native Git tool (Pure Rust)");
            println!("Run 'agit --help' for usage information.");
            Ok(())
        }
        Some(Commands::Rm { cached, files }) => commands::rm::run(*cached, files),
        Some(Commands::Mv { source, dest }) => commands::mv::run(source, dest),
        Some(Commands::Init {
            path,
            pattern,
            licence,
        }) => commands::init::run(path.as_deref(), pattern.as_deref(), licence.as_deref()),
        Some(Commands::Add { files }) => commands::add::run(files),
        Some(Commands::Config {
            global,
            list,
            unset,
            get,
            key,
            value,
        }) => commands::config_cmd::run(
            *global,
            *list,
            *unset,
            *get,
            key.as_deref(),
            value.as_deref(),
        ),
        Some(Commands::Branch {
            list,
            create,
            delete,
        }) => commands::branch::run(*list, create.clone(), delete.clone()),
        Some(Commands::Commit { message, ai }) => commands::commit::run(message.clone(), *ai),
        Some(Commands::Checkout { branch, force }) => commands::checkout::run(branch, *force),
        Some(Commands::Status) => commands::status::run(),
        Some(Commands::Log {
            oneline,
            max_count,
            all,
        }) => commands::log::run(*oneline, *max_count, *all),
        Some(Commands::Merge {
            branch,
            abort,
            r#continue,
        }) => commands::merge::run(branch.as_deref(), *abort, *r#continue),
        Some(Commands::Clone { url }) => commands::clone::run(url),
        Some(Commands::CatFile {
            show_type,
            pretty_print,
            object,
        }) => commands::cat_file::run(object, *show_type, *pretty_print),
        Some(Commands::LsTree { tree_sha1 }) => commands::ls_tree::run(tree_sha1),
        Some(Commands::Show { object }) => commands::show::run(object),
        Some(Commands::Reset {
            soft,
            mixed,
            hard,
            commit,
            files,
        }) => commands::reset::run(*soft, *mixed, *hard, commit.as_deref(), files),
        Some(Commands::Diff {
            cached,
            name_only,
            commit1,
            commit2,
        }) => commands::diff::run(*cached, *name_only, commit1.as_deref(), commit2.as_deref()),
        Some(Commands::Fetch { url }) => commands::fetch::run(url.as_deref()),
        Some(Commands::Push { remote, branch }) => {
            commands::push::run(remote.as_deref(), branch.as_deref())
        }
        Some(Commands::Pull) => commands::pull::run(),
        #[cfg(feature = "tag")]
        Some(Commands::Tag { action }) => match action {
            TagAction::List => commands::tag::run_list(),
            TagAction::Create {
                name,
                message,
                commit,
            } => commands::tag::run_create(name, message.as_deref(), commit.as_deref()),
            TagAction::Delete { name } => commands::tag::run_delete(name),
        },
        Some(Commands::Stash { action }) => match action {
            StashAction::Push => commands::stash::run_push(),
            StashAction::Pop => commands::stash::run_pop(),
            StashAction::List => commands::stash::run_list(),
            StashAction::Drop { stash } => commands::stash::run_drop(stash.as_deref()),
        },
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

/// 将 Commands 枚举转为字符串表示，用于危险命令检查（如 "push"、"stash drop"）。
fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::Push { .. } => "push".to_string(),
        Commands::Stash { action } => match action {
            StashAction::Push => "stash push".to_string(),
            StashAction::Pop => "stash pop".to_string(),
            StashAction::List => "stash list".to_string(),
            StashAction::Drop { .. } => "stash drop".to_string(),
        },
        Commands::Reset { .. } => "reset".to_string(),
        Commands::Branch { delete, .. } => {
            if delete.is_some() {
                "branch -D".to_string()
            } else {
                "branch".to_string()
            }
        }
        Commands::Merge { .. } => "merge".to_string(),
        #[cfg(feature = "tag")]
        Commands::Tag { action } => match action {
            TagAction::Delete { .. } => "tag -d".to_string(),
            _ => "tag".to_string(),
        },
        _ => "".to_string(),
    }
}

/// 将 args 中的别名替换为实际命令名称。
///
/// 例如：`agit co -m "msg"` 中 `co` 为 `commit` 的别名时，
/// 返回 `agit commit -m "msg"`。
fn resolve_aliases(
    args: Vec<String>,
    aliases: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    if args.len() < 2 || aliases.is_empty() {
        return args;
    }

    let command_pos = args.iter().position(|a| !a.starts_with('-')).unwrap_or(0);

    if command_pos == 0 || command_pos >= args.len() {
        return args;
    }

    let command = &args[command_pos];
    if let Some(resolved) = aliases.get(command) {
        let mut resolved_parts: Vec<String> =
            resolved.split_whitespace().map(|s| s.to_string()).collect();
        if resolved_parts.is_empty() {
            return args;
        }

        let mut new_args: Vec<String> = args[..command_pos].to_vec();
        new_args.append(&mut resolved_parts);
        if command_pos + 1 < args.len() {
            new_args.extend_from_slice(&args[command_pos + 1..]);
        }
        new_args
    } else {
        args
    }
}

#![allow(dead_code)]

mod ai;
mod cli;
mod config;
mod core;
mod output;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use std::sync::atomic::{AtomicBool, Ordering};

static AI_MODE: AtomicBool = AtomicBool::new(false);

fn main() {
    let cli = Cli::parse();

    if cli.ai {
        AI_MODE.store(true, Ordering::SeqCst);
    }

    match cli.command {
        None => {
            println!("agit - AI-native Git tool (Pure Rust)");
            println!("Run 'agit --help' for usage information.");
        }
        Some(Commands::Init) => {
            println!("not implemented yet: init");
        }
        Some(Commands::Add { files }) => {
            println!("not implemented yet: add {:?}", files);
        }
        Some(Commands::Commit { message, ai }) => {
            println!(
                "not implemented yet: commit (message: {:?}, ai: {})",
                message, ai
            );
        }
        Some(Commands::Status) => {
            println!("not implemented yet: status");
        }
        Some(Commands::Log) => {
            println!("not implemented yet: log");
        }
        Some(Commands::Clone { url }) => {
            println!("not implemented yet: clone {}", url);
        }
    }
}

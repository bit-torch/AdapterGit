use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agit")]
#[command(about = "AI-native Git tool (Pure Rust)")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, help = "Enable AI-powered suggestions")]
    pub ai: bool,

    #[arg(long, global = true, help = "Output in JSON format")]
    pub json: bool,

    #[arg(long, global = true, help = "Output in YAML format")]
    pub yaml: bool,

    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new git repository")]
    Init,

    #[command(about = "Add files to staging area")]
    Add {
        #[arg(help = "Files to add")]
        files: Vec<String>,
    },

    #[command(about = "Record changes to the repository")]
    Commit {
        #[arg(short = 'm', long, help = "Commit message")]
        message: Option<String>,

        #[arg(long, help = "Use AI to generate commit message")]
        ai: bool,
    },

    #[command(about = "Show working tree status")]
    Status,

    #[command(about = "Show commit logs")]
    Log,

    #[command(about = "Clone a repository")]
    Clone {
        #[arg(help = "Repository URL to clone")]
        url: String,
    },

    #[command(about = "Provide content of repository objects")]
    CatFile {
        #[arg(short = 't', group = "action", help = "Show object type")]
        show_type: bool,

        #[arg(short = 'p', group = "action", help = "Pretty-print object content")]
        pretty_print: bool,

        #[arg(help = "Object SHA-1")]
        object: String,
    },
}

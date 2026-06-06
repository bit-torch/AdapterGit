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

    #[command(about = "List, create, or delete branches")]
    Branch {
        #[arg(short, long = "list", help = "List branches (default)")]
        list: bool,

        #[arg(short = 'c', long = "create", help = "Create a new branch")]
        create: Option<String>,

        #[arg(short = 'd', long = "delete", help = "Delete a branch")]
        delete: Option<String>,
    },

    #[command(about = "Record changes to the repository")]
    Commit {
        #[arg(short = 'm', long, help = "Commit message")]
        message: Option<String>,

        #[arg(long, help = "Use AI to generate commit message")]
        ai: bool,
    },

    #[command(about = "Switch branches or restore working tree files")]
    Checkout {
        #[arg(help = "Branch name to switch to")]
        branch: String,
    },

    #[command(about = "Show working tree status")]
    Status,

    #[command(about = "Show commit logs")]
    Log,

    #[command(about = "Join two or more development histories together")]
    Merge {
        #[arg(help = "Branch to merge into current branch")]
        branch: String,
    },

    #[command(about = "Clone a repository into a new directory")]
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

    #[command(about = "List the contents of a tree object")]
    LsTree {
        #[arg(help = "Tree SHA-1")]
        tree_sha1: String,
    },

    #[command(about = "Show various types of objects")]
    Show {
        #[arg(help = "Object SHA-1 or reference")]
        object: String,
    },

    #[command(about = "Show changes between commits, commit and working tree, etc")]
    Diff,

    #[command(about = "Download objects and refs from another repository")]
    Fetch {
        #[arg(help = "Remote URL or name")]
        url: Option<String>,
    },

    #[command(about = "Update remote refs along with associated objects")]
    Push {
        #[arg(help = "Remote name")]
        remote: Option<String>,

        #[arg(help = "Branch to push")]
        branch: Option<String>,
    },

    #[command(about = "Fetch from and integrate with another repository")]
    Pull,

    #[command(about = "Manage set of tracked repositories")]
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },
}

#[derive(Subcommand)]
pub enum RemoteAction {
    #[command(about = "Add a remote")]
    Add {
        #[arg(help = "Remote name")]
        name: String,

        #[arg(help = "Remote URL")]
        url: String,
    },

    #[command(about = "List remotes")]
    List,
}

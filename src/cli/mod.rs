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
    #[command(about = "Remove files from the working tree and index")]
    Rm {
        #[arg(long, help = "Only remove from index, keep working tree file")]
        cached: bool,

        #[arg(help = "Files to remove")]
        files: Vec<String>,
    },

    #[command(about = "Move or rename a file in the working tree and index")]
    Mv {
        #[arg(help = "Source path")]
        source: String,

        #[arg(help = "Destination path")]
        dest: String,
    },

    #[command(about = "Initialize a new git repository")]
    Init,

    #[command(about = "Add files to staging area")]
    Add {
        #[arg(help = "Files to add")]
        files: Vec<String>,
    },

    #[command(about = "Get and set repository or global options")]
    Config {
        #[arg(short, long, help = "Use global config file (~/.agitconfig.toml)")]
        global: bool,

        #[arg(short, long, help = "List all config variables")]
        list: bool,

        #[arg(long, help = "Remove a config variable")]
        unset: bool,

        #[arg(long, help = "Get a config value (default)")]
        get: bool,

        #[arg(help = "Config key (e.g. user.name)")]
        key: Option<String>,

        #[arg(help = "Value to set")]
        value: Option<String>,
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

        #[arg(
            short = 'f',
            long = "force",
            help = "Force checkout even with local changes"
        )]
        force: bool,
    },

    #[command(about = "Show working tree status")]
    Status,

    #[command(about = "Show commit logs")]
    Log {
        #[arg(long, help = "Show logs in one line format")]
        oneline: bool,

        #[arg(short = 'n', long, help = "Limit number of commits shown")]
        max_count: Option<usize>,

        #[arg(long, help = "Show all branches")]
        all: bool,
    },

    #[command(about = "Join two or more development histories together")]
    Merge {
        #[arg(help = "Branch to merge into current branch")]
        branch: Option<String>,

        #[arg(long, help = "Abort the current conflict resolution process")]
        abort: bool,

        #[arg(long, help = "Continue the current conflict resolution process")]
        r#continue: bool,
    },

    #[command(about = "Create, list, or delete tags")]
    Tag {
        #[command(subcommand)]
        action: TagAction,
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

    #[command(about = "Stash the changes in a dirty working directory")]
    Stash {
        #[command(subcommand)]
        action: StashAction,
    },

    #[command(about = "Reset current HEAD to the specified state")]
    Reset {
        #[arg(short, long, help = "Keep index and working tree (move HEAD only)")]
        soft: bool,

        #[arg(long, help = "Reset index but not working tree (default)")]
        mixed: bool,

        #[arg(long, help = "Reset index and working tree")]
        hard: bool,

        #[arg(help = "Commit/tree to reset to (default: HEAD)")]
        commit: Option<String>,

        #[arg(help = "Files to unstage from index")]
        files: Vec<String>,
    },

    #[command(about = "Show changes between commits, commit and working tree, etc")]
    Diff {
        #[arg(long, help = "Show staged changes (HEAD vs index)")]
        cached: bool,

        #[arg(long, help = "Show only filenames")]
        name_only: bool,

        #[arg(help = "First commit/tree to compare")]
        commit1: Option<String>,

        #[arg(help = "Second commit/tree to compare")]
        commit2: Option<String>,
    },

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

    #[command(about = "Reapply commits on top of another base tip")]
    Rebase {
        #[arg(help = "Upstream branch/commit to rebase onto")]
        upstream: Option<String>,

        #[arg(long, help = "Starting point to place commits onto")]
        onto: Option<String>,

        #[arg(long, help = "Continue the rebase in progress")]
        r#continue: bool,

        #[arg(long, help = "Skip the current commit")]
        skip: bool,

        #[arg(long, help = "Abort the rebase in progress")]
        abort: bool,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    #[command(about = "List all tags")]
    List,

    #[command(about = "Create a new tag")]
    Create {
        #[arg(help = "Tag name")]
        name: String,

        #[arg(short = 'm', long, help = "Tag message (annotated)")]
        message: Option<String>,

        #[arg(help = "Commit SHA (default: HEAD)")]
        commit: Option<String>,
    },

    #[command(about = "Delete a tag")]
    Delete {
        #[arg(help = "Tag name to delete")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum StashAction {
    #[command(about = "Save working tree changes to stash")]
    Push,

    #[command(about = "Apply and remove the top stash")]
    Pop,

    #[command(about = "List all stashes")]
    List,

    #[command(about = "Remove a stash entry")]
    Drop {
        #[arg(help = "Stash reference (e.g. stash@{0})")]
        stash: Option<String>,
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

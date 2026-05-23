use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;
use std::sync::atomic::Ordering;

pub fn run(message: Option<String>, ai: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let index = Index::load(&repo_root)?;
    if index.entries.is_empty() {
        println!("nothing to commit (no staged changes)");
        return Ok(());
    }

    let (timestamp, time_str) = repo::get_current_timestamp();

    let author = format!("agit <agit@localhost> {} {}", timestamp, time_str);
    let committer = format!("agit <agit@localhost> {} {}", timestamp, time_str);

    let mut msg = message.unwrap_or_else(|| "Update".to_string());
    let is_ai = ai || crate::AI_MODE.load(Ordering::SeqCst);
    if is_ai {
        msg = format!("[AI-committed] {}", msg);
    }
    if !msg.ends_with('\n') {
        msg.push('\n');
    }

    let mut tree = Tree::new();
    for entry in index.entries.values() {
        tree.add_entry(&entry.mode, &entry.path, &entry.sha1);
    }
    let tree_sha1 = tree.hash();
    storage::write_object(&repo_root, "tree", &tree.serialize_raw())?;

    let mut commit = Commit::new(&tree_sha1, &author, &committer, &msg);

    if let Ok(head_sha1) = refs::read_head(&repo_root) {
        commit.add_parent(&head_sha1);
    }

    let commit_sha1 = commit.hash();
    storage::write_object(&repo_root, "commit", &commit.serialize_raw())?;

    refs::write_ref(&repo_root, "refs/heads/main", &commit_sha1)?;

    let parent_info = if commit.parents.is_empty() {
        " (root-commit)".to_string()
    } else {
        String::new()
    };
    println!(
        "[{} {:.7}]{} {}",
        if is_ai { "ai" } else { "main" },
        commit_sha1,
        parent_info,
        msg.trim_end()
    );

    Ok(())
}

use crate::ai;
use crate::config;
use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;

pub fn run(message: Option<String>, ai_flag: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let index = Index::load(&repo_root)?;
    if index.entries.is_empty() {
        println!("nothing to commit (no staged changes)");
        return Ok(());
    }

    let cfg = config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        cfg.user_name, cfg.user_email, timestamp, time_str
    );
    let committer = author.clone();

    let is_ai = ai_flag || ai::is_ai_mode();

    // 检查是否处于合并状态（.git/MERGE_HEAD 存在）
    let git_dir = repo_root.join(".git");
    let merge_head_path = git_dir.join("MERGE_HEAD");
    let merge_msg_path = git_dir.join("MERGE_MSG");
    let in_merge = merge_head_path.exists();

    // 确定最终 commit message
    let mut msg = if let Some(ref m) = message {
        m.clone()
    } else if in_merge {
        // 合并状态且无 -m：使用 MERGE_MSG
        std::fs::read_to_string(&merge_msg_path)
            .unwrap_or_else(|_| "Merge".to_string())
            .trim()
            .to_string()
    } else {
        "Update".to_string()
    };

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

    if in_merge {
        let head_sha1 = refs::read_head(&repo_root)?;
        let merge_head_sha =
            std::fs::read_to_string(&merge_head_path)?.trim().to_string();
        commit.add_parent(&head_sha1);
        commit.add_parent(&merge_head_sha);
    } else if let Ok(head_sha1) = refs::read_head(&repo_root) {
        commit.add_parent(&head_sha1);
    }

    let commit_sha1 = commit.hash();
    storage::write_object(&repo_root, "commit", &commit.serialize_raw())?;

    let head_content =
        std::fs::read_to_string(repo_root.join(".git").join("HEAD")).unwrap_or_default();
    let head_trimmed = head_content.trim();
    let branch_ref = if let Some(ref_path) = head_trimmed.strip_prefix("ref: ") {
        ref_path.trim().to_string()
    } else if head_trimmed.len() == 40 && head_trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("You are in 'detached HEAD' state. Please create a branch first.".into());
    } else {
        "refs/heads/main".to_string()
    };
    refs::write_ref(&repo_root, &branch_ref, &commit_sha1)?;

    // 清理合并状态文件
    if in_merge {
        let _ = std::fs::remove_file(&merge_head_path);
        let _ = std::fs::remove_file(&merge_msg_path);
        let _ = std::fs::remove_file(git_dir.join("MERGE_MODE"));
        let _ = std::fs::remove_file(git_dir.join("ORIG_HEAD"));
    }

    let branch_name = branch_ref.strip_prefix("refs/heads/").unwrap_or("main");

    let parent_info = if commit.parents.is_empty() {
        " (root-commit)".to_string()
    } else {
        String::new()
    };
    println!(
        "[{} {:.7}]{} {}",
        if is_ai { "ai" } else { branch_name },
        commit_sha1,
        parent_info,
        msg.trim_end()
    );

    Ok(())
}

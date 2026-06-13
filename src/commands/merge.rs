use crate::config;
use crate::core::{checkout, merge, refs, repo};
use std::fs;

pub fn run(
    branch: Option<&str>,
    abort: bool,
    r#continue: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    if abort {
        return abort_merge(&repo_root);
    }

    if r#continue {
        // merge --continue: 直接提交（commit 命令已处理 MERGE_HEAD）
        return super::commit::run(None, false);
    }

    let branch = branch.ok_or("error: branch name is required for merge")?;
    let cfg = config::load();

    // 验证目标分支存在
    let ref_path = format!("refs/heads/{}", branch);
    if !repo_root.join(".git").join(&ref_path).exists() {
        return Err(format!("branch '{}' not found", branch).into());
    }

    // 不能合并到自己
    let current = refs::get_current_branch(&repo_root)?;
    if current.as_deref() == Some(branch) {
        return Err(format!("Already on '{}'", branch).into());
    }

    // 保存 ORIG_HEAD 以便 --abort 恢复
    if let Ok(head_sha) = refs::read_head(&repo_root) {
        let _ = fs::write(
            repo_root.join(".git").join("ORIG_HEAD"),
            format!("{}\n", head_sha),
        );
    }

    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        cfg.user_name, cfg.user_email, timestamp, time_str
    );

    merge::merge_branch(&repo_root, branch, &author, &author)
}

/// 中止正在进行的合并，恢复到合并前的状态。
fn abort_merge(repo_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_root.join(".git");
    let merge_head_path = git_dir.join("MERGE_HEAD");

    if !merge_head_path.exists() {
        return Err("error: There is no merge to abort (MERGE_HEAD missing).".into());
    }

    // 读取 ORIG_HEAD 或当前 HEAD 作为恢复目标
    let orig_head_path = git_dir.join("ORIG_HEAD");
    let restore_sha = if orig_head_path.exists() {
        fs::read_to_string(&orig_head_path)?.trim().to_string()
    } else {
        refs::read_head(repo_root)?
    };

    // 恢复 index 和工作区到恢复目标
    checkout::restore_from_commit(repo_root, &restore_sha)?;
    checkout::rebuild_index_from_commit(repo_root, &restore_sha)?;

    // 清理合并状态文件
    let _ = fs::remove_file(&merge_head_path);
    let _ = fs::remove_file(git_dir.join("MERGE_MSG"));
    let _ = fs::remove_file(git_dir.join("MERGE_MODE"));
    let _ = fs::remove_file(git_dir.join("ORIG_HEAD"));

    println!("Merge aborted.");
    Ok(())
}

use crate::core::index::Index;
use crate::core::{checkout, reflog, refs, repo};

pub fn run(branch: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let cfg = crate::config::load();

    // 检查分支是否存在
    let ref_path = format!("refs/heads/{}", branch);
    if !repo_root.join(".git").join(&ref_path).exists() {
        return Err(format!(
            "pathspec '{}' did not match any branch known to agit",
            branch
        )
        .into());
    }

    // 检查是否已在该分支
    let current = refs::get_current_branch(&repo_root)?;
    if current.as_deref() == Some(branch) {
        println!("Already on '{}'", branch);
        return Ok(());
    }

    // 安全检查：工作区是否有未提交的变更
    if !force {
        let index = Index::load(&repo_root)?;
        if !repo::is_working_tree_clean(&repo_root, &index)? {
            return Err(
                "error: Your local changes to the following files would be overwritten by checkout:\n\
                 Please commit your changes or stash them before you switch branches.\n\
                 (use --force to override)"
                    .into(),
            );
        }
    }

    let old_head = refs::read_head(&repo_root)
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000".into());
    checkout::switch_branch(&repo_root, branch)?;
    let new_head = refs::read_head(&repo_root)?;
    let _ = reflog::append_reflog(
        &repo_root,
        "HEAD",
        &old_head,
        &new_head,
        &cfg.user_name,
        &format!("checkout: moving to {}", branch),
    );

    Ok(())
}

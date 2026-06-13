use crate::core::index::Index;
use crate::core::objects::blob::Blob;
use crate::core::{checkout, refs, repo};
use std::fs;

pub fn run(branch: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

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
        if !is_working_tree_clean(&repo_root, &index)? {
            return Err(
                "error: Your local changes to the following files would be overwritten by checkout:\n\
                 Please commit your changes or stash them before you switch branches.\n\
                 (use --force to override)"
                    .into(),
            );
        }
    }

    checkout::switch_branch(&repo_root, branch)
}

/// 检查工作区是否干净（tracked 文件是否被修改或删除）。
fn is_working_tree_clean(
    repo: &std::path::Path,
    index: &Index,
) -> Result<bool, Box<dyn std::error::Error>> {
    for (path, entry) in index.entries.iter() {
        let full_path = repo.join(path);
        if full_path.exists() {
            let content = fs::read(&full_path)
                .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            let blob = Blob::new(content);
            if blob.hash() != entry.sha1 {
                return Ok(false);
            }
        } else {
            // 文件在 index 中但不在工作区 → 被删除
            return Ok(false);
        }
    }
    Ok(true)
}

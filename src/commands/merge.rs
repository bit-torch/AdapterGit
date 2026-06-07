use crate::config;
use crate::core::{merge, refs, repo};

pub fn run(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
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

    let author = format!("{} <{}> 0 +0000", cfg.user_name, cfg.user_email);

    merge::merge_branch(&repo_root, branch, &author, &author)
}

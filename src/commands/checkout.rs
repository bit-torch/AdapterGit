use crate::core::{checkout, refs, repo};

pub fn run(branch: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    checkout::switch_branch(&repo_root, branch)
}

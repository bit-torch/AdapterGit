use crate::core::{refs, repo};

pub fn run(
    _list: bool,
    create: Option<String>,
    delete: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let current = refs::get_current_branch(&repo_root)?;

    if let Some(name) = delete {
        return delete_branch_action(&repo_root, &current, &name);
    }

    if let Some(name) = create {
        return create_branch_action(&repo_root, &name);
    }

    // 默认：列出所有分支
    list_branches_action(&repo_root, &current);
    Ok(())
}

fn list_branches_action(repo: &std::path::Path, current: &Option<String>) {
    let branches = refs::list_branches(repo).unwrap_or_default();
    if branches.is_empty() {
        println!("(no branches)");
        return;
    }
    for b in &branches {
        if current.as_deref() == Some(b) {
            println!("* {}", b);
        } else {
            println!("  {}", b);
        }
    }
}

fn create_branch_action(
    repo: &std::path::Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 新分支指向当前 HEAD
    let sha1 = refs::read_head(repo)?;
    refs::create_branch(repo, name, &sha1)?;
    println!(
        "Created branch '{}' at {}",
        name,
        &sha1[..7.min(sha1.len())]
    );
    Ok(())
}

fn delete_branch_action(
    repo: &std::path::Path,
    current: &Option<String>,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 不允许删除当前所在分支
    if current.as_deref() == Some(name) {
        return Err(format!("Cannot delete branch '{}' checked out at HEAD", name).into());
    }
    refs::delete_branch(repo, name)?;
    println!("Deleted branch '{}'", name);
    Ok(())
}

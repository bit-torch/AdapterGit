use crate::core::refs;
use crate::core::remote_utils;
use crate::core::repo;
use std::fs;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let branch = remote_utils::get_current_branch(&repo_root)?;
    let remote_url = remote_utils::get_remote_url(&repo_root)?;

    println!("Pulling from {} for branch '{}'...", remote_url, branch);

    super::fetch::run(None)?;

    let fetch_head = repo_root
        .join(".git")
        .join("refs")
        .join("remotes")
        .join("origin")
        .join(&branch);

    if !fetch_head.exists() {
        println!("Already up to date.");
        return Ok(());
    }

    let remote_sha1 = fs::read_to_string(&fetch_head)?.trim().to_string();
    let local_sha1 = refs::read_ref(&repo_root, &format!("refs/heads/{}", branch))?;

    if remote_sha1 == local_sha1 {
        println!("Already up to date.");
        return Ok(());
    }

    let ancestor = find_common_ancestor(&repo_root, &local_sha1, &remote_sha1)?;
    if let Some(anc) = ancestor {
        if anc == local_sha1 {
            fast_forward(&repo_root, &branch, &local_sha1, &remote_sha1)?;
        } else {
            merge_changes(&repo_root, &branch, &local_sha1, &remote_sha1)?;
        }
    } else {
        merge_changes(&repo_root, &branch, &local_sha1, &remote_sha1)?;
    }

    Ok(())
}

fn collect_all_ancestors(
    repo: &std::path::Path,
    sha1: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut current = sha1.to_string();
    loop {
        let (obj_type, content) = match crate::core::storage::read_object(repo, &current) {
            Ok(v) => v,
            Err(_) => break,
        };
        if obj_type != "commit" {
            break;
        }
        let commit_data =
            crate::core::objects::format_object_data("commit", &content);
        let commit =
            crate::core::objects::commit::Commit::deserialize(&commit_data)?;
        result.push(current.clone());
        if commit.parents.is_empty() {
            break;
        }
        current = commit.parents[0].clone();
    }
    Ok(result)
}

fn find_common_ancestor(
    repo: &std::path::Path,
    a: &str,
    b: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let ancestors_a = collect_all_ancestors(repo, a)?;
    let ancestors_b: std::collections::HashSet<String> =
        collect_all_ancestors(repo, b)?.into_iter().collect();

    for sha1 in &ancestors_a {
        if ancestors_b.contains(sha1) {
            return Ok(Some(sha1.clone()));
        }
    }
    Ok(None)
}

fn fast_forward(
    repo: &std::path::Path,
    branch: &str,
    old_sha1: &str,
    new_sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    refs::write_ref(repo, &format!("refs/heads/{}", branch), new_sha1)?;
    remote_utils::apply_tree_by_sha1(repo, "", new_sha1)?;

    println!(
        "Fast-forward\n {} -> {}",
        &old_sha1[..7],
        &new_sha1[..7]
    );

    Ok(())
}

fn merge_changes(
    repo: &std::path::Path,
    branch: &str,
    local_sha1: &str,
    remote_sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        config.user_name, config.user_email, timestamp, time_str
    );
    let committer = author.clone();

    let tree_sha1 = remote_utils::resolve_commit_to_tree(repo, remote_sha1)?;

    let mut merge_commit = crate::core::objects::commit::Commit::new(
        &tree_sha1,
        &author,
        &committer,
        &format!("Merge branch '{}' of remote into {}\n", branch, branch),
    );
    merge_commit.add_parent(local_sha1);
    merge_commit.add_parent(remote_sha1);

    let merge_sha1 = merge_commit.hash();
    crate::core::storage::write_object(repo, "commit", &merge_commit.serialize_raw())?;

    refs::write_ref(repo, &format!("refs/heads/{}", branch), &merge_sha1)?;

    remote_utils::apply_tree_by_sha1(repo, "", tree_sha1.as_str())?;

    println!(
        "Merge made.\n {} -> {}",
        &local_sha1[..7],
        &merge_sha1[..7]
    );

    Ok(())
}

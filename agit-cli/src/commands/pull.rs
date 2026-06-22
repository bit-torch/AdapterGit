use agit_core::index::Index;
use agit_core::refs;
use agit_core::remote_utils;
use agit_core::repo;
use std::collections::VecDeque;
use std::fs;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let branch = remote_utils::get_current_branch(&repo_root)?;
    let remote_url = remote_utils::get_remote_url(&repo_root, None)?;

    let index = Index::load(&repo_root)?;
    let working_clean = check_working_tree_clean(&repo_root, &index)?;
    if !working_clean {
        return Err(
            "Cannot pull: working tree has uncommitted changes. Please commit or stash them."
                .into(),
        );
    }

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

fn check_working_tree_clean(
    repo: &std::path::Path,
    index: &Index,
) -> Result<bool, Box<dyn std::error::Error>> {
    use agit_core::objects::blob::Blob;
    for (path, entry) in index.entries.iter() {
        let full_path = repo.join(path);
        if full_path.exists() {
            let content =
                fs::read(&full_path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            let blob = Blob::new(content);
            if blob.hash() != entry.sha1 {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }
    let mut untracked = Vec::new();
    collect_untracked(repo, repo, index, &mut untracked)?;
    Ok(untracked.is_empty())
}

fn collect_untracked(
    repo: &std::path::Path,
    current: &std::path::Path,
    index: &Index,
    untracked: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        if file_name == ".git" {
            continue;
        }

        let relative = path
            .strip_prefix(repo)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            collect_untracked(repo, &path, index, untracked)?;
        } else if path.is_file() && !index.entries.contains_key(&relative) {
            untracked.push(relative);
        }
    }
    Ok(())
}

fn find_common_ancestor(
    repo: &std::path::Path,
    a: &str,
    b: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut a_seen = std::collections::HashSet::new();
    let mut b_seen = std::collections::HashSet::new();
    let mut a_queue: VecDeque<String> = VecDeque::from([a.to_string()]);
    let mut b_queue: VecDeque<String> = VecDeque::from([b.to_string()]);

    loop {
        if let Some(sha1) = a_queue.pop_front() {
            if b_seen.contains(&sha1) {
                return Ok(Some(sha1));
            }
            if !a_seen.insert(sha1.clone()) {
                continue;
            }
            push_parents(repo, &sha1, &mut a_queue)?;
        }
        if let Some(sha1) = b_queue.pop_front() {
            if a_seen.contains(&sha1) {
                return Ok(Some(sha1));
            }
            if !b_seen.insert(sha1.clone()) {
                continue;
            }
            push_parents(repo, &sha1, &mut b_queue)?;
        }
        if a_queue.is_empty() && b_queue.is_empty() {
            break;
        }
    }
    Ok(None)
}

fn push_parents(
    repo: &std::path::Path,
    sha1: &str,
    queue: &mut VecDeque<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = match agit_core::storage::read_object(repo, sha1) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    if obj_type != "commit" {
        return Ok(());
    }
    let commit_data = agit_core::objects::format_object_data("commit", &content);
    let commit = agit_core::objects::commit::Commit::deserialize(&commit_data)?;
    for parent in commit.parents {
        queue.push_back(parent);
    }
    Ok(())
}

fn fast_forward(
    repo: &std::path::Path,
    branch: &str,
    old_sha1: &str,
    new_sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    refs::write_ref(repo, &format!("refs/heads/{}", branch), new_sha1)?;
    let tree_sha1 = remote_utils::resolve_commit_to_tree(repo, new_sha1)?;
    remote_utils::apply_tree_by_sha1(repo, "", &tree_sha1)?;

    println!("Fast-forward\n {} -> {}", &old_sha1[..7], &new_sha1[..7]);

    Ok(())
}

fn merge_changes(
    repo: &std::path::Path,
    branch: &str,
    local_sha1: &str,
    remote_sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 查找 merge base 进行 3-way merge
    let base_sha = find_common_ancestor(repo, local_sha1, remote_sha1)?
        .unwrap_or_else(|| local_sha1.to_string());

    let config = agit_core::config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        config.user_name, config.user_email, timestamp, time_str
    );
    let committer = author.clone();

    // 执行 3-way merge（直接操作 tree），写入工作目录和 index
    let has_conflicts =
        agit_core::merge::three_way_merge(repo, &base_sha, local_sha1, remote_sha1)?;

    if has_conflicts {
        println!("Automatic merge failed; fix conflicts and then commit the result.");
        let git_dir = repo.join(".git");
        agit_core::utils::atomic_write(
            &git_dir.join("MERGE_HEAD"),
            format!("{}\n", remote_sha1).as_bytes(),
        )?;
        agit_core::utils::atomic_write(
            &git_dir.join("MERGE_MSG"),
            format!("Merge branch '{}' of remote into {}\n", branch, branch).as_bytes(),
        )?;
        return Ok(());
    }

    // 无冲突：从 index 生成 tree 并创建 merge commit
    let index = agit_core::index::Index::load(repo)?;
    let mut merge_tree = agit_core::objects::tree::Tree::new();
    for entry in index.entries.values() {
        merge_tree.add_entry(&entry.mode, &entry.path, &entry.sha1);
    }
    let tree_sha = merge_tree.hash();
    agit_core::storage::write_object(repo, "tree", &merge_tree.serialize_raw())?;

    let mut merge_commit = agit_core::objects::commit::Commit::new(
        &tree_sha,
        &author,
        &committer,
        &format!("Merge branch '{}' of remote into {}\n", branch, branch),
    );
    merge_commit.add_parent(local_sha1);
    merge_commit.add_parent(remote_sha1);

    let merge_sha1 = merge_commit.hash();
    agit_core::storage::write_object(repo, "commit", &merge_commit.serialize_raw())?;

    refs::write_ref(repo, &format!("refs/heads/{}", branch), &merge_sha1)?;

    println!("Merge made.\n {} -> {}", &local_sha1[..7], &merge_sha1[..7]);

    Ok(())
}

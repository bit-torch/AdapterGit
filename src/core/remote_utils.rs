use crate::core::objects::commit::Commit;
use crate::core::protocol::ObjectList;
use crate::core::objects::tree::Tree;
use std::fs;
use std::path::Path;

pub fn write_objects(
    repo: &Path,
    objects: &[(String, Vec<u8>)],
) -> Result<(), Box<dyn std::error::Error>> {
    for (sha1, data) in objects {
        let obj_path = repo
            .join(".git")
            .join("objects")
            .join(&sha1[..2])
            .join(&sha1[2..]);
        if !obj_path.exists() {
            if let Some(parent) = obj_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let compressed = crate::core::compression::compress(data)?;
            fs::write(&obj_path, &compressed)?;
        }
    }
    Ok(())
}

pub fn apply_tree(
    repo: &Path,
    prefix: &str,
    tree: &Tree,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in &tree.entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        let (obj_type, content) = crate::core::storage::read_object(repo, &entry.sha1)?;

        if obj_type == "tree" {
            let full_path = repo.join(&path);
            if !full_path.exists() {
                fs::create_dir_all(&full_path)?;
            }
            let tree_data =
                crate::core::objects::format_object_data("tree", &content);
            let subtree = Tree::deserialize(&tree_data)?;
            apply_tree(repo, &path, &subtree)?;
        } else if obj_type == "blob" {
            let full_path = repo.join(&path);
            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            use std::io::Write;
            let mut f = fs::File::create(&full_path)?;
            f.write_all(&content)?;
        }
    }
    Ok(())
}

pub fn apply_tree_by_sha1(
    repo: &Path,
    prefix: &str,
    tree_sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_, tree_content) = crate::core::storage::read_object(repo, tree_sha1)?;
    let tree_data = crate::core::objects::format_object_data("tree", &tree_content);
    let tree = Tree::deserialize(&tree_data)?;
    apply_tree(repo, prefix, &tree)
}

pub fn get_remote_url(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let config = fs::read_to_string(repo.join(".git").join("config")).unwrap_or_default();
    let mut current_section = "";
    let mut first_url: Option<String> = None;

    for line in config.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("[remote \"").and_then(|s| s.strip_suffix("\"]")) {
            current_section = name;
        } else if trimmed.starts_with('[') {
            current_section = "";
        } else if let Some(url) = trimmed.strip_prefix("url = ") {
            if current_section == "origin" {
                return Ok(url.to_string());
            }
            if first_url.is_none() {
                first_url = Some(url.to_string());
            }
        }
    }

    first_url.ok_or_else(|| "No remote URL configured".into())
}

pub fn get_current_branch(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let head_content =
        fs::read_to_string(repo.join(".git").join("HEAD")).unwrap_or_default();
    let head_content = head_content.trim();
    if let Some(ref_path) = head_content.strip_prefix("ref: ") {
        ref_path
            .strip_prefix("refs/heads/")
            .map(|s| s.to_string())
            .ok_or_else(|| "Not on a branch".into())
    } else {
        Err("Not on a branch (detached HEAD)".into())
    }
}

pub fn collect_recent_commits(
    repo: &Path,
    sha1: &str,
    max: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut current = sha1.to_string();
    for _ in 0..max {
        let (obj_type, content) = match crate::core::storage::read_object(repo, &current) {
            Ok(v) => v,
            Err(_) => break,
        };
        if obj_type != "commit" {
            break;
        }
        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = Commit::deserialize(&commit_data)?;
        if commit.parents.is_empty() {
            break;
        }
        current = commit.parents[0].clone();
        result.push(current.clone());
    }
    Ok(result)
}

pub fn resolve_commit_to_tree(
    repo: &Path,
    sha1: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (_, content) = crate::core::storage::read_object(repo, sha1)?;
    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;
    Ok(commit.tree)
}

pub fn collect_local_objects_for_push(
    repo: &Path,
    local_tip: &str,
    remote_tip: Option<&str>,
) -> Result<ObjectList, Box<dyn std::error::Error>> {
    let mut objects = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![local_tip.to_string()];
    let remote_sha1s: std::collections::HashSet<String> = if let Some(rt) = remote_tip {
        collect_all_ancestors(repo, rt)?.into_iter().collect()
    } else {
        std::collections::HashSet::new()
    };

    while let Some(current) = queue.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if remote_sha1s.contains(&current) {
            continue;
        }
        let (obj_type, content) = match crate::core::storage::read_object(repo, &current) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if obj_type != "commit" {
            continue;
        }

        let full_object =
            crate::core::objects::format_object_data("commit", &content);
        objects.push((current.clone(), full_object));

        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = Commit::deserialize(&commit_data)?;

        collect_tree_objects(repo, &commit.tree, &mut objects)?;

        for parent in commit.parents {
            if !seen.contains(&parent) && !remote_sha1s.contains(&parent) {
                queue.push(parent.clone());
            }
        }
    }

    Ok(objects)
}

fn collect_all_ancestors(
    repo: &Path,
    sha1: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut queue = vec![sha1.to_string()];
    let mut seen = std::collections::HashSet::new();

    while let Some(current) = queue.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        let (obj_type, content) = match crate::core::storage::read_object(repo, &current) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if obj_type != "commit" {
            continue;
        }
        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = Commit::deserialize(&commit_data)?;
        result.push(current);
        for parent in commit.parents {
            if !seen.contains(&parent) {
                queue.push(parent.clone());
            }
        }
    }
    Ok(result)
}

fn collect_tree_objects(
    repo: &Path,
    tree_sha1: &str,
    objects: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = crate::core::storage::read_object(repo, tree_sha1)?;
    if obj_type != "tree" {
        return Ok(());
    }
    objects.push((tree_sha1.to_string(), crate::core::objects::format_object_data("tree", &content)));

    let tree_data = crate::core::objects::format_object_data("tree", &content);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let (e_type, e_content) = crate::core::storage::read_object(repo, &entry.sha1)?;
        if e_type == "tree" {
            collect_tree_objects(repo, &entry.sha1, objects)?;
        } else if e_type == "blob" {
            objects.push((
                entry.sha1.clone(),
                crate::core::objects::format_object_data("blob", &e_content),
            ));
        }
    }
    Ok(())
}

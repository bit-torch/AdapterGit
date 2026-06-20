use crate::core::index::Index;
use crate::core::protocol::create_transport;
use crate::core::refs;
use crate::core::remote_utils;
use crate::core::repo;
use crate::core::storage;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn run(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloning into '{}'...", repo_name_from_url(url));

    let transport = create_transport(url)?;
    let refs_list = transport.discover_refs()?;

    let head_ref = refs_list
        .iter()
        .find(|(_, name)| *name == "HEAD")
        .and_then(|(sha1, _)| {
            refs_list
                .iter()
                .find(|(_, name)| *name == *ref_name_from_head(sha1, &refs_list))
                .map(|(s, _)| s.clone())
        })
        .or_else(|| refs_list.first().map(|(s, _)| s.clone()))
        .ok_or("No refs found on remote")?;

    let branch_name = extract_branch_from_refs(&head_ref, &refs_list);
    let dir_name = repo_name_from_url(url);
    let repo_dir = std::env::current_dir()?.join(&dir_name);

    if repo_dir.exists() {
        return Err(format!("destination path '{}' already exists", dir_name).into());
    }
    fs::create_dir_all(&repo_dir)?;

    let objects = transport.fetch_objects(&[head_ref.clone()], &[])?;

    init_git_dir(&repo_dir)?;
    remote_utils::write_objects(&repo_dir, &objects)?;

    // Configure remote origin
    let mut config_file = fs::OpenOptions::new()
        .append(true)
        .open(repo_dir.join(".git").join("config"))?;
    writeln!(config_file, "[remote \"origin\"]")?;
    writeln!(config_file, "\turl = {}", url)?;
    writeln!(config_file, "\tfetch = +refs/heads/*:refs/remotes/origin/*")?;

    for (ref_name, ref_sha1) in &refs_list {
        eprintln!(
            "clone for_loop: name={}, starts_with_heads={}",
            ref_name,
            ref_name.starts_with("refs/heads/")
        );
        // 验证 ref 名安全（拒绝路径穿越等）
        if (ref_name.starts_with("refs/heads/") || ref_name.starts_with("refs/tags/"))
            && !ref_name.contains("..")
            && !ref_name.contains('\\')
        {
            eprintln!("clone writing ref: {}", ref_name);
            refs::write_ref(&repo_dir, ref_name, ref_sha1)?;
        } else if ref_name.starts_with("refs/") {
            eprintln!(
                "clone: skipping unsafe remote ref '{}' (contains path traversal)",
                ref_name
            );
        }
    }

    refs::write_head(&repo_dir, &format!("ref: refs/heads/{}", branch_name))?;

    checkout_head(&repo_dir, &head_ref)?;

    // Build index from the checked-out tree
    let (_, tree_content) = storage::read_object(
        &repo_dir,
        &crate::core::remote_utils::resolve_commit_to_tree(&repo_dir, &head_ref)?,
    )?;
    let tree_data = crate::core::objects::format_object_data("tree", &tree_content);
    let tree = crate::core::objects::tree::Tree::deserialize(&tree_data)?;
    build_index_from_tree(&repo_dir, &tree, "")?;

    println!(
        "Initialized empty Git repository in {}/.git/",
        repo_dir.display()
    );
    println!(
        "Cloned branch '{}' ({} objects)",
        branch_name,
        objects.len()
    );

    Ok(())
}

fn repo_name_from_url(url: &str) -> String {
    let path = url.split('/').next_back().unwrap_or("repo");
    path.strip_suffix(".git").unwrap_or(path).to_string()
}

fn extract_branch_from_refs(head_sha1: &str, refs_list: &[(String, String)]) -> String {
    for (sha1, name) in refs_list {
        if sha1 == head_sha1 {
            if let Some(branch) = name.strip_prefix("refs/heads/") {
                return branch.to_string();
            }
        }
    }
    "main".to_string()
}

fn ref_name_from_head<'a>(sha1: &str, refs_list: &'a [(String, String)]) -> &'a str {
    for (s, name) in refs_list {
        if s == sha1 && name.starts_with("refs/heads/") {
            return name;
        }
    }
    ""
}

#[allow(dead_code)]
fn object_exists_in_list(sha1: &str, objects: &[(String, Vec<u8>)]) -> bool {
    objects.iter().any(|(s, _)| s == sha1)
}

fn init_git_dir(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");
    fs::create_dir(&git_dir)?;
    fs::create_dir(git_dir.join("objects"))?;
    fs::create_dir(git_dir.join("objects").join("pack"))?;
    fs::create_dir(git_dir.join("objects").join("info"))?;
    repo::ensure_dir(&git_dir.join("refs").join("heads"))?;
    repo::ensure_dir(&git_dir.join("refs").join("tags"))?;
    repo::ensure_dir(&git_dir.join("refs").join("remotes"))?;
    repo::ensure_dir(&git_dir.join("refs").join("remotes").join("origin"))?;

    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
    fs::write(
        git_dir.join("config"),
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = false\n\tbare = false\n",
    )?;
    Ok(())
}

fn checkout_head(repo: &Path, head_sha1: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, head_sha1)?;
    if obj_type != "commit" {
        return Ok(());
    }

    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = crate::core::objects::commit::Commit::deserialize(&commit_data)?;

    let (_, tree_content) = storage::read_object(repo, &commit.tree)?;
    let tree_data = crate::core::objects::format_object_data("tree", &tree_content);
    let tree = crate::core::objects::tree::Tree::deserialize(&tree_data)?;

    remote_utils::apply_tree(repo, "", &tree)?;

    Ok(())
}

fn build_index_from_tree(
    repo: &Path,
    tree: &crate::core::objects::tree::Tree,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut index = Index::load(repo)?;
    add_tree_to_index(repo, tree, prefix, &mut index)?;
    index.save(repo)?;
    Ok(())
}

fn add_tree_to_index(
    repo: &Path,
    tree: &crate::core::objects::tree::Tree,
    prefix: &str,
    index: &mut Index,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in &tree.entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.mode == "40000" {
            let (_, sub_content) = storage::read_object(repo, &entry.sha1)?;
            let sub_data = crate::core::objects::format_object_data("tree", &sub_content);
            let sub_tree = crate::core::objects::tree::Tree::deserialize(&sub_data)?;
            add_tree_to_index(repo, &sub_tree, &path, index)?;
        } else {
            index.add_entry(&entry.mode, &entry.sha1, &path);
        }
    }
    Ok(())
}

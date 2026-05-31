use crate::core::protocol::HttpTransport;
use crate::core::refs;
use crate::core::remote_utils;
use crate::core::repo;
use crate::core::storage;
use std::fs;
use std::path::Path;

pub fn run(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloning into '{}'...", repo_name_from_url(url));

    let transport = HttpTransport::from_url(url)?;
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

    let objects = transport.clone_full(&head_ref)?;

    init_git_dir(&repo_dir)?;
    remote_utils::write_objects(&repo_dir, &objects)?;

    for (ref_name, ref_sha1) in &refs_list {
        if (ref_name.starts_with("refs/heads/") || ref_name.starts_with("refs/tags/"))
            && object_exists_in_list(ref_sha1, &objects)
        {
            refs::write_ref(&repo_dir, ref_name, ref_sha1)?;
        }
    }

    refs::write_head(&repo_dir, &format!("ref: refs/heads/{}", branch_name))?;

    checkout_head(&repo_dir, &head_ref)?;

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

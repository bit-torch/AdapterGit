use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let index = Index::load(&repo_root)?;

    let head_sha1 = match refs::read_head(&repo_root) {
        Ok(sha1) => sha1,
        Err(_) => {
            println!("On branch main");
            println!();
            println!("No commits yet");
            println!();
            list_untracked(&repo_root, &index);
            return Ok(());
        }
    };

    let branch = current_branch(&repo_root).unwrap_or_else(|| "main".to_string());
    println!("On branch {}", branch);

    let head_tree = get_head_tree(&repo_root, &head_sha1);

    let staged = get_staged_changes(&index, &head_tree);
    let modified = get_modified_changes(&repo_root, &index);
    let deleted = get_deleted_changes(&repo_root, &index, &head_tree);

    if staged.is_empty() && modified.is_empty() && deleted.is_empty() {
        println!("nothing to commit, working tree clean");
    } else {
        if !staged.is_empty() {
            println!();
            println!("Changes to be committed:");
            println!("  (use \"git restore --staged <file>...\" to unstage)");
            for file in &staged {
                println!("        new file: {}", file);
            }
        }

        if !modified.is_empty() {
            println!();
            println!("Changes not staged for commit:");
            println!("  (use \"git add <file>...\" to update what will be committed)");
            for file in &modified {
                println!("        modified: {}", file);
            }
        }

        if !deleted.is_empty() {
            println!();
            println!("Changes not staged for commit:");
            for file in &deleted {
                println!("        deleted: {}", file);
            }
        }
    }

    println!();
    list_untracked(&repo_root, &index);

    Ok(())
}

fn current_branch(repo: &Path) -> Option<String> {
    let head_content = fs::read_to_string(repo.join(".git").join("HEAD")).ok()?;
    let head_content = head_content.trim();
    if let Some(ref_path) = head_content.strip_prefix("ref: ") {
        ref_path.strip_prefix("refs/heads/").map(|s| s.to_string())
    } else {
        None
    }
}

fn get_head_tree(repo: &Path, sha1: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if let Ok((obj_type, content)) = storage::read_object(repo, sha1) {
        if obj_type == "commit" {
            if let Ok(commit) = Commit::deserialize(&crate::core::objects::format_object_data(
                "commit", &content,
            )) {
                let _ = collect_tree_recursive(repo, &commit.tree, "", &mut result);
            }
        }
    }
    result
}

/// 递归收集 tree 中所有文件路径 → SHA-1 映射。
fn collect_tree_recursive(
    repo: &Path,
    tree_sha1: &str,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, tree_sha1)?;
    if obj_type != "tree" {
        return Ok(());
    }
    let tree_data = crate::core::objects::format_object_data("tree", &content);
    let tree = Tree::deserialize(&tree_data)?;
    for entry in &tree.entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };
        if entry.mode == "40000" {
            collect_tree_recursive(repo, &entry.sha1, &path, out)?;
        } else {
            out.insert(path, entry.sha1.clone());
        }
    }
    Ok(())
}

fn get_staged_changes(index: &Index, head_tree: &BTreeMap<String, String>) -> Vec<String> {
    let mut staged = Vec::new();
    for (path, entry) in &index.entries {
        if let Some(head_sha1) = head_tree.get(path) {
            if head_sha1 != &entry.sha1 {
                staged.push(path.clone());
            }
        } else {
            staged.push(path.clone());
        }
    }
    staged
}

fn get_modified_changes(repo: &Path, index: &Index) -> Vec<String> {
    let mut modified = Vec::new();
    for (path, entry) in &index.entries {
        let full_path = repo.join(path);
        if !full_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read(&full_path) {
            let blob = crate::core::objects::blob::Blob::new(content);
            if blob.hash() != entry.sha1 {
                modified.push(path.clone());
            }
        }
    }
    modified
}

fn get_deleted_changes(
    repo: &Path,
    index: &Index,
    head_tree: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut deleted = Vec::new();
    let mut tracked: BTreeSet<&str> = BTreeSet::new();
    for path in head_tree.keys() {
        tracked.insert(path);
    }
    for path in index.entries.keys() {
        tracked.insert(path);
    }
    for path in tracked {
        if !repo.join(path).exists() {
            deleted.push(path.to_string());
        }
    }
    deleted
}

fn list_untracked(repo: &Path, index: &Index) {
    let mut untracked = Vec::new();
    let matcher = crate::core::ignore::IgnoreMatcher::load(repo, Path::new(""));
    if let Err(e) = collect_untracked(repo, repo, index, &matcher, &mut untracked) {
        eprintln!("error listing untracked: {}", e);
        return;
    }
    if !untracked.is_empty() {
        println!("Untracked files:");
        println!("  (use \"git add <file>...\" to include in what will be committed)");
        for file in &untracked {
            println!("\t{}", file);
        }
        println!();
    }
}

fn collect_untracked(
    repo: &Path,
    current: &Path,
    index: &Index,
    matcher: &crate::core::ignore::IgnoreMatcher,
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

        let is_dir = path.is_dir();

        // 跳过被 ignore 的文件和目录
        if matcher.is_ignored(&relative, is_dir) {
            continue;
        }

        if is_dir {
            collect_untracked(repo, &path, index, matcher, untracked)?;
        } else if path.is_file() && !index.entries.contains_key(&relative.to_string()) {
            untracked.push(relative.to_string());
        }
    }
    Ok(())
}

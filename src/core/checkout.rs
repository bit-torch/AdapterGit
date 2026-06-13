use crate::core::objects::blob::Blob;
use crate::core::objects::tree::Tree;
use crate::core::{index, refs, storage};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// 从 commit SHA 还原整个工作目录和索引。
/// 用于 reset --hard 等需要重建工作区的操作。
pub fn restore_from_commit(
    repo: &Path,
    commit_sha: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, body) = storage::read_object(repo, commit_sha)?;
    if obj_type != "commit" {
        return Err(format!("object {} is not a commit", commit_sha).into());
    }
    let commit_data = with_object_header("commit", &body);
    let commit = crate::core::objects::commit::Commit::deserialize(&commit_data)?;

    let old_index = index::Index::load(repo)?;
    let old_tracked: BTreeSet<String> = old_index.entries.keys().cloned().collect();

    restore_tree(repo, &commit.tree, Path::new(""))?;

    let new_tracked = collect_tree_paths(repo, &commit.tree, Path::new(""))?;

    for path in old_tracked.iter() {
        if !new_tracked.contains(path) {
            let file_path = repo.join(path);
            if file_path.is_file() || file_path.is_symlink() {
                let _ = fs::remove_file(&file_path);
            }
        }
    }
    let _ = remove_empty_dirs(repo, Path::new(""));
    Ok(())
}

/// 从 commit 的 tree 重建 index（不修改工作区）。
pub fn rebuild_index_from_commit(
    repo: &Path,
    commit_sha: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, body) = storage::read_object(repo, commit_sha)?;
    if obj_type != "commit" {
        return Err(format!("object {} is not a commit", commit_sha).into());
    }
    let commit_data = with_object_header("commit", &body);
    let commit = crate::core::objects::commit::Commit::deserialize(&commit_data)?;
    let mut new_index = index::Index::new();
    rebuild_index_from_tree(repo, &commit.tree, Path::new(""), &mut new_index)?;
    new_index.save(repo)?;
    Ok(())
}

/// 仅从 tree 递归重建 index 条目（不写工作区文件）。
fn rebuild_index_from_tree(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
    idx: &mut index::Index,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, body) = storage::read_object(repo, tree_sha)?;
    if obj_type != "tree" {
        return Ok(());
    }
    let tree_data = with_object_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;
    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);
        if entry.mode == "40000" {
            rebuild_index_from_tree(repo, &entry.sha1, &entry_path, idx)?;
        } else {
            idx.add_entry(&entry.mode, &entry.sha1, &entry_path.to_string_lossy());
        }
    }
    Ok(())
}

/// 切换到指定分支（更新 HEAD 和工作目录）。
pub fn switch_branch(repo: &Path, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_sha = refs::read_ref(repo, &format!("refs/heads/{}", branch_name))?;
    let (obj_type, body) = storage::read_object(repo, &target_sha)?;
    if obj_type != "commit" {
        return Err(format!("ref '{}' does not point to a commit", branch_name).into());
    }
    let commit_data = with_object_header("commit", &body);
    let commit = crate::core::objects::commit::Commit::deserialize(&commit_data)?;

    // 记录当前索引中的文件，用于切换后清理废弃文件
    let old_index = index::Index::load(repo)?;
    let old_tracked: BTreeSet<String> = old_index.entries.keys().cloned().collect();

    restore_tree(repo, &commit.tree, Path::new(""))?;

    // 收集目标树中的所有文件
    let new_tracked = collect_tree_paths(repo, &commit.tree, Path::new(""))?;

    // 删除旧索引中存在但新树中不存在的文件
    for path in old_tracked.iter() {
        if !new_tracked.contains(path) {
            let file_path = repo.join(path);
            if file_path.is_file() || file_path.is_symlink() {
                let _ = fs::remove_file(&file_path);
            }
        }
    }
    // 清理空目录
    let _ = remove_empty_dirs(repo, Path::new(""));

    refs::write_head(repo, &format!("ref: refs/heads/{}", branch_name))?;

    println!("Switched to branch '{}'", branch_name);
    Ok(())
}

/// 递归恢复 tree 内容到工作目录。
fn restore_tree(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, body) = storage::read_object(repo, tree_sha)?;
    if obj_type != "tree" {
        return Err(format!("object {} is not a tree", tree_sha).into());
    }

    let tree_data = with_object_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;
    let mut new_index = index::Index::new();

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);

        match entry.mode.as_str() {
            "40000" => {
                let dir_path = repo.join(&entry_path);
                fs::create_dir_all(&dir_path)?;
                restore_tree(repo, &entry.sha1, &entry_path)?;
            }
            _ => {
                let (blob_type, blob_body) = storage::read_object(repo, &entry.sha1)?;
                if blob_type != "blob" {
                    continue;
                }
                let blob_data = with_object_header("blob", &blob_body);
                let blob = Blob::deserialize(&blob_data)?;

                let file_path = repo.join(&entry_path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&file_path, &blob.content)?;

                new_index.add_entry(&entry.mode, &entry.sha1, &entry_path.to_string_lossy());
            }
        }
    }

    new_index.save(repo)?;
    Ok(())
}

/// 重新构造完整的对象数据：`{type} {len}\0{body}`
fn with_object_header(obj_type: &str, body: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type, body.len());
    let mut data = Vec::with_capacity(header.len() + body.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(body);
    data
}

/// 递归收集树中所有文件路径。
fn collect_tree_paths(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut paths = BTreeSet::new();
    let (obj_type, body) = match storage::read_object(repo, tree_sha) {
        Ok(v) => v,
        Err(_) => return Ok(paths),
    };
    if obj_type != "tree" {
        return Ok(paths);
    }
    let tree_data = with_object_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);
        let path_str = entry_path.to_string_lossy().to_string();

        if entry.mode == "40000" {
            let sub = collect_tree_paths(repo, &entry.sha1, &entry_path)?;
            paths.extend(sub);
        } else {
            paths.insert(path_str);
        }
    }
    Ok(paths)
}

/// 递归清理空目录。
fn remove_empty_dirs(repo: &Path, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let full = repo.join(dir);
    if !full.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&full)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let relative = path.strip_prefix(repo).unwrap_or(&path);
            remove_empty_dirs(repo, relative)?;
        }
    }
    // 再次读取目录，如果为空则删除
    if full
        .read_dir()
        .map(|mut d| d.next().is_none())
        .unwrap_or(false)
    {
        let _ = fs::remove_dir(&full);
    }
    Ok(())
}

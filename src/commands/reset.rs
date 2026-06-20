use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::{checkout, refs, repo, storage};
use std::collections::BTreeMap;
use std::path::Path;

pub fn run(
    soft: bool,
    _mixed: bool,
    hard: bool,
    commit: Option<&str>,
    files: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let head_sha =
        refs::read_head(&repo_root).map_err(|e| format!("Failed to read HEAD: {}", e))?;

    // 如果指定了文件路径，执行 unstage 模式
    if !files.is_empty() {
        let target_sha = match repo::resolve_commit(&repo_root, commit.unwrap_or("HEAD")) {
            Ok(sha) => Some(sha),
            Err(_) if head_sha.is_empty() => None, // 尚无提交，没有 HEAD tree
            Err(e) => return Err(e),
        };
        return unstage_files(&repo_root, target_sha.as_deref(), files);
    }

    // 否则为 commit 模式：解析目标 commit
    let target_sha = match commit {
        Some(c) => repo::resolve_commit(&repo_root, c)?,
        None => {
            // 没有文件也没有 commit → 默认重置到 HEAD（相当于重新计算 index）
            if head_sha.is_empty() {
                return Err("No commits yet".into());
            }
            head_sha.clone()
        }
    };

    if hard {
        reset_hard(&repo_root, &target_sha)?;
    } else if soft {
        reset_soft(&repo_root, &target_sha)?;
    } else {
        // --mixed 为默认行为
        reset_mixed(&repo_root, &target_sha)?;
    }

    Ok(())
}

/// 从 index 中移除指定文件的条目（取消暂存）。
fn unstage_files(
    repo: &Path,
    commit_sha: Option<&str>,
    files: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let head_tree_map = match commit_sha {
        Some(sha) => read_commit_tree(repo, sha)?,
        None => BTreeMap::new(), // 尚无提交，HEAD tree 为空
    };
    let mut index = Index::load(repo)?;

    for file in files {
        if let Some(head_sha) = head_tree_map.get(file) {
            // 文件在 HEAD 中存在 → 将 index 恢复为 HEAD 版本
            index.add_entry("100644", head_sha, file);
        } else {
            // 文件不在 HEAD 中（新文件）→ 从 index 移除
            index.remove_entry(file);
        }
    }

    index.save(repo)?;
    Ok(())
}

/// --soft：仅移动 HEAD，不动 index 和工作区。
fn reset_soft(repo: &Path, target_sha: &str) -> Result<(), Box<dyn std::error::Error>> {
    move_head(repo, target_sha)?;
    println!("HEAD is now at {}", &target_sha[..7]);
    Ok(())
}

/// --mixed（默认）：移动 HEAD + 用目标 commit 的 tree 重建 index。
fn reset_mixed(repo: &Path, target_sha: &str) -> Result<(), Box<dyn std::error::Error>> {
    move_head(repo, target_sha)?;
    checkout::rebuild_index_from_commit(repo, target_sha)?;
    println!("HEAD is now at {}", &target_sha[..7]);
    Ok(())
}

/// --hard：移动 HEAD + 重建 index + 覆盖工作区。
fn reset_hard(repo: &Path, target_sha: &str) -> Result<(), Box<dyn std::error::Error>> {
    move_head(repo, target_sha)?;
    checkout::restore_from_commit(repo, target_sha)?;
    println!("HEAD is now at {}", &target_sha[..7]);
    Ok(())
}

/// 移动当前分支引用（或 detached HEAD）到目标 commit。
fn move_head(repo: &Path, target_sha: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(branch) = refs::get_current_branch(repo)? {
        refs::write_ref(repo, &format!("refs/heads/{}", branch), target_sha)?;
    } else {
        // detached HEAD
        refs::write_head(repo, target_sha)?;
    }
    Ok(())
}

/// 读取 commit 的 tree，返回 path → sha1 映射。
fn read_commit_tree(
    repo: &Path,
    commit_sha: &str,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut result = BTreeMap::new();
    let (obj_type, content) = storage::read_object(repo, commit_sha)?;
    if obj_type != "commit" {
        return Ok(result);
    }
    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;
    collect_tree_recursive(repo, &commit.tree, "", &mut result)?;
    Ok(result)
}

fn collect_tree_recursive(
    repo: &Path,
    tree_sha: &str,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, tree_sha)?;
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

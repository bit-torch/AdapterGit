//! stash 命令：临时保存工作区变更。
//!
//! stash 结构：每个 stash 是一个 commit，parent 指向 HEAD，树捕获工作区+索引状态。
//! 多个 stash 以线性链存储在 `refs/stash`，最新 stash 在顶端。

use crate::core::index::Index;
use crate::core::objects::blob::Blob;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::{checkout, refs, repo, storage};
use std::fs;

/// `stash push`：保存当前工作区状态并重置到 HEAD。
pub fn run_push() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let head_sha = refs::read_head(&repo_root)?;
    let index = Index::load(&repo_root)?;

    // 检查是否有实际变更（工作区 vs index）
    let mut has_changes = false;
    for (path, entry) in &index.entries {
        let file_path = repo_root.join(path);
        if file_path.exists() {
            let content = fs::read(&file_path)
                .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            let blob = Blob::new(content);
            if blob.hash() != entry.sha1 {
                has_changes = true;
                break;
            }
        } else {
            // 文件被删除
            has_changes = true;
            break;
        }
    }
    if !has_changes {
        return Err("No local changes to save".into());
    }

    let config = crate::config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        config.user_name, config.user_email, timestamp, time_str
    );

    // 从当前工作区构建 tree（使用 index entries 但替换为工作区实际内容）
    let mut tree = Tree::new();
    for path in index.entries.keys() {
        let file_path = repo_root.join(path);
        if file_path.exists() {
            let content = fs::read(&file_path)
                .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            let blob = Blob::new(content);
            let sha1 = blob.hash();
            // 只在 object 不存在时写入
            if !storage::object_exists(&repo_root, &sha1) {
                storage::write_object(&repo_root, "blob", &blob.content)?;
            }
            tree.add_entry("100644", path, &sha1);
        }
    }
    let tree_sha = tree.hash();
    storage::write_object(&repo_root, "tree", &tree.serialize_raw())?;

    // 创建 stash commit，第二个 parent 链接到之前的 stash
    let mut stash_commit = Commit::new(&tree_sha, &author, &author, "WIP on stash\n");
    stash_commit.add_parent(&head_sha);

    // 如果已有 stash，链上作为第二个 parent（模拟 reflog 链）
    if let Ok(prev_stash) = refs::read_ref(&repo_root, "refs/stash") {
        stash_commit.add_parent(&prev_stash);
    }

    let stash_sha = stash_commit.hash();
    storage::write_object(&repo_root, "commit", &stash_commit.serialize_raw())?;

    // 更新 refs/stash
    refs::write_ref(&repo_root, "refs/stash", &stash_sha)?;

    // 重置工作区到 HEAD
    checkout::restore_from_commit(&repo_root, &head_sha)?;

    println!("Saved working directory and index state WIP on stash");
    Ok(())
}

/// `stash pop`：恢复最近的 stash 并删除。
pub fn run_pop() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let stash_sha = refs::read_ref(&repo_root, "refs/stash")?;

    // 读取 stash commit 获取 tree
    let (obj_type, content) = storage::read_object(&repo_root, &stash_sha)?;
    if obj_type != "commit" {
        return Err("refs/stash does not point to a commit".into());
    }
    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;

    // 恢复 stash 的 tree 到工作区
    checkout::restore_from_commit(&repo_root, &stash_sha)?;

    // 链到下一个 stash（第二个 parent）
    if commit.parents.len() >= 2 {
        let next_stash = &commit.parents[1];
        refs::write_ref(&repo_root, "refs/stash", next_stash)?;
    } else {
        // 没有更多 stash，删除引用
        let ref_path = repo_root.join(".git").join("refs").join("stash");
        let _ = fs::remove_file(&ref_path);
    }

    println!("Dropped refs/stash@{{0}}");
    Ok(())
}

/// `stash list`：列出所有 stash。
pub fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let mut current = match refs::read_ref(&repo_root, "refs/stash") {
        Ok(sha) => sha,
        Err(_) => {
            println!("No stashes found.");
            return Ok(());
        }
    };

    let mut index = 0;
    loop {
        let (obj_type, content) = match storage::read_object(&repo_root, &current) {
            Ok(v) => v,
            Err(_) => break,
        };
        if obj_type != "commit" {
            break;
        }
        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = match Commit::deserialize(&commit_data) {
            Ok(c) => c,
            Err(_) => break,
        };

        let msg = commit.message.lines().next().unwrap_or("(no message)");
        println!(
            "stash@{{{}}}: WIP on {}: {} {}",
            index,
            &commit.parents.first().map(|s| &s[..7]).unwrap_or("???????"),
            msg,
            &current[..7]
        );

        // 第二个 parent 是下一个 stash（链式存储）
        if commit.parents.len() >= 2 {
            current = commit.parents[1].clone();
        } else {
            break;
        }
        index += 1;
    }

    Ok(())
}

/// `stash drop`：删除指定的 stash。
pub fn run_drop(stash_ref: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let index = parse_stash_index(stash_ref);

    if index == 0 {
        // 删除栈顶 stash，等价于 pop 但不恢复
        let stash_sha = refs::read_ref(&repo_root, "refs/stash")?;
        let (_, content) = storage::read_object(&repo_root, &stash_sha)?;
        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = Commit::deserialize(&commit_data)?;

        if commit.parents.len() >= 2 {
            refs::write_ref(&repo_root, "refs/stash", &commit.parents[1])?;
        } else {
            let _ = fs::remove_file(repo_root.join(".git").join("refs").join("stash"));
        }
        println!("Dropped refs/stash@{{0}}");
    } else {
        // 删除非栈顶 stash：需要遍历链并重建
        return Err(format!(
            "error: stash@{{{}}} not at stack top; only stash@{{0}} can be dropped currently",
            index
        )
        .into());
    }

    Ok(())
}

/// 解析 "stash@{N}" 或 "N" 为索引号。
fn parse_stash_index(spec: Option<&str>) -> usize {
    match spec {
        None => 0,
        Some(s) => {
            // 支持 "stash@{0}" 或 "0"
            if let Some(inner) = s.strip_prefix("stash@{").and_then(|r| r.strip_suffix('}')) {
                inner.parse().unwrap_or(0)
            } else {
                s.parse().unwrap_or(0)
            }
        }
    }
}

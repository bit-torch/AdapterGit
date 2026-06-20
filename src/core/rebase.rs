//! Rebase 和 cherry-pick 共享的核心逻辑。
//!
//! 提供：pick_commit / collect_commits_between / create_commit_from_index / TODO 状态管理。

use crate::core::index::Index;
use crate::core::merge::{read_commit_tree_files, three_way_merge_with_files, FileInfo};
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::{refs, storage};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// pick_commit 的结果。
pub(crate) enum PickResult {
    /// 干净应用，返回新 commit SHA。
    Clean(String),
    /// 产生冲突，工作区已写入冲突标记。
    Conflict,
}

/// 收集两个 commit 之间的所有提交（沿 first-parent 链）。
///
/// 从 `head_sha` 回走到 `base_sha`（不含），返回时间正序列表。
/// 如果 head == base 则返回空 Vec。
pub(crate) fn collect_commits_between(
    repo: &Path,
    base_sha: &str,
    head_sha: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if head_sha == base_sha {
        return Ok(Vec::new());
    }

    let mut commits = Vec::new();
    let mut current = head_sha.to_string();
    let mut iterations = 0;

    loop {
        if iterations > 1000 {
            return Err("rebase: too many commits (possible cycle)".into());
        }
        if current == base_sha {
            break;
        }
        commits.push(current.clone());

        let (obj_type, body) = storage::read_object(repo, &current)?;
        if obj_type != "commit" {
            return Err(format!("object {} is not a commit", current).into());
        }
        let commit_data = crate::core::objects::format_object_data("commit", &body);
        let commit = Commit::deserialize(&commit_data)?;

        match commit.parents.first() {
            Some(parent) => current = parent.clone(),
            None => break, // root commit, no more parents
        }
        iterations += 1;
    }

    commits.reverse(); // 最旧的排在最前面
    Ok(commits)
}

/// 将单个 commit 应用到当前 HEAD。
///
/// `parent_sha` — 被 pick commit 的父提交（用于三路合并的 base）。
/// 若为 None 表示根提交，base 为空 BTreeMap。
pub(crate) fn pick_commit(
    repo: &Path,
    commit_sha: &str,
    parent_sha: Option<&str>,
) -> Result<PickResult, Box<dyn std::error::Error>> {
    // 读取被 pick 的 commit
    let (_, body) = storage::read_object(repo, commit_sha)?;
    let commit_data = crate::core::objects::format_object_data("commit", &body);
    let commit = Commit::deserialize(&commit_data)?;

    let head_sha = refs::read_head(repo)?;

    // 获取三路合并的文件映射
    let base_files: BTreeMap<String, FileInfo> = match parent_sha {
        Some(parent) => read_commit_tree_files(repo, parent)?,
        None => BTreeMap::new(), // 根提交：空 base
    };
    let ours_files = read_commit_tree_files(repo, &head_sha)?;
    let theirs_files = read_commit_tree_files(repo, commit_sha)?;

    // 执行三路合并
    let mut new_index = Index::new();
    let theirs_label: String = commit_sha[..7].to_string();
    let has_conflicts = three_way_merge_with_files(
        repo,
        &base_files,
        &ours_files,
        &theirs_files,
        &mut new_index,
        Some(&theirs_label),
    )?;
    new_index.save(repo)?;

    if has_conflicts {
        return Ok(PickResult::Conflict);
    }

    // 从当前索引创建新 commit
    let new_sha = create_commit_from_index(
        repo,
        &commit.author,
        &commit.committer,
        &commit.message,
        &[head_sha.as_str()],
    )?;

    Ok(PickResult::Clean(new_sha))
}

/// 从当前索引创建 commit，直接写入 HEAD（支持分离 HEAD）。
///
/// 返回新 commit SHA。
pub(crate) fn create_commit_from_index(
    repo: &Path,
    author: &str,
    committer: &str,
    message: &str,
    parents: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let index = Index::load(repo)?;
    if index.entries.is_empty() {
        return Err("Nothing to commit (empty index)".into());
    }

    // 从索引构建树
    let mut tree = Tree::new();
    for entry in index.entries.values() {
        tree.add_entry(&entry.mode, &entry.path, &entry.sha1);
    }
    let tree_sha = tree.hash();
    storage::write_object(repo, "tree", &tree.serialize_raw())?;

    // 创建 commit
    let mut commit = Commit::new(&tree_sha, author, committer, message);
    for parent in parents {
        commit.add_parent(parent);
    }

    let commit_sha = commit.hash();
    storage::write_object(repo, "commit", &commit.serialize_raw())?;

    // 直接写入 HEAD
    refs::write_head(repo, &commit_sha)?;

    Ok(commit_sha)
}

// ── TODO 状态文件管理 ──────────────────────────────────────

/// 写入 REBASE_TODO 文件（每行一个 SHA，最旧的在前）。
pub(crate) fn write_todo(
    repo: &Path,
    commits: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("REBASE_TODO");
    let content: String = commits.iter().map(|s| format!("{}\n", s)).collect();
    fs::write(path, content)?;
    Ok(())
}

/// 读取 REBASE_TODO 文件。
pub(crate) fn read_todo(repo: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("REBASE_TODO");
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// 弹出 REBASE_TODO 的第一个 commit（返回后文件缩小一项）。
pub(crate) fn pop_todo(repo: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut commits = read_todo(repo)?;
    if commits.is_empty() {
        return Ok(None);
    }
    let first = commits.remove(0);
    write_todo(repo, &commits)?;
    Ok(Some(first))
}

// ── Cherry-pick TODO 状态文件 ──────────────────────────────

/// 写入 CHERRY_PICK_TODO 文件。
pub(crate) fn write_cherry_todo(
    repo: &Path,
    commits: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("CHERRY_PICK_TODO");
    let content: String = commits.iter().map(|s| format!("{}\n", s)).collect();
    fs::write(path, content)?;
    Ok(())
}

/// 读取 CHERRY_PICK_TODO 文件。
pub(crate) fn read_cherry_todo(repo: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("CHERRY_PICK_TODO");
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// 弹出 CHERRY_PICK_TODO 的第一个 commit。
pub(crate) fn pop_cherry_todo(repo: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut commits = read_cherry_todo(repo)?;
    if commits.is_empty() {
        return Ok(None);
    }
    let first = commits.remove(0);
    write_cherry_todo(repo, &commits)?;
    Ok(Some(first))
}

// ── 单元测试 ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("agit_ut_rebase_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git").join("objects")).unwrap();
        fs::create_dir_all(dir.join(".git").join("refs").join("heads")).unwrap();
        fs::write(dir.join(".git").join("HEAD"), "ref: refs/heads/main\n").unwrap();
        dir
    }

    #[test]
    fn test_todo_roundtrip() {
        let repo = setup_test_repo("todo");
        let commits = vec![
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
            "cafebabecafebabecafebabecafebabecafebabe".to_string(),
        ];

        write_todo(&repo, &commits).unwrap();
        let read = read_todo(&repo).unwrap();
        assert_eq!(read, commits);

        let first = pop_todo(&repo).unwrap();
        assert_eq!(
            first,
            Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string())
        );

        let remaining = read_todo(&repo).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0], "cafebabecafebabecafebabecafebabecafebabe");

        let last = pop_todo(&repo).unwrap();
        assert_eq!(
            last,
            Some("cafebabecafebabecafebabecafebabecafebabe".to_string())
        );

        let empty = pop_todo(&repo).unwrap();
        assert!(empty.is_none());

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_cherry_todo_roundtrip() {
        let repo = setup_test_repo("cherry");
        let commits = vec!["1234567890abcdef1234567890abcdef12345678".to_string()];

        write_cherry_todo(&repo, &commits).unwrap();
        let read = read_cherry_todo(&repo).unwrap();
        assert_eq!(read, commits);

        let first = pop_cherry_todo(&repo).unwrap();
        assert_eq!(
            first,
            Some("1234567890abcdef1234567890abcdef12345678".to_string())
        );
        let empty = pop_cherry_todo(&repo).unwrap();
        assert!(empty.is_none());

        let _ = fs::remove_dir_all(&repo);
    }
}

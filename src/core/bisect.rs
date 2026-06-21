//! 二分查找（bisect）核心模块。
//!
//! 实现 `git bisect` 的状态管理和范围计算算法。
//!
//! 状态持久化使用 `refs/bisect/*` 命名空间（与 Git 兼容）:
//! - `refs/bisect/bad` — 当前已知的 bad 提交
//! - `refs/bisect/good-*` — 已知的 good 提交
//! - `refs/bisect/skip-*` — 跳过的提交
//!
//! 此外还有日志文件:
//! - `.git/BISECT_LOG` — 操作日志
//! - `.git/BISECT_START` — 原始的 HEAD 引用，用于 reset

use crate::core::objects::commit::Commit;
use crate::core::objects::format_object_data;
use crate::core::{refs, storage};
use std::fs;
use std::path::Path;

/// 当前二分查找的状态。
#[derive(Debug, Clone)]
pub struct BisectState {
    /// 已知 bad 提交的 SHA
    pub bad: String,
    /// 已知 good 提交的 SHA 列表
    pub good: Vec<String>,
    /// 跳过的提交 SHA 列表
    pub skip: Vec<String>,
    /// 原始 HEAD（bisect start 时的 HEAD），用于 reset
    pub original_head: String,
    /// 剩余待测试的提交列表（有序，从 old 到 new）
    pub remaining: Vec<String>,
}

impl BisectState {
    /// 检查二分查找是否仍在进行中。
    pub fn is_active(repo: &Path) -> bool {
        let bisect_ref = repo.join(".git").join("refs").join("bisect");
        bisect_ref.exists() && bisect_ref.is_dir()
    }

    /// 从 refs/bisect/* 加载当前状态。
    pub fn load(repo: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bad = refs::read_ref(repo, "refs/bisect/bad")?;
        let good = load_indexed_refs(repo, "refs/bisect/good-")?;
        let skip = load_indexed_refs(repo, "refs/bisect/skip-")?;
        let remaining = load_remaining_list(repo)?;
        let original_head = load_original_head(repo)?;

        Ok(BisectState {
            bad,
            good,
            skip,
            original_head,
            remaining,
        })
    }

    /// 将当前状态保存到 refs/bisect/*。
    pub fn save(&self, repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let bisect_dir = repo.join(".git").join("refs").join("bisect");
        fs::create_dir_all(&bisect_dir)?;

        // 清除旧的 good/skip 引用
        clear_indexed_refs(repo, "refs/bisect/good-")?;
        clear_indexed_refs(repo, "refs/bisect/skip-")?;

        // 写入 bad
        refs::write_ref(repo, "refs/bisect/bad", &self.bad)?;

        // 写入 good 列表
        for (i, sha) in self.good.iter().enumerate() {
            refs::write_ref(repo, &format!("refs/bisect/good-{}", i), sha)?;
        }

        // 写入 skip 列表
        for (i, sha) in self.skip.iter().enumerate() {
            refs::write_ref(repo, &format!("refs/bisect/skip-{}", i), sha)?;
        }

        // 保存剩余列表
        save_remaining_list(repo, &self.remaining)?;
        // 保存原始 HEAD
        save_original_head(repo, &self.original_head)?;

        Ok(())
    }

    /// 清除所有 bisect 状态（引用 + 文件）。
    pub fn clear(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let bisect_ref_dir = repo.join(".git").join("refs").join("bisect");
        if bisect_ref_dir.exists() {
            fs::remove_dir_all(&bisect_ref_dir)?;
        }
        let bisect_log = repo.join(".git").join("BISECT_LOG");
        if bisect_log.exists() {
            fs::remove_file(&bisect_log)?;
        }
        let bisect_start = repo.join(".git").join("BISECT_START");
        if bisect_start.exists() {
            fs::remove_file(&bisect_start)?;
        }
        let remaining_file = repo.join(".git").join("BISECT_REMAINING");
        if remaining_file.exists() {
            fs::remove_file(&remaining_file)?;
        }
        Ok(())
    }
}

/// 将剩余待测提交列表写入文件。
fn save_remaining_list(repo: &Path, list: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_REMAINING");
    let content = list.join("\n");
    crate::utils::atomic_write(&path, content.as_bytes())?;
    Ok(())
}

/// 读取剩余待测提交列表。
fn load_remaining_list(repo: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_REMAINING");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    Ok(content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// 保存原始 HEAD 引用。
fn save_original_head(repo: &Path, head: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_START");
    crate::utils::atomic_write(&path, format!("{}\n", head).as_bytes())?;
    Ok(())
}

/// 读取原始 HEAD 引用。
fn load_original_head(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_START");
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(fs::read_to_string(&path)?.trim().to_string())
}

/// 从 refs/bisect/<prefix>-N 读取索引引用。
fn load_indexed_refs(repo: &Path, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let bisect_dir = repo.join(".git").join("refs").join("bisect");
    if !bisect_dir.exists() {
        return Ok(result);
    }
    for entry in fs::read_dir(&bisect_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix.strip_prefix("refs/bisect/").unwrap_or(prefix)) {
            let ref_path = format!("refs/bisect/{}", name);
            if let Ok(sha) = refs::read_ref(repo, &ref_path) {
                result.push(sha);
            }
        }
    }
    Ok(result)
}

/// 清除 refs/bisect/<prefix>-* 引用。
fn clear_indexed_refs(repo: &Path, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bisect_dir = repo.join(".git").join("refs").join("bisect");
    if !bisect_dir.exists() {
        return Ok(());
    }
    let stripped = prefix.strip_prefix("refs/bisect/").unwrap_or(prefix);
    for entry in fs::read_dir(&bisect_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(stripped) {
            let path = entry.path();
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// 从 good..bad 范围收集所有候选提交（沿 first-parent 链）。
///
/// 返回从 bad 到 good（不含）的有序列表（oldest first）。
pub fn compute_range(
    repo: &Path,
    bad: &str,
    goods: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut current = bad.to_string();

    // 收集所有 good 提交的祖先集合
    let mut good_ancestors = std::collections::HashSet::new();
    for good in goods {
        let mut g = good.clone();
        loop {
            if !good_ancestors.insert(g.clone()) {
                break;
            }
            match get_parent_sha(repo, &g) {
                Ok(parent) => g = parent,
                Err(_) => break,
            }
        }
    }

    // 沿 first-parent 链遍历
    loop {
        if seen.contains(&current) {
            break;
        }
        seen.insert(current.clone());

        if good_ancestors.contains(&current) {
            // 到达 good 端，停止
            break;
        }

        result.push(current.clone());

        // 检查是否在 goods 列表中
        if goods.contains(&current) {
            break;
        }

        match get_parent_sha(repo, &current) {
            Ok(parent) => current = parent,
            Err(_) => break,
        }
    }

    result.reverse(); // 从 old 到 new
    Ok(result)
}

/// 获取提交的第一父提交 SHA（不反序列化完整提交对象）。
fn get_parent_sha(repo: &Path, sha: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, sha)?;
    if obj_type != "commit" {
        return Err(format!("{} is not a commit", sha).into());
    }
    let commit_data = format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;
    commit
        .parents
        .first()
        .cloned()
        .ok_or_else(|| format!("no parent for {}", sha).into())
}

/// 从剩余列表中选择下一个待测试的提交（二分中间点）。
pub fn pick_next(remaining: &[String]) -> Option<String> {
    if remaining.is_empty() {
        return None;
    }
    let mid = remaining.len() / 2;
    Some(remaining[mid].clone())
}

/// 记录 bisect 操作到 BISECT_LOG。
pub fn log_bisect_action(
    repo: &Path,
    action: &str,
    sha: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_LOG");
    let line = format!("# {}: {}\n", action, sha);
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

/// 读取 BISECT_LOG 内容。
pub fn read_bisect_log(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let path = repo.join(".git").join("BISECT_LOG");
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(fs::read_to_string(&path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage;
    use std::fs;
    use std::path::PathBuf;

    fn setup_repo(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("agit_test_bisect_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git").join("objects")).unwrap();
        fs::create_dir_all(dir.join(".git").join("refs").join("heads")).unwrap();
        fs::create_dir_all(dir.join(".git").join("refs").join("bisect")).unwrap();
        dir
    }

    fn make_commit(repo: &Path, tree: &str, parent: Option<&str>, msg: &str) -> String {
        let mut commit = Commit::new(
            tree,
            "test <test@test.com> 1 +0000",
            "test <test@test.com> 1 +0000",
            msg,
        );
        if let Some(p) = parent {
            commit.add_parent(p);
        }
        let raw = commit.serialize_raw();
        storage::write_object(repo, "commit", &raw).unwrap()
    }

    #[test]
    fn test_bisect_is_active() {
        let repo = setup_repo("is_active");
        // 创建 refs/bisect 目录
        fs::create_dir_all(repo.join(".git").join("refs").join("bisect")).unwrap();
        assert!(BisectState::is_active(&repo));

        // 删除后不再 active
        fs::remove_dir_all(repo.join(".git").join("refs").join("bisect")).unwrap();
        assert!(!BisectState::is_active(&repo));

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_state_save_and_load() {
        let repo = setup_repo("save_load");

        let state = BisectState {
            bad: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            good: vec!["bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()],
            skip: vec![],
            original_head: "refs/heads/main".to_string(),
            remaining: vec![
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            ],
        };
        state.save(&repo).unwrap();

        let loaded = BisectState::load(&repo).unwrap();
        assert_eq!(loaded.bad, state.bad);
        assert_eq!(loaded.good, state.good);
        assert!(loaded.skip.is_empty());
        assert_eq!(loaded.original_head, "refs/heads/main");
        assert_eq!(loaded.remaining.len(), 2);

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_state_clear() {
        let repo = setup_repo("clear");

        let state = BisectState {
            bad: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            good: vec!["bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()],
            skip: vec![],
            original_head: "refs/heads/main".to_string(),
            remaining: vec![],
        };
        state.save(&repo).unwrap();
        assert!(BisectState::is_active(&repo));

        BisectState::clear(&repo).unwrap();
        assert!(!BisectState::is_active(&repo));

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_compute_range_linear() {
        let repo = setup_repo("range_linear");

        // 构建线性历史: root -> c1 -> c2 -> c3 -> bad
        let tree = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let root = make_commit(&repo, tree, None, "root");
        let c1 = make_commit(&repo, tree, Some(&root), "c1");
        let c2 = make_commit(&repo, tree, Some(&c1), "c2");
        let c3 = make_commit(&repo, tree, Some(&c2), "c3");
        let bad = make_commit(&repo, tree, Some(&c3), "bad");

        let range = compute_range(&repo, &bad, std::slice::from_ref(&c1)).unwrap();
        // 范围: c2, c3, bad (不含 c1 及其祖先 root)
        assert_eq!(range.len(), 3);
        assert_eq!(range[0], c2);
        assert_eq!(range[1], c3);
        assert_eq!(range[2], bad);

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_pick_next() {
        let range = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        assert_eq!(pick_next(&range).unwrap(), "c"); // mid of 5

        let empty: Vec<String> = vec![];
        assert!(pick_next(&empty).is_none());

        let single = vec!["x".to_string()];
        assert_eq!(pick_next(&single).unwrap(), "x");
    }

    #[test]
    fn test_log_and_read() {
        let repo = setup_repo("log");

        log_bisect_action(
            &repo,
            "bisect start",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap();
        log_bisect_action(
            &repo,
            "bisect good",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();

        let log_content = read_bisect_log(&repo).unwrap();
        assert!(log_content.contains("bisect start"));
        assert!(log_content.contains("bisect good"));

        let _ = fs::remove_dir_all(&repo);
    }
}

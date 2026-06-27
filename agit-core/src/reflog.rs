//! 引用日志（reflog）模块。
//!
//! reflog 记录了引用（HEAD、分支等）的变更历史，
//! 存储在 `.git/logs/` 目录下。
//!
//! 格式（每行一条记录）:
//! ```text
//! <old-sha> <new-sha> <author> <timestamp> <tab> <message>
//! ```
//!
//! 其中 `<old-sha>` 可能是全零 SHA（表示引用创建），
//! `<new-sha>` 可能是全零 SHA（表示引用删除）。

use std::fs;
use std::path::{Path, PathBuf};

/// 一条 reflog 记录。
#[derive(Debug, Clone)]
pub struct ReflogEntry {
    /// 变更前的 SHA-1（全零表示引用创建）
    pub old_sha: String,
    /// 变更后的 SHA-1（全零表示引用删除）
    pub new_sha: String,
    /// 执行操作的用户
    pub author: String,
    /// Unix 时间戳 + 时区
    pub timestamp: String,
    /// 操作描述（如 "commit: ...", "checkout: ..." 等）
    pub message: String,
}

/// 返回 reflog 文件所在目录。
fn logs_dir(repo: &Path) -> PathBuf {
    repo.join(".git").join("logs")
}

/// 返回指定引用的 reflog 文件路径。
///
/// 例如 `HEAD` → `.git/logs/HEAD`，
/// `refs/heads/main` → `.git/logs/refs/heads/main`。
fn reflog_path(repo: &Path, ref_name: &str) -> PathBuf {
    logs_dir(repo).join(ref_name)
}

/// 读取指定引用的 reflog 记录，按从旧到新的顺序返回。
pub fn read_reflog(
    repo: &Path,
    ref_name: &str,
) -> Result<Vec<ReflogEntry>, Box<dyn std::error::Error>> {
    let path = reflog_path(repo, ref_name);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read reflog {}: {}", path.display(), e))?;

    let mut entries = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(entry) = parse_reflog_line(line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// 追加一条 reflog 记录到指定引用。
#[allow(dead_code)]
pub fn append_reflog(
    repo: &Path,
    ref_name: &str,
    old_sha: &str,
    new_sha: &str,
    author: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (timestamp, timestamp_str) = crate::repo::get_current_timestamp();

    let path = reflog_path(repo, ref_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 构建日志行：<old-sha> <new-sha> <author> <epoch> <tz> <tab> <message>
    let log_line = format!(
        "{} {} {} {}\t{}\n",
        old_sha, new_sha, author, timestamp_str, message
    );

    // 追加写入（文件追加是原子的？在 POSIX 上 O_APPEND 是原子的；
    // 在 Windows 上对单个 write 也是原子的）
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(log_line.as_bytes())?;
    let _ = timestamp;
    Ok(())
}

/// 解析单行 reflog 记录。
fn parse_reflog_line(line: &str) -> Option<ReflogEntry> {
    // 格式: <old-sha> <new-sha> <author> <timestamp> <tz>\t<message>
    let tab_pos = line.find('\t')?;
    let header = &line[..tab_pos];
    let message = line[tab_pos + 1..].to_string();

    // 拆分 header：前两个字段固定为 SHA，后面是 "author epoch tz"
    let parts: Vec<&str> = header.split(' ').collect();
    if parts.len() < 5 {
        // 不足 5 个字段的格式错误，尝试容错
        return None;
    }

    let old_sha = parts[0].to_string();
    let new_sha = parts[1].to_string();
    // author 可能包含空格（"User Name"）？
    // 实际上 git reflog 中 author 格式是 "name <email>"，不含空格，
    // 所以 author = parts[2], epoch = parts[3], tz = parts[4]
    let author = parts[2].to_string();
    let timestamp = format!("{} {}", parts[3], parts[4]);

    Some(ReflogEntry {
        old_sha,
        new_sha,
        author,
        timestamp,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_repo(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("agit_test_reflog_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git").join("logs")).unwrap();
        dir
    }

    #[test]
    fn test_parse_reflog_line() {
        // 使用 append_reflog 实际输出的格式：
        // <old-sha> <new-sha> <author> <epoch> <tz>\t<message>
        let line = "0000000000000000000000000000000000000000 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa testuser 1234567890 +0800\tcommit: Initial commit";
        let entry = parse_reflog_line(line).unwrap();

        assert_eq!(entry.old_sha, "0000000000000000000000000000000000000000");
        assert_eq!(entry.new_sha, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(entry.author, "testuser");
        assert_eq!(entry.timestamp, "1234567890 +0800");
        assert!(entry.message.contains("commit"));
    }

    #[test]
    fn test_append_and_read_reflog() {
        let repo = setup_repo("append_read");

        // 追加 HEAD reflog
        append_reflog(
            &repo,
            "HEAD",
            "0000000000000000000000000000000000000000",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "testuser",
            "commit: Initial commit",
        )
        .unwrap();

        append_reflog(
            &repo,
            "HEAD",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "testuser",
            "commit: Second commit",
        )
        .unwrap();

        let entries = read_reflog(&repo, "HEAD").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].old_sha,
            "0000000000000000000000000000000000000000"
        );
        assert_eq!(
            entries[1].new_sha,
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(entries[0].message, "commit: Initial commit");
        assert_eq!(entries[1].message, "commit: Second commit");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_read_nonexistent_reflog() {
        let repo = setup_repo("empty_log");
        let entries = read_reflog(&repo, "HEAD").unwrap();
        assert!(entries.is_empty());
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_append_reflog_creates_directory() {
        let repo = setup_repo("create_dir");
        // 日志目录会在 append 时自动创建
        append_reflog(
            &repo,
            "refs/heads/feature",
            "0000000000000000000000000000000000000000",
            "cccccccccccccccccccccccccccccccccccccccc",
            "testuser",
            "branch: Created from main",
        )
        .unwrap();

        let entries = read_reflog(&repo, "refs/heads/feature").unwrap();
        assert_eq!(entries.len(), 1);

        let _ = fs::remove_dir_all(&repo);
    }
}

//! `reflog` 命令 —— 查看引用日志。
//!
//! 用法：`agit reflog [ref]`
//!
//! 默认显示 HEAD 的引用日志。可以指定分支名查看特定分支的 reflog。

use crate::core::reflog::{self, ReflogEntry};
use crate::core::repo;

pub fn run(ref_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    // 解析引用名
    let log_ref = match ref_name {
        Some(name) => {
            if name == "HEAD" {
                "HEAD".to_string()
            } else if name.starts_with("refs/") {
                name.to_string()
            } else {
                // 尝试分支名
                let branch_ref = format!("refs/heads/{}", name);
                let log_path = repo_root.join(".git").join("logs").join(&branch_ref);
                if log_path.exists() {
                    branch_ref
                } else {
                    // 可能是不带 refs/ 前缀的引用，或者是 HEAD
                    let head_path = repo_root.join(".git").join("logs").join(name);
                    if head_path.exists() {
                        name.to_string()
                    } else {
                        // 默认视为 HEAD
                        "HEAD".to_string()
                    }
                }
            }
        }
        None => "HEAD".to_string(),
    };

    let entries = reflog::read_reflog(&repo_root, &log_ref)?;

    if entries.is_empty() {
        // 不是错误，只是没有日志
        return Ok(());
    }

    // 显示名称
    let display_name = if log_ref == "HEAD" {
        "HEAD".to_string()
    } else if let Some(branch) = log_ref.strip_prefix("refs/heads/") {
        branch.to_string()
    } else if let Some(tag) = log_ref.strip_prefix("refs/tags/") {
        format!("tag:{}", tag)
    } else {
        log_ref.clone()
    };

    // 按从新到旧的顺序输出（倒序遍历）
    let total = entries.len();
    for (idx, entry) in entries.iter().rev().enumerate() {
        let reflog_idx = total - 1 - idx; // 从 0 开始的索引
        print_reflog_entry(entry, &display_name, reflog_idx);
    }

    Ok(())
}

/// 格式化输出单条 reflog 记录。
fn print_reflog_entry(entry: &ReflogEntry, ref_name: &str, index: usize) {
    let short_new = &entry.new_sha[..7];

    // 颜色化输出
    let colored_sha = crate::output::colorize(short_new, "33"); // 黄色
    let colored_ref = crate::output::colorize(
        &format!("{}@{{{}}}", ref_name, index),
        "36", // 青色
    );

    println!("{} {}: {}", colored_sha, colored_ref, entry.message);

    // 显示作者、时间、变更前后 SHA
    let short_old = if entry.old_sha == "0000000000000000000000000000000000000000" {
        "0000000".to_string()
    } else {
        entry.old_sha[..7].to_string()
    };
    let detail = format!(
        "    {} → {}  author: {}  time: {}",
        short_old, short_new, entry.author, entry.timestamp
    );
    println!("  {}", crate::output::colorize(&detail, "90")); // 深灰色
}

//! `blame` 命令 —— 逐行追溯文件每行的最后修改提交。
//!
//! 用法：`agit blame [<rev>] <file>`
//!
//! 算法：
//! 1. 从指定 revision 获取文件内容
//! 2. 沿提交历史（first-parent）向前追溯
//! 3. 比较每个提交与其父提交中该文件的内容
//! 4. 将变更行归因到对应提交
//! 5. 直到所有行都被归因或到达根提交

use agit_core::objects::commit::Commit;
use agit_core::objects::format_object_data;
use agit_core::{refs, repo, storage};
use std::collections::HashMap;
use std::path::Path;

/// 按行追溯的结果。
struct BlameResult {
    /// 提交缩写（7 位 hex）
    commit_short: String,
    /// 作者名（不含 email 部分）
    author: String,
    /// 原行号（1-based），在原始提交中的行号
    orig_line: usize,
    /// 最终行号（1-based），在最终文件中的行号
    final_line: usize,
    /// 该行的内容
    content: String,
}

pub fn run(revision: Option<&str>, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    // 确定起始 revision
    let start_commit = match revision {
        Some(rev) => repo::resolve_commit(&repo_root, rev)?,
        None => refs::read_head(&repo_root)?,
    };

    // 从起始提交获取文件内容
    let file_blob_sha = find_file_in_commit(&repo_root, &start_commit, file)?;
    let initial_content = get_blob_content(&repo_root, &file_blob_sha)?;
    let initial_lines: Vec<String> = split_lines(&initial_content);

    if initial_lines.is_empty() {
        // 空文件
        return Ok(());
    }

    // 当前每行的内容 (line → content)
    let mut current_lines: Vec<String> = initial_lines.clone();
    // 每行归因状态: line_index → Option<(commit_sha, orig_line_no)>
    let mut blame_map: Vec<Option<(String, usize)>> = vec![None; current_lines.len()];

    // 遍历提交历史
    let mut commit_sha = start_commit.clone();
    let mut _commit_count = 0;

    loop {
        // 检查是否所有行都已归因
        if blame_map.iter().all(|b| b.is_some()) {
            break;
        }

        // 读取当前提交
        let (obj_type, content) = match storage::read_object(&repo_root, &commit_sha) {
            Ok(v) => v,
            Err(_) => break,
        };
        if obj_type != "commit" {
            break;
        }

        let commit_data = format_object_data("commit", &content);
        let commit = match Commit::deserialize(&commit_data) {
            Ok(c) => c,
            Err(_) => break,
        };

        _commit_count += 1;

        // 获取父提交中的文件内容（如果有父提交）
        if let Some(parent_sha) = commit.parents.first() {
            // 尝试从父提交获取文件
            if let Ok(parent_blob_sha) = find_file_in_commit(&repo_root, parent_sha, file) {
                let parent_content = get_blob_content(&repo_root, &parent_blob_sha)?;
                let parent_lines: Vec<String> = split_lines(&parent_content);

                // 比较父提交与当前提交的文件内容，归因变更行
                // 使用逐行 LCS 方法
                blame_by_diff(&commit_sha, &parent_lines, &current_lines, &mut blame_map);
            } else {
                // 文件在父提交中不存在 → 当前提交引入了该文件
                // 所有未归因的行都属于此提交
                let short = commit_sha[..7].to_string();
                for (i, entry) in blame_map.iter_mut().enumerate() {
                    if entry.is_none() {
                        *entry = Some((short.clone(), i + 1));
                    }
                }
                break;
            }
        } else {
            // 根提交：所有未归因的行都属于此提交
            let short = commit_sha[..7].to_string();
            for (i, entry) in blame_map.iter_mut().enumerate() {
                if entry.is_none() {
                    *entry = Some((short.clone(), i + 1));
                }
            }
            break;
        }

        // 如果当前提交没有改变文件内容（所有行在父提交中相同），
        // 则不归因任何行给此提交

        // 如果此提交没有变更文件，将 current_lines 更新为父提交的内容
        if let Some(parent_sha) = commit.parents.first() {
            if let Ok(parent_blob_sha) = find_file_in_commit(&repo_root, parent_sha, file) {
                let parent_content = get_blob_content(&repo_root, &parent_blob_sha)?;
                current_lines = split_lines(&parent_content);
            }
        }

        // 移动到父提交继续
        if let Some(parent_sha) = commit.parents.first().cloned() {
            commit_sha = parent_sha;
        } else {
            break;
        }
    }

    // 确保所有行都已被归因（安全网）
    let short = commit_sha[..7].to_string();
    for (i, entry) in blame_map.iter_mut().enumerate() {
        if entry.is_none() {
            *entry = Some((short.clone(), i + 1));
        }
    }

    // 收集结果：需要每个提交的作者信息
    let mut author_cache: HashMap<String, String> = HashMap::new();

    let results: Vec<BlameResult> = initial_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let (commit_short, orig_line) = blame_map[i].clone().unwrap_or_default();
            let author = author_cache
                .entry(commit_short.clone())
                .or_insert_with(|| {
                    // 通过缩写 SHA 查找完整 SHA 并读取作者
                    load_author_by_short(&repo_root, &commit_short)
                        .unwrap_or_else(|| "unknown".to_string())
                })
                .clone();

            BlameResult {
                commit_short,
                author,
                orig_line,
                final_line: i + 1,
                content: line.clone(),
            }
        })
        .collect();

    // 输出
    print_blame_results(&results);

    Ok(())
}

/// 在提交的 tree 中递归查找指定路径的文件 blob SHA。
fn find_file_in_commit(
    repo: &Path,
    commit_sha: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, commit_sha)?;
    if obj_type != "commit" {
        return Err(format!("{} is not a commit", commit_sha).into());
    }
    let commit_data = format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;

    find_in_tree(repo, &commit.tree, file_path)
}

/// 在 tree 对象中递归查找文件 blob SHA。
fn find_in_tree(
    repo: &Path,
    tree_sha: &str,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, tree_sha)?;
    if obj_type != "tree" {
        return Err(format!("{} is not a tree", tree_sha).into());
    }

    let tree_data = format_object_data("tree", &content);
    let tree = agit_core::objects::tree::Tree::deserialize(&tree_data)?;

    // 处理多级路径
    let parts: Vec<&str> = file_path.split('/').collect();
    let first = parts[0];

    for entry in &tree.entries {
        if entry.name == first {
            if parts.len() == 1 {
                // 找到目标文件
                if entry.mode == "40000" {
                    return Err(format!("'{}' is a directory, not a file", file_path).into());
                }
                return Ok(entry.sha1.clone());
            } else {
                // 进入子目录
                if entry.mode != "40000" {
                    return Err(format!(
                        "'{}' is not a directory, cannot descend into '{}'",
                        first, file_path
                    )
                    .into());
                }
                let rest = parts[1..].join("/");
                return find_in_tree(repo, &entry.sha1, &rest);
            }
        }
    }

    Err(format!("file '{}' not found in tree {}", file_path, tree_sha).into())
}

/// 获取 blob 对象的内容（UTF-8 文本）。
fn get_blob_content(repo: &Path, blob_sha: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, blob_sha)?;
    if obj_type != "blob" {
        return Err(format!("{} is not a blob", blob_sha).into());
    }
    Ok(String::from_utf8_lossy(&content).to_string())
}

/// 将文件内容按行拆分（保留末尾空行）。
fn split_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    // 如果原内容以换行结尾，保留末尾空行
    if content.ends_with('\n') {
        lines.push(String::new());
    }
    lines
}

/// 比较父提交内容和当前内容，将不同的行归因到当前提交。
///
/// 使用简化的 LCS 对比法：
/// - 在 parent_lines 中查找 current_lines 的每一行
/// - 如果某行在父提交中找不到（或内容不同），则归因到当前提交
fn blame_by_diff(
    commit_sha: &str,
    parent_lines: &[String],
    current_lines: &[String],
    blame_map: &mut [Option<(String, usize)>],
) {
    let short = commit_sha[..7].to_string();

    // 使用 LCS 对比来确定哪些行是新增/修改的
    let lcs = compute_lcs(parent_lines, current_lines);

    let mut matched_in_parent: Vec<bool> = vec![false; current_lines.len()];

    // 标记通过 LCS 匹配的行
    let mut ci = 0;
    for &pi in &lcs {
        // 在 current 中找到对应的行
        while ci < current_lines.len() && ci < matched_in_parent.len() {
            if current_lines[ci] == parent_lines[pi] && !matched_in_parent[ci] {
                matched_in_parent[ci] = true;
                ci += 1;
                break;
            }
            ci += 1;
        }
    }

    // 将未匹配的行归因到当前提交
    for (i, matched) in matched_in_parent.iter().enumerate() {
        if !matched && blame_map[i].is_none() {
            blame_map[i] = Some((short.clone(), i + 1));
        }
    }
}

/// 计算两行序列的最长公共子序列（LCS）。
///
/// 返回 LCS 中对应 parent 行的索引列表。
fn compute_lcs(a: &[String], b: &[String]) -> Vec<usize> {
    let m = a.len();
    let n = b.len();
    if m == 0 || n == 0 {
        return Vec::new();
    }

    // 动态规划表
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..m {
        for j in 0..n {
            if a[i] == b[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i][j + 1].max(dp[i + 1][j]);
            }
        }
    }

    // 回溯
    let mut result = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(i - 1); // parent 行索引
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

/// 通过缩写 SHA 查找提交的作者名。
fn load_author_by_short(repo: &Path, short_sha: &str) -> Option<String> {
    // 尝试解析为完整 SHA
    let full_sha = repo::resolve_commit(repo, short_sha).ok()?;
    let (obj_type, content) = storage::read_object(repo, &full_sha).ok()?;
    if obj_type != "commit" {
        return None;
    }
    let commit_data = format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data).ok()?;

    // 提取作者名（去掉 email 和时间戳部分）
    let author = commit.author;
    if let Some(email_start) = author.find('<') {
        Some(author[..email_start].trim().to_string())
    } else {
        // 直接按空格取第一个词
        author.split_whitespace().next().map(|s| s.to_string())
    }
}

/// 输出 blame 结果，使用类似 git blame 的格式。
fn print_blame_results(results: &[BlameResult]) {
    // 计算列宽
    let max_sha_len = results
        .iter()
        .map(|r| r.commit_short.len())
        .max()
        .unwrap_or(7)
        .max(7);
    let max_author_len = results
        .iter()
        .map(|r| r.author.len())
        .max()
        .unwrap_or(8)
        .max(8);
    let max_line_len = results
        .iter()
        .map(|r| r.final_line.to_string().len())
        .max()
        .unwrap_or(3);

    for r in results {
        let sha = crate::output::colorize(
            &format!("{:>width$}", r.commit_short, width = max_sha_len),
            "33",
        );
        let author = format!("{:<width$}", r.author, width = max_author_len);
        let line_no = format!("{:>width$}", r.final_line, width = max_line_len);
        let orig = format!("{:>width$}", r.orig_line, width = max_line_len);

        println!(
            "{} ({}) {} {} {}",
            sha,
            crate::output::colorize(&line_no, "32"),
            orig,
            author,
            r.content
        );
    }
}

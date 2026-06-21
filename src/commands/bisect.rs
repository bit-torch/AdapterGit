//! `bisect` 命令 —— 二分查找引入 bug 的提交。
//!
//! 用法：
//! ```text
//! agit bisect start [<bad> [<good>...]]
//! agit bisect good [<rev>]
//! agit bisect bad [<rev>]
//! agit bisect skip [<rev>]
//! agit bisect reset
//! agit bisect log
//! agit bisect run <cmd>...
//! ```

use crate::core::bisect::{self, BisectState};
use crate::core::checkout;
use crate::core::repo;

/// bisect 子命令类型（内部使用，与 CLI 枚举独立）。
pub enum BisectSubCmd<'a> {
    Start {
        bad: Option<&'a str>,
        good: &'a [String],
    },
    Good {
        rev: Option<&'a str>,
    },
    Bad {
        rev: Option<&'a str>,
    },
    Skip {
        rev: Option<&'a str>,
    },
    Reset,
    Log,
    Run {
        cmd: &'a [String],
    },
}

pub fn run(action: &BisectSubCmd) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    match action {
        BisectSubCmd::Start { bad, good } => bisect_start(&repo_root, *bad, good),
        BisectSubCmd::Good { rev } => bisect_good(&repo_root, *rev),
        BisectSubCmd::Bad { rev } => bisect_bad(&repo_root, *rev),
        BisectSubCmd::Skip { rev } => bisect_skip(&repo_root, *rev),
        BisectSubCmd::Reset => bisect_reset(&repo_root),
        BisectSubCmd::Log => bisect_log(&repo_root),
        BisectSubCmd::Run { cmd } => bisect_run(&repo_root, cmd),
    }
}

/// 开始二分查找。
///
/// `bisect start [<bad> [<good>...]]`
fn bisect_start(
    repo: &std::path::Path,
    bad: Option<&str>,
    good: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if BisectState::is_active(repo) {
        return Err("bisect is already in progress. Use 'agit bisect reset' first.".into());
    }

    // 保存原始 HEAD
    let original_head = crate::core::refs::read_head(repo)?;

    // 解析 bad 提交
    let bad_sha = match bad {
        Some(rev) => repo::resolve_commit(repo, rev)?,
        None => {
            // 默认使用 HEAD
            crate::core::refs::read_head(repo)?
        }
    };

    // 解析 good 提交
    let mut good_shas = Vec::new();
    for g in good {
        let sha = repo::resolve_commit(repo, g)?;
        good_shas.push(sha);
    }

    if good_shas.is_empty() {
        return Err("You must specify at least one good revision.\n\
             Usage: agit bisect start [<bad>] <good> [<good>...]"
            .into());
    }

    // 计算搜索范围
    let range = bisect::compute_range(repo, &bad_sha, &good_shas)?;
    if range.is_empty() {
        println!("The bad and good revisions are adjacent — no commits to bisect.");
        return Ok(());
    }

    // 创建初始状态
    let state = BisectState {
        bad: bad_sha.clone(),
        good: good_shas,
        skip: Vec::new(),
        original_head,
        remaining: range,
    };

    // 选择第一个待测试提交
    if let Some(next) = bisect::pick_next(&state.remaining) {
        state.save(repo)?;
        bisect::log_bisect_action(repo, "bisect start", &bad_sha)?;

        println!(
            "Bisecting: {} revisions left to test after this",
            state.remaining.len()
        );
        println!("Checking out: {}", &next[..7]);

        // 检出待测试提交
        checkout::restore_from_commit(repo, &next)?;
        write_head_detached(repo, &next)?;
    } else {
        println!("No revisions to bisect.");
    }

    Ok(())
}

/// 标记当前提交为 good。
fn bisect_good(
    repo: &std::path::Path,
    rev: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = BisectState::load(repo)?;

    let sha = match rev {
        Some(r) => repo::resolve_commit(repo, r)?,
        None => crate::core::refs::read_head(repo)?,
    };

    // 将 good SHA 加入列表
    if !state.good.contains(&sha) {
        state.good.push(sha.clone());
    }
    let _ = bisect::log_bisect_action(repo, "bisect good", &sha);

    bisect_step(repo, &mut state)
}

/// 标记当前提交为 bad。
fn bisect_bad(repo: &std::path::Path, rev: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = BisectState::load(repo)?;

    let sha = match rev {
        Some(r) => repo::resolve_commit(repo, r)?,
        None => crate::core::refs::read_head(repo)?,
    };

    // 更新 bad 为新的 bad 提交
    state.bad = sha.clone();
    let _ = bisect::log_bisect_action(repo, "bisect bad", &sha);

    bisect_step(repo, &mut state)
}

/// 跳过当前提交。
fn bisect_skip(
    repo: &std::path::Path,
    rev: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = BisectState::load(repo)?;

    let sha = match rev {
        Some(r) => repo::resolve_commit(repo, r)?,
        None => crate::core::refs::read_head(repo)?,
    };

    if !state.skip.contains(&sha) {
        state.skip.push(sha.clone());
    }
    let _ = bisect::log_bisect_action(repo, "bisect skip", &sha);

    bisect_step(repo, &mut state)
}

/// 二分查找的核心步骤：重新计算范围并检出下一个提交。
fn bisect_step(
    repo: &std::path::Path,
    state: &mut BisectState,
) -> Result<(), Box<dyn std::error::Error>> {
    // 重新计算搜索范围
    let range = bisect::compute_range(repo, &state.bad, &state.good)?;

    // 从剩余列表中移除已跳过的提交
    let filtered: Vec<String> = range
        .into_iter()
        .filter(|s| !state.skip.contains(s))
        .collect();

    if filtered.is_empty() {
        // 二分查找完成
        println!("{} is the first bad commit", &state.bad[..7]);
        println!("Bisect completed. Use 'agit bisect reset' to return to original HEAD.");
        return Ok(());
    }

    state.remaining = filtered;

    if let Some(next) = bisect::pick_next(&state.remaining) {
        state.save(repo)?;
        println!(
            "Bisecting: {} revisions left to test after this",
            state.remaining.len()
        );
        println!("Checking out: {}", &next[..7]);

        checkout::restore_from_commit(repo, &next)?;
        write_head_detached(repo, &next)?;
    }

    Ok(())
}

/// 退出二分查找，恢复原始 HEAD。
fn bisect_reset(repo: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if !BisectState::is_active(repo) {
        println!("No bisect in progress.");
        return Ok(());
    }

    let state = BisectState::load(repo)?;

    // 恢复到原始 HEAD
    if !state.original_head.is_empty() {
        if state.original_head.len() == 40
            && state.original_head.chars().all(|c| c.is_ascii_hexdigit())
        {
            // 分离 HEAD
            crate::core::refs::write_head(repo, &state.original_head)?;
        } else if let Some(branch) = state.original_head.strip_prefix("ref: refs/heads/") {
            // 符号引用
            crate::core::refs::write_head(repo, &format!("ref: refs/heads/{}", branch))?;
        } else {
            crate::core::refs::write_head(repo, &state.original_head)?;
        }

        // 恢复工作树
        let head_sha = crate::core::refs::read_head(repo)?;
        checkout::restore_from_commit(repo, &head_sha)?;
    }

    // 清除 bisect 状态
    BisectState::clear(repo)?;
    println!("Bisect reset. Returned to original HEAD.");

    Ok(())
}

/// 显示二分查找日志。
fn bisect_log(repo: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if !BisectState::is_active(repo) {
        println!("No bisect in progress.");
        return Ok(());
    }

    let log_content = bisect::read_bisect_log(repo)?;
    if log_content.is_empty() {
        println!("(empty bisect log)");
    } else {
        println!("{}", log_content.trim());
    }

    // 同时显示当前状态
    let state = BisectState::load(repo)?;
    println!();
    println!(
        "Bisect status: {} revisions remaining",
        state.remaining.len()
    );

    Ok(())
}

/// 自动执行脚本进行二分查找。
fn bisect_run(repo: &std::path::Path, cmd: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !BisectState::is_active(repo) {
        return Err("No bisect in progress. Use 'agit bisect start' first.".into());
    }

    if cmd.is_empty() {
        return Err("No test command specified for 'bisect run'.".into());
    }

    let mut state = BisectState::load(repo)?;

    loop {
        if state.remaining.is_empty() {
            println!("{} is the first bad commit", &state.bad[..7]);
            break;
        }

        if let Some(next) = bisect::pick_next(&state.remaining) {
            checkout::restore_from_commit(repo, &next)?;
            write_head_detached(repo, &next)?;

            println!("Running test on {}", &next[..7]);

            // 执行测试命令
            let status = run_test_command(cmd);

            if status == 0 {
                println!("Test passed → marking as good");
                state.good.push(next.clone());
                let _ = bisect::log_bisect_action(repo, "bisect good (run)", &next);
            } else {
                println!("Test failed → marking as bad");
                state.bad = next.clone();
                let _ = bisect::log_bisect_action(repo, "bisect bad (run)", &next);
            }

            // 重新计算范围
            let range = bisect::compute_range(repo, &state.bad, &state.good)?;
            let filtered: Vec<String> = range
                .into_iter()
                .filter(|s| !state.skip.contains(s))
                .collect();
            state.remaining = filtered;
            state.save(repo)?;
        }
    }

    Ok(())
}

/// 执行测试命令并返回退出码。
fn run_test_command(cmd: &[String]) -> i32 {
    use std::process::Command;
    if cmd.is_empty() {
        return 1;
    }
    let status = Command::new(&cmd[0]).args(&cmd[1..]).status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Failed to run test command: {}", e);
            1
        }
    }
}

/// 写入分离 HEAD 状态。
fn write_head_detached(
    repo: &std::path::Path,
    sha: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::core::refs::write_head(repo, sha)?;
    Ok(())
}

use agit_core::index::Index;
use agit_core::objects::commit::Commit;
use agit_core::{checkout, rebase as core_rebase, refs, repo, storage};
use std::fs;
use std::path::Path;

pub fn run(
    upstream: Option<&str>,
    onto: Option<&str>,
    r#continue: bool,
    skip: bool,
    abort: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::find_repo_root()?;

    if abort {
        return run_abort(&repo);
    }
    if skip {
        return run_skip(&repo);
    }
    if r#continue {
        return run_continue(&repo);
    }

    let upstream = upstream.ok_or("error: upstream is required for rebase")?;
    run_start(&repo, upstream, onto)
}

/// 启动 rebase。
fn run_start(
    repo: &Path,
    upstream: &str,
    onto: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    // 检查是否已有进行中的 rebase
    if git_dir.join("REBASE_TODO").exists() {
        return Err(
            "error: A rebase is already in progress. Use --continue, --skip, or --abort.".into(),
        );
    }

    // 脏工作树检查
    let idx = Index::load(repo)?;
    if !repo::is_working_tree_clean(repo, &idx)? {
        return Err("error: Working tree is not clean. Commit or stash your changes first.".into());
    }

    // 解析 upstream 和 onto
    let upstream_sha = repo::resolve_commit(repo, upstream)?;
    let onto_sha = match onto {
        Some(onto_spec) => repo::resolve_commit(repo, onto_spec)?,
        None => upstream_sha.clone(),
    };

    // 读取 HEAD
    let head_sha = refs::read_head(repo)?;

    // 找到 fork point
    let fork_point = agit_core::merge::find_merge_base(repo, &head_sha, &upstream_sha)?;

    // 空提交范围检查
    if fork_point == head_sha {
        println!("Current branch is up to date.");
        return Ok(());
    }

    // 收集要重放的提交
    let commits = core_rebase::collect_commits_between(repo, &fork_point, &head_sha)?;
    if commits.is_empty() {
        println!("No commits to rebase.");
        return Ok(());
    }
    println!(
        "Rebasing {} commit(s) onto {}",
        commits.len(),
        &onto_sha[..7]
    );

    // 保存 ORIG_HEAD
    fs::write(git_dir.join("ORIG_HEAD"), format!("{}\n", head_sha))?;

    // 保存原始分支名称（如果有）
    if let Some(branch) = refs::get_current_branch(repo)? {
        fs::write(git_dir.join("REBASE_APPLYING"), &branch)?;
    }

    // 写入 TODO 并分离 HEAD 到 onto
    core_rebase::write_todo(repo, &commits)?;
    refs::write_head(repo, &onto_sha)?;
    checkout::rebuild_index_from_commit(repo, &onto_sha)?;
    checkout::restore_from_commit(repo, &onto_sha)?;

    // 开始重放
    apply_todo_until_conflict_or_done(repo)
}

/// 循环 pop REBASE_TODO 并 pick，直到完成或遇到冲突。
fn apply_todo_until_conflict_or_done(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let next = core_rebase::pop_todo(repo)?;
        let commit_sha = match next {
            Some(s) => s,
            None => {
                return finish_rebase(repo);
            }
        };

        // 读取提交以获取父节点
        let (_, body) = storage::read_object(repo, &commit_sha)?;
        let commit_data = agit_core::objects::format_object_data("commit", &body);
        let commit = Commit::deserialize(&commit_data)?;
        let parent_sha = commit.parents.first().cloned();

        // 应用提交
        let pick = core_rebase::pick_commit(repo, &commit_sha, parent_sha.as_deref())?;

        match pick {
            core_rebase::PickResult::Clean(_new_sha) => {
                let msg_first_line = commit.message.lines().next().unwrap_or("");
                println!("  Applied: {} {}", &commit_sha[..7], msg_first_line);
            }
            core_rebase::PickResult::Conflict => {
                // 将当前提交保存为 REBASE_HEAD
                fs::write(
                    repo.join(".git").join("REBASE_HEAD"),
                    format!("{}\n", commit_sha),
                )?;
                let msg_first_line = commit.message.lines().next().unwrap_or("");
                println!(
                    "error: could not apply {}... {}",
                    &commit_sha[..7],
                    msg_first_line
                );
                println!("Resolve conflicts and run 'agit rebase --continue'");
                println!("Use 'agit rebase --skip' to skip this commit");
                println!("Use 'agit rebase --abort' to abort");
                return Ok(());
            }
        }
    }
}

/// rebase --continue：解决冲突后继续。
fn run_continue(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    if !git_dir.join("REBASE_TODO").exists() {
        return Err("error: No rebase in progress.".into());
    }

    let rebase_head_path = git_dir.join("REBASE_HEAD");
    if !rebase_head_path.exists() {
        return Err("error: No rebase commit to continue. No conflicts to resolve?".into());
    }

    // 读取原始提交的元数据
    let commit_sha = fs::read_to_string(&rebase_head_path)?.trim().to_string();
    let (_, body) = storage::read_object(repo, &commit_sha)?;
    let commit_data = agit_core::objects::format_object_data("commit", &body);
    let original = Commit::deserialize(&commit_data)?;

    // 从当前索引创建新 commit（保留原始 author，当前用户为 committer）
    let config = agit_core::config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let head_sha = refs::read_head(repo)?;
    let _new_sha = core_rebase::create_commit_from_index(
        repo,
        &original.author,
        &format!(
            "{} <{}> {} {}",
            config.user_name, config.user_email, timestamp, time_str
        ),
        &original.message,
        &[&head_sha],
    )?;

    // 清理 REBASE_HEAD
    fs::remove_file(&rebase_head_path)?;

    let msg_first_line = original.message.lines().next().unwrap_or("");
    println!("  Continued: {} {}", &commit_sha[..7], msg_first_line);

    // 继续处理
    apply_todo_until_conflict_or_done(repo)
}

/// rebase --skip：跳过当前 commit。
fn run_skip(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    if !git_dir.join("REBASE_TODO").exists() {
        return Err("error: No rebase in progress.".into());
    }

    let rebase_head_path = git_dir.join("REBASE_HEAD");
    if !rebase_head_path.exists() {
        return Err("error: No rebase commit to skip.".into());
    }

    // 读取跳过的提交
    let skipped_sha = fs::read_to_string(&rebase_head_path)?.trim().to_string();
    fs::remove_file(&rebase_head_path)?;

    // 将工作树和索引恢复到当前 HEAD
    let head_sha = refs::read_head(repo)?;
    checkout::rebuild_index_from_commit(repo, &head_sha)?;
    checkout::restore_from_commit(repo, &head_sha)?;

    println!("Skipped commit {}", &skipped_sha[..7]);

    // 继续处理
    apply_todo_until_conflict_or_done(repo)
}

/// rebase --abort：中止 rebase 并恢复原始状态。
fn run_abort(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    if !git_dir.join("REBASE_TODO").exists() {
        return Err("error: No rebase to abort.".into());
    }

    // 读取 ORIG_HEAD
    let orig_head_path = git_dir.join("ORIG_HEAD");
    let restore_sha = if orig_head_path.exists() {
        fs::read_to_string(&orig_head_path)?.trim().to_string()
    } else {
        refs::read_head(repo)?
    };

    // 恢复工作树和索引
    checkout::restore_from_commit(repo, &restore_sha)?;
    checkout::rebuild_index_from_commit(repo, &restore_sha)?;

    // 如果原始分支存在，重新附加 HEAD
    let applying_path = git_dir.join("REBASE_APPLYING");
    if applying_path.exists() {
        let branch_name = fs::read_to_string(&applying_path)?.trim().to_string();
        refs::write_head(repo, &format!("ref: refs/heads/{}", branch_name))?;
    } else {
        refs::write_head(repo, &restore_sha)?;
    }

    // 清理状态文件
    let _ = fs::remove_file(&applying_path);
    let _ = fs::remove_file(git_dir.join("REBASE_TODO"));
    let _ = fs::remove_file(git_dir.join("REBASE_HEAD"));
    let _ = fs::remove_file(&orig_head_path);

    println!("Rebase aborted.");
    Ok(())
}

/// 完成 rebase：更新分支引用并清理。
fn finish_rebase(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");
    let head_sha = refs::read_head(repo)?;

    // 读取我们之前所在的分支
    let applying_path = git_dir.join("REBASE_APPLYING");
    if applying_path.exists() {
        let branch_name = fs::read_to_string(&applying_path)?.trim().to_string();
        // 更新分支引用到当前 HEAD
        refs::write_ref(repo, &format!("refs/heads/{}", branch_name), &head_sha)?;
        // 重新附加 HEAD 到分支
        refs::write_head(repo, &format!("ref: refs/heads/{}", branch_name))?;
        println!("Successfully rebased and updated '{}'.", branch_name);
    } else {
        println!("Successfully rebased (detached HEAD).");
    }

    // 清理状态文件
    let _ = fs::remove_file(&applying_path);
    let _ = fs::remove_file(git_dir.join("REBASE_TODO"));
    let _ = fs::remove_file(git_dir.join("REBASE_HEAD"));
    let _ = fs::remove_file(git_dir.join("ORIG_HEAD"));

    Ok(())
}

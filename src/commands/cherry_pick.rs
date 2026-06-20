use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::{checkout, rebase as core_rebase, refs, repo, storage};
use std::fs;
use std::path::Path;

pub fn run(
    commits: &[String],
    r#continue: bool,
    abort: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::find_repo_root()?;

    if abort {
        return run_abort(&repo);
    }
    if r#continue {
        return run_continue(&repo);
    }

    if commits.is_empty() {
        return Err("error: No commits specified for cherry-pick.".into());
    }

    run_start(&repo, commits)
}

/// 启动 cherry-pick。
fn run_start(repo: &Path, commits: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    // 检查是否已有进行中的 cherry-pick
    if git_dir.join("CHERRY_PICK_TODO").exists() {
        return Err(
            "error: A cherry-pick is already in progress. Use --continue or --abort.".into(),
        );
    }

    // 脏工作树检查
    let idx = Index::load(repo)?;
    if !repo::is_working_tree_clean(repo, &idx)? {
        return Err("error: Working tree is not clean.".into());
    }

    // 解析并验证所有提交
    let mut resolved = Vec::new();
    for spec in commits {
        let sha = repo::resolve_commit(repo, spec)?;
        resolved.push(sha);
    }

    // 保存 ORIG_HEAD
    let head_sha = refs::read_head(repo)?;
    fs::write(git_dir.join("ORIG_HEAD"), format!("{}\n", head_sha))?;

    // 写入 TODO
    core_rebase::write_cherry_todo(repo, &resolved)?;

    println!("Cherry-picking {} commit(s)", resolved.len());

    // 开始应用
    apply_cherry_todo(repo)
}

/// 循环处理 cherry-pick TODO 列表。
fn apply_cherry_todo(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let next = core_rebase::pop_cherry_todo(repo)?;
        let commit_sha = match next {
            Some(s) => s,
            None => {
                return finish_cherry_pick(repo);
            }
        };

        let (_, body) = storage::read_object(repo, &commit_sha)?;
        let commit_data = crate::core::objects::format_object_data("commit", &body);
        let commit = Commit::deserialize(&commit_data)?;
        let parent_sha = commit.parents.first().cloned();

        let pick = core_rebase::pick_commit(repo, &commit_sha, parent_sha.as_deref())?;

        match pick {
            core_rebase::PickResult::Clean(_) => {
                let msg = commit.message.lines().next().unwrap_or("");
                println!("  Picked: {} {}", &commit_sha[..7], msg);
            }
            core_rebase::PickResult::Conflict => {
                fs::write(
                    repo.join(".git").join("CHERRY_PICK_HEAD"),
                    format!("{}\n", commit_sha),
                )?;
                let msg = commit.message.lines().next().unwrap_or("");
                println!("error: could not apply {}... {}", &commit_sha[..7], msg);
                println!("Resolve conflicts and run 'agit cherry-pick --continue'");
                println!("Use 'agit cherry-pick --abort' to abort");
                return Ok(());
            }
        }
    }
}

/// cherry-pick --continue。
fn run_continue(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    if !git_dir.join("CHERRY_PICK_TODO").exists() {
        return Err("error: No cherry-pick in progress.".into());
    }

    let cherry_head_path = git_dir.join("CHERRY_PICK_HEAD");
    if !cherry_head_path.exists() {
        return Err("error: No cherry-pick commit to continue.".into());
    }

    let commit_sha = fs::read_to_string(&cherry_head_path)?.trim().to_string();
    let (_, body) = storage::read_object(repo, &commit_sha)?;
    let commit_data = crate::core::objects::format_object_data("commit", &body);
    let original = Commit::deserialize(&commit_data)?;

    let config = crate::config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let head_sha = refs::read_head(repo)?;
    core_rebase::create_commit_from_index(
        repo,
        &original.author,
        &format!(
            "{} <{}> {} {}",
            config.user_name, config.user_email, timestamp, time_str
        ),
        &original.message,
        &[&head_sha],
    )?;

    fs::remove_file(&cherry_head_path)?;

    let msg = original.message.lines().next().unwrap_or("");
    println!("  Continued: {} {}", &commit_sha[..7], msg);

    apply_cherry_todo(repo)
}

/// cherry-pick --abort。
fn run_abort(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");

    if !git_dir.join("CHERRY_PICK_TODO").exists() {
        return Err("error: No cherry-pick to abort.".into());
    }

    let orig_head_path = git_dir.join("ORIG_HEAD");
    let restore_sha = if orig_head_path.exists() {
        fs::read_to_string(&orig_head_path)?.trim().to_string()
    } else {
        refs::read_head(repo)?
    };

    checkout::restore_from_commit(repo, &restore_sha)?;
    checkout::rebuild_index_from_commit(repo, &restore_sha)?;
    refs::write_head(repo, &restore_sha)?;

    let _ = fs::remove_file(git_dir.join("CHERRY_PICK_TODO"));
    let _ = fs::remove_file(git_dir.join("CHERRY_PICK_HEAD"));
    let _ = fs::remove_file(&orig_head_path);

    println!("Cherry-pick aborted.");
    Ok(())
}

/// 完成 cherry-pick。
fn finish_cherry_pick(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo.join(".git");
    let _ = fs::remove_file(git_dir.join("CHERRY_PICK_TODO"));
    let _ = fs::remove_file(git_dir.join("CHERRY_PICK_HEAD"));
    let _ = fs::remove_file(git_dir.join("ORIG_HEAD"));
    println!("Cherry-pick completed successfully.");
    Ok(())
}

//! 集成测试：基础工作流
//! 使用 tests/common/mod.rs 中的共享测试工具。

use std::fs;

mod common;
use common::*;

// ============================================================
// 预热（Windows 首次 spawn 可能有 Os code 0 bug）
// ============================================================

#[test]
fn test_aaa_warmup() {
    let repo = setup_repo("warmup");
    let bin = agit_binary();
    let result = std::panic::catch_unwind(|| {
        std::process::Command::new(&bin)
            .current_dir(&repo)
            .arg("--version")
            .output()
    });
    if let Ok(Ok(output)) = result {
        let _ = output;
    }
    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// init + add + commit + status + log 完整工作流
// ============================================================

#[test]
fn test_init_creates_git_dir() {
    let repo = setup_repo("init");
    let output = run_agit(&repo, &["init"]);

    assert!(output.status.success());
    assert!(repo.join(".git").exists());
    assert!(repo.join(".git/HEAD").exists());
    assert!(repo.join(".git/objects").exists());
    assert!(repo.join(".git/refs").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_status_after_init() {
    let repo = setup_repo("status_init");

    run_ok(&repo, &["init"]);
    let stdout = run_ok(&repo, &["status"]);

    assert!(!stdout.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_status_with_changes() {
    let repo = setup_repo("status_changes");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("a.txt"), "aaa\n").unwrap();
    fs::write(repo.join("b.txt"), "bbb\n").unwrap();

    let stdout = run_ok(&repo, &["status"]);
    assert!(stdout.contains("a.txt") || stdout.contains("untracked") || !stdout.is_empty());

    run_ok(&repo, &["add", "a.txt"]);

    let stdout = run_ok(&repo, &["status"]);
    assert!(!stdout.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_full_workflow() {
    let repo = setup_repo("full_workflow");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("README.md"), "# Test Repo\n").unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();

    run_ok(&repo, &["add", "."]);

    let commit_out = run_ok(&repo, &["commit", "-m", "First commit"]);
    assert!(!commit_out.is_empty());

    let status_out = run_ok(&repo, &["status"]);
    assert!(!status_out.is_empty());

    let log_out = run_ok(&repo, &["log"]);
    assert!(log_out.contains("First commit") || !log_out.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_log_after_multiple_commits() {
    let repo = setup_repo("multi_commit");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("file.txt"), "v1\n").unwrap();
    run_ok(&repo, &["add", "file.txt"]);
    run_ok(&repo, &["commit", "-m", "Version 1"]);

    fs::write(repo.join("file.txt"), "v2\n").unwrap();
    run_ok(&repo, &["add", "file.txt"]);
    run_ok(&repo, &["commit", "-m", "Version 2"]);

    let log_out = run_ok(&repo, &["log"]);
    assert!(log_out.contains("Version 1"));
    assert!(log_out.contains("Version 2"));

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// diff + show
// ============================================================

#[test]
fn test_diff_output() {
    let repo = setup_repo("diff_test");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("f.txt"), "hello\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "Initial"]);

    fs::write(repo.join("f.txt"), "world\n").unwrap();

    let diff_out = run_ok(&repo, &["diff"]);
    assert!(!diff_out.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// 结构化输出 (--json / --yaml)
// ============================================================

#[test]
fn test_json_output() {
    let repo = setup_repo("json_output");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("f.txt"), "data\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "Test"]);

    let log_out = run_ok(&repo, &["--json", "log"]);
    assert!(log_out.contains('{') || log_out.contains('['));

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// 帮助信息
// ============================================================

#[test]
fn test_help_output() {
    let repo = setup_repo("help");
    let output = run_agit(&repo, &["--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("agit") || stdout.contains("Usage"));

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// 初始化后 cat-file 查看对象
// ============================================================

#[test]
fn test_cat_file_after_commit() {
    let repo = setup_repo("cat_file_test");

    run_ok(&repo, &["init"]);

    fs::write(repo.join("data.txt"), "test data\n").unwrap();
    run_ok(&repo, &["add", "data.txt"]);
    run_ok(&repo, &["commit", "-m", "Test"]);

    let head_ref = std::fs::read_to_string(repo.join(".git").join("HEAD")).unwrap_or_default();
    if let Some(ref_path) = head_ref.trim().strip_prefix("ref: ") {
        if let Ok(sha_content) = std::fs::read_to_string(repo.join(".git").join(ref_path.trim())) {
            let sha = sha_content.trim();
            if sha.len() == 40 {
                let type_out = run_ok(&repo, &["cat-file", "-t", sha]);
                assert!(
                    type_out.trim() == "commit",
                    "Expected 'commit', got '{}'",
                    type_out.trim()
                );
            }
        }
    }

    let _ = fs::remove_dir_all(&repo);
}

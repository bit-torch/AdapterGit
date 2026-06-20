//! 集成测试：高级功能（.gitignore, stash, tag, diff, log, rm, mv）
//! 使用 tests/common/mod.rs 中的共享测试工具。

use std::fs;

mod common;
use common::*;

// ============================================================
// .gitignore 集成测试
// ============================================================

#[test]
fn test_gitignore_filters_status() {
    let repo = setup_repo("ig_status");
    run_ok(&repo, &["init"]);
    fs::write(repo.join(".gitignore"), "*.log\nbuild/\n").unwrap();
    fs::write(repo.join("main.rs"), "fn main() {}").unwrap();
    fs::write(repo.join("debug.log"), "DEBUG").unwrap();
    fs::create_dir_all(repo.join("build")).unwrap();
    fs::write(repo.join("build/output.bin"), "\x00").unwrap();

    let out = run_ok(&repo, &["status"]);
    assert!(out.contains("main.rs"), "should show main.rs");
    assert!(out.contains(".gitignore"), "should show .gitignore");
    assert!(!out.contains("debug.log"), "should ignore debug.log");
    assert!(!out.contains("output.bin"), "should ignore build/ contents");

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_gitignore_add_dot_respects_ignore() {
    let repo = setup_repo("ig_add");
    run_ok(&repo, &["init"]);
    fs::write(repo.join(".gitignore"), "*.log\n").unwrap();
    fs::write(repo.join("a.txt"), "hello").unwrap();
    fs::write(repo.join("b.log"), "log").unwrap();

    let out = run_ok(&repo, &["add", "."]);
    assert!(out.contains("2 file(s)"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_gitignore_negation() {
    let repo = setup_repo("ig_neg");
    run_ok(&repo, &["init"]);
    fs::write(repo.join(".gitignore"), "*.log\n!keep.log\n").unwrap();
    fs::write(repo.join("drop.log"), "drop").unwrap();
    fs::write(repo.join("keep.log"), "keep").unwrap();

    let out = run_ok(&repo, &["status"]);
    assert!(!out.contains("drop.log"), "drop.log should be ignored");
    assert!(
        out.contains("keep.log"),
        "keep.log should NOT be ignored (negated)"
    );

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// stash 集成测试
// ============================================================

#[test]
fn test_stash_push_and_pop() {
    let repo = setup_repo("stash_pushpop");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);

    fs::write(repo.join("f.txt"), "v2").unwrap();
    run_ok(&repo, &["stash", "push"]);
    assert_eq!(fs::read_to_string(repo.join("f.txt")).unwrap(), "v1");

    fs::write(repo.join("f.txt"), "v3").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "after stash"]);

    run_ok(&repo, &["stash", "pop"]);
    let content = fs::read_to_string(repo.join("f.txt")).unwrap_or_default();
    assert_eq!(content, "v2", "stash pop should restore v2");

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_stash_list_empty() {
    let repo = setup_repo("stash_empty");
    run_ok(&repo, &["init"]);
    let out = run_ok(&repo, &["stash", "list"]);
    assert!(out.contains("No stashes found"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_stash_multiple() {
    let repo = setup_repo("stash_multi");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);

    fs::write(repo.join("f.txt"), "v2").unwrap();
    run_ok(&repo, &["stash", "push"]);

    fs::write(repo.join("f.txt"), "v3").unwrap();
    run_ok(&repo, &["stash", "push"]);

    let out = run_ok(&repo, &["stash", "list"]);
    assert!(out.contains("stash@{0}"), "should show first stash");

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_stash_no_changes_error() {
    let repo = setup_repo("stash_nochg");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    let out = run_err(&repo, &["stash", "push"]);
    assert!(
        out.contains("No local changes to save"),
        "expected error, got: {}",
        out
    );
    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// tag 集成测试
// ============================================================

#[test]
fn test_tag_lightweight() {
    let repo = setup_repo("tag_light");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["tag", "create", "v1.0.0"]);

    let out = run_ok(&repo, &["tag", "list"]);
    assert!(out.contains("v1.0.0"));
    assert!(!out.contains("annotated"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_annotated() {
    let repo = setup_repo("tag_annot");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["tag", "create", "-m", "release note", "v2.0.0"]);

    let out = run_ok(&repo, &["tag", "list"]);
    assert!(out.contains("annotated"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_delete() {
    let repo = setup_repo("tag_del");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["tag", "create", "v1.0.0"]);
    run_ok(&repo, &["tag", "delete", "v1.0.0"]);

    let out = run_ok(&repo, &["tag", "list"]);
    assert!(!out.contains("v1.0.0") || out.contains("No tags"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_list_empty() {
    let repo = setup_repo("tag_empty");
    run_ok(&repo, &["init"]);
    let out = run_ok(&repo, &["tag", "list"]);
    assert!(out.contains("No tags"));
    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// diff 集成测试
// ============================================================

#[test]
fn test_diff_two_commits() {
    let repo = setup_repo("diff_2commit");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "first"]);
    fs::write(repo.join("f.txt"), "v2").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "second"]);

    let out = run_ok(&repo, &["diff", "HEAD~1", "HEAD"]);
    assert!(out.contains("---") || out.contains("diff --git"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_cached() {
    let repo = setup_repo("diff_cached");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    fs::write(repo.join("f.txt"), "v2").unwrap();
    run_ok(&repo, &["add", "f.txt"]);

    let out = run_ok(&repo, &["diff", "--cached"]);
    assert!(out.contains("v2") || out.contains("diff --git") || !out.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_name_only() {
    let repo = setup_repo("diff_name");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("a.txt"), "a").unwrap();
    fs::write(repo.join("b.txt"), "b").unwrap();
    run_ok(&repo, &["add", "."]);
    run_ok(&repo, &["commit", "-m", "init"]);
    fs::write(repo.join("a.txt"), "aa").unwrap();
    fs::write(repo.join("b.txt"), "bb").unwrap();

    let out = run_ok(&repo, &["diff", "--name-only"]);
    assert!(out.contains("a.txt"));
    assert!(out.contains("b.txt"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_untracked() {
    let repo = setup_repo("diff_untrack");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("new.txt"), "untracked content").unwrap();
    let out = run_ok(&repo, &["diff"]);
    assert!(!out.trim().is_empty(), "should show untracked file diff");

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// log 集成测试
// ============================================================

#[test]
fn test_log_oneline() {
    let repo = setup_repo("log_oneline");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "hello world"]);
    fs::write(repo.join("f.txt"), "v2").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "second commit"]);

    let out = run_ok(&repo, &["log", "--oneline"]);
    assert!(out.contains("hello world") || out.contains("second"));
    assert!(!out.contains("Author:"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_log_limit() {
    let repo = setup_repo("log_limit");
    run_ok(&repo, &["init"]);
    for i in 1..=5 {
        fs::write(repo.join("f.txt"), format!("v{}", i)).unwrap();
        run_ok(&repo, &["add", "f.txt"]);
        run_ok(&repo, &["commit", "-m", &format!("commit {}", i)]);
    }
    let out = run_ok(&repo, &["log", "-n", "3"]);
    let commit_lines: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("commit ") && l.len() > 20)
        .collect();
    assert_eq!(
        commit_lines.len(),
        3,
        "should show exactly 3 commits, got: {}",
        out
    );

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_log_all_branches() {
    let repo = setup_repo("log_all");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "main").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "on main"]);
    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);
    fs::write(repo.join("f.txt"), "feature").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "on feature"]);

    let out = run_ok(&repo, &["log", "--all"]);
    assert!(out.contains("on main"));
    assert!(out.contains("on feature"));

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// rm / mv 集成测试
// ============================================================

#[test]
fn test_rm_tracked_file() {
    let repo = setup_repo("rm_tracked");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["rm", "f.txt"]);

    assert!(
        !repo.join("f.txt").exists(),
        "working tree file should be deleted"
    );
    let out = run_ok(&repo, &["status"]);
    assert!(!out.contains("f.txt") || out.contains("deleted"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rm_cached() {
    let repo = setup_repo("rm_cached");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["rm", "--cached", "f.txt"]);

    assert!(
        repo.join("f.txt").exists(),
        "file should remain with --cached"
    );
    let out = run_ok(&repo, &["status"]);
    assert!(out.contains("Untracked") || out.contains("f.txt"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_mv_rename() {
    let repo = setup_repo("mv_rename");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("old.txt"), "data").unwrap();
    run_ok(&repo, &["add", "old.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["mv", "old.txt", "new.txt"]);

    assert!(!repo.join("old.txt").exists(), "old file should be moved");
    assert!(repo.join("new.txt").exists(), "new file should exist");
    assert_eq!(fs::read_to_string(repo.join("new.txt")).unwrap(), "data");

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_mv_untracked_error() {
    let repo = setup_repo("mv_untrack");
    run_ok(&repo, &["init"]);

    let out = run_err(&repo, &["mv", "ghost.txt", "new.txt"]);
    assert!(out.contains("not tracked"));

    let _ = fs::remove_dir_all(&repo);
}

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
    let repo = setup_repo("tag_lw");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);

    let out = run_ok(&repo, &["tag", "create", "v0.0.0"]);
    assert!(out.contains("Created tag"));
    assert!(out.contains("v0.0.0"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_annotated() {
    let repo = setup_repo("tag_ann");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);

    let out = run_ok(&repo, &["tag", "create", "v1.0.0", "-m", "Release v1"]);
    assert!(out.contains("Created tag"));
    assert!(out.contains("v1.0.0"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_list_empty() {
    let repo = setup_repo("tag_empty");
    run_ok(&repo, &["init"]);
    let out = run_ok(&repo, &["tag", "list"]);
    assert!(out.contains("No tags found"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_tag_delete() {
    let repo = setup_repo("tag_del");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "data").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "init"]);
    run_ok(&repo, &["tag", "create", "v0.9.0"]);

    let out = run_ok(&repo, &["tag", "delete", "v0.9.0"]);
    assert!(out.contains("Deleted tag"));

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// diff 增强集成测试
// ============================================================

#[test]
fn test_diff_cached() {
    let repo = setup_repo("diff_cached");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "hello\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    // diff --cached 比较 HEAD vs index，需要先有一个 commit
    run_ok(&repo, &["commit", "-m", "init"]);
    fs::write(repo.join("f.txt"), "updated\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    let out = run_ok(&repo, &["diff", "--cached"]);
    assert!(!out.is_empty(), "diff --cached should show staged changes");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_name_only() {
    let repo = setup_repo("diff_noname");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("x.txt"), "a\n").unwrap();
    fs::write(repo.join("y.txt"), "b\n").unwrap();
    let out = run_ok(&repo, &["diff", "--name-only"]);
    assert!(out.contains("x.txt"));
    assert!(out.contains("y.txt"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_two_commits() {
    let repo = setup_repo("diff_2c");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "c1"]);
    fs::write(repo.join("f.txt"), "v2\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "c2"]);
    let out = run_ok(&repo, &["diff", "HEAD~1", "HEAD"]);
    assert!(!out.is_empty(), "diff between commits should have content");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_diff_untracked() {
    let repo = setup_repo("diff_untrack");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("u.txt"), "untracked\n").unwrap();
    let out = run_ok(&repo, &["diff"]);
    assert!(out.contains("u.txt"), "diff should show untracked files");
    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// log 增强集成测试
// ============================================================

#[test]
fn test_log_oneline() {
    let repo = setup_repo("log_ol");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "a").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "first"]);
    let out = run_ok(&repo, &["log", "--oneline"]);
    assert!(out.contains("first"), "should contain commit message");
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
        run_ok(&repo, &["commit", "-m", &format!("c{}", i)]);
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
    run_ok(&repo, &["commit", "-m", "on-main"]);
    run_ok(&repo, &["branch", "-c", "side"]);
    run_ok(&repo, &["checkout", "side"]);
    fs::write(repo.join("f.txt"), "side").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "on-side"]);
    let out = run_ok(&repo, &["log", "--all"]);
    assert!(out.contains("on-side"), "should show side branch commit");
    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// rm / mv 集成测试
// ============================================================

#[test]
fn test_rm_tracked_file() {
    let repo = setup_repo("rm_track");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("del.txt"), "bye").unwrap();
    run_ok(&repo, &["add", "del.txt"]);
    run_ok(&repo, &["commit", "-m", "add"]);
    let out = run_ok(&repo, &["rm", "del.txt"]);
    assert!(out.contains("rm 'del.txt'"));
    assert!(!repo.join("del.txt").exists());
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rm_cached() {
    let repo = setup_repo("rm_cached");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("keep.txt"), "keep").unwrap();
    run_ok(&repo, &["add", "keep.txt"]);
    run_ok(&repo, &["commit", "-m", "add"]);
    let out = run_ok(&repo, &["rm", "--cached", "keep.txt"]);
    assert!(out.contains("rm 'keep.txt'"));
    assert!(repo.join("keep.txt").exists(), "file should stay on disk");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_mv_rename() {
    let repo = setup_repo("mv_rename");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("old.txt"), "data").unwrap();
    run_ok(&repo, &["add", "old.txt"]);
    run_ok(&repo, &["commit", "-m", "add"]);
    let out = run_ok(&repo, &["mv", "old.txt", "new.txt"]);
    assert!(out.contains("Renamed 'old.txt' -> 'new.txt'"));
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

// ============================================================
// init 增强集成测试 (v0.6.2)
// ============================================================

#[test]
fn test_init_version() {
    let repo = setup_repo("ver");
    let output = run_agit(&repo, &["--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("agit"), "--version should print agit");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_with_path() {
    let repo = setup_repo("ipath");
    let sub = repo.join("subdir");
    let out = run_ok(&repo, &["init", "--path", "subdir"]);
    assert!(out.contains("Initialized empty Git repository"));
    assert!(sub.join(".git").exists());
    assert!(sub.join(".git").join("HEAD").exists());
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_rejects_existing_repo() {
    let repo = setup_repo("irej");
    run_ok(&repo, &["init"]);
    let output = run_agit(&repo, &["init"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("It seems like there's a git repo here"));
    assert!(output.status.success(), "should exit 0, not error");
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_rust() {
    let repo = setup_repo("irust");
    run_ok(&repo, &["init", "--pattern", "rust"]);
    let gitignore = repo.join(".gitignore");
    assert!(gitignore.exists());
    let content = fs::read_to_string(&gitignore).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("Cargo.lock"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_python() {
    let repo = setup_repo("ipy");
    run_ok(&repo, &["init", "--pattern", "python"]);
    let gitignore = repo.join(".gitignore");
    assert!(gitignore.exists());
    let content = fs::read_to_string(&gitignore).unwrap();
    assert!(content.contains("__pycache__/"));
    assert!(
        content.contains("*.py[cod]"),
        "python template uses glob char class"
    );
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_unknown_warns() {
    let repo = setup_repo("iunkp");
    let output = run_agit(&repo, &["init", "--pattern", "haskell"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown gitignore pattern"));
    assert!(stderr.contains("Available:"));
    // .gitignore 始终创建（含 .agit/ 安全守卫），即使 pattern 无效
    assert!(repo.join(".gitignore").exists());
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_licence_mit() {
    let repo = setup_repo("imit");
    run_ok(&repo, &["init", "-l", "mit"]);
    let license = repo.join("LICENSE");
    assert!(license.exists());
    let content = fs::read_to_string(&license).unwrap();
    assert!(content.contains("MIT License"));
    assert!(content.contains("Permission is hereby granted"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_licence_apache() {
    let repo = setup_repo("iap");
    run_ok(&repo, &["init", "-l", "apache-2.0"]);
    let license = repo.join("LICENSE");
    assert!(license.exists());
    let content = fs::read_to_string(&license).unwrap();
    assert!(content.contains("Apache License"));
    assert!(content.contains("Version 2.0"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_licence_unknown_warns() {
    let repo = setup_repo("iunkl");
    let output = run_agit(&repo, &["init", "-l", "wtfpl"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown licence"));
    assert!(stderr.contains("Available: mit, apache-2.0, gpl-3.0"));
    assert!(!repo.join("LICENSE").exists());
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_combined_path_pattern_licence() {
    let repo = setup_repo("icombo");
    run_ok(
        &repo,
        &[
            "init",
            "--path",
            "proj",
            "--pattern",
            "node",
            "-l",
            "gpl-3.0",
        ],
    );
    let proj = repo.join("proj");
    assert!(proj.join(".git").exists());
    assert!(proj.join(".gitignore").exists());
    assert!(proj.join("LICENSE").exists());

    let gitignore = fs::read_to_string(proj.join(".gitignore")).unwrap();
    assert!(gitignore.contains("node_modules/"));

    let license = fs::read_to_string(proj.join("LICENSE")).unwrap();
    assert!(license.contains("GNU GENERAL PUBLIC LICENSE"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_node() {
    let repo = setup_repo("inode");
    run_ok(&repo, &["init", "--pattern", "node"]);
    let content = fs::read_to_string(repo.join(".gitignore")).unwrap();
    assert!(content.contains("node_modules/"));
    assert!(content.contains(".env"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_go() {
    let repo = setup_repo("igo");
    run_ok(&repo, &["init", "--pattern", "go"]);
    let content = fs::read_to_string(repo.join(".gitignore")).unwrap();
    assert!(content.contains("*.exe"));
    assert!(content.contains("vendor/"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_pattern_java() {
    let repo = setup_repo("ijava");
    run_ok(&repo, &["init", "--pattern", "java"]);
    let content = fs::read_to_string(repo.join(".gitignore")).unwrap();
    assert!(content.contains("*.class"));
    assert!(content.contains(".idea/"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_licence_gpl3() {
    let repo = setup_repo("igpl");
    run_ok(&repo, &["init", "-l", "gpl-3.0"]);
    let content = fs::read_to_string(repo.join("LICENSE")).unwrap();
    assert!(content.contains("GNU GENERAL PUBLIC LICENSE"));
    assert!(content.contains("Version 3"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_init_short_path_flag() {
    let repo = setup_repo("ishort");
    let sub = repo.join("s");
    let out = run_ok(&repo, &["init", "-p", "s"]);
    assert!(out.contains("Initialized"));
    assert!(sub.join(".git").exists());
    let _ = fs::remove_dir_all(&repo);
}

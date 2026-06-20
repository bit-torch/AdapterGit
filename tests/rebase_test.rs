//! 集成测试：rebase 命令
//! 使用 tests/common/mod.rs 中的共享测试工具。

use std::fs;

mod common;
use common::*;

// ── 辅助函数 ──────────────────────────────────────────────

/// 创建一个包含 base commit 的仓库，返回 repo 路径。
fn init_with_commit(name: &str, file: &str, content: &str, msg: &str) -> std::path::PathBuf {
    let repo = setup_repo(name);
    run_ok(&repo, &["init"]);
    fs::write(repo.join(file), content).unwrap();
    run_ok(&repo, &["add", file]);
    run_ok(&repo, &["commit", "-m", msg]);
    repo
}

/// 在指定仓库中添加一个 commit（修改已有文件）。
fn add_commit(repo: &std::path::PathBuf, file: &str, content: &str, msg: &str) {
    fs::write(repo.join(file), content).unwrap();
    run_ok(repo, &["add", file]);
    run_ok(repo, &["commit", "-m", msg]);
}

/// 新建一个 commit（新增文件）。
fn add_new_file_commit(repo: &std::path::PathBuf, file: &str, content: &str, msg: &str) {
    fs::write(repo.join(file), content).unwrap();
    run_ok(repo, &["add", file]);
    run_ok(repo, &["commit", "-m", msg]);
}

// ── 测试用例 ──────────────────────────────────────────────

#[test]
fn test_rebase_simple() {
    // 场景：A → 创建 feature → main 添加 B,C(main.txt) → feature 修改 D,E(f.txt)
    // rebase feature onto main 后应为 A-B-C-D'-E'，且 main.txt 存在
    let repo = init_with_commit("rb_simple", "f.txt", "A\n", "A");

    // 从 A 创建 feature 分支
    run_ok(&repo, &["branch", "-c", "feature"]);

    // main 上在另一个文件添加 B, C
    add_new_file_commit(&repo, "main.txt", "B\n", "B");
    add_commit(&repo, "main.txt", "C\n", "C");

    // 切换到 feature，修改 f.txt 加 D, E
    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "D\n", "D");
    add_commit(&repo, "f.txt", "E\n", "E");

    // rebase feature onto main
    run_ok(&repo, &["rebase", "main"]);

    // 验证：feature 现在是线性历史 A-B-C-D'-E'
    assert_eq!(fs::read_to_string(repo.join("f.txt")).unwrap(), "E\n");
    assert!(
        repo.join("main.txt").exists(),
        "main.txt should exist after rebase"
    );

    let log = run_ok(&repo, &["log", "--oneline"]);
    assert!(log.contains("E"), "log should contain E: {}", log);
    assert!(log.contains("D"), "log should contain D: {}", log);
    assert!(log.contains("C"), "log should contain C: {}", log);
    assert!(log.contains("B"), "log should contain B: {}", log);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_onto() {
    // 场景：A-B(main) → feature 从 B 分出 C-D
    // 再建 basepoint 分支从 main 分出，加 E
    // rebase --onto basepoint main feature → feature 变成 A-B-E-C'-D'
    let repo = init_with_commit("rb_onto", "f.txt", "A\n", "A");
    add_commit(&repo, "f.txt", "B\n", "B");

    // feature 从 B 分出
    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "C\n", "C");
    add_commit(&repo, "f.txt", "D\n", "D");

    // basepoint 也从 B 分出，加 E
    run_ok(&repo, &["checkout", "main"]);
    run_ok(&repo, &["branch", "-c", "basepoint"]);
    run_ok(&repo, &["checkout", "basepoint"]);
    add_new_file_commit(&repo, "g.txt", "E\n", "E");

    // rebase --onto basepoint main feature
    run_ok(&repo, &["checkout", "feature"]);
    run_ok(&repo, &["rebase", "--onto", "basepoint", "main"]);

    // feature 应有 A-B-E-C'-D'
    assert!(repo.join("g.txt").exists(), "E's file should exist");
    let log = run_ok(&repo, &["log", "--oneline"]);
    assert!(log.contains("D"), "should contain D: {}", log);
    assert!(log.contains("C"), "should contain C: {}", log);
    assert!(log.contains("E"), "should contain E: {}", log);

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_up_to_date() {
    // feature 没有新 commit → "up to date"
    let repo = init_with_commit("rb_uptodate", "f.txt", "v1\n", "init");
    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);

    // feature 没有额外 commit
    let out = run_ok(&repo, &["rebase", "main"]);
    assert!(out.contains("up to date"));

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_conflict() {
    // main 修改 f.txt → "main", feature 修改 f.txt → "feature"
    let repo = init_with_commit("rb_conflict", "f.txt", "base\n", "base");

    run_ok(&repo, &["branch", "-c", "feature"]);

    // main 上的修改
    add_commit(&repo, "f.txt", "main change\n", "on main");

    // feature 上的修改
    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "feature change\n", "on feature");

    // rebase → 冲突
    let out = run_ok(&repo, &["rebase", "main"]);
    assert!(out.contains("could not apply") || out.contains("conflict"));

    // 验证冲突标记存在
    let content = fs::read_to_string(repo.join("f.txt")).unwrap();
    assert!(content.contains("<<<<<<< HEAD"));
    assert!(content.contains("======="));
    assert!(content.contains(">>>>>>>"));

    // 验证状态文件
    assert!(repo.join(".git/REBASE_TODO").exists());
    assert!(repo.join(".git/REBASE_HEAD").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_continue_after_conflict() {
    let repo = init_with_commit("rb_cont", "f.txt", "base\n", "base");
    run_ok(&repo, &["branch", "-c", "feature"]);
    add_commit(&repo, "f.txt", "main change\n", "on main");
    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "feature change\n", "on feature");

    // 触发冲突
    let out = run_ok(&repo, &["rebase", "main"]);
    assert!(out.contains("could not apply") || out.contains("conflict"));

    // 解决冲突
    fs::write(repo.join("f.txt"), "resolved\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);

    // continue
    let out = run_ok(&repo, &["rebase", "--continue"]);
    assert!(out.contains("Successfully rebased") || out.contains("Continued"));

    // 验证最终内容
    assert_eq!(
        fs::read_to_string(repo.join("f.txt")).unwrap(),
        "resolved\n"
    );

    // 状态文件已清理
    assert!(!repo.join(".git/REBASE_TODO").exists());
    assert!(!repo.join(".git/REBASE_HEAD").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_skip() {
    let repo = init_with_commit("rb_skip", "f.txt", "base\n", "base");
    run_ok(&repo, &["branch", "-c", "feature"]);
    add_commit(&repo, "f.txt", "main change\n", "M");

    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "feature A\n", "F1");
    add_new_file_commit(&repo, "g.txt", "new file\n", "F2");

    // 触发冲突
    let out = run_ok(&repo, &["rebase", "main"]);
    assert!(out.contains("could not apply"));

    // skip
    let out = run_ok(&repo, &["rebase", "--skip"]);
    assert!(out.contains("Skipped commit") || out.contains("skipped"));

    // F2 应该被应用（新文件 g.txt 应存在）
    assert!(repo.join("g.txt").exists());

    // 状态文件已清理
    assert!(!repo.join(".git/REBASE_TODO").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_abort() {
    let repo = init_with_commit("rb_abort", "f.txt", "v1\n", "C1");

    // 记录原始状态（用于最终验证）
    let _orig_content = fs::read_to_string(repo.join("f.txt")).unwrap();

    run_ok(&repo, &["branch", "-c", "feature"]);
    add_commit(&repo, "f.txt", "v2\n", "C2 on main");
    run_ok(&repo, &["checkout", "feature"]);
    add_commit(&repo, "f.txt", "v3\n", "C3 on feature");

    // 触发冲突
    let out = run_ok(&repo, &["rebase", "main"]);
    assert!(out.contains("could not apply"));

    // abort
    let out = run_ok(&repo, &["rebase", "--abort"]);
    assert!(out.contains("aborted"));

    // 验证恢复到原始状态
    assert_eq!(fs::read_to_string(repo.join("f.txt")).unwrap(), "v3\n");

    // 状态文件已清理
    assert!(!repo.join(".git/REBASE_TODO").exists());
    assert!(!repo.join(".git/REBASE_HEAD").exists());
    assert!(!repo.join(".git/ORIG_HEAD").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_rebase_dirty_working_tree() {
    let repo = init_with_commit("rb_dirty", "f.txt", "clean\n", "init");
    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);

    // 修改文件但不提交
    fs::write(repo.join("f.txt"), "dirty\n").unwrap();

    let out = run_err(&repo, &["rebase", "main"]);
    assert!(out.contains("not clean"));

    let _ = fs::remove_dir_all(&repo);
}

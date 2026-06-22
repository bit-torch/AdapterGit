//! Git 一致性测试：对比 agit 与原生 Git 的行为。
//!
//! 这些测试在已安装原生 Git 的环境中运行（CI 的 ubuntu/macos/windows runner 均预装）。
//! 本地无 Git 时自动跳过。
//!
//! 比较策略：**语义级**对比，不要求字节级输出一致。
//! 检查点包括：仓库结构、文件系统状态、退出码、关键输出子串。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::{run_agit, setup_repo};

// ── 工具函数 ─────────────────────────────────────────────────

fn run_git(repo: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@agit.local")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@agit.local")
        .output()
        .expect("Failed to run git")
}

fn run_git_ok(repo: &PathBuf, args: &[&str]) -> String {
    let output = run_git(repo, args);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args, stdout, stderr
        );
    }
    stdout
}

fn run_agit_ok(repo: &PathBuf, args: &[&str]) -> String {
    let output = run_agit(repo, args);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "agit {:?} failed:\nstdout: {}\nstderr: {}",
            args, stdout, stderr
        );
    }
    stdout
}

/// 检查原生 Git 是否可用（不可用时跳过测试）
fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 检查 .git 目录结构是否基本完整
fn assert_git_dir_structure(repo: &Path) {
    let git_dir = repo.join(".git");
    assert!(git_dir.exists(), ".git 目录应存在");
    assert!(git_dir.join("HEAD").exists(), "HEAD 应存在");
    assert!(git_dir.join("config").exists(), "config 应存在");
    assert!(git_dir.join("objects").is_dir(), "objects/ 应存在");
    assert!(git_dir.join("refs").is_dir(), "refs/ 应存在");
}

// ═══════════════════════════════════════════════════════════════
// 测试用例
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_compat_init_creates_git_dir() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    let repo = setup_repo("compat_init");

    // agit init
    run_agit_ok(&repo, &["init"]);
    assert_git_dir_structure(&repo);
    let _ = fs::remove_dir_all(&repo);

    // git init (新目录)
    let git_repo = setup_repo("compat_init_git");
    run_git_ok(&git_repo, &["init"]);
    assert_git_dir_structure(&git_repo);
    let _ = fs::remove_dir_all(&git_repo);
}

#[test]
fn test_compat_add_and_commit() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit 工作流 ──
    let repo_a = setup_repo("compat_ac_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("hello.txt"), "hello world\n").unwrap();
    let out = run_agit_ok(&repo_a, &["add", "hello.txt"]);
    assert!(!out.is_empty(), "add 应有输出");
    let out = run_agit_ok(&repo_a, &["commit", "-m", "initial commit"]);
    assert!(
        out.contains("initial commit") || out.contains("(root-commit)"),
        "commit 应成功: {}",
        out
    );

    // agit: 3 个对象 (blob + tree + commit)
    let objects_a = fs::read_dir(repo_a.join(".git").join("objects"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();
    assert!(objects_a >= 2, "至少应有 2 个对象目录 (2 字母前缀)");

    let _ = fs::remove_dir_all(&repo_a);

    // ── git 工作流 ──
    let repo_g = setup_repo("compat_ac_g");
    run_git_ok(&repo_g, &["init"]);
    fs::write(repo_g.join("hello.txt"), "hello world\n").unwrap();
    run_git_ok(&repo_g, &["add", "hello.txt"]);
    let out = run_git_ok(&repo_g, &["commit", "-m", "initial commit"]);
    assert!(out.contains("initial commit"), "git commit 应输出消息");

    let objects_g = fs::read_dir(repo_g.join(".git").join("objects"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count();
    assert!(objects_g >= 2, "git 至少应有 2 个对象目录");

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_status_clean_and_dirty() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_status_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("f.txt"), "data\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "init"]);

    let out_clean = run_agit_ok(&repo_a, &["status"]);
    assert!(
        out_clean.contains("nothing to commit") || out_clean.contains("clean"),
        "agit: clean 状态应提示无更改: {}",
        out_clean
    );

    fs::write(repo_a.join("f.txt"), "modified\n").unwrap();
    let out_dirty = run_agit_ok(&repo_a, &["status"]);
    assert!(
        out_dirty.contains("modified") || out_dirty.contains("f.txt"),
        "agit: 修改后 status 应显示变更: {}",
        out_dirty
    );

    let _ = fs::remove_dir_all(&repo_a);

    // ── git ──
    let repo_g = setup_repo("compat_status_g");
    run_git_ok(&repo_g, &["init"]);
    fs::write(repo_g.join("f.txt"), "data\n").unwrap();
    run_git_ok(&repo_g, &["add", "f.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "init"]);

    let out_clean = run_git_ok(&repo_g, &["status", "--porcelain"]);
    assert!(
        out_clean.trim().is_empty(),
        "git: clean 状态 --porcelain 应为空: {}",
        out_clean
    );

    fs::write(repo_g.join("f.txt"), "modified\n").unwrap();
    let out_dirty = run_git_ok(&repo_g, &["status", "--porcelain"]);
    assert!(
        !out_dirty.trim().is_empty(),
        "git: 修改后 --porcelain 应有输出"
    );

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_branch_and_checkout() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_br_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("f.txt"), "main\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "on main"]);

    // 创建分支
    let out = run_agit_ok(&repo_a, &["branch", "-c", "dev"]);
    assert!(out.contains("Created branch"), "branch -c 应成功: {}", out);

    // 分支列表
    let out = run_agit_ok(&repo_a, &["branch", "--list"]);
    assert!(
        out.contains("main") && out.contains("dev"),
        "应列出两个分支: {}",
        out
    );

    // 切换分支
    let out = run_agit_ok(&repo_a, &["checkout", "dev"]);
    assert!(out.contains("dev"), "checkout 应切换到 dev: {}", out);

    // 在新分支上提交
    fs::write(repo_a.join("f.txt"), "dev\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "on dev"]);

    // 切回 main 验证文件内容
    run_agit_ok(&repo_a, &["checkout", "main"]);
    let content = fs::read_to_string(repo_a.join("f.txt")).unwrap();
    assert_eq!(content, "main\n", "切回 main 后文件应恢复");

    let _ = fs::remove_dir_all(&repo_a);

    // ── git ──
    let repo_g = setup_repo("compat_br_g");
    run_git_ok(&repo_g, &["init"]);
    fs::write(repo_g.join("f.txt"), "main\n").unwrap();
    run_git_ok(&repo_g, &["add", "f.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "on main"]);

    run_git_ok(&repo_g, &["branch", "dev"]);
    let out = run_git_ok(&repo_g, &["branch"]);
    assert!(out.contains("dev"), "git branch 应列出 dev: {}", out);

    run_git_ok(&repo_g, &["checkout", "dev"]);
    fs::write(repo_g.join("f.txt"), "dev\n").unwrap();
    run_git_ok(&repo_g, &["add", "f.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "on dev"]);

    // git 默认主分支为 master
    run_git_ok(&repo_g, &["checkout", "master"]);
    let content = fs::read_to_string(repo_g.join("f.txt")).unwrap();
    assert_eq!(
        content.trim(),
        "main",
        "git 切回主分支后文件应恢复: {:?}",
        content
    );

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_merge_fast_forward() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_merge_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("f.txt"), "v1\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "init"]);

    run_agit_ok(&repo_a, &["branch", "-c", "feat"]);
    run_agit_ok(&repo_a, &["checkout", "feat"]);
    fs::write(repo_a.join("f.txt"), "v2\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "update on feat"]);

    run_agit_ok(&repo_a, &["checkout", "main"]);
    let out = run_agit_ok(&repo_a, &["merge", "feat"]);
    assert!(
        out.contains("Fast-forward") || out.contains("Merge"),
        "merge 应成功: {}",
        out
    );

    let content = fs::read_to_string(repo_a.join("f.txt")).unwrap();
    assert_eq!(content, "v2\n", "merge 后 main 应有 feat 的内容");

    let _ = fs::remove_dir_all(&repo_a);
}

#[test]
fn test_compat_log_shows_commits() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_log_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);

    for i in 1..=3 {
        fs::write(repo_a.join("f.txt"), format!("v{}\n", i)).unwrap();
        run_agit_ok(&repo_a, &["add", "f.txt"]);
        run_agit_ok(&repo_a, &["commit", "-m", &format!("commit {}", i)]);
    }

    let out = run_agit_ok(&repo_a, &["log", "--oneline"]);
    assert!(
        out.contains("commit 3"),
        "agit log 应显示 commit 3: {}",
        out
    );
    assert!(
        out.contains("commit 1"),
        "agit log 应显示 commit 1: {}",
        out
    );

    let out_full = run_agit_ok(&repo_a, &["log", "--oneline"]);
    let out_limited = run_agit_ok(&repo_a, &["log", "--oneline", "-n", "2"]);
    // --oneline 每行一个 commit，-n 2 应输出更少行
    let full_count = out_full.lines().filter(|l| !l.is_empty()).count();
    let limited_count = out_limited.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(
        limited_count, 2,
        "log --oneline -n 2 应恰好 2 行: full={} limited={}",
        full_count, limited_count
    );

    let _ = fs::remove_dir_all(&repo_a);

    // ── git ──
    let repo_g = setup_repo("compat_log_g");
    run_git_ok(&repo_g, &["init"]);

    for i in 1..=3 {
        fs::write(repo_g.join("f.txt"), format!("v{}\n", i)).unwrap();
        run_git_ok(&repo_g, &["add", "f.txt"]);
        run_git_ok(&repo_g, &["commit", "-m", &format!("commit {}", i)]);
    }

    let out = run_git_ok(&repo_g, &["log", "--oneline"]);
    assert!(out.contains("commit 3"), "git log 应显示 commit 3");
    assert!(out.contains("commit 1"), "git log 应显示 commit 1");

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_rm_and_mv() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_rmmv_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("old.txt"), "data\n").unwrap();
    run_agit_ok(&repo_a, &["add", "old.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "add file"]);

    // rm --cached
    fs::write(repo_a.join("temp.txt"), "tmp\n").unwrap();
    run_agit_ok(&repo_a, &["add", "temp.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "add temp"]);
    run_agit_ok(&repo_a, &["rm", "--cached", "temp.txt"]);
    assert!(
        repo_a.join("temp.txt").exists(),
        "rm --cached 文件应保留在磁盘"
    );

    // mv
    let out = run_agit_ok(&repo_a, &["mv", "old.txt", "new.txt"]);
    assert!(out.contains("Renamed"), "mv 应成功: {}", out);
    assert!(!repo_a.join("old.txt").exists(), "旧文件应不存在");
    assert!(repo_a.join("new.txt").exists(), "新文件应存在");

    let _ = fs::remove_dir_all(&repo_a);

    // ── git ──
    let repo_g = setup_repo("compat_rmmv_g");
    run_git_ok(&repo_g, &["init"]);
    fs::write(repo_g.join("old.txt"), "data\n").unwrap();
    run_git_ok(&repo_g, &["add", "old.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "add file"]);

    fs::write(repo_g.join("temp.txt"), "tmp\n").unwrap();
    run_git_ok(&repo_g, &["add", "temp.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "add temp"]);
    run_git_ok(&repo_g, &["rm", "--cached", "temp.txt"]);
    assert!(
        repo_g.join("temp.txt").exists(),
        "git rm --cached 文件应保留"
    );

    run_git_ok(&repo_g, &["mv", "old.txt", "new.txt"]);
    assert!(!repo_g.join("old.txt").exists(), "git mv 后旧文件应不存在");
    assert!(repo_g.join("new.txt").exists(), "git mv 后新文件应存在");

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_tag_create_and_list() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    // ── agit ──
    let repo_a = setup_repo("compat_tag_a");
    run_agit_ok(&repo_a, &["init"]);
    run_agit_ok(&repo_a, &["config", "user.name", "Test"]);
    run_agit_ok(&repo_a, &["config", "user.email", "test@test"]);
    fs::write(repo_a.join("f.txt"), "data\n").unwrap();
    run_agit_ok(&repo_a, &["add", "f.txt"]);
    run_agit_ok(&repo_a, &["commit", "-m", "init"]);

    run_agit_ok(&repo_a, &["tag", "create", "v1.0.0"]);
    let out = run_agit_ok(&repo_a, &["tag", "list"]);
    assert!(out.contains("v1.0.0"), "tag list 应显示 v1.0.0: {}", out);

    let _ = fs::remove_dir_all(&repo_a);

    // ── git ──
    let repo_g = setup_repo("compat_tag_g");
    run_git_ok(&repo_g, &["init"]);
    fs::write(repo_g.join("f.txt"), "data\n").unwrap();
    run_git_ok(&repo_g, &["add", "f.txt"]);
    run_git_ok(&repo_g, &["commit", "-m", "init"]);

    run_git_ok(&repo_g, &["tag", "v1.0.0"]);
    let out = run_git_ok(&repo_g, &["tag", "--list"]);
    assert!(out.contains("v1.0.0"), "git tag --list 应显示 v1.0.0");

    let _ = fs::remove_dir_all(&repo_g);
}

#[test]
fn test_compat_commit_has_author_and_message() {
    if !git_available() {
        eprintln!("跳过: 原生 Git 不可用");
        return;
    }

    let repo = setup_repo("compat_author");
    run_agit_ok(&repo, &["init"]);
    run_agit_ok(&repo, &["config", "user.name", "Test User"]);
    run_agit_ok(&repo, &["config", "user.email", "test@agit.local"]);
    fs::write(repo.join("code.rs"), "fn main() {}\n").unwrap();
    run_agit_ok(&repo, &["add", "code.rs"]);
    run_agit_ok(&repo, &["commit", "-m", "feat: hello world"]);

    // 从 .git/refs/heads/main 读取完整 commit SHA
    let head_ref = repo.join(".git").join("refs").join("heads").join("main");
    let sha = fs::read_to_string(&head_ref)
        .expect("应能读取 main ref")
        .trim()
        .to_string();
    assert_eq!(sha.len(), 40, "SHA 应为 40 字符: {}", sha);

    let cat_out = run_agit_ok(&repo, &["cat-file", "-p", &sha]);
    assert!(
        cat_out.contains("Test User"),
        "commit 应包含作者名: {}",
        cat_out
    );
    assert!(
        cat_out.contains("test@agit.local"),
        "commit 应包含作者邮箱: {}",
        cat_out
    );
    assert!(
        cat_out.contains("feat: hello world"),
        "commit 应包含提交消息: {}",
        cat_out
    );

    let _ = fs::remove_dir_all(&repo);
}

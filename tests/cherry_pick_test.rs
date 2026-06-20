//! 集成测试：cherry-pick 命令
//! 使用 tests/common/mod.rs 中的共享测试工具。

use std::fs;

mod common;
use common::*;

#[test]
fn test_cherry_pick_single() {
    let repo = setup_repo("cp_single");
    run_ok(&repo, &["init"]);

    // 创建 base commit
    fs::write(repo.join("f.txt"), "base\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "base"]);

    // 创建 feature 分支，在 feature 上加一个 commit
    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);
    fs::write(repo.join("f.txt"), "feature change\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "feature commit"]);

    // 获取 feature 的 commit SHA
    let head_ref = fs::read_to_string(repo.join(".git").join("HEAD")).unwrap();
    let feature_sha = if let Some(ref_path) = head_ref.trim().strip_prefix("ref: ") {
        fs::read_to_string(repo.join(".git").join(ref_path.trim()))
            .unwrap()
            .trim()
            .to_string()
    } else {
        panic!("Expected symbolic HEAD");
    };

    // 回到 main，cherry-pick 那个 commit
    run_ok(&repo, &["checkout", "main"]);
    let out = run_ok(&repo, &["cherry-pick", &feature_sha]);
    assert!(out.contains("Picked") || out.contains("completed"));

    // 验证更改已应用
    assert_eq!(
        fs::read_to_string(repo.join("f.txt")).unwrap(),
        "feature change\n"
    );

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_cherry_pick_multiple() {
    let repo = setup_repo("cp_multi");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "base\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "base"]);

    run_ok(&repo, &["branch", "-c", "feature"]);
    run_ok(&repo, &["checkout", "feature"]);

    // F1
    fs::write(repo.join("f.txt"), "feature1\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "feature commit 1"]);

    // F2
    fs::create_dir_all(repo.join("sub")).unwrap();
    fs::write(repo.join("sub/g.txt"), "new file\n").unwrap();
    run_ok(&repo, &["add", "sub/g.txt"]);
    run_ok(&repo, &["commit", "-m", "feature commit 2"]);

    // 获取两个 commit SHA
    let log = run_ok(&repo, &["log", "--oneline"]);
    let lines: Vec<&str> = log.lines().filter(|l| !l.is_empty()).collect();
    // 第一个 line 是最新的 commit (F2)，第二个是 F1
    // log --oneline 输出格式: "<short_sha> message"
    let f2_sha = lines[0].split_whitespace().next().unwrap();
    let f1_sha = lines[1].split_whitespace().next().unwrap();

    // 回到 main，cherry-pick 两个 commit
    run_ok(&repo, &["checkout", "main"]);
    let out = run_ok(&repo, &["cherry-pick", f1_sha, f2_sha]);
    assert!(out.contains("completed") || out.contains("Picked"));

    assert_eq!(
        fs::read_to_string(repo.join("f.txt")).unwrap(),
        "feature1\n"
    );
    assert!(repo.join("sub/g.txt").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_cherry_pick_conflict_and_continue() {
    let repo = setup_repo("cp_conflict");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "base\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "base"]);

    // 从 base 创建 feature 分支
    run_ok(&repo, &["branch", "-c", "feature"]);

    // main 上的修改
    fs::write(repo.join("f.txt"), "main change\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "on main"]);

    // feature 上的不同修改（与 main 修改同一位置 → 冲突）
    run_ok(&repo, &["checkout", "feature"]);
    fs::write(repo.join("f.txt"), "feature change\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "on feature"]);

    let feature_sha = fs::read_to_string(repo.join(".git/refs/heads/feature"))
        .unwrap()
        .trim()
        .to_string();

    // 回到 main，cherry-pick → 冲突
    run_ok(&repo, &["checkout", "main"]);
    let out = run_ok(&repo, &["cherry-pick", &feature_sha]);
    assert!(
        out.contains("could not apply") || out.contains("conflict"),
        "got: {}",
        out
    );

    let content = fs::read_to_string(repo.join("f.txt")).unwrap();
    assert!(
        content.contains("<<<<<<< HEAD"),
        "no conflict markers in: {}",
        content
    );

    // 解决冲突 → continue
    fs::write(repo.join("f.txt"), "resolved\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);

    let out = run_ok(&repo, &["cherry-pick", "--continue"]);
    assert!(
        out.contains("completed") || out.contains("Continued"),
        "got: {}",
        out
    );

    assert_eq!(
        fs::read_to_string(repo.join("f.txt")).unwrap(),
        "resolved\n"
    );
    assert!(!repo.join(".git/CHERRY_PICK_TODO").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_cherry_pick_abort() {
    let repo = setup_repo("cp_abort");
    run_ok(&repo, &["init"]);
    fs::write(repo.join("f.txt"), "v1\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "C1"]);

    // 从 C1 创建 feature
    run_ok(&repo, &["branch", "-c", "feature"]);

    // main 上创建 C2
    fs::write(repo.join("f.txt"), "v2\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "C2"]);

    // feature 上创建 C3（不同于 main 的修改）
    run_ok(&repo, &["checkout", "feature"]);
    fs::write(repo.join("f.txt"), "v3\n").unwrap();
    run_ok(&repo, &["add", "f.txt"]);
    run_ok(&repo, &["commit", "-m", "C3"]);

    let feature_sha = fs::read_to_string(repo.join(".git/refs/heads/feature"))
        .unwrap()
        .trim()
        .to_string();

    // cherry-pick C3 onto main → 冲突
    run_ok(&repo, &["checkout", "main"]);
    let out = run_ok(&repo, &["cherry-pick", &feature_sha]);
    assert!(out.contains("could not apply"), "got: {}", out);

    // abort
    let out = run_ok(&repo, &["cherry-pick", "--abort"]);
    assert!(out.contains("aborted"));

    // 恢复到原始 main 状态
    assert_eq!(fs::read_to_string(repo.join("f.txt")).unwrap(), "v2\n");
    assert!(!repo.join(".git/CHERRY_PICK_TODO").exists());
    assert!(!repo.join(".git/CHERRY_PICK_HEAD").exists());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_cherry_pick_empty() {
    let repo = setup_repo("cp_empty");
    run_ok(&repo, &["init"]);

    let out = run_err(&repo, &["cherry-pick"]);
    assert!(out.contains("No commits specified"));

    let _ = fs::remove_dir_all(&repo);
}

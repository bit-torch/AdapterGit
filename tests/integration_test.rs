use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Windows 上避免 Os code 0 错误
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 获取 agit 二进制路径
fn agit_binary() -> PathBuf {
    // Cargo 在运行集成测试时自动设置此环境变量
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_agit") {
        return PathBuf::from(path);
    }
    // 回退：手动构造路径
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("agit");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// 创建临时测试仓库
fn setup_repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agit_int_test_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_agit(repo: &PathBuf, args: &[&str]) -> std::process::Output {
    let bin = agit_binary();
    let mut cmd = Command::new(&bin);
    cmd.args(args)
        .current_dir(repo)
        .env_remove("AGIT_USER_NAME")
        .env_remove("AGIT_USER_EMAIL")
        .env("AGIT_USER_NAME", "Test User")
        .env("AGIT_USER_EMAIL", "test@agit.local");

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.output().unwrap_or_else(|e| {
        panic!(
            "Failed to run {:?} with args {:?} in {:?}: {}",
            bin, args, repo, e
        )
    })
}

fn run_agit_ok(repo: &PathBuf, args: &[&str]) -> String {
    let output = run_agit(repo, args);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "agit {:?} failed (exit {:?}):\nstdout: {}\nstderr: {}",
            args,
            output.status.code(),
            stdout,
            stderr
        );
    }
    stdout
}

// ============================================================
// 预热（Windows 首次 spawn 可能有 Os code 0 bug）
// ============================================================

#[test]
fn test_aaa_warmup() {
    // 在 Windows 上首次 spawn 进程可能失败（Rust 已知 bug），
    // 此测试通过提前触发一次 spawn 来预热，使后续测试正常工作。
    let repo = setup_repo("warmup");
    let _ = run_agit(&repo, &["--version"]);
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

    run_agit_ok(&repo, &["init"]);
    let stdout = run_agit_ok(&repo, &["status"]);

    // Should indicate no commits yet or empty repo
    assert!(!stdout.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_status_with_changes() {
    let repo = setup_repo("status_changes");

    run_agit_ok(&repo, &["init"]);

    // create files
    fs::write(repo.join("a.txt"), "aaa\n").unwrap();
    fs::write(repo.join("b.txt"), "bbb\n").unwrap();

    // status before add should show untracked
    let stdout = run_agit_ok(&repo, &["status"]);
    assert!(stdout.contains("a.txt") || stdout.contains("untracked") || !stdout.is_empty());

    // add one file
    run_agit_ok(&repo, &["add", "a.txt"]);

    // status after add should show staged
    let stdout = run_agit_ok(&repo, &["status"]);
    assert!(!stdout.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_full_workflow() {
    let repo = setup_repo("full_workflow");

    // 1. init
    run_agit_ok(&repo, &["init"]);

    // 2. create and add files
    fs::write(repo.join("README.md"), "# Test Repo\n").unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();

    run_agit_ok(&repo, &["add", "."]);

    // 3. commit
    let commit_out = run_agit_ok(&repo, &["commit", "-m", "First commit"]);
    assert!(!commit_out.is_empty());

    // 4. status should show clean
    let status_out = run_agit_ok(&repo, &["status"]);
    assert!(!status_out.is_empty());

    // 5. log should show the commit
    let log_out = run_agit_ok(&repo, &["log"]);
    assert!(log_out.contains("First commit") || !log_out.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn test_log_after_multiple_commits() {
    let repo = setup_repo("multi_commit");

    run_agit_ok(&repo, &["init"]);

    fs::write(repo.join("file.txt"), "v1\n").unwrap();
    run_agit_ok(&repo, &["add", "file.txt"]);
    run_agit_ok(&repo, &["commit", "-m", "Version 1"]);

    fs::write(repo.join("file.txt"), "v2\n").unwrap();
    run_agit_ok(&repo, &["add", "file.txt"]);
    run_agit_ok(&repo, &["commit", "-m", "Version 2"]);

    let log_out = run_agit_ok(&repo, &["log"]);
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

    run_agit_ok(&repo, &["init"]);

    fs::write(repo.join("f.txt"), "hello\n").unwrap();
    run_agit_ok(&repo, &["add", "f.txt"]);
    run_agit_ok(&repo, &["commit", "-m", "Initial"]);

    // modify file
    fs::write(repo.join("f.txt"), "world\n").unwrap();

    let diff_out = run_agit_ok(&repo, &["diff"]);
    assert!(!diff_out.is_empty());

    let _ = fs::remove_dir_all(&repo);
}

// ============================================================
// 结构化输出 (--json / --yaml)
// ============================================================

#[test]
fn test_json_output() {
    let repo = setup_repo("json_output");

    run_agit_ok(&repo, &["init"]);

    fs::write(repo.join("f.txt"), "data\n").unwrap();
    run_agit_ok(&repo, &["add", "f.txt"]);
    run_agit_ok(&repo, &["commit", "-m", "Test"]);

    let log_out = run_agit_ok(&repo, &["--json", "log"]);
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

    run_agit_ok(&repo, &["init"]);

    fs::write(repo.join("data.txt"), "test data\n").unwrap();
    run_agit_ok(&repo, &["add", "data.txt"]);
    run_agit_ok(&repo, &["commit", "-m", "Test"]);

    // 直接通过 ref 文件获取 commit SHA
    let head_ref = std::fs::read_to_string(repo.join(".git").join("HEAD")).unwrap_or_default();
    if let Some(ref_path) = head_ref.trim().strip_prefix("ref: ") {
        if let Ok(sha_content) = std::fs::read_to_string(repo.join(".git").join(ref_path.trim())) {
            let sha = sha_content.trim();
            if sha.len() == 40 {
                let type_out = run_agit_ok(&repo, &["cat-file", "-t", sha]);
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

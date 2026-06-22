use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 获取 agit 二进制路径
pub fn agit_binary() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_agit") {
        return PathBuf::from(path);
    }
    // CARGO_MANIFEST_DIR 现在是 agit-cli/，workspace 的 target 在上一级
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("target");
    path.push("debug");
    path.push("agit");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// 创建临时测试仓库
pub fn setup_repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("agit_test_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// 运行 agit 并返回原始输出
pub fn run_agit(repo: &PathBuf, args: &[&str]) -> std::process::Output {
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

    cmd.output().expect("Failed to run agit")
}

/// 运行 agit 并断言成功，返回 stdout 字符串
#[allow(dead_code)]
pub fn run_ok(repo: &PathBuf, args: &[&str]) -> String {
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

/// 运行 agit 并返回 stderr（用于预期失败的情况）
#[allow(dead_code)]
pub fn run_err(repo: &PathBuf, args: &[&str]) -> String {
    let output = run_agit(repo, args);
    String::from_utf8_lossy(&output.stderr).to_string()
}

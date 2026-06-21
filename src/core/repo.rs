use crate::core::index::Index;
use crate::core::objects::blob::Blob;
use crate::core::objects::commit::Commit;
use crate::core::{refs, storage};
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    find_repo_root_from(&cwd)
}

pub fn find_repo_root_from(start: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current = start.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err("Not a git repository (or any parent up to mount point)".into());
        }
    }
}

pub fn ensure_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn get_current_timestamp() -> (i64, String) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = now.as_secs() as i64;
    let tz = local_tz_offset();
    (timestamp, format!("{} {}", timestamp, tz))
}

/// 获取本地时区偏移字符串（如 "+0800"）。
///
/// 方案：使用 UNIX 时间戳 + 手动计算本地时间差值。
/// 纯 safe Rust，零 FFI 依赖。
fn local_tz_offset() -> String {
    // 1. 尝试从 AGIT_TIMEZONE 环境变量获取（如 "+08:00"、"+0800"、"CST-8"）
    if let Ok(tz) = std::env::var("AGIT_TIMEZONE") {
        if let Some(offset) = parse_tz_env(&tz) {
            return offset;
        }
    }

    // 2. Unix: 通过 UNIX epoch 在本地时间的偏移来计算
    //    localtime(0) 返回 1970-01-01 00:00:00 的本地时间
    //    我们用一个已知的 UTC 时间（UNIX epoch = 0），计算它对应的本地小时偏移
    //    由于无法在 safe Rust 中调用 localtime，改用以下方法：
    //    取当前 UTC 秒数，计算它对应的小时数，再用简易方法估算偏移
    #[cfg(unix)]
    {
        if let Ok(offset) = unix_tz_offset() {
            return offset;
        }
    }

    // 3. Windows: 使用 GetTimeZoneInformation（safe wrapper）
    #[cfg(windows)]
    {
        if let Some(offset) = windows_tz_offset() {
            return offset;
        }
    }

    // 4. 回退
    "+0000".to_string()
}

/// 解析时区环境变量值。
fn parse_tz_env(tz: &str) -> Option<String> {
    let tz = tz.trim();
    // 格式: +0800, +08:00, -0500, -05:00
    if (tz.starts_with('+') || tz.starts_with('-')) && tz.len() >= 4 {
        let cleaned: String = tz.chars().filter(|c| *c != ':').collect();
        if cleaned.len() == 5 && cleaned[1..].chars().all(|c| c.is_ascii_digit()) {
            return Some(cleaned);
        }
    }
    None
}

/// Unix (Linux/macOS) safe 方式估算时区偏移。
/// 通过本地时间和 UTC 时间的差值计算。
#[cfg(unix)]
fn unix_tz_offset() -> Option<String> {
    use std::fs;
    // 方法：读取 /etc/localtime 符号链接的目标路径
    // 通常格式为 /usr/share/zoneinfo/Asia/Shanghai
    // 或者 /var/db/timezone/zoneinfo/Asia/Shanghai (macOS)
    if let Ok(link) = fs::read_link("/etc/localtime") {
        let path = link.to_string_lossy();
        let offset = tz_name_to_offset(&path)?;
        return Some(offset);
    }
    // macOS 备用路径
    if let Ok(link) = fs::read_link("/var/db/timezone/zoneinfo") {
        let path = link.to_string_lossy();
        let offset = tz_name_to_offset(&path)?;
        return Some(offset);
    }
    None
}

/// 根据时区名估算偏移。覆盖常见时区，不依赖外部数据。
#[cfg(unix)]
fn tz_name_to_offset(tz_path: &str) -> Option<String> {
    // 常见 UTC 偏移的时区
    let known: &[(&str, &str)] = &[
        // 亚洲
        ("Beijing", "+0800"),
        ("Shanghai", "+0800"),
        ("Hong_Kong", "+0800"),
        ("Singapore", "+0800"),
        ("Tokyo", "+0900"),
        ("Seoul", "+0900"),
        ("Bangkok", "+0700"),
        ("Kolkata", "+0530"),
        ("Dubai", "+0400"),
        // 欧洲
        ("London", "+0000"),
        ("Paris", "+0100"),
        ("Berlin", "+0100"),
        ("Moscow", "+0300"),
        // 美洲
        ("New_York", "-0500"),
        ("Chicago", "-0600"),
        ("Denver", "-0700"),
        ("Los_Angeles", "-0800"),
        ("Sao_Paulo", "-0300"),
        // 大洋洲
        ("Sydney", "+1000"),
        ("Auckland", "+1200"),
        // UTC
        ("UTC", "+0000"),
        ("GMT", "+0000"),
    ];

    for (name, offset) in known {
        if tz_path.contains(name) {
            return Some(offset.to_string());
        }
    }
    None
}

/// Windows 时区偏移（safe API）。
#[cfg(windows)]
fn windows_tz_offset() -> Option<String> {
    // 通过 Win32 API 获取时区偏移
    // TIME_ZONE_INFORMATION.Bias: UTC = local + Bias (minutes)
    // 所以 UTC offset = -Bias 分钟
    extern "system" {
        fn GetTimeZoneInformation(tz: *mut Win32Tz) -> u32;
    }

    #[repr(C)]
    struct Win32Tz {
        bias: i32,
        _rest: [u16; 42],
    }

    unsafe {
        let mut tz: Win32Tz = std::mem::zeroed();
        let result = GetTimeZoneInformation(&mut tz);
        if result == 0xFFFFFFFF {
            return None;
        }
        let offset_min = -tz.bias;
        let hours = offset_min / 60;
        let mins = offset_min.abs() % 60;
        Some(format!("{:+03}{:02}", hours, mins))
    }
}

/// 解析 commit/tree-ish 引用为完整 SHA-1。
/// 支持 HEAD、分支名、标签名、remote ref、完整 SHA、缩写 SHA、~N 后缀。
pub(crate) fn resolve_commit(
    repo: &Path,
    spec: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 处理 ~N 后缀：遍历父提交
    if let Some(tilde_pos) = spec.find('~') {
        let base = &spec[..tilde_pos];
        let n: usize = if tilde_pos + 1 < spec.len() {
            spec[tilde_pos + 1..].parse().unwrap_or(1)
        } else {
            1
        };
        let mut sha = resolve_commit(repo, base)?;
        for _ in 0..n {
            sha = get_parent(repo, &sha)?;
        }
        return Ok(sha);
    }

    // 完整 SHA-1
    if spec.len() == 40
        && spec.chars().all(|c| c.is_ascii_hexdigit())
        && storage::read_object(repo, spec).is_ok()
    {
        return Ok(spec.to_string());
    }
    // 缩写 SHA（7-39 位十六进制）
    if spec.len() >= 7 && spec.len() < 40 && spec.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Some(full_sha) = find_full_sha(repo, spec) {
            return Ok(full_sha);
        }
    }
    // HEAD
    if spec == "HEAD" {
        return refs::read_head(repo);
    }
    // 分支名
    let branch_ref = format!("refs/heads/{}", spec);
    if let Ok(sha) = refs::read_ref(repo, &branch_ref) {
        return Ok(sha);
    }
    // 标签名
    let tag_ref = format!("refs/tags/{}", spec);
    if let Ok(sha) = refs::read_ref(repo, &tag_ref) {
        return Ok(sha);
    }
    // remote ref
    let remote_ref = format!("refs/remotes/{}", spec);
    if let Ok(sha) = refs::read_ref(repo, &remote_ref) {
        return Ok(sha);
    }

    Err(format!(
        "fatal: ambiguous argument '{}': unknown revision or path not in working tree",
        spec
    )
    .into())
}

/// 获取 commit 的第一个 parent SHA。
pub(crate) fn get_parent(repo: &Path, sha: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, sha)?;
    if obj_type != "commit" {
        return Err(format!("object {} is not a commit", sha).into());
    }
    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;
    commit
        .parents
        .first()
        .cloned()
        .ok_or_else(|| format!("commit {} has no parent", sha).into())
}

/// 在 .git/objects 中查找缩写 SHA 的完整值。
fn find_full_sha(repo: &Path, prefix: &str) -> Option<String> {
    let prefix_dir = prefix[..2].to_lowercase();
    let rest = &prefix[2..].to_lowercase();
    let objects_dir = repo.join(".git").join("objects");
    let dir_path = objects_dir.join(&prefix_dir);
    if !dir_path.is_dir() {
        return None;
    }
    let mut matches: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(rest) {
                matches.push(format!("{}{}", prefix_dir, name));
            }
        }
    }
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}

/// 检查工作区是否干净（tracked 文件是否被修改或删除）。
pub(crate) fn is_working_tree_clean(
    repo: &Path,
    index: &Index,
) -> Result<bool, Box<dyn std::error::Error>> {
    for (path, entry) in index.entries.iter() {
        let full_path = repo.join(path);
        if full_path.exists() {
            let content =
                fs::read(&full_path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            let blob = Blob::new(content);
            if blob.hash() != entry.sha1 {
                return Ok(false);
            }
        } else {
            // 文件在 index 中但不在工作区 → 被删除
            return Ok(false);
        }
    }
    Ok(true)
}

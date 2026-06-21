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
fn local_tz_offset() -> String {
    // 使用 time() + localtime() 计算 UTC 偏移
    // 这两个函数在所有主流平台均可使用
    extern "C" {
        fn time(t: *mut i64) -> i64;
        fn localtime(t: *const i64) -> *mut CTime;
    }

    #[repr(C)]
    struct CTime {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        #[cfg(not(target_os = "windows"))]
        tm_gmtoff: i64,
        #[cfg(not(target_os = "windows"))]
        tm_zone: *const u8,
    }

    unsafe {
        let mut now: i64 = 0;
        time(&mut now);
        let tm = localtime(&now);
        if tm.is_null() {
            return "+0000".to_string();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let offset = (*tm).tm_gmtoff;
            let hours = offset / 3600;
            let mins = (offset.abs() % 3600) / 60;
            format!("{:+03}{:02}", hours, mins)
        }
        #[cfg(target_os = "windows")]
        {
            // Windows 没有 tm_gmtoff，用 TIME_ZONE_INFORMATION 替代
            use std::mem::zeroed;
            #[repr(C)]
            struct TimeZoneInfo {
                bias: i32,
                _rest: [u16; 42],
            }
            extern "system" {
                fn GetTimeZoneInformation(tz: *mut TimeZoneInfo) -> u32;
            }
            let mut tz: TimeZoneInfo = zeroed();
            GetTimeZoneInformation(&mut tz);
            // bias 是 UTC = local + bias (分钟)，所以 offset = -bias
            let offset_min = -tz.bias;
            let hours = offset_min / 60;
            let mins = offset_min.abs() % 60;
            format!("{:+03}{:02}", hours, mins)
        }
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

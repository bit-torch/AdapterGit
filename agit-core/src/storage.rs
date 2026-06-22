use crate::compression::{compress, decompress};
use std::fs;
use std::path::{Path, PathBuf};

fn objects_dir(repo: &Path) -> PathBuf {
    repo.join(".git").join("objects")
}

/// 验证 SHA-1 十六进制字符串格式（40 字符，小写 hex）。
fn validate_sha1(sha1: &str) -> Result<(), String> {
    if sha1.len() != 40 {
        return Err(format!(
            "Invalid SHA-1 '{}': expected 40 hex chars, got {}",
            sha1,
            sha1.len()
        ));
    }
    if !sha1.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Invalid SHA-1 '{}': non-hex characters", sha1));
    }
    Ok(())
}

fn object_path(repo: &Path, sha1: &str) -> PathBuf {
    objects_dir(repo).join(&sha1[..2]).join(&sha1[2..])
}

pub fn write_object(
    repo: &Path,
    obj_type: &str,
    content: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut data = Vec::with_capacity(header.len() + content.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(content);

    let sha1 = crate::hash::hash_bytes(&data);
    let compressed = compress(&data)?;

    let path = object_path(repo, &sha1);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, &compressed)?;

    Ok(sha1)
}

pub fn read_object(
    repo: &Path,
    sha1: &str,
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    validate_sha1(sha1)?;
    let path = object_path(repo, sha1);
    let compressed =
        fs::read(&path).map_err(|e| format!("Failed to read object {}: {}", sha1, e))?;
    let data = decompress(&compressed)?;

    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or("Invalid object: no null byte")?;
    let header = std::str::from_utf8(&data[..null_pos])?;
    let parts: Vec<&str> = header.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err("Invalid object header".into());
    }
    let obj_type = parts[0].to_string();
    let content = data[null_pos + 1..].to_vec();

    Ok((obj_type, content))
}

pub fn object_exists(repo: &Path, sha1: &str) -> bool {
    validate_sha1(sha1).is_ok() && object_path(repo, sha1).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_repo(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("agit_test_storage_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git").join("objects")).unwrap();
        dir
    }

    #[test]
    fn test_write_and_read_object() {
        let repo = setup_repo("write_read");

        let content = b"hello world\n";
        let sha1 = write_object(&repo, "blob", content).unwrap();
        assert_eq!(sha1.len(), 40);
        assert!(object_exists(&repo, &sha1));

        let (obj_type, read_content) = read_object(&repo, &sha1).unwrap();
        assert_eq!(obj_type, "blob");
        assert_eq!(read_content, content);

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_object_not_exists() {
        let repo = setup_repo("not_exists");
        assert!(!object_exists(
            &repo,
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        ));
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_write_object_structure() {
        let repo = setup_repo("structure");

        let sha1 = write_object(&repo, "blob", b"test\n").unwrap();
        let dir = repo.join(".git").join("objects").join(&sha1[..2]);
        let file = dir.join(&sha1[2..]);

        assert!(dir.exists());
        assert!(file.exists());
        assert!(!fs::read(&file).unwrap().is_empty());

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_read_nonexistent_object() {
        let repo = setup_repo("nonexistent");
        let result = read_object(&repo, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
        assert!(result.is_err());
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_write_read_roundtrip_various_sizes() {
        let repo = setup_repo("roundtrip");

        for size in [0, 1, 10, 100, 1000] {
            let content = vec![b'A'; size];
            let sha1 = write_object(&repo, "blob", &content).unwrap();
            let (obj_type, read_content) = read_object(&repo, &sha1).unwrap();
            assert_eq!(obj_type, "blob");
            assert_eq!(read_content, content);
        }

        let _ = fs::remove_dir_all(&repo);
    }
}

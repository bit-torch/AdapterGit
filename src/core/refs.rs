use std::fs;
use std::path::{Path, PathBuf};

fn refs_dir(repo: &Path) -> PathBuf {
    repo.join(".git")
}

fn head_file(repo: &Path) -> PathBuf {
    refs_dir(repo).join("HEAD")
}

fn ref_path(repo: &Path, name: &str) -> PathBuf {
    refs_dir(repo).join(name)
}

pub fn read_head(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let head_path = head_file(repo);
    let content = fs::read_to_string(&head_path)
        .map_err(|e| format!("Failed to read HEAD at {}: {}", head_path.display(), e))?;
    let content = content.trim();

    if let Some(ref_path) = content.strip_prefix("ref: ") {
        let ref_path = ref_path.trim();
        read_ref(repo, ref_path)
    } else {
        Ok(content.to_string())
    }
}

pub fn write_head(repo: &Path, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let head = head_file(repo);
    let dir = head.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;

    fs::write(&head, format!("{}\n", target))?;
    Ok(())
}

pub fn read_ref(repo: &Path, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = ref_path(repo, name);
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read ref {}: {}", path.display(), e))?;
    Ok(content.trim().to_string())
}

pub fn write_ref(repo: &Path, name: &str, sha1: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = ref_path(repo, name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, format!("{}\n", sha1))?;
    Ok(())
}

pub fn create_branch(
    repo: &Path,
    name: &str,
    sha1: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_ref(repo, &format!("refs/heads/{}", name), sha1)
}

pub fn list_branches(repo: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let heads_dir = refs_dir(repo).join("refs").join("heads");
    if !heads_dir.exists() {
        return Ok(Vec::new());
    }

    let mut branches = Vec::new();
    let entries =
        fs::read_dir(&heads_dir).map_err(|e| format!("Failed to read heads dir: {}", e))?;

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            branches.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_repo(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("agit_test_refs_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn setup_git_dir(repo: &Path) {
        let git_dir = refs_dir(repo);
        fs::create_dir_all(git_dir.join("refs").join("heads")).unwrap();
        fs::create_dir_all(git_dir.join("refs").join("tags")).unwrap();
    }

    #[test]
    fn test_read_head_symbolic() {
        let repo = setup_repo("symbolic");
        setup_git_dir(&repo);

        fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(repo.join(".git/refs/heads/main"), "abc123def456\n").unwrap();

        let result = read_head(&repo).unwrap();
        assert_eq!(result, "abc123def456");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_read_head_detached() {
        let repo = setup_repo("detached");
        setup_git_dir(&repo);

        fs::write(
            repo.join(".git/HEAD"),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n",
        )
        .unwrap();

        let result = read_head(&repo).unwrap();
        assert_eq!(result, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_write_head_symbolic() {
        let repo = setup_repo("writesym");
        setup_git_dir(&repo);

        write_head(&repo, "ref: refs/heads/feature").unwrap();
        let content = fs::read_to_string(repo.join(".git/HEAD")).unwrap();
        assert_eq!(content.trim(), "ref: refs/heads/feature");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_write_head_detached() {
        let repo = setup_repo("writedet");
        setup_git_dir(&repo);

        write_head(&repo, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
        let content = fs::read_to_string(repo.join(".git/HEAD")).unwrap();
        assert_eq!(content.trim(), "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_read_write_ref_roundtrip() {
        let repo = setup_repo("roundtrip");
        setup_git_dir(&repo);

        write_ref(&repo, "refs/heads/main", "abc123").unwrap();
        let result = read_ref(&repo, "refs/heads/main").unwrap();
        assert_eq!(result, "abc123");

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_create_and_list_branches() {
        let repo = setup_repo("branches");
        setup_git_dir(&repo);

        create_branch(&repo, "main", "abc123").unwrap();
        create_branch(&repo, "feature", "def456").unwrap();

        let mut branches = list_branches(&repo).unwrap();
        branches.sort();
        assert_eq!(branches, vec!["feature", "main"]);

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_list_branches_empty() {
        let repo = setup_repo("emptybranches");
        setup_git_dir(&repo);

        let branches = list_branches(&repo).unwrap();
        assert!(branches.is_empty());

        let _ = fs::remove_dir_all(&repo);
    }
}

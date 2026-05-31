use crate::core::index::Index;
use crate::core::objects::blob::Blob;
use crate::core::repo;
use crate::core::storage;
use std::fs;
use std::path::Path;

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            return meta.permissions().mode() & 0o111 != 0;
        }
    }
    let _ = path;
    false
}

fn file_mode(path: &Path) -> &str {
    if is_executable(path) {
        "100755"
    } else {
        "100644"
    }
}

pub fn run(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let mut index = Index::load(&repo_root)?;

    let mut added_count = 0;

    for file_path in files {
        let full_path = std::env::current_dir()?.join(file_path);

        if !full_path.exists() {
            eprintln!("error: {}: does not exist", file_path);
            continue;
        }

        if full_path.is_dir() {
            add_directory(&repo_root, &full_path, &mut index, &mut added_count)?;
        } else {
            add_file(&repo_root, &full_path, &mut index, &mut added_count)?;
        }
    }

    index.save(&repo_root)?;

    if added_count > 0 {
        println!("Added {} file(s) to index", added_count);
    }

    Ok(())
}

fn add_file(
    repo_root: &Path,
    path: &Path,
    index: &mut Index,
    count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read(path)?;
    let blob = Blob::new(content);
    let sha1 = blob.hash();

    if !storage::object_exists(repo_root, &sha1) {
        storage::write_object(repo_root, "blob", &blob.content)?;
    }

    let relative = path.strip_prefix(repo_root).unwrap_or(path);
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    let mode = file_mode(path);
    index.add_entry(mode, &sha1, &relative_str);

    *count += 1;
    Ok(())
}

fn add_directory(
    repo_root: &Path,
    dir: &Path,
    index: &mut Index,
    count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.file_name().is_some_and(|n| n == ".git") {
            continue;
        }

        if path.is_dir() {
            add_directory(repo_root, &path, index, count)?;
        } else if path.is_file() {
            add_file(repo_root, &path, index, count)?;
        }
    }
    Ok(())
}

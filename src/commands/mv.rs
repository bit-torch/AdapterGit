//! mv 命令：在索引和工作区中移动/重命名文件。

use crate::core::index::Index;
use crate::core::repo;
use std::fs;

pub fn run(source: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let mut index = Index::load(&repo_root)?;

    if !index.entries.contains_key(source) {
        return Err(format!("error: '{}' is not tracked by agit", source).into());
    }

    // 移动工作区文件
    let source_path = repo_root.join(source);
    let dest_path = repo_root.join(dest);
    if source_path.exists() {
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&source_path, &dest_path)?;
    }

    // 更新索引
    if let Some(entry) = index.entries.remove(source) {
        index.add_entry(&entry.mode, &entry.sha1, dest);
    }

    index.save(&repo_root)?;
    println!("Renamed '{}' -> '{}'", source, dest);
    Ok(())
}

//! rm 命令：从索引和工作区删除文件。

use crate::core::index::Index;
use crate::core::repo;
use std::fs;

pub fn run(cached: bool, files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let mut index = Index::load(&repo_root)?;

    for file in files {
        let full_path = repo_root.join(file);

        if !cached && full_path.exists() {
            fs::remove_file(&full_path)?;
        }

        if index.entries.contains_key(file) {
            index.remove_entry(file);
            println!("rm '{}'", file);
        } else if !cached && full_path.exists() {
            // 即使不在 index 中也删除了工作区文件
            println!("rm '{}'", file);
        } else {
            eprintln!("error: '{}' not tracked", file);
        }
    }

    index.save(&repo_root)?;
    Ok(())
}

//! rm 命令：从索引和工作区删除文件。

use crate::core::index::Index;
use crate::core::repo;
use std::fs;

pub fn run(cached: bool, files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let mut index = Index::load(&repo_root)?;

    for file in files {
        let full_path = repo_root.join(file);

        // 先检查是否在索引中，未跟踪的文件拒绝删除（与 git 行为一致）
        if !index.entries.contains_key(file) {
            eprintln!("error: '{}' not tracked", file);
            continue;
        }

        // 从索引中删除
        index.remove_entry(file);

        // 非 cached 模式从工作区删除文件
        if !cached && full_path.exists() {
            fs::remove_file(&full_path)?;
        }

        println!("rm '{}'", file);
    }

    index.save(&repo_root)?;
    Ok(())
}

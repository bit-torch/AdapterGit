use crate::core::objects::blob::Blob;
use crate::core::objects::tree::Tree;
use crate::core::{index, refs, storage};
use std::fs;
use std::path::Path;

/// 切换到指定分支（更新 HEAD 和工作目录）。
pub fn switch_branch(repo: &Path, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_sha = refs::read_ref(repo, &format!("refs/heads/{}", branch_name))?;
    let (obj_type, body) = storage::read_object(repo, &target_sha)?;
    if obj_type != "commit" {
        return Err(format!("ref '{}' does not point to a commit", branch_name).into());
    }
    let commit_data = with_object_header("commit", &body);
    let commit = crate::core::objects::commit::Commit::deserialize(&commit_data)?;

    restore_tree(repo, &commit.tree, Path::new(""))?;
    refs::write_head(repo, &format!("ref: refs/heads/{}", branch_name))?;

    println!("Switched to branch '{}'", branch_name);
    Ok(())
}

/// 递归恢复 tree 内容到工作目录。
fn restore_tree(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, body) = storage::read_object(repo, tree_sha)?;
    if obj_type != "tree" {
        return Err(format!("object {} is not a tree", tree_sha).into());
    }

    let tree_data = with_object_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;
    let mut new_index = index::Index::new();

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);

        match entry.mode.as_str() {
            "40000" => {
                let dir_path = repo.join(&entry_path);
                fs::create_dir_all(&dir_path)?;
                restore_tree(repo, &entry.sha1, &entry_path)?;
            }
            _ => {
                let (blob_type, blob_body) = storage::read_object(repo, &entry.sha1)?;
                if blob_type != "blob" {
                    continue;
                }
                let blob_data = with_object_header("blob", &blob_body);
                let blob = Blob::deserialize(&blob_data)?;

                let file_path = repo.join(&entry_path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&file_path, &blob.content)?;

                new_index.add_entry(
                    &entry.mode,
                    &entry.sha1,
                    &entry_path.to_string_lossy(),
                );
            }
        }
    }

    new_index.save(repo)?;
    Ok(())
}

/// 重新构造完整的对象数据：`{type} {len}\0{body}`
fn with_object_header(obj_type: &str, body: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type, body.len());
    let mut data = Vec::with_capacity(header.len() + body.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(body);
    data
}

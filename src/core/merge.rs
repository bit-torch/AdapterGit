use crate::core::objects::blob::Blob;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::{index, refs, storage};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// 合并指定分支到当前分支。
///
/// 先尝试 fast-forward；若分叉则做 3-way merge。
pub fn merge_branch(
    repo: &Path,
    branch_name: &str,
    author: &str,
    committer: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let head_sha = refs::read_head(repo)?;
    let target_sha = refs::read_ref(repo, &format!("refs/heads/{}", branch_name))?;

    if head_sha == target_sha {
        println!("Already up to date.");
        return Ok(());
    }

    // 查找共同祖先
    let base_sha = find_merge_base(repo, &head_sha, &target_sha)?;

    // Fast-forward: HEAD 是 target 的祖先
    if base_sha == head_sha {
        println!("Fast-forward");
        // 记录当前工作目录中已跟踪的文件
        let old_index = index::Index::load(repo)?;
        let old_tracked: BTreeSet<String> = old_index.entries.keys().cloned().collect();

        // 更新当前分支 ref 指向 target SHA
        let current_branch = refs::get_current_branch(repo)?;
        if let Some(current) = &current_branch {
            refs::write_ref(repo, &format!("refs/heads/{}", current), &target_sha)?;
        }
        // 重建工作目录到 target commit 的 tree
        let (_, body) = storage::read_object(repo, &target_sha)?;
        let commit_data = with_header("commit", &body);
        let commit = Commit::deserialize(&commit_data)?;
        restore_working_tree(repo, &commit.tree)?;

        // 收集目标树中所有文件
        let new_tracked = collect_tree_paths(repo, &commit.tree, Path::new(""))?;

        // 清理旧索引中存在但新树中不存在的文件
        for path in old_tracked.iter() {
            if !new_tracked.contains(path) {
                let file_path = repo.join(path);
                if file_path.is_file() || file_path.is_symlink() {
                    let _ = fs::remove_file(&file_path);
                }
            }
        }
        let _ = remove_empty_dirs(repo, Path::new(""));

        println!("Updated to {}", &target_sha[..7]);
        return Ok(());
    }

    // 3-way merge
    println!("Merge made by the 'agit' strategy.");
    let has_conflicts = three_way_merge(repo, &base_sha, &head_sha, &target_sha)?;

    if has_conflicts {
        println!("Automatic merge failed; fix conflicts and then commit the result.");
        // 写 MERGE_HEAD 标记
        let git_dir = repo.join(".git");
        fs::write(git_dir.join("MERGE_HEAD"), format!("{}\n", target_sha))?;
        fs::write(
            git_dir.join("MERGE_MSG"),
            format!("Merge branch '{}'\n", branch_name),
        )?;
        return Ok(());
    }

    // 创建 merge commit
    let index = index::Index::load(repo)?;
    let mut tree = Tree::new();
    for entry in index.entries.values() {
        tree.add_entry(&entry.mode, &entry.path, &entry.sha1);
    }
    let tree_sha = tree.hash();
    storage::write_object(repo, "tree", &tree.serialize_raw())?;

    let mut commit = Commit::new(
        &tree_sha,
        author,
        committer,
        &format!("Merge branch '{}'\n", branch_name),
    );
    commit.add_parent(&head_sha);
    commit.add_parent(&target_sha);

    let commit_sha = commit.hash();
    storage::write_object(repo, "commit", &commit.serialize_raw())?;

    // 更新当前分支引用
    let current_branch = refs::get_current_branch(repo)?;
    if let Some(current) = &current_branch {
        refs::write_ref(repo, &format!("refs/heads/{}", current), &commit_sha)?;
    }

    println!("Merge commit: {}", &commit_sha[..7]);
    Ok(())
}

/// 查找两个 commit 的共同祖先（简易 BFS）。
fn find_merge_base(
    repo: &Path,
    sha1: &str,
    sha2: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    fn collect_ancestors(
        repo: &Path,
        sha: &str,
        visited: &mut BTreeMap<String, usize>,
        depth: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if visited.contains_key(sha) {
            return Ok(());
        }
        visited.insert(sha.to_string(), depth);

        let (_, body) = storage::read_object(repo, sha)?;
        let commit_data = with_header("commit", &body);
        let commit = Commit::deserialize(&commit_data)?;

        // DFS on parents, limited depth
        if depth < 100 {
            for parent in &commit.parents {
                collect_ancestors(repo, parent, visited, depth + 1)?;
            }
        }
        Ok(())
    }

    let mut ancestors1 = BTreeMap::new();
    collect_ancestors(repo, sha1, &mut ancestors1, 0)?;

    // BFS from sha2 to find nearest common
    let mut queue = vec![(sha2.to_string(), 0usize)];
    let mut seen = BTreeMap::new();
    while let Some((current, depth)) = queue.pop() {
        if seen.contains_key(&current) {
            continue;
        }
        seen.insert(current.clone(), depth);
        if let Some(_d) = ancestors1.get(&current) {
            return Ok(current);
        }
        if depth < 100 {
            let (_, body) = storage::read_object(repo, &current)?;
            let commit_data = with_header("commit", &body);
            if let Ok(commit) = Commit::deserialize(&commit_data) {
                for parent in &commit.parents {
                    if !seen.contains_key(parent) {
                        queue.push((parent.clone(), depth + 1));
                    }
                }
            }
        }
    }

    // fallback: no common ancestor found, use first commit
    Ok(sha1.to_string())
}

/// 3-way merge: 比较 base/ours/theirs 的 tree，生成合并结果。
/// 返回是否产生了冲突。
pub(crate) fn three_way_merge(
    repo: &Path,
    base_sha: &str,
    ours_sha: &str,
    theirs_sha: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let base_files = read_tree_files(repo, base_sha)?;
    let ours_files = read_tree_files(repo, ours_sha)?;
    let theirs_files = read_tree_files(repo, theirs_sha)?;

    let mut has_conflicts = false;
    let mut new_index = index::Index::new();

    let all_paths: BTreeMap<String, bool> = {
        let mut paths = BTreeMap::new();
        for p in base_files
            .keys()
            .chain(ours_files.keys())
            .chain(theirs_files.keys())
        {
            paths.insert(p.clone(), true);
        }
        paths
    };

    for path in all_paths.keys() {
        let base = base_files.get(path);
        let ours = ours_files.get(path);
        let theirs = theirs_files.get(path);

        match (base, ours, theirs) {
            // 双方都未修改
            (Some(b), Some(o), Some(t)) if b == o && b == t => {
                write_file_and_index(repo, path, o, &mut new_index)?;
            }
            // 只有 theirs 修改了
            (Some(b), Some(o), Some(t)) if b == o && b != t => {
                write_file_and_index(repo, path, t, &mut new_index)?;
            }
            // 只有 ours 修改了
            (Some(b), Some(o), Some(t)) if b != o && b == t => {
                write_file_and_index(repo, path, o, &mut new_index)?;
            }
            // 双方同样修改了
            (_, Some(o), Some(t)) if o.sha1 == t.sha1 => {
                write_file_and_index(repo, path, o, &mut new_index)?;
            }
            // 新增文件 (仅在 ours)
            (None, Some(o), None) => {
                write_file_and_index(repo, path, o, &mut new_index)?;
            }
            // 新增文件 (仅在 theirs)
            (None, None, Some(t)) => {
                write_file_and_index(repo, path, t, &mut new_index)?;
            }
            // 删除文件 (双方都删了)
            (Some(_), None, None) => {
                // 删除工作区文件
                let _ = fs::remove_file(repo.join(path));
            }
            // 冲突: 双方都修改了同一个文件
            _ => {
                has_conflicts = true;
                let conflict_content = create_conflict_marker(
                    repo,
                    path,
                    base.map(|f| f.sha1.as_str()),
                    ours.map(|f| f.sha1.as_str()),
                    theirs.map(|f| f.sha1.as_str()),
                )?;
                // 写入冲突标记文件
                let file_path = repo.join(path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&file_path, &conflict_content)?;

                // 索引中使用 ours 的 blob（冲突标记版本，之后 commit 会解决）
                let sha1 = storage::write_object(repo, "blob", &conflict_content)?;
                new_index.add_entry("100644", &sha1, path);
            }
        }
    }

    new_index.save(repo)?;
    Ok(has_conflicts)
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    sha1: String,
    mode: String,
}

/// 从 commit SHA 出发，读取 tree 中的所有文件映射。
fn read_tree_files(
    repo: &Path,
    commit_sha: &str,
) -> Result<BTreeMap<String, FileInfo>, Box<dyn std::error::Error>> {
    let (_, body) = storage::read_object(repo, commit_sha)?;
    let commit_data = with_header("commit", &body);
    let commit = Commit::deserialize(&commit_data)?;
    read_tree_files_recursive(repo, &commit.tree, Path::new(""))
}

fn read_tree_files_recursive(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
) -> Result<BTreeMap<String, FileInfo>, Box<dyn std::error::Error>> {
    let mut files = BTreeMap::new();
    let (_, body) = storage::read_object(repo, tree_sha)?;
    let tree_data = with_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);
        if entry.mode == "40000" {
            let sub = read_tree_files_recursive(repo, &entry.sha1, &entry_path)?;
            files.extend(sub);
        } else {
            files.insert(
                entry_path.to_string_lossy().to_string(),
                FileInfo {
                    sha1: entry.sha1.clone(),
                    mode: entry.mode.clone(),
                },
            );
        }
    }

    Ok(files)
}

/// 将文件写入工作目录并加入索引。
fn write_file_and_index(
    repo: &Path,
    path: &str,
    info: &FileInfo,
    idx: &mut index::Index,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_, blob_body) = storage::read_object(repo, &info.sha1)?;
    let blob_data = with_header("blob", &blob_body);
    let blob = Blob::deserialize(&blob_data)?;

    let file_path = repo.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file_path, &blob.content)?;

    idx.add_entry(&info.mode, &info.sha1, path);
    Ok(())
}

/// 生成冲突标记内容。
fn create_conflict_marker(
    repo: &Path,
    path: &str,
    _base_sha: Option<&str>,
    ours_sha: Option<&str>,
    theirs_sha: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    fn read_file_content(repo: &Path, sha_opt: Option<&str>) -> Vec<u8> {
        if let Some(sha) = sha_opt {
            if let Ok((_, body)) = storage::read_object(repo, sha) {
                let blob_data = with_header("blob", &body);
                if let Ok(blob) = Blob::deserialize(&blob_data) {
                    return blob.content;
                }
            }
        }
        Vec::new()
    }

    let ours_content = read_file_content(repo, ours_sha);
    let theirs_content = read_file_content(repo, theirs_sha);

    let mut result = Vec::new();
    result.extend_from_slice(b"<<<<<<< HEAD\n");
    result.extend_from_slice(&ours_content);
    if !ours_content.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(b"=======\n");
    result.extend_from_slice(&theirs_content);
    if !theirs_content.ends_with(b"\n") {
        result.push(b'\n');
    }
    result.extend_from_slice(format!(">>>>>>> {}\n", path).as_bytes());

    Ok(result)
}

fn with_header(obj_type: &str, body: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type, body.len());
    let mut data = Vec::with_capacity(header.len() + body.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(body);
    data
}

/// 递归收集树中所有文件路径。
fn collect_tree_paths(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut paths = BTreeSet::new();
    let (_, body) = match storage::read_object(repo, tree_sha) {
        Ok(v) => v,
        Err(_) => return Ok(paths),
    };
    let tree_data = with_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);
        let path_str = entry_path.to_string_lossy().to_string();

        if entry.mode == "40000" {
            let sub = collect_tree_paths(repo, &entry.sha1, &entry_path)?;
            paths.extend(sub);
        } else {
            paths.insert(path_str);
        }
    }
    Ok(paths)
}

/// 递归清理空目录。
fn remove_empty_dirs(repo: &Path, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let full = repo.join(dir);
    if !full.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&full)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let relative = path.strip_prefix(repo).unwrap_or(&path);
            remove_empty_dirs(repo, relative)?;
        }
    }
    if full
        .read_dir()
        .map(|mut d| d.next().is_none())
        .unwrap_or(false)
    {
        let _ = fs::remove_dir(&full);
    }
    Ok(())
}

fn restore_working_tree(repo: &Path, tree_sha: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut new_index = index::Index::new();
    restore_tree_recursive(repo, tree_sha, Path::new(""), &mut new_index)?;
    new_index.save(repo)?;
    Ok(())
}

/// 递归恢复 tree 内容到工作目录和 index。
fn restore_tree_recursive(
    repo: &Path,
    tree_sha: &str,
    prefix: &Path,
    idx: &mut index::Index,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_, body) = storage::read_object(repo, tree_sha)?;
    let tree_data = with_header("tree", &body);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let entry_path = prefix.join(&entry.name);
        if entry.mode == "40000" {
            fs::create_dir_all(repo.join(&entry_path))?;
            restore_tree_recursive(repo, &entry.sha1, &entry_path, idx)?;
        } else {
            let (_, blob_body) = storage::read_object(repo, &entry.sha1)?;
            let blob_data = with_header("blob", &blob_body);
            let blob = Blob::deserialize(&blob_data)?;
            let file_path = repo.join(&entry_path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, &blob.content)?;
            idx.add_entry(&entry.mode, &entry.sha1, &entry_path.to_string_lossy());
        }
    }
    Ok(())
}

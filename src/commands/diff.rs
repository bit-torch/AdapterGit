use crate::core::index::Index;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn run(
    cached: bool,
    name_only: bool,
    commit1: Option<&str>,
    commit2: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    // 两个 commit 都指定 → 比较两个提交的 tree
    if let (Some(c1), Some(c2)) = (commit1, commit2) {
        let tree1 = resolve_to_tree(&repo_root, c1)?;
        let tree2 = resolve_to_tree(&repo_root, c2)?;
        let map1 = build_tree_map_from_tree(&repo_root, &tree1);
        let map2 = build_tree_map_from_tree(&repo_root, &tree2);
        return print_tree_diff(&repo_root, &map1, &map2, name_only, "a", "b");
    }

    // 仅一个 commit → 与工作区比较
    if let Some(c) = commit1 {
        let tree = resolve_to_tree(&repo_root, c)?;
        let tree_map = build_tree_map_from_tree(&repo_root, &tree);
        let index = Index::load(&repo_root)?;
        return print_tree_vs_working(&repo_root, &tree_map, &index, name_only);
    }

    let index = Index::load(&repo_root)?;

    // --cached: HEAD vs index
    if cached {
        let head_sha1 = refs::read_head(&repo_root)?;
        let head_tree_map = build_head_tree_map(&repo_root, &head_sha1);
        let index_map = build_index_map(&repo_root, &index);
        return print_tree_diff(&repo_root, &head_tree_map, &index_map, name_only, "a", "b");
    }

    // 默认：index vs working tree (已有逻辑)
    let head_sha1 = match refs::read_head(&repo_root) {
        Ok(sha1) => sha1,
        Err(_) => {
            show_untracked_diff(&repo_root, &index)?;
            return Ok(());
        }
    };

    let head_tree_map = build_head_tree_map(&repo_root, &head_sha1);
    let mut diff_output = Vec::new();

    for path in index.entries.keys() {
        let old_content = head_tree_map
            .get(path)
            .and_then(|sha1| read_blob_content(&repo_root, sha1))
            .unwrap_or_default();

        let full_path = repo_root.join(path);
        let new_content = if full_path.exists() {
            fs::read(&full_path).unwrap_or_else(|e| {
                eprintln!("warning: failed to read '{}': {}", path, e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        if old_content != new_content {
            if name_only {
                println!("{}", path);
            } else {
                let a_path = format!("a/{}", path);
                let b_path = format!("b/{}", path);
                let diff = generate_unified_diff(&a_path, &b_path, &old_content, &new_content);
                diff_output.push((path.clone(), diff));
            }
        }
    }

    for path in head_tree_map.keys() {
        if !index.entries.contains_key(path) {
            let full_path = repo_root.join(path);
            if !full_path.exists() {
                if name_only {
                    println!("{}", path);
                } else {
                    let old_content =
                        read_blob_content(&repo_root, &head_tree_map[path]).unwrap_or_default();
                    let a_path = format!("a/{}", path);
                    let b_path = format!("b/{}", path);
                    let diff = generate_unified_diff_deleted(&a_path, &b_path, &old_content);
                    diff_output.push((path.clone(), diff));
                }
            }
        }
    }

    if diff_output.is_empty() {
        return Ok(());
    }

    for (_path, diff) in &diff_output {
        print!("{}", diff);
    }

    Ok(())
}

/// 解析引用为 tree SHA。
fn resolve_to_tree(repo: &Path, spec: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 处理 ~N 后缀
    let (base, n) = if let Some(tilde_pos) = spec.find('~') {
        let n: usize = spec[tilde_pos + 1..].parse().unwrap_or(1);
        (&spec[..tilde_pos], n)
    } else {
        (spec, 0)
    };

    let mut sha = resolve_single_ref(repo, base)?;
    for _ in 0..n {
        sha = get_parent(repo, &sha)?;
    }
    commit_to_tree(repo, &sha)
}

fn resolve_single_ref(repo: &Path, spec: &str) -> Result<String, Box<dyn std::error::Error>> {
    if spec.len() == 40
        && spec.chars().all(|c| c.is_ascii_hexdigit())
        && storage::read_object(repo, spec).is_ok()
    {
        return Ok(spec.to_string());
    }
    if spec == "HEAD" {
        return refs::read_head(repo);
    }
    if let Ok(sha) = refs::read_ref(repo, &format!("refs/heads/{}", spec)) {
        return Ok(sha);
    }
    if let Ok(sha) = refs::read_ref(repo, &format!("refs/tags/{}", spec)) {
        return Ok(sha);
    }
    Err(format!("fatal: '{}' is not a valid tree/commit", spec).into())
}

/// 获取 commit 的 first parent。
fn get_parent(repo: &Path, sha: &str) -> Result<String, Box<dyn std::error::Error>> {
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

fn commit_to_tree(repo: &Path, commit_sha: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (_, content) = storage::read_object(repo, commit_sha)?;
    let commit_data = crate::core::objects::format_object_data("commit", &content);
    let commit = Commit::deserialize(&commit_data)?;
    Ok(commit.tree)
}

fn build_tree_map_from_tree(repo: &Path, tree_sha: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let _ = collect_tree_recursive(repo, tree_sha, "", &mut result);
    result
}

fn build_index_map(_repo: &Path, index: &Index) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for (path, entry) in &index.entries {
        result.insert(path.clone(), entry.sha1.clone());
    }
    result
}

fn print_tree_diff(
    repo: &Path,
    map1: &BTreeMap<String, String>,
    map2: &BTreeMap<String, String>,
    name_only: bool,
    label_a: &str,
    label_b: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let all_keys: Vec<&String> = map1.keys().chain(map2.keys()).collect();
    let mut seen = std::collections::BTreeSet::new();

    for path in all_keys {
        if seen.contains(path) {
            continue;
        }
        seen.insert(path.clone());

        let content1 = map1
            .get(path)
            .and_then(|s| read_blob_content(repo, s))
            .unwrap_or_default();
        let content2 = map2
            .get(path)
            .and_then(|s| read_blob_content(repo, s))
            .unwrap_or_default();

        if content1 != content2 {
            if name_only {
                println!("{}", path);
            } else {
                print!(
                    "{}",
                    generate_unified_diff(
                        &format!("{}/{}", label_a, path),
                        &format!("{}/{}", label_b, path),
                        &content1,
                        &content2,
                    )
                );
            }
        }
    }
    Ok(())
}

fn print_tree_vs_working(
    repo: &Path,
    tree_map: &BTreeMap<String, String>,
    index: &Index,
    name_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let all_keys: Vec<&String> = tree_map.keys().chain(index.entries.keys()).collect();
    let mut seen = std::collections::BTreeSet::new();

    for path in all_keys {
        if seen.contains(path) {
            continue;
        }
        seen.insert(path.clone());

        let old_content = tree_map
            .get(path)
            .and_then(|s| read_blob_content(repo, s))
            .unwrap_or_default();
        let full_path = repo.join(path);
        let new_content = if full_path.exists() {
            fs::read(&full_path).unwrap_or_else(|e| {
                eprintln!("warning: failed to read '{}': {}", path, e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        if old_content != new_content {
            if name_only {
                println!("{}", path);
            } else {
                print!(
                    "{}",
                    generate_unified_diff(
                        &format!("a/{}", path),
                        &format!("b/{}", path),
                        &old_content,
                        &new_content,
                    )
                );
            }
        }
    }
    Ok(())
}

fn show_untracked_diff(repo: &Path, index: &Index) -> Result<(), Box<dyn std::error::Error>> {
    let mut untracked = Vec::new();
    let matcher = crate::core::ignore::IgnoreMatcher::load(repo, Path::new(""));
    collect_untracked(repo, repo, index, &matcher, &mut untracked)?;
    if untracked.is_empty() {
        return Ok(());
    }
    for path in &untracked {
        let full_path = repo.join(path);
        if let Ok(content) = fs::read(&full_path) {
            let a_path = format!("a/{}", path);
            let b_path = format!("b/{}", path);
            print!("{}", generate_unified_diff(&a_path, &b_path, &[], &content));
        }
    }
    Ok(())
}

fn build_head_tree_map(repo: &Path, head_sha1: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if let Ok((obj_type, content)) = storage::read_object(repo, head_sha1) {
        if obj_type == "commit" {
            if let Ok(commit) = Commit::deserialize(&crate::core::objects::format_object_data(
                "commit", &content,
            )) {
                let _ = collect_tree_recursive(repo, &commit.tree, "", &mut result);
            }
        }
    }
    result
}

/// 递归收集 tree 中所有文件路径 → SHA-1 映射。
fn collect_tree_recursive(
    repo: &Path,
    tree_sha1: &str,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (obj_type, content) = storage::read_object(repo, tree_sha1)?;
    if obj_type != "tree" {
        return Ok(());
    }
    let tree_data = crate::core::objects::format_object_data("tree", &content);
    let tree = Tree::deserialize(&tree_data)?;
    for entry in &tree.entries {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };
        if entry.mode == "40000" {
            collect_tree_recursive(repo, &entry.sha1, &path, out)?;
        } else {
            out.insert(path, entry.sha1.clone());
        }
    }
    Ok(())
}

fn read_blob_content(repo: &Path, sha1: &str) -> Option<Vec<u8>> {
    storage::read_object(repo, sha1)
        .ok()
        .map(|(_, content)| content)
}

fn generate_unified_diff(a_path: &str, b_path: &str, old: &[u8], new: &[u8]) -> String {
    let old_str = String::from_utf8_lossy(old);
    let new_str = String::from_utf8_lossy(new);
    let old_lines: Vec<&str> = old_str.lines().collect();
    let new_lines: Vec<&str> = new_str.lines().collect();

    let old_label = if old.is_empty() { "/dev/null" } else { a_path };
    let new_label = b_path;

    let file_header = if old.is_empty() {
        format!(
            "diff --git {} {}\nnew file mode 100644\n--- {}\n+++ {}\n",
            a_path, b_path, old_label, new_label
        )
    } else if new.is_empty() {
        format!(
            "diff --git {} {}\ndeleted file mode 100644\n--- {}\n+++ /dev/null\n",
            a_path, b_path, old_label
        )
    } else {
        format!(
            "diff --git {} {}\n--- {}\n+++ {}\n",
            a_path, b_path, old_label, new_label
        )
    };

    if old.is_empty() {
        let mut output = file_header;
        output.push_str("@@ -0,0 +1,");
        output.push_str(&new_lines.len().to_string());
        output.push_str(" @@\n");
        for line in &new_lines {
            output.push_str(&format!("+{}\n", line));
        }
        return output;
    }
    if new.is_empty() {
        let mut output = file_header;
        output.push_str("@@ -1,");
        output.push_str(&old_lines.len().to_string());
        output.push_str(" +0,0 @@\n");
        for line in &old_lines {
            output.push_str(&format!("-{}\n", line));
        }
        return output;
    }

    let changed = old_lines != new_lines;
    if !changed {
        return String::new();
    }

    let mut output = file_header;
    let hunk = compute_hunk(&old_lines, &new_lines);
    output.push_str(&format!(
        "@@ -1,{} +1,{} @@\n{}",
        old_lines.len(),
        new_lines.len(),
        hunk
    ));
    output
}

fn generate_unified_diff_deleted(a_path: &str, b_path: &str, old: &[u8]) -> String {
    generate_unified_diff(a_path, b_path, old, &[])
}

fn compute_hunk(old: &[&str], new: &[&str]) -> String {
    let m = old.len();
    let n = new.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..m {
        for j in 0..n {
            if old[i] == new[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            result.push(format!(" {}", old[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(format!("+{}", new[j - 1]));
            j -= 1;
        } else {
            result.push(format!("-{}", old[i - 1]));
            i -= 1;
        }
    }
    result.reverse();

    let mut output = String::new();
    for line in result {
        output.push_str(&line);
        output.push('\n');
    }
    output
}

fn collect_untracked(
    repo: &Path,
    current: &Path,
    index: &Index,
    matcher: &crate::core::ignore::IgnoreMatcher,
    untracked: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        if file_name == ".git" {
            continue;
        }

        let relative = path
            .strip_prefix(repo)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        let is_dir = path.is_dir();
        if matcher.is_ignored(&relative, is_dir) {
            continue;
        }

        if is_dir {
            collect_untracked(repo, &path, index, matcher, untracked)?;
        } else if path.is_file() && !index.entries.contains_key(&relative.to_string()) {
            untracked.push(relative.to_string());
        }
    }
    Ok(())
}

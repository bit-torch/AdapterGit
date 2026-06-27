use agit_core::objects::commit::Commit;
use agit_core::refs;
use agit_core::repo;
use agit_core::storage;

pub fn run(
    oneline: bool,
    max_count: Option<usize>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let start_points = if all {
        // 收集所有分支的 HEAD
        let mut refs_list = Vec::new();
        if let Ok(branches) = refs::list_branches(&repo_root) {
            for b in &branches {
                if let Ok(sha) = refs::read_ref(&repo_root, &format!("refs/heads/{}", b)) {
                    refs_list.push((b.clone(), sha));
                }
            }
        }
        if refs_list.is_empty() {
            match refs::read_head(&repo_root) {
                Ok(sha) => vec![("main".to_string(), sha)],
                Err(_) => return Ok(()),
            }
        } else {
            refs_list
        }
    } else {
        match refs::read_head(&repo_root) {
            Ok(sha) => vec![("".to_string(), sha)],
            Err(_) => {
                println!("fatal: your current branch does not have any commits yet");
                return Ok(());
            }
        }
    };

    // 对于 --all，简单地从每个 ref 头开始走链
    for (_branch, start_sha) in &start_points {
        let limit = max_count.unwrap_or(usize::MAX);
        let mut count = 0;
        let mut current_sha = start_sha.clone();

        loop {
            if count >= limit {
                break;
            }

            let (obj_type, content) = match storage::read_object(&repo_root, &current_sha) {
                Ok(v) => v,
                Err(_) => break,
            };

            if obj_type != "commit" {
                break;
            }

            let commit_data = agit_core::objects::format_object_data("commit", &content);
            let commit = match Commit::deserialize(&commit_data) {
                Ok(c) => c,
                Err(_) => break,
            };

            let short_hash = &current_sha[..7];

            if oneline {
                let msg_first_line = commit.message.lines().next().unwrap_or("");
                let author_name = commit
                    .author
                    .split('<')
                    .next()
                    .unwrap_or(&commit.author)
                    .trim();
                if all {
                    let branch = resolve_branch_label(&repo_root, &current_sha);
                    println!(
                        "{} ({}) {} — {}",
                        short_hash, branch, author_name, msg_first_line
                    );
                } else {
                    println!("{} {} — {}", short_hash, author_name, msg_first_line);
                }
            } else {
                println!(
                    "{}",
                    crate::output::colorize(&format!("commit {}", short_hash), "33")
                );
                if commit.parents.len() > 1 {
                    let parents: Vec<&str> = commit.parents.iter().map(|p| &p[..7]).collect();
                    println!("Merge: {}", parents.join(" "));
                }
                println!(
                    "Author: {}",
                    commit
                        .author
                        .split('<')
                        .next()
                        .unwrap_or(&commit.author)
                        .trim()
                );
                println!();
                println!("    {}", commit.message.lines().next().unwrap_or(""));
                println!();
            }

            count += 1;
            if commit.parents.is_empty() {
                break;
            }
            current_sha = commit.parents[0].clone();
        }
    }

    Ok(())
}

fn resolve_branch_label(repo: &std::path::Path, sha: &str) -> String {
    if let Ok(branches) = refs::list_branches(repo) {
        for b in &branches {
            if let Ok(ref_sha) = refs::read_ref(repo, &format!("refs/heads/{}", b)) {
                if ref_sha == sha {
                    return b.clone();
                }
            }
        }
    }
    "detached".to_string()
}

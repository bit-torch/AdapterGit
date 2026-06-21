use crate::ai;
use crate::config;
use crate::core::index::Index;
use crate::core::objects::blob::Blob;
use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;

pub fn run(message: Option<String>, ai_flag: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let index = Index::load(&repo_root)?;
    if index.entries.is_empty() {
        println!("nothing to commit (no staged changes)");
        return Ok(());
    }

    let cfg = config::load();
    let (timestamp, time_str) = repo::get_current_timestamp();
    let author = format!(
        "{} <{}> {} {}",
        cfg.user_name, cfg.user_email, timestamp, time_str
    );
    let committer = author.clone();

    let is_ai = ai_flag || ai::is_ai_mode();

    // 检查是否处于合并状态（.git/MERGE_HEAD 存在）
    let git_dir = repo_root.join(".git");
    let merge_head_path = git_dir.join("MERGE_HEAD");
    let merge_msg_path = git_dir.join("MERGE_MSG");
    let in_merge = merge_head_path.exists();

    // 确定最终 commit message
    let mut msg = if let Some(ref m) = message {
        m.clone()
    } else if in_merge {
        // 合并状态且无 -m：使用 MERGE_MSG
        std::fs::read_to_string(&merge_msg_path)
            .unwrap_or_else(|_| "Merge".to_string())
            .trim()
            .to_string()
    } else if is_ai {
        // AI 模式无 -m：尝试从 LLM 生成 commit message
        generate_ai_message(&repo_root, &index)
    } else {
        "Update".to_string()
    };

    if is_ai && !msg.starts_with("[AI-committed]") {
        msg = format!("{}{}", ai::ai_commit_marker(), msg);
    }
    if !msg.ends_with('\n') {
        msg.push('\n');
    }

    let mut tree = Tree::new();
    for entry in index.entries.values() {
        tree.add_entry(&entry.mode, &entry.path, &entry.sha1);
    }
    let tree_sha1 = tree.hash();
    storage::write_object(&repo_root, "tree", &tree.serialize_raw())?;

    let mut commit = Commit::new(&tree_sha1, &author, &committer, &msg);

    if in_merge {
        let head_sha1 = refs::read_head(&repo_root)?;
        let merge_head_sha = std::fs::read_to_string(&merge_head_path)?
            .trim()
            .to_string();
        commit.add_parent(&head_sha1);
        commit.add_parent(&merge_head_sha);
    } else if let Ok(head_sha1) = refs::read_head(&repo_root) {
        commit.add_parent(&head_sha1);
    }

    let commit_sha1 = commit.hash();
    storage::write_object(&repo_root, "commit", &commit.serialize_raw())?;

    let head_path = repo_root.join(".git").join("HEAD");
    let head_content = if head_path.exists() {
        std::fs::read_to_string(&head_path)?
    } else {
        // 新仓库，HEAD 尚未创建 → 初始化为 main
        String::new()
    };
    let head_trimmed = head_content.trim();
    let branch_ref = if let Some(ref_path) = head_trimmed.strip_prefix("ref: ") {
        ref_path.trim().to_string()
    } else if head_trimmed.len() == 40 && head_trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        // 允许 rebase / cherry-pick 进行中的分离 HEAD 提交
        let in_rebase = git_dir.join("REBASE_TODO").exists();
        let in_cherry_pick = git_dir.join("CHERRY_PICK_TODO").exists();
        if in_rebase || in_cherry_pick {
            // 直接写入 HEAD（不更新分支引用）
            refs::write_head(&repo_root, &commit_sha1)?;
            let parent_info = if commit.parents.is_empty() {
                " (root-commit)"
            } else {
                ""
            };
            println!("[{:.7}]{}{}", commit_sha1, parent_info, msg.trim_end());
            return Ok(());
        }
        return Err("You are in 'detached HEAD' state. Please create a branch first.".into());
    } else if head_trimmed.is_empty() {
        // HEAD 为空：初始化默认分支 main（首次 commit）
        let default_branch = "ref: refs/heads/main";
        refs::write_head(&repo_root, default_branch)?;
        "refs/heads/main".to_string()
    } else {
        return Err(format!("Unexpected HEAD content: '{}'", head_trimmed).into());
    };
    refs::write_ref(&repo_root, &branch_ref, &commit_sha1)?;

    // 清理合并状态文件
    if in_merge {
        let _ = std::fs::remove_file(&merge_head_path);
        let _ = std::fs::remove_file(&merge_msg_path);
        let _ = std::fs::remove_file(git_dir.join("MERGE_MODE"));
        let _ = std::fs::remove_file(git_dir.join("ORIG_HEAD"));
    }

    let branch_name = branch_ref.strip_prefix("refs/heads/").unwrap_or("main");

    let parent_info = if commit.parents.is_empty() {
        " (root-commit)".to_string()
    } else {
        String::new()
    };
    println!(
        "[{} {:.7}]{} {}",
        if is_ai { "ai" } else { branch_name },
        commit_sha1,
        parent_info,
        msg.trim_end()
    );

    Ok(())
}

/// 使用 AI 从暂存区 diff 生成 commit message。
#[cfg(feature = "ai")]
fn generate_ai_message(repo_root: &std::path::Path, index: &Index) -> String {
    let summary = build_staged_summary(repo_root, index);
    if let Some(config) = ai::llm::LlmConfig::from_env() {
        println!("[AI] Generating commit message via {}...", config.model);
        match ai::llm::generate_commit_message(&config, &summary, None) {
            Ok(msg) => {
                println!("[AI] Generated: {}", msg);
                return msg;
            }
            Err(e) => {
                eprintln!("[AI] LLM call failed: {}", e);
            }
        }
    } else {
        println!("[AI] AGIT_LLM_API_KEY not set, using basic template.");
    }
    // 回退：基于文件列表生成简单消息
    let paths: Vec<&str> = index.entries.keys().map(|s| s.as_str()).collect();
    if paths.len() == 1 {
        format!("Update {}", paths[0])
    } else {
        format!("Update {} files", paths.len())
    }
}

#[cfg(not(feature = "ai"))]
fn generate_ai_message(_repo_root: &std::path::Path, _index: &Index) -> String {
    println!("[AI] LLM support not compiled (enable 'ai' feature).");
    "AI Update".to_string()
}

/// 构建暂存区文件变更摘要（供 AI 使用）。
fn build_staged_summary(repo_root: &std::path::Path, index: &Index) -> String {
    let mut lines = vec!["Staged changes:".to_string()];

    for (path, entry) in index.entries.iter() {
        let full_path = repo_root.join(path);
        let status = if full_path.exists() {
            match std::fs::read(&full_path) {
                Ok(content) => {
                    let blob = Blob::new(content);
                    if blob.hash() == entry.sha1 {
                        "modified".to_string()
                    } else {
                        "modified (staged != working)".to_string()
                    }
                }
                Err(_) => "modified".to_string(),
            }
        } else {
            "deleted".to_string()
        };
        lines.push(format!("  {} ({})", path, status));
    }

    lines.join("\n")
}

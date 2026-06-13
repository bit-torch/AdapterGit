//! tag 命令：创建、列出、删除标签。

use crate::core::objects::tag::Tag;
use crate::core::{refs, repo, storage};

pub fn run_create(
    name: &str,
    message: Option<&str>,
    commit: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let target_sha = match commit {
        Some(c) => {
            if c == "HEAD" {
                refs::read_head(&repo_root)?
            } else if let Ok(sha) =
                refs::read_ref(&repo_root, &format!("refs/heads/{}", c))
            {
                sha
            } else if let Ok(sha) =
                refs::read_ref(&repo_root, &format!("refs/tags/{}", c))
            {
                sha
            } else {
                c.to_string()
            }
        }
        None => refs::read_head(&repo_root)?,
    };

    if let Some(msg) = message {
        // Annotated tag
        let config = crate::config::load();
        let (timestamp, time_str) = repo::get_current_timestamp();
        let tagger = format!(
            "{} <{}> {} {}",
            config.user_name, config.user_email, timestamp, time_str
        );

        let (obj_type, _) = storage::read_object(&repo_root, &target_sha)?;
        let tag = Tag::new(&target_sha, &obj_type, name, &tagger, msg);
        let tag_sha = tag.hash();
        storage::write_object(&repo_root, "tag", &tag.serialize_raw())?;
        refs::create_tag(&repo_root, name, &tag_sha)?;
    } else {
        // Lightweight tag
        refs::create_tag(&repo_root, name, &target_sha)?;
    }

    println!("Created tag '{}'", name);
    Ok(())
}

pub fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let tags = refs::list_tags(&repo_root)?;
    if tags.is_empty() {
        println!("No tags found.");
        return Ok(());
    }
    for tag_name in &tags {
        if let Ok(tag_sha) = refs::read_ref(&repo_root, &format!("refs/tags/{}", tag_name)) {
            if let Ok((obj_type, _)) = storage::read_object(&repo_root, &tag_sha) {
                if obj_type == "tag" {
                    println!("{} (annotated)", tag_name);
                    continue;
                }
            }
        }
        println!("{}", tag_name);
    }
    Ok(())
}

pub fn run_delete(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    refs::delete_tag(&repo_root, name)?;
    println!("Deleted tag '{}'", name);
    Ok(())
}

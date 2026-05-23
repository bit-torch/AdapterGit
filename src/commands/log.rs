use crate::core::objects::commit::Commit;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let head_sha1 = match refs::read_head(&repo_root) {
        Ok(sha1) => sha1,
        Err(_) => {
            println!(
                "fatal: your current branch 'main' does not have any commits yet"
            );
            return Ok(());
        }
    };

    let mut current_sha1 = head_sha1;

    loop {
        let (obj_type, content) = match storage::read_object(&repo_root, &current_sha1) {
            Ok(v) => v,
            Err(_) => break,
        };

        if obj_type != "commit" {
            break;
        }

        let commit_data = crate::core::objects::format_object_data("commit", &content);
        let commit = match Commit::deserialize(&commit_data) {
            Ok(c) => c,
            Err(_) => break,
        };

        let short_hash = &current_sha1[..7];

        let author_name = commit
            .author
            .split('<')
            .next()
            .unwrap_or(&commit.author)
            .trim();

        let msg_first_line = commit.message.lines().next().unwrap_or("");

        println!(
            "{}",
            crate::output::colorize(
                &format!("commit {}", short_hash),
                "33"
            )
        );
        if commit.parents.len() > 1 {
            let parents: Vec<&str> = commit.parents.iter().map(|p| &p[..7]).collect();
            println!("Merge: {}", parents.join(" "));
        }
        println!("Author: {}", author_name);
        println!();
        println!("    {}", msg_first_line);
        println!();

        if commit.parents.is_empty() {
            break;
        }
        current_sha1 = commit.parents[0].clone();
    }

    Ok(())
}

use crate::core::objects::commit::Commit;
use crate::core::objects::tree::Tree;
use crate::core::refs;
use crate::core::repo;
use crate::core::storage;

pub fn run(object: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;

    let sha1 = resolve_ref(&repo_root, object)?;
    let (obj_type, content) = storage::read_object(&repo_root, &sha1)?;

    match obj_type.as_str() {
        "commit" => show_commit(&content),
        "tree" => show_tree(&content),
        "blob" => show_blob(&content),
        _ => {
            print!("{}", String::from_utf8_lossy(&content));
            Ok(())
        }
    }
}

fn resolve_ref(repo: &std::path::Path, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    if name.len() == 40 && name.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(name.to_string());
    }
    let ref_name = format!("refs/heads/{}", name);
    match refs::read_ref(repo, &ref_name) {
        Ok(sha1) => Ok(sha1),
        Err(_) => Err(format!("ambiguous argument '{}': unknown revision", name).into()),
    }
}

fn show_commit(content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let commit_data = crate::core::objects::format_object_data("commit", content);
    let commit = Commit::deserialize(&commit_data)?;

    println!(
        "{}",
        crate::output::colorize(&format!("commit {}", commit.hash()), "33")
    );
    if commit.parents.len() > 1 {
        let shorts: Vec<&str> = commit.parents.iter().map(|p| &p[..7]).collect();
        println!("Merge: {}", shorts.join(" "));
    }
    println!("Author: {}", commit.author);
    println!("Date:   {}", commit.committer);
    println!();
    println!("{}", commit.message);

    Ok(())
}

fn show_tree(content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let tree_data = crate::core::objects::format_object_data("tree", content);
    let tree = Tree::deserialize(&tree_data)?;

    for entry in &tree.entries {
        let type_str = if entry.mode == "40000" {
            "tree"
        } else {
            "blob"
        };
        println!("{} {} {}\t{}", entry.mode, type_str, entry.sha1, entry.name);
    }

    Ok(())
}

fn show_blob(content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    print!("{}", String::from_utf8_lossy(content));
    Ok(())
}

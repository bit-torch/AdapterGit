use agit_core::objects::tree::Tree;
use agit_core::repo;
use agit_core::storage;

pub fn run(tree_sha1: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let (obj_type, content) = storage::read_object(&repo_root, tree_sha1)?;

    if obj_type != "tree" {
        eprintln!("fatal: not a tree object");
        return Ok(());
    }

    let tree_data = agit_core::objects::format_object_data("tree", &content);
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

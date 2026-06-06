use crate::core::objects::tree::Tree;
use crate::core::repo;
use crate::core::storage;

pub fn run(object: &str, show_type: bool, pretty_print: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let (obj_type, content) = storage::read_object(&repo_root, object)?;

    if show_type {
        println!("{}", obj_type);
        return Ok(());
    }

    if pretty_print {
        match obj_type.as_str() {
            "blob" => {
                print!("{}", String::from_utf8_lossy(&content));
            }
            "tree" => {
                let tree_data = crate::core::objects::format_object_data("tree", &content);
                let tree = Tree::deserialize(&tree_data)?;
                for entry in &tree.entries {
                    let type_str = if entry.mode == "40000" {
                        "tree"
                    } else {
                        "blob"
                    };
                    println!("{} {} {}\t{}", entry.mode, type_str, entry.sha1, entry.name);
                }
            }
            "commit" => {
                print!("{}", String::from_utf8_lossy(&content));
            }
            _ => {
                print!("{}", String::from_utf8_lossy(&content));
            }
        }
        return Ok(());
    }

    print!("{}", String::from_utf8_lossy(&content));
    Ok(())
}

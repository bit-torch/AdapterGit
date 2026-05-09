use crate::core::hash::hash_git_object;

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub sha1: String,
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, mode: &str, name: &str, sha1: &str) {
        self.entries.push(TreeEntry {
            mode: mode.to_string(),
            name: name.to_string(),
            sha1: sha1.to_string(),
        });
    }

    pub fn hash(&self) -> String {
        let data = self.serialize_raw();
        hash_git_object("tree", &data)
    }

    fn serialize_raw(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in &self.entries {
            data.extend_from_slice(format!("{} {}\0", entry.mode, entry.name).as_bytes());
            let sha1_bytes = hex_to_bytes(&entry.sha1);
            data.extend_from_slice(&sha1_bytes);
        }
        data
    }

    pub fn serialize(&self) -> Vec<u8> {
        let raw = self.serialize_raw();
        let header = format!("tree {}\0", raw.len());
        let mut data = Vec::with_capacity(header.len() + raw.len());
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(&raw);
        data
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let null_pos = data
            .iter()
            .position(|&b| b == 0)
            .ok_or("Invalid tree: no null byte found")?;
        let header = std::str::from_utf8(&data[..null_pos])?;
        let parts: Vec<&str> = header.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0] != "tree" {
            return Err("Invalid tree header: expected 'tree <size>'".into());
        }
        let _expected_size: usize = parts[1].parse()?;

        let raw = &data[null_pos + 1..];
        let mut entries = Vec::new();
        let mut pos = 0;

        while pos < raw.len() {
            let entry_null = raw[pos..]
                .iter()
                .position(|&b| b == 0)
                .ok_or("Invalid tree entry: missing null byte")?;
            let entry_header = std::str::from_utf8(&raw[pos..pos + entry_null])?;
            let entry_parts: Vec<&str> = entry_header.splitn(2, ' ').collect();
            if entry_parts.len() != 2 {
                return Err("Invalid tree entry header".into());
            }
            let mode = entry_parts[0].to_string();
            let name = entry_parts[1].to_string();

            pos += entry_null + 1;
            if pos + 20 > raw.len() {
                return Err("Invalid tree entry: not enough bytes for SHA-1".into());
            }
            let sha1_bytes: &[u8] = &raw[pos..pos + 20];
            let sha1 = bytes_to_hex(sha1_bytes);
            pos += 20;

            entries.push(TreeEntry { mode, name, sha1 });
        }

        Ok(Tree { entries })
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0))
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_new_empty() {
        let tree = Tree::new();
        assert!(tree.entries.is_empty());
    }

    #[test]
    fn test_tree_add_entry() {
        let mut tree = Tree::new();
        tree.add_entry(
            "100644",
            "hello.txt",
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
        );
        assert_eq!(tree.entries.len(), 1);
        assert_eq!(tree.entries[0].mode, "100644");
        assert_eq!(tree.entries[0].name, "hello.txt");
    }

    #[test]
    fn test_tree_serialize_deserialize() {
        let mut tree = Tree::new();
        tree.add_entry(
            "100644",
            "hello.txt",
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
        );
        tree.add_entry(
            "100644",
            "world.txt",
            "9daeafb9864cf43055ae93beb0afd6c7d144bfa4",
        );

        let data = tree.serialize();
        let deserialized = Tree::deserialize(&data).unwrap();

        assert_eq!(deserialized.entries.len(), 2);
        assert_eq!(deserialized.entries[0].mode, "100644");
        assert_eq!(deserialized.entries[0].name, "hello.txt");
        assert_eq!(
            deserialized.entries[0].sha1,
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad"
        );
        assert_eq!(deserialized.entries[1].name, "world.txt");
    }

    #[test]
    fn test_tree_hash_consistent() {
        let mut tree = Tree::new();
        tree.add_entry(
            "100644",
            "test.txt",
            "9daeafb9864cf43055ae93beb0afd6c7d144bfa4",
        );

        let hash = tree.hash();
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tree_with_subdirectory() {
        let mut tree = Tree::new();
        tree.add_entry(
            "40000",
            "subdir",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        let data = tree.serialize();
        let deserialized = Tree::deserialize(&data).unwrap();
        assert_eq!(deserialized.entries.len(), 1);
        assert_eq!(deserialized.entries[0].mode, "40000");
        assert_eq!(deserialized.entries[0].name, "subdir");
    }
}

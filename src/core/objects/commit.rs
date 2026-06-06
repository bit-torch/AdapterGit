use crate::core::hash::hash_git_object;

pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

impl Commit {
    pub fn new(tree: &str, author: &str, committer: &str, message: &str) -> Self {
        Commit {
            tree: tree.to_string(),
            parents: Vec::new(),
            author: author.to_string(),
            committer: committer.to_string(),
            message: message.to_string(),
        }
    }

    pub fn add_parent(&mut self, parent: &str) {
        self.parents.push(parent.to_string());
    }

    pub fn hash(&self) -> String {
        let data = self.serialize_raw();
        hash_git_object("commit", &data)
    }

    pub fn serialize_raw(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(format!("tree {}\n", self.tree).as_bytes());

        for parent in &self.parents {
            data.extend_from_slice(format!("parent {}\n", parent).as_bytes());
        }

        data.extend_from_slice(format!("author {}\n", self.author).as_bytes());
        data.extend_from_slice(format!("committer {}\n", self.committer).as_bytes());
        data.extend_from_slice(b"\n");
        data.extend_from_slice(self.message.as_bytes());

        if !self.message.ends_with('\n') {
            data.push(b'\n');
        }

        data
    }

    #[allow(dead_code)]
    pub fn serialize(&self) -> Vec<u8> {
        let raw = self.serialize_raw();
        let header = format!("commit {}\0", raw.len());
        let mut data = Vec::with_capacity(header.len() + raw.len());
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(&raw);
        data
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let null_pos = data
            .iter()
            .position(|&b| b == 0)
            .ok_or("Invalid commit: no null byte found")?;
        let header = std::str::from_utf8(&data[..null_pos])?;
        let parts: Vec<&str> = header.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0] != "commit" {
            return Err("Invalid commit header: expected 'commit <size>'".into());
        }
        let _expected_size: usize = parts[1].parse()?;

        let raw = &data[null_pos + 1..];
        let raw_str = std::str::from_utf8(raw)?;
        let mut header_end = 0;

        let mut tree = String::new();
        let mut parents = Vec::new();
        let mut author = String::new();
        let mut committer = String::new();

        for line in raw_str.split('\n') {
            header_end += line.len() + 1;
            if let Some(stripped) = line.strip_prefix("tree ") {
                tree = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("parent ") {
                parents.push(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("author ") {
                author = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("committer ") {
                committer = stripped.to_string();
            } else if line.is_empty() {
                break;
            }
        }

        let message = if header_end < raw.len() {
            std::str::from_utf8(&raw[header_end..])?.to_string()
        } else {
            String::new()
        };

        Ok(Commit {
            tree,
            parents,
            author,
            committer,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_new() {
        let commit = Commit::new(
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
            "Test User <test@example.com> 1234567890 +0000",
            "Test User <test@example.com> 1234567890 +0000",
            "Initial commit",
        );
        assert_eq!(commit.tree, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
        assert!(commit.parents.is_empty());
        assert_eq!(commit.message, "Initial commit");
    }

    #[test]
    fn test_commit_hash() {
        let commit = Commit::new(
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
            "A U Thor <author@example.com> 1000000000 +0000",
            "C O Mitter <committer@example.com> 1000000000 +0000",
            "Initial commit\n",
        );
        let hash = commit.hash();
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_commit_serialize_deserialize() {
        let mut commit = Commit::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "Author <author@test.com> 1000000000 +0800",
            "Committer <committer@test.com> 1000000000 +0800",
            "This is a commit message\nwith multiple lines\n",
        );
        commit.add_parent("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        let data = commit.serialize();
        let deserialized = Commit::deserialize(&data).unwrap();

        assert_eq!(deserialized.tree, commit.tree);
        assert_eq!(deserialized.parents, commit.parents);
        assert_eq!(deserialized.author, commit.author);
        assert_eq!(deserialized.committer, commit.committer);
        assert_eq!(deserialized.message, commit.message);
    }

    #[test]
    fn test_commit_hash_consistent() {
        let commit = Commit::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "A <a@a.com> 1 +0000",
            "C <c@c.com> 1 +0000",
            "test\n",
        );
        let deserialized = Commit::deserialize(&commit.serialize()).unwrap();
        assert_eq!(commit.hash(), deserialized.hash());
    }

    #[test]
    fn test_commit_multiple_parents() {
        let mut commit = Commit::new(
            "treehash",
            "author <a@a.com> 1 +0",
            "committer <c@c.com> 1 +0",
            "Merge commit\n",
        );
        commit.add_parent("parent1");
        commit.add_parent("parent2");

        let data = commit.serialize();
        let deserialized = Commit::deserialize(&data).unwrap();
        assert_eq!(deserialized.parents.len(), 2);
    }
}

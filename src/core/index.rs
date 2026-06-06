use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub struct Index {
    pub entries: BTreeMap<String, IndexEntry>,
}

pub struct IndexEntry {
    pub mode: String,
    pub sha1: String,
    pub flags: u16,
    pub path: String,
}

impl Index {
    pub fn new() -> Self {
        Index {
            entries: BTreeMap::new(),
        }
    }

    pub fn load(repo: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let index_path = repo.join(".git").join("index");
        if !index_path.exists() {
            return Ok(Index::new());
        }
        let data = fs::read(&index_path)?;
        Index::deserialize(&data)
    }

    pub fn save(&self, repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = repo.join(".git").join("index");
        let data = self.serialize();
        fs::write(&index_path, &data)?;
        Ok(())
    }

    pub fn add_entry(&mut self, mode: &str, sha1: &str, path: &str) {
        self.entries.insert(
            path.to_string(),
            IndexEntry {
                mode: mode.to_string(),
                sha1: sha1.to_string(),
                flags: path.len() as u16,
                path: path.to_string(),
            },
        );
    }

    #[allow(dead_code)]
    pub fn remove_entry(&mut self, path: &str) {
        self.entries.remove(path);
    }

    #[allow(dead_code)]
    pub fn get_entry(&self, path: &str) -> Option<&IndexEntry> {
        self.entries.get(path)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let entry_count = self.entries.len() as u32;
        let header = b"DIRC";
        data.extend_from_slice(header);
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&entry_count.to_be_bytes());

        for entry in self.entries.values() {
            let ctime = 0u32.to_be_bytes();
            let mtime = 0u32.to_be_bytes();
            data.extend_from_slice(&ctime);
            data.extend_from_slice(&ctime);
            data.extend_from_slice(&mtime);
            data.extend_from_slice(&mtime);
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&0u32.to_be_bytes());
            data.extend_from_slice(&0u32.to_be_bytes());

            let sha1_bytes = hex_to_bytes(&entry.sha1);
            data.extend_from_slice(&sha1_bytes);

            data.extend_from_slice(&entry.flags.to_be_bytes());

            let padded_mode = format!("{:0>6}", entry.mode);
            data.extend_from_slice(padded_mode.as_bytes());

            data.push(b' ');

            data.extend_from_slice(entry.path.as_bytes());
            data.push(0);

            let current_len = data.len();
            let padding_needed = (8 - (current_len % 8)) % 8;
            data.extend(std::iter::repeat_n(0, padding_needed));
        }

        let sha1 = crate::core::hash::hash_bytes(&data);
        let sha1_bytes = hex_to_bytes(&sha1);
        data.extend_from_slice(&sha1_bytes);

        data
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.len() < 12 {
            return Err("Index too short".into());
        }
        if &data[..4] != b"DIRC" {
            return Err("Invalid index signature".into());
        }
        let _version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let entry_count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;

        let mut entries = BTreeMap::new();
        let mut pos = 12;

        for _ in 0..entry_count {
            if pos + 62 > data.len() {
                return Err("Index entry truncated".into());
            }

            pos += 40;

            let sha1_bytes: &[u8] = &data[pos..pos + 20];
            let sha1 = bytes_to_hex(sha1_bytes);
            pos += 20;

            let flags = u16::from_be_bytes([data[pos], data[pos + 1]]);
            pos += 2;

            let mode = std::str::from_utf8(&data[pos..pos + 6])?.to_string();
            pos += 7;

            let name_start = pos;
            while pos < data.len() && data[pos] != 0 {
                pos += 1;
            }
            if pos >= data.len() {
                return Err("Index entry: unterminated path".into());
            }
            let path = std::str::from_utf8(&data[name_start..pos])?.to_string();
            pos += 1;

            let current_len = pos;
            let padding_needed = (8 - (current_len % 8)) % 8;
            pos += padding_needed;

            entries.insert(
                path.clone(),
                IndexEntry {
                    mode,
                    sha1,
                    flags: flags & 0x0FFF,
                    path,
                },
            );
        }

        Ok(Index { entries })
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
    fn test_index_new_empty() {
        let index = Index::new();
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_add_and_get_entry() {
        let mut index = Index::new();
        index.add_entry(
            "100644",
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
            "hello.txt",
        );
        assert_eq!(index.entries.len(), 1);

        let entry = index.get_entry("hello.txt").unwrap();
        assert_eq!(entry.mode, "100644");
        assert_eq!(entry.sha1, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn test_index_remove_entry() {
        let mut index = Index::new();
        index.add_entry("100644", "abc123", "file.txt");
        assert_eq!(index.entries.len(), 1);

        index.remove_entry("file.txt");
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_index_serialize_deserialize() {
        let mut index = Index::new();
        index.add_entry(
            "100644",
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
            "hello.txt",
        );
        index.add_entry(
            "100644",
            "9daeafb9864cf43055ae93beb0afd6c7d144bfa4",
            "world.txt",
        );

        let data = index.serialize();
        let deserialized = Index::deserialize(&data).unwrap();

        assert_eq!(deserialized.entries.len(), 2);
        assert!(deserialized.get_entry("hello.txt").is_some());
        assert!(deserialized.get_entry("world.txt").is_some());
    }
}

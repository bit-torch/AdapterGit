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

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
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
        let data = self.serialize()?;
        crate::utils::atomic_write(&index_path, &data)?;
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

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();

        let entry_count = self.entries.len() as u32;
        data.extend_from_slice(b"DIRC");
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&entry_count.to_be_bytes());

        for entry in self.entries.values() {
            let entry_start = data.len(); // 记录条目起始偏移（用于对齐）

            // 统计区 40 字节，mode 写实际值（0o100644 → 0x000081A4）
            let mode_stat: u32 = u32::from_str_radix(&entry.mode, 8).map_err(|e| {
                format!("Invalid mode '{}' for '{}': {}", entry.mode, entry.path, e)
            })?;
            data.extend_from_slice(&0u32.to_be_bytes()); // ctime sec
            data.extend_from_slice(&0u32.to_be_bytes()); // ctime nsec
            data.extend_from_slice(&0u32.to_be_bytes()); // mtime sec
            data.extend_from_slice(&0u32.to_be_bytes()); // mtime nsec
            data.extend_from_slice(&0u32.to_be_bytes()); // dev
            data.extend_from_slice(&0u32.to_be_bytes()); // ino
            data.extend_from_slice(&mode_stat.to_be_bytes()); // mode
            data.extend_from_slice(&0u32.to_be_bytes()); // uid
            data.extend_from_slice(&0u32.to_be_bytes()); // gid
            data.extend_from_slice(&0u32.to_be_bytes()); // file size

            // SHA-1 (20 bytes binary)
            data.extend_from_slice(
                &hex_to_bytes(&entry.sha1)
                    .map_err(|e| format!("Invalid SHA-1 hex for '{}': {}", entry.path, e))?,
            );

            // Flags (u16BE) — 仅路径长度，无扩展位
            let flags: u16 = entry.flags & 0x0FFF;
            data.extend_from_slice(&flags.to_be_bytes());

            // 路径 (NUL 结尾)
            data.extend_from_slice(entry.path.as_bytes());
            data.push(0);

            // 条目对齐到 8 字节边界（相对条目自身起始）
            let entry_len = data.len() - entry_start;
            let pad = (8 - (entry_len % 8)) % 8;
            data.extend(std::iter::repeat_n(0, pad));
        }

        // SHA-1 校验和
        let sha1 = crate::hash::hash_bytes(&data);
        data.extend_from_slice(&hex_to_bytes(&sha1)?);

        Ok(data)
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

            let entry_start = pos;

            // 从统计区第 7 个字段读取 mode (offset 24..28 从 entry 开始)
            let stat_mode = u32::from_be_bytes([
                data[pos + 24],
                data[pos + 25],
                data[pos + 26],
                data[pos + 27],
            ]);
            let mode = format_mode_from_u32(stat_mode);

            pos += 40; // 跳过全部统计区

            let sha1_bytes: &[u8] = &data[pos..pos + 20];
            let sha1 = bytes_to_hex(sha1_bytes);
            pos += 20;

            let flags = u16::from_be_bytes([data[pos], data[pos + 1]]);
            pos += 2;

            // 路径名紧随 flags
            let name_start = pos;
            while pos < data.len() && data[pos] != 0 {
                pos += 1;
            }
            if pos >= data.len() {
                return Err("Index entry: unterminated path".into());
            }
            let path = std::str::from_utf8(&data[name_start..pos])?.to_string();
            pos += 1;

            // 条目对齐到 8 字节边界（相对条目自身起始）
            let entry_len = pos - entry_start;
            let pad = (8 - (entry_len % 8)) % 8;
            pos += pad;

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

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err(format!("hex string has odd length: {}", hex.len()));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("invalid hex at pos {}: {}", i, e))
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 将 mode 字符串（八进制，如 "100644"）转为 u32（高位 16-bit 为零）。
#[allow(dead_code)]
fn parse_mode_to_u32(mode: &str) -> u32 {
    u32::from_str_radix(mode, 8).unwrap_or(0o100644)
}

/// 将 u32 统计区 mode 值转为八进制字符串（取低 16 位）。
fn format_mode_from_u32(val: u32) -> String {
    format!("{:o}", val & 0xFFFF)
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

        let data = index.serialize().unwrap();
        let deserialized = Index::deserialize(&data).unwrap();

        assert_eq!(deserialized.entries.len(), 2);
        assert!(deserialized.get_entry("hello.txt").is_some());
        assert!(deserialized.get_entry("world.txt").is_some());
    }

    #[test]
    fn test_index_git_compat() {
        // 验证生成的 index 与原生 Git 兼容：
        // 1. 统计区 mode 写入实际值（非 0）
        // 2. flags 仅含路径长度，无扩展位
        // 3. 路径紧随 flags（无扩展 mode 区）
        let mut index = Index::new();
        index.add_entry(
            "100644",
            "3b18e512dba79e4c8300dd08aeb37f8e728b8dad",
            "hello.txt",
        );
        let data = index.serialize().unwrap();

        // 统计区 mode 在 entry 偏移 24 (第 7 个 u32)
        let stat_mode =
            u32::from_be_bytes([data[12 + 24], data[12 + 25], data[12 + 26], data[12 + 27]]);
        assert_eq!(stat_mode, 0o100644, "stat mode should be 0o100644");

        // flags 在 entry 偏移 60 (40 stat + 20 SHA1)
        let flags = u16::from_be_bytes([data[72], data[73]]);
        assert_eq!(flags & 0xFFF, 9, "flags low 12 bits = path length");
        assert_eq!(
            flags, 9,
            "flags should only contain path length, no extended bits"
        );

        // 路径紧随 flags (偏移 74)，无扩展 mode 区
        assert_eq!(&data[74..83], b"hello.txt");
        assert_eq!(data[83], 0u8, "NUL terminator");
    }
}

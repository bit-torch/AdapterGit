use crate::core::hash::hash_git_object;

/// Git 注释标签（annotated tag）对象。
///
/// 格式:
/// ```text
/// object <sha1>
/// type <object_type>
/// tag <tag_name>
/// tagger <tagger_info>
///
/// <message>
/// ```
pub struct Tag {
    /// 标签指向的对象 SHA-1（通常是一个 commit）
    pub object: String,
    /// 对象类型（通常为 "commit"）
    pub object_type: String,
    /// 标签名称
    pub tag_name: String,
    /// 标签创建者信息，格式: `name <email> timestamp timezone`
    pub tagger: String,
    /// 标签消息（可选）
    pub message: String,
}

impl Tag {
    pub fn new(object: &str, object_type: &str, tag_name: &str, tagger: &str, message: &str) -> Self {
        Tag {
            object: object.to_string(),
            object_type: object_type.to_string(),
            tag_name: tag_name.to_string(),
            tagger: tagger.to_string(),
            message: message.to_string(),
        }
    }

    pub fn hash(&self) -> String {
        let data = self.serialize_raw();
        hash_git_object("tag", &data)
    }

    pub fn serialize_raw(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(format!("object {}\n", self.object).as_bytes());
        data.extend_from_slice(format!("type {}\n", self.object_type).as_bytes());
        data.extend_from_slice(format!("tag {}\n", self.tag_name).as_bytes());
        data.extend_from_slice(format!("tagger {}\n", self.tagger).as_bytes());
        data.extend_from_slice(b"\n");

        if !self.message.is_empty() {
            data.extend_from_slice(self.message.as_bytes());
            if !self.message.ends_with('\n') {
                data.push(b'\n');
            }
        }

        data
    }

    pub fn serialize(&self) -> Vec<u8> {
        let raw = self.serialize_raw();
        super::format_object_data("tag", &raw)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let null_pos = data
            .iter()
            .position(|&b| b == 0)
            .ok_or("Invalid tag: no null byte found")?;
        let header = std::str::from_utf8(&data[..null_pos])?;
        let parts: Vec<&str> = header.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0] != "tag" {
            return Err("Invalid tag header: expected 'tag <size>'".into());
        }

        let raw = &data[null_pos + 1..];
        let raw_str = std::str::from_utf8(raw)?;

        let mut object = String::new();
        let mut object_type = String::new();
        let mut tag_name = String::new();
        let mut tagger = String::new();
        let mut header_end = 0;

        for line in raw_str.split('\n') {
            header_end += line.len() + 1;
            if let Some(stripped) = line.strip_prefix("object ") {
                object = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("type ") {
                object_type = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("tag ") {
                tag_name = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("tagger ") {
                tagger = stripped.to_string();
            } else if line.is_empty() {
                break;
            }
        }

        let message = if header_end < raw.len() {
            std::str::from_utf8(&raw[header_end..])?.to_string()
        } else {
            String::new()
        };

        Ok(Tag {
            object,
            object_type,
            tag_name,
            tagger,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_new() {
        let tag = Tag::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "commit",
            "v1.0.0",
            "Tagger <tagger@example.com> 1000000000 +0000",
            "Release v1.0.0\n",
        );
        assert_eq!(tag.object, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
        assert_eq!(tag.object_type, "commit");
        assert_eq!(tag.tag_name, "v1.0.0");
        assert_eq!(tag.message, "Release v1.0.0\n");
    }

    #[test]
    fn test_tag_hash() {
        let tag = Tag::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "commit",
            "v1.0.0",
            "Tagger <tagger@example.com> 1000000000 +0000",
            "Release v1.0.0\n",
        );
        let hash = tag.hash();
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tag_serialize_deserialize() {
        let tag = Tag::new(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "commit",
            "v2.0.0",
            "Someone <someone@test.com> 1600000000 +0800",
            "Second release\nwith multi-line message\n",
        );

        let data = tag.serialize();
        let deserialized = Tag::deserialize(&data).unwrap();

        assert_eq!(deserialized.object, tag.object);
        assert_eq!(deserialized.object_type, tag.object_type);
        assert_eq!(deserialized.tag_name, tag.tag_name);
        assert_eq!(deserialized.tagger, tag.tagger);
        assert_eq!(deserialized.message, tag.message);
    }

    #[test]
    fn test_tag_hash_consistent() {
        let tag = Tag::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "commit",
            "v1.0.0",
            "T <t@t.com> 1 +0000",
            "test\n",
        );
        let deserialized = Tag::deserialize(&tag.serialize()).unwrap();
        assert_eq!(tag.hash(), deserialized.hash());
    }

    #[test]
    fn test_tag_empty_message() {
        let tag = Tag::new(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "commit",
            "v0.0.0",
            "Tagger <t@t.com> 1 +0000",
            "",
        );
        let data = tag.serialize();
        let deserialized = Tag::deserialize(&data).unwrap();
        assert_eq!(deserialized.message, "");
    }

    #[test]
    fn test_tag_object_type_blob() {
        let tag = Tag::new(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "blob",
            "file-tag",
            "Tagger <t@t.com> 1 +0000",
            "Tag a blob\n",
        );
        let data = tag.serialize();
        let deserialized = Tag::deserialize(&data).unwrap();
        assert_eq!(deserialized.object_type, "blob");
    }
}

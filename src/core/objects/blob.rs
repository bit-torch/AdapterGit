use crate::core::hash::hash_git_object;

pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    pub fn new(content: Vec<u8>) -> Self {
        Blob { content }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.content.len()
    }

    pub fn hash(&self) -> String {
        hash_git_object("blob", &self.content)
    }

    #[allow(dead_code)]
    pub fn serialize(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.content.len());
        let mut data = Vec::with_capacity(header.len() + self.content.len());
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(&self.content);
        data
    }

    #[allow(dead_code)]
    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let null_pos = data
            .iter()
            .position(|&b| b == 0)
            .ok_or("Invalid blob: no null byte found")?;
        let header = std::str::from_utf8(&data[..null_pos])?;
        let parts: Vec<&str> = header.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0] != "blob" {
            return Err("Invalid blob header: expected 'blob <size>'".into());
        }
        let _expected_size: usize = parts[1].parse()?;
        let content = data[null_pos + 1..].to_vec();
        Ok(Blob { content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_new_and_size() {
        let blob = Blob::new(b"hello".to_vec());
        assert_eq!(blob.size(), 5);
    }

    #[test]
    fn test_blob_hash() {
        let blob = Blob::new(b"hello world\n".to_vec());
        assert_eq!(blob.hash(), "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn test_blob_serialize() {
        let blob = Blob::new(b"hello".to_vec());
        let data = blob.serialize();
        assert_eq!(&data[..5], b"blob ");
        assert_eq!(data[5], b'5');
        assert_eq!(data[6], 0);
        assert_eq!(&data[7..], b"hello");
    }

    #[test]
    fn test_blob_deserialize() {
        let blob = Blob::new(b"hello world".to_vec());
        let data = blob.serialize();
        let deserialized = Blob::deserialize(&data).unwrap();
        assert_eq!(deserialized.content, b"hello world");
    }

    #[test]
    fn test_blob_roundtrip() {
        let content = b"test content for roundtrip check".to_vec();
        let blob = Blob::new(content.clone());
        let data = blob.serialize();
        let deserialized = Blob::deserialize(&data).unwrap();
        assert_eq!(deserialized.content, content);
        assert_eq!(deserialized.hash(), blob.hash());
    }
}

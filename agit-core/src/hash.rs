use sha1::{Digest, Sha1};

pub fn hash_bytes(data: &[u8]) -> String {
    format!("{:x}", Sha1::digest(data))
}

pub fn hash_git_object(obj_type: &str, content: &[u8]) -> String {
    let header = format!("{} {}", obj_type, content.len());
    let mut data = Vec::with_capacity(header.len() + 1 + content.len());
    data.extend_from_slice(header.as_bytes());
    data.push(0);
    data.extend_from_slice(content);
    hash_bytes(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes_known_input() {
        let result = hash_bytes(b"hello");
        assert_eq!(result, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_hash_git_object_blob_hello_world() {
        let result = hash_git_object("blob", b"hello world\n");
        assert_eq!(result, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn test_hash_git_object_blob_test() {
        let result = hash_git_object("blob", b"test\n");
        assert_eq!(result, "9daeafb9864cf43055ae93beb0afd6c7d144bfa4");
    }

    #[test]
    fn test_hash_git_object_empty_blob() {
        let result = hash_git_object("blob", b"");
        assert_eq!(result, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }
}

pub mod blob;
pub mod commit;
pub mod tree;

pub fn format_object_data(obj_type: &str, content: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut data = Vec::with_capacity(header.len() + content.len());
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(content);
    data
}

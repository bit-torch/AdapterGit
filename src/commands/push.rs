use crate::core::protocol::HttpTransport;
use crate::core::refs;
use crate::core::remote_utils;
use crate::core::repo;

pub fn run(remote: Option<&str>, branch: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let remote_url = resolve_url(&repo_root, remote)?;
    let branch_name = resolve_branch(&repo_root, branch)?;

    let head_sha1 = refs::read_ref(&repo_root, &format!("refs/heads/{}", branch_name))?;

    let transport = HttpTransport::from_url(&remote_url)?;
    let remote_refs = transport.discover_refs()?;

    let remote_branch_ref = format!("refs/heads/{}", branch_name);
    let remote_sha1 = remote_refs
        .iter()
        .find(|(_, name)| name == &remote_branch_ref)
        .map(|(sha1, _)| sha1.clone());

    let objects = remote_utils::collect_local_objects_for_push(
        &repo_root,
        &head_sha1,
        remote_sha1.as_deref(),
    )?;

    let old_sha1 =
        remote_sha1.unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());

    let ref_update = format!("{} {} refs/heads/{}", old_sha1, head_sha1, branch_name);

    let pack_data = generate_pack(&objects)?;
    transport.push_pack(&ref_update, &pack_data)?;

    refs::write_ref(
        &repo_root,
        &format!("refs/remotes/origin/{}", branch_name),
        &head_sha1,
    )?;

    println!(
        "To {}\n   {} -> {}",
        remote_url,
        &head_sha1[..7],
        branch_name
    );

    Ok(())
}

fn resolve_url(
    repo: &std::path::Path,
    remote: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(remote_name) = remote {
        let refs_path = repo
            .join(".git")
            .join("refs")
            .join("remotes")
            .join(remote_name);
        if !refs_path.exists() {
            return Err(format!("remote '{}' not found", remote_name).into());
        }
    }
    remote_utils::get_remote_url(repo)
}

fn resolve_branch(
    repo: &std::path::Path,
    branch: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(b) = branch {
        return Ok(b.to_string());
    }
    remote_utils::get_current_branch(repo)
}

fn generate_pack(objects: &[(String, Vec<u8>)]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut pack = Vec::new();
    pack.extend_from_slice(b"PACK");
    pack.extend_from_slice(&2u32.to_be_bytes());
    pack.extend_from_slice(&(objects.len() as u32).to_be_bytes());

    for (_sha1, data) in objects {
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(0);
        let header = &data[..null_pos];
        let header_str = std::str::from_utf8(header)?;
        let obj_type_str = header_str.split(' ').next().unwrap_or("blob");

        let type_code: u8 = match obj_type_str {
            "commit" => 1,
            "tree" => 2,
            "blob" => 3,
            "tag" => 4,
            _ => 3,
        };

        let raw_content = &data[null_pos + 1..];
        let size = raw_content.len();
        let mut size_bytes = Vec::new();
        let mut remaining = size;
        let mut first_byte = (type_code << 4) | (remaining as u8 & 0x0F);
        remaining >>= 4;
        if remaining > 0 {
            first_byte |= 0x80;
        }
        size_bytes.push(first_byte);
        while remaining > 0 {
            let byte = if remaining > 0x7F {
                0x80 | (remaining as u8 & 0x7F)
            } else {
                remaining as u8 & 0x7F
            };
            size_bytes.push(byte);
            remaining >>= 7;
        }

        let compressed = crate::core::compression::compress(raw_content)?;

        pack.extend_from_slice(&size_bytes);
        pack.extend_from_slice(&compressed);
    }

    let sha1 = crate::core::hash::hash_bytes(&pack);
    let sha1_bytes: Vec<u8> = (0..40)
        .step_by(2)
        .map(|i| u8::from_str_radix(&sha1[i..i + 2], 16).unwrap_or(0))
        .collect();
    pack.extend_from_slice(&sha1_bytes);

    Ok(pack)
}

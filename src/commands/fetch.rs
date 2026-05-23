use crate::core::protocol::HttpTransport;
use crate::core::refs;
use crate::core::remote_utils;
use crate::core::repo;

pub fn run(url: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let remote_url = resolve_url(&repo_root, url)?;
    let branch = remote_utils::get_current_branch(&repo_root)?;

    let transport = HttpTransport::from_url(&remote_url)?;
    let remote_refs = transport.discover_refs()?;

    let remote_branch_ref = format!("refs/heads/{}", branch);
    let want_sha1 = remote_refs
        .iter()
        .find(|(_, name)| name == &remote_branch_ref)
        .map(|(sha1, _)| sha1.clone())
        .ok_or_else(|| format!("Remote branch '{}' not found", branch))?;

    let local_sha1 = refs::read_ref(&repo_root, &format!("refs/heads/{}", branch)).ok();
    let mut haves = Vec::new();
    if let Some(ref sha1) = local_sha1 {
        haves.push(sha1.clone());
        if let Ok(commits) = remote_utils::collect_recent_commits(&repo_root, sha1, 20) {
            haves.extend(commits);
        }
    }

    let objects = transport.fetch_objects(&[want_sha1.clone()], &haves)?;

    if objects.is_empty() {
        println!("Already up to date.");
        return Ok(());
    }

    remote_utils::write_objects(&repo_root, &objects)?;
    refs::write_ref(
        &repo_root,
        &format!("refs/remotes/origin/{}", branch),
        &want_sha1,
    )?;

    let head_sha1 = refs::read_head(&repo_root)?;

    println!(
        "From {}\n   {}..{}  {} -> {}",
        remote_url,
        &head_sha1[..7],
        &want_sha1[..7],
        branch,
        branch
    );

    Ok(())
}

fn resolve_url(repo: &std::path::Path, url: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(u) = url {
        return Ok(u.to_string());
    }
    remote_utils::get_remote_url(repo)
}

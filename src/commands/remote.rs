use std::fs;

use crate::core::repo;

pub fn run_add(name: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let config_path = repo_root.join(".git").join("config");

    let mut config = fs::read_to_string(&config_path).unwrap_or_default();

    let section = format!(
        "[remote \"{}\"]\n\turl = {}\n\tfetch = +refs/heads/*:refs/remotes/{}/*\n",
        name, url, name
    );

    if config.contains(&format!("[remote \"{}\"]", name)) {
        println!("Remote '{}' already exists.", name);
        return Ok(());
    }

    config.push('\n');
    config.push_str(&section);

    fs::write(&config_path, &config)?;

    let remotes_dir = repo_root
        .join(".git")
        .join("refs")
        .join("remotes")
        .join(name);
    if !remotes_dir.exists() {
        fs::create_dir_all(&remotes_dir)?;
    }

    println!("Added remote '{}' -> {}", name, url);

    Ok(())
}

pub fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo::find_repo_root()?;
    let config_path = repo_root.join(".git").join("config");

    if !config_path.exists() {
        println!("No remotes configured.");
        return Ok(());
    }

    let config = fs::read_to_string(&config_path)?;
    let mut remotes = Vec::new();

    for line in config.lines() {
        if let Some(name) = line
            .trim()
            .strip_prefix("[remote \"")
            .and_then(|s| s.strip_suffix("\"]"))
        {
            remotes.push(name.to_string());
        }
    }

    if remotes.is_empty() {
        println!("No remotes configured.");
        return Ok(());
    }

    for remote in &remotes {
        let mut url = String::new();
        let mut in_section = false;
        for line in config.lines() {
            let trimmed = line.trim();
            if trimmed == format!("[remote \"{}\"]", remote) {
                in_section = true;
            } else if in_section && trimmed.starts_with('[') {
                in_section = false;
            } else if in_section {
                if let Some(u) = trimmed.strip_prefix("url = ") {
                    url = u.to_string();
                    break;
                }
            }
        }
        println!("{}\t{}", remote, url);
    }

    Ok(())
}

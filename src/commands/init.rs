use crate::core::repo;
use std::fs;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let git_dir = cwd.join(".git");

    if git_dir.exists() {
        println!("Reinitialized existing Git repository in {}", git_dir.display());
        return Ok(());
    }

    fs::create_dir(&git_dir)?;
    fs::create_dir(git_dir.join("objects"))?;
    fs::create_dir(git_dir.join("objects").join("pack"))?;
    fs::create_dir(git_dir.join("objects").join("info"))?;
    repo::ensure_dir(&git_dir.join("refs").join("heads"))?;
    repo::ensure_dir(&git_dir.join("refs").join("tags"))?;

    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

    fs::write(
        git_dir.join("config"),
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = false\n\tbare = false\n\
         [user]\n\tname = agit\n\temail = agit@localhost\n",
    )?;
    fs::write(
        git_dir.join("description"),
        "Unnamed repository; edit this file to name it for gitweb.\n",
    )?;

    let info_dir = git_dir.join("info");
    if !info_dir.exists() {
        fs::create_dir(&info_dir)?;
    }
    fs::write(info_dir.join("exclude"), "# git ls-files --others --exclude-standard\n")?;

    println!(
        "Initialized empty Git repository in {}/.git/",
        cwd.display()
    );

    Ok(())
}

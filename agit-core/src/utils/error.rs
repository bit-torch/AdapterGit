use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AgitError {
    Io(std::io::Error),
    ObjectNotFound(String),
    InvalidObject(String),
    InvalidRef(String),
    CompressionError(String),
    RepoNotFound(PathBuf),
    NotAGitRepo(PathBuf),
    Other(anyhow::Error),
}

impl std::fmt::Display for AgitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgitError::Io(e) => write!(f, "IO error: {}", e),
            AgitError::ObjectNotFound(id) => write!(f, "Object '{}' not found", id),
            AgitError::InvalidObject(msg) => write!(f, "Invalid object: {}", msg),
            AgitError::InvalidRef(msg) => write!(f, "Invalid ref: {}", msg),
            AgitError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            AgitError::RepoNotFound(path) => write!(f, "Repository not found: {}", path.display()),
            AgitError::NotAGitRepo(path) => write!(f, "Not a git repository: {}", path.display()),
            AgitError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AgitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AgitError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AgitError {
    fn from(e: std::io::Error) -> Self {
        AgitError::Io(e)
    }
}

impl From<anyhow::Error> for AgitError {
    fn from(e: anyhow::Error) -> Self {
        AgitError::Other(e)
    }
}

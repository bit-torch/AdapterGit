use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// 全局配置（合并自环境变量、配置文件、默认值）。
#[derive(Debug, Clone)]
pub struct Config {
    pub user_name: String,
    pub user_email: String,
    pub aliases: HashMap<String, String>,
}

/// TOML 配置文件的反序列化结构。
#[derive(Debug, Clone, Default, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    user: Option<UserSection>,
    #[serde(default)]
    alias: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct UserSection {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email: Option<String>,
}

impl Config {
    /// 按优先级加载配置：环境变量 > 仓库级 .agit/config.toml > 全局 ~/.agitconfig.toml > 默认值。
    pub fn load(repo_path: Option<&Path>) -> Self {
        let mut file_user_name: Option<String> = None;
        let mut file_user_email: Option<String> = None;
        let mut aliases = HashMap::new();

        // 1. 读取全局配置文件 ~/.agitconfig.toml
        if let Some(home) = dirs_fallback() {
            let global_path = home.join(".agitconfig.toml");
            if let Some(cfg) = read_config_file(&global_path) {
                merge_config(
                    &cfg,
                    &mut file_user_name,
                    &mut file_user_email,
                    &mut aliases,
                );
            }
        }

        // 2. 读取仓库级配置文件 .agit/config.toml（覆盖全局）
        if let Some(repo) = repo_path {
            let repo_config = repo.join(".agit").join("config.toml");
            if let Some(cfg) = read_config_file(&repo_config) {
                merge_config(
                    &cfg,
                    &mut file_user_name,
                    &mut file_user_email,
                    &mut aliases,
                );
            }
        }

        // 3. 优先级：环境变量 > 配置文件 > 默认值
        let user_name = env::var("AGIT_USER_NAME")
            .or_else(|_| env::var("GIT_AUTHOR_NAME"))
            .ok()
            .or(file_user_name)
            .unwrap_or_else(|| "agit".to_string());

        let user_email = env::var("AGIT_USER_EMAIL")
            .or_else(|_| env::var("GIT_AUTHOR_EMAIL"))
            .ok()
            .or(file_user_email)
            .unwrap_or_else(|| "agit@localhost".to_string());

        Config {
            user_name,
            user_email,
            aliases,
        }
    }
}

fn read_config_file(path: &Path) -> Option<ConfigFile> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

fn merge_config(
    cfg: &ConfigFile,
    user_name: &mut Option<String>,
    user_email: &mut Option<String>,
    aliases: &mut HashMap<String, String>,
) {
    if let Some(ref user) = cfg.user {
        if let Some(ref name) = user.name {
            *user_name = Some(name.clone());
        }
        if let Some(ref email) = user.email {
            *user_email = Some(email.clone());
        }
    }
    if let Some(ref alias_map) = cfg.alias {
        for (k, v) in alias_map {
            aliases.insert(k.clone(), v.clone());
        }
    }
}

/// 不依赖 `dirs` crate 的 home 目录获取。
fn dirs_fallback() -> Option<PathBuf> {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// 便捷函数：使用当前工作目录作为仓库路径加载配置。
pub fn load() -> Config {
    Config::load(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        // 清除可能影响测试的环境变量
        env::remove_var("AGIT_USER_NAME");
        env::remove_var("AGIT_USER_EMAIL");
        let config = Config::load(None);
        assert_eq!(config.user_name, "agit");
        assert_eq!(config.user_email, "agit@localhost");
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_config_from_file() {
        env::remove_var("AGIT_USER_NAME");
        env::remove_var("AGIT_USER_EMAIL");
        let dir = std::env::temp_dir().join(format!("agit_test_config_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let git_dir = dir.join(".agit");
        fs::create_dir_all(&git_dir).unwrap();

        let config_toml = r#"
[user]
name = "Test User"
email = "test@example.com"
"#;
        let config_path = git_dir.join("config.toml");
        let mut f = fs::File::create(&config_path).unwrap();
        f.write_all(config_toml.as_bytes()).unwrap();

        let config = Config::load(Some(&dir));
        assert_eq!(config.user_name, "Test User");
        assert_eq!(config.user_email, "test@example.com");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_aliases() {
        env::remove_var("AGIT_USER_NAME");
        env::remove_var("AGIT_USER_EMAIL");
        let dir = std::env::temp_dir().join(format!("agit_test_aliases_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let git_dir = dir.join(".agit");
        fs::create_dir_all(&git_dir).unwrap();

        let config_toml = r#"
[alias]
co = "commit"
br = "branch"
"#;
        let config_path = git_dir.join("config.toml");
        let mut f = fs::File::create(&config_path).unwrap();
        f.write_all(config_toml.as_bytes()).unwrap();

        let config = Config::load(Some(&dir));
        assert_eq!(config.aliases.get("co").unwrap(), "commit");
        assert_eq!(config.aliases.get("br").unwrap(), "branch");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_partial() {
        env::remove_var("AGIT_USER_NAME");
        env::remove_var("AGIT_USER_EMAIL");
        let dir = std::env::temp_dir().join(format!("agit_test_partial_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let git_dir = dir.join(".agit");
        fs::create_dir_all(&git_dir).unwrap();

        // Only set user name, not email
        let config_toml = r#"
[user]
name = "Partial User"
"#;
        let config_path = git_dir.join("config.toml");
        let mut f = fs::File::create(&config_path).unwrap();
        f.write_all(config_toml.as_bytes()).unwrap();

        let config = Config::load(Some(&dir));
        assert_eq!(config.user_name, "Partial User");
        assert_eq!(config.user_email, "agit@localhost"); // falls back to default

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_env_override() {
        env::remove_var("AGIT_USER_NAME");
        env::remove_var("AGIT_USER_EMAIL");
        let dir = std::env::temp_dir().join(format!("agit_test_env_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let git_dir = dir.join(".agit");
        fs::create_dir_all(&git_dir).unwrap();

        let config_toml = r#"
[user]
name = "File User"
email = "file@example.com"
"#;
        let config_path = git_dir.join("config.toml");
        let mut f = fs::File::create(&config_path).unwrap();
        f.write_all(config_toml.as_bytes()).unwrap();

        // Set env var - should override file
        env::set_var("AGIT_USER_NAME", "Env User");
        let config = Config::load(Some(&dir));
        assert_eq!(config.user_name, "Env User");
        assert_eq!(config.user_email, "file@example.com"); // not overridden by env

        env::remove_var("AGIT_USER_NAME");
        let _ = fs::remove_dir_all(&dir);
    }
}

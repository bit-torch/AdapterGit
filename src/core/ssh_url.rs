//! SSH URL 解析与 ~/.ssh/config 处理。
//!
//! 支持两种 SSH URL 格式：
//! 1. SSH 标准格式: `ssh://[user@]host[:port]/path/to/repo.git`
//! 2. SCP 风格:    `[user@]host:path/to/repo.git`
//!
//! 同时解析 `~/.ssh/config` 以支持主机别名、自定义端口和密钥文件。

use std::path::PathBuf;

/// 解析后的 SSH 连接信息
#[derive(Debug, Clone, PartialEq)]
pub struct SshUrl {
    pub user: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl SshUrl {
    /// 从 SSH URL 字符串解析。
    /// 支持 `ssh://` 和 SCP 风格 (`git@host:path`)。
    pub fn parse(input: &str) -> Option<SshUrl> {
        // 1. 尝试 SSH 标准格式: ssh://[user@]host[:port]/path
        if input.starts_with("ssh://") {
            return Self::parse_ssh_scheme(input);
        }

        // 2. 尝试 SCP 风格: [user@]host:path
        if input.contains('@') && input.contains(':') && !input.contains("://") {
            return Self::parse_scp(input);
        }

        // 3. 简化 SCP 风格 (无 user): host:path
        if input.contains(':') && !input.contains("://") {
            return Self::parse_scp_simple(input);
        }

        None
    }

    /// 解析 `ssh://[user@]host[:port]/path`
    fn parse_ssh_scheme(input: &str) -> Option<SshUrl> {
        let without_scheme = input.strip_prefix("ssh://")?;
        let (rest, path) = without_scheme.split_once('/')?;
        let path = format!("/{}", path);

        let (user, host, port) = Self::parse_user_host_port(rest)?;

        Some(SshUrl {
            user,
            host,
            port,
            path,
        })
    }

    /// 解析 `[user@]host:path`
    fn parse_scp(input: &str) -> Option<SshUrl> {
        let (user_host, path) = input.split_once(':')?;
        let (user, host, port) = Self::parse_user_host_port(user_host)?;

        Some(SshUrl {
            user,
            host,
            port,
            path: path.to_string(),
        })
    }

    /// 解析 `host:path`（无用户名）
    fn parse_scp_simple(input: &str) -> Option<SshUrl> {
        let (host, path) = input.split_once(':')?;
        let port = 22;

        Some(SshUrl {
            user: "git".to_string(), // 默认 SSH 用户
            host: host.to_string(),
            port,
            path: path.to_string(),
        })
    }

    /// 从 `user@host[:port]` 中提取各组件
    fn parse_user_host_port(rest: &str) -> Option<(String, String, u16)> {
        let (user, host_port) = if let Some((user, host)) = rest.split_once('@') {
            (user.to_string(), host.to_string())
        } else {
            ("git".to_string(), rest.to_string())
        };

        let (host, port) = if let Some((host, port_str)) = host_port.split_once(':') {
            (host.to_string(), port_str.parse::<u16>().unwrap_or(22))
        } else {
            (host_port, 22)
        };

        if host.is_empty() {
            return None;
        }

        Some((user, host, port))
    }
}

/// 加载并应用 `~/.ssh/config`（如果存在）。
///
/// 根据别名解析真实主机名、用户和端口。
#[allow(dead_code)]
pub fn apply_ssh_config(host: &str) -> SshConfigEntry {
    let config_path = ssh_config_path();
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        let entries = parse_ssh_config(&content);
        if let Some(entry) = entries.iter().find(|e| e.matches(host)) {
            return entry.clone();
        }
    }
    SshConfigEntry::default_host(host)
}

/// SSH 配置条目
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SshConfigEntry {
    /// Host 指令的模式（用于匹配，如 "gh"、"*.example.com"）
    pub host_pattern: String,
    /// HostName 指令的值（真实主机名，默认为 host_pattern）
    pub hostname: String,
    pub user: String,
    pub port: u16,
    pub identity_file: Option<PathBuf>,
}

impl SshConfigEntry {
    /// 无配置时的默认值
    pub fn default_host(host: &str) -> Self {
        SshConfigEntry {
            host_pattern: host.to_string(),
            hostname: host.to_string(),
            user: "git".to_string(),
            port: 22,
            identity_file: None,
        }
    }

    /// 检查此条目是否匹配给定的主机名或别名
    fn matches(&self, host: &str) -> bool {
        let patterns: Vec<&str> = self.host_pattern.split_whitespace().collect();
        patterns.iter().any(|p| {
            if *p == "*" {
                true
            } else if let Some((prefix, suffix)) = p.split_once('*') {
                // 通配符匹配: prefix*suffix
                host.starts_with(prefix)
                    && host.ends_with(suffix)
                    && host.len() >= prefix.len() + suffix.len()
            } else {
                *p == host
            }
        })
    }
}

/// 解析 `~/.ssh/config` 内容
#[allow(dead_code)]
fn parse_ssh_config(content: &str) -> Vec<SshConfigEntry> {
    let mut entries = Vec::new();
    let mut current_pattern: Option<String> = None;
    let mut current_hostname: Option<String> = None;
    let mut current_user: Option<String> = None;
    let mut current_port: Option<u16> = None;
    let mut current_identity: Option<PathBuf> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 跳过空行和注释
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // 分割关键字和值
        let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        if parts.len() < 2 {
            continue;
        }
        let keyword = parts[0].to_lowercase();
        let value = parts[1].trim();

        match keyword.as_str() {
            "host" => {
                // 遇到新 Host 块，保存上一个
                if let Some(pattern) = current_pattern.take() {
                    entries.push(SshConfigEntry {
                        hostname: current_hostname.take().unwrap_or_else(|| pattern.clone()),
                        host_pattern: pattern,
                        user: current_user.take().unwrap_or_else(|| "git".to_string()),
                        port: current_port.take().unwrap_or(22),
                        identity_file: current_identity.take(),
                    });
                }
                current_pattern = Some(value.to_string());
            }
            "hostname" => {
                current_hostname = Some(value.to_string());
            }
            "user" => {
                current_user = Some(value.to_string());
            }
            "port" => {
                current_port = value.parse::<u16>().ok();
            }
            "identityfile" => {
                let path = shellexpand(value);
                current_identity = Some(PathBuf::from(path));
            }
            _ => {}
        }
    }

    // 保存最后一个条目
    if let Some(pattern) = current_pattern.take() {
        entries.push(SshConfigEntry {
            hostname: current_hostname.take().unwrap_or_else(|| pattern.clone()),
            host_pattern: pattern,
            user: current_user.take().unwrap_or_else(|| "git".to_string()),
            port: current_port.take().unwrap_or(22),
            identity_file: current_identity.take(),
        });
    }

    entries
}

/// 展开 `~` 为家目录
#[allow(dead_code)]
fn shellexpand(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs_home() {
            return home.to_string_lossy().to_string() + &path[1..];
        }
    }
    if path == "~" {
        if let Some(home) = dirs_home() {
            return home.to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// 获取家目录
#[allow(dead_code)]
fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOMEDRIVE")
                    .ok()
                    .zip(std::env::var("HOMEPATH").ok())
                    .map(|(drive, path)| PathBuf::from(format!("{}{}", drive, path)))
            })
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

/// 获取 `~/.ssh/config` 路径
#[allow(dead_code)]
fn ssh_config_path() -> PathBuf {
    let home = dirs_home().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ssh").join("config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url_standard() {
        let url = SshUrl::parse("ssh://git@github.com/user/repo.git").unwrap();
        assert_eq!(url.user, "git");
        assert_eq!(url.host, "github.com");
        assert_eq!(url.port, 22);
        assert_eq!(url.path, "/user/repo.git");
    }

    #[test]
    fn test_parse_ssh_url_standard_with_port() {
        let url = SshUrl::parse("ssh://git@github.com:2222/user/repo.git").unwrap();
        assert_eq!(url.user, "git");
        assert_eq!(url.host, "github.com");
        assert_eq!(url.port, 2222);
        assert_eq!(url.path, "/user/repo.git");
    }

    #[test]
    fn test_parse_ssh_url_scp() {
        let url = SshUrl::parse("git@github.com:user/repo.git").unwrap();
        assert_eq!(url.user, "git");
        assert_eq!(url.host, "github.com");
        assert_eq!(url.port, 22);
        assert_eq!(url.path, "user/repo.git");
    }

    #[test]
    fn test_parse_ssh_url_scp_no_user() {
        let url = SshUrl::parse("github.com:user/repo.git").unwrap();
        assert_eq!(url.user, "git");
        assert_eq!(url.host, "github.com");
        assert_eq!(url.port, 22);
        assert_eq!(url.path, "user/repo.git");
    }

    #[test]
    fn test_parse_ssh_url_not_ssh() {
        assert!(SshUrl::parse("https://github.com/user/repo.git").is_none());
        assert!(SshUrl::parse("http://example.com/repo.git").is_none());
    }

    #[test]
    fn test_parse_ssh_config_basic() {
        let config = r#"
Host gh
    HostName github.com
    User git
    Port 22
"#;
        let entries = parse_ssh_config(config);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert!(e.matches("gh"));
        assert!(!e.matches("github.com"));
    }

    #[test]
    fn test_parse_ssh_config_wildcard() {
        let config = r#"
Host *.example.com
    User deploy
"#;
        let entries = parse_ssh_config(config);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert!(e.matches("git.example.com"));
        assert!(e.matches("api.example.com"));
        assert!(!e.matches("other.org"));
    }

    #[test]
    fn test_parse_ssh_config_multiple_hosts() {
        let config = r#"
Host gh
    HostName github.com
    User git

Host mylab
    HostName 192.168.1.100
    Port 2222
    User admin
"#;
        let entries = parse_ssh_config(config);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_shellexpand_tilde() {
        let expanded = shellexpand("~/.ssh/id_rsa");
        assert!(
            !expanded.starts_with('~'),
            "~ should be expanded: {}",
            expanded
        );
    }
}

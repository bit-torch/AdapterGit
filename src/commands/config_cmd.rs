use crate::core::repo;
use std::fs;
use std::path::PathBuf;
use toml::map::Map;

/// Config 命令入口。
pub fn run(
    global: bool,
    list: bool,
    unset: bool,
    _get: bool,
    key: Option<&str>,
    value: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = resolve_config_path(global)?;

    if list {
        return list_config(&config_path);
    }

    let key = key.ok_or("error: key is required")?;

    if unset {
        return unset_key(&config_path, key);
    }

    match value {
        Some(val) => set_key(&config_path, key, val),
        None => get_key(&config_path, key),
    }
}

/// 确定配置文件路径。
fn resolve_config_path(global: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if global {
        let home = dirs_fallback()
            .ok_or("error: cannot determine home directory (set HOME or USERPROFILE)")?;
        Ok(home.join(".agitconfig.toml"))
    } else {
        let repo_root = repo::find_repo_root()?;
        Ok(repo_root.join(".agit").join("config.toml"))
    }
}

/// 列出所有配置项。
fn list_config(config_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let table = read_toml(config_path);
    if table.is_empty() {
        // 即使文件不存在也输出默认配置的键值
        println!("user.name=agit");
        println!("user.email=agit@localhost");
        return Ok(());
    }
    // 遍历顶层 sections
    for (section, section_val) in &table {
        if let Some(inner) = section_val.as_table() {
            for (k, v) in inner {
                let display_key = format!("{}.{}", section, k);
                match v {
                    toml::Value::String(s) => println!("{}={}", display_key, s),
                    _ => println!("{}={}", display_key, v),
                }
            }
        }
    }
    Ok(())
}

/// 获取单个配置值。
fn get_key(config_path: &std::path::Path, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (section, field) = parse_key(key)?;

    let table = read_toml(config_path);
    if let Some(section_val) = table.get(&section) {
        if let Some(inner) = section_val.as_table() {
            if let Some(val) = inner.get(&field) {
                match val {
                    toml::Value::String(s) => println!("{}", s),
                    _ => println!("{}", val),
                }
                return Ok(());
            }
        }
    }

    // 尝试返回默认值
    match key {
        "user.name" => println!("agit"),
        "user.email" => println!("agit@localhost"),
        _ => {} // key not found, no output (matches git config behavior)
    }

    Ok(())
}

/// 设置配置值。
fn set_key(
    config_path: &std::path::Path,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (section, field) = parse_key(key)?;
    let mut table = read_toml(config_path);

    // 插入或更新 section.field = value
    let inner = table
        .entry(section.clone())
        .or_insert_with(|| toml::Value::Table(Map::new()));
    if let Some(t) = inner.as_table_mut() {
        t.insert(field, toml::Value::String(value.to_string()));
    }

    write_toml(config_path, &table)?;
    Ok(())
}

/// 删除配置项。
fn unset_key(config_path: &std::path::Path, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (section, field) = parse_key(key)?;
    let mut table = read_toml(config_path);

    if let Some(section_val) = table.get_mut(&section) {
        if let Some(t) = section_val.as_table_mut() {
            t.remove(&field);
            // 如果 section 变空了，删除整个 section
            if t.is_empty() {
                table.remove(&section);
            }
        }
    }

    write_toml(config_path, &table)?;
    Ok(())
}

/// 解析 "section.field" 为 (section, field)。
fn parse_key(key: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(format!(
            "error: invalid config key '{}' (expected section.key format)",
            key
        )
        .into());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// 读取 TOML 文件返回 table Map。
fn read_toml(path: &std::path::Path) -> Map<String, toml::Value> {
    match fs::read_to_string(path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => Map::new(),
    }
}

/// 将 table 写入 TOML 文件。
fn write_toml(path: &std::path::Path, table: &Map<String, toml::Value>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(table)?;
    fs::write(path, content)?;
    Ok(())
}

/// 不依赖 `dirs` crate 的 home 目录获取。
fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

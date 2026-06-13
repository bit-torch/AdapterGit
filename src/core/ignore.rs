//! .gitignore 规则解析与路径匹配模块。
//!
//! 支持：`*`、`?`、`**`、`[abc]` 字符类、`!` 否定、`#` 注释、目录标记 `/`。
//! 不依赖外部 crate，纯手写实现。

use std::fs;
use std::path::Path;

/// 单条 ignore 规则。
#[derive(Debug, Clone)]
struct IgnoreRule {
    /// 是否为否定规则（`!` 前缀）。
    negated: bool,
    /// 是否仅匹配目录。
    dir_only: bool,
    /// 原始模式字符串。
    pattern: String,
    /// 是否为绝对匹配（以 `/` 开头）。
    anchored: bool,
}

impl IgnoreRule {
    fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        // 空行或注释
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let mut negated = false;
        let mut dir_only = false;
        let rest;

        // 处理否定
        if let Some(s) = trimmed.strip_prefix('!') {
            negated = true;
            rest = s;
        } else {
            rest = trimmed;
        }

        let rest = rest.trim();

        // 处理目录标记（末尾 `/`）
        let rest = if let Some(s) = rest.strip_suffix('/') {
            dir_only = true;
            s
        } else {
            rest
        };

        // 判断是否 anchored（以 `/` 开头，或包含 `/` 在中间）
        let anchored = rest.starts_with('/') || rest.contains('/');

        // 去除开头的 `/`
        let pattern = rest.trim_start_matches('/').to_string();

        if pattern.is_empty() {
            return None;
        }

        Some(IgnoreRule {
            negated,
            dir_only,
            pattern,
            anchored,
        })
    }

    /// 检查路径是否匹配本规则。
    fn matches(&self, path: &str, is_dir: bool) -> bool {
        if self.dir_only && !is_dir {
            return false;
        }

        if self.anchored {
            gitignore_match(&self.pattern, path)
        } else {
            // 非 anchored：匹配路径的任意后缀
            gitignore_match_suffix(&self.pattern, path)
        }
    }
}

/// Ignore 匹配器，聚合多层 .gitignore 规则。
#[derive(Debug)]
pub struct IgnoreMatcher {
    rules: Vec<IgnoreRule>,
}

impl IgnoreMatcher {
    /// 创建空的匹配器。
    pub fn new() -> Self {
        IgnoreMatcher { rules: Vec::new() }
    }

    /// 从 .gitignore 文件加载规则。
    pub fn load(repo: &Path, relative_dir: &Path) -> Self {
        let mut matcher = IgnoreMatcher::new();

        // 从当前目录向上遍历，收集每个 .gitignore 的规则
        let mut file_rules: Vec<Vec<IgnoreRule>> = Vec::new();
        let mut current = repo.to_path_buf();
        current.push(relative_dir);

        while current.starts_with(repo) {
            let gitignore = current.join(".gitignore");
            if gitignore.exists() {
                if let Ok(content) = fs::read_to_string(&gitignore) {
                    let mut rules = Vec::new();
                    for line in content.lines() {
                        if let Some(rule) = IgnoreRule::parse(line) {
                            rules.push(rule);
                        }
                    }
                    if !rules.is_empty() {
                        file_rules.push(rules);
                    }
                }
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        // 父目录规则先添加，子目录规则后添加（后匹配=高优先级）
        // file_rules 顺序：[子目录, ..., 根目录]，反转后：[根目录, ..., 子目录]
        for rules in file_rules.iter().rev() {
            matcher.rules.extend(rules.clone());
        }

        // 加载 .git/info/exclude（优先级低于所有 .gitignore）
        let exclude_path = repo.join(".git").join("info").join("exclude");
        if exclude_path.exists() {
            if let Ok(content) = fs::read_to_string(&exclude_path) {
                for line in content.lines() {
                    if let Some(rule) = IgnoreRule::parse(line) {
                        matcher.rules.push(rule);
                    }
                }
            }
        }

        matcher
    }

    /// 判断给定路径是否应被忽略。
    pub fn is_ignored(&self, path: &str, is_dir: bool) -> bool {
        let mut ignored = false;

        // 检查路径本身
        for rule in &self.rules {
            if rule.matches(path, is_dir) {
                ignored = !rule.negated;
            }
        }

        // 如果路径本身未被忽略，检查祖先目录是否被忽略
        if !ignored {
            let mut parts: Vec<&str> = path.split('/').collect();
            while parts.len() > 1 {
                parts.pop(); // 移除最后一段
                let ancestor = parts.join("/");
                for rule in &self.rules {
                    if rule.matches(&ancestor, true) {
                        ignored = !rule.negated;
                    }
                }
                if ignored {
                    break;
                }
            }
        }

        ignored
    }
}

impl Default for IgnoreMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 gitignore 通配符模式转为匹配逻辑。
fn gitignore_match(pattern: &str, path: &str) -> bool {
    // 处理 `**` 模式
    if pattern.contains("**") {
        return globstar_match(pattern, path);
    }

    let parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if parts.len() != path_parts.len() {
        return false;
    }

    parts
        .iter()
        .zip(path_parts.iter())
        .all(|(p, t)| glob_match_component(p, t))
}

fn gitignore_match_suffix(pattern: &str, path: &str) -> bool {
    // 先尝试全路径匹配
    if gitignore_match(pattern, path) {
        return true;
    }
    // 也尝试作为 base name 匹配
    let base = path.rsplit('/').next().unwrap_or(path);
    if !pattern.contains('/') && !pattern.contains("**") {
        return glob_match_component(pattern, base);
    }
    false
}

/// `**` 匹配。
fn globstar_match(pattern: &str, path: &str) -> bool {
    let parts: Vec<&str> = pattern.split("**").collect();
    if parts.is_empty() || parts.len() > 2 {
        // 简化版：只处理单个 `**`
        return false;
    }

    let prefix = parts[0].trim_end_matches('/');
    let suffix = parts
        .get(1)
        .map(|s| s.trim_start_matches('/'))
        .unwrap_or("");

    if prefix.is_empty() && suffix.is_empty() {
        return true; // 纯 `**`
    }

    if prefix.is_empty() {
        // `**/suffix` — 匹配 path 任意后缀
        return path.ends_with(suffix)
            || path
                .rsplit('/')
                .any(|component| glob_match_component(suffix, component));
    }

    if suffix.is_empty() {
        // `prefix/**` — 匹配 path 前缀
        return path.starts_with(prefix);
    }

    // `prefix/**/suffix`
    let path_parts: Vec<&str> = path.split('/').collect();
    let prefix_parts: Vec<&str> = prefix.split('/').filter(|s| !s.is_empty()).collect();
    let suffix_parts: Vec<&str> = suffix.split('/').filter(|s| !s.is_empty()).collect();

    if path_parts.len() < prefix_parts.len() + suffix_parts.len() {
        return false;
    }

    // 匹配前缀
    for (i, part) in prefix_parts.iter().enumerate() {
        if !glob_match_component(part, path_parts[i]) {
            return false;
        }
    }

    // 匹配后缀
    for (i, part) in suffix_parts.iter().enumerate() {
        if !glob_match_component(part, path_parts[path_parts.len() - suffix_parts.len() + i]) {
            return false;
        }
    }

    true
}

/// 字符类匹配：支持 `[abc]` 和 `[a-z]` 范围。
fn char_class_match(class: &[u8], ch: u8) -> bool {
    let mut i = 0;
    while i < class.len() {
        if i + 2 < class.len() && class[i + 1] == b'-' {
            // 范围 a-z
            let start = class[i];
            let end = class[i + 2];
            if ch >= start && ch <= end {
                return true;
            }
            i += 3;
        } else {
            if class[i] == ch {
                return true;
            }
            i += 1;
        }
    }
    false
}

/// 单个路径组件的通配符匹配。
fn glob_match_component(pattern: &str, text: &str) -> bool {
    let pat = pattern.as_bytes();
    let txt = text.as_bytes();

    let mut pi = 0;
    let mut ti = 0;
    let mut star_pos: isize = -1;
    let mut match_pos: isize = -1;

    while ti < txt.len() {
        if pi < pat.len() {
            let pc = pat[pi];
            if pc == b'*' {
                star_pos = pi as isize;
                match_pos = ti as isize;
                pi += 1;
                continue;
            }
            if pc == b'?' || pc == txt[ti] {
                pi += 1;
                ti += 1;
                continue;
            }
            if pc == b'[' {
                // 字符类 [abc] 或 [a-z] 范围
                if let Some(end) = pat[pi..].iter().position(|&b| b == b']') {
                    let class = &pat[pi + 1..pi + end];
                    if char_class_match(class, txt[ti]) {
                        pi += end + 1;
                        ti += 1;
                        continue;
                    }
                }
            }
        }
        if star_pos >= 0 {
            pi = star_pos as usize + 1;
            match_pos += 1;
            ti = match_pos as usize;
            continue;
        }
        return false;
    }

    while pi < pat.len() && pat[pi] == b'*' {
        pi += 1;
    }

    pi == pat.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ignore() {
        let rules = vec![IgnoreRule::parse("*.txt").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("foo.txt", false));
        assert!(!m.is_ignored("foo.rs", false));
    }

    #[test]
    fn test_negation() {
        let rules = vec![
            IgnoreRule::parse("*.txt").unwrap(),
            IgnoreRule::parse("!important.txt").unwrap(),
        ];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("foo.txt", false));
        assert!(!m.is_ignored("important.txt", false));
    }

    #[test]
    fn test_dir_only() {
        let rules = vec![IgnoreRule::parse("target/").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("target", true));
        assert!(!m.is_ignored("target", false)); // 名为 target 的文件不忽略
        // 目录内的文件因祖先目录被忽略而忽略
        assert!(m.is_ignored("target/foo", false));
    }

    #[test]
    fn test_wildcard() {
        let rules = vec![IgnoreRule::parse("*.log").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("debug.log", false));
        assert!(m.is_ignored("error.log", false));
        assert!(!m.is_ignored("log.txt", false));
    }

    #[test]
    fn test_question_mark() {
        let rules = vec![IgnoreRule::parse("a?.txt").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("ab.txt", false));
        assert!(!m.is_ignored("abc.txt", false));
    }

    #[test]
    fn test_char_class() {
        let rules = vec![IgnoreRule::parse("file[0-9].txt").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("file1.txt", false));
        assert!(!m.is_ignored("fileA.txt", false));
    }

    #[test]
    fn test_globstar() {
        let rules = vec![IgnoreRule::parse("**/node_modules").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("node_modules", true));
        assert!(m.is_ignored("foo/node_modules", true));
        assert!(m.is_ignored("foo/bar/node_modules", true));
    }

    #[test]
    fn test_globstar_prefix() {
        let rules = vec![IgnoreRule::parse("target/**").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("target/debug/foo", false));
        assert!(m.is_ignored("target/release", false));
    }

    #[test]
    fn test_comment_empty() {
        assert!(IgnoreRule::parse("# comment").is_none());
        assert!(IgnoreRule::parse("").is_none());
        assert!(IgnoreRule::parse("   ").is_none());
    }

    #[test]
    fn test_anchored() {
        let rules = vec![IgnoreRule::parse("/build").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("build", true));
        assert!(!m.is_ignored("src/build", true));
    }

    #[test]
    fn test_nested_directory() {
        let rules = vec![IgnoreRule::parse("target/debug/").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("target/debug", true));
        assert!(!m.is_ignored("target/release", true));
    }

    #[test]
    fn test_exact_filename() {
        let rules = vec![IgnoreRule::parse("Cargo.lock").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("Cargo.lock", false));
        assert!(m.is_ignored("foo/Cargo.lock", false));
        assert!(!m.is_ignored("Cargo.lock.bak", false));
    }

    #[test]
    fn test_char_class_letters() {
        let rules = vec![IgnoreRule::parse("[abc].txt").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("a.txt", false));
        assert!(m.is_ignored("b.txt", false));
        assert!(!m.is_ignored("d.txt", false));
    }

    #[test]
    fn test_globstar_middle() {
        let rules = vec![IgnoreRule::parse("src/**/test").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("src/test", false));
        assert!(m.is_ignored("src/foo/test", false));
        assert!(m.is_ignored("src/foo/bar/test", false));
    }

    #[test]
    fn test_double_star_start() {
        let rules = vec![IgnoreRule::parse("**/temp").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("temp", false));
        assert!(m.is_ignored("a/temp", false));
        assert!(!m.is_ignored("template", false));
    }

    #[test]
    fn test_question_mark_multiple() {
        let rules = vec![IgnoreRule::parse("???.tmp").unwrap()];
        let m = IgnoreMatcher { rules };
        assert!(m.is_ignored("abc.tmp", false));
        assert!(!m.is_ignored("ab.tmp", false));
    }

    #[test]
    fn test_load_from_file() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("agit_ig_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let mut f = std::fs::File::create(dir.join(".gitignore")).unwrap();
        f.write_all(b"*.log\nbuild/\n").unwrap();
        let mut f2 = std::fs::File::create(dir.join("src").join(".gitignore")).unwrap();
        f2.write_all(b"!debug.log\n").unwrap();

        let matcher = IgnoreMatcher::load(&dir, std::path::Path::new("src"));
        assert!(matcher.is_ignored("error.log", false));
        assert!(!matcher.is_ignored("debug.log", false)); // negated
        assert!(matcher.is_ignored("build/output", false));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_exclude_file() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("agit_ex_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".git").join("info")).unwrap();

        let mut f = std::fs::File::create(dir.join(".git").join("info").join("exclude")).unwrap();
        f.write_all(b"*.private\n").unwrap();

        let matcher = IgnoreMatcher::load(&dir, std::path::Path::new(""));
        assert!(matcher.is_ignored("secret.private", false));
        assert!(!matcher.is_ignored("public.txt", false));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_matcher() {
        let m = IgnoreMatcher::new();
        assert!(!m.is_ignored("anything.txt", false));
        assert!(!m.is_ignored("dir", true));
    }

    #[test]
    fn test_default_matcher() {
        let m: IgnoreMatcher = Default::default();
        assert!(!m.is_ignored("file.txt", false));
    }
}

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

static JSON_MODE: AtomicBool = AtomicBool::new(false);
static YAML_MODE: AtomicBool = AtomicBool::new(false);
static NO_COLOR: AtomicBool = AtomicBool::new(false);

pub fn set_json_mode(v: bool) {
    JSON_MODE.store(v, Ordering::SeqCst);
}

pub fn set_yaml_mode(v: bool) {
    YAML_MODE.store(v, Ordering::SeqCst);
}

pub fn set_no_color(v: bool) {
    NO_COLOR.store(v, Ordering::SeqCst);
}

pub fn is_json() -> bool {
    JSON_MODE.load(Ordering::SeqCst)
}

pub fn is_yaml() -> bool {
    YAML_MODE.load(Ordering::SeqCst)
}

pub fn is_no_color() -> bool {
    NO_COLOR.load(Ordering::SeqCst)
}

pub fn colorize(text: &str, code: &str) -> String {
    if is_no_color() || code.is_empty() {
        text.to_string()
    } else {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    }
}

pub fn print_structured<T: Serialize>(data: &T) {
    if is_json() {
        if let Ok(json) = serde_json::to_string_pretty(data) {
            println!("{}", json);
        }
    } else if is_yaml() {
        if let Ok(yaml) = serde_yaml::to_string(data) {
            println!("{}", yaml);
        }
    }
}

pub fn print_lines_json(lines: &[(&str, &str)]) {
    if !is_json() {
        return;
    }
    let map: std::collections::BTreeMap<&str, &str> =
        lines.iter().copied().collect();
    if let Ok(json) = serde_json::to_string_pretty(&map) {
        println!("{}", json);
    }
}

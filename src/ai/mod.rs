use std::sync::atomic::{AtomicBool, Ordering};

static AI_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_ai_mode(v: bool) {
    AI_MODE.store(v, Ordering::SeqCst);
}

pub fn is_ai_mode() -> bool {
    AI_MODE.load(Ordering::SeqCst)
}

pub fn ai_commit_marker() -> &'static str {
    if is_ai_mode() {
        "[AI-committed] "
    } else {
        ""
    }
}

pub const DANGEROUS_COMMANDS: &[&str] = &["mergetool", "rebase", "bisect"];

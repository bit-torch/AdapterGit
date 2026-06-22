#[cfg(feature = "ai")]
pub use agit_ai as llm;

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

/// AI 模式下禁止执行的破坏性命令列表。
pub const DANGEROUS_COMMANDS: &[&str] = &[
    "mergetool",
    "rebase",
    "cherry-pick", // 可能引入不兼容的变更
    "bisect",
    "push",       // 可能推送到远程
    "stash drop", // 删除暂存
    "branch -D",  // 强制删除分支
];

/// 检查命令是否为 AI 模式下的危险命令。
/// 在 AI 模式下返回 Err 阻止执行；非 AI 模式始终返回 Ok。
pub fn check_dangerous_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !is_ai_mode() {
        return Ok(());
    }
    let cmd_lower = command.to_lowercase();
    for dangerous in DANGEROUS_COMMANDS {
        if cmd_lower.contains(dangerous) {
            return Err(format!(
                "AI mode: '{}' is a dangerous command and is blocked. Use --ai flag explicitly to override.",
                command
            )
            .into());
        }
    }
    Ok(())
}

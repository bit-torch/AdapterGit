pub mod error;

/// 原子写入文件：先写临时文件，再 rename 到目标路径。
/// 防止进程崩溃时留下空文件或半截文件。
pub fn atomic_write(
    path: &std::path::Path,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = path.with_extension(".tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

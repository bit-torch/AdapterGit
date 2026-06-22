//! SSH 传输层 — 通过系统 `ssh` 命令与远程 Git 仓库通信。
//!
//! 流程：
//! 1. 解析 SSH URL（`ssh://` 或 SCP 格式）
//! 2. 通过 `~/.ssh/config` 解析主机别名
//! 3. 执行 `ssh user@host git-upload-pack '/path'` 或 `git-receive-pack`
//! 4. 通过 stdin/stdout 进行 pkt-line 协议交互
//!
//! 优点：零额外依赖，继承用户 SSH 配置（密钥、known_hosts、agent）。

use std::io::{Read, Write};
use std::process::{Command, Stdio};

use crate::protocol::Transport;
use crate::protocol::{
    find_pack_start, parse_packfile, parse_refs_data, pkt_line_encode, pkt_line_flush, ObjectList,
};
use crate::ssh_url::SshUrl;

/// SSH 传输实现
pub struct SshTransport {
    user: String,
    host: String,
    port: u16,
    path: String,
}

impl SshTransport {
    pub fn from_url(url_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed =
            SshUrl::parse(url_str).ok_or_else(|| format!("Invalid SSH URL: {}", url_str))?;
        Ok(SshTransport {
            user: parsed.user,
            host: parsed.host,
            port: parsed.port,
            path: parsed.path,
        })
    }

    /// 核心方法：执行远程命令，发送输入，返回输出
    fn ssh_execute(
        &self,
        command: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let host_port = if self.port != 22 {
            format!("{}@{}:{}", self.user, self.host, self.port)
        } else {
            format!("{}@{}", self.user, self.host)
        };

        // 构建远程命令：git-upload-pack 或 git-receive-pack
        let remote_cmd = format!("{} '{}'", command, self.path);

        let mut child = Command::new("ssh")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new")
            .arg("-o")
            .arg("PasswordAuthentication=no")
            .arg("-p")
            .arg(self.port.to_string())
            .arg(&host_port)
            .arg(&remote_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn ssh: {}. Is ssh installed?", e))?;

        // 写入输入
        if !input.is_empty() {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or("SSH stdin pipe not available (process may have exited)")?;
            stdin
                .write_all(input)
                .map_err(|e| format!("Failed to write to ssh stdin: {}", e))?;
            // 关闭 stdin 以告知对方输入结束
            drop(child.stdin.take());
        }

        // 读取输出
        let mut output = Vec::new();
        let stdout = child
            .stdout
            .as_mut()
            .ok_or("SSH stdout pipe not available (process may have exited)")?;
        stdout
            .read_to_end(&mut output)
            .map_err(|e| format!("Failed to read ssh stdout: {}", e))?;

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait on ssh: {}", e))?;

        if !status.success() {
            return Err(format!(
                "SSH command '{}' on {} failed with exit code: {:?}",
                command,
                host_port,
                status.code()
            )
            .into());
        }

        Ok(output)
    }
}

impl Transport for SshTransport {
    fn discover_refs(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        // 发送空输入 → 远程 git-upload-pack 输出引用列表
        let output = self.ssh_execute("git-upload-pack", &[])?;
        let refs = parse_refs_data(&output);
        eprintln!("DEBUG SSH discover_refs: parsed {} refs", refs.len());
        for (sha, name) in &refs {
            eprintln!("  {} -> {}", &sha[..sha.len().min(7)], name);
        }
        Ok(refs)
    }

    fn fetch_objects(
        &self,
        wants: &[String],
        haves: &[String],
    ) -> Result<ObjectList, Box<dyn std::error::Error>> {
        let mut body = Vec::new();
        for want in wants {
            body.extend_from_slice(&pkt_line_encode(
                format!(
                    "want {} multi_ack_detailed no-done side-band-64k thin-pack ofs-delta agent=agit/0.1.0\n",
                    want
                )
                .as_bytes(),
            ));
        }
        for have in haves {
            body.extend_from_slice(&pkt_line_encode(format!("have {}\n", have).as_bytes()));
        }
        body.extend_from_slice(&pkt_line_flush());
        body.extend_from_slice(&pkt_line_encode(b"done\n"));
        body.extend_from_slice(&pkt_line_flush());

        let output = self.ssh_execute("git-upload-pack", &body)?;
        let pack_start = find_pack_start(&output);
        if pack_start >= output.len() {
            return Err("No packfile in SSH response".into());
        }

        parse_packfile(&output[pack_start..])
    }

    fn push_pack(
        &self,
        ref_update: &str,
        pack_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut body = Vec::new();
        let report_cap = "report-status side-band-64k agent=agit/0.1.0";
        body.extend_from_slice(&pkt_line_encode(
            format!("{}\0{}\n", ref_update, report_cap).as_bytes(),
        ));
        body.extend_from_slice(&pkt_line_flush());
        body.extend_from_slice(pack_data);

        self.ssh_execute("git-receive-pack", &body)?;

        Ok(())
    }
}

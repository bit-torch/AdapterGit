# Changelog

All notable changes to AdapterGit (agit) will be documented in this file.

---

## [v0.11.0] — Unreleased

> Blame & Reflog — 逐行追溯与引用日志

### Features
- **blame**: 逐行追溯文件每行的最后修改提交（LCS 行比对算法）
- **reflog**: 查看引用变更历史（HEAD / 分支 / 标签）

### Core
- `core/reflog`: 引用日志读写模块，支持 `.git/logs/` 解析和原子追加

### Tests
- 172 测试 (103 单元 + 69 集成)

---

## [v0.10.0] — 2026-06-21

> SSH 传输协议 — SSH Transport Protocol

### Features
- **SSH 传输协议**: 通过系统 `ssh` 命令支持 `git@host:path` 和 `ssh://` URL
- **Transport trait**: 统一 HTTP/SSH 传输抽象（discover_refs / fetch_objects / push_pack）
- **SSH URL 解析器**: 支持 SCP 格式、ssh:// 标准格式、~/.ssh/config 主机别名和通配符
- **create_transport()**: 根据 URL scheme 自动分发 HTTP 或 SSH

### Architecture
- `SshTransport`: 零额外依赖，子进程调用系统 ssh，透明继承密钥/known_hosts/agent
- `HttpTransport`: 重构为 Transport trait 实现
- clone/fetch/push 命令统一使用 Transport trait

### Tests
- 168 测试 (99 单元 + 9 兼容 + 60 集成)

---

## [v0.9.0] — 2026-06-20

> CI/CD 增强与 Git 一致性测试 — CI/CD Pipeline & Git Compatibility

### CI/CD
- 新增 **macOS 测试**：全平台矩阵 (Linux + macOS + Windows)
- 新增 **烟雾测试 (Smoke)**：端到端用户场景在 CI 中自动运行
- 新增 **Security Audit**：cargo-audit 依赖漏洞扫描
- Release 构建现在依赖于 smoke tests 通过

### Tests
- 新增 **Git 兼容性测试** (`tests/git_compat_test.rs`)：9 个对比测试
  - init / add+commit / status / branch+checkout / merge FF / log / rm+mv / tag
  - 无原生 Git 时自动跳过
- 测试总数：159 (90 单元 + 9 兼容 + 60 集成)

---

## [v0.8.0] — 2026-06-20

> 变基与遴选 — Rebase & Cherry-Pick

### Features
- `rebase` 命令: 完整变基操作 (--onto, --continue, --skip, --abort)
- `cherry-pick` 命令: 遴选单个或多提交 (--continue, --abort)
- `core::rebase` 模块: 共享的后端逻辑
- Detached HEAD commit 支持

### Tests
- 134 测试 (89 单元 + 45 集成)

---

## [v0.6.1] — 2025-06-14

> 代码审计修复版本 — Code Audit Fixes

### 🐛 Bug Fixes — 缺陷修复

- **fix(rm)**: 修复 `rm` 删除未跟踪文件的问题。现在先检查索引再删除，与 Git 行为一致。
  Fix `rm` deleting untracked files. Now checks index before deleting, matching Git behavior. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

- **fix(push)**: 修复 `push` 忽略 remote name 的问题。`get_remote_url()` 现在支持按名称匹配 `[remote "<name>"]`。
  Fix `push` ignoring the remote name. `get_remote_url()` now matches `[remote "<name>"]` by name. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

- **fix**: 传播 I/O 错误，替换 6 处 `unwrap_or_default()` 导致的静默错误吞没。涉及 `remote.rs`、`branch.rs`、`reset.rs`、`commit.rs`、`config_cmd.rs`、`remote_utils.rs`。
  Propagate I/O errors instead of silently swallowing them via `unwrap_or_default()` across 6 files. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

### ✨ Features — 新功能

- **feat(ai)**: 实现 AI 模式危险命令守卫。AI 模式下阻止执行 `push`、`stash drop`、`branch -D`、`mergetool`、`rebase`、`bisect`。
  Implement dangerous command guard for AI mode. Blocks `push`, `stash drop`, `branch -D`, `mergetool`, `rebase`, `bisect` when AI mode is active. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

### 📝 Documentation — 文档

- **docs**: 添加完整代码审计报告（20 条发现，按严重程度分类）。
  Add full code audit report with 20 findings across severity levels. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

---

## [v0.6.0] — 2025-06-13

> P1 功能增强版本 — P1 Feature Enhancements

### ✨ Features
- feat(tag): add tag CLI command for create/list/delete (P1-7)
- feat: add diff/log/rm/mv enhancements (P1-8, P1-9, P1-10)

### 🐛 Bug Fixes
- fix: propagate IO errors instead of swallowing with `unwrap_or_default()`
- style: fix clippy warnings (needless_return, collapsible_if, op_ref, for_kv_map)

### 🧪 Tests
- 119 total tests (87 unit + 32 integration), 0 failed

---

## [v0.5.5] and earlier

See [GitHub Releases](https://github.com/bit-torch/AdapterGit/releases) for earlier versions.

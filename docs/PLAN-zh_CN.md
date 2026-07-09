# AdapterGit 开发计划

[English](PLAN.md)

## 项目目标

AdapterGit (agit) 是一个专为 AI 时代设计的 Git 工具，完全从底层用 Rust 原生实现 Git 核心协议和算法。

版本: **v0.4.1** | 总体进度: **77%**

### 核心价值

- 🤖 **AI 优先**: 零 TUI 阻塞，结构化输出，AI Agent 安全调用
- 📦 **双版本分发**: Full 安装包 + Lite 单文件便携，同一套原生 Git 内核
- 🔒 **安全防护**: 危险操作拦截，`[AI-committed]` 自动标记
- ⚡ **永不卡死**: 自动跳过编辑器，非 TTY 环境友好

## 已完成 (v0.1.0 – v0.4.1)

### Phase 1: 项目初始化 ✅

| 任务 | 状态 |
|------|------|
| Rust 项目初始化 | ✅ |
| Cargo.toml 依赖 (sha1, flate2, clap, serde, anyhow, url, native-tls) | ✅ |
| 目录结构 (core/cli/commands/ai/output/config/utils) | ✅ |
| CLI 框架 (15 个子命令, clap derive) | ✅ |
| 错误处理 (anyhow + AgitError) | ✅ |

### Phase 2: 核心对象系统 ✅ (8/9)

| 任务 | 状态 |
|------|------|
| SHA-1 哈希 (hash_bytes + hash_git_object) | ✅ |
| zlib 压缩/解压 (compress + decompress + decompress_stream) | ✅ |
| Blob 对象 | ✅ |
| Tree 对象 (支持子目录) | ✅ |
| Commit 对象 (多 parent) | ✅ |
| 对象存储 (loose objects) | ✅ |
| 引用系统 (HEAD + refs CRUD) | ✅ |
| 索引文件 (DIRC v2) | ✅ |

### Phase 3: 基础命令 ✅

| 命令 | 功能 | 状态 |
|------|------|------|
| init | 初始化仓库 (.git/ + config + HEAD) | ✅ |
| add | 文件→暂存区 (递归目录, 模式检测) | ✅ |
| commit | 提交 (tree → commit → update branch) | ✅ |
| status | 区域状态 (staged/modified/deleted/untracked) | ✅ |
| log | 提交历史遍历 | ✅ |
| cat-file | 对象查看 (-t/-p) | ✅ |
| ls-tree | 树内容列表 | ✅ |
| diff | 差异比较 (LCS 算法) | ✅ |
| show | 提交/对象详情 | ✅ |

### Phase 4: AI 模式和输出 ✅ (6/7)

| 任务 | 状态 |
|------|------|
| AI 模式 (`--ai` 参数) | ✅ |
| `[AI-committed]` 自动标记 | ✅ |
| JSON 输出 (`--json`) | ✅ |
| YAML 输出 (`--yaml`) | ✅ |
| 危险操作防护 (DANGEROUS_COMMANDS) | ✅ |
| 颜色控制 (`--no-color`) | ✅ |

### Phase 5: 网络功能 ✅

| 命令 | 功能 | 状态 |
|------|------|------|
| clone | 克隆仓库 (HTTP + TLS) | ✅ |
| push | 推送 (packfile 生成) | ✅ |
| pull | fetch + merge/fast-forward | ✅ |
| fetch | 获取 (want/have 协商) | ✅ |
| remote add/list | 远程管理 | ✅ |

**协议层实现：**
- pkt-line 编解码
- HTTP Smart Transport + TLS
- Packfile 解析 + delta 解码 (ofs_delta + ref_delta)
- Ref discovery

### Phase 6: 配置 (部分) 🔨

| 任务 | 状态 |
|------|------|
| 环境变量 (AGIT_USER_NAME/EMAIL, GIT_AUTHOR_*) | ✅ |

## 待完成 (v0.4.1+)

### Phase 2 剩余

| 任务 | 优先级 |
|------|--------|
| Tag 对象 | P1 |

### Phase 4 剩余

| 任务 | 优先级 |
|------|--------|
| 命令自动转换 | P2 |

### Phase 6: 配置和扩展

| 任务 | 优先级 |
|------|--------|
| 配置文件 (.toml) | P2 |
| Git 别名 | P2 |
| Hooks | P3 |
| Submodule | P3 |

### Phase 7: 测试和发布

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 单元测试 | P0 | 🔨 (7/9 模块已覆盖) |
| 集成测试 | P0 | ⏳ |
| Git 一致性测试 | P1 | ⏳ |
| 跨平台编译 | P1 | ⏳ |
| 静态编译 (musl) | P1 | ⏳ |
| Release 构建 | P0 | ⏳ |

### Phase 8: Full/Lite 双版本分发

| 任务 | 优先级 |
|------|--------|
| Lite 单文件便携版 | P1 |
| Full .deb 安装包 | P1 |
| Full .rpm 安装包 | P2 |
| Full .msi 安装包 | P2 |
| Full .dmg 安装包 | P2 |
| CI/CD 双版本流水线 | P1 |
| GitHub Release 自动发布 | P1 |

## 技术决策

| 库 | 用途 | 决策 |
|----|------|------|
| sha1 0.10 | SHA-1 哈希 | ✅ |
| flate2 1 | zlib 压缩 | ✅ |
| clap 4 | CLI 解析 | ✅ |
| serde 1 + serde_json + serde_yaml | 结构化输出 | ✅ |
| anyhow 1 | 错误处理 | ✅ |
| url 2 | URL 解析 | ✅ |
| native-tls 0.2 | TLS/HTTPS | ✅ |
| gix / gitoxide | ❌ 不使用 (纯原生) | ❌ |
| 系统 git 命令 | ❌ 不依赖 | ❌ |

## 里程碑

| 版本 | 内容 | 状态 | 日期 |
|------|------|------|------|
| v0.1.0 | 项目骨架 + 核心对象系统 + 基础命令 | ✅ | 2025-07 |
| v0.2.0 | AI 模式 + 结构化输出 + P2 命令 | ✅ | 2025-07 |
| v0.3.0 | 网络功能 (clone/push/pull/fetch/remote) | ✅ | 2025-07 |
| **v0.4.1** | Tag + 配置文件 + 集成测试 + 分支切换清理 | ✅ 当前 | TBD |
| v1.0.0 | 完整 Git 子集 + 全平台安装包 + 文档 | 🎯 | TBD |

## 相关文档

- [架构设计](docs/ARCHITECTURE.md)
- [待办事项](../TODO.md)

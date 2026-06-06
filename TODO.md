# TODO - 待办事项

AdapterGit 项目待办事项清单。

## Phase 1: 项目初始化

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 1.1 | 初始化 Rust 项目 | P0 | ✅ 已完成 | cargo init |
| 1.2 | 配置 Cargo.toml 依赖 | P0 | ✅ 已完成 | sha1, flate2, clap, serde, anyhow, url, native-tls |
| 1.3 | 创建目录结构 | P0 | ✅ 已完成 | core/, cli/, commands/, ai/, output/, config/, utils/ |
| 1.4 | 实现基础 CLI 框架 | P0 | ✅ 已完成 | 15 个子命令, clap derive |
| 1.5 | 设置错误处理 | P1 | ✅ 已完成 | anyhow, Box<dyn Error>, AgitError |

## Phase 2: 核心对象系统

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 2.1 | SHA-1 哈希实现 | P0 | ✅ 已完成 | sha1 crate, hash_bytes + hash_git_object |
| 2.2 | zlib 压缩/解压 | P0 | ✅ 已完成 | flate2, compress + decompress + decompress_stream |
| 2.3 | Blob 对象实现 | P0 | ✅ 已完成 | new / serialize / deserialize / hash |
| 2.4 | Tree 对象实现 | P0 | ✅ 已完成 | TreeEntry + Tree, 支持子目录 |
| 2.5 | Commit 对象实现 | P0 | ✅ 已完成 | tree / parents / author / committer / message |
| 2.6 | Tag 对象实现 | P1 | ✅ 已完成 | 轻量标签 + 注释标签 |
| 2.7 | 对象存储 | P0 | ✅ 已完成 | loose objects 读写, 压缩存储 |
| 2.8 | 引用系统 | P0 | ✅ 已完成 | HEAD 符号/分离引用, 分支 CRUD, 标签 CRUD |
| 2.9 | 索引文件 | P1 | ✅ 已完成 | DIRC v2 格式 |

## Phase 3: 基础命令实现

| ID | 命令 | 功能描述 | 优先级 | 状态 |
|----|------|----------|--------|------|
| 3.1 | `init` | 初始化新仓库 | P0 | ✅ 已完成 |
| 3.2 | `add` | 添加文件到暂存区 | P0 | ✅ 已完成 |
| 3.3 | `commit` | 提交更改 (含 AI 模式) | P0 | ✅ 已完成 |
| 3.4 | `status` | 查看工作区状态 | P1 | ✅ 已完成 |
| 3.5 | `log` | 查看提交历史 | P1 | ✅ 已完成 |
| 3.6 | `cat-file` | 查看对象内容 (-t/-p) | P2 | ✅ 已完成 |
| 3.7 | `ls-tree` | 列出树对象内容 | P2 | ✅ 已完成 |
| 3.8 | `diff` | 比较差异 (LCS 算法) | P2 | ✅ 已完成 |
| 3.9 | `show` | 显示提交/对象信息 | P2 | ✅ 已完成 |

## Phase 4: AI 模式和输出

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 4.1 | AI 模式 `--ai` 参数 | P1 | ✅ 已完成 | AtomicBool 全局标志 |
| 4.2 | `[AI-committed]` 自动标记 | P1 | ✅ 已完成 | commit 命令自动添加 |
| 4.3 | JSON 输出 `--json` | P1 | ✅ 已完成 | serde_json + print_structured |
| 4.4 | YAML 输出 `--yaml` | P2 | ✅ 已完成 | serde_yaml |
| 4.5 | 危险操作防护 | P1 | ✅ 已完成 | DANGEROUS_COMMANDS 常量 |
| 4.6 | 命令自动转换 | P2 | ⏳ 待开始 | v0.4.0 |
| 4.7 | 颜色控制 `--no-color` | P2 | ✅ 已完成 | ANSI 转义序列 |

## Phase 5: 网络功能

| ID | 命令 | 功能描述 | 优先级 | 状态 |
|----|------|----------|--------|------|
| 5.1 | `clone` | 克隆仓库 (HTTP + TLS) | P1 | ✅ 已完成 |
| 5.2 | `push` | 推送到远程 | P1 | ✅ 已完成 |
| 5.3 | `pull` | fetch + merge/fast-forward | P1 | ✅ 已完成 |
| 5.4 | `fetch` | 获取更新 (negotation) | P1 | ✅ 已完成 |
| 5.5 | `remote` | 远程管理 (add/list) | P2 | ✅ 已完成 |

**协议实现：**
- pkt-line 编解码
- HTTP Smart Transport (TLS/HTTPS 支持)
- Packfile 解析 (含 ofs_delta + ref_delta 解码)
- Ref discovery (git-upload-pack)

## Phase 6: 配置和扩展

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 6.1 | 环境变量支持 | P1 | ✅ 已完成 | AGIT_USER_NAME/EMAIL, GIT_AUTHOR_* |
| 6.2 | 配置文件支持 | P2 | ✅ 已完成 | .toml 格式，全局 + 仓库级配置 |
| 6.3 | Git 别名支持 | P2 | ✅ 已完成 | 支持带参数别名 |
| 6.4 | Hooks 支持 | P3 | ⏳ 待开始 | |
| 6.5 | Submodule 支持 | P3 | ⏳ 待开始 | |

## Phase 7: 测试和发布

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 7.1 | 单元测试 | P0 | ✅ 已完成 | 55 个单元测试全部通过 |
| 7.2 | 集成测试 | P0 | ✅ 已完成 | 10 个端到端工作流测试 |
| 7.3 | 与原生 Git 一致性测试 | P1 | ⏳ 待开始 | |
| 7.4 | 跨平台编译 | P1 | ⏳ 待开始 | Linux/macOS/Windows |
| 7.5 | 静态编译 (musl) | P1 | ⏳ 待开始 | |
| 7.6 | Release 构建 | P0 | ⏳ 待开始 | |

## Phase 8: Full/Lite 双版本分发

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 8.1 | Lite 单文件便携版 | P1 | ⏳ 待开始 | |
| 8.2 | Full .deb 安装包 | P1 | ⏳ 待开始 | |
| 8.3 | Full .rpm 安装包 | P2 | ⏳ 待开始 | |
| 8.4 | Full .msi 安装包 | P2 | ⏳ 待开始 | |
| 8.5 | Full .dmg 安装包 | P2 | ⏳ 待开始 | |
| 8.6 | CI/CD 双版本流水线 | P1 | ⏳ 待开始 | |
| 8.7 | GitHub Release 自动发布 | P1 | ⏳ 待开始 | |

## 进度汇总

| Phase | 主题 | 完成度 | 状态 |
|-------|------|--------|------|
| 1 | 项目初始化 | 100% | ✅ |
| 2 | 核心对象系统 | 100% (9/9) | ✅ |
| 3 | 基础命令 | 100% (9/9) | ✅ |
| 4 | AI 模式和输出 | 86% (6/7) | ✅ |
| 5 | 网络功能 | 100% (5/5) | ✅ |
| 6 | 配置和扩展 | 60% (3/5) | ✅ |
| 7 | 测试和发布 | 33% (2/6) | 🔨 |
| 8 | Full/Lite 分发 | 0% (0/7) | ⏳ |

**总体: 41/53 ≈ 77%**

### 里程碑

| 版本 | 描述 | 状态 |
|------|------|------|
| v0.1.0 | 项目骨架 + 核心对象系统 | ✅ 已发布 |
| v0.2.0 | 本地命令 + AI 模式 | ✅ 已发布 |
| v0.3.0 | 网络功能 (clone/push/pull/fetch/remote) | ✅ 已发布 |
| v0.4.0 | Tag 对象 + 配置文件 + 集成测试 | ✅ 当前版本 |
| v1.0.0 | 完整 Git 子集 + 全平台安装包 | 🎯 目标 |

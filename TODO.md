# TODO - 待办事项

AdapterGit 项目待办事项清单。最后更新: 2026-06-20 (v0.10.0 开发中)。

## Phase 1: 项目初始化 ✅

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 1.1 | 初始化 Rust 项目 | P0 | ✅ 已完成 | cargo init |
| 1.2 | 配置 Cargo.toml 依赖 | P0 | ✅ 已完成 | sha1, flate2, clap, serde, anyhow, url, native-tls |
| 1.3 | 创建目录结构 | P0 | ✅ 已完成 | core/, cli/, commands/, ai/, output/, config/, utils/ |
| 1.4 | 实现基础 CLI 框架 | P0 | ✅ 已完成 | 24 个子命令, clap derive |
| 1.5 | 设置错误处理 | P1 | ✅ 已完成 | anyhow, Box<dyn Error>, AgitError |

## Phase 2: 核心对象系统 ✅

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
| 2.9 | 索引文件 | P1 | ✅ 已完成 | DIRC v2 格式, .gitignore 过滤 |

## Phase 3: 基础命令实现 ✅ (全部完成)

| ID | 命令 | 功能描述 | 优先级 | 状态 | 备注 |
|----|------|----------|--------|------|------|
| 3.1 | `init` | 初始化新仓库 | P0 | ✅ 已完成 |
| 3.2 | `add` | 添加文件到暂存区 (.gitignore 尊重) | P0 | ✅ 已完成 |
| 3.3 | `commit` | 提交更改 (含 AI 模式) | P0 | ✅ 已完成 |
| 3.4 | `status` | 查看工作区状态 | P1 | ✅ 已完成 |
| 3.5 | `log` | 查看提交历史 (--oneline/-n/--all) | P1 | ✅ 已完成 |
| 3.6 | `cat-file` | 查看对象内容 (-t/-p) | P2 | ✅ 已完成 |
| 3.7 | `ls-tree` | 列出树对象内容 | P2 | ✅ 已完成 |
| 3.8 | `diff` | 比较差异 (LCS + --cached/--name-only) | P2 | ✅ 已完成 |
| 3.9 | `show` | 显示提交/对象信息 | P2 | ✅ 已完成 |
| 3.10 | `branch` | 分支管理 (list/create/delete) | P1 | ✅ 已完成 | v0.5.0 |
| 3.11 | `checkout` | 分支切换 / 工作树恢复 (--force) | P1 | ✅ 已完成 | v0.5.0 |
| 3.12 | `merge` | 合并分支 (fast-forward + 3-way + 冲突标记) | P1 | ✅ 已完成 | v0.5.0 |
| 3.13 | `stash` | 暂存工作区 (push/pop/list/drop) | P2 | ✅ 已完成 | v0.5.0 |
| 3.14 | `reset` | 重置 HEAD (--soft/--mixed/--hard) | P1 | ✅ 已完成 | v0.5.0 |
| 3.15 | `rm` | 删除文件 (--cached) | P2 | ✅ 已完成 | v0.5.0 |
| 3.16 | `mv` | 移动/重命名文件 | P2 | ✅ 已完成 | v0.5.0 |
| 3.17 | `config` | 配置管理 (--global/--list/--unset/--get) | P2 | ✅ 已完成 | v0.5.0 |

**Phase 3 额外完成: 8 个命令 (branch/checkout/merge/stash/reset/rm/mv/config)**

## Phase 4: AI 模式和输出

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 4.1 | AI 模式 `--ai` 参数 | P1 | ✅ 已完成 | AtomicBool 全局标志 |
| 4.2 | `[AI-committed]` 自动标记 | P1 | ✅ 已完成 | commit 命令自动添加 |
| 4.3 | JSON 输出 `--json` | P1 | ✅ 已完成 | serde_json + print_structured |
| 4.4 | YAML 输出 `--yaml` | P2 | ✅ 已完成 | serde_yaml |
| 4.5 | 危险操作防护 | P1 | ✅ 已完成 | DANGEROUS_COMMANDS 常量 |
| 4.6 | 命令自动转换 | P2 | 🔨 部分 | AI commit message 生成已完成，NL2CMD 待开始 |
| 4.7 | 颜色控制 `--no-color` | P2 | ✅ 已完成 | ANSI 转义序列 |

## Phase 5: 网络功能 ✅

| ID | 命令 | 功能描述 | 优先级 | 状态 |
|----|------|----------|--------|------|
| 5.1 | `clone` | 克隆仓库 (HTTP + TLS) | P1 | ✅ 已完成 |
| 5.2 | `push` | 推送到远程 | P1 | ✅ 已完成 |
| 5.3 | `pull` | fetch + merge/fast-forward | P1 | ✅ 已完成 |
| 5.4 | `fetch` | 获取更新 (negotation) | P1 | ✅ 已完成 |
| 5.5 | `remote` | 远程管理 (add/list) | P2 | ✅ 已完成 |

**协议实现：**
- [x] pkt-line 编解码
- [x] HTTP Smart Transport (TLS/HTTPS)
- [x] Packfile 解析 (ofs_delta + ref_delta)
- [x] Ref discovery (git-upload-pack)
- [ ] SSH 传输协议
- [ ] Git 协议 v2

## Phase 6: 配置和扩展

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 6.1 | 环境变量支持 | P1 | ✅ 已完成 | AGIT_USER_NAME/EMAIL, GIT_AUTHOR_* |
| 6.2 | 配置文件支持 | P2 | ✅ 已完成 | .toml 格式，全局 + 仓库级 |
| 6.3 | Git 别名支持 | P2 | ✅ 已完成 | 支持带参数别名 |
| 6.4 | Hooks 支持 | P3 | ⏳ 待开始 | pre-commit, post-commit, pre-push 等 |
| 6.5 | Submodule 支持 | P3 | ⏳ 待开始 | |

## Phase 7: 测试和 CI

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 7.1 | 单元测试 | P0 | ✅ 已完成 | **87** 个单元测试全部通过 |
| 7.2 | 集成测试 | P0 | ✅ 已完成 | **32** 个端到端工作流测试 (10+22) |
| 7.3 | 与原生 Git 一致性测试 | P1 | ✅ 已完成 | 9 个对比测试 (init/add/commit/status/branch/merge/log/rm/tag) |
| 7.4 | CI 流水线 (GitHub Actions) | P1 | ✅ 已完成 | fmt + clippy + test + smoke + security on push/PR |
| 7.5 | 跨平台编译验证 | P2 | ✅ 已完成 | Linux/macOS/Windows CI matrix + 双平台 smoke 测试 |
| 7.6 | Release 构建 + 发布 | P1 | ✅ 已完成 | cargo build --release + GitHub Release on tag

## Phase 8: 高级命令

| ID | 命令 | 功能描述 | 优先级 | 状态 | 备注 |
|----|------|----------|--------|------|------|
| 8.1 | `rebase` | 变基操作（非交互式） | P1 | ✅ 已完成 | v0.8.0 |
| 8.2 | `cherry-pick` | 遴选提交 | P1 | ✅ 已完成 | v0.8.0 |
| 8.3 | `blame` | 逐行追溯 | P2 | ✅ 已完成 | v0.11.0 |
| 8.4 | `reflog` | 引用日志 | P2 | ✅ 已完成 | v0.11.0 |
| 8.5 | `bisect` | 二分查找引入 bug 的提交 | P2 | ✅ 已完成 | v0.12.0 |
| 8.6 | `grep` | 搜索工作树内容 | P3 | ⏳ 待开始 | |
| 8.7 | SSH 传输协议 | P1 | ✅ 已完成 | v0.10.0：子进程 ssh，零额外依赖 |
| 8.8 | Git 协议 v2 | P2 | ⏳ 待开始 | |

## Phase 9: 分发和打包 (原 Phase 8 重编号)

| ID | 任务 | 优先级 | 状态 | 备注 |
|----|------|--------|------|------|
| 9.1 | 静态编译 (musl) | P1 | ✅ 已完成 | v0.13.0 |
| 9.2 | Linux 安装包 (.deb/.rpm) | P2 | ✅ 已完成 | v0.13.0 (.deb via cargo-deb) |
| 9.3 | macOS 安装包 (.dmg / Homebrew) | P2 | ✅ 已完成 | v0.13.0 (二进制 tar.gz) |
| 9.4 | Windows 安装包 (.msi / Scoop) | P2 | ⏳ 待开始 | |
| 9.5 | Docker 镜像 | P2 | ✅ 已完成 | v0.13.0 |
| 9.6 | CI/CD 自动发布流水线 | P1 | ✅ 已完成 | v0.13.0 (4 平台自动发布) |

## 进度汇总

| Phase | 主题 | 完成度 | 状态 |
|-------|------|--------|------|
| 1 | 项目初始化 | 100% (5/5) | ✅ |
| 2 | 核心对象系统 | 100% (9/9) | ✅ |
| 3 | 基础命令 | 100% (17/17) | ✅ |
| 4 | AI 模式和输出 | 93% (6.5/7) | 🔨 |
| 5 | 网络功能 | 100% (5/5) | ✅ |
| 6 | 配置和扩展 | 60% (3/5) | 🔨 |
| 7 | 测试和 CI | 100% (6/6) | ✅ |
| 8 | 高级命令 | 75% (6/8) | 🔨 |
| 9 | 分发和打包 | 83% (5/6) | 🔨 |

**总体: 61/68 ≈ 90%**

### 里程碑

| 版本 | 描述 | 状态 | 关键交付 |
|------|------|------|----------|
| v0.1.0 | 项目骨架 + 核心对象 | ✅ 已发布 | init, hash, blob, tree, commit, storage |
| v0.2.0 | 本地命令 + AI 模式 | ✅ 已发布 | add, commit, status, log, diff, --ai |
| v0.3.0 | 网络功能 | ✅ 已发布 | clone, push, pull, fetch, remote, HTTP+TLS |
| v0.4.0 | Tag + 配置 | ✅ 已发布 | tag, config, 环境变量, .toml 配置 |
| v0.5.0 | 高级本地命令 | ✅ 已发布 | branch, checkout, merge, stash, reset, rm, mv |
| v0.6.0 | 安全加固 + 测试扩展 | ✅ 已发布 | 安全审计修复, 87 单元 + 32 集成 |
| v0.8.0 | 变基 + 遴选 | ✅ 已发布 | rebase, cherry-pick, 150 测试 |
| v0.9.0 | CI/CD + Smoke + Git 兼容性 | ✅ 已发布 | 三平台 CI, Smoke 测试, Git 一致性测试 |
| **v0.10.0** | **SSH 传输协议** | ✅ **已发布** | SSH 传输 (子进程), Transport trait |
| **v0.11.0** | **Blame + Reflog** | ✅ **已发布** | blame, reflog, 103 单元测试 |
| **v0.12.0** | **Bisect** | ✅ **已发布** | bisect (start/good/bad/skip/reset/log/run), 109 单元测试 |
| **v0.13.0** | **双版本分发 + AI 提交** | ✅ **已发布** | lite/full, musl, .deb, macOS, Docker, LLM commit |
| v1.0.0 | 完整 Git 子集 + 全平台分发 | 🏁 目标 | 全命令覆盖 + 多平台安装包 |

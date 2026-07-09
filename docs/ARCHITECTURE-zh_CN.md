# AdapterGit 架构设计

[English](ARCHITECTURE.md)

## 概述

AdapterGit (agit) 是一个从底层原生实现的 Git 工具，完全使用 Rust 语言编写，不依赖任何外部 Git 库或系统 Git 命令。

版本: **v0.14.0**

## 设计目标

1. **AI 优先** - 零 TUI 阻塞，结构化输出
2. **原生实现** - 从底层实现 Git 核心协议和算法
3. **便携性** - 单文件静态编译，无依赖
4. **安全性** - AI Agent 安全调用，危险操作防护

## 命令概览

```
agit init             初始化仓库
agit add <files>      添加文件到暂存区
agit commit -m <msg>  提交更改 (支持 --ai)
agit status           查看工作区状态
agit log              查看提交历史
agit diff             比较差异
agit show <ref>       显示提交/对象信息
agit cat-file <sha>  查看对象内容 (-t/-p)
agit ls-tree <sha>    列出树对象
──
本地分支与历史
agit branch <name>    创建/列出/删除分支
agit checkout <ref>   切换分支或恢复工作树文件
agit merge <branch>   合并分支 (fast-forward + 3-way)
agit rebase <branch>  变基操作
agit stash            暂存/恢复工作进度
agit reset <ref>      重置 HEAD 到指定状态
──
远程操作
agit clone <url>      克隆仓库 (HTTPS)
agit fetch [url]      获取更新
agit pull             获取并合并
agit push [remote]    推送更新
agit remote add/list  远程管理
──
高级操作
agit cherry-pick <sha> 应用指定提交到当前分支
agit blame <file>     文件每行归属追踪
agit reflog           引用日志查询
agit bisect           二分查找引入 bug 的提交
──
其他
agit tag <name>       创建/列出标签
agit rm <files>       从工作区和索引删除文件
agit mv <src> <dst>   移动或重命名文件
agit config           读取/写入仓库或全局配置
```

> 全局参数: `--ai` `--json` `--yaml` `--no-color`

## 目录结构

```
D:\AdapterGit\                   ← Cargo workspace root
├── Cargo.toml                   ← [workspace] members = ["agit-core", "agit-ai", "agit-cli"]
├── agit-core/   (lib)
│   └── src/
│       ├── lib.rs              库入口，重导出公开 API
│       ├── hash.rs             SHA-1 哈希
│       ├── compression.rs      zlib 压缩/解压
│       ├── storage.rs          对象存储 (.git/objects)
│       ├── refs.rs             引用系统 (HEAD, refs/heads/*)
│       ├── index.rs            索引文件 (.git/index)
│       ├── repo.rs             仓库工具 (find_root, ensure_dir, timestamp)
│       ├── protocol.rs         Git 智能传输协议 (HTTP + pkt-line + packfile)
│       ├── remote_utils.rs     网络命令共享工具
│       ├── checkout.rs         分支切换与树恢复
│       ├── merge.rs            三方合并 + fast-forward + 冲突标记
│       ├── ignore.rs           .gitignore 解析器
│       ├── objects/            Git 对象
│       │   ├── blob.rs         Blob 对象
│       │   ├── tree.rs         Tree 对象
│       │   ├── commit.rs       Commit 对象
│       │   └── tag.rs          Tag 对象 (feature-gated)
│       ├── config/mod.rs       配置 (环境变量 > 文件 > 默认值)
│       └── utils/
│           └── error.rs        错误类型 (AgitError)
├── agit-ai/     (lib)
│   └── src/lib.rs              LlmConfig、chat_completion、generate_commit_message
└── agit-cli/    (bin → agit)
    ├── Cargo.toml              依赖：agit-core, agit-ai
    ├── src/
    │   ├── main.rs             入口点 + cat-file dispatch
    │   ├── cli/mod.rs          clap 命令定义 (29 个子命令)
    │   ├── commands/           命令实现 (每个命令一个文件)
    │   ├── ai/mod.rs           AI 模式标志 + DANGEROUS_COMMANDS 列表
    │   └── output/mod.rs       JSON/YAML/颜色 模式标志 + 输出格式化
    └── tests/                  (集成测试)
```

## 核心模块详解

### 1. 对象系统 (core/objects)

```rust
// Blob: 文件内容
pub struct Blob { pub content: Vec<u8> }

// Tree: 目录快照
pub struct Tree { pub entries: Vec<TreeEntry> }
pub struct TreeEntry { pub mode: String, pub name: String, pub sha1: String }

// Commit: 提交记录
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}
```

每个对象通过 `hash_git_object(type, content)` 计算 SHA-1：`SHA1("type size\0content")`

### 2. 存储系统 (core/storage)

Loose objects 存储在 `.git/objects/{sha1[0..2]}/{sha1[2..]}`，写入前 zlib 压缩。

```rust
pub fn write_object(repo, obj_type, content) -> Result<()>
pub fn read_object(repo, sha1) -> Result<(type, content)>
pub fn object_exists(repo, sha1) -> bool
```

### 3. 引用系统 (core/refs)

```
.git/HEAD              # 符号引用或 SHA-1
.git/refs/heads/*      # 本地分支
.git/refs/tags/*       # 标签
.git/refs/remotes/*    # 远程跟踪分支
```

### 4. 索引文件 (core/index)

DIRC v2 格式：`"DIRC" + version + entry_count + entries[] + SHA1`

### 5. 传输协议 (core/protocol)

| 组件 | 功能 |
|------|------|
| pkt-line | 4 字节 hex 长度前缀协议 |
| HttpTransport | HTTP Smart Protocol (GET/POST) |
| TransportStream | TCP + TLS 统一抽象 |
| parse_packfile | Packfile 解析, 含 delta 解码 |

协议流程：
```
discover_refs → clone_full / fetch_objects → parse_packfile → delta 重建
push: collect_local_objects → generate_pack → push_pack
```

### 6. 远程工具 (core/remote_utils)

| 函数 | 用途 |
|------|------|
| write_objects | 批量写入 loose objects |
| apply_tree | 递归检出目录树 |
| get_remote_url | section-aware config 解析 |
| get_current_branch | HEAD 解析 |
| collect_recent_commits | 最近 N 个祖先 |
| collect_local_objects_for_push | 全父链遍历收集 |
| resolve_commit_to_tree | 提取 tree SHA-1 |

## 网络命令流程

```
clone:  discover_refs → clone_full → parse_packfile → write_objects → checkout
fetch:  discover_refs → fetch_objects → write_objects → update remote ref
push:   discover_refs → collect_local_objects → generate_pack → push_pack
pull:   fetch → find_common_ancestor → fast_forward 或 merge_changes
```

`pull` 合并前检查工作树是否干净，如有未提交更改则中止。

## 错误处理

```rust
pub enum AgitError {
    Io(std::io::Error),
    ObjectNotFound(String),
    InvalidObject(String),
    InvalidRef(String),
    CompressionError(String),
    RepoNotFound(PathBuf),
    NotAGitRepo(PathBuf),
    Other(anyhow::Error),
}
```

网络命令使用 `Box<dyn Error>`，核心模块可独立使用 `AgitError`。

## 技术栈

### Crate 分层

```
agit-cli (bin)  →  agit-ai (lib)    AI 功能
                →  agit-core (lib)   Git 核心逻辑
agit-ai (lib)   →  reqwest           HTTP 客户端 (LLM API)
agit-core (lib)  →  (pure Rust, no external Git dep)
```

| 依赖 | 用途 | 所属 |
|------|------|------|
| sha1 0.10 | SHA-1 哈希 | agit-core |
| flate2 1 | zlib 压缩 | agit-core |
| clap 4 | CLI 解析 | agit-cli |
| serde 1 + serde_json + serde_yaml | 结构化输出 | agit-cli |
| anyhow 1 | 错误处理 | agit-core / agit-cli |
| url 2 | URL 解析 | agit-core |
| native-tls 0.2 | TLS/HTTPS | agit-core |
| reqwest 0.11 | HTTP 客户端 | agit-ai |
| tokio 1 | 异步运行时 | agit-ai |

## 实现阶段

| 版本 | 范围 | 状态 |
|------|------|------|
| v0.1.0 | Phase 1-2: 项目骨架 + 核心对象 | ✅ |
| v0.2.0 | Phase 3-4: 本地命令 + AI 模式 | ✅ |
| v0.3.0 | Phase 5: 网络功能 | ✅ |
| v0.4.1 | Tag + 配置文件 + 集成测试 + 分支切换清理 | ✅ |
| v0.14.0 | Workspace 拆分 (agit-core/agit-ai/agit-cli) + AI 提交信息 + 29 子命令 | ✅ 当前 |

## 参考资料

- [Git 内部原理](https://git-scm.com/book/zh/v2/Git-内部原理)
- [Pro Git 书籍](https://git-scm.com/book/zh/v2)
- [Git Pack Format](https://git-scm.com/docs/pack-format)

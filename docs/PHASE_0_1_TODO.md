# Phase 0 & 1 待办事项

> 细化 AdapterGit 项目初始化 + 核心对象系统开发任务

## Phase 0: 项目初始化

### P0 任务
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 0.1 | `cargo init --name agit` | ✅ | 初始化 Rust 项目 |
| 0.2 | 配置 `Cargo.toml` | ✅ | clap, serde_json, flate2, sha1, anyhow |
| 0.3 | 创建 src/ 目录结构 | ✅ | core/ cli/ ai/ output/ config/ utils/ |
| 0.4 | 创建 `src/main.rs` | ✅ | 入口文件 |
| 0.5 | 定义 `AgitError` 错误类型 | ✅ | `src/utils/error.rs` |
| 0.6 | 添加日志支持 | ✅ | 最小日志宏 |
| 0.7 | 编译验证 | ✅ | `cargo build` + `cargo check` |

### P1 任务
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 1.1 | CLI 全局参数 | ✅ | `--ai` / `--json` / `--yaml` / `--no-color` |
| 1.2 | 命令枚举定义 | ✅ | Init / Add / Commit / Status / Log |
| 1.3 | 子命令参数 | ✅ | commit -m / --ai |
| 1.4 | 命令路由分发 | ✅ | main.rs match dispatch |
| 1.5 | 占位处理函数 | ✅ | "not implemented yet" |
| 1.6 | 帮助信息验证 | ✅ | `agit --help` 正确 |

---

## Phase 1: 核心对象系统

### P0 任务 — SHA-1 哈希
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 2.1 | 创建 `src/core/hash.rs` | ✅ | |
| 2.2 | `hash_bytes()` | ✅ | 返回 40 位 hex 字符串 |
| 2.3 | `hash_git_object()` | ✅ | `{type} {len}\0{content}` 格式 |
| 2.4 | 单元测试 | ✅ | 与已知 SHA-1 对比 |
| 2.5 | 交叉验证 | ✅ | 与 `git hash-object` 对比 |

### P0 任务 — zlib 压缩
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 3.1 | 创建 `src/core/compression.rs` | ✅ | |
| 3.2 | `compress()` | ✅ | flate2 zlib 压缩 |
| 3.3 | `decompress()` | ✅ | flate2 zlib 解压 |
| 3.4 | 回合测试 | ✅ | 压缩→解压→原始数据 |
| 3.5 | 交叉验证 | ✅ | 与原生 git 对象互换 |

### P0 任务 — Blob 对象
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 4.1 | 创建 `src/core/objects/blob.rs` | ✅ | |
| 4.2 | `struct Blob` | ✅ | `content: Vec<u8>` |
| 4.3 | `Blob::new()` | ✅ | |
| 4.4 | `Blob::serialize()` | ✅ | `blob {len}\0{content}` |
| 4.5 | `Blob::deserialize()` | ✅ | |
| 4.6 | `Blob::hash()` | ✅ | |
| 4.7 | 单元测试 | ✅ | 与 `git hash-object` 一致 |

### P0 任务 — Tree 对象
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 5.1 | 创建 `src/core/objects/tree.rs` | ✅ | |
| 5.2 | `struct TreeEntry` | ✅ | mode / name / sha1 |
| 5.3 | `struct Tree` | ✅ | `entries: Vec<TreeEntry>` |
| 5.4 | `Tree::serialize()` | ✅ | |
| 5.5 | `Tree::deserialize()` | ✅ | |
| 5.6 | `Tree::hash()` | ✅ | |
| 5.7 | 单元测试 | ✅ | 与 `git ls-tree` 一致 |

### P0 任务 — Commit 对象
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 6.1 | 创建 `src/core/objects/commit.rs` | ✅ | |
| 6.2 | `struct Commit` | ✅ | tree / parents / author / committer / message |
| 6.3 | `Commit::serialize()` | ✅ | |
| 6.4 | `Commit::deserialize()` | ✅ | |
| 6.5 | `Commit::hash()` | ✅ | |
| 6.6 | 单元测试 | ✅ | 与原生 git 一致 |

### P0 任务 — 对象存储
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 7.1 | 创建 `src/core/storage.rs` | ✅ | |
| 7.2 | `write_object()` | ✅ | 写入 .git/objects/xx/xxx... |
| 7.3 | `read_object()` | ✅ | 读取并解压返回 (type, content) |
| 7.4 | 路径规则 | ✅ | `{sha1[0..2]}/{sha1[2..]}` |
| 7.5 | 回合测试 | ✅ | 写→读→验证 |

### P1 任务 — 引用系统
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 8.1 | 创建 `src/core/refs.rs` | ✅ | |
| 8.2 | `read_head()` | ✅ | symbolic + detached HEAD |
| 8.3 | `write_head()` | ✅ | |
| 8.4 | `read_ref()` | ✅ | refs/heads/* refs/tags/* |
| 8.5 | `write_ref()` | ✅ | |
| 8.6 | 单元测试 | ✅ | |

### P1 任务 — 索引文件
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 9.1 | 创建 `src/core/index.rs` | ✅ | |
| 9.2 | `struct IndexEntry` | ✅ | ctime/mtime/dev/ino/mode/uid/gid/size/sha1/flags/path |
| 9.3 | `struct Index` | ✅ | version 2 + entries |
| 9.4 | 二进制序列化/反序列化 | ✅ | |
| 9.5 | 单元测试 | ✅ | |

### P2 任务 — 集成验证
| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 10.1 | `cargo test` | ✅ | 全部通过 (40 tests) |
| 10.2 | `cargo clippy` | ✅ | 无严重警告 |
| 10.3 | `cargo fmt --check` | ✅ | 格式正确 |
| 10.4 | 原生 git 交叉验证 | ✅ | hash-object / cat-file / ls-tree |
| 10.5 | 更新本文档状态 | ✅ | |

---

## 依赖关系图

```
Task 2 (SHA-1)  ──┐
Task 3 (zlib)   ──┼──→ Task 4 (Blob) ──┐
                   │                     │
                   └──→ Task 5 (Tree) ──┼──→ Task 7 (Storage)
                                        │
                           Task 6 (Commit)┘
                                       
Task 8 (Refs)   ←─── Task 1 (Project)
Task 9 (Index)  ←─── Task 1 + Task 4
Task 10 (Verify)←─── All above + Task 1 CLI
```

## 优先级说明

- **P0**: 必须实现，MVP 基础
- **P1**: 重要，提升可用性
- **P2**: 补充验证和测试

## 状态图例

- ⏳ 待开始
- 🔨 进行中
- ✅ 已完成
- ❌ 已取消

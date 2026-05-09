# Tasks: Phase 0 & 1 实现

## Task 0: 编写 docs/PHASE_0_1_TODO.md
- [x] 在 docs/ 下创建 PHASE_0_1_TODO.md
- [x] 列出 Phase 0 所有子任务和状态
- [x] 列出 Phase 1 所有子任务和状态
- [x] 标注优先级 P0/P1/P2
- [x] 标注当前状态（待开始/进行中/已完成）

## Task 1: Phase 0 - 项目初始化
- [x] 1.1 用 `cargo init --name agit` 初始化 Rust 项目
- [x] 1.2 配置 Cargo.toml 依赖项（clap + serde/serde_json + flate2 + sha1 + anyhow）
- [x] 1.3 创建 src/ 目录结构
  - [x] src/core/mod.rs
  - [x] src/cli/mod.rs
  - [x] src/ai/mod.rs
  - [x] src/output/mod.rs
  - [x] src/config/mod.rs
  - [x] src/utils/mod.rs
- [x] 1.4 创建 src/main.rs 入口文件
- [x] 1.5 定义统一错误类型 AgitError（src/utils/error.rs）
- [x] 1.6 添加日志宏或最小日志支持
- [x] 1.7 验证 `cargo build` 编译成功、`cargo check` 无警告

## Task 2: Phase 0 - CLI 命令路由
- [x] 2.1 用 clap derive 模式定义全局参数（--ai / --json / --yaml / --no-color）
- [x] 2.2 定义命令枚举（Init / Add / Commit / Status / Log / Clone）
- [x] 2.3 定义各命令的子参数（如 commit 的 -m / --ai）
- [x] 2.4 实现命令路由分发（main.rs 中 match 命令）
- [x] 2.5 暂存占位处理函数（打印 "not implemented yet"）
- [x] 2.6 验证 `cargo run -- --help` 正确输出帮助信息
- [x] 2.7 验证 `cargo run -- init` 输出 "not implemented yet"

## Task 3: Phase 1 - SHA-1 哈希
- [x] 3.1 创建 src/core/hash.rs
- [x] 3.2 实现 `hash_bytes(data: &[u8]) -> String` 返回 40 位十六进制字符串
- [x] 3.3 实现 `hash_git_object(obj_type: &str, content: &[u8]) -> String`
  - [x] 计算 `{obj_type} {len}\0{content}` 的 SHA-1
- [x] 3.4 添加单元测试验证与已知 SHA-1 字符串一致
- [x] 3.5 验证与原生 `git hash-object` 输出一致

## Task 4: Phase 1 - zlib 压缩/解压
- [x] 4.1 创建 src/core/compression.rs
- [x] 4.2 实现 `compress(data: &[u8]) -> Result<Vec<u8>>`
- [x] 4.3 实现 `decompress(data: &[u8]) -> Result<Vec<u8>>`
- [x] 4.4 添加压缩/解压回合测试
- [x] 4.5 验证与原生 git cat-file 可互换读取

## Task 5: Phase 1 - Blob 对象
- [x] 5.1 创建 src/core/objects/blob.rs
- [x] 5.2 定义 `struct Blob { content: Vec<u8> }`
- [x] 5.3 实现 `Blob::new(content: Vec<u8>) -> Self`
- [x] 5.4 实现 `Blob::serialize(&self) -> Vec<u8>` 生成 `blob {len}\0{content}`
- [x] 5.5 实现 `Blob::deserialize(data: &[u8]) -> Result<Self>` 解析 blob 数据
- [x] 5.6 实现 `Blob::hash(&self) -> String` 计算 blob SHA-1
- [x] 5.7 添加单元测试

## Task 6: Phase 1 - Tree 对象
- [x] 6.1 创建 src/core/objects/tree.rs
- [x] 6.2 定义 `struct TreeEntry { mode: String, name: String, sha1: String }`
- [x] 6.3 定义 `struct Tree { entries: Vec<TreeEntry> }`
- [x] 6.4 实现 `Tree::serialize(&self) -> Vec<u8>` 生成 tree 内容
- [x] 6.5 实现 `Tree::deserialize(data: &[u8]) -> Result<Self>` 解析 tree
- [x] 6.6 实现 `Tree::hash(&self) -> String`
- [x] 6.7 添加单元测试，验证与 git mktree / git ls-tree 一致

## Task 7: Phase 1 - Commit 对象
- [x] 7.1 创建 src/core/objects/commit.rs
- [x] 7.2 定义 `struct Commit { tree: String, parents: Vec<String>, author: String, committer: String, message: String }`
- [x] 7.3 实现 `Commit::serialize(&self) -> Vec<u8>`
- [x] 7.4 实现 `Commit::deserialize(data: &[u8]) -> Result<Self>`
- [x] 7.5 实现 `Commit::hash(&self) -> String`
- [x] 7.6 添加单元测试

## Task 8: Phase 1 - 对象存储
- [x] 8.1 创建 src/core/storage.rs
- [x] 8.2 实现 `write_object(repo: &Path, obj_type: &str, content: &[u8]) -> Result<String>` 返回 SHA-1
- [x] 8.3 实现 `read_object(repo: &Path, sha1: &str) -> Result<(String, Vec<u8>)>` 返回 (type, content)
- [x] 8.4 对象存放路径: `.git/objects/{sha1[0..2]}/{sha1[2..]}`
- [x] 8.5 添加读写回合测试

## Task 9: Phase 1 - 引用系统
- [x] 9.1 创建 src/core/refs.rs
- [x] 9.2 实现 `read_head(repo: &Path) -> Result<String>`
  - [x] 解析 `ref: refs/heads/xxx` 格式
  - [x] 解析直接 SHA-1（detached HEAD）
- [x] 9.3 实现 `write_head(repo: &Path, target: &str)`
- [x] 9.4 实现 `read_ref(repo: &Path, name: &str) -> Result<String>`
- [x] 9.5 实现 `write_ref(repo: &Path, name: &str, sha1: &str)`
- [x] 9.6 支持 refs/heads/* 和 refs/tags/*
- [x] 9.7 添加单元测试

## Task 10: Phase 1 - 索引文件
- [x] 10.1 创建 src/core/index.rs
- [x] 10.2 定义 `struct IndexEntry`（ctime, mtime, dev, ino, mode, uid, gid, size, sha1, flags, path）
- [x] 10.3 定义 `struct Index`（version, entries）
- [x] 10.4 实现 Index 二进制序列化/反序列化
- [x] 10.5 添加单元测试

## Task 11: 集成验证
- [x] 11.1 运行 `cargo test` 所有测试通过 (40 passed)
- [x] 11.2 运行 `cargo clippy` 无严重警告 (仅 dead_code，模块尚未接入 CLI)
- [x] 11.3 运行 `cargo fmt --check` 格式正确
- [x] 11.4 与原生 git hash-object 交叉验证
- [x] 11.5 更新 docs/PHASE_0_1_TODO.md 状态

# Task Dependencies
- [Task 3] depends on [Task 1]
- [Task 4] depends on [Task 1]
- [Task 5] depends on [Task 3, Task 4]
- [Task 6] depends on [Task 3]
- [Task 7] depends on [Task 3, Task 6]
- [Task 8] depends on [Task 3, Task 4, Task 5, Task 6, Task 7]
- [Task 9] depends on [Task 1]
- [Task 10] depends on [Task 1, Task 5]
- [Task 11] depends on [Task 2, Task 3, Task 4, Task 5, Task 6, Task 7, Task 8, Task 9, Task 10]
- [Task 0] can be done anytime, no dependencies

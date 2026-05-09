# Phase 0 & 1: 项目初始化 + 核心对象系统 Spec

## Why
AdapterGit 需要从底层原生实现 Git 核心协议和算法。Phase 0 搭建 Rust 项目骨架和 CLI 框架，Phase 1 实现 Git 对象存储核心——这是所有本地命令（init/add/commit/status/log）的基础。

## What Changes
- 创建 Rust 二进制项目，配置 Cargo.toml 依赖
- 建立 src/ 目录结构（core/cli/ai/output/config/utils）
- 实现 CLI 命令路由框架（clap），支持 --ai / --json / --yaml / --no-color 全局参数
- 实现 SHA-1 哈希工具模块
- 实现 zlib 压缩/解压工具模块
- 实现 Git 对象模型（Blob / Tree / Commit）
- 实现松散对象存储（.git/objects 读写）
- 实现引用系统（HEAD / refs/heads / refs/tags）
- 实现 .git/index 索引文件
- 统一错误类型和日志系统
- 根目录输出一份完整 TODOlist 到 docs/PHASE_0_1_TODO.md

## Impact
- Affected specs: 无（第一个规格）
- Affected code: 全新项目，无现有代码影响

## ADDED Requirements

### Requirement: Phase 0 - Rust 项目骨架
系统 SHALL 提供一个可编译的 Rust 二进制项目，包含完整的依赖声明和目录结构。

#### Scenario: 项目初始化成功
- **WHEN** 执行 `cargo init` 并配置 Cargo.toml
- **THEN** `cargo build` 编译成功
- **AND** `cargo check` 无警告

#### Scenario: 目录结构完整
- **WHEN** 查看 src/ 目录
- **THEN** 存在 core/ cli/ ai/ output/ config/ utils/ 子目录
- **AND** 每个子目录包含 mod.rs

### Requirement: Phase 0 - CLI 命令路由
系统 SHALL 解析命令行参数并路由到对应命令处理函数。

#### Scenario: 无参数调用
- **WHEN** 执行 `agit`
- **THEN** 输出帮助信息
- **AND** 显示可用命令列表

#### Scenario: 全局参数解析
- **WHEN** 执行 `agit --ai commit -m "msg"`
- **THEN** AI 模式标志被正确设置
- **WHEN** 执行 `agit --json status`
- **THEN** 输出格式标志被正确设置

#### Scenario: 未知命令
- **WHEN** 执行 `agit unknown-command`
- **THEN** 输出错误信息并提示 `agit --help`

### Requirement: Phase 1 - SHA-1 哈希
系统 SHALL 使用 sha1 crate 计算 SHA-1 哈希值。

#### Scenario: 已知输入哈希
- **WHEN** 对 "hello" 计算 SHA-1
- **THEN** 返回 `aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d`

#### Scenario: Git 对象哈希
- **WHEN** 对 `blob 11\0hello world` 计算 SHA-1
- **THEN** 返回 `3b18e512dba79e4c8300dd08aeb37f8e728b8dad`

### Requirement: Phase 1 - zlib 压缩/解压
系统 SHALL 使用 flate2 crate 对数据进行 zlib 压缩和解压。

#### Scenario: 压缩后解压还原
- **WHEN** 对任意字节序列压缩后再解压
- **THEN** 解压结果与原始数据完全一致

#### Scenario: 压缩比正常
- **WHEN** 压缩一个 1KB 文本文件
- **THEN** 压缩后大小明显小于原始大小

### Requirement: Phase 1 - Blob 对象
系统 SHALL 支持创建和读取 Git Blob 对象。

#### Scenario: 创建 Blob
- **WHEN** 用文件内容创建 Blob 对象
- **THEN** 生成正确的 blob 头 + zlib 压缩数据
- **AND** SHA-1 哈希与原生 git hash-object 结果一致

#### Scenario: 读取 Blob
- **WHEN** 从 .git/objects 读取 blob 对象
- **THEN** 解压后还原原始内容

### Requirement: Phase 1 - Tree 对象
系统 SHALL 支持创建和解析 Git Tree 对象。

#### Scenario: 创建单层 Tree
- **WHEN** 创建一个包含 3 个文件的 tree 对象
- **THEN** 每个条目包含 mode / name / SHA-1
- **AND** 序列化格式与原生 Git 一致

#### Scenario: Tree SHA-1 一致性
- **WHEN** 对相同目录结构生成 tree
- **THEN** SHA-1 与 `git ls-tree` 输出一致

### Requirement: Phase 1 - Commit 对象
系统 SHALL 支持创建和解析 Git Commit 对象。

#### Scenario: 创建 Commit
- **WHEN** 创建 commit（tree + parent + author + message）
- **THEN** 生成正确的 commit 对象格式
- **AND** SHA-1 与原生 git 一致

#### Scenario: 解析 Commit
- **WHEN** 读取 commit 对象
- **THEN** 正确提取 tree、parent、author、committer、message 字段

### Requirement: Phase 1 - 对象存储
系统 SHALL 支持向 .git/objects 写入和读取松散对象。

#### Scenario: 写入对象
- **WHEN** 写入一个 blob 对象
- **THEN** 在 .git/objects/xx/xxx... 创建对应文件
- **AND** 文件内容为 zlib 压缩后的对象数据

#### Scenario: 读取不存在的对象
- **WHEN** 读取不存在的 SHA-1 对象
- **THEN** 返回 ObjectNotFound 错误

### Requirement: Phase 1 - 引用系统
系统 SHALL 支持读写 Git 引用（HEAD、分支、标签）。

#### Scenario: 读取 HEAD
- **WHEN** 仓库在 main 分支
- **THEN** HEAD 内容为 `ref: refs/heads/main`

#### Scenario: 创建分支
- **WHEN** 创建名为 "feature" 的分支指向某 commit
- **THEN** refs/heads/feature 文件包含该 commit SHA-1

#### Scenario: 读取标签
- **WHEN** 仓库有 v1.0.0 标签
- **THEN** 读取 refs/tags/v1.0.0 获得对应 commit SHA-1

### Requirement: Phase 1 - 索引文件
系统 SHALL 支持读写 .git/index 索引文件。

#### Scenario: 创建空索引
- **WHEN** 初始化空仓库
- **THEN** .git/index 存在且包含有效签名

#### Scenario: 添加文件到索引
- **WHEN** 向索引添加文件条目（path, blob_sha, mode, stage）
- **THEN** 索引正确序列化并可反序列化恢复

#### Scenario: 索引一致性
- **WHEN** 用 agit 创建的索引
- **THEN** 原生 git 可以正常读取和使用

# AdapterGit 架构设计

## 概述

AdapterGit (agit) 是一个从底层原生实现的 Git 工具，完全使用 Rust 语言编写，不依赖任何外部 Git 库（如 gitoxide）或系统 Git 命令。

## 设计目标

1. **AI 优先** - 零 TUI 阻塞，结构化输出
2. **原生实现** - 从底层实现 Git 核心协议和算法
3. **便携性** - 单文件静态编译，无依赖
4. **安全性** - AI Agent 安全调用，防止误操作

## 核心技术挑战

Git 的底层实现涉及多个复杂层次：

| 层级 | 内容 | 复杂度 |
|------|------|--------|
| **对象模型** | blob, tree, commit, tag 对象 | ⭐⭐ |
| **存储系统** | loose objects, pack files, index | ⭐⭐⭐ |
| **引用系统** | refs, HEAD, branches, tags | ⭐⭐ |
| **差异算法** | diff, patch generation | ⭐⭐⭐ |
| **压缩算法** | zlib 压缩, delta 压缩 | ⭐⭐⭐ |
| **传输协议** | HTTP(S) smart protocol, SSH | ⭐⭐⭐⭐ |

## 系统架构

```
┌─────────────────────────────────────────┐
│             AI Agent / Script           │
└─────────────────┬───────────────────────┘
                  │ JSON / 结构化输出
┌─────────────────▼───────────────────────┐
│              agit (适配层)               │
│  ┌───────────────────────────────────┐  │
│  │  TUI 消除  │ 便携封装 │ AI 安全    │  │
│  └─────────────────┬─────────────────┘  │
└─────────────────┬───────────────────────┘
                  │ 纯 Rust 原生实现
┌─────────────────▼───────────────────────┐
│         原生 Git 核心实现 (Pure Rust)    │
└─────────────────────────────────────────┘
```

## 目录结构

```
src/
├── core/              # 核心 Git 算法
│   ├── objects/       # Git 对象 (blob, tree, commit, tag)
│   ├── storage/       # 对象存储 (.git/objects)
│   ├── pack/          # Pack 文件读写
│   ├── index/         # 索引文件操作
│   ├── diff/          # Diff 和 Patch 算法
│   └── hash/          # SHA-1 哈希实现
├── refs/              # 引用管理
│   ├── heads/         # 分支 refs/heads/*
│   ├── tags/          # 标签 refs/tags/*
│   └── packed_refs/   # 打包引用
├── protocol/          # Git 协议
│   ├── client/        # fetch/push 客户端
│   └── server/        # receive-pack/upload-pack
├── cli/               # 命令行解析
│   ├── commands/      # 命令定义
│   ├── parser/        # 参数解析
│   └── help/         # 帮助系统
├── ai/                # AI 模式
│   ├── tagger/       # 自动标记 [AI-committed]
│   ├── safety/       # 危险操作防护
│   └── converter/    # 命令转换
├── output/            # 格式化输出
│   ├── json/         # JSON 输出
│   ├── yaml/         # YAML 输出
│   └── text/         # 文本输出
├── config/            # 配置管理
│   ├── env/          # 环境变量
│   ├── file/         # 配置文件
│   └── loader/       # 配置加载
└── utils/            # 工具函数
    ├── error/        # 错误处理
    ├── path/         # 路径操作
    └── crypto/       # 加密工具
```

## 核心模块详解

### 1. 对象系统 (core/objects)

Git 的核心是基于内容寻址的对象存储。

#### 对象类型

```rust
pub enum ObjectType {
    Blob,     // 文件内容
    Tree,     // 目录快照
    Commit,   // 提交记录
    Tag,      // 标签引用
}
```

#### 对象结构

- **Blob**: 原始文件内容
- **Tree**: 包含多个条目，每个条目指向一个 blob 或子树
- **Commit**: 指向一个 tree，包含作者、提交者、消息和父提交
- **Tag**: 指向任意 Git 对象的符号引用

### 2. 存储系统 (core/storage)

#### Loose Objects

```
.git/objects/
├── ab/
│   └── cdef1234...  # SHA-1 前两位为目录，后38位为文件名
└── ...
```

#### Pack Files

用于高效存储大量对象：
- 使用 delta 压缩减少空间
- 创建 .idx 索引文件加速查找

### 3. 引用系统 (refs)

```
.git/
├── HEAD              # 当前分支指针
├── refs/
│   ├── heads/        # 本地分支
│   │   ├── main
│   │   └── develop
│   └── tags/         # 标签
│       └── v1.0.0
├── packed-refs       # 打包的引用
└── ORIG_HEAD         # 操作前的 HEAD
```

### 4. 压缩算法

Git 使用 zlib 压缩对象：

```rust
// 压缩流程
content -> zlib::compress() -> compressed -> write to .git/objects

// 解压缩流程
read from .git/objects -> zlib::decompress() -> content
```

### 5. SHA-1 哈希

每个 Git 对象通过 SHA-1 哈希标识：

```rust
sha1 = SHA1(header + content)
// header = "blob {size}\0"
```

## 关键技术点

### 1. 对象存储

- **SHA-1 哈希计算**: 唯一标识每个对象
- **zlib 压缩/解压**: 存储优化
- **对象序列化/反序列化**: 持久化和读取

### 2. Pack 文件

- **Delta 压缩算法**: 存储增量差异
- **Pack 索引 (.idx)**: 快速对象查找
- **对象查找优化**: 二分查找

### 3. 协议层 (Phase 2)

- **Git HTTP 协议** (smart protocol)
- **refs 发现和更新**
- **对象传输**

## 实现优先级

### Phase 1: 本地操作 (当前)

| 优先级 | 功能 | 描述 |
|--------|------|------|
| P0 | init | 初始化仓库 |
| P0 | add | 添加文件到暂存区 |
| P0 | commit | 提交更改 |
| P1 | status | 查看状态 |
| P1 | log | 查看提交历史 |
| P2 | cat-file | 查看对象内容 |
| P2 | ls-tree | 查看树对象 |
| P2 | diff | 比较差异 |

### Phase 2: 网络操作 (后续)

| 优先级 | 功能 | 描述 |
|--------|------|------|
| P1 | clone | 克隆仓库 |
| P1 | push | 推送到远程 |
| P1 | pull/fetch | 拉取更新 |

## 错误处理

统一的错误类型：

```rust
pub enum AgitError {
    ObjectNotFound(String),
    InvalidObject,
    IoError(std::io::Error),
    CompressionError,
    InvalidRef(String),
    MergeConflict,
    // ...
}
```

## 性能考虑

1. **延迟加载**: 不一次性加载所有对象
2. **缓存**: 使用 LRU 缓存频繁访问的对象
3. **批量操作**: 合并多个小操作为批量操作
4. **增量索引**: 避免全量重建索引

## 测试策略

1. **单元测试**: 每个模块独立测试
2. **集成测试**: 测试完整的 Git 工作流
3. **一致性测试**: 与原生 Git 输出对比
4. **模糊测试**: 随机输入测试鲁棒性

## 参考资料

- [Git 内部原理](https://git-scm.com/book/zh/v2/Git-内部原理-Git-对象)
- [Git 对象模型](https://github.com/git/git)
- [Pro Git 书籍](https://git-scm.com/book/zh/v2)

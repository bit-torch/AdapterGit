# Good First Issues - 新手任务

欢迎 contributing！以下是适合新手贡献者的任务列表。这些任务经过筛选，适合了解 Rust 但不熟悉项目的新人。

## 如何选择任务

1. 查看任务列表，选择感兴趣的
2. 在 issue 中留言领取
3. Fork 项目并创建分支
4. 完成开发后提交 PR

---

## 🎯 入门任务 (Easy)

### 1. 完善错误消息
**优先级**: P2  
**模块**: cli  
**描述**: 改进错误消息，使用户更容易理解问题  
**难度**: ⭐ (入门)  
**要求**: Rust 基础  

**示例改进**:
```rust
// 之前
Err("Object not found")

// 之后
Err(format!("Object '{}' not found in repository", object_id))
```

### 2. 添加更多帮助信息
**优先级**: P2  
**模块**: cli  
**描述**: 为命令添加更详细的帮助信息和使用示例  
**难度**: ⭐ (入门)  
**要求**: 文档写作  

**任务内容**:
- 为每个命令添加 `--help` 示例
- 添加常见用法示例
- 添加故障排除指南

### 3. 改进代码注释
**优先级**: P3  
**模块**: 通用  
**描述**: 在关键代码处添加或改进注释  
**难度**: ⭐ (入门)  
**要求**: 理解代码逻辑  

---

## 📝 文档任务 (Documentation)

### 4. 编写核心概念教程
**优先级**: P1  
**模块**: docs  
**描述**: 编写 Git 内部原理的教程文档  
**难度**: ⭐⭐ (简单)  
**要求**: 了解 Git 基本概念  

**内容建议**:
- Git 对象模型介绍
- SHA-1 哈希解释
- .git 目录结构
- Git 引用系统

**相关文件**: `docs/ARCHITECTURE.md`

### 5. 添加使用示例
**优先级**: P2  
**模块**: docs  
**描述**: 为每个命令添加完整使用示例  
**难度**: ⭐⭐ (简单)  
**要求**: 基本写作能力  

### 6. 创建故障排除指南
**优先级**: P2  
**模块**: docs  
**描述**: 收集常见问题和解决方案  
**难度**: ⭐⭐ (简单)  
**要求**: 了解 Git 常见问题  

---

## 🔧 编码任务 (Coding)

### 7. 实现 `git rev-parse` 命令
**优先级**: P1  
**模块**: core/refs  
**描述**: 实现 rev-parse 命令，用于解析引用  
**难度**: ⭐⭐⭐ (中等)  
**要求**: 
- Rust 基础
- 理解 Git 引用系统

**功能**:
```bash
./agit rev-parse HEAD
./agit rev-parse --symbolic-full-name HEAD
./agit rev-parse --sqrq HEAD
```

**学习资源**:
- [Git 内部原理 - 引用](https://git-scm.com/book/zh/v2/Git-内部原理-Git-引用)

### 8. 实现 `git rev-list` 命令
**优先级**: P1  
**模块**: core/objects  
**描述**: 实现 rev-list 命令，列出提交历史  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- 理解 Git 对象图

**功能**:
```bash
./agit rev-list HEAD
./agit rev-list --count HEAD
./agit rev-list --max-count=5 HEAD
```

### 9. 实现 `git branch -l` (列出分支)
**优先级**: P1  
**模块**: core/refs  
**描述**: 实现基本的分支列表功能  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- 理解 Git refs

**功能**:
```bash
./agit branch          # 列出本地分支
./agit branch -a      # 列出所有分支
./agit branch -v      # 显示详细信息
```

### 10. 添加配置文件解析
**优先级**: P1  
**模块**: config  
**描述**: 实现 TOML 配置文件解析  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- TOML 格式了解

**配置文件**:
```toml
# ~/.config/agit/config.toml
[ai]
auto_tag = true
tag_format = "suffix"

[output]
format = "json"
color = true
```

### 11. 实现 JSON 输出格式化
**优先级**: P1  
**模块**: output  
**描述**: 为 status、log 等命令添加 JSON 输出  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- serde 使用经验

**功能**:
```bash
./agit status --json
./agit log --json
```

### 12. 实现 `git diff --stat`
**优先级**: P2  
**模块**: core/diff  
**描述**: 实现 diff 统计信息显示  
**难度**: ⭐⭐⭐⭐ (较难)  
**要求**:
- Rust 基础
- 理解 diff 算法

**功能**:
```bash
./agit diff --stat
# example output:
#  file1.txt | 5 +++++
#  file2.txt | 2 --
#  2 files changed, 3 insertions(+), 2 deletions(-)
```

---

## 🧪 测试任务 (Testing)

### 13. 添加核心算法单元测试
**优先级**: P1  
**模块**: core  
**描述**: 为 SHA-1、zlib 等核心算法添加测试  
**难度**: ⭐⭐ (简单)  
**要求**:
- Rust 测试基础
- 单元测试经验

**测试内容**:
- SHA-1 已知值测试
- zlib 压缩/解压测试
- 对象序列化测试

### 14. 添加命令集成测试
**优先级**: P1  
**模块**: cli  
**描述**: 为每个命令添加集成测试  
**难度**: ⭐⭐ (简单)  
**要求**:
- Rust 测试基础
- 了解集成测试

**测试框架**: `assert_cmd`, `predicates`

### 15. 与原生 Git 输出对比测试
**优先级**: P2  
**模块**: tests  
**描述**: 创建测试自动对比 agit 和 git 输出  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- 了解 Git 命令行

---

## 🚀 优化任务 (Optimization)

### 16. 添加性能基准测试
**优先级**: P2  
**模块**: tests  
**描述**: 使用 criterion 创建性能基准测试  
**难度**: ⭐⭐ (简单)  
**要求**:
- Rust 基础
- criterion 使用经验

### 17. 实现 LRU 缓存
**优先级**: P2  
**模块**: core/storage  
**描述**: 为对象读取添加 LRU 缓存  
**难度**: ⭐⭐⭐ (中等)  
**要求**:
- Rust 基础
- LRU 算法了解

---

## 🎨 工具任务 (Tooling)

### 18. 配置 GitHub Actions CI
**优先级**: P1  
**模块**: .github/workflows  
**描述**: 配置自动化 CI/CD  
**难度**: ⭐⭐ (简单)  
**要求**:
- GitHub Actions 了解
- YAML 编写经验

**功能**:
- Rust 测试
- 代码格式化检查
- Clippy 检查
- 多平台构建

### 19. 创建 Cargo workspace 配置
**优先级**: P3  
**模块**: 根目录  
**描述**: 配置 Cargo workspace 支持多 crate  
**难度**: ⭐ (入门)  
**要求**: Cargo 使用经验

---

## 📋 领取任务

1. 在 Issue 中留言："I'd like to work on this"
2. Fork 项目
3. 创建新分支: `git checkout -b your-name/issue-name`
4. 开发并测试
5. 提交 PR

## 任务标签说明

| 标签 | 含义 |
|------|------|
| `good first issue` | 适合新手的入门任务 |
| `documentation` | 文档相关任务 |
| `enhancement` | 功能增强 |
| `bug` | Bug 修复 |
| `help wanted` | 需要帮助的任务 |

## 资源链接

- [Rust 教程](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Pro Git 书籍](https://git-scm.com/book/zh/v2)
- [项目架构](docs/ARCHITECTURE.md)
- [开发计划](docs/PLAN.md)

---

**有问题？** 欢迎在 [GitHub Discussions](https://github.com/bit-torch/AdapterGit/discussions) 提问！

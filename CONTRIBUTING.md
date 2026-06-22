# 贡献指南

感谢您对 AdapterGit 项目的兴趣！我们欢迎所有形式的贡献，无论是代码、文档还是问题反馈。

## 如何参与

### 报告问题

发现 bug 或有新想法？请：

1. 在 [GitHub Issues](https://github.com/bit-torch/AdapterGit/issues) 创建新 issue
2. 选择合适的标签 (bug, feature, documentation 等)
3. 提供详细的问题描述和复现步骤

### 贡献代码

#### 1. Fork 并克隆

```bash
git clone https://github.com/bit-torch/AdapterGit.git
cd AdapterGit
```

#### 2. 创建分支

使用清晰的分支命名：

```bash
# 功能分支
git checkout -b feat/add-clone-command

# Bug 修复
git checkout -b fix/status-command-error

# 文档更新
git checkout -b docs/update-readme
```

#### 3. 开发环境

**要求**:
- Rust 1.70+
- Cargo (随 Rust 安装)

**安装 Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**构建项目**:
```bash
cargo build                       # Debug 构建
cargo build --release             # Release 构建
cargo build --release --all-features  # Full 版本（含 AI）
cargo build --release --no-default-features -F tag  # Lite 版本
```

#### 4. 运行测试

```bash
cargo test            # 运行所有测试
cargo test --doc      # 文档测试
cargo clippy          # 代码检查
cargo fmt            # 代码格式化
```

#### 5. 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构（不是新功能或修复）
- `test`: 测试相关
- `chore`: 构建或辅助工具

**示例**:

```bash
git commit -m "feat(core): add SHA-1 hash implementation"
git commit -m "fix(cli): resolve init command panic on empty directory"
git commit -m "docs(readme): update installation instructions"
```

#### 6. Push 并创建 PR

```bash
git push origin feat/add-clone-command
```

在 GitHub 上创建 Pull Request，描述您的更改。

## 代码规范

### Rust 编码规范

- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 编写文档注释 (///)
- 添加单元测试

### 文档要求

- 所有公共 API 需要文档注释
- 更新相关文档
- 添加使用示例

### 测试要求

- 核心功能必须有测试
- 保持测试快速执行
- 测试边界情况

## 项目结构

```
agit/                         # Workspace 根目录
├── Cargo.toml                # Workspace 定义
├── agit-core/                # Rust 原生 Git 核心库
│   └── src/
│       ├── objects/          # Blob, Tree, Commit, Tag
│       ├── storage.rs        # Loose 对象读写
│       ├── refs.rs           # 引用管理（HEAD, 分支, 标签）
│       ├── index.rs          # DIRC v2 暂存区
│       ├── protocol.rs       # Git smart-HTTP 协议
│       ├── merge.rs          # 3 路合并
│       └── checkout.rs       # 分支切换 / 树恢复
├── agit-ai/                  # AI 模式（可选，feature 门控）
│   └── src/
│       └── lib.rs            # AI 自动标记、安全防护
├── agit-cli/                 # CLI 二进制入口
│   └── src/
│       ├── main.rs           # 入口点
│       ├── commands/         # 每个子命令一个文件
│       └── output/           # JSON / YAML / 无颜色输出
└── tests/                    # 集成测试
```

详见: [架构设计](ARCHITECTURE.md)

## 开发阶段

查看当前开发进度: [开发计划](docs/PLAN.md)

### 当前优先级

1. **P0**: 核心对象系统 (SHA-1, zlib, 对象模型)
2. **P0**: 基础命令 (init, add, commit)
3. **P1**: AI 模式
4. **P1**: 网络功能 (clone, push, pull)

## 新手任务

想要开始贡献？查看 [新手任务](../GOOD_FIRST_ISSUES.md)

## 获取帮助

- 📖 查看 [README](../README.md)
- 💬 加入讨论: [GitHub Discussions](https://github.com/bit-torch/AdapterGit/discussions)
- 🐛 报告问题: [GitHub Issues](https://github.com/bit-torch/AdapterGit/issues)

## 许可证

贡献即表示您同意您的代码遵循 Apache 2.0 许可证。

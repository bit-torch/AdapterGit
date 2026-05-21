# AdapterGit - Git for AI, not for editors

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/bit-torch/AdapterGit)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Static Binary](https://img.shields.io/badge/binary-static%20musl-green.svg)](https://github.com/bit-torch/AdapterGit/releases)

**AdapterGit (agit)** - 让 Git 在 AI 时代不再卡死。为自动化、脚本、CI/CD 和公共电脑环境设计的无 TUI Git 实现。

## 🎯 痛点解决

还在为这些事抓狂吗？
- 🤖 **AI Agent 调用 Git 时被 TUI 编辑器卡死**
- 🏫 **在学校机房每次都要重新安装 Git**
- 🔧 **脚本中 Git 命令意外触发交互界面**
- 🐌 **原生 Git 在非 TTY 环境表现诡异**

**agit 一劳永逸解决这些问题。**

## ✨ 核心特性

### 🤖 **AI 优先设计**
- 零 TUI 阻塞，AI Agent 的安全选择
- 结构化 JSON 输出，机器可读
- 自动添加 `[AI-committed]` 标记
- 危险操作防护，防止 AI 误操作

### 📦 **双版本策略**
agit 提供两个版本，均从底层原生实现 Git 核心逻辑，无任何外部 Git 依赖：

| | **Full 版本** | **Lite 版本** |
|---|---|---|
| 形态 | 已打包的可安装应用程序安装包 | 单文件便携二进制 |
| 安装方式 | 安装包一键安装（.msi / .deb / .dmg） | 下载即用，无需安装 |
| 体积 | ~20MB 安装包 | ~10MB 单文件 |
| 适用场景 | 个人开发机、企业批量部署 | AI Agent、CI/CD、临时环境、U 盘携带 |
| 系统集成 | 注册 PATH、右键菜单、文件关联 | 纯绿色，无系统痕迹 |
| Git 核心实现 | ✅ 原生 Rust 实现 | ✅ 原生 Rust 实现 |

**两个版本都完整实现了原生 Git 底层逻辑**（SHA-1、zlib、Blob/Tree/Commit 对象、引用系统、索引、网络协议），仅分发形态不同。

### ⚡ **永不卡死**
- 自动跳过所有编辑器
- 智能转换交互式命令
- 非 TTY 环境友好
- CI/CD 环境零配置

### 🔄 **Git 兼容**
- 兼容现有 Git 仓库和工作流
- 支持常用 Git 命令子集
- 可渐进式替换 git 命令
- 透明回退机制

## 🚀 快速开始

### 下载即用

#### 🪶 Lite 版本（单文件便携）
```bash
# Linux / macOS
curl -L https://github.com/bit-torch/AdapterGit/releases/latest/download/agit-lite-x86_64-unknown-linux-musl -o agit
chmod +x agit
./agit --help

# 直接运行，无需安装
./agit init
```

#### 📦 Full 版本（安装包）
```bash
# Linux (.deb)
curl -LO https://github.com/bit-torch/AdapterGit/releases/latest/download/agit_0.1.0_amd64.deb
sudo dpkg -i agit_0.1.0_amd64.deb

# macOS (.dmg)
# 下载 .dmg 文件，双击安装即可

# Windows (.msi)
# 下载 .msi 安装包，双击运行安装向导
```

### 从源码构建
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建 agit
git clone https://github.com/bit-torch/AdapterGit.git
cd agit
cargo build --release

# 静态编译（推荐）
cargo build --release --target x86_64-unknown-linux-musl
```

## 📖 使用示例

### 基础使用（和 Git 一样）
```bash
agit init
agit add .
agit commit -m "feat: add new feature"
agit push origin main
```

### 🤖 **AI 模式**
```bash
# AI 调用 - 永远不会卡在编辑器
agit commit --ai "fix: login bug"

# 输出结构化 JSON
{
  "status": "success",
  "command": "commit",
  "commit_hash": "abc123def456",
  "message": "fix: login bug\n\n[AI-committed]",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 🎯 **便携模式**
```bash
# 在任何地方，无需安装
cd /tmp/some-project
/path/to/agit add -A
/path/to/agit commit -m "work from public computer"
```

### 📁 **配置别名**
```bash
# 临时替换 git
alias git=agit

# 或只替换特定场景
alias gai='agit --ai'
```

## 🔧 安装指南

### 🪶 Lite 版本安装
```bash
# 使用 cargo
cargo install agit --features lite

# 或直接下载单文件
curl -L https://github.com/bit-torch/AdapterGit/releases/latest/download/agit-lite -o agit
chmod +x agit
sudo mv agit /usr/local/bin/  # 可选，移到 PATH
```

### 📦 Full 版本安装
```bash
# 使用 cargo
cargo install agit

# Linux (deb)
sudo dpkg -i agit_0.1.0_amd64.deb

# Linux (rpm)
sudo rpm -i agit-0.1.0-1.x86_64.rpm

# macOS
# 下载 .dmg 双击安装，或使用 Homebrew：
brew install bit-torch/tap/agit

# Windows
# 下载 .msi 安装包，双击运行安装向导
# 或使用 winget：
winget install bit-torch.agit
```

### 手动安装（两种版本通用）
1. 从 https://github.com/bit-torch/AdapterGit/releases 页面选择对应版本下载
2. Lite 版本：`chmod +x agit` 后直接运行
3. Full 版本：运行安装包或添加可执行权限后移到 PATH 目录

## 📊 对比表

### agit vs 原生 Git

| 特性 | agit Full | agit Lite | 原生 Git |
|------|-----------|-----------|----------|
| AI 调用安全 | ✅ 永不卡 TUI | ✅ 永不卡 TUI | ❌ 会卡编辑器 |
| 分发形态 | 📦 安装包（.msi/.deb/.dmg） | 🪶 单文件二进制 | ❌ 需要完整安装 |
| 单文件便携 | ❌ 需安装 | ✅ ~10MB 单文件 | ❌ 需安装 |
| 系统集成 | ✅ PATH/右键/文件关联 | ❌ 纯绿色无痕迹 | ✅ 完整集成 |
| 结构化输出 | ✅ JSON / YAML | ✅ JSON / YAML | ❌ 纯文本 |
| 零配置运行 | ✅ 开箱即用 | ✅ 开箱即用 | ❌ 需要 git config |
| 原生 Git 核心 | ✅ 纯 Rust 实现 | ✅ 纯 Rust 实现 | ✅ C 实现 |
| 交互式操作 | ❌ 不支持 | ❌ 不支持 | ✅ 完整支持 |

### Full vs Lite 版本选择指南

| 场景 | 推荐版本 |
|------|----------|
| 个人开发机日常使用 | 📦 Full |
| 企业批量部署 | 📦 Full |
| AI Agent / 自动化脚本 | 🪶 Lite |
| CI/CD 流水线 | 🪶 Lite |
| Docker 容器 | 🪶 Lite |
| U 盘携带 / 公共电脑 | 🪶 Lite |
| 需要右键菜单集成 | 📦 Full |
| 临时环境快速使用 | 🪶 Lite |

## 🎨 AI 模式详解

agit 专为 AI Agent 设计：

### 自动标记
```bash
agit commit --ai "修复登录问题"
# 提交信息自动添加：[AI-committed]
```

### 无交互转换
```bash
# agit 自动转换这些危险命令
git commit          → agit commit -m "[AI] auto-commit"
git rebase -i       → agit rebase --no-edit
git add -p          → agit add -A
git mergetool       → ❌ 拒绝执行
```

### 机器可读输出
```bash
agit log --json
agit status --json
agit diff --json
```

## 🏗️ 架构设计

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
│  ┌───────────────────────────────────┐  │
│  │  对象存储 │ 引用管理 │ Diff 算法   │  │
│  │  Pack 文件 │ 协议层 │ 索引系统    │  │
│  └───────────────────────────────────┘  │
└─────────────────┬───────────────────────┘
                  │ 统一核心 → 双版本分发
┌─────────────────▼───────────────────────┐
│              版本分发层                   │
│  ┌─────────────────┬─────────────────┐  │
│  │  📦 Full 版本    │  🪶 Lite 版本   │  │
│  │  .msi/.deb/.dmg │  单文件二进制     │  │
│  │  安装包分发      │  下载即用        │  │
│  └─────────────────┴─────────────────┘  │
└─────────────────────────────────────────┘
```

**Full 和 Lite 共享同一套原生 Rust 实现的 Git 核心，仅分发形态不同。**

## 📁 支持的命令

### ✅ 已实现
- `init` - 初始化仓库
- `add` - 添加文件
- `commit` - 提交更改
- `push` / `pull` - 远程操作
- `status` - 查看状态
- `log` - 查看日志
- `clone` - 克隆仓库

### 🚧 开发中
- `branch` - 分支管理
- `checkout` - 切换分支
- `merge` - 合并分支
- `stash` - 暂存更改

### ❌ 不计划支持
- `rebase -i` (交互式变基)
- `add -p` (交互式添加)
- `git mergetool` (合并工具)
- 其他所有 TUI 交互命令

## 🔧 配置

### 环境变量
```bash
# 强制 AI 模式
export AGIT_AI_MODE=1

# 设置输出格式
export AGIT_OUTPUT_FORMAT=json  # json, yaml, text

# 禁用颜色
export AGIT_NO_COLOR=1
```

### 配置文件
`~/.config/agit/config.toml`
```toml
[ai]
auto_tag = true
tag_format = "suffix"  # prefix, suffix, trailer

[output]
format = "json"
color = true

[safety]
prevent_force_push = true
max_commit_length = 100
```

## 🐳 集成示例

### GitHub Actions
```yaml
- name: Checkout with agit
  uses: bit-torch/agit-action@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

### Docker
```dockerfile
COPY --from=ghcr.io/bit-torch/AdapterGit:latest /agit /usr/local/bin/
RUN agit clone https://github.com/user/repo.git
```

### AI Agent (AutoGPT)
```python
# 用 agit 替代 git，避免卡死
import subprocess

result = subprocess.run(
    ["agit", "commit", "--ai", "Auto-commit by AI"],
    capture_output=True,
    text=True
)
print(result.stdout)  # JSON 输出
```

## 🤝 贡献指南

欢迎贡献！agit 是开源项目，我们欢迎所有形式的贡献。

### 开发环境设置
```bash
# 1. Fork 并克隆仓库
git clone https://github.com/bit-torch/AdapterGit.git
cd agit

# 2. 安装 Rust
rustup toolchain install stable

# 3. 构建
cargo build

# 4. 运行测试
cargo test
```

### 提交规范
agit 使用 Conventional Commits：
- `feat:` 新功能
- `fix:` bug 修复
- `docs:` 文档更新
- `test:` 测试相关
- `refactor:` 重构

### 项目结构
```
agit/
├── src/
│   ├── cli/      # 命令行解析
│   ├── git/      # Git 核心功能
│   ├── ai/       # AI 模式实现
│   ├── output/   # 输出格式化
│   └── utils/    # 工具函数
├── tests/        # 集成测试
└── examples/     # 使用示例
```

## 📄 许可证

本项目采用 **Apache-2.0** 许可证。

## 🙏 致谢

- **完全原生实现**：从底层用 Rust 实现 Git 核心协议和算法，无任何外部 Git 库依赖
- 受 [GitButler](https://gitbutler.com/) 和 [gitui](https://github.com/extrawurst/gitui) 启发
- 感谢所有在公共电脑上被 Git 折磨过的开发者

## 🐛 问题反馈

发现 bug 或有新想法？欢迎：
- [提交 Issue](https://github.com/bit-torch/AdapterGit/issues)
- [提交 PR](https://github.com/bit-torch/AdapterGit/pulls)
- [参与讨论](https://github.com/bit-torch/AdapterGit/discussions)

## 🌟 星星历史

[![Star History Chart](https://api.star-history.com/svg?repos=bit-torch/AdapterGit&type=Date)](https://star-history.com/#bit-torch/AdapterGit&Date)

---

## 📢 一句话介绍

**agit - 让你在 AI 时代还能愉快地用 Git,不再被TUI编辑器限制。**

无论你是：
- 🤖 在写 AI Agent
- 🎓 在学校机房 coding
- 🏢 在受限的企业环境
- 🐳 在 Docker 容器中
- 📱 在临时环境中

agit 都能让你的 Git 工作流**永不卡死，开箱即用**。

---

> ✨ 专为 AI 时代设计的 Git 工具 ✨

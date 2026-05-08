# AdapterGit - Git for AI, not for editors

https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg](https://github.com/bit-torch/AdapterGit)
https://img.shields.io/badge/rust-1.70%2B-orange.svg](https://www.rust-lang.org/)
https://img.shields.io/badge/binary-static%20musl-green.svg](https://github.com/bit-torch/AdapterGit/releases)

> **AdapterGit (agit)** - 让 Git 在 AI 时代不再卡死。为自动化、脚本、CI/CD 和公共电脑环境设计的无 TUI Git 实现。

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

### 📦 **单文件便携**
- 静态编译，无任何依赖
- 10MB 单文件，拷了就走
- 无需安装，无需 root 权限
- 跨平台支持（Linux/macOS/Windows）

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
```bash
# Linux
curl -L https://github.com/bit-torch/AdapterGit/releases/latest/download/agit-x86_64-unknown-linux-musl -o agit
chmod +x agit
./agit --help

# 或直接运行（无需安装）
./agit init
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

### 一键安装
```bash

# 使用 cargo
cargo install agit
```

### 手动安装
1. 从 https://github.com/bit-torch/AdapterGit/releases 页面下载对应平台的二进制文件
2. 添加可执行权限：`chmod +x agit`
3. 移动到 PATH 目录：`sudo mv agit /usr/local/bin/`（可选）

## 📊 对比表

| 特性 | agit | 原生 Git |
|------|------|----------|
| AI 调用安全 | ✅ 永不卡 TUI | ❌ 会卡编辑器 |
| 单文件便携 | ✅ 10MB 静态二进制 | ❌ 需要完整安装 |
| 结构化输出 | ✅ JSON / YAML | ❌ 纯文本 |
| 公共电脑友好 | ✅ 无需安装 | ❌ 需要 sudo |
| 零配置运行 | ✅ 开箱即用 | ❌ 需要 git config |
| 完整 Git 功能 | ⚠️ 常用子集 | ✅ 全部功能 |
| 交互式操作 | ❌ 不支持 | ✅ 完整支持 |

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
                  │ 纯 Rust 调用
┌─────────────────▼───────────────────────┐
│            gitoxide (gix)               │
│          纯 Rust Git 实现                │
└─────────────────────────────────────────┘
```

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

- 基于 https://github.com/Byron/gitoxide 构建，一个优秀的纯 Rust Git 实现
- 受 https://gitbutler.com/ 和 https://github.com/extrawurst/gitui 启发
- 感谢所有在公共电脑上被 Git 折磨过的开发者

## 🐛 问题反馈

发现 bug 或有新想法？欢迎：
- https://github.com/bit-torch/AdapterGit/issues
- https://github.com/bit-torch/AdapterGit/pulls
- 在 https://github.com/bit-torch/AdapterGit/discussions 中讨论

## 🌟 星星历史

https://api.star-history.com/svg?repos=bit-torch/AdapterGit&type=Date](https://star-history.com/#bit-torch/AdapterGit&Date)

---

## 📢 一句话介绍

**agit - 让你在 2026 年还能愉快地用 Git。**

无论你是：
- 🤖 在写 AI Agent
- 🎓 在学校机房 coding
- 🏢 在受限的企业环境
- 🐳 在 Docker 容器中
- 📱 在临时环境中

agit 都能让你的 Git 工作流**永不卡死，开箱即用**。

---

> ✨ 专为 AI 时代设计的 Git 工具 ✨

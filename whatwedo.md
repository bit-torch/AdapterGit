# agit 做了什么 — 用户视角全流程

> 白话梳理：打开终端，用 agit 从零到一把 Git 事儿全办了。

## 1. 拿到手

- **Lite 版**：单文件，下载就能跑，不用安装。丢 U 盘、CI 容器、公用电脑都行。
- **Full 版**：装完自动配 PATH，有 AI 写提交信息。
- 也支持 `cargo install`，或者自己 `cargo build`（180 秒左右出二进制）。

## 2. 开仓库

```
agit init                      # 当前目录变仓库
agit init --path /tmp/project  # 指定目录
agit init --pattern rust       # 顺手生成 .gitignore (rust/python/node/go/java)
agit init --licence mit        # 顺手生成 LICENCE 文件
```

## 3. 日常提交

```
agit add .                     # 全加
agit add src/main.rs           # 加单个
      ↓ 自动尊重 .gitignore
agit commit -m "fix: typo"     # 正常提交
agit commit --ai "修登录"       # AI 帮你写 commit message
      ↓ 自动打 [AI-committed] 标记，方便追溯
```

**AI 写提交信息怎么玩？** 配置好 `AGIT_LLM_API_KEY`（OpenAI / DeepSeek / 月之暗面 / 智谱 / Ollama 都行），`agit commit --ai` 会自动把暂存区 diff 喂给 LLM，吐一条 Conventional Commits 格式的提交信息。

## 4. 看状态

```
agit status                    # 哪些改了、哪些暂存了
agit log                       # 提交历史
agit log --oneline -n 10       # 精简模式
agit log --all                 # 所有分支
agit diff                      # 改了什么
agit diff --cached             # 暂存区改了什么
agit diff --name-only          # 只看文件名
agit diff <commit1> <commit2>  # 两次提交之间
agit show HEAD                 # 最近一次提交的详情
```

## 5. 分支和跳转

```
agit branch                    # 列出本地分支
agit branch -c feat/xxx        # 新建分支
agit branch -d feat/xxx        # 删分支
agit checkout feat/xxx          # 切换过去
agit checkout --force feat/xxx  # 有未提交改动也切（自动 stash 再 pop）
```

## 6. 和别人协作（远程）

```
agit clone https://...         # 克隆（HTTP/HTTPS）
agit clone git@github.com:...  # 克隆（SSH，走系统 ssh）
agit fetch                     # 看看远程有什么更新
agit pull                      # 拉下来并自动合并
agit push origin main          # 推上去
agit remote add upstream ...   # 加个远程
agit remote list               # 看看有哪些远程
```

## 7. 合并代码

```
agit merge feat/xxx            # 把分支合进来
      ↓ 能 fast-forward 就 fast-forward
      ↓ 有冲突就写冲突标记，让你手动解决
```

## 8. 高级操作

### 变基（rebase）
```
agit rebase main               # 把当前分支"搬"到 main 上面
agit rebase --onto main HEAD~3 # 选三笔搬过去
agit rebase --continue         # 解决冲突后继续
agit rebase --skip             # 跳过当前这笔
agit rebase --abort            # 不玩了，回退
```

### 遴选（cherry-pick）
```
agit cherry-pick abc123        # 挑一笔提交过来
agit cherry-pick abc123 def456 # 多笔一起
agit cherry-pick --continue    # 解决冲突后继续
agit cherry-pick --abort       # 不玩了
```

### 储藏（stash）
```
agit stash                     # 把改动暂存起来
agit stash pop                 # 拿出来
agit stash list                # 看看有哪些
agit stash drop                # 丢掉
```

### 重置（reset）
```
agit reset --soft HEAD~1       # 撤销提交，改动回暂存区
agit reset --mixed             # 默认：回暂存区
agit reset --hard HEAD~1       # 彻底丢掉（慎用）
```

### 二分查找（bisect）
```
agit bisect start --bad HEAD --good v0.1.0
agit bisect good               # 这个版本没问题
agit bisect bad                # 这个版本有 bug
agit bisect run "cargo test"   # 自动跑脚本定位
```

### 追溯（blame）
```
agit blame src/main.rs         # 每行代码谁写的
agit blame --revision HEAD~5 src/main.rs  # 看历史版本
```

### 引用日志（reflog）
```
agit reflog                    # HEAD 的所有变动记录
agit reflog main               # main 分支的变动记录
```

## 9. 标签

```
agit tag                       # 列标签
agit tag -c v1.0.0             # 打轻量标签
agit tag -c v1.0.0 -m "发版"   # 打注释标签
agit tag -d v1.0.0             # 删标签
```

## 10. 文件操作

```
agit rm file.txt               # 删文件（同时从暂存区移除）
agit rm --cached file.txt      # 只从暂存区移除，保留文件
agit mv old.txt new.txt        # 改名/移动
```

## 11. 查看内部对象

```
agit cat-file -p abc123        # 看对象内容
agit cat-file -t abc123        # 看对象类型
agit ls-tree abc123            # 看一棵树里有什么
```

## 12. 配置

```
agit config user.name "Me"     # 设用户名
agit config --global user.email "me@x.com"  # 全局配置
agit config --list             # 看所有配置
agit config --get user.name    # 查一项
agit config --unset user.name  # 删一项
```

**配置优先级**：环境变量 > 仓库 `.agit/config.toml` > 全局 `~/.agitconfig.toml` > 默认值

还支持命令别名：
```toml
# ~/.agitconfig.toml
[alias]
co = "commit"
st = "status"
```

## 13. 输出格式

```
agit log --json                # JSON 输出（给脚本/AI 用）
agit status --yaml             # YAML 输出
agit --no-color log            # 不要颜色（管道用）
```

## 14. AI 模式

```
agit --ai commit -m "fix"      # 开启 AI 模式
```

AI 模式下会自动：
- 提交信息加 `[AI-committed]` 前缀
- 阻止危险命令：`push`、`stash drop`、`branch -D`、`rebase`、`cherry-pick`、`bisect`

---

## 一张图：从打开终端到推上 GitHub

```
agit init
  → agit add .
    → agit commit -m "init"
      → (改代码)
        → agit add .
          → agit commit --ai "新功能"
            → agit push origin main
```

## 还没做的（用户一眼就能看出缺什么）

| 功能 | 状态 | 说明 |
|------|------|------|
| 交互式 rebase (`-i`) | ❌ 不支持 | 需要 TUI，和"永不挂起"的设计理念冲突 |
| 交互式 add (`-p`) | ❌ 不支持 | 同上 |
| submodule | ❌ 不支持 | |
| hooks | ❌ 不支持 | pre-commit / post-commit 等 |
| grep | ❌ 不支持 | |
| Git 协议 v2 | ❌ 不支持 | 目前走 v1 协议 |
| Windows 安装包 (.msi) | ❌ 待做 | |
| musl 静态编译 | ⏳ 待修复 | CI 缺 musl-gcc |

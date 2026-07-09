# agit 功能缺口分析

[English](MISSING_FEATURES.md)

> 版本：0.5.4 | 日期：2026-06-13 | 分支：feat/missing-features-doc

本文档分析 agit 相对于标准 Git 的功能缺失，按优先级分为 P0（阻塞基本工作流）→ P3（锦上添花）。每个条目包含问题描述和实现建议，不包含具体代码实现。

---

## 🔴 P0 — 阻塞基本工作流（无这些功能无法正常使用）✅ 已完成 (v0.5.4)

### 1. `reset` / 取消暂存 ✅

**问题：** `status` 输出中提示 `"use git restore --staged <file>..."`，但项目根本没有 `reset` 或 `restore` 命令。用户执行 `add` 后无法取消暂存文件。`core::index::Index::remove_entries()` API 已实现但无任何命令调用它。

**实现：** `commands/reset.rs` — 支持 `reset HEAD <file>`（取消暂存）、`reset --soft/--mixed/--hard [<commit>]`（移动 HEAD）、`HEAD~N` 父提交遍历。`core::checkout` 暴露 `restore_from_commit()` 和 `rebuild_index_from_commit()` 公共 API。
> 提交：`7233db6`

### 2. checkout 缺少工作区安全检查 ✅

**问题：** `commands/checkout.rs:23` 直接调用 `checkout::switch_branch()`，**不检查工作区是否有未提交的变更**。Git checkout 默认会检查并拒绝（除非 `-f/--force`），以防止用户丢失未保存修改。

**实现：** `commands/checkout.rs` — 切换分支前检查 tracked 文件是否被修改/删除，不干净时拒绝并提示。添加 `-f`/`--force` 强制切换。
> 提交：`0094649`

### 3. `config` 命令 ✅

**问题：** 用户无法通过命令行设置 `user.name` / `user.email`。只能手动编辑 `.agit/config.toml` 或 `~/.agitconfig.toml`。这是新用户 onboarding 的第一步，也是 Git 使用频率最高的命令之一。

**实现：** `commands/config_cmd.rs` — 支持 `config <key>`（get）、`config <key> <val>`（set）、`--list`、`--unset`、`--global`。直接读写 TOML 文件，支持 `section.key` 嵌套键。
> 提交：`80d9b5e`

### 4. 合并冲突解决流程残缺 ✅

**问题：** merge 产生冲突后写入 `<<<<<<<` 标记文件和 `MERGE_HEAD` / `MERGE_MSG`，但：
- `commit` 不检测 `MERGE_HEAD` 存在 → 无法完成合并提交。
- 没有 `merge --abort` → 无法回退合并状态。
- 没有 `merge --continue` → 无法继续中止的合并。

**实现：** `commands/commit.rs` — 自动检测 MERGE_HEAD 并创建双 parent 合并提交、使用 MERGE_MSG 作为默认消息、提交后清理合并状态文件。`commands/merge.rs` — `--abort` 恢复 ORIG_HEAD + 清理状态文件，`--continue` 委托给 commit，merge 开始时自动保存 ORIG_HEAD。
> 提交：`be91be1`

---

## 🟠 P1 — 严重阻碍日常使用

### 5. `.gitignore` 支持

**问题：** `status`、`add`、`diff` 的未跟踪文件列表会包含 `target/`、`node_modules/`、`.DS_Store` 等。对任何有构建产物的项目，输出极嘈杂，`add .` 会误添加大量垃圾文件。

**影响：** `status` 和 `add .` 在真实项目中几乎不可用。

**实现建议：**
- 新增 `core::ignore` 模块，实现 `IgnoreMatcher` 结构体。
- 解析规则：读取 `.gitignore`（支持 `*`、`**`、`?`、`[abc]`、`!` 负向、`#`注释、目录标记 `/`）。
- 继承 `.gitignore` 的级联搜索（从当前目录逐级向上到 repo root）。
- 在 `status::collect_untracked()`、`add` 路径展开、`diff` 未跟踪中过滤 ignored 文件。
- `.git/info/exclude` 文件支持可后做。

### 6. `stash` — 临时保存工作区

**问题：** 无法临时保存工作区变更。`stash` / `stash pop` 是 Git 用户的高频操作，用于在未完成工作时切换分支或拉取更新。

**影响：** 工作区有未提交变更时完全无法进行任何需要干净工作区的操作（checkout、pull、merge 等）。

**实现建议：**
- `stash push`：① 从 index 和工作区差异生成 tree objects；② 创建 stash commit（结构：merge commit 有 2-3 个 parent——HEAD commit、index 状态、未跟踪文件）；③ 更新 `refs/stash`；④ 重置工作区到 HEAD。
- `stash pop`：将 `refs/stash` 的变更 apply 回工作区，成功则删除该 stash。
- `stash list`：遍历 `refs/stash` 的 reflog 或线性 parent 链列出所有 stash。
- `stash drop`：删除指定 stash。

### 7. `tag` CLI 命令

**问题：** `core::objects::tag.rs`（annotated tag 模型）、`core::refs` 中的 `create_tag`/`list_tags`/`delete_tag` 均已 feature-gated 实现。但 CLI 层完全缺少 `tag` 子命令，用户无法创建或查看标签。

**影响：** 无法标记发布版本，`git describe` 等依赖 tag 的功能也无法实现。

**实现建议：**
- 在 `cli/mod.rs` 的 `Commands` enum 添加 `Tag { action: TagAction }`，子动作 `list` / `create {name, message}` / `delete {name}`。
- 新增 `commands/tag.rs`，调用已有的 core API。
- `tag create` 可加 `-a`（annotated）、`-m`（message）、`-s`（签名的 stub）。

### 8. `diff` 功能局限

**问题：** 当前 `diff` 只能做 HEAD vs index + 未跟踪文件。不支持：
- `diff <commit1> <commit2>` — 比较任意两个提交。
- `diff <branch1>..<branch2>` — 分支间比较。
- `diff --cached` — 比较 HEAD 和索引（查看即将提交的内容）。
- `diff --name-only` — 仅列出文件名。

**影响：** `diff` 在代码审查、分支比较场景几乎无用。

**实现建议：**
- 修改 `diff run()` 接受可选的两个对象参数（SHA/分支名/tag 名），解析为 tree SHA 后比较两个 tree。
- `--cached`：比较 HEAD tree 和 index 中的 blob。
- `--name-only`：仅输出文件名不输出差异内容。
- 参数默认值：无参 = 比较 index 和工作区（当前行为）；一参 = 比较给定 commit 和工作区。

### 9. `log` 功能简陋

**问题：** 当前 `log` 只走 first-parent 链，无任何过滤或格式化选项：
- 无 `--oneline` — 简洁单行格式。
- 无 `--graph` — ASCII 分支图。
- 无 `--all` — 显示所有分支的历史。
- 无 `-n <N>` — 限制输出条数。
- 无 `--since` / `--until` — 时间范围。
- 无 `--author` — 按作者筛选。
- 只追 first-parent，看不到 merge 的另一条线。

**影响：** 查看历史非常不便，稍微复杂的分支历史完全不可见。

**实现建议：**
- 优先实现 `-n N`（限制遍历步数）和 `--oneline`（`<short_hash> <first_line_of_message>`）。
- `--all`：读取 `refs/heads/*` 和 `refs/tags/*`，各起点同时 BFS/DFS 遍历。
- `--graph`：用简单列偏移绘制 ASCII 竖线和分支。
- `--author`：解析 commit author 字段做子串匹配。
- `--since`/`--until`：解析 timestamp 字段过滤。

### 10. `rm` / `mv` — 删除和移动跟踪文件

**问题：** 无法从版本控制中删除或重命名文件。用户只能手动操作文件系统但索引同步需要直接编辑二进制 index。

**影响：** 重构代码（移动文件、删除废弃文件）无法用版本控制跟踪。

**实现建议：**
- `rm <file>`：① 从索引移除条目（`Index::remove_entries()`），② 删除工作区文件（默认），③ `--cached` 仅删除索引条目。
- `mv <old> <new>`：① 索引中查找旧路径，修改为新路径，② 移动/重命名工作区文件。

---

## 🟡 P2 — 明显缺失但不阻塞基本操作

### 11. `rebase`

**问题：** 无法将当前分支的 commits 变基到另一分支之上。这是保持线性历史的常用操作。

**实现建议：**
- 简单版本 `rebase <target_branch>`：① 计算当前分支独有 commits（merge-base → HEAD）；② 依次 cherry-pick 到 target_branch；③ 移动 HEAD 到 target_branch 的 HEAD。
- 交互式 rebase（`-i`）需要编辑器交互，违反 "不阻塞" 设计原则，可后做或用 `GIT_SEQUENCE_EDITOR` 模式。
- 冲突时写入 `REBASE_HEAD` 状态并支持 `rebase --abort` / `rebase --continue`。

### 12. `cherry-pick`

**问题：** 无法将单个 commit 应用到当前分支。

**实现建议：**
- 读取目标 commit 与其 parent 的 tree diff → 将 diff apply 到当前工作区 → 创建新 commit（message 复用，author 保留）。
- 支持 `-n`（no-commit，仅 apply 到工作区+索引）。

### 13. `revert`

**问题：** 无法撤销一个已有 commit。

**实现建议：**
- 类似 cherry-pick 但逆向 apply diff。创建新 commit，message 为 `Revert "<original subject>"`。
- 与 cherry-pick 共享 diff 引擎。

### 14. `blame` / `annotate`

**问题：** 无法逐行追溯代码的最后修改者。

**实现建议：**
- 对指定文件的每个 commit 做 diff（相对于 parent），逐行追踪来源。
- 纯文本输出格式：`<short_hash> (<author> <date> <line_no>) <content>`。

### 15. `clean`

**问题：** 无法一键清理未跟踪文件。

**实现建议：**
- `-n` dry-run（默认）、`-f` 强制删除、`-d` 包含目录。
- 与 `.gitignore` 模块配合，`-x` 也删除 ignored 文件。

### 16. HTTP 认证 / 凭据

**问题：** `HttpTransport` 无任何认证机制。所有私有仓库完全不可访问。

**实现建议：**
- 最低实现：`AGIT_TOKEN` / `GIT_TOKEN` 环境变量 → HTTP `Authorization: Bearer <token>` 头。
- URL 中提取 `user:token@host` → HTTP Basic Auth。
- 更完善：`~/.agitcredentials` 文件、`credential.helper` 配置。

### 17. SSH 协议支持

**问题：** 当前仅实现 HTTP(S) 传输。SSH 是私有仓库和自托管 Git 服务器的主流协议。

**实现建议：**
- 解析 `git@host:path` 格式 URL。
- 方法一：通过 `ssh2` crate 直接实现 SSH 连接和 `git-upload-pack`/`git-receive-pack` 子进程。
- 方法二：调用系统 `ssh` 命令建立管道（更简单，兼容性更好）。

### 18. `--version` 标志

**问题：** `Cli` struct 未设置 clap version，运行 `agit --version` 无输出版本号。

**实现建议：**
- 在 `#[derive(Parser)]` 上添加 `#[command(version = env!("CARGO_PKG_VERSION"))]`。

---

## 🟢 P3 — 锦上添花

### 19. 浅克隆 `--depth`

**问题：** clone/fetch 总是获取完整历史，大型仓库极慢。

**实现建议：** protocol 层在 `want` 行添加 `--depth` 参数（`want <sha> depth=<n>`）。服务器返回截断历史，本地创建 "shallow" 标记文件（`.git/shallow`）。

### 20. `bisect` — 二分查找 bug 引入点

需要维护 `BISECT_LOG`、`BISECT_GOOD`/`BISECT_BAD` 状态文件。`bisect start` → `bisect good <commit>` / `bisect bad <commit>` → 自动 checkout 中间 commit → 用户测试后 `bisect good/bad` → 循环至找到首个 bad commit → `bisect reset` 恢复。

### 21. `grep` — 在工作区/tree 中搜索

搜索指定 pattern（支持 `-i` 忽略大小写、`-n` 行号、`-r` 递归、`--name-only`），可在工作区、index 或指定 tree 上运行。

### 22. Hooks — 生命周期钩子

执行 `pre-commit`、`post-commit`、`pre-push`、`post-checkout` 等脚本。在 `commit`、`push`、`checkout` 等命令的适当时机检测并运行 `.git/hooks/<name>` 文件。

### 23. `submodule` — 子模块管理

解析 `.gitmodules` 配置，递归 clone/update 子模块仓库。`submodule add`、`submodule update --init --recursive`、`submodule status`。

### 24. `worktree` — 多工作目录

并行 checkout 多个分支到不同目录。管理 `.git/worktrees/` 下的 worktree 链接。

### 25. `reflog` — 引用变更日志

每次分支更新时在 `.git/logs/refs/heads/<name>` 追加一行记录。`reflog show` 查看历史引用值。用于恢复误 `reset --hard`、误 `commit --amend` 等。

### 26. `archive` — 打包导出

不包含 `.git` 目录的文件快照导出（tar/zip）。`archive -o <file> <tree-ish>`。

### 27. `describe` — 基于 tag 的版本描述

`git describe --tags`：找到最近的 annotated tag，输出 `<tag>-<N>-g<short_hash>` 格式的可读版本号。

### 28. Packfile 生成优化（Delta 编码）

**问题：** `push` 的 `generate_pack()` 将每个 object 独立 zlib 压缩，不使用 delta 编码。大文件或大量相似文件的网络传输效率极低。

**实现建议：** 实现 `git diff-delta` 算法：在两个 blob 之间生成 binary delta patch，选择相似度高的 pairs 做 ofs_delta/ref_delta 编码。

### 29. Index 多 stage 支持（合并用）

**问题：** 当前 Index 用 `BTreeMap<String, IndexEntry>`，同一路径只有一条。Git index 的 DIRC v2 格式允许同一路径最多 4 个 stage（0=normal, 1=base, 2=ours, 3=theirs），用于三路合并时标记冲突。

**实现建议：** 扩展 `IndexEntry` 添加 `stage` 字段（0-3），`Index::entries` 改为 `BTreeMap<(String, u8), IndexEntry>`。

### 30. Diff 算法增强

当前基于 LCS 的行级比较对大文件和二进制文件效率低下。Git 的 patience/histogram 算法通常在代码文件上产生更可读的 diff。可引入 `similar` crate 或手动实现 patience diff。

### 31. 文件模式检测

**问题：** `add` 中硬编码 `100644`（普通文件），不检测符号链接或可执行位。

**实现建议：** `add` 中通过 `fs::symlink_metadata` 和权限位判断：普通文件 `100644`，可执行 `100755`，符号链接 `120000`（存储链接目标路径为 blob 内容）。

### 32. 配置项扩展

**问题：** `Config` 结构体只有 `user_name`、`user_email`、`aliases` 三个字段。

**实现建议：** 添加常用配置项：`core.editor`、`remote.origin.url`、`remote.origin.fetch`、`credential.helper`、`init.defaultBranch`、`merge.tool`。


---

## 🔧 架构改进建议（非功能缺失）

| # | 问题 | 建议 |
|---|------|------|
| A | `with_header()`/`with_object_header()` 在 `checkout.rs`、`merge.rs`、`diff.rs` 等多个文件重复定义 | 提取到 `core::objects` 作为公共函数 `pub fn format_object_data(type, content) -> Vec<u8>`（已存在但未在所有地方使用） |
| B | `collect_tree_paths()` / `collect_untracked()` 在 `checkout.rs`、`merge.rs`、`status.rs`、`diff.rs`、`pull.rs` 重复 | 提取到 `core::tree_utils` 或 `core::repo` |
| C | `pull.rs` 和 `merge.rs` 各自实现了共同祖先查找算法，逻辑略有不同 | 统一使用 `core::merge::find_merge_base()` |
| D | Index `remove_entries()` API 已实现但无命令调用 | 由 `reset` 和 `rm` 命令使用 |
| E | `status` 提示 "use git restore --staged" 但项目名为 agit | 提示文本应改为 "agit reset HEAD <file>" |
| F | Windows 路径使用 `replace('\\', '/')` 做临时规范化 | 应建立统一的路径处理层（`utils::normalize_path` 或 `core::repo::normalize_path`）|

---

## 优先级汇总

```
第一轮 — 最小可用产品（P0）:
  ┌─ 1. reset / 取消暂存
  ├─ 2. checkout 安全检查
  ├─ 3. config 命令
  └─ 4. merge 冲突解决流程完整化

第二轮 — 日常无摩擦（P1）:
  ├─ 5. .gitignore
  ├─ 6. stash
  ├─ 7. tag CLI
  ├─ 8. diff 比较任意提交
  ├─ 9. log --oneline / -n
  └─ 10. rm / mv

第三轮 — 团队协作（P2）:
  ├─ 11. rebase（简单版）
  ├─ 12. cherry-pick
  ├─ 13. revert
  ├─ 14. blame
  ├─ 15. clean
  ├─ 16. HTTP 认证
  ├─ 17. SSH 协议
  └─ 18. --version

第四轮 — 完整度提升（P3）:
  └─ 19-32. 按需实现
```

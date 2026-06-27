# CLAUDE.md

This file provides guidance to Claude Code when working in this repo.
> 默认使用简体中文回复。代码注释使用简体中文。

## Project

**agit (AdapterGit)** — a pure-Rust, zero-external-Git-dependency Git implementation. Objects, refs, index, smart-HTTP protocol all from scratch. Ships as a single static binary. Designed for AI agents, CI/CD, and portable use. Never blocks on interactive prompts.

Version: 0.14.0 | Edition: 2021 | License: Apache-2.0

## Workspace Structure

```
D:\AdapterGit\                   ← Cargo workspace root
├── agit-core/   (lib)           ← Git 核心库
├── agit-ai/     (lib)           ← AI 功能（LLM 客户端）
└── agit-cli/    (bin → agit)    ← CLI 二进制
```

## Build / Test / Lint

```bash
cargo build                           # workspace 全量构建
cargo build --all-features            # Full 版 (tag + tls + ai)
cargo build --no-default-features -F tag  # Lite 版 (纯本地, 无 TLS)

cargo build -p agit-core              # 单独构建核心库
cargo build -p agit-ai                # 单独构建 AI 库
cargo build -p agit-cli               # 单独构建 CLI

cargo test                            # 全部 178 测试 (109 单元 + 69 集成)
cargo test -p agit-core               # 核心库单元测试
cargo test -p agit-cli                # CLI 集成测试

cargo clippy                          # Must pass w/ 0 warnings
cargo fmt                             # Must pass w/ no diff
```

## Architecture

### agit-core (lib) — Git 核心库
```
agit-core/src/
├── lib.rs              ← 模块声明（扁平化，无嵌套 core::）
├── hash.rs             ← SHA-1 (sha1 crate)
├── compression.rs      ← zlib via flate2
├── storage.rs          ← Loose object R/W
├── refs.rs             ← HEAD, refs/heads/*, refs/tags/*, refs/remotes/*, CRUD
├── reflog.rs           ← Reflog 管理
├── index.rs            ← DIRC v2 staging area
├── ignore.rs           ← .gitignore parser
├── repo.rs             ← find_repo_root(), timestamp helpers
├── checkout.rs         ← Branch switch, tree restore, index rebuild
├── merge.rs            ← 3-way merge + fast-forward + conflict markers
├── rebase.rs           ← Rebase/cherry-pick 核心
├── bisect.rs           ← Bisect 状态管理与算法
├── protocol.rs         ← Git smart-HTTP: pkt-line, packfile, ref discovery
├── ssh_transport.rs    ← SSH 传输（子进程 ssh）
├── ssh_url.rs          ← SSH URL 解析 + ~/.ssh/config
├── remote_utils.rs     ← 网络命令共享工具
├── objects/
│   ├── blob.rs, commit.rs, tree.rs, tag.rs (feature-gated)
│   └── mod.rs
├── config/
│   └── mod.rs          ← Config: user.name, user.email, aliases, LLM
└── utils/
    ├── mod.rs           ← atomic_write()
    └── error.rs         ← AgitError enum
```

### agit-ai (lib) — AI 功能
```
agit-ai/src/
└── lib.rs              ← LlmConfig, chat_completion(), generate_commit_message()
                          depends on agit-core::config, reqwest
```

### agit-cli (bin → agit) — CLI 二进制
```
agit-cli/src/
├── main.rs             ← Config → alias resolve → CLI parse → dispatch
├── cli/mod.rs          ← clap derive, 29 subcommands, global flags
├── commands/           ← One file per subcommand, each run(…) → Result
├── ai/mod.rs           ← AI-mode flag, DANGEROUS_COMMANDS, re-exports agit-ai
├── output/mod.rs       ← JSON/YAML/no-color output
└── tests/              ← Integration tests (69 tests)
    └── common/mod.rs   ← Test helpers (agit_binary, setup_repo, run_agit)
```

## Key Patterns

- **Workspace**: 3 crates — `agit-core` (lib), `agit-ai` (lib), `agit-cli` (bin). Root `Cargo.toml` is workspace-only.
- **Dual edition**: Lite = `--no-default-features -F tag` (no TLS, no AI). Full = `--all-features`.
- **Feature propagation**: `agit-cli/tag → agit-core/tag`, `agit-cli/tls → agit-core/tls`, `agit-cli/ai → agit-ai`.
- **Global state**: `AI_MODE`, `JSON_MODE`, `YAML_MODE`, `NO_COLOR` are `AtomicBool` statics in `agit-cli`.
- **Error handling**: Commands return `Box<dyn Error>`. Core uses `AgitError` enum.
- **Config cascade**: env vars > repo `.agit/config.toml` > global `~/.agitconfig.toml` > defaults.
- **Commit flow**: `Index::load()` → build `Tree` → write tree → build `Commit` (+parent) → write commit → update ref.
- **Network flow**: Clone = discover_refs → fetch_objects → parse_packfile → write_objects → checkout.

## Rules

### Branch & PR

1. **main is protected** — never push directly to main. Create `feat/`, `fix/`, `docs/`, `chore/` branches and use PRs.
2. **Branch naming**: `feat/<name>`, `fix/<name>`, `docs/<name>`, `chore/<name>`.

### Commit

3. **Single Logical Change** — every commit must be one atomic, self-contained change.
4. **Multi-commit PR is OK** — splitting across commits is encouraged (e.g. `refactor:` → `feat:` → `test:`).
4a. **Commit per logical unit** — 每个 commit 只包含一个逻辑单元。
4b. **Format per commit** — 每个 commit 前单独运行 `cargo fmt`。
5. **Conventional Commits**: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `chore:`.
5a. **No `@` in commit messages**.

### Quality Gate

6. **Tests required** — `cargo test` must pass with 0 failures before push.
7. **Clippy clean** — `cargo clippy` must produce 0 warnings before push.
8. **Formatted** — `cargo fmt` must produce no diff before push.

### Code

9. **No silent error swallowing** — use `?` or explicit `map_err` for IO.
10. **Feature gate tag** — `#[cfg(feature = "tag")]` code must compile with both `--features tag` and `--no-default-features`.
11. **Windows compatibility** — use `std::fs` APIs, normalize paths with `replace('\\', '/')`.

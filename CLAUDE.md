# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
> 默认使用简体中文进行所有回复和代码注释。
## Project

**agit (AdapterGit)** — an AI-native Git tool in pure Rust. Zero external Git dependencies; implements the Git object model, refs, index, and network protocol from scratch. Designed for AI agents, CI/CD, and portable use. No TUI editors — it never blocks on interactive prompts.

Version: 0.4.1 | Edition: 2021 | License: Apache-2.0

## Build / Test / Lint

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # All tests (unit + integration)
cargo test -p agit       # Run only the agit crate tests
cargo clippy             # Lint
cargo fmt                # Format code
```

Static compilation (musl):
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

Integration tests (`tests/integration_test.rs`) shell out to the agit binary. Cargo sets `CARGO_BIN_EXE_agit` automatically; run with `cargo test`.

## Feature flags

- `tag` (default, enabled in `default` features) — gates `core::objects::tag` and tag-related functions in `core::refs` (`create_tag`, `list_tags`, `delete_tag`). Tag module code uses `#[cfg(feature = "tag")]`.

## Architecture

```
main.rs          → Entry point. Loads config → resolves aliases → parses CLI → dispatches to commands.
                   Sets global AI/JSON/YAML/no-color modes before dispatch.
cli/mod.rs       → clap derive (Parser + Subcommand). 15 subcommands + 4 global flags (--ai --json --yaml --no-color).
commands/        → One file per subcommand. Each exposes a `run(...)` function returning `Result<(), Box<dyn Error>>`.
core/            → Pure Git implementation (no I/O to stdout, no CLI coupling).
  objects/       → blob.rs, tree.rs, commit.rs, tag.rs (feature-gated). Each has new/serialize/hash.
  storage.rs     → Loose object read/write to .git/objects/{prefix}/{rest}, zlib compressed.
  refs.rs        → HEAD (symbolic/detached), refs/heads/*, refs/tags/* (CRUD).
  index.rs       → Staging area (.git/index), DIRC v2 format.
  hash.rs        → SHA-1 via sha1 crate: hash_bytes(), hash_git_object(type, content).
  compression.rs → zlib compress/decompress via flate2.
  protocol.rs    → Git smart HTTP: pkt-line codec, ref discovery, packfile parse (ofs_delta + ref_delta), push.
  remote_utils.rs→ Shared helpers: write_objects, apply_tree, collect commits, resolve URLs.
  repo.rs        → find_repo_root(), ensure_dir(), get_current_timestamp().
  checkout.rs    → Working-tree checkout logic (used by clone/checkout commands).
  merge.rs       → Merge logic (used by pull/merge commands).
ai/mod.rs        → AtomicBool for AI mode, ai_commit_marker(), DANGEROUS_COMMANDS list.
output/mod.rs    → AtomicBools for JSON/YAML/no-color modes, print_structured(), colorize().
config/mod.rs    → Config struct: user_name, user_email, aliases. Priority: env > repo .agit/config.toml > ~/.agitconfig.toml > defaults.
utils/error.rs   → AgitError enum (Io, ObjectNotFound, InvalidObject, InvalidRef, CompressionError, RepoNotFound, NotAGitRepo, Other).
```

### Key patterns

- **Global state**: `AI_MODE`, `JSON_MODE`, `YAML_MODE`, `NO_COLOR` are `AtomicBool` statics. Set once in `main()` before dispatch, read anywhere via `is_ai_mode()` / `is_json()` etc.
- **Error handling**: Commands return `Box<dyn Error>`. Core modules use both `Box<dyn Error>` and the `AgitError` enum. `anyhow` is a dependency but `AgitError` is used for structured errors.
- **Alias resolution**: `main.rs::resolve_aliases()` rewrites the CLI args before clap parsing — e.g., `agit co -m "msg"` becomes `agit commit -m "msg"`.
- **Config loading**: `Config::load(repo_path)` cascades: env vars (`AGIT_USER_NAME` / `AGIT_USER_EMAIL` / `GIT_AUTHOR_*`) → repo `.agit/config.toml` → global `~/.agitconfig.toml` → defaults (`"agit"` / `"agit@localhost"`).
- **Commit flow**: `Index::load()` → build `Tree` from index → write tree object → build `Commit` (with parent from HEAD) → write commit object → update branch ref.
- **Network commands** share protocol code in `core/protocol.rs` and helpers in `core/remote_utils.rs`. Clone = discover_refs → clone_full → parse_packfile → write_objects → checkout. Push = discover_refs → collect_local_objects → generate_pack → push_pack. Pull = fetch → merge or fast-forward.

## Commit conventions

Conventional Commits: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `chore:`. Scope optional: `feat(core): ...`.

Branch naming: `feat/<name>`, `fix/<name>`, `docs/<name>`.

## Git compatibility note

This project stores its own config in `.agit/` (not `.git/config`), uses its own object storage under `.git/objects/`, and is compatible with standard Git repositories. The `.git/` directory layout follows Git conventions so `git` and `agit` can operate on the same repo.

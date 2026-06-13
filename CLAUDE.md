# CLAUDE.md

This file provides guidance to Claude Code when working in this repo.
> 默认使用简体中文回复。代码注释使用简体中文。

## Project

**agit (AdapterGit)** — a pure-Rust, zero-external-Git-dependency Git implementation. Objects, refs, index, smart-HTTP protocol all from scratch. Ships as a single static binary. Designed for AI agents, CI/CD, and portable use. Never blocks on interactive prompts.

Version: 0.6.1 | Edition: 2021 | License: Apache-2.0

## Build / Test / Lint

```bash
cargo build              # Debug
cargo build --release    # Release
cargo test               # All 119 tests (87 unit + 32 integration)
cargo clippy             # Must pass w/ 0 warnings
cargo fmt                # Must pass w/ no diff
```

## Architecture

```
main.rs          → Config → alias resolve → CLI parse → dispatch
cli/mod.rs       → clap derive, 24+ subcommands, global flags (--ai --json --yaml --no-color)
commands/        → One file per subcommand, each exposes run(…) → Result<(), Box<dyn Error>>
core/
  objects/       → blob, tree, commit, tag (feature-gated)
  storage.rs     → Loose object R/W (.git/objects/XX/XXXXXX), zlib
  refs.rs        → HEAD, refs/heads/*, refs/tags/*, refs/remotes/*, CRUD
  index.rs       → DIRC v2 staging area
  hash.rs        → SHA-1 (sha1 crate)
  compression.rs → zlib via flate2
  protocol.rs    → Git smart-HTTP: pkt-line, ref discovery, packfile parse, push
  remote_utils.rs→ Shared helpers for network commands
  checkout.rs    → Branch switch, tree restore, index rebuild
  merge.rs       → 3-way merge + fast-forward + conflict markers
  ignore.rs      → .gitignore parser (glob, negation, char-class, dir-only)
  repo.rs        → find_repo_root(), timestamp helpers
ai/mod.rs        → AI-mode flag, DANGEROUS_COMMANDS list
output/mod.rs    → JSON/YAML/no-color mode flags + output formatting
config/mod.rs    → Config: user.name, user.email, aliases (env > .agit/config.toml > ~/.agitconfig.toml > defaults)
utils/error.rs   → AgitError enum
```

## Key Patterns

- **Global state**: `AI_MODE`, `JSON_MODE`, `YAML_MODE`, `NO_COLOR` are `AtomicBool` statics, set once in main before dispatch.
- **Error handling**: Commands return `Box<dyn Error>`. Core uses `AgitError` enum. IO errors propagate via `?` — never swallow with `unwrap_or_default()` on file reads.
- **Config cascade**: env vars (`AGIT_USER_NAME`/`AGIT_USER_EMAIL`) > repo `.agit/config.toml` > global `~/.agitconfig.toml` > defaults.
- **Commit flow**: `Index::load()` → build `Tree` → write tree → build `Commit` (+parent from HEAD/MERGE_HEAD) → write commit → update ref.
- **Network flow**: Clone = discover_refs → fetch_objects → parse_packfile → write_objects → checkout. Push = discover_refs → collect_local_objects → generate_pack → push_pack. Pull = fetch → merge/ff.

## Rules

### Branch & PR

1. **main is protected** — never push directly to main. Create `feat/`, `fix/`, `docs/`, `chore/` branches and use PRs.
2. **Branch naming**: `feat/<name>`, `fix/<name>`, `docs/<name>`, `chore/<name>`.

### Commit

3. **Single Logical Change** — every commit must be one atomic, self-contained change. No "Fix stuff" or "Update code".
4. **Multi-commit PR is OK** — splitting across commits is encouraged (e.g. `refactor:` → `feat:` → `test:`), but never squash unrelated changes into one.
5. **Conventional Commits**: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `chore:`. Scope optional: `feat(core): ...`.

### Quality Gate

6. **Tests required** — new features must have unit + integration tests. `cargo test` must pass with 0 failures before push.
7. **Clippy clean** — `cargo clippy` must produce 0 warnings before push.
8. **Formatted** — `cargo fmt` must produce no diff before push.

### Code

9. **No silent error swallowing** — use `?` or explicit `map_err` for IO. Never `unwrap_or_default()` on file reads.
10. **Feature gate tag** — `#[cfg(feature = "tag")]` code must compile with both `--features tag` and `--no-default-features`.
11. **Windows compatibility** — use `std::fs` APIs, normalize paths with `replace('\\', '/')`.

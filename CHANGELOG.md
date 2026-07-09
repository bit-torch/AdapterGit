# Changelog

[中文文档](CHANGELOG-zh_CN.md)

All notable changes to AdapterGit (agit) will be documented in this file.

---

## [v0.14.0] — 2026-06-27

> Workspace Split + Security Fix — Multi-crate Architecture & Security Fix

### Refactor
- **workspace split**: single crate → 3-crate workspace (agit-core + agit-ai + agit-cli)
- **agit-core**: pure Rust Git core library, independently reusable (19 modules)
- **agit-ai**: AI commit message generation as a standalone crate, carries the reqwest dependency
- **agit-cli**: CLI binary, lite/full dual-edition distribution
- **Dual editions**: `cargo build --no-default-features -F tag` = Lite, `--all-features` = Full

### Security
- **quinn-proto**: upgraded 0.11.14 → 0.11.15 to fix RUSTSEC-2026-0185 (CVSS 7.5)

### Fixes
- **CI**: fix smoke test paths to fit the workspace structure

### Documentation
- Update all 7 documents: CLAUDE.md, README, README-zh_CN, ARCHITECTURE, CONTRIBUTING, CHANGELOG, TODO
- Add `*.bundle` to .gitignore

### Tests
- 178 tests (109 unit + 69 integration), all passing

## [v0.13.0] — 2026-06-21

> Dual-Edition Distribution & AI Commit

### Features
- **lite/full feature flags**: lite (no TLS, pure local), full (TLS + AI)
- **AI commit message**: `agit commit --ai` auto-generates message via LLM from staged diff
- **LLM API module**: OpenAI-compatible, configurable via `AGIT_LLM_API_KEY/URL/MODEL`

### Distribution
- **musl static build**: `x86_64-unknown-linux-musl` CI artifact
- **macOS release**: x86_64 binary tar.gz
- **Linux .deb**: via cargo-deb
- **Docker image**: multi-stage alpine musl build

### CI
- test-lite job (no TLS compilation check)
- 4-platform auto-release (Linux GNU + musl + macOS + Windows)

---

## [v0.12.0] — 2026-06-21

> Bisect — Binary Search

### Features
- **bisect**: binary search for the commit that introduced a bug (start / good / bad / skip / reset / log / run)
- **bisect run**: automatically run a test script to perform the binary search

### Core
- `core/bisect`: state management + range calculation + bisection selection algorithm, persisted to `refs/bisect/*`

### Tests
- 178 tests (109 unit + 69 integration)

---

## [v0.11.0] — 2026-06-21

> Blame & Reflog — Line-by-line Blame and Reference Log

### Features
- **blame**: trace the last modifying commit for each line of a file (LCS line-matching algorithm)
- **reflog**: view reference change history (HEAD / branches / tags)

### Core
- `core/reflog`: reference log read/write module, supports `.git/logs/` parsing and atomic append

### Tests
- 172 tests (103 unit + 69 integration)

---

## [v0.10.0] — 2026-06-21

> SSH Transport Protocol

### Features
- **SSH transport protocol**: supports `git@host:path` and `ssh://` URLs via the system `ssh` command
- **Transport trait**: unified HTTP/SSH transport abstraction (discover_refs / fetch_objects / push_pack)
- **SSH URL parser**: supports SCP format, standard ssh:// format, ~/.ssh/config host aliases and wildcards
- **create_transport()**: auto-dispatches HTTP or SSH based on the URL scheme

### Architecture
- `SshTransport`: zero extra dependencies, invokes the system ssh as a subprocess, transparently inherits keys/known_hosts/agent
- `HttpTransport`: refactored into a Transport trait implementation
- clone/fetch/push commands uniformly use the Transport trait

### Tests
- 168 tests (99 unit + 9 compatibility + 60 integration)

---

## [v0.9.0] — 2026-06-20

> CI/CD Pipeline & Git Compatibility

### CI/CD
- Added **macOS testing**: full-platform matrix (Linux + macOS + Windows)
- Added **smoke tests**: end-to-end user scenarios run automatically in CI
- Added **Security Audit**: cargo-audit dependency vulnerability scanning
- Release builds now depend on smoke tests passing

### Tests
- Added **Git compatibility tests** (`tests/git_compat_test.rs`): 9 comparison tests
  - init / add+commit / status / branch+checkout / merge FF / log / rm+mv / tag
  - Automatically skipped when native Git is not present
- Total tests: 159 (90 unit + 9 compatibility + 60 integration)

---

## [v0.8.0] — 2026-06-20

> Rebase & Cherry-Pick

### Features
- `rebase` command: full rebase operation (--onto, --continue, --skip, --abort)
- `cherry-pick` command: cherry-pick a single commit or multiple commits (--continue, --abort)
- `core::rebase` module: shared backend logic
- Detached HEAD commit support

### Tests
- 134 tests (89 unit + 45 integration)

---

## [v0.6.1] — 2025-06-14

> Code Audit Fixes

### 🐛 Bug Fixes

- **fix(rm)**: Fix `rm` deleting untracked files. Now checks index before deleting, matching Git behavior. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

- **fix(push)**: Fix `push` ignoring the remote name. `get_remote_url()` now matches `[remote "<name>"]` by name. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

- **fix**: Propagate I/O errors instead of silently swallowing them via `unwrap_or_default()` across 6 files (`remote.rs`, `branch.rs`, `reset.rs`, `commit.rs`, `config_cmd.rs`, `remote_utils.rs`). ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

### ✨ Features

- **feat(ai)**: Implement dangerous command guard for AI mode. Blocks `push`, `stash drop`, `branch -D`, `mergetool`, `rebase`, `bisect` when AI mode is active. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

### 📝 Documentation

- **docs**: Add full code audit report with 20 findings across severity levels. ([#11](https://github.com/bit-torch/AdapterGit/pull/11))

---

## [v0.6.0] — 2025-06-13

> P1 Feature Enhancements

### ✨ Features
- feat(tag): add tag CLI command for create/list/delete (P1-7)
- feat: add diff/log/rm/mv enhancements (P1-8, P1-9, P1-10)

### 🐛 Bug Fixes
- fix: propagate IO errors instead of swallowing with `unwrap_or_default()`
- style: fix clippy warnings (needless_return, collapsible_if, op_ref, for_kv_map)

### 🧪 Tests
- 119 total tests (87 unit + 32 integration), 0 failed

---

## [v0.5.5] and earlier

See [GitHub Releases](https://github.com/bit-torch/AdapterGit/releases) for earlier versions.

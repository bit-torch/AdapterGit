# TODO - Task List

[中文文档](TODO-zh_CN.md)

AdapterGit project to-do list. Last updated: 2026-06-27 (v0.14.0 release).

## Phase 10: Workspace Split ✅

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 10.1 | Clean up bundle files + .gitignore | P0 | ✅ Done | Removed 2 bundles, added *.bundle |
| 10.2 | Create workspace root Cargo.toml | P0 | ✅ Done | 3-crate workspace |
| 10.3 | Create agit-core library crate | P0 | ✅ Done | core/ + config/ + utils/, flattened |
| 10.4 | Create agit-ai library crate | P0 | ✅ Done | ai/llm.rs extracted into its own crate |
| 10.5 | Create agit-cli binary crate | P0 | ✅ Done | cli/ + commands/ + output/ + ai/ |
| 10.6 | Migrate integration tests | P0 | ✅ Done | tests/ → agit-cli/tests/ |
| 10.7 | Update all documentation | P0 | ✅ Done | CLAUDE.md, README, ARCHITECTURE, CHANGELOG |

## Phase 1: Project Initialization ✅

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 1.1 | Initialize Rust project | P0 | ✅ Done | cargo init |
| 1.2 | Configure Cargo.toml dependencies | P0 | ✅ Done | sha1, flate2, clap, serde, anyhow, url, native-tls |
| 1.3 | Create directory structure | P0 | ✅ Done | core/, cli/, commands/, ai/, output/, config/, utils/ |
| 1.4 | Implement base CLI framework | P0 | ✅ Done | 24 subcommands, clap derive |
| 1.5 | Set up error handling | P1 | ✅ Done | anyhow, Box<dyn Error>, AgitError |

## Phase 2: Core Object System ✅

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 2.1 | SHA-1 hash implementation | P0 | ✅ Done | sha1 crate, hash_bytes + hash_git_object |
| 2.2 | zlib compression/decompression | P0 | ✅ Done | flate2, compress + decompress + decompress_stream |
| 2.3 | Blob object implementation | P0 | ✅ Done | new / serialize / deserialize / hash |
| 2.4 | Tree object implementation | P0 | ✅ Done | TreeEntry + Tree, supports subdirectories |
| 2.5 | Commit object implementation | P0 | ✅ Done | tree / parents / author / committer / message |
| 2.6 | Tag object implementation | P1 | ✅ Done | lightweight + annotated tags |
| 2.7 | Object storage | P0 | ✅ Done | loose objects read/write, compressed storage |
| 2.8 | Reference system | P0 | ✅ Done | symbolic/detached HEAD, branch CRUD, tag CRUD |
| 2.9 | Index file | P1 | ✅ Done | DIRC v2 format, .gitignore filtering |

## Phase 3: Basic Commands ✅ (all done)

| ID | Command | Description | Priority | Status | Notes |
|----|------|----------|--------|------|------|
| 3.1 | `init` | Initialize a new repository | P0 | ✅ Done |
| 3.2 | `add` | Add files to the staging area (respects .gitignore) | P0 | ✅ Done |
| 3.3 | `commit` | Commit changes (incl. AI mode) | P0 | ✅ Done |
| 3.4 | `status` | Show working tree status | P1 | ✅ Done |
| 3.5 | `log` | Show commit history (--oneline/-n/--all) | P1 | ✅ Done |
| 3.6 | `cat-file` | View object content (-t/-p) | P2 | ✅ Done |
| 3.7 | `ls-tree` | List tree object contents | P2 | ✅ Done |
| 3.8 | `diff` | Compare differences (LCS + --cached/--name-only) | P2 | ✅ Done |
| 3.9 | `show` | Show commit/object info | P2 | ✅ Done |
| 3.10 | `branch` | Branch management (list/create/delete) | P1 | ✅ Done | v0.5.0 |
| 3.11 | `checkout` | Switch branches / restore working tree (--force) | P1 | ✅ Done | v0.5.0 |
| 3.12 | `merge` | Merge branches (fast-forward + 3-way + conflict markers) | P1 | ✅ Done | v0.5.0 |
| 3.13 | `stash` | Stash working changes (push/pop/list/drop) | P2 | ✅ Done | v0.5.0 |
| 3.14 | `reset` | Reset HEAD (--soft/--mixed/--hard) | P1 | ✅ Done | v0.5.0 |
| 3.15 | `rm` | Remove files (--cached) | P2 | ✅ Done | v0.5.0 |
| 3.16 | `mv` | Move/rename files | P2 | ✅ Done | v0.5.0 |
| 3.17 | `config` | Configuration management (--global/--list/--unset/--get) | P2 | ✅ Done | v0.5.0 |

**Phase 3 extra: 8 commands completed (branch/checkout/merge/stash/reset/rm/mv/config)**

## Phase 4: AI Mode and Output

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 4.1 | AI mode `--ai` flag | P1 | ✅ Done | AtomicBool global flag |
| 4.2 | `[AI-committed]` auto-tag | P1 | ✅ Done | Auto-added by commit command |
| 4.3 | JSON output `--json` | P1 | ✅ Done | serde_json + print_structured |
| 4.4 | YAML output `--yaml` | P2 | ✅ Done | serde_yaml |
| 4.5 | Dangerous operation guard | P1 | ✅ Done | DANGEROUS_COMMANDS constant |
| 4.6 | Command auto-translation | P2 | 🔨 Partial | AI commit message generation done, NL2CMD not started |
| 4.7 | Color control `--no-color` | P2 | ✅ Done | ANSI escape sequences |

## Phase 5: Networking ✅

| ID | Command | Description | Priority | Status |
|----|------|----------|--------|------|
| 5.1 | `clone` | Clone a repository (HTTP + TLS) | P1 | ✅ Done |
| 5.2 | `push` | Push to a remote | P1 | ✅ Done |
| 5.3 | `pull` | fetch + merge/fast-forward | P1 | ✅ Done |
| 5.4 | `fetch` | Fetch updates (negotiation) | P1 | ✅ Done |
| 5.5 | `remote` | Remote management (add/list) | P2 | ✅ Done |

**Protocol implementation:**
- [x] pkt-line encode/decode
- [x] HTTP Smart Transport (TLS/HTTPS)
- [x] Packfile parsing (ofs_delta + ref_delta)
- [x] Ref discovery (git-upload-pack)
- [ ] SSH transport protocol
- [ ] Git protocol v2

## Phase 6: Configuration and Extensions

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 6.1 | Environment variable support | P1 | ✅ Done | AGIT_USER_NAME/EMAIL, GIT_AUTHOR_* |
| 6.2 | Config file support | P2 | ✅ Done | .toml format, global + repo level |
| 6.3 | Git alias support | P2 | ✅ Done | Supports aliases with arguments |
| 6.4 | Hooks support | P3 | ⏳ Not started | pre-commit, post-commit, pre-push, etc. |
| 6.5 | Submodule support | P3 | ⏳ Not started | |

## Phase 7: Testing and CI

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 7.1 | Unit tests | P0 | ✅ Done | **87** unit tests all passing |
| 7.2 | Integration tests | P0 | ✅ Done | **32** end-to-end workflow tests (10+22) |
| 7.3 | Native Git compatibility tests | P1 | ✅ Done | 9 comparison tests (init/add/commit/status/branch/merge/log/rm/tag) |
| 7.4 | CI pipeline (GitHub Actions) | P1 | ✅ Done | fmt + clippy + test + smoke + security on push/PR |
| 7.5 | Cross-platform build verification | P2 | ✅ Done | Linux/macOS/Windows CI matrix + dual-platform smoke tests |
| 7.6 | Release build + publish | P1 | ✅ Done | cargo build --release + GitHub Release on tag

## Phase 8: Advanced Commands

| ID | Command | Description | Priority | Status | Notes |
|----|------|----------|--------|------|------|
| 8.1 | `rebase` | Rebase (non-interactive) | P1 | ✅ Done | v0.8.0 |
| 8.2 | `cherry-pick` | Cherry-pick commits | P1 | ✅ Done | v0.8.0 |
| 8.3 | `blame` | Line-by-line blame | P2 | ✅ Done | v0.11.0 |
| 8.4 | `reflog` | Reference log | P2 | ✅ Done | v0.11.0 |
| 8.5 | `bisect` | Binary search for the commit that introduced a bug | P2 | ✅ Done | v0.12.0 |
| 8.6 | `grep` | Search working tree contents | P3 | ⏳ Not started | |
| 8.7 | SSH transport protocol | P1 | ✅ Done | v0.10.0: subprocess ssh, zero extra dependencies |
| 8.8 | Git protocol v2 | P2 | ⏳ Not started | |

## Phase 9: Distribution and Packaging (formerly Phase 8, renumbered)

| ID | Task | Priority | Status | Notes |
|----|------|--------|------|------|
| 9.1 | Static build (musl) | P1 | ⏳ Pending fix | CI lacks musl-gcc toolchain, needs a Linux test environment |
| 9.2 | Linux installers (.deb/.rpm) | P2 | ✅ Done | v0.13.0 (.deb via cargo-deb) |
| 9.3 | macOS installers (.dmg / Homebrew) | P2 | ✅ Done | v0.13.0 (binary tar.gz) |
| 9.4 | Windows installers (.msi / Scoop) | P2 | ⏳ Not started | |
| 9.5 | Docker image | P2 | ✅ Done | v0.13.0 |
| 9.6 | CI/CD auto-release pipeline | P1 | ✅ Done | v0.13.0 (4-platform auto-release) |

## Progress Summary

| Phase | Topic | Completion | Status |
|-------|------|--------|------|
| 1 | Project initialization | 100% (5/5) | ✅ |
| 2 | Core object system | 100% (9/9) | ✅ |
| 3 | Basic commands | 100% (17/17) | ✅ |
| 4 | AI mode and output | 93% (6.5/7) | 🔨 |
| 5 | Networking | 100% (5/5) | ✅ |
| 6 | Configuration and extensions | 60% (3/5) | 🔨 |
| 7 | Testing and CI | 100% (6/6) | ✅ |
| 8 | Advanced commands | 75% (6/8) | 🔨 |
| 9 | Distribution and packaging | 83% (5/6) | 🔨 |

**Overall: 61/68 ≈ 90%**

### Milestones

| Version | Description | Status | Key deliverables |
|------|------|------|----------|
| v0.1.0 | Project skeleton + core objects | ✅ Released | init, hash, blob, tree, commit, storage |
| v0.2.0 | Local commands + AI mode | ✅ Released | add, commit, status, log, diff, --ai |
| v0.3.0 | Networking | ✅ Released | clone, push, pull, fetch, remote, HTTP+TLS |
| v0.4.0 | Tag + config | ✅ Released | tag, config, env vars, .toml config |
| v0.5.0 | Advanced local commands | ✅ Released | branch, checkout, merge, stash, reset, rm, mv |
| v0.6.0 | Security hardening + test expansion | ✅ Released | Security audit fixes, 87 unit + 32 integration |
| v0.8.0 | Rebase + cherry-pick | ✅ Released | rebase, cherry-pick, 150 tests |
| v0.9.0 | CI/CD + Smoke + Git compatibility | ✅ Released | 3-platform CI, smoke tests, Git compatibility tests |
| **v0.10.0** | **SSH transport protocol** | ✅ **Released** | SSH transport (subprocess), Transport trait |
| **v0.11.0** | **Blame + Reflog** | ✅ **Released** | blame, reflog, 103 unit tests |
| **v0.12.0** | **Bisect** | ✅ **Released** | bisect (start/good/bad/skip/reset/log/run), 109 unit tests |
| **v0.13.0** | **Dual-edition distribution + AI commit** | ✅ **Released** | lite/full, musl, .deb, macOS, Docker, LLM commit |
| **v0.14.0** | **Workspace split** | ✅ **Released** | agit-core + agit-ai + agit-cli, three crates |
| v1.0.0 | Complete Git subset + all-platform distribution | 🏁 Target | Full command coverage + multi-platform installers |

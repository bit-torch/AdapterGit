# AdapterGit Development Plan

[中文文档](PLAN-zh_CN.md)

## Project Goals

AdapterGit (agit) is a Git tool designed for the AI era, implementing Git core protocols and algorithms natively from the ground up in Rust.

Version: **v0.4.1** | Overall progress: **77%**

### Core Values

- 🤖 **AI-First**: Zero TUI blocking, structured output, safe AI Agent invocation
- 📦 **Dual Distribution**: Full installer + Lite single-file portable, sharing the same native Git core
- 🔒 **Safety Protection**: Dangerous operation interception, automatic `[AI-committed]` tagging
- ⚡ **Never Hangs**: Automatically skips editors, friendly to non-TTY environments

## Completed (v0.1.0 – v0.4.1)

### Phase 1: Project Initialization ✅

| Task | Status |
|------|------|
| Rust project initialization | ✅ |
| Cargo.toml dependencies (sha1, flate2, clap, serde, anyhow, url, native-tls) | ✅ |
| Directory structure (core/cli/commands/ai/output/config/utils) | ✅ |
| CLI framework (15 subcommands, clap derive) | ✅ |
| Error handling (anyhow + AgitError) | ✅ |

### Phase 2: Core Object System ✅ (8/9)

| Task | Status |
|------|------|
| SHA-1 hashing (hash_bytes + hash_git_object) | ✅ |
| zlib compression/decompression (compress + decompress + decompress_stream) | ✅ |
| Blob object | ✅ |
| Tree object (with subdirectory support) | ✅ |
| Commit object (multiple parents) | ✅ |
| Object storage (loose objects) | ✅ |
| Reference system (HEAD + refs CRUD) | ✅ |
| Index file (DIRC v2) | ✅ |

### Phase 3: Basic Commands ✅

| Command | Function | Status |
|------|------|------|
| init | Initialize repository (.git/ + config + HEAD) | ✅ |
| add | Files → staging area (recursive directories, mode detection) | ✅ |
| commit | Commit (tree → commit → update branch) | ✅ |
| status | Area status (staged/modified/deleted/untracked) | ✅ |
| log | Commit history traversal | ✅ |
| cat-file | Object viewing (-t/-p) | ✅ |
| ls-tree | Tree content listing | ✅ |
| diff | Diff comparison (LCS algorithm) | ✅ |
| show | Commit/object details | ✅ |

### Phase 4: AI Mode and Output ✅ (6/7)

| Task | Status |
|------|------|
| AI mode (`--ai` flag) | ✅ |
| Automatic `[AI-committed]` tagging | ✅ |
| JSON output (`--json`) | ✅ |
| YAML output (`--yaml`) | ✅ |
| Dangerous operation protection (DANGEROUS_COMMANDS) | ✅ |
| Color control (`--no-color`) | ✅ |

### Phase 5: Network Features ✅

| Command | Function | Status |
|------|------|------|
| clone | Clone repository (HTTP + TLS) | ✅ |
| push | Push (packfile generation) | ✅ |
| pull | fetch + merge/fast-forward | ✅ |
| fetch | Fetch (want/have negotiation) | ✅ |
| remote add/list | Remote management | ✅ |

**Protocol layer implementation:**
- pkt-line encoding/decoding
- HTTP Smart Transport + TLS
- Packfile parsing + delta decoding (ofs_delta + ref_delta)
- Ref discovery

### Phase 6: Configuration (partial) 🔨

| Task | Status |
|------|------|
| Environment variables (AGIT_USER_NAME/EMAIL, GIT_AUTHOR_*) | ✅ |

## To Do (v0.4.1+)

### Phase 2 Remaining

| Task | Priority |
|------|--------|
| Tag object | P1 |

### Phase 4 Remaining

| Task | Priority |
|------|--------|
| Automatic command conversion | P2 |

### Phase 6: Configuration and Extensions

| Task | Priority |
|------|--------|
| Config file (.toml) | P2 |
| Git aliases | P2 |
| Hooks | P3 |
| Submodule | P3 |

### Phase 7: Testing and Release

| Task | Priority | Status |
|------|--------|------|
| Unit tests | P0 | 🔨 (7/9 modules covered) |
| Integration tests | P0 | ⏳ |
| Git consistency tests | P1 | ⏳ |
| Cross-platform compilation | P1 | ⏳ |
| Static compilation (musl) | P1 | ⏳ |
| Release builds | P0 | ⏳ |

### Phase 8: Full/Lite Dual Distribution

| Task | Priority |
|------|--------|
| Lite single-file portable version | P1 |
| Full .deb installer | P1 |
| Full .rpm installer | P2 |
| Full .msi installer | P2 |
| Full .dmg installer | P2 |
| CI/CD dual-version pipeline | P1 |
| GitHub Release auto-publishing | P1 |

## Technical Decisions

| Library | Purpose | Decision |
|----|------|------|
| sha1 0.10 | SHA-1 hashing | ✅ |
| flate2 1 | zlib compression | ✅ |
| clap 4 | CLI parsing | ✅ |
| serde 1 + serde_json + serde_yaml | Structured output | ✅ |
| anyhow 1 | Error handling | ✅ |
| url 2 | URL parsing | ✅ |
| native-tls 0.2 | TLS/HTTPS | ✅ |
| gix / gitoxide | ❌ Not used (pure native) | ❌ |
| System git command | ❌ No dependency | ❌ |

## Milestones

| Version | Content | Status | Date |
|------|------|------|------|
| v0.1.0 | Project skeleton + core object system + basic commands | ✅ | 2025-07 |
| v0.2.0 | AI mode + structured output + P2 commands | ✅ | 2025-07 |
| v0.3.0 | Network features (clone/push/pull/fetch/remote) | ✅ | 2025-07 |
| **v0.4.1** | Tag + config files + integration tests + branch switching cleanup | ✅ Current | TBD |
| v1.0.0 | Complete Git subset + all-platform installers + documentation | 🎯 | TBD |

## Related Documentation

- [Architecture Design](docs/ARCHITECTURE.md)
- [To-Do](../TODO.md)

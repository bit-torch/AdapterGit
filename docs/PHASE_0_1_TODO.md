# Phase 0 & 1 To-Do

[中文文档](PHASE_0_1_TODO-zh_CN.md)

> ✅ **All completed (v0.4.1)**
> 
> The detailed tasks of Phase 0-1 were all implemented during the v0.1.0–v0.4.1 development process.
> Current project version: **v0.4.1**, see [TODO.md](../TODO.md) and [PLAN.md](PLAN.md) for details.

## Phase 0: Project Initialization ✅

| # | Task | Status |
|---|------|------|
| 0.1 | cargo init | ✅ |
| 0.2 | Cargo.toml dependencies | ✅ |
| 0.3 | Directory structure | ✅ |
| 0.4 | main.rs | ✅ |
| 0.5 | AgitError | ✅ |
| 0.6 | Logging support | ✅ |
| 0.7 | Compile verification | ✅ |

| # | Task | Status |
|---|------|------|
| 1.1 | CLI global flags (--ai/--json/--yaml/--no-color) | ✅ |
| 1.2 | Command enum (15 subcommands) | ✅ |
| 1.3 | Subcommand arguments (commit -m/--ai) | ✅ |
| 1.4 | Command routing (main.rs match dispatch) | ✅ |
| 1.5 | Handler function implementation | ✅ |
| 1.6 | cargo check + clippy pass | ✅ |

## Phase 1: Core Object System ✅

| # | Submodule | Status |
|---|--------|------|
| 2 | SHA-1 hashing | ✅ |
| 3 | zlib compression | ✅ |
| 4 | Blob object | ✅ |
| 5 | Tree object | ✅ |
| 6 | Commit object | ✅ |
| 7 | Object storage | ✅ |
| 8 | Reference system | ✅ |
| 9 | Index file | ✅ |
| 10 | Integration verification | ✅ |

## Status Legend

- ⏳ Not started
- 🔨 In progress
- ✅ Completed
- ❌ Cancelled

> This document is the historical record of Phase 0-1; for the current development plan, see [PLAN.md](PLAN.md) and [TODO.md](../TODO.md).

# Good First Issues

[中文文档](GOOD_FIRST_ISSUES-zh_CN.md)

Welcome and thanks for contributing! Below is a list of tasks suitable for new contributors. These tasks have been curated for newcomers who know Rust but are not yet familiar with the project.

## How to Choose a Task

1. Browse the task list and pick one that interests you
2. Leave a comment on the issue to claim it
3. Fork the project and create a branch
4. Submit a PR once development is complete

---

## 🎯 Getting Started Tasks (Easy)

### 1. Improve Error Messages
**Priority**: P2  
**Module**: cli  
**Description**: Improve error messages so users can more easily understand the problem  
**Difficulty**: ⭐ (entry)  
**Requirements**: Basic Rust  

**Example improvement**:
```rust
// Before
Err("Object not found")

// After
Err(format!("Object '{}' not found in repository", object_id))
```

### 2. Add More Help Information
**Priority**: P2  
**Module**: cli  
**Description**: Add more detailed help information and usage examples for commands  
**Difficulty**: ⭐ (entry)  
**Requirements**: Documentation writing  

**Task scope**:
- Add `--help` examples for each command
- Add common usage examples
- Add a troubleshooting guide

### 3. Improve Code Comments
**Priority**: P3  
**Module**: general  
**Description**: Add or improve comments at key points in the code  
**Difficulty**: ⭐ (entry)  
**Requirements**: Understanding of code logic  

---

## 📝 Documentation Tasks

### 4. Write a Core Concepts Tutorial
**Priority**: P1  
**Module**: docs  
**Description**: Write tutorial documentation on Git internals  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**: Understanding of basic Git concepts  

**Suggested content**:
- Introduction to the Git object model
- SHA-1 hashing explained
- The .git directory structure
- The Git references system

**Related file**: `docs/ARCHITECTURE.md`

### 5. Add Usage Examples
**Priority**: P2  
**Module**: docs  
**Description**: Add complete usage examples for each command  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**: Basic writing ability  

### 6. Create a Troubleshooting Guide
**Priority**: P2  
**Module**: docs  
**Description**: Collect common problems and their solutions  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**: Familiarity with common Git issues  

---

## 🔧 Coding Tasks

### 7. Implement the `git rev-parse` command
**Priority**: P1  
**Module**: core/refs  
**Description**: Implement the rev-parse command, used to resolve references  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**: 
- Basic Rust
- Understanding of the Git references system

**Functionality**:
```bash
./agit rev-parse HEAD
./agit rev-parse --symbolic-full-name HEAD
./agit rev-parse --sqrq HEAD
```

**Learning resources**:
- [Git Internals - References](https://git-scm.com/book/zh/v2/Git-内部原理-Git-引用)

### 8. Implement the `git rev-list` command
**Priority**: P1  
**Module**: core/objects  
**Description**: Implement the rev-list command to list commit history  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Understanding of the Git object graph

**Functionality**:
```bash
./agit rev-list HEAD
./agit rev-list --count HEAD
./agit rev-list --max-count=5 HEAD
```

### 9. Implement `git branch -l` (list branches)
**Priority**: P1  
**Module**: core/refs  
**Description**: Implement basic branch listing functionality  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Understanding of Git refs

**Functionality**:
```bash
./agit branch          # List local branches
./agit branch -a      # List all branches
./agit branch -v      # Show verbose information
```

### 10. Add Configuration File Parsing
**Priority**: P1  
**Module**: config  
**Description**: Implement TOML configuration file parsing  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Familiarity with the TOML format

**Configuration file**:
```toml
# ~/.config/agit/config.toml
[ai]
auto_tag = true
tag_format = "suffix"

[output]
format = "json"
color = true
```

### 11. Implement JSON Output Formatting
**Priority**: P1  
**Module**: output  
**Description**: Add JSON output for commands such as status and log  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Experience using serde

**Functionality**:
```bash
./agit status --json
./agit log --json
```

### 12. Implement `git diff --stat`
**Priority**: P2  
**Module**: core/diff  
**Description**: Implement diff statistics display  
**Difficulty**: ⭐⭐⭐⭐ (hard)  
**Requirements**:
- Basic Rust
- Understanding of diff algorithms

**Functionality**:
```bash
./agit diff --stat
# example output:
#  file1.txt | 5 +++++
#  file2.txt | 2 --
#  2 files changed, 3 insertions(+), 2 deletions(-)
```

---

## 🧪 Testing Tasks

### 13. Add Unit Tests for Core Algorithms
**Priority**: P1  
**Module**: core  
**Description**: Add tests for core algorithms such as SHA-1 and zlib  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**:
- Basic Rust testing
- Unit testing experience

**Test scope**:
- SHA-1 known-value tests
- zlib compression/decompression tests
- Object serialization tests

### 14. Add Command Integration Tests
**Priority**: P1  
**Module**: cli  
**Description**: Add integration tests for each command  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**:
- Basic Rust testing
- Familiarity with integration testing

**Test framework**: `assert_cmd`, `predicates`

### 15. Add Comparison Tests Against Native Git Output
**Priority**: P2  
**Module**: tests  
**Description**: Create tests that automatically compare agit and git output  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Familiarity with the Git command line

---

## 🚀 Optimization Tasks

### 16. Add Performance Benchmarks
**Priority**: P2  
**Module**: tests  
**Description**: Use criterion to create performance benchmarks  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**:
- Basic Rust
- Experience using criterion

### 17. Implement an LRU Cache
**Priority**: P2  
**Module**: core/storage  
**Description**: Add an LRU cache for object reads  
**Difficulty**: ⭐⭐⭐ (medium)  
**Requirements**:
- Basic Rust
- Familiarity with LRU algorithms

---

## 🎨 Tooling Tasks

### 18. Configure GitHub Actions CI
**Priority**: P1  
**Module**: .github/workflows  
**Description**: Configure automated CI/CD  
**Difficulty**: ⭐⭐ (easy)  
**Requirements**:
- Familiarity with GitHub Actions
- YAML writing experience

**Functionality**:
- Rust testing
- Code formatting checks
- Clippy checks
- Multi-platform builds

### 19. Create Cargo Workspace Configuration
**Priority**: P3  
**Module**: root directory  
**Description**: Configure the Cargo workspace to support multiple crates  
**Difficulty**: ⭐ (entry)  
**Requirements**: Cargo usage experience

---

## 📋 Claiming a Task

1. Leave a comment on the issue: "I'd like to work on this"
2. Fork the project
3. Create a new branch: `git checkout -b your-name/issue-name`
4. Develop and test
5. Submit a PR

## Task Labels

| Label | Meaning |
|------|------|
| `good first issue` | Entry-level tasks suitable for newcomers |
| `documentation` | Documentation-related tasks |
| `enhancement` | Feature enhancement |
| `bug` | Bug fix |
| `help wanted` | Tasks that need help |

## Resource Links

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Pro Git Book](https://git-scm.com/book/en/v2)
- [Project Architecture](docs/ARCHITECTURE.md)
- [Development Plan](docs/PLAN.md)

---

**Questions?** Feel free to ask in [GitHub Discussions](https://github.com/bit-torch/AdapterGit/discussions)!

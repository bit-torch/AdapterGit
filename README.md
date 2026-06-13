# AdapterGit - Git for AI, not for editors

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/bit-torch/AdapterGit)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Static Binary](https://img.shields.io/badge/binary-static%20musl-green.svg)](https://github.com/bit-torch/AdapterGit/releases)

[中文文档](README-zh_CN.md)

**AdapterGit (agit)** — A TUI-free Git implementation that never hangs. Designed for automation, scripting, CI/CD, and shared computer environments.

## The Problem

- AI agents get stuck when Git opens a TUI editor
- Installing Git on every school lab machine is tedious
- Git commands unexpectedly launch interactive interfaces in scripts
- Native Git behaves unpredictably in non-TTY environments

**agit solves all of these.**

## Key Features

### AI-First Design
- Zero TUI blocking — safe for AI agents
- Structured JSON output, machine-readable
- Automatic `[AI-committed]` tagging
- Dangerous operation guards to prevent AI misoperations

### Dual Edition Strategy

agit ships in two editions. Both implement Git core logic natively from scratch in Rust with zero external Git dependencies:

| | **Full Edition** | **Lite Edition** |
|---|---|---|
| Format | Installable application package | Single-file portable binary |
| Installation | One-click installer (.msi / .deb / .dmg) | Download and run, no install needed |
| Size | ~20MB installer | ~10MB single file |
| Use case | Personal dev machines, enterprise deployment | AI agents, CI/CD, temporary environments, USB drives |
| System integration | PATH registration, context menu, file associations | Zero footprint, no system traces |
| Git core | Native Rust implementation | Native Rust implementation |

Both editions share the same native Rust Git core (SHA-1, zlib, Blob/Tree/Commit objects, refs, index, network protocols) — only the distribution format differs.

### Never Hangs
- Automatically skips all editors
- Intelligently converts interactive commands
- Non-TTY friendly
- Zero-configuration in CI/CD environments

### Git Compatible
- Works with existing Git repositories and workflows
- Supports a common subset of Git commands
- Gradual replacement of `git` commands
- Transparent fallback mechanism

## Quick Start

### Lite Edition (single-file portable)

```bash
# Linux / macOS
curl -L https://github.com/bit-torch/AdapterGit/releases/latest/download/agit-lite-x86_64-unknown-linux-musl -o agit
chmod +x agit
./agit --help

# Run directly — no installation required
./agit init
```

### Full Edition (installer)

```bash
# Linux (.deb)
curl -LO https://github.com/bit-torch/AdapterGit/releases/latest/download/agit_0.1.0_amd64.deb
sudo dpkg -i agit_0.1.0_amd64.deb

# macOS (.dmg) — download and double-click to install

# Windows (.msi) — download and run the installer wizard
```

### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build agit
git clone https://github.com/bit-torch/AdapterGit.git
cd agit
cargo build --release

# Static build (recommended)
cargo build --release --target x86_64-unknown-linux-musl
```

## Usage Examples

### Basic Usage (just like Git)

```bash
agit init
agit add .
agit commit -m "feat: add new feature"
agit push origin main
```

### AI Mode

```bash
# AI invocation — never stuck in an editor
agit commit --ai "fix: login bug"

# Structured JSON output
{
  "status": "success",
  "command": "commit",
  "commit_hash": "abc123def456",
  "message": "fix: login bug\n\n[AI-committed]",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Portable Mode

```bash
# Anywhere, no installation needed
cd /tmp/some-project
/path/to/agit add -A
/path/to/agit commit -m "work from public computer"
```

### Shell Alias

```bash
# Temporarily replace git
alias git=agit

# Or only for specific scenarios
alias gai='agit --ai'
```

## Installation

### Lite Edition

```bash
# Via cargo
cargo install agit --features lite

# Or download the single binary directly
curl -L https://github.com/bit-torch/AdapterGit/releases/latest/download/agit-lite -o agit
chmod +x agit
sudo mv agit /usr/local/bin/  # optional, add to PATH
```

### Full Edition

```bash
# Via cargo
cargo install agit

# Linux (deb)
sudo dpkg -i agit_0.1.0_amd64.deb

# Linux (rpm)
sudo rpm -i agit-0.1.0-1.x86_64.rpm

# macOS — download .dmg and double-click, or use Homebrew:
brew install bit-torch/tap/agit

# Windows — download .msi installer, or use winget:
winget install bit-torch.agit
```

### Manual Installation (both editions)

1. Download the appropriate release from [GitHub Releases](https://github.com/bit-torch/AdapterGit/releases)
2. Lite: `chmod +x agit` and run directly
3. Full: run the installer or add execute permission and move to a PATH directory

## Comparison

### agit vs Native Git

| Feature | agit Full | agit Lite | Native Git |
|---------|-----------|-----------|------------|
| AI-safe (no TUI hang) | Yes | Yes | No |
| Distribution | Installer (.msi/.deb/.dmg) | Single binary (~10MB) | Full installation required |
| Single-file portable | No (requires install) | Yes | No |
| System integration | PATH / context menu / file associations | Zero footprint | Full integration |
| Structured output | JSON / YAML | JSON / YAML | Plain text only |
| Zero-config run | Yes | Yes | No (requires git config) |
| Native Git core | Pure Rust | Pure Rust | C |
| Interactive operations | Not supported | Not supported | Full support |

### Full vs Lite — Which to Choose?

| Scenario | Recommended Edition |
|----------|---------------------|
| Personal dev machine daily use | Full |
| Enterprise batch deployment | Full |
| AI Agent / automation scripts | Lite |
| CI/CD pipelines | Lite |
| Docker containers | Lite |
| USB drive / public computers | Lite |
| Context menu integration needed | Full |
| Quick use in temporary environments | Lite |

## AI Mode in Depth

agit is purpose-built for AI agents:

### Auto-Tagging

```bash
agit commit --ai "fix login issue"
# Commit message automatically includes: [AI-committed]
```

### Non-Interactive Conversion

```bash
# agit automatically converts these dangerous commands
git commit          -> agit commit -m "[AI] auto-commit"
git rebase -i       -> agit rebase --no-edit
git add -p          -> agit add -A
git mergetool       -> Rejected
```

### Machine-Readable Output

```bash
agit log --json
agit status --json
agit diff --json
```

## Architecture

```
┌─────────────────────────────────────────┐
│             AI Agent / Script           │
└─────────────────┬───────────────────────┘
                  │ JSON / Structured Output
┌─────────────────▼───────────────────────┐
│              agit (Adapter Layer)        │
│  ┌───────────────────────────────────┐  │
│  │  TUI Elimination │ Portability │ AI Safety  │  │
│  └─────────────────┬─────────────────┘  │
└─────────────────┬───────────────────────┘
                  │ Pure Rust Native Implementation
┌─────────────────▼───────────────────────┐
│         Native Git Core (Pure Rust)      │
│  ┌───────────────────────────────────┐  │
│  │  Object Store │ Refs │ Diff Algorithm   │  │
│  │  Pack Files   │ Protocol │ Index System  │  │
│  └───────────────────────────────────┘  │
└─────────────────┬───────────────────────┘
                  │ Unified Core → Dual Distribution
┌─────────────────▼───────────────────────┐
│            Distribution Layer            │
│  ┌─────────────────┬─────────────────┐  │
│  │  Full Edition    │  Lite Edition   │  │
│  │  .msi/.deb/.dmg │  Single Binary  │  │
│  │  Installer       │  Download & Run │  │
│  └─────────────────┴─────────────────┘  │
└─────────────────────────────────────────┘
```

**Full and Lite share the same native Rust Git core — only the distribution format differs.**

## Supported Commands

### Implemented
- `init` — Initialize a repository
- `add` — Stage files
- `commit` — Commit changes
- `push` / `pull` — Remote operations
- `status` — Show working tree status
- `log` — Show commit history
- `clone` — Clone a repository

### In Progress
- `branch` — Branch management
- `checkout` — Switch branches
- `merge` — Merge branches
- `stash` — Stash changes

### Not Planned
- `rebase -i` (interactive rebase)
- `add -p` (interactive patching)
- `git mergetool` (merge tool)
- All other TUI-interactive commands

## Configuration

### Environment Variables

```bash
# Force AI mode
export AGIT_AI_MODE=1

# Set output format
export AGIT_OUTPUT_FORMAT=json  # json, yaml, text

# Disable color
export AGIT_NO_COLOR=1
```

### Config File

`~/.config/agit/config.toml`

```toml
[ai]
auto_tag = true
tag_format = "suffix"  # prefix, suffix, trailer

[output]
format = "json"
color = true

[safety]
prevent_force_push = true
max_commit_length = 100
```

## Integration Examples

### GitHub Actions

```yaml
- name: Checkout with agit
  uses: bit-torch/agit-action@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

### Docker

```dockerfile
COPY --from=ghcr.io/bit-torch/AdapterGit:latest /agit /usr/local/bin/
RUN agit clone https://github.com/user/repo.git
```

### AI Agent (Python)

```python
import subprocess

result = subprocess.run(
    ["agit", "commit", "--ai", "Auto-commit by AI"],
    capture_output=True,
    text=True
)
print(result.stdout)  # JSON output
```

## Contributing

Contributions are welcome! agit is open source and we appreciate all forms of contribution.

### Development Setup

```bash
# 1. Fork and clone the repository
git clone https://github.com/bit-torch/AdapterGit.git
cd agit

# 2. Install Rust
rustup toolchain install stable

# 3. Build
cargo build

# 4. Run tests
cargo test
```

### Commit Convention

agit follows [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation update
- `test:` — Test related
- `refactor:` — Code refactoring

### Project Structure

```
agit/
├── src/
│   ├── cli/      # Command-line parsing
│   ├── git/      # Git core functionality
│   ├── ai/       # AI mode implementation
│   ├── output/   # Output formatting
│   └── utils/    # Utility functions
├── tests/        # Integration tests
└── examples/     # Usage examples
```

## License

This project is licensed under **Apache-2.0**.

## Acknowledgments

- **Fully native implementation** — Git core protocols and algorithms implemented from scratch in Rust, with zero external Git library dependencies
- Inspired by [GitButler](https://gitbutler.com/) and [gitui](https://github.com/extrawurst/gitui)
- Thanks to every developer who has been frustrated by Git on shared machines

## Reporting Issues

Found a bug or have an idea?
- [Open an Issue](https://github.com/bit-torch/AdapterGit/issues)
- [Submit a PR](https://github.com/bit-torch/AdapterGit/pulls)
- [Join the Discussion](https://github.com/bit-torch/AdapterGit/discussions)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=bit-torch/AdapterGit&type=Date)](https://star-history.com/#bit-torch/AdapterGit&Date)

---

> A Git tool designed for the AI era — never hang, ready out of the box.

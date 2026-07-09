# Contributing Guide

[中文文档](CONTRIBUTING-zh_CN.md)

Thank you for your interest in the AdapterGit project! We welcome contributions of all kinds, whether code, documentation, or issue reports.

## How to Contribute

### Reporting Issues

Found a bug or have a new idea? Please:

1. Open a new issue in [GitHub Issues](https://github.com/bit-torch/AdapterGit/issues)
2. Choose an appropriate label (bug, feature, documentation, etc.)
3. Provide a detailed description of the problem and reproduction steps

### Contributing Code

#### 1. Fork and Clone

```bash
git clone https://github.com/bit-torch/AdapterGit.git
cd AdapterGit
```

#### 2. Create a Branch

Use a clear branch naming convention:

```bash
# Feature branch
git checkout -b feat/add-clone-command

# Bug fix
git checkout -b fix/status-command-error

# Documentation update
git checkout -b docs/update-readme
```

#### 3. Development Environment

**Requirements**:
- Rust 1.70+
- Cargo (installed with Rust)

**Install Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Build the project**:
```bash
cargo build                       # Debug build
cargo build --release             # Release build
cargo build --release --all-features  # Full version (with AI)
cargo build --release --no-default-features -F tag  # Lite version
```

#### 4. Run Tests

```bash
cargo test            # Run all tests
cargo test --doc      # Doc tests
cargo clippy          # Lint checks
cargo fmt            # Code formatting
```

#### 5. Commit Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting (no functional impact)
- `refactor`: Refactoring (not a new feature or fix)
- `test`: Test-related
- `chore`: Build or auxiliary tooling

**Examples**:

```bash
git commit -m "feat(core): add SHA-1 hash implementation"
git commit -m "fix(cli): resolve init command panic on empty directory"
git commit -m "docs(readme): update installation instructions"
```

#### 6. Push and Create a PR

```bash
git push origin feat/add-clone-command
```

Create a Pull Request on GitHub describing your changes.

## Code Standards

### Rust Coding Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Format code with `cargo fmt`
- Lint code with `cargo clippy`
- Write doc comments (///)
- Add unit tests

### Documentation Requirements

- All public APIs require doc comments
- Update related documentation
- Add usage examples

### Testing Requirements

- Core functionality must have tests
- Keep tests fast to run
- Test edge cases

## Project Structure

```
agit/                         # Workspace root
├── Cargo.toml                # Workspace definition
├── agit-core/                # Rust native Git core library
│   └── src/
│       ├── objects/          # Blob, Tree, Commit, Tag
│       ├── storage.rs        # Loose object read/write
│       ├── refs.rs           # Reference management (HEAD, branches, tags)
│       ├── index.rs          # DIRC v2 staging area
│       ├── protocol.rs       # Git smart-HTTP protocol
│       ├── merge.rs          # 3-way merge
│       └── checkout.rs       # Branch switching / tree restoration
├── agit-ai/                  # AI mode (optional, feature-gated)
│   └── src/
│       └── lib.rs            # AI auto-tagging, safety guards
├── agit-cli/                 # CLI binary entry point
│   └── src/
│       ├── main.rs           # Entry point
│       ├── commands/         # One file per subcommand
│       └── output/           # JSON / YAML / no-color output
└── tests/                    # Integration tests
```

See: [Architecture Design](ARCHITECTURE.md)

## Development Phases

View the current development progress: [Development Plan](docs/PLAN.md)

### Current Priorities

1. **P0**: Core object system (SHA-1, zlib, object model)
2. **P0**: Basic commands (init, add, commit)
3. **P1**: AI mode
4. **P1**: Networking features (clone, push, pull)

## Good First Issues

Want to start contributing? Check out the [Good First Issues](../GOOD_FIRST_ISSUES.md)

## Getting Help

- 📖 See the [README](../README.md)
- 💬 Join the discussion: [GitHub Discussions](https://github.com/bit-torch/AdapterGit/discussions)
- 🐛 Report issues: [GitHub Issues](https://github.com/bit-torch/AdapterGit/issues)

## License

By contributing, you agree that your code will be licensed under the Apache 2.0 license.

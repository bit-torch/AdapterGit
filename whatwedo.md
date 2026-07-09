# What agit Does — Full Workflow from a User's Perspective

[中文文档](whatwedo-zh_CN.md)

> Plain-language walkthrough: open a terminal and use agit to do all your Git work, end to end.

## 1. Getting it

- **Lite edition**: a single file — download and run, no install needed. Toss it on a USB stick, in a CI container, or on a shared machine.
- **Full edition**: install auto-configures PATH; includes AI commit message generation.
- Also supports `cargo install`, or build it yourself with `cargo build` (~180 seconds to a binary).

## 2. Start a repository

```
agit init                      # Turn the current directory into a repo
agit init --path /tmp/project  # Specify a directory
agit init --pattern rust       # Also generate a .gitignore (rust/python/node/go/java)
agit init --licence mit        # Also generate a LICENCE file
```

## 3. Everyday commits

```
agit add .                     # Add everything
agit add src/main.rs           # Add a single file
      ↓ Automatically respects .gitignore
agit commit -m "fix: typo"     # Normal commit
agit commit --ai "修登录"       # AI writes the commit message for you
      ↓ Auto-tags [AI-committed] for traceability
```

**How does AI commit message generation work?** Set up `AGIT_LLM_API_KEY` (OpenAI / DeepSeek / Moonshot / Zhipu / Ollama all work), and `agit commit --ai` will automatically feed the staged diff to an LLM and produce a Conventional Commits-formatted commit message.

## 4. Check status

```
agit status                    # What changed, what's staged
agit log                       # Commit history
agit log --oneline -n 10       # Compact mode
agit log --all                 # All branches
agit diff                      # What changed
agit diff --cached             # What's staged
agit diff --name-only          # File names only
agit diff <commit1> <commit2>  # Between two commits
agit show HEAD                 # Details of the latest commit
```

## 5. Branches and switching

```
agit branch                    # List local branches
agit branch -c feat/xxx        # Create a branch
agit branch -d feat/xxx        # Delete a branch
agit checkout feat/xxx          # Switch to it
agit checkout --force feat/xxx  # Switch even with uncommitted changes (auto stash then pop)
```

## 6. Collaborating with others (remotes)

```
agit clone https://...         # Clone (HTTP/HTTPS)
agit clone git@github.com:...  # Clone (SSH, via system ssh)
agit fetch                     # See what updates the remote has
agit pull                      # Pull down and auto-merge
agit push origin main          # Push up
agit remote add upstream ...   # Add a remote
agit remote list               # List remotes
```

## 7. Merging code

```
agit merge feat/xxx            # Merge the branch in
      ↓ Fast-forwards when possible
      ↓ On conflict, writes conflict markers for you to resolve manually
```

## 8. Advanced operations

### Rebase
```
agit rebase main               # "Move" the current branch on top of main
agit rebase --onto main HEAD~3 # Move three commits over
agit rebase --continue         # Continue after resolving conflicts
agit rebase --skip             # Skip the current commit
agit rebase --abort            # Give up and roll back
```

### Cherry-pick
```
agit cherry-pick abc123        # Pick a commit over
agit cherry-pick abc123 def456 # Multiple at once
agit cherry-pick --continue    # Continue after resolving conflicts
agit cherry-pick --abort       # Give up
```

### Stash
```
agit stash                     # Stash your changes
agit stash pop                 # Pop them back
agit stash list                # List stashes
agit stash drop                # Drop one
```

### Reset
```
agit reset --soft HEAD~1       # Undo the commit, changes go back to staging
agit reset --mixed             # Default: back to staging
agit reset --hard HEAD~1       # Discard entirely (use with care)
```

### Bisect
```
agit bisect start --bad HEAD --good v0.1.0
agit bisect good               # This version is fine
agit bisect bad                # This version has the bug
agit bisect run "cargo test"   # Auto-run a script to locate it
```

### Blame
```
agit blame src/main.rs         # Who wrote each line
agit blame --revision HEAD~5 src/main.rs  # View a historical version
```

### Reflog
```
agit reflog                    # All changes to HEAD
agit reflog main               # Changes to the main branch
```

## 9. Tags

```
agit tag                       # List tags
agit tag -c v1.0.0             # Create a lightweight tag
agit tag -c v1.0.0 -m "发版"   # Create an annotated tag
agit tag -d v1.0.0             # Delete a tag
```

## 10. File operations

```
agit rm file.txt               # Delete a file (also removes from staging)
agit rm --cached file.txt      # Remove from staging only, keep the file
agit mv old.txt new.txt        # Rename/move
```

## 11. Inspect internal objects

```
agit cat-file -p abc123        # View object content
agit cat-file -t abc123        # View object type
agit ls-tree abc123            # See what's in a tree
```

## 12. Configuration

```
agit config user.name "Me"     # Set username
agit config --global user.email "me@x.com"  # Global config
agit config --list             # Show all config
agit config --get user.name    # Get one entry
agit config --unset user.name  # Remove one entry
```

**Config precedence**: environment variables > repo `.agit/config.toml` > global `~/.agitconfig.toml` > defaults

Command aliases are also supported:
```toml
# ~/.agitconfig.toml
[alias]
co = "commit"
st = "status"
```

## 13. Output formats

```
agit log --json                # JSON output (for scripts/AI)
agit status --yaml             # YAML output
agit --no-color log            # No color (for piping)
```

## 14. AI mode

```
agit --ai commit -m "fix"      # Enable AI mode
```

In AI mode, automatically:
- Prefix commit messages with `[AI-committed]`
- Block dangerous commands: `push`, `stash drop`, `branch -D`, `rebase`, `cherry-pick`, `bisect`

---

## One diagram: from opening a terminal to pushing to GitHub

```
agit init
  → agit add .
    → agit commit -m "init"
      → (edit code)
        → agit add .
          → agit commit --ai "新功能"
            → agit push origin main
```

## What's not done yet (so users can see what's missing at a glance)

| Feature | Status | Notes |
|------|------|------|
| Interactive rebase (`-i`) | ❌ Not supported | Requires a TUI, conflicts with the "never hang" design philosophy |
| Interactive add (`-p`) | ❌ Not supported | Same as above |
| submodule | ❌ Not supported | |
| hooks | ❌ Not supported | pre-commit / post-commit, etc. |
| grep | ❌ Not supported | |
| Git protocol v2 | ❌ Not supported | Currently uses the v1 protocol |
| Windows installer (.msi) | ❌ To do | |
| musl static build | ⏳ Pending fix | CI lacks musl-gcc |

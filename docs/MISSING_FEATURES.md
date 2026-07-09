# agit Missing Features Analysis

[中文文档](MISSING_FEATURES-zh_CN.md)

> Version: 0.5.4 | Date: 2026-06-13 | Branch: feat/missing-features-doc

This document analyzes agit's feature gaps relative to standard Git, prioritized as P0 (blocks basic workflow) → P3 (nice to have). Each entry includes a problem description and implementation suggestions, but no concrete code implementation.

---

## 🔴 P0 — Blocks basic workflow (unusable without these features) ✅ Completed (v0.5.4)

### 1. `reset` / unstage ✅

**Problem:** The `status` output suggests `"use git restore --staged <file>..."`, but the project has no `reset` or `restore` command at all. After running `add`, users cannot unstage files. The `core::index::Index::remove_entries()` API is implemented but no command calls it.

**Implementation:** `commands/reset.rs` — support `reset HEAD <file>` (unstage), `reset --soft/--mixed/--hard [<commit>]` (move HEAD), `HEAD~N` parent commit traversal. `core::checkout` exposes the `restore_from_commit()` and `rebuild_index_from_commit()` public APIs.
> Commit: `7233db6`

### 2. checkout lacks working tree safety checks ✅

**Problem:** `commands/checkout.rs:23` directly calls `checkout::switch_branch()`, **without checking whether the working tree has uncommitted changes**. Git checkout checks and refuses by default (unless `-f/--force`), to prevent users from losing unsaved modifications.

**Implementation:** `commands/checkout.rs` — before switching branches, check whether tracked files are modified/deleted; refuse and prompt if not clean. Add `-f`/`--force` to force the switch.
> Commit: `0094649`

### 3. `config` command ✅

**Problem:** Users cannot set `user.name` / `user.email` via the command line. They can only manually edit `.agit/config.toml` or `~/.agitconfig.toml`. This is the first step of onboarding for new users and one of the most frequently used Git commands.

**Implementation:** `commands/config_cmd.rs` — support `config <key>` (get), `config <key> <val>` (set), `--list`, `--unset`, `--global`. Directly read/write TOML files, supporting `section.key` nested keys.
> Commit: `80d9b5e`

### 4. Incomplete merge conflict resolution flow ✅

**Problem:** After a merge produces conflicts, it writes `<<<<<<<` marker files and `MERGE_HEAD` / `MERGE_MSG`, but:
- `commit` does not detect the presence of `MERGE_HEAD` → cannot complete the merge commit.
- No `merge --abort` → cannot roll back the merge state.
- No `merge --continue` → cannot resume an aborted merge.

**Implementation:** `commands/commit.rs` — auto-detect MERGE_HEAD and create a two-parent merge commit, use MERGE_MSG as the default message, and clean up merge state files after committing. `commands/merge.rs` — `--abort` restores ORIG_HEAD + cleans up state files, `--continue` delegates to commit, and ORIG_HEAD is automatically saved when the merge starts.
> Commit: `be91be1`

---

## 🟠 P1 — Severely hampers daily use

### 5. `.gitignore` support

**Problem:** The untracked file lists of `status`, `add`, and `diff` will include `target/`, `node_modules/`, `.DS_Store`, etc. For any project with build artifacts, the output is extremely noisy, and `add .` will mistakenly add a large amount of junk files.

**Impact:** `status` and `add .` are almost unusable in real projects.

**Implementation suggestions:**
- Add a `core::ignore` module, implementing an `IgnoreMatcher` struct.
- Parse rules: read `.gitignore` (support `*`, `**`, `?`, `[abc]`, `!` negation, `#` comments, `/` directory markers).
- Inherit the cascading search of `.gitignore` (from the current directory up level by level to the repo root).
- Filter ignored files in `status::collect_untracked()`, `add` path expansion, and `diff` untracked.
- `.git/info/exclude` file support can be done later.

### 6. `stash` — temporarily save the working tree

**Problem:** Cannot temporarily save working tree changes. `stash` / `stash pop` are high-frequency operations for Git users, used to switch branches or pull updates when work is unfinished.

**Impact:** When the working tree has uncommitted changes, no operation requiring a clean working tree (checkout, pull, merge, etc.) can be performed at all.

**Implementation suggestions:**
- `stash push`: ① generate tree objects from the diff between index and working tree; ② create a stash commit (structure: a merge commit with 2-3 parents — HEAD commit, index state, untracked files); ③ update `refs/stash`; ④ reset the working tree to HEAD.
- `stash pop`: apply the changes from `refs/stash` back to the working tree, and delete that stash on success.
- `stash list`: traverse the reflog of `refs/stash` or the linear parent chain to list all stashes.
- `stash drop`: delete a specified stash.

### 7. `tag` CLI command

**Problem:** `core::objects::tag.rs` (annotated tag model), and `create_tag`/`list_tags`/`delete_tag` in `core::refs` are all implemented behind a feature gate. But the CLI layer is completely missing a `tag` subcommand, so users cannot create or view tags.

**Impact:** Cannot mark release versions, and tag-dependent features like `git describe` cannot be implemented either.

**Implementation suggestions:**
- Add `Tag { action: TagAction }` to the `Commands` enum in `cli/mod.rs`, with sub-actions `list` / `create {name, message}` / `delete {name}`.
- Add `commands/tag.rs`, calling the existing core API.
- `tag create` can take `-a` (annotated), `-m` (message), `-s` (signed stub).

### 8. `diff` limitations

**Problem:** Currently `diff` can only do HEAD vs index + untracked files. It does not support:
- `diff <commit1> <commit2>` — compare any two commits.
- `diff <branch1>..<branch2>` — compare between branches.
- `diff --cached` — compare HEAD and the index (view what is about to be committed).
- `diff --name-only` — list file names only.

**Impact:** `diff` is almost useless in code review and branch comparison scenarios.

**Implementation suggestions:**
- Modify `diff run()` to accept two optional object parameters (SHA/branch name/tag name), resolve them to tree SHAs, then compare the two trees.
- `--cached`: compare the HEAD tree and the blobs in the index.
- `--name-only`: output only file names, not diff content.
- Default parameter values: no args = compare index and working tree (current behavior); one arg = compare the given commit and the working tree.

### 9. `log` is rudimentary

**Problem:** Currently `log` only walks the first-parent chain, with no filtering or formatting options:
- No `--oneline` — concise single-line format.
- No `--graph` — ASCII branch graph.
- No `--all` — show history of all branches.
- No `-n <N>` — limit the number of entries.
- No `--since` / `--until` — time range.
- No `--author` — filter by author.
- Only follows first-parent, so the other line of a merge is invisible.

**Impact:** Viewing history is very inconvenient; slightly complex branch histories are completely invisible.

**Implementation suggestions:**
- Prioritize implementing `-n N` (limit traversal steps) and `--oneline` (`<short_hash> <first_line_of_message>`).
- `--all`: read `refs/heads/*` and `refs/tags/*`, and BFS/DFS traverse from each starting point simultaneously.
- `--graph`: draw ASCII vertical lines and branches with simple column offsets.
- `--author`: parse the commit author field for substring matching.
- `--since`/`--until`: parse the timestamp field for filtering.

### 10. `rm` / `mv` — delete and move tracked files

**Problem:** Cannot delete or rename files from version control. Users can only operate on the file system manually, but index synchronization requires directly editing the binary index.

**Impact:** Refactoring code (moving files, deleting obsolete files) cannot be tracked with version control.

**Implementation suggestions:**
- `rm <file>`: ① remove the entry from the index (`Index::remove_entries()`), ② delete the working tree file (by default), ③ `--cached` only removes the index entry.
- `mv <old> <new>`: ① look up the old path in the index and change it to the new path, ② move/rename the working tree file.

---

## 🟡 P2 — Clearly missing but does not block basic operations

### 11. `rebase`

**Problem:** Cannot rebase the current branch's commits onto another branch. This is a common operation for maintaining linear history.

**Implementation suggestions:**
- Simple version `rebase <target_branch>`: ① compute the commits unique to the current branch (merge-base → HEAD); ② cherry-pick them onto target_branch one by one; ③ move HEAD to the HEAD of target_branch.
- Interactive rebase (`-i`) requires editor interaction, which violates the "non-blocking" design principle; it can be done later or via the `GIT_SEQUENCE_EDITOR` pattern.
- On conflict, write `REBASE_HEAD` state and support `rebase --abort` / `rebase --continue`.

### 12. `cherry-pick`

**Problem:** Cannot apply a single commit to the current branch.

**Implementation suggestions:**
- Read the tree diff between the target commit and its parent → apply the diff to the current working tree → create a new commit (message reused, author preserved).
- Support `-n` (no-commit, only apply to working tree + index).

### 13. `revert`

**Problem:** Cannot undo an existing commit.

**Implementation suggestions:**
- Similar to cherry-pick but applies the diff in reverse. Create a new commit with message `Revert "<original subject>"`.
- Share the diff engine with cherry-pick.

### 14. `blame` / `annotate`

**Problem:** Cannot trace the last modifier of code line by line.

**Implementation suggestions:**
- For each commit of the specified file, do a diff (relative to its parent) and trace the origin line by line.
- Plain text output format: `<short_hash> (<author> <date> <line_no>) <content>`.

### 15. `clean`

**Problem:** Cannot clean up untracked files in one shot.

**Implementation suggestions:**
- `-n` dry-run (default), `-f` force delete, `-d` include directories.
- Work with the `.gitignore` module; `-x` also deletes ignored files.

### 16. HTTP authentication / credentials

**Problem:** `HttpTransport` has no authentication mechanism. All private repositories are completely inaccessible.

**Implementation suggestions:**
- Minimum implementation: `AGIT_TOKEN` / `GIT_TOKEN` environment variables → HTTP `Authorization: Bearer <token>` header.
- Extract `user:token@host` from the URL → HTTP Basic Auth.
- More complete: `~/.agitcredentials` file, `credential.helper` configuration.

### 17. SSH protocol support

**Problem:** Currently only HTTP(S) transport is implemented. SSH is the mainstream protocol for private repositories and self-hosted Git servers.

**Implementation suggestions:**
- Parse `git@host:path` format URLs.
- Method 1: implement SSH connection and `git-upload-pack`/`git-receive-pack` subprocesses directly via the `ssh2` crate.
- Method 2: invoke the system `ssh` command to establish a pipe (simpler, better compatibility).

### 18. `--version` flag

**Problem:** The `Cli` struct does not set a clap version, so running `agit --version` outputs no version number.

**Implementation suggestions:**
- Add `#[command(version = env!("CARGO_PKG_VERSION"))]` on `#[derive(Parser)]`.

---

## 🟢 P3 — Nice to have

### 19. Shallow clone `--depth`

**Problem:** clone/fetch always fetches the full history, which is very slow for large repositories.

**Implementation suggestions:** At the protocol layer, add the `--depth` parameter to the `want` line (`want <sha> depth=<n>`). The server returns truncated history, and the local side creates a "shallow" marker file (`.git/shallow`).

### 20. `bisect` — binary search for the bug introduction point

Need to maintain `BISECT_LOG`, `BISECT_GOOD`/`BISECT_BAD` state files. `bisect start` → `bisect good <commit>` / `bisect bad <commit>` → automatically checkout the middle commit → after the user tests, `bisect good/bad` → loop until the first bad commit is found → `bisect reset` to restore.

### 21. `grep` — search in working tree/tree

Search for a specified pattern (supporting `-i` ignore case, `-n` line numbers, `-r` recursive, `--name-only`), runnable on the working tree, index, or a specified tree.

### 22. Hooks — lifecycle hooks

Execute `pre-commit`, `post-commit`, `pre-push`, `post-checkout`, etc. scripts. At appropriate moments in commands like `commit`, `push`, `checkout`, detect and run the `.git/hooks/<name>` file.

### 23. `submodule` — submodule management

Parse the `.gitmodules` configuration, recursively clone/update submodule repositories. `submodule add`, `submodule update --init --recursive`, `submodule status`.

### 24. `worktree` — multiple working directories

Check out multiple branches in parallel to different directories. Manage the worktree links under `.git/worktrees/`.

### 25. `reflog` — reference change log

Append a record line to `.git/logs/refs/heads/<name>` each time a branch is updated. `reflog show` views historical reference values. Used to recover from mistaken `reset --hard`, mistaken `commit --amend`, etc.

### 26. `archive` — package and export

Export a file snapshot without the `.git` directory (tar/zip). `archive -o <file> <tree-ish>`.

### 27. `describe` — tag-based version description

`git describe --tags`: find the nearest annotated tag and output a readable version number in the `<tag>-<N>-g<short_hash>` format.

### 28. Packfile generation optimization (Delta encoding)

**Problem:** `generate_pack()` of `push` zlib-compresses each object independently, without using delta encoding. Network transfer efficiency for large files or many similar files is extremely low.

**Implementation suggestions:** Implement the `git diff-delta` algorithm: generate binary delta patches between two blobs, and select pairs with high similarity for ofs_delta/ref_delta encoding.

### 29. Index multi-stage support (for merging)

**Problem:** The current Index uses `BTreeMap<String, IndexEntry>`, with only one entry per path. Git's index DIRC v2 format allows up to 4 stages per path (0=normal, 1=base, 2=ours, 3=theirs), used to mark conflicts during three-way merge.

**Implementation suggestions:** Extend `IndexEntry` to add a `stage` field (0-3), and change `Index::entries` to `BTreeMap<(String, u8), IndexEntry>`.

### 30. Diff algorithm enhancement

The current LCS-based line-level comparison is inefficient for large files and binary files. Git's patience/histogram algorithms usually produce more readable diffs on code files. The `similar` crate can be introduced, or patience diff can be implemented manually.

### 31. File mode detection

**Problem:** `add` hardcodes `100644` (regular file) and does not detect symbolic links or executable bits.

**Implementation suggestions:** In `add`, determine via `fs::symlink_metadata` and permission bits: regular file `100644`, executable `100755`, symbolic link `120000` (store the link target path as blob content).

### 32. Configuration option expansion

**Problem:** The `Config` struct only has three fields: `user_name`, `user_email`, `aliases`.

**Implementation suggestions:** Add common configuration options: `core.editor`, `remote.origin.url`, `remote.origin.fetch`, `credential.helper`, `init.defaultBranch`, `merge.tool`.


---

## 🔧 Architecture improvement suggestions (not feature gaps)

| # | Problem | Suggestion |
|---|------|------|
| A | `with_header()`/`with_object_header()` are duplicated across `checkout.rs`, `merge.rs`, `diff.rs`, etc. | Extract to `core::objects` as a public function `pub fn format_object_data(type, content) -> Vec<u8>` (already exists but not used everywhere) |
| B | `collect_tree_paths()` / `collect_untracked()` are duplicated across `checkout.rs`, `merge.rs`, `status.rs`, `diff.rs`, `pull.rs` | Extract to `core::tree_utils` or `core::repo` |
| C | `pull.rs` and `merge.rs` each implement their own common ancestor search algorithm, with slightly different logic | Unify on `core::merge::find_merge_base()` |
| D | Index `remove_entries()` API is implemented but no command calls it | Used by the `reset` and `rm` commands |
| E | `status` suggests "use git restore --staged" but the project is named agit | The prompt text should be changed to "agit reset HEAD <file>" |
| F | Windows paths use `replace('\\', '/')` as a temporary normalization | A unified path handling layer should be established (`utils::normalize_path` or `core::repo::normalize_path`)|

---

## Priority summary

```
Round 1 — Minimum Viable Product (P0):
  ┌─ 1. reset / unstage
  ├─ 2. checkout safety check
  ├─ 3. config command
  └─ 4. merge conflict resolution flow completion

Round 2 — Daily frictionless (P1):
  ├─ 5. .gitignore
  ├─ 6. stash
  ├─ 7. tag CLI
  ├─ 8. diff compare any commits
  ├─ 9. log --oneline / -n
  └─ 10. rm / mv

Round 3 — Team collaboration (P2):
  ├─ 11. rebase (simple version)
  ├─ 12. cherry-pick
  ├─ 13. revert
  ├─ 14. blame
  ├─ 15. clean
  ├─ 16. HTTP authentication
  ├─ 17. SSH protocol
  └─ 18. --version

Round 4 — Completeness improvements (P3):
  └─ 19-32. Implement as needed
```

#!/usr/bin/env bash
# ============================================================
# agit 仿真用户烟雾测试 (Linux)
# 模拟真实用户操作链，验证 agit 在日常工作流中的行为。
# 任何 scenario 失败 → exit 1
# ============================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
AGIT="${AGIT_BIN:-./target/release/agit}"

# 编译（如果二进制不存在）
if [ ! -x "$AGIT" ]; then
    echo -e "${YELLOW}==> Building agit...${NC}"
    cargo build --release 2>&1 || { echo -e "${RED}FATAL: build failed${NC}"; exit 1; }
    AGIT="./target/release/agit"
fi

# 转为绝对路径（后续场景会 cd 到临时目录，相对路径会失效）
if [ "${AGIT#/}" = "$AGIT" ]; then
    AGIT="$PWD/$AGIT"
fi

echo -e "${YELLOW}==> agit smoke tests (Linux)${NC}"
echo "    binary: $AGIT"
echo "    version: $($AGIT --version)"
echo ""

# ── 工具函数 ───────────────────────────────────────────────

run_agit() {
    "$AGIT" "$@" 2>&1
}

assert_contains() {
    local desc="$1" output="$2" pattern="$3"
    if echo "$output" | grep -qF "$pattern"; then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc"
        echo "    expected to contain: '$pattern'"
        echo "    actual output:"
        echo "$output" | sed 's/^/      /'
        FAIL=$((FAIL + 1))
    fi
}

assert_file_exists() {
    local desc="$1" path="$2"
    if [ -f "$path" ]; then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc — file not found: $path"
        FAIL=$((FAIL + 1))
    fi
}

assert_file_contains() {
    local desc="$1" path="$2" pattern="$3"
    if [ -f "$path" ] && grep -qF "$pattern" "$path"; then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc"
        echo "    file: $path"
        echo "    expected to contain: '$pattern'"
        FAIL=$((FAIL + 1))
    fi
}

assert_eq() {
    local desc="$1" expected="$2" actual="$3"
    if [ "$expected" = "$actual" ]; then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc"
        echo "    expected: '$expected'"
        echo "    actual:   '$actual'"
        FAIL=$((FAIL + 1))
    fi
}

# ── Scenario A: 新人第一天 ─────────────────────────────────

scenario_new_project() {
    echo -e "\n${YELLOW}── Scenario A: 新人第一天 ──${NC}"

    local tmp=$(mktemp -d)
    trap "rm -rf $tmp" RETURN
    cd "$tmp"

    # init
    local out=$(run_agit init)
    assert_contains "init creates repo" "$out" "Initialized empty Git repository"

    # config
    run_agit config user.name "Alice Dev" > /dev/null
    run_agit config user.email "alice@example.com" > /dev/null
    local name=$(run_agit config user.name | tail -1)
    local email=$(run_agit config user.email | tail -1)
    assert_eq "config user.name" "Alice Dev" "$name"
    assert_eq "config user.email" "alice@example.com" "$email"

    # 创建项目文件
    mkdir -p src
    echo 'fn main() { println!("hello"); }' > src/main.rs
    echo 'target/' > .gitignore
    echo "# My Project" > README.md

    # add
    out=$(run_agit add .)
    assert_contains "add all files" "$out" "Added"
    assert_file_contains ".gitignore created" ".gitignore" "target/"

    # commit
    out=$(run_agit commit -m "feat: initial commit")
    assert_contains "commit succeeds" "$out" "(root-commit)"

    # 从 ref 文件读完整 commit hash
    local commit_hash=$(cat .git/refs/heads/main)

    # status (clean)
    out=$(run_agit status)
    assert_contains "status clean" "$out" "nothing to commit"

    # log
    out=$(run_agit log --oneline)
    assert_contains "log shows commit" "$out" "initial commit"

    # cat-file
    out=$(run_agit cat-file -p "$commit_hash")
    assert_contains "cat-file shows tree" "$out" "tree"

    # branch
    out=$(run_agit branch --list)
    assert_contains "branch list shows main" "$out" "* main"

    cd - > /dev/null
    rm -rf "$tmp"
}

# ── Scenario B: 修 bug 提 PR ───────────────────────────────

scenario_fix_bug() {
    echo -e "\n${YELLOW}── Scenario B: 修 bug 日常流程 ──${NC}"

    local tmp=$(mktemp -d)
    trap "rm -rf $tmp" RETURN
    cd "$tmp"

    run_agit init > /dev/null
    run_agit config user.name "Bob Fixer" > /dev/null
    run_agit config user.email "bob@example.com" > /dev/null

    # 初始提交
    echo "v1" > app.txt
    run_agit add app.txt > /dev/null
    run_agit commit -m "feat: base" > /dev/null

    # 创建 fix 分支
    local out=$(run_agit branch -c fix/bug-42)
    assert_contains "create branch" "$out" "Created branch"
    run_agit checkout fix/bug-42 > /dev/null

    # 修改文件
    echo "v2-fixed" > app.txt
    echo "new-feature" > feat.txt

    # diff 查看
    out=$(run_agit diff)
    assert_contains "diff shows app.txt" "$out" "app.txt"
    assert_contains "diff shows feat.txt" "$out" "feat.txt"

    # 暂存 + 提交
    run_agit add app.txt feat.txt > /dev/null
    out=$(run_agit commit -m "fix: resolve bug #42")
    assert_contains "commit on branch" "$out" "resolve bug #42"

    # 确认文件内容
    assert_eq "app.txt content" "v2-fixed" "$(cat app.txt)"
    assert_file_exists "feat.txt exists" "feat.txt"

    # 切回 main 并 merge
    run_agit checkout main > /dev/null
    out=$(run_agit merge fix/bug-42)
    # merge 可能 fast-forward 或产生 merge commit
    if echo "$out" | grep -qF "Fast-forward"; then
        assert_contains "merge ff" "$out" "Fast-forward"
    else
        assert_contains "merge ok" "$out" "Merge"
    fi

    # main 上应该有 fix 的内容
    assert_eq "main has fix" "v2-fixed" "$(cat app.txt)"
    assert_file_exists "main has feat.txt" "feat.txt"

    cd - > /dev/null
    rm -rf "$tmp"
}

# ── Scenario C: stash 救场 ─────────────────────────────────

scenario_stash_rescue() {
    echo -e "\n${YELLOW}── Scenario C: stash 误操作救回 ──${NC}"

    local tmp=$(mktemp -d)
    trap "rm -rf $tmp" RETURN
    cd "$tmp"

    run_agit init > /dev/null
    run_agit config user.name "Dev" > /dev/null
    run_agit config user.email "dev@test" > /dev/null

    echo "v1" > work.txt
    run_agit add work.txt > /dev/null
    run_agit commit -m "init" > /dev/null

    # 修改未保存
    echo "v2-wip" > work.txt

    # stash 保存
    out=$(run_agit stash push)
    assert_contains "stash push" "$out" "Saved working directory"

    # 工作区回到 HEAD 状态
    assert_eq "work.txt reverted" "v1" "$(cat work.txt)"

    # stash list
    out=$(run_agit stash list)
    assert_contains "stash list not empty" "$out" "stash@{0}"

    # stash pop 恢复
    out=$(run_agit stash pop)
    assert_contains "stash pop" "$out" "Dropped refs/stash"

    # 恢复后文件内容正确
    assert_eq "work.txt restored" "v2-wip" "$(cat work.txt)"

    cd - > /dev/null
    rm -rf "$tmp"
}

# ── Scenario D: 仓库一致性（对象完整性）───────────────────

scenario_repo_integrity() {
    echo -e "\n${YELLOW}── Scenario D: 对象完整性验证 ──${NC}"

    local tmp=$(mktemp -d)
    trap "rm -rf $tmp" RETURN
    cd "$tmp"

    run_agit init > /dev/null
    run_agit config user.name "Tester" > /dev/null
    run_agit config user.email "t@t" > /dev/null

    # 多次提交
    for i in 1 2 3; do
        echo "line $i" >> "data.txt"
        run_agit add data.txt > /dev/null
        run_agit commit -m "commit $i" > /dev/null
    done

    # log 验证
    out=$(run_agit log --oneline)
    assert_contains "log has commit 3" "$out" "commit 3"
    assert_contains "log has commit 1" "$out" "commit 1"

    # ls-tree 验证 HEAD
    out=$(run_agit ls-tree HEAD)
    assert_contains "ls-tree shows data.txt" "$out" "data.txt"

    # cat-file 验证 blob 可读
    local blob_sha=$(echo "$out" | grep data.txt | awk '{print $3}')
    out=$(run_agit cat-file -p "$blob_sha")
    assert_contains "cat-file blob content" "$out" "line 3"

    # 多个 commit 后 status 干净
    out=$(run_agit status)
    assert_contains "status clean after 3 commits" "$out" "nothing to commit"

    cd - > /dev/null
    rm -rf "$tmp"
}

# ── Scenario E: rm / mv 文件管理 ───────────────────────────

scenario_file_management() {
    echo -e "\n${YELLOW}── Scenario E: 文件删除/重命名 ──${NC}"

    local tmp=$(mktemp -d)
    trap "rm -rf $tmp" RETURN
    cd "$tmp"

    run_agit init > /dev/null
    run_agit config user.name "Dev" > /dev/null
    run_agit config user.email "dev@test" > /dev/null

    echo "keep-me" > stay.txt
    echo "delete-me" > gone.txt
    run_agit add stay.txt gone.txt > /dev/null
    run_agit commit -m "add files" > /dev/null

    # rm 删除文件
    out=$(run_agit rm gone.txt)
    assert_contains "rm output" "$out" "rm 'gone.txt'"
    if [ -f gone.txt ]; then
        echo -e "  ${RED}FAIL${NC} gone.txt should be deleted"
        FAIL=$((FAIL + 1))
    else
        echo -e "  ${GREEN}PASS${NC} gone.txt deleted from disk"
        PASS=$((PASS + 1))
    fi

    # mv 重命名
    out=$(run_agit mv stay.txt renamed.txt)
    assert_contains "mv output" "$out" "Renamed 'stay.txt' -> 'renamed.txt'"
    if [ -f stay.txt ]; then
        echo -e "  ${RED}FAIL${NC} stay.txt should be moved"
        FAIL=$((FAIL + 1))
    else
        echo -e "  ${GREEN}PASS${NC} stay.txt moved"
        PASS=$((PASS + 1))
    fi
    assert_file_exists "renamed.txt exists" "renamed.txt"

    cd - > /dev/null
    rm -rf "$tmp"
}

# ── 运行所有场景 ────────────────────────────────────────────

scenario_new_project
scenario_fix_bug
scenario_stash_rescue
scenario_repo_integrity
scenario_file_management

# ── 汇总 ────────────────────────────────────────────────────

echo ""
echo "============================================="
if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}All smoke tests passed: $PASS assertions${NC}"
    exit 0
else
    echo -e "${RED}FAILURES: $FAIL / $((PASS + FAIL))${NC}"
    exit 1
fi

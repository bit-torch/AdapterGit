# ============================================================
# agit 仿真用户烟雾测试 (Windows PowerShell)
# 模拟真实用户操作链，验证 agit 在日常工作流中的行为。
# 任何 scenario 失败 → exit 1
# ============================================================

$ErrorActionPreference = "Stop"

$PASS = 0
$FAIL = 0
$AGIT = if ($env:AGIT_BIN) { $env:AGIT_BIN } else { ".\target\release\agit.exe" }

# 编译（如果二进制不存在）
if (-not (Test-Path $AGIT)) {
    Write-Host "==> Building agit..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FATAL: build failed" -ForegroundColor Red
        exit 1
    }
    $AGIT = ".\target\release\agit.exe"
}

Write-Host "==> agit smoke tests (Windows)" -ForegroundColor Yellow
Write-Host "    binary: $AGIT"
$ver = & $AGIT --version 2>&1
Write-Host "    version: $ver"

# ── 工具函数 ───────────────────────────────────────────────

function Run-Agit {
    $out = & $AGIT @args 2>&1 | Out-String
    return $out.TrimEnd()
}

function Assert-Contains {
    param($Desc, $Output, $Pattern)
    if ($Output -match [regex]::Escape($Pattern)) {
        Write-Host "  PASS $Desc" -ForegroundColor Green
        $script:PASS++
    } else {
        Write-Host "  FAIL $Desc" -ForegroundColor Red
        Write-Host "    expected to contain: '$Pattern'"
        Write-Host "    actual output:"
        Write-Host "      $Output"
        $script:FAIL++
    }
}

function Assert-FileExists {
    param($Desc, $Path)
    if (Test-Path $Path -PathType Leaf) {
        Write-Host "  PASS $Desc" -ForegroundColor Green
        $script:PASS++
    } else {
        Write-Host "  FAIL $Desc — file not found: $Path" -ForegroundColor Red
        $script:FAIL++
    }
}

function Assert-FileContains {
    param($Desc, $Path, $Pattern)
    if ((Test-Path $Path -PathType Leaf) -and ((Get-Content $Path -Raw) -match [regex]::Escape($Pattern))) {
        Write-Host "  PASS $Desc" -ForegroundColor Green
        $script:PASS++
    } else {
        Write-Host "  FAIL $Desc" -ForegroundColor Red
        Write-Host "    file: $Path"
        Write-Host "    expected to contain: '$Pattern'"
        $script:FAIL++
    }
}

function Assert-Eq {
    param($Desc, $Expected, $Actual)
    if ($Expected -eq $Actual) {
        Write-Host "  PASS $Desc" -ForegroundColor Green
        $script:PASS++
    } else {
        Write-Host "  FAIL $Desc" -ForegroundColor Red
        Write-Host "    expected: '$Expected'"
        Write-Host "    actual:   '$Actual'"
        $script:FAIL++
    }
}

# ── Scenario A: 新人第一天 ─────────────────────────────────

function Scenario-NewProject {
    Write-Host ""
    Write-Host "── Scenario A: 新人第一天 ──" -ForegroundColor Yellow

    $tmp = Join-Path $env:TEMP "agit_smoke_A_$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    Push-Location $tmp
    try {
        $out = Run-Agit init
        Assert-Contains "init creates repo" $out "Initialized empty Git repository"

        Run-Agit config user.name "Alice Dev" | Out-Null
        Run-Agit config user.email "alice@example.com" | Out-Null
        $name = Run-Agit config user.name
        $email = Run-Agit config user.email
        Assert-Eq "config user.name" "Alice Dev" ($name -split "`n")[-1].Trim()
        Assert-Eq "config user.email" "alice@example.com" ($email -split "`n")[-1].Trim()

        New-Item -ItemType Directory -Force -Path src | Out-Null
        'fn main() { println!("hello"); }' | Out-File -Encoding utf8 src/main.rs
        'target/' | Out-File -Encoding utf8 .gitignore
        '# My Project' | Out-File -Encoding utf8 README.md

        $out = Run-Agit add .
        Assert-Contains "add all files" $out "Added"
        Assert-FileContains ".gitignore created" ".gitignore" "target/"

        $out = Run-Agit commit -m "feat: initial commit"
        Assert-Contains "commit succeeds" $out "Created commit"

        if ($out -match '([a-f0-9]{40})') {
            $commitHash = $Matches[1]
        } else {
            $commitHash = ""
        }

        $out = Run-Agit status
        Assert-Contains "status clean" $out "nothing to commit"

        $out = Run-Agit log --oneline
        Assert-Contains "log shows commit" $out "initial commit"

        $out = Run-Agit cat-file -p $commitHash
        Assert-Contains "cat-file shows tree" $out "tree"

        $out = Run-Agit branch --list
        Assert-Contains "branch list shows main" $out "* main"
    } finally {
        Pop-Location
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# ── Scenario B: 修 bug 提 PR ───────────────────────────────

function Scenario-FixBug {
    Write-Host ""
    Write-Host "── Scenario B: 修 bug 日常流程 ──" -ForegroundColor Yellow

    $tmp = Join-Path $env:TEMP "agit_smoke_B_$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    Push-Location $tmp
    try {
        Run-Agit init | Out-Null
        Run-Agit config user.name "Bob Fixer" | Out-Null
        Run-Agit config user.email "bob@example.com" | Out-Null

        "v1" | Out-File -Encoding utf8 app.txt
        Run-Agit add app.txt | Out-Null
        Run-Agit commit -m "feat: base" | Out-Null

        $out = Run-Agit branch -c fix/bug-42
        Assert-Contains "create branch" $out "Created branch"
        Run-Agit checkout fix/bug-42 | Out-Null

        "v2-fixed" | Out-File -Encoding utf8 app.txt
        "new-feature" | Out-File -Encoding utf8 feat.txt

        $out = Run-Agit diff
        Assert-Contains "diff shows app.txt" $out "app.txt"
        Assert-Contains "diff shows feat.txt" $out "feat.txt"

        Run-Agit add app.txt feat.txt | Out-Null
        $out = Run-Agit commit -m "fix: resolve bug #42"
        Assert-Contains "commit on branch" $out "Created commit"

        Assert-Eq "app.txt content" "v2-fixed" (Get-Content app.txt -Raw).Trim()
        Assert-FileExists "feat.txt exists" "feat.txt"

        Run-Agit checkout main | Out-Null
        $out = Run-Agit merge fix/bug-42
        if ($out -match "Fast-forward") {
            Assert-Contains "merge ff" $out "Fast-forward"
        } else {
            Assert-Contains "merge ok" $out "Merge"
        }

        Assert-Eq "main has fix" "v2-fixed" (Get-Content app.txt -Raw).Trim()
        Assert-FileExists "main has feat.txt" "feat.txt"
    } finally {
        Pop-Location
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# ── Scenario C: stash 救场 ─────────────────────────────────

function Scenario-StashRescue {
    Write-Host ""
    Write-Host "── Scenario C: stash 误操作救回 ──" -ForegroundColor Yellow

    $tmp = Join-Path $env:TEMP "agit_smoke_C_$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    Push-Location $tmp
    try {
        Run-Agit init | Out-Null
        Run-Agit config user.name "Dev" | Out-Null
        Run-Agit config user.email "dev@test" | Out-Null

        "v1" | Out-File -Encoding utf8 work.txt
        Run-Agit add work.txt | Out-Null
        Run-Agit commit -m "init" | Out-Null

        "v2-wip" | Out-File -Encoding utf8 work.txt

        $out = Run-Agit stash push
        Assert-Contains "stash push" $out "Saved working directory"

        Assert-Eq "work.txt reverted" "v1" (Get-Content work.txt -Raw).Trim()

        $out = Run-Agit stash list
        Assert-Contains "stash list not empty" $out "stash@{0}"

        $out = Run-Agit stash pop
        Assert-Contains "stash pop" $out "Dropped refs/stash"

        Assert-Eq "work.txt restored" "v2-wip" (Get-Content work.txt -Raw).Trim()
    } finally {
        Pop-Location
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# ── Scenario D: 仓库一致性 ─────────────────────────────────

function Scenario-RepoIntegrity {
    Write-Host ""
    Write-Host "── Scenario D: 对象完整性验证 ──" -ForegroundColor Yellow

    $tmp = Join-Path $env:TEMP "agit_smoke_D_$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    Push-Location $tmp
    try {
        Run-Agit init | Out-Null
        Run-Agit config user.name "Tester" | Out-Null
        Run-Agit config user.email "t@t" | Out-Null

        for ($i = 1; $i -le 3; $i++) {
            "line $i" | Out-File -Encoding utf8 -Append data.txt
            Run-Agit add data.txt | Out-Null
            Run-Agit commit -m "commit $i" | Out-Null
        }

        $out = Run-Agit log --oneline
        Assert-Contains "log has commit 3" $out "commit 3"
        Assert-Contains "log has commit 1" $out "commit 1"

        $out = Run-Agit ls-tree HEAD
        Assert-Contains "ls-tree shows data.txt" $out "data.txt"

        if ($out -match 'data.txt.*?\s([a-f0-9]{40})') {
            $blobSha = $Matches[1]
        } else {
            $blobSha = ""
        }
        $out = Run-Agit cat-file -p $blobSha
        Assert-Contains "cat-file blob content" $out "line 3"

        $out = Run-Agit status
        Assert-Contains "status clean after 3 commits" $out "nothing to commit"
    } finally {
        Pop-Location
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# ── Scenario E: rm / mv ────────────────────────────────────

function Scenario-FileManagement {
    Write-Host ""
    Write-Host "── Scenario E: 文件删除/重命名 ──" -ForegroundColor Yellow

    $tmp = Join-Path $env:TEMP "agit_smoke_E_$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    Push-Location $tmp
    try {
        Run-Agit init | Out-Null
        Run-Agit config user.name "Dev" | Out-Null
        Run-Agit config user.email "dev@test" | Out-Null

        "keep-me" | Out-File -Encoding utf8 stay.txt
        "delete-me" | Out-File -Encoding utf8 gone.txt
        Run-Agit add stay.txt gone.txt | Out-Null
        Run-Agit commit -m "add files" | Out-Null

        $out = Run-Agit rm gone.txt
        Assert-Contains "rm output" $out "rm 'gone.txt'"
        if (Test-Path gone.txt) {
            Write-Host "  FAIL gone.txt should be deleted" -ForegroundColor Red
            $script:FAIL++
        } else {
            Write-Host "  PASS gone.txt deleted from disk" -ForegroundColor Green
            $script:PASS++
        }

        $out = Run-Agit mv stay.txt renamed.txt
        Assert-Contains "mv output" $out "Renamed 'stay.txt' -> 'renamed.txt'"
        if (Test-Path stay.txt) {
            Write-Host "  FAIL stay.txt should be moved" -ForegroundColor Red
            $script:FAIL++
        } else {
            Write-Host "  PASS stay.txt moved" -ForegroundColor Green
            $script:PASS++
        }
        Assert-FileExists "renamed.txt exists" "renamed.txt"
    } finally {
        Pop-Location
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

# ── 运行所有场景 ────────────────────────────────────────────

Scenario-NewProject
Scenario-FixBug
Scenario-StashRescue
Scenario-RepoIntegrity
Scenario-FileManagement

# ── 汇总 ────────────────────────────────────────────────────

Write-Host ""
Write-Host "============================================="
if ($FAIL -eq 0) {
    Write-Host "All smoke tests passed: $PASS assertions" -ForegroundColor Green
    exit 0
} else {
    Write-Host "FAILURES: $FAIL / $($PASS + $FAIL)" -ForegroundColor Red
    exit 1
}

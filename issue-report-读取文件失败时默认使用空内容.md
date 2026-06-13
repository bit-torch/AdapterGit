# Issue1: 读取文件失败时默认使用空内容

## 严重程度

中

## 位置

- 文件: `src/commands/checkout.rs`
- 行号: L49-50

## 问题描述

在 `is_working_tree_clean` 函数中，当通过 `fs::read()` 读取工作区文件内容时，使用了 `unwrap_or_default()` 处理错误：

```rust
let content = fs::read(&full_path).unwrap_or_default();
```

当文件读取失败（例如权限错误、IO 错误等）时，`unwrap_or_default()` 会返回空的 `Vec<u8>`，而非传播错误。这会导致以下问题：

1. **误判文件已被修改**：空内容计算出的 blob hash 几乎不可能与 index 中记录的 `entry.sha1` 匹配，因此该文件会被错误地判定为"已修改"，`is_working_tree_clean` 返回 `false`，阻止正常的分支切换。
2. **错误被静默吞掉**：用户无法得知文件读取失败的真正原因（如权限不足），只会看到"本地变更会被覆盖"的通用错误信息，增加了排查难度。

## 复现场景

1. 在 Git 仓库中创建一个被 index 跟踪的文件。
2. 修改该文件的权限，使当前用户无法读取（例如在 Linux 上执行 `chmod 000 file.txt`）。
3. 尝试切换分支（不带 `--force`）。
4. 预期行为：应返回明确的权限错误信息。
5. 实际行为：返回"本地变更会被覆盖"的误导性错误。

## 建议修复

使用 `?` 运算符传播错误，让调用方感知并处理读取失败：

```rust
let content = fs::read(&full_path)?;
```

由于 `is_working_tree_clean` 的返回类型已经是 `Result<..., Box<dyn std::error::Error>>`，`?` 运算符可以直接使用，无需额外修改函数签名。

## 影响范围

- `checkout` 命令的分支切换安全检查逻辑
- 任何依赖 `is_working_tree_clean` 判断工作区状态的流程

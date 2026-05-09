# Checklist: Phase 0 & 1

## Phase 0 验证

- [x] Cargo.toml 包含所有必要依赖（clap, serde, serde_json, flate2, sha1, anyhow）
- [x] `cargo build` 编译成功
- [x] `cargo check` 无严重警告（仅 dead_code，模块尚未接入 CLI）
- [x] src/ 目录结构完整，每个子目录有 mod.rs
- [x] `agit --help` 输出帮助信息
- [x] `agit --ai` 全局参数可用
- [x] `agit --json` 全局参数可用
- [x] `agit init` 路由到对应占位处理函数
- [x] 统一错误类型 AgitError 已定义

## Phase 1 - SHA-1 验证

- [x] `hash_bytes` 对已知输入输出正确 SHA-1
- [x] `hash_git_object` 格式正确（`{type} {len}\0{content}`）
- [x] 与 `git hash-object` 交叉验证通过

## Phase 1 - zlib 验证

- [x] 压缩后能完整解压还原
- [x] 压缩数据可被原生 git zlib 解压
- [x] 能解压原生 git 生成的 zlib 对象

## Phase 1 - Blob 验证

- [x] Blob 序列化格式：`blob {len}\0{content}`
- [x] Blob SHA-1 与 `git hash-object` 一致
- [x] 反序列化还原原始内容

## Phase 1 - Tree 验证

- [x] Tree 条目格式正确：`{mode} {name}\0{sha1_20bytes}`
- [x] Tree SHA-1 与 `git mktree` 一致
- [x] 能解析原生 git 生成的 tree 对象

## Phase 1 - Commit 验证

- [x] Commit 格式正确（tree / parent / author / committer / message）
- [x] Commit SHA-1 与原生 git 一致
- [x] 能解析原生 git 生成的 commit 对象

## Phase 1 - 对象存储验证

- [x] 写入后文件位于 `.git/objects/{xx}/{xxxx...}`
- [x] 文件内容为 zlib 压缩格式
- [x] 读取还原的对象类型和内容正确
- [x] 读取不存在的对象返回错误

## Phase 1 - 引用系统验证

- [x] HEAD 解析正确（symbolic 和 detached 两种模式）
- [x] 分支创建和读取正确
- [x] 标签读取正确
- [x] 写入后原生 git 可以读取

## Phase 1 - 索引验证

- [x] 空索引有正确签名和版本号
- [x] 索引条目序列化/反序列化回合正确
- [x] agit 创建的索引能被原生 git 读取

## 代码质量验证

- [x] `cargo test` 全部通过 (40 passed)
- [x] `cargo clippy` 无严重警告 (仅 dead_code)
- [x] `cargo fmt --check` 通过
- [x] 公共 API 有文档注释

## 文档验证

- [x] docs/PHASE_0_1_TODO.md 已创建
- [x] TODO.md 状态完整准确

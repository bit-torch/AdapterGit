# ── 多阶段构建：Debian slim 运行时 ──
# 注：Alpine/musl 暂不可用，需 Linux 测试环境
FROM rust:1.89-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# 全功能构建（含 TLS + AI）
RUN cargo build --release --all-features

# ── 运行时镜像 ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git openssh-client && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/agit /usr/local/bin/agit

ENTRYPOINT ["/usr/local/bin/agit"]
CMD ["--help"]

# ── 多阶段构建：musl 静态编译 → alpine 最小镜像 ──
FROM rust:1.89-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

# 全功能构建（含 TLS + AI）
RUN cargo build --release --all-features

# ── 运行时镜像 ──
FROM alpine:3.21

RUN apk add --no-cache ca-certificates git openssh-client

COPY --from=builder /app/target/release/agit /usr/local/bin/agit

ENTRYPOINT ["/usr/local/bin/agit"]
CMD ["--help"]

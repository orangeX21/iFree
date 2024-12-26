# 构建阶段
FROM rust:1.83.0-slim-bookworm as builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY . .

# 构建
RUN rustup target add x86_64-unknown-linux-gnu && \
    cargo build --target x86_64-unknown-linux-gnu --release && \
    cp target/x86_64-unknown-linux-gnu/release/cursor-api /app/cursor-api

# 运行阶段
FROM debian:bookworm-slim

WORKDIR /app

ENV TZ=Asia/Shanghai

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# 复制构建产物
COPY --from=builder /app/cursor-api .

# 设置默认端口
ENV PORT=3000

# 动态暴露端口
EXPOSE ${PORT}

CMD ["./cursor-api"]
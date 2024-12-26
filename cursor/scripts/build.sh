#!/bin/bash
set -euo pipefail

# 颜色输出函数
info() { echo -e "\033[1;34m[INFO]\033[0m $*"; }
warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
error() { echo -e "\033[1;31m[ERROR]\033[0m $*" >&2; exit 1; }

# 检查是否在 Linux 环境
is_linux() {
    [ "$(uname -s)" = "Linux" ]
}

# 检查必要的工具
check_requirements() {
    local missing_tools=()

    # 基础工具检查
    for tool in cargo protoc npm node; do
        if ! command -v "$tool" &>/dev/null; then
            missing_tools+=("$tool")
        fi
    done

    # Linux 特定检查
    if [[ $USE_CROSS == true ]] && ! command -v cross &>/dev/null; then
        missing_tools+=("cross")
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        error "缺少必要工具: ${missing_tools[*]}"
    fi
}

# 帮助信息
show_help() {
    cat << EOF
用法: $(basename "$0") [选项]

选项:
  --cross         使用 cross 进行交叉编译（仅在 Linux 上有效）
  --static        使用静态链接（默认动态链接）
  --help          显示此帮助信息

不带参数时只编译当前平台
EOF
}

# 并行构建函数
build_target() {
    local target=$1
    local extension=""
    local rustflags="${2:-}"

    info "正在构建 $target..."

    # 确定文件后缀
    [[ $target == *"windows"* ]] && extension=".exe"

    # 设置目标特定的环境变量
    local build_env=()
    if [[ $target == "aarch64-unknown-linux-gnu" ]]; then
        build_env+=(
            "CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc"
            "CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++"
            "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc"
            "PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig"
            "PKG_CONFIG_ALLOW_CROSS=1"
            "OPENSSL_DIR=/usr"
            "OPENSSL_INCLUDE_DIR=/usr/include"
            "OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu"
        )
    fi

    # 判断是否使用 cross（仅在 Linux 上）
    if [[ $target != "$CURRENT_TARGET" ]]; then
        env ${build_env[@]+"${build_env[@]}"} RUSTFLAGS="$rustflags" cargo build --target "$target" --release
    else
        env ${build_env[@]+"${build_env[@]}"} RUSTFLAGS="$rustflags" cargo build --release
    fi

    # 移动编译产物到 release 目录
    local binary_name="cursor-api"
    [[ $USE_STATIC == true ]] && binary_name+="-static"

    if [[ -f "target/$target/release/cursor-api$extension" ]]; then
        cp "target/$target/release/cursor-api$extension" "release/${binary_name}-$target$extension"
        info "完成构建 $target"
    else
        warn "构建产物未找到: $target"
        return 1
    fi
}

# 获取 CPU 架构
ARCH=$(uname -m | sed 's/^aarch64\|arm64$/aarch64/;s/^x86_64\|x86-64\|x64\|amd64$/x86_64/')
OS=$(uname -s)

# 确定当前系统的目标平台
get_target() {
    local arch=$1
    local os=$2
    case "$os" in
        "Darwin") echo "${arch}-apple-darwin" ;;
        "Linux") echo "${arch}-unknown-linux-gnu" ;;
        "MINGW"*|"MSYS"*|"CYGWIN"*|"Windows_NT") echo "${arch}-pc-windows-msvc" ;;
        "FreeBSD") echo "x86_64-unknown-freebsd" ;;
        *) error "不支持的系统: $os" ;;
    esac
}

# 设置当前目标平台
CURRENT_TARGET=$(get_target "$ARCH" "$OS")

# 检查是否成功获取目标平台
[ -z "$CURRENT_TARGET" ] && error "无法确定当前系统的目标平台"

# 获取系统对应的所有目标
get_targets() {
    case "$1" in
        "linux") echo "x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu" ;;
        "windows") echo "x86_64-pc-windows-msvc aarch64-pc-windows-msvc" ;;
        "macos") echo "x86_64-apple-darwin aarch64-apple-darwin" ;;
        "freebsd") echo "x86_64-unknown-freebsd" ;;
        *) error "不支持的系统组: $1" ;;
    esac
}

# 解析参数
USE_CROSS=false
USE_STATIC=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --cross) USE_CROSS=true ;;
        --static) USE_STATIC=true ;;
        --help) show_help; exit 0 ;;
        *) error "未知参数: $1" ;;
    esac
    shift
done

# 检查依赖
check_requirements

# 确定要构建的目标
if [[ $USE_CROSS == true ]] && is_linux; then
    # 只在 Linux 上使用 cross 进行多架构构建
    TARGETS=($(get_targets "linux"))
else
    # 其他系统或不使用 cross 时只构建当前系统的所有架构
    case "$OS" in
        "Darwin") TARGETS=($(get_targets "macos")) ;;
        "Linux") TARGETS=("$CURRENT_TARGET") ;;
        "MINGW"*|"MSYS"*|"CYGWIN"*|"Windows_NT") TARGETS=($(get_targets "windows")) ;;
        "FreeBSD") TARGETS=("$CURRENT_TARGET") ;;
        *) error "不支持的系统: $OS" ;;
    esac
fi

# 创建 release 目录
mkdir -p release

# 设置静态链接标志
RUSTFLAGS=""
[[ $USE_STATIC == true ]] && RUSTFLAGS="-C target-feature=+crt-static"

# 并行构建所有目标
info "开始构建..."
for target in "${TARGETS[@]}"; do
    build_target "$target" "$RUSTFLAGS" &
done

# 等待所有构建完成
wait

# 为 macOS 平台创建通用二进制
if [[ "$OS" == "Darwin" ]] && [[ ${#TARGETS[@]} -gt 1 ]]; then
    binary_suffix=""
    [[ $USE_STATIC == true ]] && binary_suffix="-static"

    if [[ -f "release/cursor-api${binary_suffix}-x86_64-apple-darwin" ]] && \
       [[ -f "release/cursor-api${binary_suffix}-aarch64-apple-darwin" ]]; then
        info "创建 macOS 通用二进制..."
        lipo -create \
            "release/cursor-api${binary_suffix}-x86_64-apple-darwin" \
            "release/cursor-api${binary_suffix}-aarch64-apple-darwin" \
            -output "release/cursor-api${binary_suffix}-universal-apple-darwin"
    fi
fi

info "构建完成！"
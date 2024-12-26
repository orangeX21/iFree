# 设置 PowerShell 语言为 UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

# 检查是否以管理员权限运行
if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Warning "请以管理员权限运行此脚本"
    exit 1
}

# 检查并安装 Chocolatey
if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Output "正在安装 Chocolatey..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
}

# 安装必要的工具
Write-Output "正在安装必要的工具..."
choco install -y mingw
choco install -y protoc
choco install -y git

# 安装 Rust 工具
Write-Output "正在安装 Rust 工具..."
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu
cargo install cross

Write-Output "安装完成！"
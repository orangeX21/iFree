# Cursor Token 获取工具

这个工具用于从 Cursor 编辑器的本地数据库中获取访问令牌。

## 系统要求

- Rust 编程环境
- Cargo 包管理器

## 构建说明

### Windows

1. 安装 Rust
   ```powershell
   winget install Rustlang.Rust
   # 或访问 https://rustup.rs/ 下载安装程序
   ```

2. 克隆项目并构建
   ```powershell
   git clone <repository-url>
   cd get-token
   cargo build --release
   ```

3. 构建完成后，可执行文件位于 `target/release/get-token.exe`

### macOS

1. 安装 Rust
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. 克隆项目并构建
   ```bash
   git clone <repository-url>
   cd get-token
   cargo build --release
   ```

3. 构建完成后，可执行文件位于 `target/release/get-token`

## 使用方法

直接运行编译好的可执行文件即可：

- Windows: `.\target\release\get-token.exe`
- macOS: `./target/release/get-token`

程序将自动查找并显示 Cursor 编辑器的访问令牌。

## 注意事项

- 确保 Cursor 编辑器已经安装并且至少登录过一次
- Windows 数据库路径：`%USERPROFILE%\AppData\Roaming\Cursor\User\globalStorage\state.vscdb`
- macOS 数据库路径：`~/Library/Application Support/Cursor/User/globalStorage/state.vscdb`
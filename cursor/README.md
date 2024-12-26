# cursor-api

## 获取key

1. 访问 [www.cursor.com](https://www.cursor.com) 并完成注册登录（赠送 250 次快速响应，可通过删除账号再注册重置）
2. 在浏览器中打开开发者工具（F12）
3. 找到 应用-Cookies 中名为 `WorkosCursorSessionToken` 的值并保存(相当于 openai 的密钥)

## 接口说明

### 基础对话（请求格式和响应格式参考 openai）

- 接口地址：`/v1/chat/completions`
- 请求方法：POST
- 认证方式：Bearer Token（支持两种认证方式）
  1. 使用环境变量 `AUTH_TOKEN` 进行认证
  2. 使用 `.token` 文件中的令牌列表进行轮询认证

### 获取模型列表

- 接口地址：`/v1/models`
- 请求方法：GET

### 获取环境变量中的x-cursor-checksum

- 接口地址：`/env-checksum`
- 请求方法：GET

### 获取随机x-cursor-checksum

- 接口地址：`/checksum`
- 请求方法：GET

### 健康检查接口

- 接口地址：`/`
- 请求方法：GET

### 获取日志接口

- 接口地址：`/logs`
- 请求方法：GET

### Token管理接口

- 获取Token信息页面：`/tokeninfo`
- 更新Token信息：`/update-tokeninfo`
- 获取Token信息：`/get-tokeninfo`

## 配置说明

### 环境变量

- `PORT`: 服务器端口号（默认：3000）
- `AUTH_TOKEN`: 认证令牌（必须，用于API认证）
- `ROUTE_PREFIX`: 路由前缀（可选）
- `TOKEN_FILE`: token文件路径（默认：.token）
- `TOKEN_LIST_FILE`: token列表文件路径（默认：.token-list）

### Token文件格式

1. `.token` 文件：每行一个token，支持以下格式：
   ```
   token1
   alias::token2
   ```

2. `.token-list` 文件：每行为token和checksum的对应关系：
   ```
   token1,checksum1
   token2,checksum2
   ```

## 部署

### 本地部署

#### 从源码编译

需要安装 Rust 工具链和 protobuf 编译器：

```bash
# 安装依赖（Debian/Ubuntu）
apt-get install -y build-essential protobuf-compiler

# 编译并运行
cargo build --release
./target/release/cursor-api
```

#### 使用预编译二进制

从 [Releases](https://github.com/wisdgod/cursor-api/releases) 下载对应平台的二进制文件。

### Docker 部署

#### Docker 运行示例

```bash
docker run -d -e PORT=3000 -e AUTH_TOKEN=your_token -p 3000:3000 wisdgod/cursor-api:latest
```

#### Docker 构建示例

```bash
docker build -t cursor-api .
docker run -p 3000:3000 cursor-api
```

### huggingface部署

1. duplicate项目:
   [huggingface链接](https://huggingface.co/login?next=%2Fspaces%2Fstevenrk%2Fcursor%3Fduplicate%3Dtrue)

2. 配置环境变量

   在你的space中，点击settings，找到`Variables and secrets`，添加Variables
   - name: `AUTH_TOKEN` （注意大写）
   - value: 你随意

3. 重新部署
   
   点击`Factory rebuild`，等待部署完成

4. 接口地址（`Embed this Space`中查看）：
   ```
   https://{username}-{space-name}.hf.space/v1/models
   ```

## 注意事项

1. 请妥善保管您的 AuthToken，不要泄露给他人
2. 配置 AUTH_TOKEN 环境变量以增加安全性
3. 本项目仅供学习研究使用，请遵守 Cursor 的使用条款

## 开发

### 跨平台编译

使用提供的构建脚本：

```bash
# 仅编译当前平台
./scripts/build.sh

# 交叉编译所有支持的平台
./scripts/build.sh --cross
```

支持的目标平台：
- x86_64-unknown-linux-gnu
- x86_64-pc-windows-msvc
- aarch64-unknown-linux-gnu
- x86_64-apple-darwin
- aarch64-apple-darwin

### 获取token

- 使用 [get-token](https://github.com/wisdgod/cursor-api/tree/main/get-token) 获取读取当前设备token，仅支持windows与macos

## 鸣谢

- [cursor-api](https://github.com/wisdgod/cursor-api)
- [zhx47/cursor-api](https://github.com/zhx47/cursor-api)
- [luolazyandlazy/cursorToApi](https://github.com/luolazyandlazy/cursorToApi)

## 许可证

版权所有 (c) 2024

本软件仅供学习和研究使用。未经授权，不得用于商业用途。
保留所有权利。
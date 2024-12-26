# 颜色输出函数
function Write-Info { Write-Host "[INFO] $args" -ForegroundColor Blue }
function Write-Warn { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Write-Error { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

# 检查必要的工具
function Test-Requirements {
  $tools = @("cargo", "protoc", "npm", "node")
  $missing = @()

  foreach ($tool in $tools) {
    if (!(Get-Command $tool -ErrorAction SilentlyContinue)) {
      $missing += $tool
    }
  }

  if ($missing.Count -gt 0) {
    Write-Error "缺少必要工具: $($missing -join ', ')"
  }
}

# 在 Test-Requirements 函数后添加新函数
function Initialize-VSEnvironment {
    Write-Info "正在初始化 Visual Studio 环境..."

    # 直接使用已知的 vcvarsall.bat 路径
    $vcvarsallPath = "E:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"

    if (-not (Test-Path $vcvarsallPath)) {
        Write-Error "未找到 vcvarsall.bat: $vcvarsallPath"
        return
    }

    Write-Info "使用 vcvarsall.bat 路径: $vcvarsallPath"

    # 获取环境变量
    $archArg = "x64"
    $command = "`"$vcvarsallPath`" $archArg && set"

    try {
        $output = cmd /c "$command" 2>&1

        # 检查命令是否成功执行
        if ($LASTEXITCODE -ne 0) {
            Write-Error "vcvarsall.bat 执行失败，退出码: $LASTEXITCODE"
            return
        }

        # 更新当前 PowerShell 会话的环境变量
        foreach ($line in $output) {
            if ($line -match "^([^=]+)=(.*)$") {
                $name = $matches[1]
                $value = $matches[2]
                if (![string]::IsNullOrEmpty($name)) {
                    Set-Item -Path "env:$name" -Value $value -ErrorAction SilentlyContinue
                }
            }
        }

        Write-Info "Visual Studio 环境初始化完成"
    }
    catch {
        Write-Error "初始化 Visual Studio 环境时发生错误: $_"
    }
}

# 帮助信息
function Show-Help {
  Write-Host @"
用法: $(Split-Path $MyInvocation.MyCommand.Path -Leaf) [选项]

选项:
  --static        使用静态链接（默认动态链接）
  --help          显示此帮助信息

默认编译所有 Windows 支持的架构 (x64 和 arm64)
"@
}

# 构建函数
function New-Target {
  param (
    [string]$target,
    [string]$rustflags
  )

  Write-Info "正在构建 $target..."

  # 设置环境变量并执行构建
  $env:RUSTFLAGS = $rustflags
  cargo build --target $target --release

  # 移动构建产物
  $binaryName = "cursor-api"
  if ($UseStatic) {
    $binaryName += "-static"
  }

  $sourcePath = "target/$target/release/cursor-api.exe"
  $targetPath = "release/${binaryName}-${target}.exe"

  if (Test-Path $sourcePath) {
    Copy-Item $sourcePath $targetPath -Force
    Write-Info "完成构建 $target"
  }
  else {
    Write-Warn "构建产物未找到: $target"
    return $false
  }
  return $true
}

# 参数解析
$UseStatic = $false

foreach ($arg in $args) {
  switch ($arg) {
    "--static" { $UseStatic = $true }
    "--help" { Show-Help; exit 0 }
    default { Write-Error "未知参数: $arg" }
  }
}

# 主程序
try {
  # 检查依赖
  Test-Requirements

  # 初始化 Visual Studio 环境
  Initialize-VSEnvironment

  # 创建 release 目录
  New-Item -ItemType Directory -Force -Path "release" | Out-Null

  # 设置目标平台
  $targets = @(
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc"
  )

  # 设置静态链接标志
  $rustflags = ""
  if ($UseStatic) {
    $rustflags = "-C target-feature=+crt-static"
  }

  Write-Info "开始构建..."

  # 构建所有目标
  foreach ($target in $targets) {
    New-Target -target $target -rustflags $rustflags
  }

  Write-Info "构建完成！"
}
catch {
  Write-Error "构建过程中发生错误: $_"
}
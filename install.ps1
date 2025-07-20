# RuPing 安装脚本
# 需要以管理员身份运行

param(
    [string]$InstallPath = "$env:ProgramFiles\RuPing",
    [switch]$Uninstall
)

# 检查管理员权限
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Error "此脚本需要管理员权限。请以管理员身份运行PowerShell。"
    exit 1
}

if ($Uninstall) {
    Write-Host "正在卸载 RuPing..." -ForegroundColor Yellow
    
    # 从PATH中移除
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    $newPath = $currentPath -replace [regex]::Escape(";$InstallPath"), ""
    $newPath = $newPath -replace [regex]::Escape("$InstallPath;"), ""
    $newPath = $newPath -replace [regex]::Escape("$InstallPath"), ""
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "Machine")
    
    # 删除安装目录
    if (Test-Path $InstallPath) {
        Remove-Item -Path $InstallPath -Recurse -Force
        Write-Host "已删除安装目录: $InstallPath" -ForegroundColor Green
    }
    
    Write-Host "RuPing 已成功卸载。" -ForegroundColor Green
    Write-Host "请重新启动命令提示符以使PATH更改生效。" -ForegroundColor Yellow
    exit 0
}

Write-Host "正在安装 RuPing..." -ForegroundColor Green

# 检查是否存在构建的可执行文件
$exePath = "target\release\ruping.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "未找到构建的可执行文件。正在构建..." -ForegroundColor Yellow
    
    # 检查Rust是否已安装
    try {
        $rustVersion = cargo --version
        Write-Host "检测到 Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Error "未检测到 Rust。请先安装 Rust: https://rustup.rs/"
        exit 1
    }
    
    # 构建项目
    Write-Host "正在构建 RuPing (Release模式)..." -ForegroundColor Yellow
    cargo build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "构建失败。请检查错误信息。"
        exit 1
    }
    
    Write-Host "构建完成。" -ForegroundColor Green
}

# 创建安装目录
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    Write-Host "创建安装目录: $InstallPath" -ForegroundColor Green
}

# 复制可执行文件
Copy-Item -Path $exePath -Destination "$InstallPath\ruping.exe" -Force
Write-Host "已复制 ruping.exe 到 $InstallPath" -ForegroundColor Green

# 添加到系统PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
if ($currentPath -notlike "*$InstallPath*") {
    $newPath = "$currentPath;$InstallPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "Machine")
    Write-Host "已将 $InstallPath 添加到系统 PATH" -ForegroundColor Green
} else {
    Write-Host "$InstallPath 已在系统 PATH 中" -ForegroundColor Yellow
}

# 创建开始菜单快捷方式
$startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
$shortcutPath = "$startMenuPath\RuPing.lnk"

$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = "cmd.exe"
$Shortcut.Arguments = "/k echo RuPing - Rust Ping Implementation && echo 使用方法: ruping [选项] 目标地址 && echo 示例: ruping 8.8.8.8 && echo 帮助: ruping --help"
$Shortcut.WorkingDirectory = "$env:USERPROFILE"
$Shortcut.Description = "RuPing - Rust实现的Ping命令"
$Shortcut.Save()

Write-Host "已创建开始菜单快捷方式" -ForegroundColor Green

Write-Host ""
Write-Host "安装完成！" -ForegroundColor Green
Write-Host ""
Write-Host "使用方法:" -ForegroundColor Cyan
Write-Host "  ruping 8.8.8.8              # 基本ping" -ForegroundColor White
Write-Host "  ruping google.com -n 5      # 发送5个数据包" -ForegroundColor White
Write-Host "  ruping 192.168.1.1 -t       # 持续ping" -ForegroundColor White
Write-Host "  ruping --help                # 显示帮助" -ForegroundColor White
Write-Host ""
Write-Host "注意事项:" -ForegroundColor Yellow
Write-Host "- RuPing需要管理员权限才能运行" -ForegroundColor White
Write-Host "- 请重新启动命令提示符以使PATH更改生效" -ForegroundColor White
Write-Host "- 使用 'ruping --help' 查看所有可用选项" -ForegroundColor White
Write-Host ""
Write-Host "卸载:" -ForegroundColor Cyan
Write-Host "  .\install.ps1 -Uninstall     # 卸载RuPing" -ForegroundColor White

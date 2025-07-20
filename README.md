# RuPing

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Windows](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![Release](https://img.shields.io/github/v/release/username/ruping)](https://github.com/username/ruping/releases)

> 🦀 一个用Rust编写的高性能Windows ping命令完整重新实现

RuPing是一个用Rust编写的Windows ping命令的完整重新实现，提供与原生Windows ping命令相同的功能和参数支持。

## 📋 目录

- [特性](#-特性)
- [快速开始](#-快速开始)
- [安装](#-安装)
- [使用方法](#-使用方法)
- [支持的参数](#-支持的参数)
- [卸载](#-卸载)
- [开发](#-开发)
- [故障排除](#-故障排除)
- [贡献](#-贡献)
- [许可证](#-许可证)

## ✨ 特性

- 🔒 **安全**: 使用Rust的内存安全特性，避免缓冲区溢出等安全问题
- ⚡ **高效**: 基于Tokio异步运行时，支持高性能网络操作
- 🎯 **简洁**: 清晰的代码结构，易于理解和维护
- 🔧 **独立**: 完全独立的可执行文件，无需额外依赖
- 📋 **完整**: 支持Windows ping的所有参数和功能

## 🚀 快速开始

```cmd
# 下载并安装
.\ruping-installer.exe

# 基本使用
ruping 8.8.8.8
rustping google.com -n 5
rping 192.168.1.1 -t
```

## 📦 安装

### 方法1: 使用安装程序（推荐）

1. 从 [Releases](https://github.com/cyberlife7766/ruping-rping-rustping/releases) 下载最新的 `ruping-installer.exe`
2. 以管理员身份运行命令提示符
3. 运行安装程序：

```cmd
.\ruping-installer.exe --install-path "YOUR_INSTALL_PATH"
```

### 方法2: 从源码构建

```bash
# 克隆仓库
git clone https://github.com/cyberlife7766/ruping-rping-rustping.git
cd ruping-rping-rustping

# 构建
cargo build --release

# 可执行文件位于 target/release/ruping.exe
```

## 💻 使用方法

安装后，您可以使用以下任一命令：

```cmd
# 基本用法
ruping 8.8.8.8
rustping google.com -n 5
rping 192.168.1.1 -t

# 高级用法
ruping google.com -l 64 -w 2000 -4
rustping 8.8.8.8 -a -n 10
rping localhost -i 32
```

## 📖 支持的参数

与Windows ping完全兼容的所有参数：

```
用法: ruping <target> [选项]

参数:
  <target>              目标主机名或IP地址

选项:
  -t                    持续ping指定主机直到停止
  -a                    将地址解析为主机名
  -n <count>            要发送的回显请求数
  -l <size>             发送缓冲区大小
  -f                    在数据包中设置"不分段"标记(仅IPv4)
  -i <TTL>              生存时间
  -v <TOS>              服务类型(仅IPv4，已弃用)
  -r <count>            记录计数跃点的路由(仅IPv4)
  -s <count>            计数跃点的时间戳(仅IPv4)
  -j <host-list>        与主机列表一起使用的松散源路由(仅IPv4)
  -k <host-list>        与主机列表一起使用的严格源路由(仅IPv4)
  -w <timeout>          等待每次回复的超时时间(毫秒)
  -R                    同样使用路由标头测试反向路由(仅IPv6)
  -S <srcaddr>          要使用的源地址
  -c <compartment>      路由隔离舱标识符
  -p                    Ping Hyper-V网络虚拟化提供程序地址
  -4                    强制使用IPv4
  -6                    强制使用IPv6
  -h, --help            显示帮助信息
  -V, --version         显示版本信息
```

## 🗑️ 卸载

在安装后的程序根目录启动 `ruping-uninstaller.exe`，删除完成后手动删除安装程序即可。

```cmd
# 静默卸载
.\ruping-uninstaller.exe --silent
```


## 故障排除

如果遇到问题，请运行：

```cmd
# 启用调试模式
$env:RUST_LOG="debug"
ruping 8.8.8.8 -n 1
```

## 开发

### 从源码构建


```bash
# 克隆项目
git clone https://github.com/cyberlife7766/ruping-rping-rustping.git
cd ruping-rping-rustping

# 构建
cargo build --release

# 运行测试
cargo test

# 测试功能
.\test_all_features.cmd
```

## 贡献

欢迎提交Issue和Pull Request来改进RuPing！

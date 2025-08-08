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

- ⚡ **多目标并发**：同时 Ping 多个目标，支持并发上限配置（1-256）
- 📚 **批量输入**：位置参数多目标、`--file` 文件读入、`--cidr` 批量展开（IPv4）
- 🔁 **自动回退**：权限不足无法创建 RAW 套接字时，自动回退至 Windows ICMP API（IPv4）
- 🖥️ **经典输出**：逐条回显、每主机总结与总体汇总，贴近 Windows ping 风格
- 📊 **增强统计**：最短/最长/平均 + P50/P90/P99、Jitter、StdDev
- 🔒 **安全可靠**：Rust 内存安全 + Tokio 异步高性能

## 🚀 快速开始

```cmd
# 下载并安装
.\ruping-installer.exe

# 基本使用
ruping 8.8.8.8
ruping google.com -n 5
ruping 192.168.1.1 -t

# 多目标与批量
ruping 1.1.1.1 8.8.8.8 --concurrency 2 -n 2 --interval 500
ruping --file .\targets.txt -n 3
ruping --cidr 192.168.1.0/30,10.0.0.0/31 -n 1 --deadline 5
```

## 安装

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
ruping google.com -n 5
ruping 192.168.1.1 -t

# 高级用法
ruping google.com -l 64 -w 2000 -4
ruping 8.8.8.8 -a -n 10
ruping localhost -i 32

# 并发与批量
ruping 1.1.1.1 8.8.8.8 -P 64 --interval 1000 -n 4
ruping --file .\hosts.txt --concurrency 32 -n 2
ruping --cidr 10.0.0.0/30 -n 1 --deadline 10

# 指定源地址/网卡示例
ruping example.com --source 192.168.1.10
ruping example.com --iface "Ethernet" --ttl 64
ruping example.com -6 --iface 12 --ttl 32
```

## 📖 支持的参数

与Windows ping完全兼容的所有参数：

```
用法: ruping [TARGETS...] [选项]

参数:
  TARGETS               目标主机名或IP地址（支持多个）

选项（兼容 Windows ping）:
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
  --iface <name|index>  指定网卡/接口（网卡友好名/描述/适配器名，或 ifIndex 数值）
  -c <compartment>      路由隔离舱标识符
  -p                    Ping Hyper-V网络虚拟化提供程序地址
  -4                    强制使用IPv4
  -6                    强制使用IPv6
  -h, --help            显示帮助信息
  -V, --version         显示版本信息
```

### 扩展参数（RuPing 专有）

```
--file <PATH>           从文件读取目标（一行一个，支持注释#）
--cidr <CIDR>[,...]     通过 IPv4 CIDR 批量展开目标，支持逗号分隔与多次指定
-P, --concurrency <N>   限制并发主机数（1-256，默认64）
--interval <ms>         每主机发送间隔（毫秒，默认1000）
--deadline <sec>        全局截止时间（秒），到达后停止未完成任务
--json                  以 JSON 输出结果（抑制逐条回显与头部/总结）
--csv                   以 CSV 输出结果（抑制逐条回显与头部/总结）
--summary-only          仅打印每主机与总体总结（不打印逐条回显）
--quiet                 抑制逐条回显（仍打印头部与总结）
--include-replies       在 JSON/CSV 输出中包含每次回复明细
--output <path>         将 JSON/CSV 输出写入文件（默认输出到标准输出）
--pretty                JSON 美化（缩进/换行）。未指定时输出紧凑 JSON
--headers <mode>        CSV 表头模式：all|none（默认 all）
```

说明：当权限不足无法创建 RAW 套接字时，IPv4 将自动回退到 Windows ICMP API（无需管理员权限）。

## 📈 输出与统计

- 每主机：逐条回显、统计汇总（最短/最长/平均），并包含 P50/P90/P99、Jitter、StdDev。
- 总体：跨主机发送/接收/丢包与丢包率，同时提供 P50/P90/P99、Jitter、StdDev 聚合指标。

示例：

```cmd
# JSON 输出
ruping 1.1.1.1 8.8.8.8 -n 3 --json > result.json

# CSV 输出
ruping --file .\targets.txt -n 2 --csv > result.csv

# 仅打印总结
ruping 8.8.8.8 -n 5 --summary-only

# 静默逐条回显，但保留头部与总结
ruping 8.8.8.8 -n 5 --quiet
```

### 示例输出片段

#### 经典输出（节选）

```text
正在 Ping 8.8.8.8 具有 32 字节的数据:
来自 8.8.8.8 的回复: 字节=32 时间=23ms TTL=117
来自 8.8.8.8 的回复: 字节=32 时间=22ms TTL=117
请求超时。

8.8.8.8 的 Ping 统计信息:
    数据包: 已发送 = 3，已接收 = 2，丢失 = 1 (33% 丢失)
往返行程的估计时间(毫秒): 最短 = 22ms，最长 = 23ms，平均 = 22ms
    P50 = 22ms，P90 = 23ms，P99 = 23ms，Jitter = 1.0ms，StdDev = 0.5ms
```

#### JSON 输出（含 `--include-replies`，节选）

```json
{
  "hosts": [
    {
      "name": "8.8.8.8",
      "ip": "8.8.8.8",
      "stats": {
        "sent": 3,
        "received": 2,
        "lost": 1,
        "loss_pct": 33.33,
        "min": 22.000,
        "avg": 22.500,
        "max": 23.000,
        "p50": 22.000,
        "p90": 23.000,
        "p99": 23.000,
        "jitter": 1.000,
        "stddev": 0.500,
        "replies": [
          { "seq": 1, "time_ms": 23.000, "outcome": "ok" },
          { "seq": 2, "time_ms": 22.000, "outcome": "ok" },
          { "seq": 3, "time_ms": 0.000,  "outcome": "timeout" }
        ]
      }
    }
  ],
  "overall": {
    "sent": 3, "received": 2, "lost": 1,
    "loss_pct": 33.33,
    "min": 22.000, "avg": 22.500, "max": 23.000,
    "p50": 22.000, "p90": 23.000, "p99": 23.000,
    "jitter": 1.000, "stddev": 0.500
  }
}
```

#### CSV 输出（含 `--include-replies`，节选）

```csv
scope,name,ip,sent,received,lost,loss_pct,min,avg,max,p50,p90,p99,jitter,stddev
host,8.8.8.8,8.8.8.8,3,2,1,33.33,22.000,22.500,23.000,22.000,23.000,23.000,1.000,0.500
scope,name,ip,seq,time_ms,outcome
reply,8.8.8.8,8.8.8.8,1,23.000,ok
reply,8.8.8.8,8.8.8.8,2,22.000,ok
reply,8.8.8.8,8.8.8.8,3,0.000,timeout
overall,,,3,2,1,33.33,22.000,22.500,23.000,22.000,23.000,23.000,1.000,0.500
```

### 结构化输出字段说明（JSON/CSV）

#### JSON 顶层

- `schema`: 固定为 `"ruping-stats"`
- `version`: 整数版本号，当前为 `1`
- `hosts`: 主机数组，每个元素包含：
  - `name`: 目标名称（输入主机名或地址）
  - `ip`: 实际探测的 IP 地址
  - `stats`: 统计对象
    - `sent`/`received`/`lost`/`loss_pct`
    - `min`/`avg`/`max`
    - `p50`/`p90`/`p99`
    - `jitter`/`stddev`
    - `replies`（可选，启用 `--include-replies`）：数组，元素为
      - `seq`: 序号（从 1 递增）
      - `time_ms`: RTT 毫秒。成功为数值，超时/错误为 `null`
      - `outcome`: `ok` | `timeout` | `error`
- `overall`: 总体统计（字段与 `stats` 相同，但不包含 `replies`）

说明：当使用 `--pretty` 时，输出为易读格式；未指定时输出紧凑格式，便于机器处理与减小体积。

#### CSV 列说明

默认包含两类记录，首列 `scope` 指示类型：

1) `host` 行（每主机一行）：

```
scope,name,ip,sent,received,lost,loss_pct,min,avg,max,p50,p90,p99,jitter,stddev
```

2) `reply` 行（仅在 `--include-replies` 时输出，逐条回复一行）：

```
scope,name,ip,seq,time_ms,outcome
```

3) `overall` 行（总体一行）：

```
overall,,,<sent>,<received>,<lost>,<loss_pct>,<min>,<avg>,<max>,<p50>,<p90>,<p99>,<jitter>,<stddev>
```

备注：当回复为超时或错误时，`time_ms` 留空（空字符串），`outcome` 分别为 `timeout` 或 `error`。

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

## 📦 发布/打包

面向开发者的发布与打包说明：

1) Release 构建二进制

```powershell
# 可选：先激活虚拟环境（若存在）
if (Test-Path .\.venv\Scripts\Activate.ps1) { . .\.venv\Scripts\Activate.ps1 }

# 构建 release
cargo build --release

# 产物位置
# ruping\target\release\ruping.exe
```

2) 生成安装器（可选）

```powershell
# 在 ruping 目录执行 PowerShell 时请使用相对路径调用脚本
.\build_installer.cmd

# 或在 CMD 下执行
cmd /c build_installer.cmd

# 产物位置（已在 .gitignore 忽略）
# ruping\release\ruping-installer.exe
# ruping\release\ruping-uninstaller.exe
```

说明：
- 使用原始套接字需要以管理员身份运行程序或控制台。
- 指定网卡/接口：`--iface <名称|索引>` 支持网卡友好名/描述/适配器名或 ifIndex 数值；当同时指定 `--source` 与 `--iface` 时，以 `--source` 为准。
- IPv6 下 `--ttl` 设置为 Hop Limit。
```

## 贡献

欢迎提交Issue和Pull Request来改进RuPing！

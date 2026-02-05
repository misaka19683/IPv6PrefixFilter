# IPV6PrefixFilter
[README](README.md) | [中文文档](README_ZH.md)

本程序是一个适用于 Linux 的路由器通告（RA, Routher Advertisement）过滤器，可以过滤试图设置非*指定 IPv6 前缀*的 RA。有时，错误配置的路由器会发送错误的前缀设置 RA，或者有人故意发送错误的 RA 来干扰您的网络。

## 开发进度
当前程序已经初步可用。

待办列表

- [x] 通过 `nftables` 劫持 RA 到 Queue 中，并在程序中获取。
- [x] 分析通告内容，判断是否丢弃。
- [x] 完善的命令行
- [x] 集成的 NFTables 规则设置。
- [ ] 支持基于正则表达式的规则匹配
- [ ] 符合 Unix 哲学的程序行为

## 使用方式

### 先决条件

#### Linux 
本程序依赖 `nftables` 劫持 RA 数据包。请确保您的系统支持 `nftables`，并且安装有 `libnetfilter_queue`（以及对应的 `kmod`）。


### 命令行帮助

通过 `-h` 或 `--help` 参数获得帮助
```shell
# IPv6PrefixFilter --help
A simple IPv6 Router Advertisement prefix filter using nftables.

用法: IPv6PrefixFilter [选项]

选项:
  -p, --prefix <PREFIXES>     指定允许的 IPv6 前缀（默认）或要阻止的前缀（如果开启了 -b）
  -i, --interface <INTERFACE>  指定要过滤的网络接口（如 eth0）
  -b, --blacklist              开启黑名单模式：指定的前缀将被阻止（Drop）
  -c, --clear                  清除由程序设置的 nftables 规则并退出
      --no-nft                 禁用自动设置 nftables 规则
  -v, --verbose...             显示详细运行信息。-v 为 info 级别，-vv 为 debug 级别
  -h, --help                   显示帮助
  -V, --version                显示版本
```

### 例子

#### 1. 仅允许 eth0 接口上的特定前缀
```shell
IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64
```

#### 2. 阻止 eth0 接口上的特定前缀（黑名单模式）
```shell
IPv6PrefixFilter -i eth0 -b -p 2001:db8:bad::/48
```

#### 3. 允许 eth0 接口上的多个前缀
```shell
IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64 -p 2001:db8:2::/64
```

#### 3. 作为 Systemd 服务运行（推荐用于长期保护）
创建一个文件 `/etc/systemd/system/ra-filter.service`:

```ini
[Unit]
Description=IPv6 Router Advertisement 前缀过滤器
After=network.target

[Service]
Type=simple
# 请根据实际路径修改 ExecStart
ExecStart=/usr/local/bin/IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64
Restart=always
# 运行所需的权限
CapabilityBoundingSet=CAP_NET_ADMIN
AmbientCapabilities=CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```

然后启用并启动服务:
```shell
systemctl enable ra-filter
systemctl start ra-filter
```

## 编译指南

使用了 `cargo` 作为编译工具

获取动态编译版本

```shell
cargo build --release
```

获取静态编译版本

```shell
cargo build --release --target x86_64-unknown-linux-musl
```

请注意，你可能需要安装 `musl` 工具链来使用这个目标三元组。在某些系统上，你可以使用包管理器来安装它，例如在 Ubuntu 上：
```shell
sudo apt-get install musl-tools
```
你的cargo可能也需要配置正确的交叉编译。

```shell
rustup target add x86_64-unknown-linux-musl
```

# IPV6PrefixFilter
[README](README.md) | [中文文档](README_ZH.md)

本程序是一个适用于 Linux 和 Windows 的路由器通告（RA, Routher Advertisement）过滤器，可以过滤试图设置非*指定 IPv6 前缀*的 RA。有时，错误配置的路由器会发送错误的前缀设置 RA，或者有人故意发送错误的 RA 来干扰您的网络。

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

#### Windows
Windows 平台需要安装 WinDivert。具体请参考 [Windivert Install Guide](Windivert_Install_Guide_ZH.md)

### 命令行帮助

通过 `-h` 或 `--help` 参数获得帮助

```
# IPV6PrefixFilter --help
Use IPv6PrefixFilter [COMMAND] --help to see the detail help for each subcommand

Usage: IPv6PrefixFilter [OPTIONS] [COMMAND]

Commands:
  run      Run the program (in the foreground)
  clear    Clear the nft rules set by the program, especially when the program exits improperly without executing the cleanup process
  version  Run as a daemon process. Print version info
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Display detailed runtime information. The default log level is warning. Use -v to set to info, and -vv for debug
  -h, --help        Print help
  -V, --version     Print version
```

对于每条 Command，可以单独使用 `-h` 或 `--help`。
```
# IPV6PrefixFilter run --help
Run the program (in the foreground)

Usage: IPv6PrefixFilter run [OPTIONS]

Options:
  -p, --ipv6-prefixs <IPV6_PREFIXS>  Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option
  -i, --interface <INTERFACE>        Specify the wan interface
  -b, --blacklist-mode               Enable blacklist mode. Prefixes specified with `-p` will be blocked
      --disable-nft-autoset          Disable the feature of auto set nftables rules
  -h, --help                         Print help
```
### 例子

如果您想要拦截来自 wan 接口的 RA 通告，仅保留前缀 `FFFF:FFFF:FFFF::/48`，您可以使用如下命令
```
IPV6PrefixFilter run -i wan -p FFFF:FFFF:FFFF::/48
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

在 Windows 构建：

```shell
cargo build --release --target=x86_64-pc-windows-msvc
```
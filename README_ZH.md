# IPV6PrefixFilter
[README](README.md) | [中文文档](README_ZH.md)

**施工中，不可用**

该程序是一个过滤器，可以过滤试图设置非指定 IPv6 前缀的路由器通告（RA, Routher Advertisement）。有时，配置错误的路由器会为错误的前缀设置发送错误的 RA，或者有人故意发送错误的 RA 来干扰您的网络。

## TODO

- [x] 通过 NFTables 劫持 RA 到 Queue 中，并在程序中获取。
- [x] 分析通告内容，判断是否丢弃。
- [ ] 完善的命令行
- [x] 集成的 NFTables 规则设置。
- [ ] 支持基于正则表达式的规则匹配
- [ ] 符合 Unix 哲学的程序行为

## 使用方式(计划)

通过 `-h` 或 `--help` 参数获得帮助

```
# IPV6PrefixFilter --help
Some version information here.

Example: IPV6PrefixFilter command [options]

Commands:
    run:        Run the program (in the foreground).
    clear:      Clear nftables rules (especially when the program exits abnormally).
    daemon:     Run as a daemon process.
    version:    Print version information.
Options:
    -p, --prefix            Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option.
    -b, --blacklist-mode    Enable blacklist mode. Prefixes specified with `-p` will be blocked.
    -v, --verbose           Display detailed runtime information.
    -d, --daemon            Run the program in the background as a daemon.
    -h, --help              Display this help menu.
    --disable-nft-autoset   Disable the feature of auto set nftables rules.
```
## 如何编译

使用了`cargo`作为编译工具

获取动态编译版本

```shell
cargo build --release
```

获取静态编译版本

```shell
cargo build --release --target x86_64-unknown-linux-musl
```

请注意，你可能需要安装 musl 工具链来使用这个目标三元组。在某些系统上，你可以使用包管理器来安装它，例如在 Ubuntu 上：
```shell
sudo apt-get install musl-tools
```
你的cargo可能也需要配置正确的交叉编译。

```shell
rustup target add x86_64-unknown-linux-musl
```
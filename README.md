# IPV6PrefixFilter
[README](README.md) | [中文文档](README_ZH.md)

**Under Construction - Not Yet Functional**

TThis program is a Router Advertisement (RA) filter suitable for Linux, which will drop router advertisement packets that trying to set a *non-specified IPv6 prefix*. Sometimes a misconfigured router will send you the wrong RA for the wrong prefix setting, or someone will intentionally send you the wrong RA to attack your network.

## TODO

- [x] Intercept RAs using NFTables and redirect them to a queue for processing.
- [x] Analyze RA content and determine whether to discard them.
- [ ] Implement a comprehensive command-line interface.
- [x] Integrate NFTables rule management.
- [ ] Supports rule matching based on regular expressions
- [ ] Ensure compliance with the Unix philosophy.

## Planned Usage

### Prerequisites

This program relies on `nftables` to intercept RA packets. Please ensure that your system supports `nftables` and has `libnetfilter_queue` installed (along with the corresponding `kmod`).

### Command Line Help
Use the `-h` or `--help` option to view the help menu.

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
    -i, --interface         Specify the interface.
    -b, --blacklist-mode    Enable blacklist mode. Prefixes specified with `-p` will be blocked.
    -v, --verbose           Display detailed runtime information. The default log level is warning. Use -v to set to info, and -vv for debug.
    -h, --help              Display this help menu.
    --disable-nft-autoset   Disable the feature of auto set nftables rules.
```
## How to Build

We use `cargo` as the build tool.

To obtain a dynamically compiled version:

```shell
cargo build --release
```

To obtain a statically compiled version:

```shell
cargo build --release --target x86_64-unknown-linux-musl
```

Please note that you may need to install the musl toolchain to use this target triple. On some systems, you can use the package manager to install it, for example, on Ubuntu:

```shell
sudo apt-get install musl-tools
```
Your cargo may also need to be configured for proper cross compilation.

```shell
rustup target add x86_64-unknown-linux-musl
```
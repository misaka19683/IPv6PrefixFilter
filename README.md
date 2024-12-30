# IPV6PrefixFilter

[README](README.md) | [中文文档](README_ZH.md)

This program is a router advertisement (RA) filter compatible with Linux and Windows that filters RA attempting to set unspecified IPv6 prefixes. Occasionally, misconfigured routers may send RAs with incorrect prefix settings, or someone may intentionally send false RAs to disrupt your network.

## Development Progress
The program is currently in a preliminary usable state.

### To-Do List

- [x] Capture RA using `nftables` and redirect them to the queue, accessible by the program.
- [x] Analyze advertisement content and determine whether to discard it.
- [x] Implement a comprehensive command line interface.
- [x] Integrate NFTables rule setup.
- [ ] Support for regex-based rule matching.
- [ ] Conformity with Unix philosophy in program behavior.

## Usage

### Prerequisites

#### Linux
This program uses `nftables` to intercept RA packets. Ensure your system supports `nftables` and has `libnetfilter_queue` (along with the respective `kmod`) installed.

#### Windows
Windows platforms require the installation of WinDivert. Refer to the [WinDivert Install Guide](Windivert_Install_Guide.md) for details.

### Command-Line Help

Use the `-h` or `--help` parameter to get help:

```shell
# IPV6PrefixFilter --help
Use IPv6PrefixFilter [COMMAND] --help to see detailed help for each subcommand.

Usage: IPv6PrefixFilter [OPTIONS] [COMMAND]

Commands:
  run      Run the program (in the foreground)
  clear    Clear the nft rules set by the program, especially when the program exits improperly without executing the cleanup process
  version  Print version info
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Display detailed runtime information. The default log level is warning. Use -v to set to info, and -vv for debug
  -h, --help        Print help
  -V, --version     Print version
```

For each command, you can use `-h` or `--help` to get detailed help:

```shell
# IPV6PrefixFilter run --help
Run the program (in the foreground)

Usage: IPv6PrefixFilter run [OPTIONS]

Options:
  -p, --ipv6-prefixes <IPV6_PREFIXES>  Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option
  -i, --interface <INTERFACE>          Specify the WAN interface
  -b, --blacklist-mode                 Enable blacklist mode. Prefixes specified with `-p` will be blocked
      --disable-nft-autoset            Disable the feature of auto-setting nftables rules
  -h, --help                           Print help
```

### Examples

If you want to intercept RA advertisements from the WAN interface, only allowing the prefix `FFFF:FFFF:FFFF::/48`, you can use the following command:

```shell
IPV6PrefixFilter run -i wan -p FFFF:FFFF:FFFF::/48
```

## Build Guide

The program uses `cargo` as the build tool.

To get a dynamically linked build:

```shell
cargo build --release
```

To get a statically linked build:

```shell
cargo build --release --target x86_64-unknown-linux-musl
```

Note that you may need to install the `musl` toolchain to use this target triple. On some systems, you can install it via the package manager. For example, on Ubuntu:

```shell
sudo apt-get install musl-tools
```

Your `cargo` may also need to be configured for proper cross-compilation:

```shell
rustup target add x86_64-unknown-linux-musl
```

### Building on Windows

```shell
cargo build --release --target=x86_64-pc-windows-msvc
```


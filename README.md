# IPV6PrefixFilter

[README](README.md) | [中文文档](README_ZH.md)

This program is a router advertisement (RA) filter for Linux that filters RA attempting to set unspecified IPv6 prefixes. Occasionally, misconfigured routers may send RAs with incorrect prefix settings, or someone may intentionally send false RAs to disrupt your network.

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


### Command-Line Help

Use the `-h` or `--help` parameter to get help:

```shell
# IPv6PrefixFilter --help
A simple IPv6 Router Advertisement prefix filter using nftables.

Usage: IPv6PrefixFilter [OPTIONS]

Options:
  -p, --prefix <PREFIXES>     IPv6 prefixes to allow (default) or block (if -b is set)
  -i, --interface <INTERFACE>  Network interface to filter on (e.g., eth0)
  -b, --blacklist              Enable blacklist mode: prefixes specified with `-p` will be BLOCKED
  -c, --clear                  Clear the nftables rules set by the program and exit
      --no-nft                 Disable automatic setup of nftables rules
  -v, --verbose...             Verbosity level. Use -v for info, -vv for debug
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

#### 1. Allow only specific prefix on eth0
```shell
IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64
```

#### 2. Block a specific prefix on eth0 (Blacklist mode)
```shell
IPv6PrefixFilter -i eth0 -b -p 2001:db8:bad::/48
```

#### 3. Allow multiple prefixes on eth0
```shell
IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64 -p 2001:db8:2::/64
```

#### 3. Run as a Systemd Service (Recommended for persistent protection)
Create a file at `/etc/systemd/system/ra-filter.service`:

```ini
[Unit]
Description=IPv6 Router Advertisement Prefix Filter
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64

Restart=on-failure
RestartSec=2

# Capabilities required for nftables and NFQUEUE
CapabilityBoundingSet=CAP_NET_ADMIN
AmbientCapabilities=CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```

Then enable and start it:
```shell
systemctl enable ra-filter
systemctl start ra-filter
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



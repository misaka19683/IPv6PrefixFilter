# IPV6PrefixFilter
[README](README.md) | [中文文档](README_ZH.md)

**Under Construction - Not Yet Functional**

This program is a filter that drops router advertisement that trying to set a non-specified IPv6 prefix. Sometimes a misconfigured router will send you the wrong RA for the wrong prefix setting, or someone will intentionally send you the wrong RA to interfere with your network.

## TODO

- [x] Intercept RAs using NFTables and redirect them to a queue for processing.
- [ ] Analyze RA content and determine whether to discard them.
- [ ] Implement a comprehensive command-line interface.
- [ ] Integrate NFTables rule management.
- [ ] Ensure compliance with the Unix philosophy.

## Planned Usage

Use the `-h` or `--help` option to view the help menu.

```shell
# IPV6PrefixFilter --help
Some version information here.

-p, --prefix      Specify the allowed IPv6 prefixes. Multiple prefixes can be allowed by repeating the `-p` option.
-b, --blacklist-mode  Enable blacklist mode. Prefixes specified with `-p` will be blocked.
-v, --verbose     Display detailed runtime information.
-d, --daemon      Run the program in the background as a daemon.
-h, --help        Display this help menu.
```

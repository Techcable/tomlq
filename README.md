tomlq
======
[jq](https://stedolan.github.io/jq/) for [TOML](https://toml.io/en/) (and yaml) files

Written in Rust.

This is an extremely lightweight frontend to jq. It doesn't even do any argument processing. It just translates from TOML/YAML -> JSON.
Whether using TOML/YAML mode is inferred from --yaml or --toml flags.
When using a file as input, it can auto-detect the input format (based on file extension).

Binaries freely available for Linux on Github releases :)

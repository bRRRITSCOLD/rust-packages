# rust-packages

Shared Rust crates for GPUI desktop apps. The main one, `brrritscold-core-gpui`, renders the shared Penpot design library in GPUI. It is the second renderer of that library: it follows [`@brrritscold/core-react`](https://github.com/bRRRITSCOLD/ts-packages/tree/main/packages/core-react), the reference renderer.

Linux is first-class, macOS best-effort, Windows not targeted.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option, unless a crate's manifest says otherwise.

## Crates

| Crate | What it is |
|---|---|
| `brrritscold-core-gpui` | The styled component layer on `gpui-base`. |
| `brrritscold-core-gpui-tokens` | Design tokens generated from core-react's `tokens.json`. |
| `xtask` | Workspace tasks (internal). |

## Development

The toolchain is pinned in `rust-toolchain.toml`. On Linux, install GPUI's packages with `script/install-linux-deps.sh`.

| Command | What it does |
|---|---|
| `cargo test-unit` | Unit tier (nextest profile `unit`, plus doctests). |
| `cargo test-unit-ci` | Setup → unit tier → teardown (always); exits with the tests' code. |
| `bacon unit` | Unit tier in watch mode. |

Commits follow [Conventional Commits](https://www.conventionalcommits.org/): release-plz derives the version bump and `CHANGELOG.md` from them. Releases are git-only: one workspace version, one `v<version>` tag. Apps pin every `brrritscold-*` crate to the same tag, and copy the GPUI pin (`gpui-base`, `gpui` = `gpui-pre`) from this workspace's `Cargo.toml`.

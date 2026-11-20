# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.9](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.8...v0.1.9) - 2026-11-20

**GPUI pin:** gpui-kit 0.10.0 (`gpui-base = "=0.10.0"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.9` tag.

**core-react pin:** 1.7.1 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node 0.1.3 + design-core-js 0.2.0).

### Added

- *(gpui)* gpui-kit 0.10.0 ([#71](https://github.com/bRRRITSCOLD/rust-packages/pull/71))
- *(tokens)* core-react 1.7.1 tokens ([#74](https://github.com/bRRRITSCOLD/rust-packages/pull/74))

## [0.1.8](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.7...v0.1.8) - 2026-11-17

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.8` tag.

**core-react pin:** 1.7.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node 0.1.3 + design-core-js 0.2.0).

### Added

- *(tokens)* core-react 1.7.0 tokens ([#64](https://github.com/bRRRITSCOLD/rust-packages/pull/64))
- *(gpui)* DropdownMenu::align; fix field wrap, input selection, dialog Enter ([#65](https://github.com/bRRRITSCOLD/rust-packages/pull/65))

### Other

- *(gpui)* read core-react's per-mode state tokens ([#68](https://github.com/bRRRITSCOLD/rust-packages/pull/68))

## [0.1.7](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.6...v0.1.7) - 2026-11-17

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.7` tag.

**core-react pin:** 1.6.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Added

- *(tokens)* core-react 1.6.0 tokens ([#60](https://github.com/bRRRITSCOLD/rust-packages/pull/60))

## [0.1.6](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.5...v0.1.6) - 2026-11-17

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.6` tag.

**core-react pin:** 1.5.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Added

- *(tokens)* core-react 1.5.0 tokens ([#56](https://github.com/bRRRITSCOLD/rust-packages/pull/56))

## [0.1.5](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.4...v0.1.5) - 2026-11-15

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.5` tag.

**core-react pin:** 1.4.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Fixed

- *(gpui)* step spinning icons on a shared 15 fps tick; spin(bool) on StateDot and Checks ([#53](https://github.com/bRRRITSCOLD/rust-packages/pull/53))

## [0.1.4](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.3...v0.1.4) - 2026-11-15

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.4` tag.

**core-react pin:** 1.4.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Added

- *(gpui)* StateDot and Checks, with a spinning LoaderCircle ([#50](https://github.com/bRRRITSCOLD/rust-packages/pull/50))

## [0.1.3](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.2...v0.1.3) - 2026-11-14

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.3` tag.

**core-react pin:** 1.4.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Fixed

- *(gpui)* paint toasts above dialogs and sheets ([#46](https://github.com/bRRRITSCOLD/rust-packages/pull/46))

## [0.1.2](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.1...v0.1.2) - 2026-11-12

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.2` tag.

**core-react pin:** 1.4.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.2).

### Added

- *(tokens)* core-react 1.4.0 tokens ([#42](https://github.com/bRRRITSCOLD/rust-packages/pull/42))
- *(tokens)* core-react 1.3.0 tokens ([#40](https://github.com/bRRRITSCOLD/rust-packages/pull/40))

## [0.1.1](https://github.com/bRRRITSCOLD/rust-packages/compare/v0.1.0...v0.1.1) - 2026-11-11

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.1` tag.

**core-react pin:** 1.2.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node 0.1.2 + design-core-js 0.1.1).

### Added

- *(tokens)* core-react 1.2.0 tokens ([#37](https://github.com/bRRRITSCOLD/rust-packages/pull/37))

## [0.1.0](https://github.com/bRRRITSCOLD/rust-packages/releases/tag/v0.1.0) - 2026-11-11

**GPUI pin:** gpui-kit 0.9.1 (`gpui-base = "=0.9.1"`, `gpui = { package = "gpui-pre", version = "=0.3.12" }` and its `gpui-pre-*` siblings at `=0.3.12`). Apps copy these lines next to the `v0.1.0` tag.

**core-react pin:** 1.1.0 (tokens, library snapshot, Lucide 1.68.0, drift checker design-core-node/js 0.1.0).

### Added

- *(core-gpui)* composites and data table (accordion, tabs, calendar, date picker, data table parts) ([#28](https://github.com/bRRRITSCOLD/rust-packages/pull/28))
- *(core-gpui)* combobox, with an internal input group ([#27](https://github.com/bRRRITSCOLD/rust-packages/pull/27))
- *(core-gpui)* overlays and toast (dialog, alert dialog, sheet, popover, tooltip, select, dropdown menu, toast) ([#26](https://github.com/bRRRITSCOLD/rust-packages/pull/26))
- *(core-gpui)* form controls (input, textarea, checkbox, switch, radio group, toggle, toggle group, field) ([#24](https://github.com/bRRRITSCOLD/rust-packages/pull/24))
- *(core-gpui)* presentational set (badge, card, alert, avatar, separator, skeleton, label, breadcrumb, table, pagination) ([#23](https://github.com/bRRRITSCOLD/rust-packages/pull/23))
- cargo xtask drift with a pending list and a blocking CI job ([#18](https://github.com/bRRRITSCOLD/rust-packages/pull/18))
- Button, gallery, registry and captures ([#17](https://github.com/bRRRITSCOLD/rust-packages/pull/17))
- *(core-gpui)* Lucide icons vendored at core-react's Lucide version ([#15](https://github.com/bRRRITSCOLD/rust-packages/pull/15))
- *(tokens)* vendor core-react 1.1.0 tokens and generate generated.rs ([#12](https://github.com/bRRRITSCOLD/rust-packages/pull/12))
- *(core-gpui)* init, bundled Inter, Mode/ThemePreference and the theme root plugin ([#14](https://github.com/bRRRITSCOLD/rust-packages/pull/14))
- workspace skeleton, CI, release-plz and the GPUI pin ([#11](https://github.com/bRRRITSCOLD/rust-packages/pull/11))

### Fixed

- *(core-gpui)* button paints focus and disabled with the control helpers ([#30](https://github.com/bRRRITSCOLD/rust-packages/pull/30))
- *(core-gpui)* licence expression and NOTICE name Inter and Lucide ([#21](https://github.com/bRRRITSCOLD/rust-packages/pull/21))

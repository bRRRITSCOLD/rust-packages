# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

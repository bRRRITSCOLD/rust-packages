# Fonts

## Inter

Static Inter TTFs, weights 400 (Regular), 500 (Medium), 600 (SemiBold) and 700 (Bold), from
[Inter 4.1](https://github.com/rsms/inter/releases/tag/v4.1) (`extras/ttf/`). Licensed under the
SIL Open Font License 1.1 ([`OFL.txt`](OFL.txt)).

core-react loads Inter through `@fontsource/inter` 5.3.0, which ships Inter 4.1. When a
core-react bump PR flags an `@fontsource/inter` change, re-vendor the four files from the
matching Inter release and update this note.

## JetBrains Mono

Static JetBrains Mono TTFs, weights 400 (`JetBrainsMono-Regular.ttf`) and 700
(`JetBrainsMono-Bold.ttf`), unmodified from `fonts/ttf/` in `JetBrainsMono-2.304.zip` of the
[JetBrains Mono 2.304 release](https://github.com/JetBrains/JetBrainsMono/releases/tag/v2.304).
Licensed under the SIL Open Font License 1.1
([`JetBrainsMono-OFL.txt`](JetBrainsMono-OFL.txt), the release's `OFL.txt`).

| File | SHA-256 |
|---|---|
| `JetBrainsMono-2.304.zip` | `6f6376c6ed2960ea8a963cd7387ec9d76e3f629125bc33d1fdcd7eb7012f7bbf` |
| `JetBrainsMono-Regular.ttf` | `a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f` |
| `JetBrainsMono-Bold.ttf` | `5590990c82e097397517f275f430af4546e1c45cff408bde4255dad142479dcb` |

core-react (1.3.0 on) loads JetBrains Mono through `@fontsource/jetbrains-mono` 5.3.0. That
package ships only WOFF/WOFF2 subsets of Google Fonts' build (its name table says
`Version 2.211`, a version with no JetBrains release), so the TTFs come from the newest
official release instead. When a core-react bump moves `@fontsource/jetbrains-mono`, check
whether a newer JetBrains Mono release exists and re-vendor both files if so.

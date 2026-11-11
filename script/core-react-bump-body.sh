#!/usr/bin/env bash
# Prints the body of the daily core-react bump PR: run after `cargo xtask tokens update latest`.
# Usage: script/core-react-bump-body.sh <old-version> <new-version>
set -euo pipefail

old=$1
new=$2
registry=https://registry.npmjs.org/@brrritscold%2fcore-react
dep() { curl -fsSL "$registry/$1" | jq -r --arg d "$2" '.dependencies[$d] // .devDependencies[$d] // "none"'; }

echo "Bumps the core-react pin **${old} → ${new}**: \`tokens.json\`, \`library.snapshot.json\` and \`generated.rs\`."
echo
echo "Advisory: merge when green and checked; it may stay open, and versions may be skipped. A renamed or removed token shows up as a compile error in \`core-gpui\`; fix it on this branch."
echo
echo "**Required before merge when tokens or the snapshot changed:** a local \`cargo xtask captures compare\`, linked here."
echo

echo "## core-react CHANGELOG"
echo
changelog=$(curl -fsSL "https://raw.githubusercontent.com/bRRRITSCOLD/ts-packages/%40brrritscold/core-react%40${new}/packages/core-react/CHANGELOG.md" || true)
if [ -n "$changelog" ]; then
  # No early exit or head: under pipefail a reader that stops early SIGPIPEs the writer (exit 141).
  awk -v old="## ${old}" 'index($0, old) == 1 { stop = 1 } stop { next } /^## / { show = 1 } show' <<<"$changelog"
else
  echo "_Couldn't fetch the CHANGELOG at the \`@brrritscold/core-react@${new}\` tag._"
fi
echo

for pair in "lucide-react:Lucide icons (re-vendored by the tokens xtask)" "@fontsource/inter:Inter (re-vendor the TTFs in core-gpui by hand)" "@brrritscold/design-core-node:drift checker CLI (pin moved by the tokens xtask)" "@brrritscold/design-core-js:drift checker rules (pin moved by the tokens xtask)"; do
  name=${pair%%:*}
  label=${pair#*:}
  before=$(dep "$old" "$name")
  after=$(dep "$new" "$name")
  if [ "$before" != "$after" ]; then
    echo "- ⚠️ **${label}:** \`${name}\` ${before} → ${after}"
  else
    echo "- ${label}: \`${name}\` unchanged (${after})"
  fi
done
echo

for file in crates/core-gpui-tokens/tokens.json design/library.snapshot.json; do
  echo "## \`${file}\` diff"
  echo
  stat=$(git diff --stat -- "$file" | tail -1)
  if [ -z "$stat" ]; then
    echo "No change."
  else
    echo "${stat}"
    echo
    echo '<details><summary>Diff</summary>'
    echo
    echo '```diff'
    git diff -- "$file" | sed -n '1,400p' # not head: see above
    echo '```'
    echo
    echo '</details>'
  fi
  echo
done

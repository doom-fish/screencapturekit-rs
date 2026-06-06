#!/usr/bin/env bash
#
# Cross-check FFI symbol names between the Rust `extern "C"` declarations and the
# Swift `@_cdecl` exports, to catch symbol drift (a renamed thunk on one side
# only) before it turns into a link error.
#
# This is a standalone developer tool — run it manually. It is intentionally NOT
# wired into CI.
#
# Caveat: some Rust `extern "C"` declarations resolve against the `apple-cf` /
# `apple-metal` Swift bridges rather than this repo's `swift-bridge/`. Those show
# up under "Rust-only" and are expected (they are provided by the dependency).
#
# Usage:
#   scripts/check_ffi_symbols.sh
#
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

# Swift: every @_cdecl("symbol") export across the in-repo Swift bridge.
swift_syms="$(grep -rhoE '@_cdecl\("[A-Za-z0-9_]+"\)' swift-bridge \
  | grep -oE '"[A-Za-z0-9_]+"' | tr -d '"' | sort -u)"

# Rust: every `fn name` declared inside an `extern "C" { ... }` block in src/.
# FFI extern blocks contain only declarations (no bodies), so a non-greedy match
# up to the first closing brace is reliable.
rust_files="$(grep -rl 'extern "C"' src --include='*.rs')"
rust_syms="$(perl -0777 -ne '
  while (/extern\s*"C"\s*\{(.*?)\n\}/sg) {
    my $block = $1;
    while ($block =~ /\bfn\s+([A-Za-z0-9_]+)/g) { print "$1\n"; }
  }
' $rust_files | sort -u)"

rust_only="$(comm -23 <(printf '%s\n' "$rust_syms") <(printf '%s\n' "$swift_syms"))"
swift_only="$(comm -13 <(printf '%s\n' "$rust_syms") <(printf '%s\n' "$swift_syms"))"

echo "Rust extern \"C\" symbols:        $(printf '%s\n' "$rust_syms"  | grep -c .)"
echo "Swift @_cdecl symbols:          $(printf '%s\n' "$swift_syms" | grep -c .)"
echo
echo "== Rust-only (no in-repo @_cdecl; may be provided by apple-cf/apple-metal) =="
printf '%s\n' "$rust_only" | sed '/^$/d;s/^/  /'
echo
echo "== Swift-only (@_cdecl exported but no Rust extern references it) =="
printf '%s\n' "$swift_only" | sed '/^$/d;s/^/  /'

# Swift-only symbols are the actionable signal: a thunk no Rust code declares.
if [ -n "$swift_only" ]; then
  echo
  echo "warning: found Swift @_cdecl exports with no matching Rust extern declaration." >&2
fi

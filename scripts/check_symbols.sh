#!/usr/bin/env bash
# check_symbols.sh — Verify macOS version-gated symbols in the Swift bridge static library.
#
set -euo pipefail

if [[ $# != 3 || ( $1 != --assert-absent && $1 != --assert-present ) || -z $2 ]]; then
    echo "Usage: $0 --assert-absent|--assert-present SYMBOL ARCHIVE" >&2
    exit 2
fi

archive=$3
if [[ $archive != /* ]]; then archive=./$archive; fi
members=$(/usr/bin/ar -t "$archive")
if [[ -z $members ]]; then
    echo "The archive has no members: $archive" >&2
    exit 1
fi

symbols=$(/usr/bin/nm -u -j "$archive")
if grep -Fxq -- "_OBJC_CLASS_\$_$2" <<< "$symbols"; then
    actual=present
else
    actual=absent
fi
printf '%s: %s is %s\n' "$archive" "$2" "$actual"
[[ $1 == "--assert-$actual" ]]

#!/usr/bin/env bash
# check_symbols.sh — Verify macOS version-gated symbols in the Swift bridge static library.
#
# Usage:
#   scripts/check_symbols.sh --assert-absent SCScreenshotConfiguration
#   scripts/check_symbols.sh --assert-present SCScreenshotConfiguration
#
# Exits 0 on success, 1 on assertion failure.

set -euo pipefail

MODE=""
SYMBOL=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --assert-absent) MODE="absent"; shift; SYMBOL="$1"; shift ;;
        --assert-present) MODE="present"; shift; SYMBOL="$1"; shift ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$MODE" || -z "$SYMBOL" ]]; then
    echo "Usage: $0 --assert-absent|--assert-present SYMBOL" >&2
    exit 2
fi

# Find the Swift bridge static library in Cargo's build output
LIB=$(find target -name "libScreenCaptureKitBridge.a" -path "*/release/*" 2>/dev/null | head -1)
if [[ -z "$LIB" ]]; then
    LIB=$(find target -name "libScreenCaptureKitBridge.a" 2>/dev/null | head -1)
fi
if [[ -z "$LIB" ]]; then
    echo "ERROR: libScreenCaptureKitBridge.a not found in target/" >&2
    exit 1
fi

echo "Checking $LIB for OBJC_CLASS_\$_${SYMBOL}..."

FOUND=$(/usr/bin/nm -u "$LIB" 2>/dev/null | grep -c "OBJC_CLASS.*${SYMBOL}" || true)

case "$MODE" in
    absent)
        if [[ "$FOUND" -gt 0 ]]; then
            echo "FAIL: Symbol ${SYMBOL} found ($FOUND references) but should be ABSENT"
            /usr/bin/nm -u "$LIB" 2>/dev/null | grep "OBJC_CLASS.*${SYMBOL}"
            exit 1
        else
            echo "OK: Symbol ${SYMBOL} is absent (as expected)"
        fi
        ;;
    present)
        if [[ "$FOUND" -eq 0 ]]; then
            echo "FAIL: Symbol ${SYMBOL} not found but should be PRESENT"
            exit 1
        else
            echo "OK: Symbol ${SYMBOL} is present ($FOUND references, as expected)"
        fi
        ;;
esac

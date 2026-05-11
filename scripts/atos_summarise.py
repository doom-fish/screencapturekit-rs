#!/usr/bin/env python3
"""Symbolicate a samply profile using `atos` (macOS canonical symbolicator).

Works reliably against profile_capture's own debug info; system frameworks
get fall-back to module name + offset.

Usage:
    python3 atos_summarise.py /tmp/profile_capture.json.gz
"""

import gzip
import json
import os
import re
import subprocess
import sys
from collections import Counter

PROFILE_PATH = sys.argv[1] if len(sys.argv) > 1 else "/tmp/profile_capture.json.gz"
TOP = 25

with gzip.open(PROFILE_PATH, "rt") as f:
    profile = json.load(f)

libs = profile["libs"]


def lib_path(idx):
    if 0 <= idx < len(libs):
        return libs[idx].get("path") or libs[idx].get("name")
    return None


# Group all unique (lib_idx, address) tuples per lib, then atos one batch per
# lib (atos accepts multiple addresses on one invocation).
def collect_pairs(threads):
    pairs = set()
    for t in threads:
        ft = t["frameTable"]
        fnt = t["funcTable"]
        for frame_idx in range(ft["length"]):
            res_idx = fnt["resource"][ft["func"][frame_idx]]
            addr = ft["address"][frame_idx]
            if res_idx >= 0 and addr >= 0:
                pairs.add((res_idx, addr))
    return pairs


threads = profile["threads"]
total_samples = sum(len(t.get("samples", {}).get("stack", [])) for t in threads)

per_thread = sorted(
    ((t.get("name") or "<unnamed>", len(t["samples"]["stack"]), t) for t in threads),
    key=lambda x: -x[1],
)

print(f"Total samples: {total_samples}")
print()
print("=== samples per thread (top 8) ===")
for name, c, _ in per_thread[:8]:
    print(f"  {100*c/total_samples:6.2f}%  {c:>6}  {name}")
print()

# Only profile_capture / our libs are worth atos-symbolicating.
own_lib_indices = {
    i for i, lib in enumerate(libs) if "profile_capture" in (lib.get("name") or "")
}
print(f"Own libs: {[libs[i]['name'] for i in own_lib_indices]}")
print()


def atos(lib_idx, addresses):
    path = lib_path(lib_idx)
    if not path or not os.path.exists(path):
        return {addr: f"{libs[lib_idx]['name']}!0x{addr:x}" for addr in addresses}
    args = ["xcrun", "atos", "-o", path, "-l", "0x100000000"]
    args += [f"0x{0x100000000 + addr:x}" for addr in addresses]
    try:
        result = subprocess.run(args, capture_output=True, text=True, check=False, timeout=30)
    except subprocess.TimeoutExpired:
        return {addr: f"{libs[lib_idx]['name']}!0x{addr:x}" for addr in addresses}
    lines = result.stdout.strip().split("\n")
    out = {}
    for addr, line in zip(addresses, lines):
        # If atos couldn't resolve, it returns the input address unchanged.
        if line.startswith("0x"):
            out[addr] = f"{libs[lib_idx]['name']}!0x{addr:x}"
        else:
            # Strip the trailing "(in <bin>)" and source location for compact display.
            name = re.sub(r"\s+\(in [^)]+\).*$", "", line)
            name = re.sub(r"::h[0-9a-f]{8,}.*$", "", name)
            out[addr] = name
    return out


# Symbolicate all pairs in our own libs.
all_pairs = collect_pairs([t for _, _, t in per_thread if len(t["samples"]["stack"]) >= 50])
own_pairs_by_lib = {}
for li, addr in all_pairs:
    if li in own_lib_indices:
        own_pairs_by_lib.setdefault(li, []).append(addr)

resolved: dict = {}
for li, addrs in own_pairs_by_lib.items():
    addrs = sorted(set(addrs))
    print(f"atos {len(addrs)} addresses in {libs[li]['name']}...")
    resolved.update(((li, a), name) for a, name in atos(li, addrs).items())

print(f"Resolved {len(resolved)} own-code frames.")
print()


def name_for(t, frame_idx):
    ft = t["frameTable"]
    fnt = t["funcTable"]
    res_idx = fnt["resource"][ft["func"][frame_idx]]
    addr = ft["address"][frame_idx]
    if 0 <= res_idx < len(libs):
        module = libs[res_idx]["name"]
    else:
        module = "?"
    return module, resolved.get((res_idx, addr), f"{module}!0x{addr:x}")


for name, count, t in per_thread[:6]:
    if count < 50:
        continue
    pct = 100 * count / total_samples
    print("=" * 80)
    print(f"Thread: {name}  ({count} samples, {pct:.2f}%)")
    print("=" * 80)

    ft = t["frameTable"]
    st = t["stackTable"]

    # Self-time
    self_c: Counter = Counter()
    incl_c: Counter = Counter()
    own_incl_c: Counter = Counter()
    for s in t["samples"]["stack"]:
        if s is None:
            continue
        leaf = st["frame"][s]
        self_c[leaf] += 1
        seen = set()
        seen_own = set()
        cur = s
        while cur is not None:
            fi = st["frame"][cur]
            if fi not in seen:
                incl_c[fi] += 1
                seen.add(fi)
            module, _ = name_for(t, fi)
            if module and ("profile_capture" in module or "screencapturekit" in module):
                if fi not in seen_own:
                    own_incl_c[fi] += 1
                    seen_own.add(fi)
            cur = st["prefix"][cur]

    def show(label, counter, top=20):
        total = count
        print(f"--- {label} (top {top}) ---")
        for fi, c in counter.most_common(top):
            module, sym = name_for(t, fi)
            pct = 100 * c / total if total else 0
            print(f"  {pct:6.2f}%  {c:>5}  [{module[:18]:<18}]  {sym[:130]}")
        print()

    show("self-time (LEAF — where the CPU actually was)", self_c, 15)
    show("inclusive (in stack at all)", incl_c, 12)
    show("inclusive — OUR CODE (profile_capture + screencapturekit)", own_incl_c, 25)

#!/usr/bin/env python3
"""Symbolicate a samply profile by hitting the local samply server's
/symbolicate/v5 endpoint, then summarise the hot self-time stacks.

Prereq: have `samply load --no-open --port <PORT> /path/to/profile.json.gz`
running. Pass the same port via SAMPLY_PORT env (default 3939).

Usage:
    samply load --no-open --port 3939 /tmp/profile_capture.json.gz &
    SAMPLY_PORT=3939 python3 scripts/symbolicate_profile.py /tmp/profile_capture.json.gz
"""

import gzip
import json
import os
import re
import sys
import urllib.request
from collections import Counter, defaultdict

PORT = int(os.environ.get("SAMPLY_PORT", "3939"))
TOP_N = 30


def load(path):
    with gzip.open(path, "rt") as f:
        return json.load(f)


def find_path_hash():
    """Read samply's saved log to discover the URL path hash."""
    for log_path in ("/tmp/samply_server.log", "/tmp/samply.log"):
        try:
            with open(log_path) as f:
                content = f.read()
            # Format examples seen:
            #   .../profile.json/?...
            #   .../from-url/...%2F<hash>%2Fprofile.json/?...
            # Look for the percent-encoded variant first.
            m = re.search(r"%2F([A-Za-z0-9]{20,})%2Fprofile\.json", content)
            if m:
                return m.group(1)
            m = re.search(r"/([A-Za-z0-9]{20,})/profile\.json", content)
            if m:
                return m.group(1)
        except FileNotFoundError:
            continue
    raise RuntimeError("Could not find samply path hash; ensure samply server is running")


def codeid_to_breakpad(code_id, arch=None):
    """Mozilla breakpad ID = uppercase debugId (no dashes) + zero-padded age '0'."""
    return code_id + "0"


def symbolicate_one(stack_pairs, mem_map, path_hash):
    """Given (lib_idx, address) pairs, return resolved [(lib, function), ...]."""
    body = {
        "memoryMap": mem_map,
        "stacks": [stack_pairs],
    }
    req = urllib.request.Request(
        f"http://localhost:{PORT}/{path_hash}/symbolicate/v5",
        data=json.dumps(body).encode(),
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def main():
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(1)
    profile = load(sys.argv[1])
    path_hash = find_path_hash()
    print(f"samply path hash: {path_hash}")

    libs = profile["libs"]
    mem_map = []
    for lib in libs:
        name = lib["name"]
        code_id = lib.get("codeId") or ""
        mem_map.append([name, codeid_to_breakpad(code_id)])

    threads = profile["threads"]
    total_samples = sum(len(t.get("samples", {}).get("stack", [])) for t in threads)
    print(f"Total samples: {total_samples}")
    per_thread = []
    for t in threads:
        per_thread.append(
            (t.get("name") or "<unnamed>", len(t.get("samples", {}).get("stack", [])), t)
        )
    per_thread.sort(key=lambda x: -x[1])

    print()
    print("=== samples per thread (top 8) ===")
    for name, count, _ in per_thread[:8]:
        pct = 100 * count / total_samples if total_samples else 0
        print(f"  {pct:6.2f}%  {count:>6}  {name}")

    # Bulk symbolicate every (lib_idx, address) pair across every thread we
    # care about. One big API call is much faster than per-leaf.
    interesting_threads = [(n, c, t) for n, c, t in per_thread if c >= 50]

    all_pairs = set()
    for _, _, t in interesting_threads:
        ft = t["frameTable"]
        fnt = t["funcTable"]
        addresses = ft["address"]
        funcs = ft["func"]
        resources = fnt["resource"]
        for frame_idx in range(ft["length"]):
            res_idx = resources[funcs[frame_idx]]
            if 0 <= res_idx < len(libs):
                all_pairs.add((res_idx, addresses[frame_idx]))

    pair_list = sorted(all_pairs)
    print(f"Symbolicating {len(pair_list)} unique frames...")
    addr_to_name: dict = {}
    # Smaller chunks — samply server can be flaky on huge POSTs.
    CHUNK = 200
    for i in range(0, len(pair_list), CHUNK):
        chunk = pair_list[i : i + CHUNK]
        # Retry once on connection error (samply server occasionally drops
        # the connection mid-batch on macOS).
        for attempt in range(3):
            try:
                resp = symbolicate_one(
                    [[int(li), int(addr)] for li, addr in chunk], mem_map, path_hash
                )
                break
            except Exception as e:
                if attempt == 2:
                    print(f"  chunk {i} failed after retries: {e}")
                    resp = None
                else:
                    import time
                    time.sleep(0.3)
        if resp is None:
            continue
        results = resp["results"][0]["stacks"][0]
        for (li, addr), r in zip(chunk, results):
            fn = r.get("function")
            if fn:
                addr_to_name[(li, addr)] = fn

    print(f"Resolved {len(addr_to_name)}/{len(pair_list)} frames")

    def name_for(t, frame_idx):
        ft = t["frameTable"]
        fnt = t["funcTable"]
        res_idx = fnt["resource"][ft["func"][frame_idx]]
        addr = ft["address"][frame_idx]
        if 0 <= res_idx < len(libs):
            module = libs[res_idx]["name"]
        else:
            module = "?"
        sym = addr_to_name.get((res_idx, addr))
        if sym is None:
            return module, f"{module}!0x{addr:x}"
        sym = re.sub(r"::h[0-9a-f]{8,}$", "", sym)
        return module, sym

    for name, count, t in interesting_threads[:6]:
        pct = 100 * count / total_samples
        print()
        print("=" * 72)
        print(f"Thread: {name}  ({count} samples, {pct:.1f}%)")
        print("=" * 72)

        ft = t["frameTable"]
        st = t["stackTable"]

        # Self-time (leaves)
        self_counter: Counter = Counter()
        for stack_idx in t["samples"]["stack"]:
            if stack_idx is None:
                continue
            self_counter[st["frame"][stack_idx]] += 1

        # Inclusive (every frame in stack, dedup per stack)
        incl_counter: Counter = Counter()
        # Inclusive but ONLY counting our crate's frames
        own_counter: Counter = Counter()
        for stack_idx in t["samples"]["stack"]:
            if stack_idx is None:
                continue
            seen = set()
            seen_own = set()
            cur = stack_idx
            while cur is not None:
                fi = st["frame"][cur]
                if fi not in seen:
                    incl_counter[fi] += 1
                    seen.add(fi)
                module, sym = name_for(t, fi)
                # Filter to OUR code: anything that demangles into the crate
                # name, or the binary itself with a Rust-ish name.
                if (
                    "screencapturekit" in sym
                    or sym.startswith("profile_capture")
                    or "<screencapturekit::" in sym
                    or "screencapturekit::" in sym
                ):
                    if fi not in seen_own:
                        own_counter[fi] += 1
                        seen_own.add(fi)
                cur = st["prefix"][cur]

        def show(label, counter, top=15):
            total = count
            print(f"--- {label} (top {top}) ---")
            for fi, c in counter.most_common(top):
                module, sym = name_for(t, fi)
                pct = 100 * c / total if total else 0
                print(f"  {pct:6.2f}%  {c:>5}  [{module[:18]:<18}]  {sym[:130]}")
            print()

        show("self-time (leaf frames)", self_counter, 20)
        show("inclusive — all frames in stack", incl_counter, 15)
        show("inclusive — OUR CODE ONLY (screencapturekit + profile_capture)", own_counter, 20)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Spec registry sync — hash-based spec-code drift detection.

Maintains a JSONL registry of spec file hashes and referenced code paths.
On check, compares current state to detect:
  - Modified spec files (spec changed since last review)
  - Stale files (code referenced in spec has changed, but spec wasn't updated)
  - Missing spec files (registry entry exists but file is gone)

Usage:
    python3 .dijiang/scripts/spec_sync.py check          # Check for drift
    python3 .dijiang/scripts/spec_sync.py update         # Update registry
    python3 .dijiang/scripts/spec_sync.py status         # Show registry status
"""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

from .paths import get_repo_root, get_spec_dir
from .io import walk_markdown_files


# ── Constants ──────────────────────────────────────────────────────────

REGISTRY_FILE = "registry.jsonl"


# ── Hashing ─────────────────────────────────────────────────────────────

def hash_file(path: Path) -> str:
    """SHA-256 hash of a file's content."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


# ── Registry I/O ────────────────────────────────────────────────────────

def load_registry(root: Path) -> dict[str, dict]:
    """Load the spec registry from ``.dijiang/spec/registry.jsonl``.

    Returns ``{relative_path: entry_dict}``.
    """
    registry_path = get_spec_dir(root) / REGISTRY_FILE
    registry: dict[str, dict] = {}
    if not registry_path.is_file():
        return registry
    for line in registry_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
            file_path = entry.get("file", "")
            if file_path:
                registry[file_path] = entry
        except json.JSONDecodeError:
            continue
    return registry


def save_entry(entry: dict, root: Path) -> bool:
    """Append or update a single registry entry."""
    registry_path = get_spec_dir(root) / REGISTRY_FILE
    try:
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        # For simplicity, rewrite whole file
        registry = load_registry(root)
        file_path = entry.get("file", "")
        registry[file_path] = entry
        lines = [json.dumps(e, ensure_ascii=False) for e in registry.values()]
        registry_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return True
    except OSError:
        return False


# ── Spec file scanning ─────────────────────────────────────────────────

def scan_spec_files(root: Path) -> list[dict]:
    """Scan all spec files and compute their hashes and referenced code paths."""
    spec_dir = get_spec_dir(root)
    results: list[dict] = []

    for md_file in walk_markdown_files(spec_dir):
        rel = md_file.relative_to(root).as_posix()
        content_hash = hash_file(md_file)
        referenced_paths = extract_referenced_paths(md_file)
        results.append({
            "file": rel,
            "hash": content_hash,
            "referenced": referenced_paths,
        })

    return results


def extract_referenced_paths(spec_file: Path) -> list[str]:
    """Extract file paths referenced inside a spec markdown file.

    Looks for:
      - Backtick code paths: `` `src/...` ``
      - File links: `` [text](path) ``
    """
    text = spec_file.read_text(encoding="utf-8")
    paths: set[str] = set()

    # Match backtick paths: `src/...` or `.dijang/...`
    for m in re.finditer(r"`([a-zA-Z0-9_./-]+\.[a-zA-Z0-9]+)`", text):
        p = m.group(1)
        if p.startswith((".dijiang/", "src/", "lib/", "tests/", "scripts/")):
            paths.add(p)

    # Match markdown links that look like file paths
    for m in re.finditer(r"\]\(([a-zA-Z0-9_./-]+\.[a-zA-Z0-9]+)\)", text):
        p = m.group(1)
        if p.startswith((".dijiang/", "src/", "lib/", "tests/", "scripts/")):
            paths.add(p)

    return sorted(paths)


def verify_referenced_paths(paths: list[str], root: Path) -> dict[str, str]:
    """Check which referenced paths exist and their current hashes."""
    result: dict[str, str] = {}
    for p in paths:
        full = root / p
        if full.is_file():
            result[p] = hash_file(full)
        elif full.is_dir():
            result[p] = "directory"
        else:
            result[p] = "missing"
    return result


# ── Check logic ────────────────────────────────────────────────────────

RunSummary = dict[str, list[dict[str, str]]]


def check_sync(root: Path) -> RunSummary:
    """Check spec registry against current state.

    Returns a dict with keys ``"modified"``, ``"stale"``, ``"missing"``,
    each containing a list of ``{"file": ..., "detail": ...}`` entries.
    """
    spec_dir = get_spec_dir(root)
    registry = load_registry(root)
    current = {e["file"]: e for e in scan_spec_files(root)}

    summary: RunSummary = {
        "modified": [],
        "stale": [],
        "missing": [],
    }

    # Compare registry to current
    for file_path, reg_entry in registry.items():
        cur = current.get(file_path)
        if cur is None:
            # Was in registry but file is gone
            summary["missing"].append({
                "file": file_path,
                "detail": "spec file not found on disk",
            })
            continue

        # Check hash
        reg_hash = reg_entry.get("hash", "")
        cur_hash = cur.get("hash", "")
        if reg_hash and cur_hash and reg_hash != cur_hash:
            summary["modified"].append({
                "file": file_path,
                "detail": "hash changed (spec was edited since last registry update)",
            })

        # Check referenced paths
        reg_refs = reg_entry.get("referenced", [])
        cur_refs = cur.get("referenced", [])
        stale_refs = []
        for ref_path in reg_refs:
            if ref_path not in cur_refs and (spec_dir / ref_path).is_file():
                # Referenced file was in old spec but not in current spec
                stale_refs.append(ref_path)

        # Check if referenced code files have changed
        for ref_path in cur_refs:
            full = root / ref_path
            if full.is_file():
                file_hash = hash_file(full)
                # If the code path is recorded but no previous hash, flag it
                if ref_path not in reg_refs:
                    stale_refs.append(ref_path)

        if stale_refs:
            summary["stale"].append({
                "file": file_path,
                "detail": f"referenced files changed: {', '.join(stale_refs[:5])}",
            })

    return summary


# ── Update logic ────────────────────────────────────────────────────────

def update_registry(root: Path) -> int:
    """Rescan all spec files and rewrite the registry."""
    spec_dir = get_spec_dir(root)
    entries = scan_spec_files(root)
    registry_path = spec_dir / REGISTRY_FILE

    try:
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        lines = [json.dumps(e, ensure_ascii=False) for e in entries]
        registry_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"Registry updated: {len(entries)} entries → {registry_path}")
        return 0
    except OSError as e:
        print(f"Error: {e}")
        return 1


# ── CLI ─────────────────────────────────────────────────────────────────

def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description="Spec registry sync")
    parser.add_argument(
        "command", nargs="?", default="check",
        choices=["check", "update", "status"],
        help="check: compare hashes | update: rescan and save | status: show registry",
    )
    args = parser.parse_args()

    try:
        root = get_repo_root()
    except SystemExit:
        return 1

    if args.command == "check":
        summary = check_sync(root)
        if any(summary.values()):
            print("Spec sync issues found:")
            for category, items in summary.items():
                if items:
                    print(f"\n  [{category}]")
                    for item in items:
                        print(f"    {item['file']}: {item['detail']}")
            return 1
        else:
            print("All spec files in sync.")
            return 0

    elif args.command == "update":
        return update_registry(root)

    elif args.command == "status":
        count = len(load_registry(root))
        current = len(scan_spec_files(root))
        print(f"Registry entries: {count}")
        print(f"Spec files on disk: {current}")
        if count != current:
            print(f"  (registry may be stale — run 'update')")
        return 0

    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())

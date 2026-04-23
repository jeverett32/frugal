#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import subprocess
import tempfile
from pathlib import Path


def estimate_tokens(text: str) -> int:
    return math.ceil(len(text.encode("utf-8")) / 4)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def hash_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=str(cwd),
        text=True,
        capture_output=True,
        check=True,
    )


def repo_files(repo: Path) -> list[Path]:
    paths: list[Path] = []
    for root, dirs, files in os.walk(repo):
        root_path = Path(root)
        dirs[:] = [
            d
            for d in dirs
            if d not in {".git", ".fgl", "target", "node_modules", ".venv", "__pycache__", "dist", "build"}
        ]
        for file in files:
            path = root_path / file
            if ".git" in path.parts or ".fgl" in path.parts:
                continue
            paths.append(path)
    return sorted(paths)


def raw_repo_snapshot(repo: Path) -> str:
    blocks: list[str] = []
    for path in repo_files(repo):
        rel = path.relative_to(repo).as_posix()
        try:
            text = read_text(path)
        except UnicodeDecodeError:
            continue
        blocks.append(f"## `{rel}`\n\n```\n{text}\n```\n")
    return "\n".join(blocks)


def mutate_file(path: Path) -> None:
    original = read_text(path)
    path.write_text(original + "\n# benchmark edit\n", encoding="utf-8")


def benchmark(repo: Path, active: list[str], fgl_bin: str) -> dict:
    raw_before = raw_repo_snapshot(repo)
    pack_before = run([fgl_bin, "pack", *active], repo).stdout

    active_path = repo / active[0]
    original = read_text(active_path)
    mutate_file(active_path)

    try:
        raw_after = raw_repo_snapshot(repo)
        pack_after = run([fgl_bin, "pack", *active], repo).stdout
    finally:
        active_path.write_text(original, encoding="utf-8")

    return {
        "repo": str(repo),
        "active_files": active,
        "raw_before_tokens": estimate_tokens(raw_before),
        "pack_before_tokens": estimate_tokens(pack_before),
        "raw_after_tokens": estimate_tokens(raw_after),
        "pack_after_tokens": estimate_tokens(pack_after),
        "raw_before_sha256": hash_text(raw_before),
        "raw_after_sha256": hash_text(raw_after),
        "pack_before_sha256": hash_text(pack_before),
        "pack_after_sha256": hash_text(pack_after),
        "pack_prefix_unchanged": pack_before.split("# Active Zone")[0] == pack_after.split("# Active Zone")[0],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark frugal pack vs raw repo context.")
    parser.add_argument("repo", type=Path, help="Path to repo to benchmark")
    parser.add_argument("active", nargs="+", help="Active file paths relative to repo")
    parser.add_argument("--fgl", default="target/debug/fgl", help="Path to fgl binary")
    args = parser.parse_args()

    result = benchmark(args.repo.resolve(), args.active, args.fgl)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

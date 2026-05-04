#!/usr/bin/env python3
"""Continuously stress-test Litter slicing with random opensourced roots."""

from __future__ import annotations

import argparse
import glob
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


DEFAULT_SOURCE = Path("/tmp/rustninja-litter-sparse/shared/rust-bridge")
DEFAULT_OUTPUT_PREFIX = Path("/tmp/rustninja-litter-loop")
DEFAULT_KINDS = ("fn", "mod", "trait", "struct", "enum")

FN_RE = re.compile(
    r"^(?:(?:pub(?:\([^)]*\))?|async|const|unsafe|extern\s+\"[^\"]+\"|extern)\s+)*"
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
ITEM_RE = {
    "fn": FN_RE,
    "enum": re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)"),
    "trait": re.compile(
        r"^(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)"
    ),
    "struct": re.compile(
        r"^(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)"
    ),
    "mod": re.compile(r"^(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)(?:;|\s*\{)"),
}
EXTERNAL_MOD_RE = re.compile(r"^(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTR_RE = re.compile(r"#\[path\s*=\s*\"([^\"]+)\"\]")
SECTION_RE = re.compile(r"^\s*\[[^\]]+\]\s*$")


@dataclass(frozen=True)
class Package:
    name: str
    root: Path
    entry: Path


@dataclass(frozen=True)
class Candidate:
    kind: str
    path: Path
    line: int
    name: str
    package: Package
    module_path: tuple[str, ...]

    @property
    def key(self) -> tuple[Path, int]:
        return (self.path, self.line)

    def display(self, workspace: Path) -> str:
        try:
            relative = self.path.relative_to(workspace)
        except ValueError:
            relative = self.path
        module = "::".join((self.package.name, *self.module_path, self.name))
        return f"{relative}:{self.line}:{self.kind}:{self.name} ({module})"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Randomly mark 5 Rust items in Litter, slice, cargo-check, clean, repeat."
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="agentic slicer repo root",
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=DEFAULT_SOURCE,
        help="Litter rust-bridge workspace root",
    )
    parser.add_argument(
        "--output-prefix",
        type=Path,
        default=DEFAULT_OUTPUT_PREFIX,
        help="output directory prefix; batch number is appended",
    )
    parser.add_argument("--start", type=int, default=1, help="first batch number")
    parser.add_argument(
        "--feedback-loop",
        type=int,
        default=2,
        help="slicers --feedback-loop iteration count",
    )
    parser.add_argument(
        "--roots-per-batch",
        type=int,
        default=5,
        help="number of random roots to mark each batch",
    )
    parser.add_argument(
        "--kinds",
        nargs="+",
        default=list(DEFAULT_KINDS),
        choices=sorted(ITEM_RE),
        help="root kinds to prefer before filling from the remaining pool",
    )
    parser.add_argument(
        "--public-only",
        action="store_true",
        help="only pick lines whose item declaration starts with pub",
    )
    parser.add_argument(
        "--seed",
        type=int,
        help="deterministic random seed; omit for SystemRandom",
    )
    parser.add_argument(
        "--max-batches",
        type=int,
        help="stop after this many successful batches; omit to run forever",
    )
    parser.add_argument(
        "--keep-success-outputs",
        type=int,
        default=50,
        help="keep this many successful output directories; use -1 to keep all",
    )
    parser.add_argument(
        "--no-git-restore",
        action="store_true",
        help="do not restore/clean the source workspace before each batch",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.roots_per_batch <= 0:
        raise SystemExit("--roots-per-batch must be greater than zero")
    if args.feedback_loop <= 0:
        raise SystemExit("--feedback-loop must be greater than zero")

    repo = args.repo.resolve()
    source = args.source.resolve()
    opensourced_path = repo / "crates" / "opensourced"
    if not (repo / "Cargo.toml").exists():
        raise SystemExit(f"missing Cargo.toml under repo {repo}")
    if not (source / "Cargo.toml").exists():
        raise SystemExit(f"missing Litter Cargo.toml under source {source}")
    if not opensourced_path.exists():
        raise SystemExit(f"missing opensourced crate at {opensourced_path}")

    rng = random.Random(args.seed) if args.seed is not None else random.SystemRandom()
    successes = 0
    batch = args.start
    while args.max_batches is None or successes < args.max_batches:
        print(f"=== litter batch {batch} ===", flush=True)
        result = run_batch(args, repo, source, opensourced_path, rng, batch)
        if result != 0:
            return result
        successes += 1
        batch += 1
    return 0


def run_batch(
    args: argparse.Namespace,
    repo: Path,
    source: Path,
    opensourced_path: Path,
    rng: random.Random,
    batch: int,
) -> int:
    if not args.no_git_restore:
        restore_source(source)

    packages = load_packages(source)
    candidates = discover_candidates(packages, args.public_only)
    selected = select_roots(candidates, args.kinds, args.roots_per_batch, rng)
    if len(selected) != args.roots_per_batch:
        raise RuntimeError(f"expected {args.roots_per_batch} roots, selected {len(selected)}")

    inject_workspace_dependency(source / "Cargo.toml", opensourced_path)
    for package in sorted({candidate.package for candidate in selected}, key=lambda pkg: pkg.name):
        inject_package_dependency(package.root / "Cargo.toml")
    annotate(selected)

    for candidate in sorted(selected, key=lambda item: (str(item.path), item.line, item.kind)):
        print(candidate.display(source), flush=True)

    output_root = args.output_prefix.with_name(f"{args.output_prefix.name}-{batch}")
    shutil.rmtree(output_root, ignore_errors=True)

    command = [
        "cargo",
        "run",
        "-p",
        "opensource_cli",
        "--bin",
        "slicers",
        "--",
        "--feedback-loop",
        str(args.feedback_loop),
        str(source),
        str(output_root),
    ]
    completed = subprocess.run(command, cwd=repo)
    report_path = output_root / "slice-feedback.json"
    shutil.rmtree(output_root / "target-feedback", ignore_errors=True)

    if not report_path.exists():
        print(
            f"litter batch {batch} did not produce slice-feedback.json; output kept at {output_root}",
            flush=True,
        )
        return completed.returncode or 2

    report = json.loads(report_path.read_text())
    errors = count_diagnostics(report, "error")
    warnings = count_diagnostics(report, "warning")
    duration_text = (
        f" duration_ms={report['duration_ms']}" if report.get("duration_ms") is not None else ""
    )
    if completed.returncode != 0 or errors or warnings:
        print(
            f"litter batch {batch} produced errors={errors} warnings={warnings}; "
            f"target cleaned; output kept at {output_root}{duration_text}",
            flush=True,
        )
        print_priority_diagnostics(report)
        return completed.returncode or 3

    print(f"litter batch {batch}: target cleaned, warnings=0{duration_text}", flush=True)
    prune_success_outputs(args.output_prefix, args.keep_success_outputs, batch)
    return 0


def restore_source(source: Path) -> None:
    git_root = subprocess.run(
        ["git", "-C", str(source), "rev-parse", "--show-toplevel"],
        check=False,
        capture_output=True,
        text=True,
    )
    if git_root.returncode != 0:
        return
    root = git_root.stdout.strip()
    subprocess.run(["git", "-C", root, "restore", "."], check=True)
    subprocess.run(["git", "-C", root, "clean", "-fd"], check=True, stdout=subprocess.DEVNULL)


def load_packages(workspace: Path) -> list[Package]:
    manifest = read_toml(workspace / "Cargo.toml")
    members = manifest.get("workspace", {}).get("members", ["."])
    packages = []
    for member in expand_members(workspace, members):
        package_root = (workspace / member).resolve()
        manifest_path = package_root / "Cargo.toml"
        if not manifest_path.exists():
            continue
        package_manifest = read_toml(manifest_path)
        package_name = package_manifest.get("package", {}).get("name")
        if not package_name:
            continue
        entry = entry_source_path(package_root, package_manifest)
        if entry.exists():
            packages.append(Package(package_name, package_root, entry.resolve()))
    packages.sort(key=lambda package: package.name)
    return packages


def read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def expand_members(workspace: Path, members: list[str]) -> list[str]:
    expanded: list[str] = []
    for member in members:
        if "*" in member:
            for match in glob.glob(str(workspace / member)):
                if (Path(match) / "Cargo.toml").exists():
                    expanded.append(os.path.relpath(match, workspace))
        else:
            expanded.append(member)
    return sorted(set(expanded))


def entry_source_path(package_root: Path, manifest: dict) -> Path:
    lib_path = manifest.get("lib", {}).get("path")
    if lib_path:
        return (package_root / lib_path).resolve()
    default_lib = package_root / "src" / "lib.rs"
    if default_lib.exists():
        return default_lib.resolve()
    return (package_root / "src" / "main.rs").resolve()


def discover_candidates(packages: list[Package], public_only: bool) -> list[Candidate]:
    candidates: list[Candidate] = []
    for package in packages:
        visited: set[Path] = set()
        crawl_file(
            package=package,
            file_path=package.entry,
            module_path=(),
            module_dir=package.entry.parent,
            visited=visited,
            candidates=candidates,
            public_only=public_only,
        )
    return candidates


def crawl_file(
    package: Package,
    file_path: Path,
    module_path: tuple[str, ...],
    module_dir: Path,
    visited: set[Path],
    candidates: list[Candidate],
    public_only: bool,
) -> None:
    file_path = file_path.resolve()
    if file_path in visited or not file_path.exists():
        return
    visited.add(file_path)
    lines = file_path.read_text(errors="ignore").splitlines()

    for index, line in enumerate(lines, start=1):
        stripped = line.strip()
        if public_only and not stripped.startswith("pub"):
            continue
        if has_recent_test_attr(lines, index):
            continue
        for kind, pattern in ITEM_RE.items():
            match = pattern.match(stripped)
            if match:
                candidates.append(
                    Candidate(kind, file_path, index, match.group(1), package, module_path)
                )
                break

    for index, line in enumerate(lines, start=1):
        stripped = line.strip()
        match = EXTERNAL_MOD_RE.match(stripped)
        if not match or has_recent_test_attr(lines, index):
            continue
        name = match.group(1)
        child = resolve_external_module(module_dir, name, recent_path_attr(lines, index))
        if child is None:
            continue
        crawl_file(
            package=package,
            file_path=child,
            module_path=(*module_path, name),
            module_dir=child_module_dir(child, name),
            visited=visited,
            candidates=candidates,
            public_only=public_only,
        )


def has_recent_test_attr(lines: list[str], line_number: int) -> bool:
    for offset in range(line_number - 2, max(line_number - 7, -1), -1):
        if offset < 0:
            break
        stripped = lines[offset].strip()
        if not stripped:
            continue
        if not stripped.startswith("#["):
            break
        if "cfg(test)" in stripped or stripped == "#[test]":
            return True
    return False


def recent_path_attr(lines: list[str], line_number: int) -> str | None:
    for offset in range(line_number - 2, max(line_number - 7, -1), -1):
        if offset < 0:
            break
        stripped = lines[offset].strip()
        if not stripped:
            continue
        if not stripped.startswith("#["):
            break
        match = PATH_ATTR_RE.search(stripped)
        if match:
            return match.group(1)
    return None


def resolve_external_module(module_dir: Path, name: str, path_attr: str | None) -> Path | None:
    if path_attr:
        path = (module_dir / path_attr).resolve()
        return path if path.exists() else None
    file_module = (module_dir / f"{name}.rs").resolve()
    if file_module.exists():
        return file_module
    mod_module = (module_dir / name / "mod.rs").resolve()
    if mod_module.exists():
        return mod_module
    return None


def child_module_dir(child: Path, module_name: str) -> Path:
    if child.name == "mod.rs":
        return child.parent
    return child.parent / module_name


def select_roots(
    candidates: list[Candidate],
    preferred_kinds: list[str],
    roots_per_batch: int,
    rng: random.Random,
) -> list[Candidate]:
    by_kind: dict[str, list[Candidate]] = {kind: [] for kind in ITEM_RE}
    for candidate in candidates:
        by_kind[candidate.kind].append(candidate)

    selected: list[Candidate] = []
    used: set[tuple[Path, int]] = set()
    for kind in preferred_kinds:
        pool = [candidate for candidate in by_kind[kind] if candidate.key not in used]
        if not pool:
            continue
        item = rng.choice(pool)
        selected.append(item)
        used.add(item.key)
        if len(selected) == roots_per_batch:
            return selected

    remaining = [candidate for candidate in candidates if candidate.key not in used]
    while len(selected) < roots_per_batch and remaining:
        item = rng.choice(remaining)
        remaining = [candidate for candidate in remaining if candidate.key != item.key]
        selected.append(item)
        used.add(item.key)

    if len(selected) < roots_per_batch:
        counts = {kind: len(items) for kind, items in by_kind.items()}
        raise RuntimeError(f"not enough candidates for roots: {counts}")
    return selected


def inject_workspace_dependency(manifest_path: Path, opensourced_path: Path) -> None:
    line = f'opensourced = {{ path = "{opensourced_path}" }}'
    text = manifest_path.read_text()
    if re.search(r"(?m)^opensourced\s*=", text):
        return
    updated = insert_into_section(text, "[workspace.dependencies]", line)
    manifest_path.write_text(updated)


def inject_package_dependency(manifest_path: Path) -> None:
    text = manifest_path.read_text()
    if re.search(r"(?m)^opensourced\s*=", text):
        return
    updated = insert_into_section(text, "[dependencies]", "opensourced = { workspace = true }")
    manifest_path.write_text(updated)


def insert_into_section(text: str, section: str, line: str) -> str:
    lines = text.splitlines()
    try:
        index = next(idx for idx, current in enumerate(lines) if current.strip() == section)
    except StopIteration:
        suffix = "" if text.endswith("\n") or not text else "\n"
        return f"{text}{suffix}\n{section}\n{line}\n"

    insert_at = index + 1
    while insert_at < len(lines) and lines[insert_at].strip().startswith("#"):
        insert_at += 1
    while insert_at < len(lines) and lines[insert_at].strip() == "":
        insert_at += 1
    if insert_at < len(lines) and SECTION_RE.match(lines[insert_at]):
        lines.insert(insert_at, line)
    else:
        lines.insert(insert_at, line)
    return "\n".join(lines) + "\n"


def annotate(selected: list[Candidate]) -> None:
    by_path: dict[Path, list[Candidate]] = {}
    for candidate in selected:
        by_path.setdefault(candidate.path, []).append(candidate)

    for path, items in by_path.items():
        lines = path.read_text().splitlines()
        for candidate in sorted(items, key=lambda item: item.line, reverse=True):
            insert_at = candidate.line - 1
            if insert_at > 0 and "opensourced::opensourced" in lines[insert_at - 1]:
                continue
            lines.insert(insert_at, "#[opensourced::opensourced]")
        path.write_text("\n".join(lines) + "\n")


def count_diagnostics(report: dict, level: str) -> int:
    return sum(1 for diagnostic in report.get("diagnostics", []) if diagnostic.get("level") == level)


def print_priority_diagnostics(report: dict) -> None:
    diagnostics = [
        diagnostic
        for diagnostic in report.get("diagnostics", [])
        if diagnostic.get("level") in {"error", "warning"}
    ]
    diagnostics.sort(key=lambda diagnostic: 0 if diagnostic.get("level") == "error" else 1)
    for diagnostic in diagnostics[:12]:
        code = diagnostic.get("code")
        code_text = f"[{code}]" if code else ""
        print(f"  {diagnostic.get('level')}{code_text}: {diagnostic.get('message')}", flush=True)
        for span in diagnostic.get("spans", []):
            if not span.get("is_primary"):
                continue
            print(
                f"    at {span.get('file_name')}:{span.get('line_start')}:{span.get('column_start')}",
                flush=True,
            )
            text = span.get("text") or []
            if text:
                print(f"    | {text[0]}", flush=True)
            break


def prune_success_outputs(prefix: Path, keep: int, current_batch: int) -> None:
    if keep < 0:
        return
    parent = prefix.parent
    stem = prefix.name
    output_dirs: list[tuple[int, Path]] = []
    for path in parent.glob(f"{stem}-*"):
        if not path.is_dir():
            continue
        suffix = path.name.removeprefix(f"{stem}-")
        if suffix.isdigit():
            output_dirs.append((int(suffix), path))
    output_dirs.sort()
    for batch, path in output_dirs[: max(0, len(output_dirs) - keep)]:
        if batch != current_batch:
            shutil.rmtree(path, ignore_errors=True)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        print("interrupted", file=sys.stderr)
        raise SystemExit(130)

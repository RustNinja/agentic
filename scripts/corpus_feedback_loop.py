#!/usr/bin/env python3
"""Generic corpus runner for slicers.

The runner mutates a throwaway or git-backed Rust workspace by selecting random
Rust item roots, adding `#[opensourced::opensourced]`, generating a slice, and
recording compiler-feedback metrics as JSONL.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import random
import re
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_OUTPUT_PREFIX = Path("/tmp/slicers-corpus")
DEFAULT_KINDS = ("fn", "mod", "trait", "struct", "enum")
PARSE_TARGET_KINDS = {"lib", "proc-macro", "bin", "example", "test", "bench"}
DEV_TARGET_KINDS = {"example", "test", "bench"}

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
    target_name: str
    target_kinds: tuple[str, ...]


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
        target = "+".join(self.package.target_kinds)
        return (
            f"{relative}:{self.line}:{self.kind}:{self.name} "
            f"({module}; target={target}:{self.package.target_name})"
        )


@dataclass(frozen=True)
class CargoSource:
    user_source: Path
    manifest_path: Path
    cargo_root: Path
    metadata: dict[str, Any]


@dataclass
class CommandResult:
    command: list[str]
    cwd: Path
    exit_code: int
    timed_out: bool
    duration_ms: int
    stdout: str
    stderr: str

    @property
    def success(self) -> bool:
        return self.exit_code == 0 and not self.timed_out


def parse_args() -> argparse.Namespace:
    repo_default = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(
        description=(
            "Randomly mark Rust item roots in any Cargo workspace, run slicers "
            "with compiler feedback, and append JSONL corpus metrics."
        )
    )
    parser.add_argument("--repo", type=Path, default=repo_default, help="slicers repo root")
    parser.add_argument("--source", type=Path, required=True, help="Rust workspace under test")
    parser.add_argument(
        "--output-prefix",
        type=Path,
        default=DEFAULT_OUTPUT_PREFIX,
        help="output directory prefix; batch number is appended",
    )
    parser.add_argument(
        "--report",
        type=Path,
        default=repo_default / "reports" / "corpus_feedback.jsonl",
        help="JSONL metrics path",
    )
    parser.add_argument("--start", type=int, default=1, help="first batch number")
    parser.add_argument(
        "--max-batches",
        type=int,
        default=1,
        help="number of batches to run; use --continuous to run until interrupted",
    )
    parser.add_argument(
        "--continuous",
        action="store_true",
        help="ignore --max-batches and keep running until a failure or interrupt",
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
        help="only pick declarations whose item line starts with pub",
    )
    parser.add_argument("--seed", type=int, help="deterministic random seed")
    parser.add_argument(
        "--analyzer",
        default="syn",
        choices=("syn", "ra-hir"),
        help="slicers analyzer backend",
    )
    parser.add_argument(
        "--features",
        nargs="*",
        default=[],
        help="cargo features to enable when running slicers from source",
    )
    parser.add_argument(
        "--feedback-loop",
        type=int,
        default=1,
        help="slicers --feedback-loop iteration count",
    )
    parser.add_argument(
        "--feedback-timeout",
        type=int,
        default=600,
        help="slicers --feedback-timeout seconds; use 0 to disable",
    )
    parser.add_argument(
        "--feedback-target-dir",
        type=Path,
        help=(
            "shared CARGO_TARGET_DIR for feedback/repair validation; "
            "use this to keep dependency builds warm across corpus cases"
        ),
    )
    parser.add_argument(
        "--cargo-check-arg",
        action="append",
        default=[],
        help=(
            "extra argument to forward to slicers --cargo-check-arg for "
            "feedback/repair/production cargo check runs; repeat for flags "
            "that take values"
        ),
    )
    parser.add_argument(
        "--validation",
        choices=("preflight", "feedback", "repair", "production"),
        default="feedback",
        help=(
            "validation tier: preflight is fast and does not build dependencies; "
            "repair runs the conservative compiler repair loop; production uses "
            "the strict slicers production preset"
        ),
    )
    parser.add_argument(
        "--case-timeout",
        type=int,
        default=1200,
        help="outer timeout for each slicers invocation in seconds",
    )
    parser.add_argument(
        "--baseline-check",
        action="store_true",
        help="run cargo check --message-format=json on the source before mutation",
    )
    parser.add_argument(
        "--allow-baseline-failures",
        action="store_true",
        help=(
            "continue after a failing source baseline and classify generated "
            "baseline-matching errors separately from slicer regressions"
        ),
    )
    parser.add_argument(
        "--stop-on-warning",
        action="store_true",
        help="classify generated warnings as corpus failures",
    )
    parser.add_argument(
        "--deny-warnings",
        action="store_true",
        help="pass slicers --deny-warnings during feedback, repair, or production validation",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="continue after failed batches instead of stopping at first failure",
    )
    parser.add_argument(
        "--keep-success-outputs",
        type=int,
        default=10,
        help="keep this many successful output directories; use -1 to keep all",
    )
    parser.add_argument(
        "--no-git-restore",
        action="store_true",
        help="do not restore/clean the source workspace before and after each batch",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    validate_args(args)

    repo = args.repo.resolve()
    source = args.source.resolve()
    opensourced_path = repo / "crates" / "opensourced"
    rng = random.Random(args.seed) if args.seed is not None else random.SystemRandom()
    max_batches = None if args.continuous else args.max_batches

    if not (repo / "Cargo.toml").exists():
        raise SystemExit(f"missing Cargo.toml under repo {repo}")
    cargo_source = resolve_cargo_source(source)
    if not opensourced_path.exists():
        raise SystemExit(f"missing opensourced crate at {opensourced_path}")

    failures = 0
    successes = 0
    batch = args.start
    while max_batches is None or successes + failures < max_batches:
        row = run_batch(args, repo, cargo_source, opensourced_path, rng, batch)
        append_jsonl(args.report.resolve(), row)
        print_batch_summary(row)

        if row["passed"]:
            successes += 1
            prune_success_outputs(args.output_prefix.resolve(), args.keep_success_outputs, batch)
        else:
            failures += 1
            if not args.keep_going:
                return 1
        batch += 1

    return 1 if failures else 0


def validate_args(args: argparse.Namespace) -> None:
    if args.start <= 0:
        raise SystemExit("--start must be greater than zero")
    if args.max_batches <= 0:
        raise SystemExit("--max-batches must be greater than zero")
    if args.roots_per_batch <= 0:
        raise SystemExit("--roots-per-batch must be greater than zero")
    if args.feedback_loop <= 0:
        raise SystemExit("--feedback-loop must be greater than zero")
    if args.feedback_timeout < 0:
        raise SystemExit("--feedback-timeout must be zero or greater")
    if args.case_timeout <= 0:
        raise SystemExit("--case-timeout must be greater than zero")
    if args.allow_baseline_failures:
        args.baseline_check = True
    if args.validation == "production":
        args.baseline_check = True
        args.deny_warnings = True


def resolve_cargo_source(source: Path) -> CargoSource:
    source = source.resolve()
    manifest = direct_manifest_candidate(source)
    if manifest is not None:
        metadata = load_metadata_for_manifest(manifest)
        return cargo_source_from_metadata(source, manifest, metadata)

    if not source.is_dir():
        raise SystemExit(f"--source must be a Cargo.toml or directory: {source}")

    resolved = []
    failures: list[str] = []
    for candidate in discover_manifest_candidates(source):
        result = cargo_metadata(candidate, timeout_seconds=45)
        if not result.success:
            reason = first_nonempty_line(result.stderr) or str(result.exit_code)
            failures.append(f"{candidate}: {reason}")
            continue
        metadata = json.loads(result.stdout)
        resolved.append(cargo_source_from_metadata(source, candidate, metadata))

    by_root: dict[Path, CargoSource] = {}
    for candidate in resolved:
        by_root.setdefault(candidate.cargo_root, candidate)

    if len(by_root) == 1:
        return next(iter(by_root.values()))
    if not by_root:
        searched = "\n  ".join(str(path) for path in discover_manifest_candidates(source)[:12])
        failure_text = "\n  ".join(failures[:12])
        detail = failure_text or searched or "no Cargo.toml candidates found"
        raise SystemExit(
            f"could not resolve a Cargo workspace under {source}; pass --source path/to/Cargo.toml\n  {detail}"
        )

    choices = "\n  ".join(str(candidate.manifest_path) for candidate in by_root.values())
    raise SystemExit(
        f"multiple Cargo workspaces found under {source}; pass --source path/to/Cargo.toml\n  {choices}"
    )


def direct_manifest_candidate(source: Path) -> Path | None:
    if source.is_file() and source.name == "Cargo.toml":
        return source
    manifest = source / "Cargo.toml"
    if manifest.exists():
        return manifest
    return None


def discover_manifest_candidates(source: Path) -> list[Path]:
    ignored_dirs = {
        ".git",
        ".hg",
        ".svn",
        "target",
        "node_modules",
        "__pycache__",
    }
    candidates: list[Path] = []
    for root, dirs, files in os.walk(source):
        dirs[:] = [
            directory
            for directory in dirs
            if directory not in ignored_dirs
            and not directory.startswith(".")
            and not directory.startswith("slicers-corpus-")
        ]
        if "Cargo.toml" in files:
            candidates.append(Path(root) / "Cargo.toml")
    candidates.sort()
    return candidates


def load_metadata_for_manifest(manifest_path: Path) -> dict[str, Any]:
    result = cargo_metadata(manifest_path, timeout_seconds=120)
    if not result.success:
        reason = first_nonempty_line(result.stderr) or str(result.exit_code)
        raise SystemExit(f"cargo metadata failed for {manifest_path}: {reason}")
    return json.loads(result.stdout)


def cargo_metadata(manifest_path: Path, timeout_seconds: int) -> CommandResult:
    return run_command(
        [
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest_path),
        ],
        manifest_path.parent,
        timeout_seconds=timeout_seconds,
    )


def cargo_source_from_metadata(
    user_source: Path,
    manifest_path: Path,
    metadata: dict[str, Any],
) -> CargoSource:
    workspace_root = Path(metadata.get("workspace_root") or manifest_path.parent).resolve()
    workspace_manifest = workspace_root / "Cargo.toml"
    if workspace_manifest.exists() and workspace_manifest != manifest_path:
        metadata = load_metadata_for_manifest(workspace_manifest)
        manifest_path = workspace_manifest
    return CargoSource(
        user_source=user_source,
        manifest_path=manifest_path.resolve(),
        cargo_root=workspace_root,
        metadata=metadata,
    )


def run_batch(
    args: argparse.Namespace,
    repo: Path,
    cargo_source: CargoSource,
    opensourced_path: Path,
    rng: random.Random,
    batch: int,
) -> dict[str, Any]:
    started = time.monotonic()
    output_root = args.output_prefix.resolve().with_name(f"{args.output_prefix.name}-{batch}")
    slice_report_path = output_root / "slice-report.json"
    preflight_report_path = output_root / "slice-preflight.json"
    feedback_report_path = output_root / "slice-feedback.json"
    repair_report_path = output_root / "slice-repair.json"
    validation_report_path = output_root / "slice-validation.json"

    baseline = None
    roots: list[Candidate] = []
    candidate_counts: dict[str, int] = {}
    command_result: CommandResult | None = None

    try:
        if not args.no_git_restore:
            restore_source(cargo_source.cargo_root)

        packages = load_packages(cargo_source)
        candidates = discover_candidates(packages, args.public_only)
        candidate_counts = count_candidates(candidates)
        roots = select_roots(candidates, args.kinds, args.roots_per_batch, rng)
        cargo_check_args = validation_cargo_check_args(args, roots)

        if args.baseline_check:
            baseline = run_baseline_check(
                cargo_source.manifest_path,
                args.case_timeout,
                cargo_check_args,
            )
            if not baseline["success"] and not args.allow_baseline_failures:
                return build_row(
                    args,
                    cargo_source,
                    output_root,
                    batch,
                    started,
                    roots,
                    candidate_counts,
                    baseline,
                    command_result,
                    None,
                    None,
                    None,
                    None,
                    None,
                    "baseline_failed",
                )

        uses_workspace_dependency = source_uses_workspace_dependencies(cargo_source.cargo_root)
        inject_opensourced_dependency(
            cargo_source.cargo_root,
            opensourced_path,
            roots,
            uses_workspace_dependency,
        )
        annotate(roots)

        command = slicers_command(
            args,
            cargo_source.manifest_path,
            output_root,
            slice_report_path,
            preflight_report_path,
            feedback_report_path,
            repair_report_path,
            validation_report_path,
            cargo_check_args,
        )
        command_result = run_command(command, repo, args.case_timeout)

        generation = read_json(slice_report_path)
        preflight = read_json(preflight_report_path)
        feedback = read_json(feedback_report_path)
        repair = read_json(repair_report_path)
        validation_report = read_json(validation_report_path)
        classification = classify(command_result, preflight, feedback, baseline, args)
        return build_row(
            args,
            cargo_source,
            output_root,
            batch,
            started,
            roots,
            candidate_counts,
            baseline,
            command_result,
            generation,
            preflight,
            feedback,
            repair,
            validation_report,
            classification,
        )
    except Exception as error:
        return build_row(
            args,
            cargo_source,
            output_root,
            batch,
            started,
            roots,
            candidate_counts,
            baseline,
            command_result,
            None,
            None,
            None,
            None,
            None,
            "slice_generation_failed",
            error=str(error),
        )
    finally:
        shutil.rmtree(output_root / "target-feedback", ignore_errors=True)
        if not args.no_git_restore:
            restore_source(cargo_source.cargo_root)


def load_packages(cargo_source: CargoSource) -> list[Package]:
    workspace_members = set(cargo_source.metadata.get("workspace_members", []))
    packages = []
    for package in cargo_source.metadata.get("packages", []):
        if package.get("id") not in workspace_members:
            continue
        manifest_path = Path(package["manifest_path"]).resolve()
        root = manifest_path.parent
        for target in package.get("targets", []):
            kinds = tuple(str(kind) for kind in target.get("kind", []))
            if not (set(kinds) & PARSE_TARGET_KINDS):
                continue
            entry = Path(target["src_path"]).resolve()
            if entry.exists():
                packages.append(
                    Package(
                        package["name"],
                        root,
                        entry,
                        str(target.get("name") or package["name"]),
                        kinds,
                    )
                )

    packages.sort(
        key=lambda package: (
            package.name,
            package.target_kinds,
            package.target_name,
            str(package.entry),
        )
    )
    if not packages:
        raise RuntimeError(
            f"no local parse-candidate package targets found under {cargo_source.manifest_path}"
        )
    return packages


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


def count_candidates(candidates: list[Candidate]) -> dict[str, int]:
    counts = {kind: 0 for kind in ITEM_RE}
    for candidate in candidates:
        counts[candidate.kind] += 1
    return counts


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
        raise RuntimeError(
            f"not enough candidates for {roots_per_batch} roots; counts={count_candidates(candidates)}"
        )
    return selected


def source_uses_workspace_dependencies(source: Path) -> bool:
    text = (source / "Cargo.toml").read_text(errors="ignore")
    return re.search(r"(?m)^\s*\[workspace(?:\.dependencies)?\]\s*$", text) is not None


def inject_opensourced_dependency(
    source: Path,
    opensourced_path: Path,
    selected: list[Candidate],
    uses_workspace_dependency: bool,
) -> None:
    package_roots = {candidate.package.root for candidate in selected}
    if uses_workspace_dependency:
        inject_workspace_dependency(source / "Cargo.toml", opensourced_path)
        for package_root in package_roots:
            inject_package_dependency(
                package_root / "Cargo.toml",
                "opensourced = { workspace = true }",
            )
    else:
        for package_root in package_roots:
            inject_package_dependency(
                package_root / "Cargo.toml",
                f'opensourced = {{ path = "{toml_path(opensourced_path)}" }}',
            )


def inject_workspace_dependency(manifest_path: Path, opensourced_path: Path) -> None:
    line = f'opensourced = {{ path = "{toml_path(opensourced_path)}" }}'
    text = manifest_path.read_text()
    if re.search(r"(?m)^opensourced\s*=", text):
        return
    manifest_path.write_text(insert_into_section(text, "[workspace.dependencies]", line))


def inject_package_dependency(manifest_path: Path, dependency_line: str) -> None:
    text = manifest_path.read_text()
    if re.search(r"(?m)^opensourced\s*=", text):
        return
    manifest_path.write_text(insert_into_section(text, "[dependencies]", dependency_line))


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
    lines.insert(insert_at, line)
    return "\n".join(lines) + "\n"


def toml_path(path: Path) -> str:
    return str(path).replace("\\", "\\\\").replace('"', '\\"')


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


def slicers_command(
    args: argparse.Namespace,
    manifest_path: Path,
    output_root: Path,
    slice_report_path: Path,
    preflight_report_path: Path,
    feedback_report_path: Path,
    repair_report_path: Path,
    validation_report_path: Path,
    cargo_check_args: list[str],
) -> list[str]:
    features = list(args.features)
    if args.analyzer == "ra-hir" and "ra-hir" not in features:
        features.append("ra-hir")

    command = ["cargo", "run", "-p", "opensource_cli", "--bin", "slicers"]
    if features:
        command.extend(["--features", ",".join(features)])
    command.extend(
        [
            "--",
            "--analyzer",
            args.analyzer,
            "--slice-report",
            str(slice_report_path),
            "--preflight-report",
            str(preflight_report_path),
            "--validation-report",
            str(validation_report_path),
        ]
    )
    if args.validation == "preflight":
        command.append("--preflight")
    elif args.validation == "production":
        command.extend(
            [
                "--production",
                "--feedback-repair-loop",
                str(args.feedback_loop),
                "--feedback-timeout",
                str(args.feedback_timeout),
                "--feedback-report",
                str(feedback_report_path),
                "--repair-report",
                str(repair_report_path),
            ]
        )
    elif args.validation == "repair":
        command.extend(
            [
                "--feedback-repair-loop",
                str(args.feedback_loop),
                "--feedback-timeout",
                str(args.feedback_timeout),
                "--feedback-report",
                str(feedback_report_path),
                "--repair-report",
                str(repair_report_path),
            ]
        )
    else:
        command.extend(
            [
                "--feedback-loop",
                str(args.feedback_loop),
                "--feedback-timeout",
                str(args.feedback_timeout),
                "--feedback-report",
                str(feedback_report_path),
            ]
        )
    if args.validation != "preflight" and args.feedback_target_dir:
        command.extend(
            [
                "--feedback-target-dir",
                str(args.feedback_target_dir.resolve()),
            ]
        )
    if args.validation != "preflight":
        for cargo_arg in cargo_check_args:
            command.extend(["--cargo-check-arg", cargo_arg])
    if args.validation != "preflight" and args.deny_warnings:
        command.append("--deny-warnings")
    command.extend(
        [
            str(manifest_path),
            str(output_root),
        ]
    )
    return command


def validation_cargo_check_args(
    args: argparse.Namespace,
    roots: list[Candidate],
) -> list[str]:
    cargo_args = list(args.cargo_check_arg)
    if args.validation == "preflight":
        return cargo_args
    selected_dev_kinds = selected_root_dev_kinds(roots)
    if selected_dev_kinds and not cargo_args_cover_dev_targets(cargo_args, selected_dev_kinds):
        cargo_args.append("--all-targets")
    return cargo_args


def selected_root_dev_kinds(roots: list[Candidate]) -> set[str]:
    kinds: set[str] = set()
    for candidate in roots:
        kinds.update(set(candidate.package.target_kinds) & DEV_TARGET_KINDS)
    return kinds


def cargo_args_cover_dev_targets(cargo_args: list[str], selected_kinds: set[str]) -> bool:
    if "--all-targets" in cargo_args:
        return True
    coverage = {
        "example": any(
            arg == "--examples" or arg == "--example" or arg.startswith("--example=")
            for arg in cargo_args
        ),
        "test": any(
            arg == "--tests" or arg == "--test" or arg.startswith("--test=")
            for arg in cargo_args
        ),
        "bench": any(
            arg == "--benches" or arg == "--bench" or arg.startswith("--bench=")
            for arg in cargo_args
        ),
    }
    return all(coverage[kind] for kind in selected_kinds)


def run_baseline_check(
    manifest_path: Path,
    timeout_seconds: int,
    cargo_check_args: list[str],
) -> dict[str, Any]:
    result = run_command(
        [
            "cargo",
            "check",
            "--manifest-path",
            str(manifest_path),
            "--message-format=json",
            *cargo_check_args,
        ],
        manifest_path.parent,
        timeout_seconds=timeout_seconds,
    )
    diagnostics = parse_cargo_diagnostics(result.stdout)
    return {
        "success": result.success,
        "timed_out": result.timed_out,
        "exit_code": result.exit_code,
        "duration_ms": result.duration_ms,
        "errors": count_diagnostics(diagnostics, "error"),
        "warnings": count_diagnostics(diagnostics, "warning"),
        "semantic_warnings": semantic_hazard_warning_count(diagnostics, None),
        "diagnostic_error_keys": diagnostic_error_keys(diagnostics),
        "diagnostic_semantic_warning_keys": semantic_hazard_warning_keys(diagnostics),
        "first_diagnostics": summarize_diagnostics(diagnostics),
    }


def run_command(command: list[str], cwd: Path, timeout_seconds: int) -> CommandResult:
    started = time.monotonic()
    process = subprocess.Popen(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=(os.name != "nt"),
    )
    timed_out = False
    try:
        stdout, stderr = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        timed_out = True
        kill_process_tree(process)
        stdout, stderr = process.communicate()

    duration_ms = int((time.monotonic() - started) * 1000)
    exit_code = process.returncode if process.returncode is not None else -1
    return CommandResult(command, cwd, exit_code, timed_out, duration_ms, stdout, stderr)


def kill_process_tree(process: subprocess.Popen[str]) -> None:
    if os.name != "nt":
        try:
            os.killpg(process.pid, signal.SIGKILL)
            return
        except ProcessLookupError:
            return
        except OSError:
            pass
    process.kill()


def classify(
    command_result: CommandResult | None,
    preflight: dict[str, Any] | None,
    feedback: dict[str, Any] | None,
    baseline: dict[str, Any] | None,
    args: argparse.Namespace,
) -> str:
    if command_result is None or command_result.timed_out:
        return "slice_generation_failed"
    if preflight is not None and not preflight.get("success"):
        return "slice_preflight_failed"
    if args.validation == "preflight":
        if command_result.exit_code == 0 and preflight and preflight.get("success"):
            return "slice_preflight_passed"
        return "slice_generation_failed"
    if feedback is None:
        return "slice_generation_failed"
    errors = count_diagnostics(feedback.get("diagnostics", []), "error")
    warnings = count_diagnostics(feedback.get("diagnostics", []), "warning")
    semantic_warnings = semantic_hazard_warning_count(
        feedback.get("diagnostics", []), baseline
    )
    if command_result.exit_code != 0 or not feedback.get("success") or errors:
        if feedback_errors_are_baseline_known(feedback, baseline):
            if semantic_warnings:
                return "slice_semantic_warning_failed"
            if warnings and (args.deny_warnings or args.stop_on_warning):
                return "slice_feedback_failed"
            return "slice_baseline_limited_passed"
        return "slice_feedback_failed"
    if semantic_warnings:
        return "slice_semantic_warning_failed"
    if args.stop_on_warning and warnings:
        return "slice_feedback_failed"
    return "slice_check_passed"


def build_row(
    args: argparse.Namespace,
    cargo_source: CargoSource,
    output_root: Path,
    batch: int,
    started: float,
    roots: list[Candidate],
    candidate_counts: dict[str, int],
    baseline: dict[str, Any] | None,
    command_result: CommandResult | None,
    generation: dict[str, Any] | None,
    preflight: dict[str, Any] | None,
    feedback: dict[str, Any] | None,
    repair: dict[str, Any] | None,
    validation_report: dict[str, Any] | None,
    classification: str,
    error: str | None = None,
) -> dict[str, Any]:
    diagnostics = (feedback or {}).get("diagnostics", [])
    widening = (feedback or {}).get("widening") or {}
    widening_candidates = widening.get("candidates") or []
    feedback_hazards = widening.get("hazards") or []
    warnings = count_diagnostics(diagnostics, "warning")
    errors = count_diagnostics(diagnostics, "error")
    semantic_warnings = semantic_hazard_warning_count(diagnostics, baseline)
    preflight_diagnostics = (preflight or {}).get("diagnostics", [])
    production = (generation or {}).get("production") or {}
    production_hazards = production.get("hazards") or []
    validation_status = (validation_report or {}).get("status")
    validation_gates = validation_gate_statuses(validation_report)
    cargo_check_args = validation_cargo_check_args(args, roots)
    passed = classification in {
        "slice_check_passed",
        "slice_preflight_passed",
        "slice_baseline_limited_passed",
    }
    if validation_status:
        passed = validation_status == "accepted"
    if semantic_warnings or ((args.stop_on_warning or args.deny_warnings) and warnings):
        passed = False

    row: dict[str, Any] = {
        "iteration": batch,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source": str(cargo_source.user_source),
        "cargo_root": str(cargo_source.cargo_root),
        "cargo_manifest": str(cargo_source.manifest_path),
        "source_git_head": git_head(cargo_source.cargo_root),
        "seed": args.seed,
        "validation": {
            "tier": args.validation,
            "feedback_loop": args.feedback_loop,
            "feedback_timeout": args.feedback_timeout,
            "cargo_check_args": cargo_check_args,
            "user_cargo_check_args": args.cargo_check_arg,
            "allow_baseline_failures": args.allow_baseline_failures,
            "deny_warnings": args.deny_warnings,
            "stop_on_warning": args.stop_on_warning,
            "report_status": validation_status,
            "report_reason": (validation_report or {}).get("reason"),
            "report_gates": validation_gates,
            "production_ready_status": validation_gates.get("production_ready"),
            "report_attempts": len((validation_report or {}).get("attempts") or []),
        },
        "roots": [candidate.display(cargo_source.cargo_root) for candidate in roots],
        "candidate_counts": candidate_counts,
        "baseline": baseline,
        "slice": {
            "output": str(output_root),
            "command": command_result.command if command_result else None,
            "command_exit": command_result.exit_code if command_result else None,
            "timed_out": command_result.timed_out if command_result else False,
            "duration_ms": command_result.duration_ms if command_result else None,
            "files_written": (generation or {}).get("files_written"),
            "packages": (generation or {}).get("packages", []),
            "roots": (generation or {}).get("roots", []),
            "feedback_widened_roots": (generation or {}).get("feedback_widened_roots", []),
            "feedback_widened_root_count": len(
                (generation or {}).get("feedback_widened_roots", [])
            ),
            "reachable_callables": len((generation or {}).get("reachable", [])),
            "reachable_items": len((generation or {}).get("reachable_items", [])),
            "production_status": production.get("status"),
            "production_hazards": len(production_hazards),
            "production_hazard_codes": production_hazard_codes(production_hazards),
        },
        "preflight": {
            "success": (preflight or {}).get("success"),
            "errors": count_diagnostics(preflight_diagnostics, "error"),
            "warnings": count_diagnostics(preflight_diagnostics, "warning"),
            "packages": (preflight or {}).get("packages"),
            "rust_files": (preflight or {}).get("rust_files"),
            "local_path_dependencies": (preflight or {}).get("local_path_dependencies"),
            "external_dependencies": (preflight or {}).get("external_dependencies"),
            "build_scripts": (preflight or {}).get("build_scripts"),
            "diagnostic_codes": diagnostic_codes(preflight_diagnostics),
            "first_diagnostics": summarize_diagnostics(preflight_diagnostics),
        },
        "feedback": {
            "success": (feedback or {}).get("success"),
            "timed_out": (feedback or {}).get("timed_out"),
            "exit_code": (feedback or {}).get("exit_code"),
            "duration_ms": (feedback or {}).get("duration_ms"),
            "target_dir": feedback_target_dir(args, output_root),
            "errors": errors,
            "warnings": warnings,
            "semantic_warnings": semantic_warnings,
            "diagnostic_codes": diagnostic_codes(diagnostics),
            "widening_candidates": len(widening_candidates),
            "widening_candidate_kinds": keyed_values(widening_candidates, "kind"),
            "widening_candidate_suggestions": sum_ints(
                widening_candidates, "suggestions"
            ),
            "widening_candidate_machine_suggestions": sum_ints(
                widening_candidates, "machine_applicable_suggestions"
            ),
            "widening_hazards": len(feedback_hazards),
            "widening_hazard_kinds": keyed_values(feedback_hazards, "kind"),
            "suggestions": diagnostic_suggestion_count(diagnostics),
            "machine_applicable_suggestions": diagnostic_suggestion_count(
                diagnostics, applicability="MachineApplicable"
            ),
            "first_diagnostics": summarize_diagnostics(diagnostics),
        },
        "repair": {
            "removed_items": (repair or {}).get("removed_items"),
            "removed_imports": (repair or {}).get("removed_imports"),
            "normalized_paths": (repair or {}).get("normalized_paths"),
            "applied_suggestions": (repair or {}).get("applied_suggestions"),
            "added_dead_code_allows": (repair or {}).get("added_dead_code_allows"),
            "deferred_dead_code_allows": (repair or {}).get("deferred_dead_code_allows"),
            "skipped_diagnostics": (repair or {}).get("skipped_diagnostics"),
            "changed_files": len((repair or {}).get("changed_files") or []),
            "total_changes": repair_total_changes(repair),
        },
        "classification": classification,
        "passed": passed,
        "elapsed_ms": int((time.monotonic() - started) * 1000),
    }
    if error:
        row["error"] = error
    if command_result and not passed:
        row["stderr_tail"] = command_result.stderr[-4000:]
        row["stdout_tail"] = command_result.stdout[-4000:]
    return row


def parse_cargo_diagnostics(stdout: str) -> list[dict[str, Any]]:
    diagnostics = []
    for line in stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-message":
            continue
        diagnostic = message.get("message")
        if isinstance(diagnostic, dict):
            diagnostic = dict(diagnostic)
            diagnostic["package_id"] = message.get("package_id")
            diagnostic["target"] = message.get("target")
            diagnostics.append(diagnostic)
    return diagnostics


def count_diagnostics(diagnostics: list[dict[str, Any]], level: str) -> int:
    return sum(1 for diagnostic in diagnostics if diagnostic.get("level") == level)


def diagnostic_codes(diagnostics: list[dict[str, Any]]) -> list[str]:
    codes = []
    for diagnostic in diagnostics:
        code = diagnostic.get("code")
        if isinstance(code, dict):
            code = code.get("code")
        if code and code not in codes:
            codes.append(str(code))
    return codes


def diagnostic_error_keys(diagnostics: list[dict[str, Any]]) -> list[str]:
    keys: list[str] = []
    for diagnostic in diagnostics:
        if diagnostic.get("level") != "error":
            continue
        key = diagnostic_key(diagnostic)
        if key not in keys:
            keys.append(key)
    return keys


def diagnostic_key(diagnostic: dict[str, Any]) -> str:
    code = diagnostic.get("code")
    if isinstance(code, dict):
        code = code.get("code")
    primary_files = diagnostic_primary_files(diagnostic)
    rendered_fingerprint = (
        diagnostic_rendered_fingerprint(diagnostic) if not primary_files else ""
    )
    return "|".join(
        [
            str(diagnostic.get("level") or ""),
            str(code or ""),
            str(diagnostic.get("package_id") or ""),
            diagnostic_target_key(diagnostic.get("target")),
            ",".join(primary_files),
            rendered_fingerprint,
            str(diagnostic.get("message") or ""),
        ]
    )


def diagnostic_primary_files(diagnostic: dict[str, Any]) -> list[str]:
    files = {
        str(span.get("file_name"))
        for span in diagnostic.get("spans", [])
        if isinstance(span, dict) and span.get("is_primary") and span.get("file_name")
    }
    return sorted(files)


def diagnostic_rendered_fingerprint(diagnostic: dict[str, Any]) -> str:
    rendered = diagnostic.get("rendered")
    if not isinstance(rendered, str) or not rendered:
        return ""
    return hashlib.sha256(rendered.encode("utf-8")).hexdigest()[:16]


def diagnostic_target_key(target: Any) -> str:
    if not isinstance(target, dict):
        return ""
    kind = target.get("kind") or []
    if isinstance(kind, list):
        kind_text = ",".join(str(value) for value in kind)
    else:
        kind_text = str(kind)
    return f"{kind_text}:{target.get('name') or ''}"


def semantic_hazard_warning_count(
    diagnostics: list[dict[str, Any]],
    baseline: dict[str, Any] | None,
) -> int:
    baseline_keys = set((baseline or {}).get("diagnostic_semantic_warning_keys") or [])
    return sum(
        1
        for key in semantic_hazard_warning_keys(diagnostics)
        if key not in baseline_keys
    )


def semantic_hazard_warning_keys(diagnostics: list[dict[str, Any]]) -> list[str]:
    keys: list[str] = []
    for diagnostic in diagnostics:
        if not semantic_warning_is_hazard(diagnostic):
            continue
        key = diagnostic_key(diagnostic)
        if key not in keys:
            keys.append(key)
    return keys


def semantic_warning_is_hazard(diagnostic: dict[str, Any]) -> bool:
    if diagnostic.get("level") != "warning":
        return False
    code = diagnostic.get("code")
    if isinstance(code, dict):
        code = code.get("code")
    if code in {
        "unreachable_patterns",
        "irrefutable_let_patterns",
        "bindings_with_variant_name",
    }:
        return True
    message = str(diagnostic.get("message") or "")
    if code == "non_snake_case" and "variable `" in message:
        return True
    return "unreachable pattern" in message or "irrefutable" in message


def validation_gate_statuses(report: dict[str, Any] | None) -> dict[str, str]:
    gates = (report or {}).get("gates") or []
    statuses: dict[str, str] = {}
    for gate in gates:
        name = gate.get("name")
        status = gate.get("status")
        if name and status:
            statuses[str(name)] = str(status)
    return statuses


def production_hazard_codes(hazards: list[dict[str, Any]]) -> list[str]:
    codes: list[str] = []
    for hazard in hazards:
        code = hazard.get("code")
        if isinstance(code, str) and code not in codes:
            codes.append(code)
    return codes


def keyed_values(items: list[dict[str, Any]], key: str) -> list[str]:
    values: list[str] = []
    for item in items:
        value = item.get(key)
        if isinstance(value, str) and value not in values:
            values.append(value)
    return values


def sum_ints(items: list[dict[str, Any]], key: str) -> int:
    total = 0
    for item in items:
        value = item.get(key)
        if isinstance(value, int):
            total += value
    return total


def diagnostic_suggestion_count(
    diagnostics: list[dict[str, Any]], applicability: str | None = None
) -> int:
    count = 0
    for diagnostic in diagnostics:
        for suggestion in diagnostic.get("suggestions") or []:
            if applicability is None or suggestion.get("suggestion_applicability") == applicability:
                count += 1
    return count


def feedback_errors_are_baseline_known(
    feedback: dict[str, Any] | None,
    baseline: dict[str, Any] | None,
) -> bool:
    if not baseline or baseline.get("success"):
        return False
    baseline_keys = set(baseline.get("diagnostic_error_keys") or [])
    if not baseline_keys:
        return False
    feedback_keys = diagnostic_error_keys((feedback or {}).get("diagnostics", []))
    return bool(feedback_keys) and all(key in baseline_keys for key in feedback_keys)


def repair_total_changes(repair: dict[str, Any] | None) -> int | None:
    if repair is None:
        return None
    return (
        int(repair.get("removed_items") or 0)
        + int(repair.get("removed_imports") or 0)
        + int(repair.get("normalized_paths") or 0)
        + int(repair.get("applied_suggestions") or 0)
        + int(repair.get("added_dead_code_allows") or 0)
    )


def feedback_target_dir(args: argparse.Namespace, output_root: Path) -> str:
    if args.feedback_target_dir:
        return str(args.feedback_target_dir.resolve())
    return str(output_root / "target-feedback")


def summarize_diagnostics(
    diagnostics: list[dict[str, Any]],
    limit: int = 12,
) -> list[dict[str, Any]]:
    prioritized = [
        diagnostic
        for diagnostic in diagnostics
        if diagnostic.get("level") in {"error", "warning"}
    ]
    prioritized.sort(key=lambda diagnostic: 0 if diagnostic.get("level") == "error" else 1)
    summaries = []
    for diagnostic in prioritized[:limit]:
        span = next(
            (span for span in diagnostic.get("spans", []) if span.get("is_primary")),
            None,
        )
        summaries.append(
            {
                "level": diagnostic.get("level"),
                "code": diagnostic.get("code"),
                "message": diagnostic.get("message"),
                "file": span.get("file_name") if span else None,
                "line": span.get("line_start") if span else None,
                "column": span.get("column_start") if span else None,
            }
        )
    return summaries


def read_json(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return json.loads(path.read_text())


def append_jsonl(path: Path, row: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(row, sort_keys=True) + "\n")


def print_batch_summary(row: dict[str, Any]) -> None:
    feedback = row["feedback"]
    preflight = row["preflight"]
    repair = row["repair"]
    diagnostic_block = feedback if feedback["first_diagnostics"] else preflight
    repair_text = ""
    if repair["total_changes"] is not None:
        repair_text = (
            f"repair_changes={repair['total_changes']} "
            f"suggestions={repair.get('applied_suggestions')} "
        )
    readiness_text = ""
    if row["validation"].get("production_ready_status"):
        readiness_text = f"production_ready={row['validation']['production_ready_status']} "
    print(
        "corpus batch "
        f"{row['iteration']}: {row['classification']} "
        f"passed={row['passed']} "
        f"files={row['slice']['files_written']} "
        f"preflight_errors={preflight['errors']} "
        f"feedback_errors={feedback['errors']} warnings={feedback['warnings']} "
        f"{repair_text}"
        f"{readiness_text}"
        f"output={row['slice']['output']}",
        flush=True,
    )
    for diagnostic in diagnostic_block["first_diagnostics"][:5]:
        code = diagnostic.get("code")
        code_text = f"[{code}]" if code else ""
        location = ""
        if diagnostic.get("file"):
            location = f" at {diagnostic['file']}:{diagnostic['line']}:{diagnostic['column']}"
        print(
            f"  {diagnostic.get('level')}{code_text}: {diagnostic.get('message')}{location}",
            flush=True,
        )


def restore_source(source: Path) -> None:
    git_context = source.parent if source.is_file() else source
    git_root = subprocess.run(
        ["git", "-C", str(git_context), "rev-parse", "--show-toplevel"],
        check=False,
        capture_output=True,
        text=True,
    )
    if git_root.returncode != 0:
        return
    root = git_root.stdout.strip()
    subprocess.run(["git", "-C", root, "restore", "."], check=True)
    subprocess.run(["git", "-C", root, "clean", "-fd"], check=True, stdout=subprocess.DEVNULL)


def git_head(source: Path) -> str | None:
    git_context = source.parent if source.is_file() else source
    result = subprocess.run(
        ["git", "-C", str(git_context), "rev-parse", "--short", "HEAD"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def first_nonempty_line(text: str) -> str | None:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped
    return None


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

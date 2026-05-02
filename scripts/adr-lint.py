#!/usr/bin/env -S uv run --quiet --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""
adr-lint — validate the integrity of architectural decision records.

Checks performed:
  1. Filename format: NNNN-slug.md (four-digit zero-padded prefix)
  2. Numbering: no gaps, no duplicates, starts at 0001
  3. Status: every ADR has a Status line with an allowed value
  4. Supersession: 'superseded by ADR-NNNN' references resolve to a real ADR
  5. Index sync: every ADR file has an entry in docs/adrs/README.md, and
     every entry in the README points to a real file

Exit codes:
  0  all checks passed
  1  one or more checks failed
  2  invocation error (missing directory, etc.)
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ALLOWED_STATUSES = {"proposed", "accepted", "deprecated"}
SUPERSEDED_PATTERN = re.compile(r"^superseded by ADR-(\d{4})\b", re.IGNORECASE)
FILENAME_PATTERN = re.compile(r"^(\d{4})-([a-z0-9]+(?:-[a-z0-9]+)*)\.md$")
STATUS_LINE_PATTERN = re.compile(r"^\*\*Status:\*\*\s*(.+?)\s*$", re.MULTILINE)
ADR_LINK_PATTERN = re.compile(r"\[(?:ADR-)?(\d{4})[^\]]*\]\((\d{4}-[^)]+\.md)\)")


@dataclass
class ADR:
    path: Path
    number: int
    slug: str
    status: str | None
    superseded_by: int | None


@dataclass
class Finding:
    level: str  # "error" or "warning"
    message: str

    def __str__(self) -> str:
        prefix = "ERROR" if self.level == "error" else "WARN "
        return f"  [{prefix}] {self.message}"


def parse_adr(path: Path) -> tuple[ADR | None, list[Finding]]:
    findings: list[Finding] = []
    match = FILENAME_PATTERN.match(path.name)
    if not match:
        findings.append(
            Finding("error", f"{path.name}: filename does not match NNNN-slug.md")
        )
        return None, findings

    number = int(match.group(1))
    slug = match.group(2)

    try:
        content = path.read_text(encoding="utf-8")
    except OSError as e:
        findings.append(Finding("error", f"{path.name}: cannot read file: {e}"))
        return None, findings

    status_match = STATUS_LINE_PATTERN.search(content)
    if not status_match:
        findings.append(Finding("error", f"{path.name}: missing **Status:** line"))
        return ADR(path, number, slug, None, None), findings

    status_raw = status_match.group(1).strip().lower()
    superseded_by: int | None = None

    if status_raw in ALLOWED_STATUSES:
        status = status_raw
    elif sup_match := SUPERSEDED_PATTERN.match(status_raw):
        status = "superseded"
        superseded_by = int(sup_match.group(1))
    else:
        findings.append(
            Finding(
                "error",
                f"{path.name}: status '{status_match.group(1)}' is not "
                f"one of: {sorted(ALLOWED_STATUSES)} or 'superseded by ADR-NNNN'",
            )
        )
        status = None

    return ADR(path, number, slug, status, superseded_by), findings


def check_numbering(adrs: list[ADR]) -> list[Finding]:
    findings: list[Finding] = []
    if not adrs:
        return findings

    numbers = sorted(adr.number for adr in adrs)

    seen: dict[int, list[Path]] = {}
    for adr in adrs:
        seen.setdefault(adr.number, []).append(adr.path)
    for num, paths in seen.items():
        if len(paths) > 1:
            names = ", ".join(p.name for p in paths)
            findings.append(
                Finding("error", f"ADR-{num:04d} appears in multiple files: {names}")
            )

    expected = list(range(1, max(numbers) + 1))
    missing = sorted(set(expected) - set(numbers))
    for num in missing:
        findings.append(
            Finding("error", f"ADR-{num:04d} is missing (gap in numbering)")
        )

    if 0 in numbers:
        findings.append(Finding("error", "ADR numbering must start at 0001, not 0000"))

    return findings


def check_supersession(adrs: list[ADR]) -> list[Finding]:
    findings: list[Finding] = []
    by_number = {adr.number: adr for adr in adrs}
    for adr in adrs:
        if adr.superseded_by is None:
            continue
        target = by_number.get(adr.superseded_by)
        if target is None:
            findings.append(
                Finding(
                    "error",
                    f"{adr.path.name}: superseded by ADR-{adr.superseded_by:04d} "
                    f"but that ADR does not exist",
                )
            )
        elif target.number == adr.number:
            findings.append(
                Finding("error", f"{adr.path.name}: cannot supersede itself")
            )
    return findings


def check_index_sync(adrs: list[ADR], readme_path: Path) -> list[Finding]:
    findings: list[Finding] = []
    if not readme_path.exists():
        findings.append(Finding("error", f"{readme_path} does not exist"))
        return findings

    content = readme_path.read_text(encoding="utf-8")
    referenced_files: set[str] = set()
    for match in ADR_LINK_PATTERN.finditer(content):
        referenced_files.add(match.group(2))

    actual_files = {adr.path.name for adr in adrs}
    in_dir_not_index = actual_files - referenced_files
    in_index_not_dir = referenced_files - actual_files

    for name in sorted(in_dir_not_index):
        findings.append(Finding("warning", f"{name} is not listed in {readme_path}"))
    for name in sorted(in_index_not_dir):
        findings.append(
            Finding("error", f"index references {name} but file does not exist")
        )

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    parser.add_argument(
        "--adr-dir",
        default="docs/adrs",
        help="ADR directory (default: docs/adrs)",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as errors",
    )
    args = parser.parse_args()

    adr_dir = Path(args.adr_dir)
    if not adr_dir.is_dir():
        print(f"adr-lint: {adr_dir} is not a directory", file=sys.stderr)
        return 2

    files = sorted(
        p
        for p in adr_dir.iterdir()
        if p.is_file()
        and p.suffix == ".md"
        and p.name not in {"README.md", "template.md"}
    )

    all_findings: list[Finding] = []
    adrs: list[ADR] = []

    for path in files:
        adr, findings = parse_adr(path)
        all_findings.extend(findings)
        if adr is not None:
            adrs.append(adr)

    all_findings.extend(check_numbering(adrs))
    all_findings.extend(check_supersession(adrs))
    all_findings.extend(check_index_sync(adrs, adr_dir / "README.md"))

    errors = [f for f in all_findings if f.level == "error"]
    warnings = [f for f in all_findings if f.level == "warning"]

    if all_findings:
        print(f"adr-lint: scanned {len(adrs)} ADR(s) in {adr_dir}")
        for finding in all_findings:
            print(finding)
        print(f"\n{len(errors)} error(s), {len(warnings)} warning(s)")
    else:
        print(f"adr-lint: scanned {len(adrs)} ADR(s) in {adr_dir} — all checks passed")

    if errors or (args.strict and warnings):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

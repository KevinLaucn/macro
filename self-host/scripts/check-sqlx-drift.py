#!/usr/bin/env python3
"""Fail early if compile-time SQLx query macros were changed without updating .sqlx metadata.

In SQLX_OFFLINE=true builds (such as the Nix self-host email production build),
any invocation of compile-time SQLx macros (sqlx::query!, sqlx::query_as!, etc.)
requires pre-generated query metadata cached in .sqlx/ directories.

Running a full cargo check in CI takes ~9 minutes on a cold runner just to
catch missing .sqlx metadata. This script checks git diff in milliseconds and
fails fast before any heavy Nix or Cargo build starts.

Run: python3 self-host/scripts/check-sqlx-drift.py [--base <base>] [--head <head>]
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys

# Compile-time SQLx queries use the '!' macro invocation syntax
QUERY_MACRO_RE = re.compile(r"\b(?:sqlx::)?query(?:_as|_scalar|_file|_file_as)?\s*!")

def is_sqlx_path(path: str) -> bool:
    p = path.strip().replace("\\", "/")
    return p == ".sqlx" or p.startswith(".sqlx/") or "/.sqlx/" in p or p.endswith("/.sqlx")

def run_git(args: list[str]) -> tuple[int, str]:
    try:
        res = subprocess.run(
            ["git"] + args,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        return res.returncode, res.stdout.strip()
    except Exception as e:
        return 1, f"Git execution error: {e}"

def get_changed_files(base: str | None, head: str | None) -> list[str]:
    if base and head:
        code, out = run_git(["diff", "--name-only", base, head])
    elif base:
        code, out = run_git(["diff", "--name-only", base])
    else:
        code, out = run_git(["diff", "--name-only", "HEAD"])
        if code != 0 or not out:
            code, out = run_git(["diff", "--name-only", "HEAD~1", "HEAD"])
    if code != 0:
        return []
    return [line.strip() for line in out.splitlines() if line.strip()]

def get_rs_diff(base: str | None, head: str | None) -> str:
    diff_args = ["diff", "-U0"]
    if base and head:
        diff_args.extend([base, head])
    elif base:
        diff_args.append(base)
    else:
        diff_args.append("HEAD")

    diff_args.extend(["--", "*.rs"])
    code, out = run_git(diff_args)
    if code != 0:
        if not base and not head:
            code, out = run_git(["diff", "-U0", "HEAD~1", "HEAD", "--", "*.rs"])
    return out if code == 0 else ""

def check_sqlx_drift(base: str | None, head: str | None) -> int:
    if base and (base.strip() == "" or set(base.strip()) == {"0"}):
        if head:
            code, root_commit = run_git(["rev-list", "--max-parents=0", head])
            if code == 0 and root_commit:
                base = root_commit.splitlines()[-1]
            else:
                base = None
        else:
            base = None

    changed_files = get_changed_files(base, head)
    if not changed_files:
        print("check-sqlx-drift: No changed files detected in git diff; skipping check.")
        return 0

    has_sqlx_metadata = any(is_sqlx_path(f) for f in changed_files)
    migration_changes = [f for f in changed_files if "migrations/" in f.replace("\\", "/")]

    if migration_changes:
        print(f"check-sqlx-drift: Detected {len(migration_changes)} migration file change(s).")

    diff_text = get_rs_diff(base, head)
    if not diff_text:
        print("check-sqlx-drift: No Rust code changes in diff; passed.")
        return 0

    offending_lines: list[tuple[str, str]] = []
    current_file = "unknown"

    for line in diff_text.splitlines():
        if line.startswith("+++ b/"):
            current_file = line[6:].strip()
            continue
        elif line.startswith("+++"):
            continue
        if not line.startswith("+"):
            continue

        added_content = line[1:].strip()
        if not added_content:
            continue

        if added_content.startswith("//") or added_content.startswith("/*") or added_content.startswith("*"):
            continue

        code_without_comment = added_content.split("//", 1)[0]
        if QUERY_MACRO_RE.search(code_without_comment):
            offending_lines.append((current_file, added_content))

    if offending_lines:
        print(f"check-sqlx-drift: Found {len(offending_lines)} added/modified compile-time SQLx macro query call(s).")
        if has_sqlx_metadata:
            print("check-sqlx-drift: OK - .sqlx query metadata is included in the changed files.")
            return 0
        else:
            print("\n" + "=" * 70, file=sys.stderr)
            print("ERROR: Compile-time SQLx query changed but .sqlx metadata was not updated.", file=sys.stderr)
            print("=" * 70, file=sys.stderr)
            print("Offending changes without matching .sqlx metadata:", file=sys.stderr)
            for file_path, code in offending_lines[:10]:
                print(f"  {file_path}: {code}", file=sys.stderr)
            if len(offending_lines) > 10:
                print(f"  ... and {len(offending_lines) - 10} more", file=sys.stderr)
            print("\nHow to fix:", file=sys.stderr)
            print("  1. Run the repository SQLx prepare recipe inside nix develop:", file=sys.stderr)
            print("     nix develop --command just prepare_db", file=sys.stderr)
            print("  2. Commit the updated .sqlx/ directory alongside your Rust changes.", file=sys.stderr)
            print("  Important: SQLX_OFFLINE=true production builds require compile-time SQL metadata.", file=sys.stderr)
            print("=" * 70 + "\n", file=sys.stderr)
            return 1

    print("check-sqlx-drift: No compile-time SQLx macro query additions detected; passed.")
    return 0

def main() -> None:
    parser = argparse.ArgumentParser(description="Check SQLx query macro drift against .sqlx metadata")
    parser.add_argument("--base", default=None, help="Base commit or ref")
    parser.add_argument("--head", default=None, help="Head commit or ref")
    args = parser.parse_args()

    exit_code = check_sqlx_drift(args.base, args.head)
    sys.exit(exit_code)

if __name__ == "__main__":
    main()

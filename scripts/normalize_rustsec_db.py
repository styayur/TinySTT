#!/usr/bin/env python3
"""Normalize RustSec advisory Markdown wrappers without dropping advisory data."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def normalize_database(root: Path) -> tuple[list[str], list[str]]:
    changed = []
    removed = []
    for path in sorted(root.glob('*.md')) + sorted((root / '.github').rglob('*.md')):
        path.unlink()
        removed.append(str(path.relative_to(root)))
    for path in sorted(root.rglob('*.md')):
        text = path.read_text()
        lines = text.splitlines()
        if lines and lines[0].strip() == '```toml':
            closing = next((i for i, line in enumerate(lines[1:], start=1) if line.strip() == '```'), None)
            if closing is not None:
                body = '\n'.join(lines[1:closing]).rstrip() + '\n'
                path.write_text(body)
                changed.append(str(path.relative_to(root)))
    return changed, removed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--db', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    changed, removed = normalize_database(args.db)
    report = {'schemaVersion': 1, 'normalizedFiles': changed, 'removedNonAdvisoryDocs': removed, 'count': len(changed), 'note': 'Exact Markdown code fences are removed from advisory files; top-level repository documentation is excluded from the advisory parser. Advisory TOML content is retained.'}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'normalized': len(changed), 'removedNonAdvisoryDocs': len(removed)}))


if __name__ == '__main__':
    main()

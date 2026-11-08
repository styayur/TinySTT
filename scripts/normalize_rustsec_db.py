#!/usr/bin/env python3
"""Normalize RustSec advisory Markdown wrappers without dropping advisory data."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def normalize_database(root: Path) -> list[str]:
    changed = []
    for path in sorted(root.rglob('*.md')):
        text = path.read_text()
        stripped = text.strip()
        if stripped.startswith('```toml') and stripped.endswith('```'):
            body = stripped.split('\n', 1)[1]
            body = body.rsplit('```', 1)[0].rstrip() + '\n'
            path.write_text(body)
            changed.append(str(path.relative_to(root)))
    return changed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--db', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    changed = normalize_database(args.db)
    report = {'schemaVersion': 1, 'normalizedFiles': changed, 'count': len(changed), 'note': 'Only exact Markdown code-fence wrappers around TOML advisory files are removed; advisory content is retained.'}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'normalized': len(changed)}))


if __name__ == '__main__':
    main()

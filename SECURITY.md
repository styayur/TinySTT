# Security Policy

## Reporting a vulnerability

Please do not open a public issue for a vulnerability that could expose user
audio, transcripts, arbitrary code execution, or local files.

Use GitHub private vulnerability reporting for this repository. Include:

- affected TinySTT version or commit;
- Windows version;
- reproduction steps;
- impact;
- any suggested mitigation.

## Scope

TinySTT is offline and has no server component. Reports involving local
microphone capture, clipboard handling, automatic Ctrl+V injection, Windows
DLL search paths, model-file parsing, or memory safety in Rust code are in
scope.

Reports about the content or accuracy of a third-party model are not product
security vulnerabilities; use the upstream model project for those issues.

## Privacy expectations

TinySTT should never upload audio or transcript data. Any network behavior at
runtime is considered a bug unless it is explicitly documented and configured
by the user.

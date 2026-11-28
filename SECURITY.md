# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| `0.1.x` | Yes |

## Reporting a Vulnerability

Please do **not** open a public GitHub issue for suspected security vulnerabilities (such as digest verification bypasses, parser memory exhaustion, or path/symlink traversal flaws).

Report vulnerabilities privately through [GitHub Security Advisories](https://github.com/david-engelmann/onrecord/security/advisories/new) on `david-engelmann/onrecord`, including:

1. A minimal reproduction command or `.jsonl` input triggering the issue.
2. Affected version or commit SHA and operating system.
3. Expected vs observed behavior and potential impact on record integrity.

Reports are acknowledged within 72 hours, and coordinated fixes are released with regression tests before public disclosure.

## Security Model Notes

- `onrecord check` recomputes every claim's Canonical Form 1 SHA-256 prefix against its 0-based line position and enforces strict container nesting caps (`9,994` total / `9,991` objects) and integer digit bounds (`4,300` digits) to prevent untrusted `.jsonl` files from exhausting stack or CPU budgets.
- Claim IDs verify structural integrity against accidental corruption or line tampering, not adversarial origin authenticity; sign record snapshots externally when untrusted actors have write access to storage.

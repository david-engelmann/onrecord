# Contributing to Onrecord

Thank you for contributing to `onrecord`.

## Development Prerequisites

- Rust stable toolchain (`1.74` or newer) with `rustfmt` and `clippy`
- Python `3.10+` (for `scripts/codepoint_scan.py` and JSON Schema checks)

## Local Gate

Every change must pass all four repository checks with zero warnings and zero failures before opening a pull request:

```console
$ python3 scripts/codepoint_scan.py
$ cargo fmt --check
$ cargo clippy --all-targets -- -D warnings
$ cargo test
```

If you edit or add shell scripts or JSON files, also verify:

```console
$ sh -n install.sh
$ python3 -m json.tool schema/claim.schema.json > /dev/null
$ python3 -m json.tool schema/views.schema.json > /dev/null
```

## Style and Repository Rules

- **Zero em-dashes or en-dashes**: Never introduce `U+2014` or `U+2013` in source files, tests, documentation, manifests, or workflows. `python3 scripts/codepoint_scan.py` enforces this rule in CI.
- **Determinism and behavioral pins**: `onrecord` pins exact CLI exit codes (`0`, `1`, `2`, `120`), stderr refusal phrasing, and Canonical Form 1 `claim-<16hex>` ID derivation (`tests/fixtures/tape_synthetic.json`). Any behavioral change requires explicit contract review and test coverage.
- **Minimal dependencies**: Keep the dependency footprint minimal (`flighttape` and `sha2`).

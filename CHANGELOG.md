# Changelog

All notable changes to `onrecord` are documented in this file.

The format follows Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

- Crate packaging metadata in `Cargo.toml` (`repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`, `rust-version = "1.78"`, and `exclude`).
- Least-privilege GitHub Actions permissions (`contents: read`) and non-persisted checkout credentials in `.github/workflows/ci.yml`.
- Tag-triggered multi-platform release workflow (`.github/workflows/release.yml`) with `SHA256SUMS` generation and POSIX `install.sh` verifier.
- Verified sample claim store (`examples/demo_record.jsonl`) demonstrating linear supersession, repeatable evidence, and subject conflict reporting.
- Draft 2020-12 JSON Schemas for stored claim lines (`schema/claim.schema.json`) and CLI/library view outputs (`schema/views.schema.json`).
- Operator and Rust library manual (`docs/GUIDE.md`), `SECURITY.md`, and `CONTRIBUTING.md`.

## [0.1.0] - 2026-01-01

### Added

- Append-only JSONL claim store (`onrecord::Store`) and five-subcommand CLI (`add`, `get`, `subject`, `due`, `check`).
- Position-bound `claim-<16hex>` SHA-256 ID derivation over Canonical Form 1 (`flighttape` value and timestamp grammar).
- Linear supersession chains with fork detection, subject inheritance (`OR-8b`), conflict-preserving `subject` and `due` views, and full-file `check` auditing.

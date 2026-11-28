<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="onrecord" width="780">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/david-engelmann/onrecord/actions/workflows/ci.yml"><img src="https://github.com/david-engelmann/onrecord/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://github.com/david-engelmann/onrecord/actions/workflows/clean-install.yml"><img src="https://github.com/david-engelmann/onrecord/actions/workflows/clean-install.yml/badge.svg" alt="clean-install"></a>
  <a href="https://github.com/david-engelmann/onrecord/actions/workflows/docs.yml"><img src="https://github.com/david-engelmann/onrecord/actions/workflows/docs.yml/badge.svg" alt="docs"></a>
  <a href="../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blueviolet.svg" alt="MIT License"></a>
</p>

# onrecord

Cross-session memory where nothing is stored as a bare fact: every memory is a claim on the record, carrying who asserted it, when, the evidence behind it, and the chain of claims it superseded.

Onrecord is an append-only claim store for agent memory. Every memory is stored as a claim with its receipts, who asserted it, when, and the evidence behind it, entry by entry. Claims are never edited, only superseded, and the record keeps the whole chain, so a later session can inspect or argue with what an earlier one believed. The store is a Rust library with a thin command line over a JSON lines file, and its own check recomputes every claim id and names any line that does not belong.

<p align="center">
  <img src="media/money-shot.gif" alt="onrecord subject conflict and self-audit CLI showcase" width="880">
</p>

## Install

### Prebuilt Binary (Linux & macOS)

```console
$ curl -fsSL https://raw.githubusercontent.com/david-engelmann/onrecord/main/install.sh | sh
```

### Cargo Binstall or Build from Source

```console
$ cargo binstall onrecord

# Or build CLI + library crate from source
$ git clone https://github.com/david-engelmann/onrecord
$ cd onrecord
$ cargo install --path . --locked
```

## Quickstart

From a local checkout of this repository:

Try `subject`, `due`, and `check` immediately against the shipped `examples/demo_record.jsonl` ledger, or create a new ledger from scratch:

```console
$ onrecord subject examples/demo_record.jsonl deploy
$ onrecord check examples/demo_record.jsonl
the record is sound
```

Adding and reading back a new claim:

```console
$ onrecord add record.jsonl --text "The staging deploy is broken" --asserter agent-a --at 2026-01-01T09:00:00Z --evidence trace:run-17 --subject deploy --due 2026-02-01
claim-2946b699934e0f6b
$ onrecord get record.jsonl claim-2946b699934e0f6b
```

## Architecture

<p align="center">
  <img src="media/architecture.svg" alt="onrecord position-bound claim ledger architecture" width="900">
</p>

## Usage

- **`add`**: Appends a validated claim with caller-supplied ISO timestamp (`--at`) and one or more `--evidence source:ref` receipts; prints its position-bound `claim-<16hex>` ID.
- **`get`**: Returns one claim with its oldest-first supersession chain.
- **`subject`**: Answers a subject key with all live claims; sets `"conflict": true` when two or more independent live heads exist on the subject.
- **`due`**: Lists live heads whose `--due` date has arrived by `<YYYY-MM-DD>`.
- **`check`**: Recomputes every claim ID from content + line position and verifies all supersession links.

## Performance

<p align="center">
  <img src="media/benchmark-chart.svg" alt="benchmark profile" width="900">
</p>

Measured on Apple M3 Max (`--release`, `745.8 KiB` binary; details in [docs/PERFORMANCE.md](PERFORMANCE.md)):

| Claims in `record.jsonl` | Ledger Size | `onrecord check` | `onrecord subject` | `onrecord due` | `check` Throughput |
| ---: | ---: | ---: | ---: | ---: | ---: |
| `100` | `25.7 KiB` | `49.16 ms` | `39.83 ms` | `28.38 ms` | `2,034 claims/s` |
| `1,000` | `263.8 KiB` | `254.18 ms` | `289.49 ms` | `284.4 ms` | `3,934 claims/s` |
| `5,000` | `1326.1 KiB` | `1284.58 ms` | `1233.73 ms` | `1233.54 ms` | `3,892 claims/s` |

## Where `onrecord` Fits in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="Retrospective Line ten-repository architecture" width="900">
</p>

## Documentation

- [Documentation Index](README.md) & [Searchable Handbook](https://david-engelmann.github.io/onrecord/)
- [Operator & Integrator Guide](GUIDE.md)
- [Technical Architecture](ARCHITECTURE.md)
- [Wire Formats & JSON Schemas](FORMATS.md)
- [Ecosystem Comparison](COMPARISON.md)
- [Performance Benchmarks](PERFORMANCE.md)
- [FAQ](FAQ.md)

## Status

The claim store is complete at its contracted scope and in daily use by its sibling tools. Closed perfection rounds, 33 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See the [LICENSE](../LICENSE) file.

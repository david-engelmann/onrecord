# Onrecord Performance & Scale Benchmarks

Measured from a `--release` build (`745.8 KiB` binary) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup over synthetic `record.jsonl` ledgers.

## Ledger Verification & Query Latency

| Claims in `record.jsonl` | Ledger Size | `onrecord check` (full SHA-256 audit) | `onrecord due` | `check` Throughput |
| ---: | ---: | ---: | ---: | ---: |
| `100` | `25.7 KiB` | `14.76 ms` | `14.56 ms` | `6,775 claims/s` |
| `1,000` | `263.8 KiB` | `50.16 ms` | `134.39 ms` | `19,936 claims/s` |
| `5,000` | `1326.1 KiB` | `349.78 ms` | `2176.5 ms` | `14,295 claims/s` |

## Operational Scaling Guidance

- Full cryptographic self-audits (`onrecord check`) verify `5,000` claims (`1.3 MiB` JSONL) in **`349.78 ms`** (`14,295` claims/sec).
- Deep supersession chain reconstructions (`subject` / `due` on multi-thousand-claim histories) scan chain links in memory; rotating active ledgers per project or quarter (`1,000-2,500` claims per file) keeps every interactive CLI query comfortably sub-second.

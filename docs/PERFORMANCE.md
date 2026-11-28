# Onrecord Performance & Scale Benchmarks

<p align="center">
  <img src="media/benchmark-chart.svg" alt="onrecord benchmark profile" width="920">
</p>

Measured from a thin-LTO symbol-stripped `--release` build (`548.6 KiB` binary with `O(N)` `index_by_id` chain resolution) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup over synthetic `record.jsonl` ledgers.

## Ledger Verification & Query Latency

| Claims in `record.jsonl` | Ledger Size | `onrecord check` | `onrecord subject` | `onrecord due` | `check` Throughput |
| ---: | ---: | ---: | ---: | ---: | ---: |
| `100` | `25.7 KiB` | `49.16 ms` | `39.83 ms` | `28.38 ms` | `2,034 claims/s` |
| `1,000` | `263.8 KiB` | `254.18 ms` | `289.49 ms` | `284.4 ms` | `3,934 claims/s` |
| `5,000` | `1326.1 KiB` | `1284.58 ms` | `1233.73 ms` | `1233.54 ms` | `3,892 claims/s` |

## Ledger Rotation Guidance

Every query (`check`, `subject`, `due`) executes in linear `O(N)` time, auditing and querying `5,000` claims in ~`1233.73 ms`. When archiving historical ledgers into a fresh file, rotate only independent subject histories (keep active supersession chains in the ledger containing their ancestor claims so parent links remain resolvable).

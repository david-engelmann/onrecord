# Onrecord Documentation

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="onrecord" width="760">
  </picture>
</p>

Welcome to the documentation suite for `onrecord`, the append-only JSONL claim store for cross-session agent memory where nothing is stored as a bare fact.

## Documentation Index

| Document | Audience | Contents |
| :--- | :--- | :--- |
| [Operator & Integrator Guide](GUIDE.md) | Agent & platform engineers | Installation, all 5 CLI commands (`add`, `get`, `subject`, `due`, `check`), Rust library embedding |
| [Technical Architecture](ARCHITECTURE.md) | Systems engineers & contributors | Position-bound Canonical Form 1 claim IDs, linear supersession vs subject conflict, closed-FD pre-init |
| [Wire Formats & Schemas](FORMATS.md) | Integrators & schema authors | Stored JSONL claim record (`schema/claim.schema.json`), `get`/`subject`/`due` view schemas |
| [Ecosystem Comparison](COMPARISON.md) | Architects & researchers | How `onrecord` differs from vector RAG memory (Mem0/Zep), mutable KV stores, and `flighttape` |
| [Performance & Scale](PERFORMANCE.md) | Infrastructure teams | Measured benchmarks up to `5,000` claims (`19,936` claims/sec self-audit on Apple M3 Max) |
| [Frequently Asked Questions](FAQ.md) | All users | Why forks in supersession are refused, how conflicts work, multi-writer locking note, timestamp rules |
| [Storefront README Draft](STOREFRONT_README_DRAFT.md) | Maintainers & Muse seat | Drop-in visual storefront upgrade preserving `32 of 100` rounds pin |

## Machine-Readable Contracts

- Schemas: [`../schema/claim.schema.json`](../schema/claim.schema.json), [`../schema/views.schema.json`](../schema/views.schema.json)
- Runnable Example Ledger: [`../examples/demo_record.jsonl`](../examples/demo_record.jsonl)

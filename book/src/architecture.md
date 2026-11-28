# Onrecord Technical Architecture

<p align="center">
  <img src="media/architecture.svg" alt="onrecord technical architecture" width="920">
</p>

Most "agent memory" stores flatten observations into unattributed vector chunks or overwrite keys in place: when an earlier agent hallucinated or a deployment status changed, later sessions cannot tell *who* believed *what*, *when*, on *what evidence*, or whether two active agents currently disagree. `onrecord` models agent memory as an **immutable, append-only ledger of receipted claims**.

## 1. Core Architectural Invariants

1. **Position-Bound Content Addressing (`derive_id`)**:
   - Every claim's `id` (`claim-<16 lowercase hex>`) is the first 16 hex digits of the Canonical Form 1 SHA-256 digest over `{"claim": claim_without_id, "position": zero_based_line_index}`.
   - Binding `position` into the hash means editing, inserting, reordering, or deleting any interior line in `record.jsonl` invalidates subsequent claim IDs, and partial tail writes fail JSON parsing (`not JSON`). To also detect clean removal of the final complete line, pair `onrecord check` with an expected line count or head claim ID checkpoint.
2. **Shared Grammar with `flighttape` (`OR-18`)**:
   - `onrecord` consumes `flighttape::timeparse` and Canonical Form 1 parsing directly from the `flighttape` crate rather than forking timestamp or JSON rules. The store never calls `SystemTime::now()`; every `asserted_at` timestamp is caller-supplied and grammar-checked.
3. **Linear Supersession vs Explicit Subject Conflict**:
   - **Supersession (`--supersedes <id>`)** updates an earlier belief linearly (`claim-A <- claim-B`). Attempting to supersede a nonexistent claim or an already-superseded claim (forking a chain) is refused with exit code `1`.
   - **Subject Conflict (`onrecord subject <store> <key>`)** occurs when two independent live chains assert claims on the same `subject` (for instance `agent-ops` reporting a healthy rollback while `agent-canary` reports elevated 502s). Rather than last-write-wins silently hiding disagreement, `subject` returns `"conflict": true` with every live claim and its full chain attached.
4. **Embeddable Rust Library (`onrecord::Store`)**:
   - Exposes `Store`, `StoreError`, `load_record`, `derive_id`, and `claim_defects` directly for sibling Rust crates such as `tetlock`.

## 2. Position in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="onrecord in the Retrospective Line" width="920">
</p>

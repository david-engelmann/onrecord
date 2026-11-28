# Ecosystem Comparison

| Dimension | `onrecord` | Vector / Graph Agent Memory (Mem0, Zep, LangMem) | Mutable SQLite / KV State | `flighttape` (Sibling Tool) |
| :--- | :--- | :--- | :--- | :--- |
| **Storage primitive** | Append-only receipted claim (`asserter`, `asserted_at`, `evidence[]`, `supersedes`) | Cosine-similarity chunks or LLM-summarized entity nodes | Last-write-wins key/value rows | Single finished run transcript (`1.0` bundle) |
| **Contradiction handling** | Explicit `"conflict": true` returning all competing live chains & receipts | Silent overwrite or prompt-stuffing top-K contradictory chunks | Overwrites prior value | N/A (single session scope) |
| **Tamper auditability** | `onrecord check` recomputes every position-bound SHA-256 claim ID | Opaque embeddings; no cryptographic line integrity | Mutable rows | Canonical Form 1 `tape_digest` per run |
| **External services** | Zero (`record.jsonl` plain file, `746 KiB` binary) | Embedding API + vector DB / graph DB | Local file | Zero (`*.tape.json` plain file) |

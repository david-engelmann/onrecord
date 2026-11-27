# Onrecord Wire Formats & Schemas

## 1. Stored JSONL Claim Line (`schema/claim.schema.json`)

Each line in `record.jsonl` is a standalone UTF-8 JSON object:

```json
{
  "id": "claim-f649cbd377931778",
  "text": "Staging deploy v2.4.1 passed health check after config rollback",
  "asserter": "agent-ops",
  "asserted_at": "2026-01-11T14:30:00Z",
  "evidence": [
    {"source": "trace", "reference": "run-108"},
    {"source": "log", "reference": "deploy-20260111"}
  ],
  "supersedes": "claim-32f4daa576031420",
  "subject": "deploy",
  "due": "2026-01-25"
}
```

## 2. Query Views (`schema/views.schema.json`)

- **`onrecord get <store> <id>`**: Returns a retrieval object `{ "claim": {...}, "chain": [oldest, ..., target] }`.
- **`onrecord subject <store> <subject>`**:
  - When **1** live claim answers `<subject>`, returns the single retrieval object `{ "claim": {...}, "chain": [...] }`.
  - When **2 or more** live claims answer `<subject>`, returns `{ "conflict": true, "subject": "<subject>", "claims": [{ "claim": {...}, "chain": [...] }, ...] }`.
- **`onrecord due <store> <YYYY-MM-DD>`**: `{ "as_of": "YYYY-MM-DD", "due": [{ "claim": {...}, "chain": [...] }], "conflicts": [...] }` (`conflicts` included when multiple due heads share a subject).
- **`onrecord check <store>`**: Prints `the record is sound` (exit `0`) or line-numbered defect diagnostics (exit `1`).

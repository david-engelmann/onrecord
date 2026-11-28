<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="onrecord" width="760">
  </picture>
</p>

<p align="center">
  <img src="media/money-shot.gif" alt="onrecord conflict view and check showcase" width="880">
</p>

<p align="center">
  <img src="media/architecture.svg" alt="onrecord claim ledger architecture" width="900">
</p>

# Onrecord Operator and Library Guide

`onrecord` is an append-only, self-verifying claim store for cross-session agent memory. Every entry in a record file (`record.jsonl`) is an immutable claim carrying its assertion text, asserter identity, caller-supplied timestamp, non-empty evidence list, linear supersession pointer, optional subject and due date, and a position-bound cryptographic digest ID (`claim-<16hex>`).

While `flighttape` records what happened inside a single session, `onrecord` preserves what agents believe across sessions without silent overwrites.

---

## 1. Installation

### Prebuilt Release Binary (`install.sh`)

On Linux and macOS, install a prebuilt binary verified against `SHA256SUMS`:

```console
$ ./install.sh
```

Environment variables accepted by `install.sh`:

- `ONRECORD_INSTALL_DIR`: destination directory (default `$HOME/.local/bin`)
- `ONRECORD_VERSION`: git tag such as `v0.1.0` (default `latest`)
- `ONRECORD_REPO`: GitHub repository slug (default `david-engelmann/onrecord`)

### Build from Source

```console
$ git clone https://github.com/david-engelmann/onrecord
$ cd onrecord
$ cargo install --path .
```

Minimum supported Rust version is `1.74`.

---

## 2. Record Format and Position-Bound Claim IDs

An `onrecord` store is a UTF-8 newline-delimited JSON (`.jsonl`) file with one claim object per line (an optional leading UTF-8 BOM is accepted on read). Blank lines, duplicate JSON keys within an object, non-object lines, and tampered fields or line positions are flagged by name.

### Claim Fields

| Field | Required | Type | Rules |
| --- | --- | --- | --- |
| `id` | yes | `string` | Derived `claim-<16 lowercase hex>` digest |
| `text` | yes | `string` | Non-empty claim statement |
| `asserter` | yes | `string` | Non-empty agent, tool, or human identifier |
| `asserted_at` | yes | `string` | Caller-supplied ISO 8601 timestamp (`flighttape::timeparse`) |
| `evidence` | yes | `array` | Non-empty list of `{"source": "...", "reference": "..."}` objects with non-empty strings |
| `supersedes` | yes | `string \| null` | Replaced `claim-...` ID in the same record, or `null` |
| `subject` | optional | `string \| null` | Subject key for live-state lookup and conflict detection |
| `due` | optional | `string \| null` | Real proleptic Gregorian calendar date in exact `YYYY-MM-DD` ASCII form |
| `<extra>` | optional | any JSON value | Arbitrary extension fields supplied through the Rust library `AddClaim.extra` API |

Machine-readable Draft 2020-12 JSON Schemas are shipped in `schema/claim.schema.json` and `schema/views.schema.json`.

### Canonical Form 1 Position-Bound Derivation (`OR-6a`)

Every claim ID is derived from both the claim payload (all fields of the claim object except `id` itself) and its 0-based line index `position` in the record:

1. Construct the envelope object:
   ```json
   {"claim": <claim-without-id>, "position": <0-based-line-index>}
   ```
2. Render the envelope under Canonical Form 1:
   - No insignificant whitespace (separators `,` and `:`).
   - Object keys sorted by UTF-16 code unit order (`u16` slice comparison); lone surrogates in keys or strings are rejected.
   - Integers rendered as exact base-10 digits (up to 4,300 digits).
   - Finite floating-point numbers rendered using CPython shortest round-trip digits under the ECMAScript number-to-string exponent thresholds (`NaN` and `Infinity` are rejected).
3. Compute `SHA-256` over the canonical UTF-8 bytes and take the first 16 lowercase hex characters:
   ```text
   claim-<hex[0..16]>
   ```

Because `position` is inside the hashed envelope, copying a valid line to another line index, deleting an earlier line, or reordering two lines invalidates the digest at the moved line.

---

## 3. CLI Reference (All Five Subcommands)

The repository includes a ready-to-inspect three-line store at `examples/demo_record.jsonl`.

### 3.1 `onrecord add`

Appends one validated claim to `<record>` (creating the file if it does not yet exist, and completing a missing trailing newline on an existing non-empty file before appending in place). Prints the new `claim-<16hex>` ID on stdout.

```console
$ onrecord add record.jsonl \
    --text "Staging deploy v2.4.0 failed health check on us-east-1" \
    --asserter agent-ops \
    --at 2026-01-10T09:00:00Z \
    --evidence trace:run-101 \
    --subject deploy \
    --due 2026-01-15
claim-32f4daa576031420
```

#### Repeatable `--evidence source:reference`

Pass `--evidence` multiple times to attach multiple receipts to a single claim. Each argument splits on its first `:` into `source` (non-empty) and `reference` (non-empty; may itself contain colons such as URLs or `sha256:abc...`):

```console
$ onrecord add record.jsonl \
    --text "Staging deploy v2.4.1 passed health check after config rollback" \
    --asserter agent-ops \
    --at 2026-01-11T14:30:00Z \
    --evidence trace:run-108 \
    --evidence log:deploy-20260111 \
    --supersedes claim-32f4daa576031420 \
    --subject deploy \
    --due 2026-01-25
claim-f649cbd377931778
```

Note on timestamps: `onrecord` never reads the system clock. Every `--at` timestamp is caller-supplied and validated strictly at `add` time.

### 3.2 `onrecord get`

Retrieves any claim by ID (whether live or superseded) along with its linear supersession `chain` ordered oldest-first:

```console
$ onrecord get examples/demo_record.jsonl claim-f649cbd377931778
{
  "claim": {
    "id": "claim-f649cbd377931778",
    "text": "Staging deploy v2.4.1 passed health check after config rollback",
    "asserter": "agent-ops",
    "asserted_at": "2026-01-11T14:30:00Z",
    "evidence": [
      {
        "source": "trace",
        "reference": "run-108"
      },
      {
        "source": "log",
        "reference": "deploy-20260111"
      }
    ],
    "supersedes": "claim-32f4daa576031420",
    "subject": "deploy",
    "due": "2026-01-25"
  },
  "chain": [
    {
      "id": "claim-32f4daa576031420",
      "text": "Staging deploy v2.4.0 failed health check on us-east-1",
      "asserter": "agent-ops",
      "asserted_at": "2026-01-10T09:00:00Z",
      "evidence": [
        {
          "source": "trace",
          "reference": "run-101"
        }
      ],
      "supersedes": null,
      "subject": "deploy",
      "due": "2026-01-15"
    },
    {
      "id": "claim-f649cbd377931778",
      "text": "Staging deploy v2.4.1 passed health check after config rollback",
      "asserter": "agent-ops",
      "asserted_at": "2026-01-11T14:30:00Z",
      "evidence": [
        {
          "source": "trace",
          "reference": "run-108"
        },
        {
          "source": "log",
          "reference": "deploy-20260111"
        }
      ],
      "supersedes": "claim-32f4daa576031420",
      "subject": "deploy",
      "due": "2026-01-25"
    }
  ]
}
```

### 3.3 `onrecord subject`

Returns the live state for a subject key:

- **Zero live claims**: exits `1` with `error, no live claim on subject <key>` on stderr.
- **One live claim**: exits `0` and prints the same `{"claim": ..., "chain": [...]}` retrieval document as `onrecord get`.
- **Two or more live claims (unresolved conflict)**: exits `0` and returns `"conflict": true` with every competing live claim and its full receipt `chain` side-by-side:

```console
$ onrecord subject examples/demo_record.jsonl deploy
{
  "conflict": true,
  "subject": "deploy",
  "claims": [
    {
      "claim": {
        "id": "claim-f649cbd377931778",
        "text": "Staging deploy v2.4.1 passed health check after config rollback",
        "asserter": "agent-ops",
        "asserted_at": "2026-01-11T14:30:00Z",
        "evidence": [
          {
            "source": "trace",
            "reference": "run-108"
          },
          {
            "source": "log",
            "reference": "deploy-20260111"
          }
        ],
        "supersedes": "claim-32f4daa576031420",
        "subject": "deploy",
        "due": "2026-01-25"
      },
      "chain": [
        {
          "id": "claim-32f4daa576031420",
          "text": "Staging deploy v2.4.0 failed health check on us-east-1",
          "asserter": "agent-ops",
          "asserted_at": "2026-01-10T09:00:00Z",
          "evidence": [
            {
              "source": "trace",
              "reference": "run-101"
            }
          ],
          "supersedes": null,
          "subject": "deploy",
          "due": "2026-01-15"
        },
        {
          "id": "claim-f649cbd377931778",
          "text": "Staging deploy v2.4.1 passed health check after config rollback",
          "asserter": "agent-ops",
          "asserted_at": "2026-01-11T14:30:00Z",
          "evidence": [
            {
              "source": "trace",
              "reference": "run-108"
            },
            {
              "source": "log",
              "reference": "deploy-20260111"
            }
          ],
          "supersedes": "claim-32f4daa576031420",
          "subject": "deploy",
          "due": "2026-01-25"
        }
      ]
    },
    {
      "claim": {
        "id": "claim-6d14311a110300c3",
        "text": "Canary synthetic probe still reports elevated 502 rate on staging",
        "asserter": "agent-canary",
        "asserted_at": "2026-01-11T15:00:00Z",
        "evidence": [
          {
            "source": "metric",
            "reference": "probe-884"
          }
        ],
        "supersedes": null,
        "subject": "deploy",
        "due": "2026-01-20"
      },
      "chain": [
        {
          "id": "claim-6d14311a110300c3",
          "text": "Canary synthetic probe still reports elevated 502 rate on staging",
          "asserter": "agent-canary",
          "asserted_at": "2026-01-11T15:00:00Z",
          "evidence": [
            {
              "source": "metric",
              "reference": "probe-884"
            }
          ],
          "supersedes": null,
          "subject": "deploy",
          "due": "2026-01-20"
        }
      ]
    }
  ]
}
```

### 3.4 `onrecord due`

Lists every live claim carrying a `due` date less than or equal to `<date>` (`YYYY-MM-DD`), sorted by `due` date ascending and then by 0-based record position. When two or more due claims answer for the same subject (including inherited subjects), `due` includes a sorted `"conflicts"` array naming each conflicting subject:

```console
$ onrecord due examples/demo_record.jsonl 2026-02-01
{
  "as_of": "2026-02-01",
  "due": [
    {
      "claim": {
        "id": "claim-6d14311a110300c3",
        "...": "..."
      },
      "chain": ["..."]
    },
    {
      "claim": {
        "id": "claim-f649cbd377931778",
        "...": "..."
      },
      "chain": ["..."]
    }
  ],
  "conflicts": [
    "deploy"
  ]
}
```

### 3.5 `onrecord check`

Audits every line of `<record>` from scratch, recomputing every `claim-<16hex>` ID against its 0-based line position, checking field types and timestamp/date grammars, verifying that every `supersedes` target exists, and verifying that no two claims supersede the same target (a fork):

```console
$ onrecord check examples/demo_record.jsonl
the record is sound
```

When defects are present, `check` prints every defect with its 1-based line number to stdout and exits `1`.

---

## 4. Supersession, Subject Inheritance (`OR-8b`), and Due Lifecycle

1. **Linear supersession, never forks**: A claim may supersede at most one earlier claim (`supersedes: "claim-..."`), and each claim may be superseded at most once. Attempting to `add` a second claim superseding an already-superseded ID is refused (`claim <id> has already been superseded`). A hand-edited fork in a file is caught by `check` (`line N: claim <target> (line M) is superseded by two claims, <a1> and <a2>, a fork`).
2. **Subject inheritance (`OR-8b`)**:
   - If a live head carries its own `subject` string, it answers **only** for that `subject` (moving a chain to a new subject stops answering for the old subject).
   - If a live head omits `subject` (or sets `"subject": null`), it inherits every distinct `subject` string carried anywhere along its `chain`.
3. **Due dates belong to the live head**:
   - A superseded claim's `due` date dies when it is superseded.
   - Superseding a due claim with an undated claim withdraws the due obligation while keeping the history in `chain`.
   - Superseding a due claim with a new `--due YYYY-MM-DD` reschedules it to the new date.

---

## 5. Multi-Agent Concurrency Guidance

Every claim ID binds to its 0-based line `position` in `record.jsonl`. `Store::add` reads the current record to determine `position`, derives the ID, and appends the new line. If multiple OS processes or threads append concurrently to the same `record.jsonl`, serialize `add` calls with an external advisory lock on a companion lockfile (for example `flock record.jsonl.lock onrecord add record.jsonl ...` in shell scripts) so two writers never compute the same `position`.

---

## 6. Rust Library Embedding

`onrecord` is both a binary and a library crate. Downstream tools in the portfolio (such as `tetlock`, which stores structured confession fields via `AddClaim.extra`) embed `Store` directly.

### Key Types and Entry Points

- `onrecord::Store` (constructed via `Store::new(RecPath)`):
  - `store.add(request: AddClaim) -> Result<String, StoreError>`
  - `store.get(claim_id: &Text) -> Result<Value, StoreError>`
  - `store.subject(key: &Text) -> Result<Value, StoreError>`
  - `store.due(as_of: &Text) -> Result<Value, StoreError>`
  - `store.check() -> Result<Vec<Msg>, StoreError>`
- `onrecord::store::RecPath`:
  - `RecPath::new("path/to/record.jsonl")` builds both the OS `PathBuf` and normalized display path.
- `onrecord::store::AddClaim`:
  - `text: Text`, `asserter: Text`, `asserted_at: Text`, `evidence: Value`
  - `supersedes: Option<Text>`, `subject: Option<Text>`, `due: Option<Text>`
  - `extra: Vec<(Text, Value)>`: attaches custom domain keys onto the stored claim object. Extra keys are included in Canonical Form 1 ID derivation (`OR-6a`), preserved verbatim in `get`/`subject`/`due` views, and refused by name (`extra field <key> collides with a claim field`) if they collide with `id`, `text`, `asserter`, `asserted_at`, `evidence`, `supersedes`, `subject`, `due`, or an earlier key in `extra`.
- Low-level verification helpers: `onrecord::{derive_id, claim_defects, load_record}`.

### Library Example (with `AddClaim.extra`)

```rust,ignore
use flighttape::value::{Text, Value};
use onrecord::store::{AddClaim, RecPath, Store};

fn append_confession() -> Result<String, onrecord::StoreError> {
    let store = Store::new(RecPath::new("memory/record.jsonl"));
    let evidence = Value::Array(vec![Value::Object(vec![
        (Text::from_str("source"), Value::Str(Text::from_str("tape"))),
        (Text::from_str("reference"), Value::Str(Text::from_str("session-42"))),
    ])]);

    store.add(AddClaim {
        text: Text::from_str("Migration tested cleanly against staging snapshot"),
        asserter: Text::from_str("tetlock-runner"),
        asserted_at: Text::from_str("2026-01-15T18:00:00Z"),
        evidence,
        supersedes: None,
        subject: Some(Text::from_str("db-migration")),
        due: Some(Text::from_str("2026-01-22")),
        extra: vec![
            (Text::from_str("confidence"), Value::Float(0.85)),
            (Text::from_str("grade_status"), Value::Str(Text::from_str("pending"))),
        ],
    })
}
```

---

## 7. Exit Codes

| Exit Code | Meaning |
| --- | --- |
| `0` | Command succeeded (`check` found zero defects; `add`, `get`, `subject`, or `due` completed cleanly; `-h` / `--help` printed help). |
| `1` | Domain or I/O refusal (`error, <reason>` on stderr), `check` found one or more record defects (printed on stdout), or unrecoverable stream/encoding failure when standard streams were closed at launch. |
| `2` | CLI usage / argument parse error (`argparse`-compatible usage and error message on stderr). |
| `120` | Buffered stdout or stderr flush failed at interpreter-mode shutdown (for example writing redirected output to a full device or broken pipe). |

---

## Shell Completions & `cargo-binstall`

```sh
cargo binstall onrecord

# Shell completions (bash, zsh, fish, PowerShell)
source completions/onrecord.bash
```


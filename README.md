# onrecord

Cross-session memory where nothing is stored as a bare fact: every memory is a claim on the record, carrying who asserted it, when, the evidence behind it, and the chain of claims it superseded.

Onrecord is an append-only claim store for agent memory. Every memory is stored as a claim with its receipts, who asserted it, when, and the evidence behind it, entry by entry. Claims are never edited, only superseded, and the record keeps the whole chain, so a later session can inspect or argue with what an earlier one believed. The store is a Rust library with a thin command line over a JSON lines file, and its own check recomputes every claim id and names any line that does not belong.

[![ci](https://github.com/david-engelmann/onrecord/actions/workflows/ci.yml/badge.svg)](https://github.com/david-engelmann/onrecord/actions/workflows/ci.yml)

```console
$ onrecord add record.jsonl --text "The staging deploy is broken" --asserter agent-a --at 2026-01-01T09:00:00Z --evidence trace:run-17 --subject deploy --due 2026-02-01
claim-2946b699934e0f6b
$ onrecord check record.jsonl
the record is sound
```

## Install

A Rust toolchain is the only prerequisite. Clone the repo and install the command from it.

```console
$ git clone https://github.com/david-engelmann/onrecord
$ cd onrecord
$ cargo install --path .
```

The build pulls in one more crate from this fleet, flighttape, at a pinned revision in Cargo.toml. Onrecord takes the timestamp grammar behind every asserted_at value and the canonical digest its claim ids rest on from flighttape, so both are defined once in the crate that owns them and never forked into this repo.

The same build gives the library. The crate is named onrecord and other crates in this fleet consume it as a library, tetlock among them.

## Quickstart

Add a claim and read it back. The add prints the claim id, which the store derives from the claim content and its position in the record.

```console
$ onrecord add record.jsonl --text "The staging deploy is broken" --asserter agent-a --at 2026-01-01T09:00:00Z --evidence trace:run-17 --subject deploy --due 2026-02-01
claim-2946b699934e0f6b
$ onrecord get record.jsonl claim-2946b699934e0f6b
{
  "claim": {
    "id": "claim-2946b699934e0f6b",
    "text": "The staging deploy is broken",
    "asserter": "agent-a",
    "asserted_at": "2026-01-01T09:00:00Z",
    "evidence": [
      {
        "source": "trace",
        "reference": "run-17"
      }
    ],
    "supersedes": null,
    "subject": "deploy",
    "due": "2026-02-01"
  },
  "chain": [
    {
      "id": "claim-2946b699934e0f6b",
      "text": "The staging deploy is broken",
      "asserter": "agent-a",
      "asserted_at": "2026-01-01T09:00:00Z",
      "evidence": [
        {
          "source": "trace",
          "reference": "run-17"
        }
      ],
      "supersedes": null,
      "subject": "deploy",
      "due": "2026-02-01"
    }
  ]
}
```

The get view returns the claim with its supersession chain, oldest first. A claim that nothing has superseded is its own chain of one.

## Usage

The command line carries five commands. Add appends a claim. Get reads one claim back with its chain. Subject answers a subject key with the live claims on it, and when two live claims conflict on a subject, the conflict itself is the answer, with both receipts attached. Due lists the live claims that have come due by a date. Check recomputes every id in the record and names every defect it finds.

Superseding is how a claim changes. The new claim names the one it replaces, and the chain stays in the record.

```console
$ onrecord add record.jsonl --text "The staging deploy is fixed" --asserter agent-b --at 2026-01-02T09:00:00Z --evidence trace:run-19 --supersedes claim-2946b699934e0f6b --subject deploy
claim-b7d8a8f8fdeebc2d
```

Superseding a claim that does not exist, or one already superseded, is refused by name. Supersession is a chain, never a fork.

The due view answers the live claims whose due dates have arrived by the date given. A claim whose due date passed to a superseding claim no longer answers, because the live head carries no due date.

```console
$ onrecord add record.jsonl --text "The deploy checklist needs a second reader" --asserter agent-a --at 2026-01-03T09:00:00Z --evidence review:checklist-4 --subject deploy --due 2026-01-20
claim-32510ef27c144013
$ onrecord due record.jsonl 2026-02-01
{
  "as_of": "2026-02-01",
  "due": [
    {
      "claim": {
        "id": "claim-32510ef27c144013",
        "text": "The deploy checklist needs a second reader",
        "asserter": "agent-a",
        "asserted_at": "2026-01-03T09:00:00Z",
        "evidence": [
          {
            "source": "review",
            "reference": "checklist-4"
          }
        ],
        "supersedes": null,
        "subject": "deploy",
        "due": "2026-01-20"
      },
      "chain": [
        {
          "id": "claim-32510ef27c144013",
          "text": "The deploy checklist needs a second reader",
          "asserter": "agent-a",
          "asserted_at": "2026-01-03T09:00:00Z",
          "evidence": [
            {
              "source": "review",
              "reference": "checklist-4"
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

Check is the record's self audit. It walks every line, recomputes each id from the claim and its position, and reports the record sound or names the defects by line.

```console
$ onrecord check record.jsonl
the record is sound
```

The store never invents a time. Every timestamp in a claim is one the caller supplied, and a timestamp the grammar refuses is refused by name at add time. Flighttape records the run and onrecord remembers across runs.

## Status

The claim store is complete at its contracted scope and in daily use by its sibling tools. Closed perfection rounds, 16 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See the LICENSE file.

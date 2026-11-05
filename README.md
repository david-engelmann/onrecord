# onrecord

Cross-session memory where nothing is stored as a bare fact, every memory is a claim on the record, carrying who asserted it, when, the evidence behind it, and the chain of claims it superseded.

## What this is

Agent memory tools store conclusions without their evidence, so a fresh session inherits beliefs it cannot inspect or argue with. Onrecord stores claims with receipts instead, asserted by a named agent or human, with the evidence named entry by entry, source and reference. Claims are never edited, only superseded, and the supersession chain is part of the record. When two live claims conflict, the conflict itself is the answer retrieval returns, with both receipts attached, instead of a blended fiction. The boundary with its sibling is one sentence. Flighttape records the run, onrecord remembers across runs.

The store is a library plus a thin command line, add, get, subject, due, and check, over an append-only JSON lines file. A claim's id is derived from its content and its position, so the record's own check can recompute every id and name any line that does not belong. Superseding a claim that does not exist, or one already superseded, is refused by name: supersession is a chain, never a fork. A claim may carry a subject key and a due date. Round 1 stored and retrieved both and acted on neither. Round 2 acts on the due dates: the due view answers the live claims that have come due by a date the caller supplies, each with its receipts, conflicts named rather than resolved, and wallofwrong's confessions remain the due dates' other consumer.

## Status

Round 1 is closed. The claim store: append-only
claims with receipts, supersession chains, conflict
as the answer.

Round 2 is closed. The due view over the stored due
dates. Setup landed earlier in October 2026.

Round 3 is closed. The claim store ported to Rust:
a library crate with the same thin command line,
behavior reproduced byte for byte against the
Python it replaces, which stands as the round's
specification. The JSON reader, the value model,
and the timestamp grammar are consumed from the
flighttape crate at a pinned revision.

## Programme status

- Closed perfection rounds, 6 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, OR-1 to OR-12 as amended, met for the Round 1 scope.
  OR-13 to OR-16 with amendments OR-13a, OR-13b, and
  OR-16a, written ahead of the code and met in Round 2.
  OR-17 to OR-23 with OR-18a, the port obligations,
  written ahead of the code and met in Round 3.
- Local gate, python3 scripts/codepoint_scan.py,
  cargo fmt --check, cargo clippy --all-targets with
  warnings denied, and cargo test. CI runs the same
  commands.

## Licence

MIT. See LICENSE.

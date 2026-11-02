# onrecord

Cross-session memory where nothing is stored as a bare fact, every memory is a claim on the record, carrying who asserted it, when, the evidence behind it, and the chain of claims it superseded.

## What this is

Agent memory tools store conclusions without their evidence, so a fresh session inherits beliefs it cannot inspect or argue with. Onrecord stores claims with receipts instead, asserted by a named agent or human, with the evidence named entry by entry, source and reference. Claims are never edited, only superseded, and the supersession chain is part of the record. When two live claims conflict, the conflict itself is the answer retrieval returns, with both receipts attached, instead of a blended fiction. The boundary with its sibling is one sentence. Flighttape records the run, onrecord remembers across runs.

The store is a library plus a thin command line, add, get, subject, and check, over an append-only JSON lines file. A claim's id is derived from its content and its position, so the record's own check can recompute every id and name any line that does not belong. Superseding a claim that does not exist, or one already superseded, is refused by name: supersession is a chain, never a fork. A claim may carry a subject key and a due date; Round 1 stores and retrieves both and acts on neither, and wallofwrong's confessions are the due dates' first consumer.

## Status

Round 1 is built and in review on its branch. Setup landed earlier in October 2026.

## Programme status

- Closed perfection rounds, 1 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, OR-1 to OR-12 as amended, met for the Round 1 scope.
- Local gate, python3 scripts/codepoint_scan.py and
  python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.

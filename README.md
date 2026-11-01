# onrecord

Cross-session memory where nothing is stored as a bare fact, every memory is a claim on the record, carrying who asserted it, when, the evidence behind it, and the chain of claims it superseded.

## What this is

Agent memory tools store conclusions without their evidence, so a fresh session inherits beliefs it cannot inspect or argue with. Onrecord stores claims with receipts instead, asserted by a named agent or human, in a named session, from a named commit, trace, or document, at a stated confidence. Claims are never edited, only superseded, and the supersession chain is part of the record. When two live claims conflict, the conflict itself is the answer retrieval returns, with both receipts attached, instead of a blended fiction. The boundary with its sibling is one sentence. Flighttape records the run, onrecord remembers across runs.

## Status

Seeded in October 2026. Local setup is in progress on the setup
branch. The remote seeding pass is waiting on a token access fix, so
work is banked locally until pushes clear. This repo is in phase 3
of the Retrospective Line and its build waits on the phase gate, the
main chat confirmation that the Gate Truth evidence pack schema has
merged and passed acceptance. Design work may proceed, build does
not start before the gate.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, stub in place, not yet met.
- Local gate, python3 scripts/codepoint_scan.py and, once tests
  exist, python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.

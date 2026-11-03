# onrecord Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- OR-1. Nothing is stored as a bare fact. Every claim carries who asserted it, when, the evidence, and the chain it superseded.
- OR-2. Claims are never edited, only superseded, and the supersession chain is part of the record.
- OR-3. Retrieval returns the claim and its receipts together, never the claim alone.
- OR-4. When two live claims conflict, the conflict is the answer, with both receipts attached.

## Round 1. The claim store (OR-5 to OR-12)

Built in Round 1, ahead of wallofwrong, whose
confessions are stored here. The round also added the
amendments below (OR-5a to OR-11a before code, OR-5c,
OR-8b, OR-11b, and OR-12a after the reviews, before
the repair code).

- OR-5. A record is an append-only JSON lines file. Each
  line is one claim object: an id, the claim text, who
  asserted it, when it was asserted (an ISO timestamp the
  caller supplies; the store never invents a time), an
  evidence list whose entries name a source and a
  reference, and a supersedes field naming the claim it
  replaces or null. A line that fails this shape is a
  named defect at read time, with its line number.
  Test, tests/test_store.py.
- OR-6. Adding a claim appends exactly one line and
  returns the new claim's id, derived deterministically
  from the claim's own content and its position, so the
  same append against the same record yields the same
  id. The file's earlier bytes are never rewritten.
  Test, tests/test_store.py.
- OR-7. A claim whose supersedes names a claim that does
  not exist, or that has already been superseded, is
  refused by name and nothing is appended. Supersession
  is a chain, never a fork. Test, tests/test_store.py.
- OR-8. The live claim on a subject is the head of its
  chain. Retrieval by id returns the claim and its full
  receipts: the claim itself, its evidence, and the whole
  chain behind it, oldest first. Retrieval of a subject
  returns its live claim the same way. Test,
  tests/test_store.py.
- OR-9. Two live claims conflict when they share a
  subject key and neither supersedes the other. Retrieval
  that meets a conflict returns the conflict as the
  answer, both claims with both receipt chains, and a
  marker that says conflict, never a silent pick.
  A subject key is a field the caller sets on the claim;
  claims without one never conflict. Test,
  tests/test_store.py.
- OR-10. A claim may carry a due date. Nothing in Round 1
  acts on due dates; they are stored and retrieved, and
  wallofwrong's confessions will use them. Storing them
  is this round's whole obligation. Test,
  tests/test_store.py.
- OR-11. The store is a library plus a thin command line:
  add, get, subject, and check. Check reads the whole
  record and names every defect it finds (bad lines,
  broken chains, forks), exiting 1 when any exist. All
  failures are named errors with exit 1, never
  tracebacks. Test, tests/test_store.py.
- OR-12. Every write is atomic in the family's sense for
  appends: the line is written and flushed in one append
  operation, and a crash mid-append is detectable by
  check as a defective final line, never as silent
  corruption of earlier claims. Test, tests/test_store.py.

## Round 1 amendments, the pinned shapes (recorded before code)

- OR-5a. A claim's fields: id, text, asserter,
  asserted_at (an ISO timestamp string), evidence (a
  list of objects with source and reference strings),
  supersedes (a claim id or null), and optionally
  subject and due (an ISO date string). Fields beyond
  these are part of the claim and are kept verbatim;
  missing or mistyped required fields are defects.
  Test, tests/test_store.py.
- OR-6a. The id is the word claim, a dash, and the
  first 16 hex characters of the canonical form 1
  digest of the claim without its id, together with
  its position, the count of lines already in the
  record. Check recomputes every id and flags a
  mismatch as a defect. Test, tests/test_store.py.
- OR-8a. Retrieval returns an object with the claim
  under claim and its chain under chain, the chain
  being every claim it supersedes back to the root,
  oldest first, ending with the claim itself. Test,
  tests/test_store.py.
- OR-9a. A conflict answer is an object with conflict
  set to true, the subject, and under claims one
  retrieval object per live claim, in record order.
  Test, tests/test_store.py.
- OR-11a. Check names, with line numbers: a line that
  is not JSON, a line that is not a claim object, a
  missing or mistyped field, an id that does not match
  its derivation, a supersedes naming no claim in the
  record, a claim superseded by two others (a fork),
  and a timestamp or due date that does not parse.
  Reads (add, get, subject) on a record with defects
  refuse, naming the defects; check is the tool that
  reports them all. Test, tests/test_store.py.

## Round 1 amendments, after the reviews (recorded before the repair code)

- OR-5c. The evidence list holds at least one entry,
  and each entry's source and reference are non-empty
  strings. OR-1 stores nothing as a bare fact, so an
  empty list is a defect at read time and a refusal at
  add time. Test, tests/test_store.py.
- OR-8b. A chain answers for the subjects its claims
  carry. A live head carrying a subject answers for
  that subject alone; a live head carrying none
  answers for every subject named along its chain.
  Superseding a claim without repeating its subject
  therefore keeps the subject answered by the head.
  Test, tests/test_store.py.
- OR-11b. A claim that cannot be rendered under
  canonical form 1 is a named defect: its id cannot
  be verified, and an unverifiable id is never
  silently passed over. Test, tests/test_store.py.
- OR-12a. A record whose final line lacks its
  terminating newline is still sound when the line
  itself is a sound claim. An append completes that
  line with the missing newline first, then appends
  its own line; earlier bytes are never rewritten and
  claims are never glued. Test, tests/test_store.py.

## Round 2. The due view (OR-13 to OR-16)

- OR-13. The store gains a due view: given a date the
  caller supplies, it answers which live claims have
  come due. The date is an ISO calendar date, validated
  like a claim's due date; anything else is a named
  refusal. The store never invents a time here either:
  no date supplied, no answer. Test,
  tests/test_store.py.
- OR-14. A claim is due when it is live (no claim
  supersedes it) and carries a due date on or before
  the supplied date. A superseded claim never appears,
  whatever its due date said: its successor answers
  for the chain, with its own due date or none. A live
  claim with no due date never appears. Test,
  tests/test_store.py.
- OR-15. The answer is a document naming the supplied
  date and the due claims in order: due date
  ascending, then record position ascending. Each due
  claim arrives as its retrieval document, the claim
  and its full chain, exactly as get returns it
  (OR-3, OR-8). Date comparison is calendar
  comparison: ISO dates order as strings. Test,
  tests/test_store.py.
- OR-16. The due view never resolves a conflict and
  never hides one. When two live claims sharing a
  subject are both due, both appear with their
  receipts, and the document names the shared subject
  in a conflicts list, present only when non-empty,
  so the conflict is part of the answer (OR-4). Test,
  tests/test_store.py.

## Round 2 post-review amendments, recorded before the repairs

- OR-13a. A date in this store is exactly one form:
  four digits, a hyphen, two digits, a hyphen, two
  digits, and a real calendar day. The basic form
  without hyphens and the week forms are not dates
  here, at add time, at read time, or as the due
  view's supplied date. The parser alone accepts
  those forms, and the due view compares dates as
  strings, so an alien form stored silently answered
  wrongly, due on no day or due early. Test,
  tests/test_store.py.
- OR-13b. The due view refuses a missing record by
  name, like check does. An existing record, empty
  included, answers with an empty list; a mistyped
  path must never read as nothing due. Test,
  tests/test_store.py.
- OR-16a. The due view's conflicts are counted over
  the subjects each due claim answers for under
  OR-8b: its own subject when it carries one, else
  every subject named along its chain. One answering
  rule, shared with subject retrieval, so a conflict
  the store itself recognises is never hidden by the
  due view. A subject one due claim alone answers
  for is no conflict, and a chain repeating a subject
  does not conflict with itself. Test,
  tests/test_store.py.

## Round 3. The Rust port (OR-17 to OR-23)

- OR-17. The port reproduces the Python's behavior
  exactly: every command, refusal, defect phrase,
  rendered byte, and exit code. Deviations are named
  in the round dossier, never silent. Test, the
  differential harness of OR-22.
- OR-18. A Rust library crate plus a thin onrecord
  binary. The flighttape crate at pinned revision
  ba8d4b550850179722f48b8f9c909e199efe3548 is the only
  direct dependency: the JSON reader, the value model,
  the timestamp grammar, and canonical form 1 come
  from the crate, never re-implemented. Claim ids are
  derived under OR-6a from the crate's canonical form;
  where the crate's float digits are shown by probe to
  differ from the Python digest's, the port renders
  the digest with the Python's digit source and names
  it here and in the dossier.
- OR-19. Where the record's reading differs from the
  crate reader, the Python governs: duplicate keys in
  a record line are refused by name, and a line past
  the Python's parse depth is refused with its own
  phrase, each surfacing as that line's named defect.
  The port closes the differences in its loader and
  names them in the dossier.
- OR-20. Appended lines and views reproduce the
  Python's rendering byte for byte: appended claims in
  the compact separators with insertion key order,
  views at an indent of two, ensure_ascii=False, and
  exact float rendering.
- OR-21. The Python tests port to cargo tests file for
  file, and the Python package, its tests, and
  pyproject leave the tree only in the landing state.
  Cargo tests are the suite.
- OR-22. A differential harness over the whole command
  surface byte-compares stdout, stderr, exit codes,
  record bytes, and view output against the Python:
  the happy path, the defect catalogue, every refusal,
  the due view's orderings and conflicts, and an argv
  battery across both argparse levels.
- OR-23. The round dossier carries the port report:
  port or full rewrite with reasons, the Rust versus
  Python line share on the landing tree, the bindings
  determination, and the non-Rust remainder register
  with a justification per item.

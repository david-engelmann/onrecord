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

Built in Round 1, ahead of tetlock, whose
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
  tetlock's confessions will use them. Storing them
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
- OR-18a. Amendment, recorded before the port code:
  sha2 is a second direct dependency, the very crate
  and version the pinned flighttape tree already
  brings (0.10), because claim ids hash this port's
  own canonical rendering under OR-18 and no other
  surface consumes it. OR-18's only-direct-dependency
  sentence reads with this amendment.
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
- OR-24. Round 4 re-pins the flighttape crate from
  ba8d4b550850179722f48b8f9c909e199efe3548 to
  eebce0be75a9c2ffa534635e37160c7cbef5d96e, the
  Round 12 merge. No onrecord behavior changes
  except what the crate's timestamp grammar
  carries: the stray digit at end of string now
  refuses (FT-60) and the one-character separator
  skip before the offset now accepts (FT-60a).
- OR-25. The named deviation for the timestamp
  stray digit (Round 3 dossier, deviation 6)
  retires: the differential harness's cells for
  the four recorded shapes now expect agreement,
  and cells for the FT-60a shapes (a space before
  Z, a space before an offset, the component
  colon before the sign) assert the new grammar
  flows through asserted_at validation with the
  interpreter's instants. No other deviation
  moves.
- OR-26. The round's verification is the full
  local gate plus the differential harness re-run
  over its standing corpus at zero failures,
  with the timestamp cells of OR-25 included.
  CodeRabbit's advisory pass on Round 3's PR 4
  posted no findings (its summary shell and an
  action-not-completed notice stand on the PR;
  the review endpoint returns zero entries), so
  no disposition is owed from it.
- OR-27. Round 5 re-pins flighttape at 74a83298,
  the Round 13 merge, in every table that names
  it. Nothing else in the crate's behavior
  changes.
- OR-28. Deviation 6's successors retire with
  the re-pin: the FT-63 to FT-66 colon-group
  and tail shapes and the FT-68 lone-NUL
  terminator shapes, divergent under the old
  pin because the crate's parse_timestamp moved,
  agree with the Python original once the pin
  moves, proven by control evidence: the
  unmodified harness against the re-pinned
  binary fails exactly the cells the harness
  carries for those shapes, if any; new harness
  cells carry representative shapes of both
  families (colon fraction, time-side leftover,
  NUL terminator at a completion point, NUL
  refused inside a window) through the add,
  subject, and due surfaces, and the final
  harness is clean.
- OR-29. README records 5 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- OR-30. The CodeRabbit disposition owed from
  Round 5 discharges here on evidence: the
  Round 5 pull request's late findings were
  read back in the 2026-11-04 rotation sweep
  and numbered zero, so there is nothing to
  fix and the absence is the disposition.
- OR-31. Round 6 takes the two write-seam
  probes the rotation has not yet run on this
  crate. First, temp placement at every seam
  that writes through a temp: the oracle's
  mkstemp absolutizes the given parent
  lexically, and a port creating its temp
  under the spelled parent diverges exactly
  where a popped symlink makes the lexical
  and physical parents differ in existence.
  The view seam (get, subject, due, check
  writing to an output path) and the append
  seam (add) are each measured on their own
  oracle call structure, convicted shapes
  repaired per the measured law, agreement
  shapes pinned. Second, the temp suffix:
  mkstemp's suffix is exactly eight
  characters of its own alphabet; this crate's
  generator is measured for width, and any
  name-length boundary that moves with the
  generator's width is convicted and repaired
  to the oracle's fixed width or filed with
  its measurement if the seam proves to carry
  no temp. Whichever arms convict, the repair
  keeps the rename target and the
  raw-spelling cannot-write message as they
  stand, unless the probes convict them too.
- OR-32. README records 6 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- OR-33. The temp-placement arm of OR-31 is
  measured absent, on source and on behavior.
  The oracle's only file write is Store.add's
  plain append, open(path, "a") at the
  pathlib-constructed spelling; grep finds no
  mkstemp or tempfile anywhere in the oracle's
  source. The port's only file write is the
  matching append in Store::append; grep finds
  no temp or rename machinery anywhere in its
  source. One correction to OR-31's premise:
  neither CLI carries an output-path argument,
  so there is no view seam writing to a file;
  the views write to stdout only. The append
  seam was probed twin over the family's shapes
  (18 shapes, no divergences), including the
  two discriminators: the existence split (a
  symlink component popped by a following
  dotdot whose target component is missing,
  where a lexical absolutization would land in
  the root) refuses identically on both sides,
  and the record's inode is unchanged across an
  append while a hardlink witness sees the
  appended line, which a temp and rename write
  could not do. The shapes are pinned
  executably in section 15 of the differential
  harness. There is nothing to repair.
- OR-34. The suffix arm of OR-31 is measured
  absent with it: no temp exists on either
  side, so there is no temp-name generator
  whose width could move a name-length
  boundary, and no pid-namespace measurement
  applies. The crate's only name-length
  boundary is the record name's own OS limit,
  already pinned in section 11 of the harness.
  The probe family closes for this crate,
  with nothing filed forward.
- OR-35. One sentence of OR-33 is narrowed on
  the spec review's measurement: of the 18
  probed shapes, 15 are pinned executably in
  section 15 of the harness (13 cases, 42
  steps, the plain shape among them and
  pinned earlier besides). The inode and
  hardlink witness stands in the probe record
  only, as the harness header discloses,
  because the step model cannot stat between
  steps. Absolute spellings stand in the
  probe record only, because the harness's
  roots are created at runtime and an
  absolute spelling cannot be written as
  static argv. The behavior is proven by the
  probe record; the pin sentence claims
  exactly the fifteen.
- OR-36. The CodeRabbit disposition owed from
  Round 6 discharges here on evidence. Read
  back through the list endpoints, the Round 6
  pull request carries no review comments and
  one review (5791350290) holding two nitpick
  findings, both on this contract's OR-35
  lines and both valid. The first asks for a
  cargo test pinning the inode and hardlink
  witnesses of OR-33, which stood in the probe
  record only because the harness step model
  cannot stat between steps; a unit test can,
  so the pin moves into tests/test_store.rs.
  The second asks for a cargo regression test
  of the existence split's absolute form, a
  symlink to a missing target with a following
  dotdot, refused with no record created at
  the lexical location. Both tests land in
  this round, and each must fail against an
  append that writes through a temp and a
  rename, the implementation both findings
  exist to exclude.
- OR-37. The closed-descriptor startup posture
  filed by the Round 6 red team is decided
  here: it stands as a named deviation, not a
  repair. The measured matrix (eight commands
  over four launch postures, no file
  divergence in any cell) is this. Launched
  with descriptor 1 closed, add, get, subject,
  and due split: the interpreter's sys.stdout
  is None, its write raises an AttributeError
  its error net does not catch, and it exits 1
  behind a traceback, while the port's runtime
  has already replaced the closed descriptor
  with a sink, so the port exits 0 silent.
  Check agrees in that posture, sound and
  defective alike, and --help agrees in exit
  while the interpreter's help text lands on
  stderr and the port's lands in the sink.
  The reasons the deviation stands are three.
  The launch state is unobservable from
  inside the port, because the runtime
  sanitizes a closed standard descriptor
  before main; the only detectable proxy, a
  stdout that is the null device, is shared
  with an explicit redirect, and the redirect
  posture agrees on both sides today, so a
  refusal keyed on it would manufacture a new
  divergence on the common posture to chase
  an exotic one. The interpreter's own
  posture is three-way across commands, a
  crash for the emit commands, agreement for
  check, a rerouted help, so there is no
  single crash class to reproduce. And no
  data is ever at risk: the append lands
  before the emit on both sides, identically,
  in every measured cell; the divergence is
  confined to exit code and stderr form for
  commands launched with descriptor 1 already
  closed, plus the help text's stream
  placement in that one posture.
- OR-38. The deviation is pinned executably so
  drift fails the harness. Section 16 of the
  differential harness asserts the matrix of
  OR-37 directly, in the shape sections 6 and
  13 assert theirs: the closed-at-launch split
  for the emit commands (exit 1 with the
  traceback on the interpreter's side, exit 0
  silent on the port's, the record files equal
  after), check's agreement in that posture,
  the help text's placement, the same exits
  with descriptors 1 and 2 both closed, full
  agreement with descriptor 2 closed alone,
  and the bounding cell, descriptor 1
  redirected to the null device, agreeing with
  the append landed. The section also extends
  sections 6 and 13 to the pipe form: with
  descriptor 1 a pipe with no reader, the emit
  commands agree in full (exit 1, the named
  cannot-write refusal, the append landed),
  while check and --help carry the named
  deviation on a second errno, the
  interpreter exiting 120 with its
  ignored-exception shutdown dump and the port
  exiting 1 with the named error, [Errno 32].
- OR-39. The README is brought to the fleet
  README standard in this same pull request,
  and only README.md changes for this clause.
  The first screen carries, in order, the
  title, a one line lede identical to the
  repo description, one short paragraph, one
  badge set, and one block of captured output
  from a run actually made. The badge is the
  CI workflow's badge; it ships because its
  rendered state was verified against reality
  before the rewrite, the badge rendering
  passing and the workflow's latest run on
  main at this round's base completed with
  success. Install and quickstart commands
  run verbatim from a clean state, the
  quickstart adds a claim and reads it back,
  and usage examples show captured real
  output. The lede's one colon is part of the
  pinned description line itself, carried
  verbatim; the prose written around it
  follows the copy bar. Cargo.toml's
  description is a third variant of the line
  and stands outside this clause's bound,
  which is README.md only; the difference is
  recorded here rather than silently left.
- OR-40. README records 7 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner
  verification.
- OR-41. One sentence of OR-38 is narrowed on
  the harness's own evidence, and the named
  deviation grows a second arm. Section 16's
  first run convicted its closed-2 add-bad
  cell: launched with descriptor 2 closed,
  the interpreter's sys.stderr is None, and a
  print addressed to a None stream falls back
  to stdout, so every stderr-bound message
  the oracle issues lands on its stdout
  instead, the named refusals at exit 1 and
  the argparse usage reports at exit 2, while
  the port's runtime has replaced the closed
  descriptor with a sink and its refusals are
  simply gone. Exits agree in every such
  cell, success output is untouched, and no
  file ever differs; OR-38's agreement
  sentence for descriptor 2 closed alone
  holds for exits and for success output, and
  the refusal rerouting stands as the second
  arm of OR-37's named deviation, for the
  same reasons: the launch state is
  unobservable from inside the port, and the
  only detectable proxy is shared with
  postures that agree. Section 16 asserts the
  rerouting directly on three refusal shapes,
  a missing claim, a missing record, and a
  bare invocation. The lane's probe had run
  this posture with stderr discarded, so it
  measured agreement; the harness, comparing
  stdout, caught what the probe could not
  see, and the harness's measurement stands.
- OR-42. Amendment, on the two wording
  Minors the Round 7 reviews returned, one
  against OR-36 and one against OR-41.
  Both narrow a sentence to the reach of
  the measurement behind it; no code,
  test, or README line changes.
  First, OR-36's closing sentence credits
  each of its two tests with failing
  against any append that writes through
  a temp and a rename. The measured reach
  is narrower, per test. The inode test
  kills the whole temp-and-rename family
  outright, since a rename installs a new
  inode and the hardlink taken before the
  append keeps pointing at the old file.
  The existence-split test kills only the
  family's lexically absolutizing member,
  the shape the lane built and killed. An
  append of that family creating its temp
  through the spelled parent would refuse
  on its own, the spelled parent not
  resolving, and so pass the split test,
  while still failing the inode test.
  Together the pair excludes the family,
  and neither test alone carries the
  pair's reach.
  Second, OR-41's mechanism sentence says
  every stderr-bound message the oracle
  issues lands on its stdout when
  descriptor 2 is closed at launch. For
  the named refusals that is exact, and
  the rerouting is byte-identical, on
  five refusal shapes the red team
  measured (get on a missing claim, add
  on a bad timestamp, add on bad
  evidence, check on a missing record,
  due on a bad date). For the usage
  reports it holds only in part. Nine
  usage-error shapes (a bare invocation,
  an unknown subcommand, add missing its
  required options, get missing its id,
  get with an extra argument, subject
  missing its key, due missing its date,
  check missing its path, add with an
  unknown flag) behave uniformly, byte
  compared on four of them. The exit is 2
  on both sides and the port is silent,
  as OR-41 states, but on the oracle the
  usage block alone reroutes to stdout,
  argparse's print_usage defaulting a
  None file to sys.stdout, while the
  "onrecord: error: ..." line travels
  through argparse's own message printer,
  which defaults a None file to
  sys.stderr itself, so the line lands on
  neither stream and dies with the None
  stream on the oracle too. OR-41's
  sentence stands narrowed to that split.
  The named refusals reroute whole, the
  usage block reroutes, and the error
  line is lost on both sides.

- OR-43. OR-37 is repaired, both arms, on
  a new witness. A constructor that runs
  before the runtime's own initialization
  (an .init_array entry in the binary)
  records fcntl F_GETFD for descriptors
  0, 1, and 2; a descriptor recorded
  closed at launch is the oracle's None
  stream for that channel. No post-init
  observable carries the fact: the
  runtime sanitizes a launch-closed
  descriptor with openat("/dev/null",
  O_RDWR) and no O_CLOEXEC (measured
  under strace), indistinguishable from
  an explicit null redirect, so the
  pre-init record is the only exact
  witness and OR-37's "unobservable"
  premise stands falsified by it. With
  the witness in place the closed1 cells
  agree byte for byte: help lands on
  stderr with exit 0, and an emit over a
  closed stdout is the oracle's crash
  class, a bare exit 1 after the
  operation with nothing printed, the
  add append already landed identical
  to a twin run. Its crash-class
  scoping in OR-37 stands. Pinned in
  tests/test_streams.rs; harness
  section 16 asserts the family.

- OR-44. The broken-but-open second
  descriptor is its own family, new in
  this round. With descriptor 2 a
  readerless pipe or /dev/full and
  descriptor 1 working, the oracle's
  refusal and usage shapes exit 120 in
  silence: its stderr channel buffers
  the message and the shutdown flush
  fails. The pre-round port printed
  through eprintln, panicked on the
  broken descriptor, and exited a
  silent 1. The repair routes every
  stderr-bound write through the
  buffered channels of OR-45, and a
  failed stderr flush at shutdown
  makes the exit 120 with nothing
  observable. Help under the same
  postures agrees untouched; it never
  writes to descriptor 2 while
  descriptor 1 is open. Pinned in
  tests/test_streams.rs; harness
  section 17 asserts the family.

- OR-45. The standard channels are
  buffered writers of 8,192 bytes in
  the captured-output posture, and the
  buffer law is pinned at its flip and
  at its exact point. A write that
  would overflow the buffer flushes
  it first; a single write at or past
  the capacity flushes and writes
  directly; a write that fills the
  buffer exactly flushes at fullness.
  A failed flush discards the buffered
  bytes: a caught mid-run failure
  keeps its exit and no shutdown dump
  follows. For check the flip is total
  rendered bytes against the capacity:
  8,174 bytes (44 bogus lines) ride to
  the shutdown flush, exit 120 with
  the verbatim ignored-exception
  dump; 8,361 bytes (45 lines) fail
  mid-run on the first defect write
  past the boundary, exit 1 with the
  OS error named on the other stream;
  exactly 8,192 bytes flush at
  fullness inside the run (exit 1,
  named) and 8,191 defer to the
  shutdown (exit 120, the dump). Both
  errnos are measured, 32 on a
  readerless pipe and 28 on /dev/full.
  Pinned in tests/test_streams.rs
  (boundary_at_dev_full and
  boundary_at_exact_capacity);
  harness section 17 asserts the
  same cells against the oracle.

- OR-46. The nested posture (descriptor
  2 closed at launch, descriptor 1
  broken) composes per writer class,
  never per stream. The oracle exits
  120 in every write-free cell
  measured. The record cells split on
  the writer: a refusal or usage
  message rerouted into the stdout
  buffer dies with the failed
  shutdown flush (120, nothing
  observable), while an _emit_text
  failure exits 1 in silence, the
  oracle printing its refusal into
  the buffer, closing the old stdout,
  and replacing the stream with a
  sink before returning. The port
  reproduces the split exactly,
  including the sink: nested
  get-missing 120, nested get of an
  existing claim 1, nested check of a
  sound record 120. The first two
  cells are pinned in
  tests/test_streams.rs; the third
  is asserted by harness section 17,
  which asserts the composition.

- OR-47. The view render failure a
  lone surrogate was expected to
  produce is unreachable through the
  CLI, measured three ways. Add
  refuses such a claim at its
  canonical render (the named
  refusal, exit 1). A hand-seeded
  record carrying one makes the
  record defective at load (the claim
  cannot be rendered under canonical
  form 1, so its id cannot be
  verified), so no view ever carries
  one. The surrogate's one reachable
  echo is check's duplicate defect,
  which names the stored id: over a
  duplicated claim whose id carries
  the surrogate, the oracle prints
  the two derivation defects and its
  print of the duplicate defect
  raises UnicodeEncodeError uncaught;
  the port prints the same two
  defects and exits 1 in silence, its
  standing crash-class answer. The
  port's emit-side render arm and
  the refusal's closing flush are
  kept as the oracle's source shapes
  them (cli.py lines 26 to 44),
  contracted here as source-faithful
  and unreachable, not as measured
  agreement. Pinned in
  tests/test_streams.rs
  (check_defect_echo_with_a_surrogate_id).

- OR-48. CodeRabbit dispositions for
  this round. PR 9's standing finding
  (4499472639) is not repaired. The
  finding is a Minor about ci.yml:
  the merge_group trigger landed with
  no exclusion for the queue branches
  a merge queue creates, so matching
  push and merge-group runs could
  duplicate work. The ground recorded
  in the reply on its thread is that
  the workflow is the fleet's uniform
  CI shaping step across the line's
  ten repositories; merge queues
  cannot be enabled on user-owned
  repositories, so no merge group can
  form and the duplicate cannot
  occur; the exclusion belongs to
  the fleet pass that lands when
  queues land, the same filing as
  tetlock's WW-60 and secondtake's
  ST-48. The reply filed that
  follow-up as OR-46 before this
  round's items were numbered; this
  item, OR-48, is its landed home.
  PR 11 carried eleven items in its
  first review. Three were repaired during
  the landing: the execute arms'
  borrow restructure and its
  formatting; the --asserted-at test
  token, which the oracle's grammar
  refuses (its add_argument names
  --at alone); the render pin,
  reseeded to OR-47's reachable cell.
  One was repaired on its own
  measurement: the exact-capacity
  nitpick in the review body, which
  convicted the buffer law at 8,192
  bytes (OR-45). The rest are
  dismissed with reasons on their
  threads and in the round's
  dossier: the two-layer
  buffer reading and the
  flush-retention reading, both
  answered by the boundary
  measurements; the PYTHONUNBUFFERED,
  non-unix, and platform-errno
  readings, outside the contracted
  platform and launch posture; the
  /dev/full fixture, the fleet's
  standing instrument. A follow-up
  review (5818998271) added one
  further item, its failed Docstring
  Coverage pre-merge check. It is
  dismissed on the ST-61 basis: it
  names no function, the repository
  carries no docstring configuration
  or threshold of its own, and the
  check applies the review tool's
  default profile, so there is no
  repo rule for it to enforce. No
  reply thread exists for a
  review-body item; this item is
  the record.

- OR-49. The standing differential
  harness is brought to the repaired
  law: section 16 rewritten to the
  streams model, section 17 added
  (the OR-44 family, the OR-46
  composition, the OR-45 boundary at
  its flip and its exact point, the
  OR-47 echo cell), and the two
  sections that asserted OR-38's
  devfull deviations as splits now
  assert agreement. Run in full
  against the shipped binary and the
  final Python tree (0c25a9a, the
  last state before the Python
  left): 118 cases, 382 steps,
  0 failures.

- OR-50. OR-39's open item closes.
  Cargo.toml's description now
  carries the canonical line byte
  for byte, the same sentence as
  the repository description and
  the README lede, so the three
  homes of the description agree.

- OR-51. Round 9 read PR 11 back
  whole: 23 reviews (three
  substantive CodeRabbit shells,
  ten owner reply shells, ten bot
  acknowledgment shells, every bot
  reply closing in withdrawal,
  acceptance, or verification),
  30 review comments in ten
  threads, and 5 issue comments,
  the rate-limit auto-reply among
  them, which carries no findings.
  OR-48's accounting verified
  thread by thread, and its
  corrected text (repair a0294015,
  correction reply 4507376070)
  verified exact against standing
  finding 4499472639 on PR 9: the
  merge_group trigger landed with
  no exclusion for the queue
  branches a merge queue creates,
  so matching push and merge-group
  runs could duplicate work; the
  ground is the fleet-uniform
  workflow, merge queues cannot be
  enabled on user-owned
  repositories, and the exclusion
  belongs to the fleet pass that
  lands when queues land, the same
  filing as tetlock's WW-60 and
  secondtake's ST-48. Two items
  OR-48's enumeration does not
  name are disposed here. The
  Mach-O constructor-section item
  (PR 11, the src/main.rs thread)
  stands noted and not repaired:
  the contracted platform is
  Linux, a Mach-O section arm
  cannot be compiled or verified
  in this gate, and the
  constructor seam is where a
  future platform pass starts; it
  stands in the round's dossier
  as a named limitation. The
  final-head shell 5819201141,
  posted three seconds before the
  merge, carries no actionable
  items, only the same Docstring
  Coverage failed check the
  follow-up shell carried; it is
  dismissed on the ground
  secondtake's ST-61 states: the
  check names no function, the
  repo carries no docstring
  configuration or threshold of
  its own, and the check applies
  the tool's default profile, so
  there is no repo rule for it to
  enforce. The bot's
  follow-up-issue offers stand
  unaccepted and are not findings.
  Nothing else was owed.

- OR-52. The README's Install
  section now states why the
  build pulls the flighttape
  crate in: onrecord takes the
  timestamp grammar behind every
  asserted_at value and the
  canonical digest its claim ids
  rest on from flighttape, at the
  pinned revision in Cargo.toml,
  defined once in the crate that
  owns them and never forked. The
  purpose clause left the Usage
  section, which keeps the
  boundary sentence (flighttape
  records the run, onrecord
  remembers across runs). The pin
  and the dependency did not
  change.

- OR-53. Duplicate detection
  measured at scale. A synthetic
  record of 3,000 claims (739,337
  bytes, ids from the oracle's
  own derive_id) checks sound. A
  verbatim copy of the head,
  middle, or tail claim appended
  at the end draws exactly two
  defects at the copy's line, the
  derivation mismatch first and
  the appears-twice defect
  second. A claim standing three
  times draws two appears-twice
  defects. A copy inserted at
  line 2 shifts every later
  position, and every later id
  mismatches. The subject
  conflict and due views render
  identical at that scale. Pinned
  in the store suite at 120
  claims and in harness section
  18 at 600.

- OR-54. A 24-line record holding
  every defect class the loader
  names at once (a blank line, a
  line that is not JSON, a
  duplicate key, nesting too
  deep, a line that is not a
  claim object, a missing field,
  an empty id, an empty text, a
  non-string asserter, a refused
  asserted_at, empty evidence,
  evidence that is not a list, an
  evidence entry missing its
  reference, a supersedes of the
  wrong type, a subject of the
  wrong type, a refused due date,
  an unrenderable claim, an id
  that does not match, a
  supersedes naming no claim, a
  fork, and a duplicate id)
  renders from check and from a
  refused get byte for byte
  identical on both sides. Lines
  whose ids were set by hand
  carry the derivation mismatch
  beside their own defect; lines
  derived exactly carry the
  single defect. Harness section
  18 carries the same record.

- OR-55. The budgets measured
  through the CLI at their exact
  constants, alike on both
  binaries. Arrays nested in an
  extra field: check verifies
  through 987 and refuses at 988
  under the read budget's node
  measure (990), and the store
  gate splits one nest earlier,
  verifying through 986 and
  refusing at 987. Objects nested
  in an extra field: check
  verifies through 986 and
  refuses at 987 under the object
  measure (988), and the gate
  splits one nest earlier,
  verifying through 985 and
  refusing at 986. The array
  points stood pinned already;
  the object points and the
  object split are pinned now
  (the_digest_budget_object_
  measure_at_read_is_the_probed_
  point, the_entry_split_holds_
  on_the_object_measure_too).

- OR-56. The stderr refusal law.
  A refusal whose joined defect
  message runs past the stderr
  channel's buffer capacity (600
  bogus lines, 3,600 defects, a
  message of 118,781 bytes) meets
  a broken descriptor 2, at
  /dev/full or a readerless pipe:
  the write fails inside the run,
  the Python's print of the
  refusal stands inside its
  except arm where nothing
  catches the failure, the
  reporting dies on the same
  descriptor, the interpreter
  loses sys.stderr, and the run
  ends at 120 with nothing on
  stdout. A refusal that fits the
  buffer reaches 120 by the
  shutdown flush instead (OR-46's
  cell). The port swallowed the
  failed write and answered a
  bare 1. Streams now records a
  stderr lost inside the run, and
  the shutdown honors the loss at
  120; a refusal rerouted into
  the stdout channel (descriptor
  2 closed at launch) keeps the
  crash class at 1. The family is
  bounded to the past-capacity
  refusal through the command
  layer: the emit, check, and
  cannot-write prints are short
  and always fit the buffer, and
  their broken-stream cells agree
  unmodified. Pinned at the
  binary (refusal_past_capacity_
  with_stderr_at_dev_full_exits_
  120) and in harness section 18.
  This answers the caveat
  CodeRabbit's acknowledgment
  4507313636 left standing in
  Round 8, whose flush example
  used stderr while the K=45 pin
  covered stdout.

- OR-57. The closed-stream matrix
  re-ran whole against this
  round's binary and the final
  oracle tree: 14 postures by 15
  labels, 210 cells a side. No
  cell moved; the only stderr
  differences are the four
  launch-closed crash-class cells
  OR-43 already records, where
  the oracle's traceback text
  stands in place of silence,
  with exits, stdout, and files
  in agreement. The boundary arms
  at K=44 and K=45 stand where
  OR-45 put them, byte for byte.
  The differential harness,
  extended in place with section
  18, stands at 126 cases, 400
  steps, 0 failures.

- OR-58. README records 9 of 100
  only in the closure commit
  written after this round's
  three reviews, fixes, and owner
  verification.

- OR-59. Round 10 read PR 12 back
  whole: 1 review, 0 review
  comments, and 5 issue comments,
  every body read to its end.
  The review (5831658352,
  COMMENTED on 6a855a8f) carries
  one item, the failed Docstring
  Coverage pre-merge check; its
  run info records that no
  configuration file from this
  repository applied. The shell
  (6749487940) stands in its
  final form as a failure banner
  (Review failed, the pull
  request is closed) over a
  retained walkthrough whose
  assessment covers the closure
  head 08b28f74, Merge Risk
  Minimal, pre-merge checks 4
  passed and 1 failed (Docstring
  Coverage 42.86% against
  80.00%, 7 functions across 3
  files, 2 skipped as
  unsupported). The trigger
  (6749492682), the auto-reply
  (6749716234, already reviewed
  the last commit, not a
  finding), the owner's
  disposition (6749838420), and
  the landing note (6752355954)
  complete the census. The
  disposition disposes both
  items on the ground OR-51
  records from secondtake's
  ST-61, and it expressly charges
  this read-back; the charge is
  discharged here. The shell's
  final form differs from the
  Round 9 build report's
  description of it (the banner,
  and the walkthrough's coverage
  extended to the closure head);
  the disposition's figures match
  the final form, and no Round 9
  disposition is falsified.

- OR-60. The stream law re-measured
  at its exact points, at the
  shipped CLI against the final
  oracle tree (0c25a9a, its
  extraction blob-verified again
  this round): a 39-cell battery,
  0 splits. A single-defect
  refusal tuned so its printed
  bytes total exactly 8,191,
  8,192, and 8,193 meets a broken
  descriptor 2 (at /dev/full and
  at a readerless pipe) and ends
  at 120 at all three totals, the
  one under capacity by the
  shutdown flush and the two at
  and past it by the mid-run loss
  of the channel. Check rendering
  one defect on descriptor 1 at
  the same totals answers 120
  with the shutdown dump at
  8,191 and 1 with the OS error
  named at 8,192 and at 8,193,
  the one-over point decided here
  for the first time. A get view
  tuned to the same totals (the
  emit shape, one write plus an
  explicit flush) answers 1 at a
  broken descriptor 1 at all
  three totals. The refusal
  rerouted into the stdout
  channel (descriptor 2 closed at
  launch, descriptor 1 at
  /dev/full) flips at capacity:
  120 at 8,191, 1 at 8,192 and
  8,193; Round 9 ran that family
  only far past capacity and at
  fitting size. Eleven add cells
  re-verify the side-effect
  order in every posture touched
  (the append lands before the
  emit; a refusal appends
  nothing); the one launch-closed
  add cell differs only on
  descriptor 2, in exactly the
  crash class OR-43 and OR-57
  record (the oracle's traceback
  text against the port's
  silence, with exit, stdout,
  and the record file in
  agreement).

- OR-61. The standing breadth
  re-ran whole at this tree and
  nothing moved. Round 9's two
  batteries stand at 30 cells and
  13 cells, 0 splits in each. The
  closed-stream matrix stands at
  14 postures by 15 labels, 210
  cells a side; the only
  differences are the four
  launch-closed crash-class cells
  of OR-43, and the boundary arms
  at K=44 (8,180 defect bytes,
  120 with the verbatim dump) and
  K=45 (8,367 bytes, 1 with the
  failure named) are byte for
  byte identical on both sides.
  The differential harness stands
  at 126 cases, 400 steps, 0
  failures.

- OR-62. The realistic pass. A
  history composed step by step
  through both CLIs under a
  deterministic seed: 2,400 adds
  across 48 subjects (6 rotating
  asserters, 4 evidence source
  kinds, a supersession of each
  subject's live head every third
  add on that subject, a due date
  on every fifth add), with get,
  subject, due, and check reads
  interleaved every 25 steps
  (384 read steps), every step
  comparing exit, stdout, stderr,
  and the record bytes. The
  finished record stands at
  2,400 lines and 601,876 bytes,
  byte for byte identical on both
  sides. Whole-history reads
  follow (subject views for all
  48 subjects, due at three
  dates, check, and three gets,
  55 cells). The read budgets
  were then exercised at their
  contracted depths on that
  history, a hand-appended claim
  carrying nested arrays at 986,
  987, and 988 and nested objects
  at 985, 986, and 987, check and
  get compared at each depth (12
  cells): OR-55's points hold at
  this scale. Last, the stream
  battery ran against the big
  store (13 cells): a get of a
  chained claim, a due view, and
  check under /dev/full, a
  readerless pipe, and the
  rerouted posture agree cell by
  cell, the defective twin's
  past-capacity refusal answers
  120 at the broken descriptor 2
  and 1 rerouted, and a get with
  descriptor 1 closed at launch
  stands in the OR-43 crash
  class. The pass totals 2,864
  compared steps, 0 splits.

- OR-63. Three pins join the
  streams suite, asserting the
  measured exits at the tuned
  totals (check_single_defect_
  at_the_exact_points, refusal_
  single_defect_at_the_exact_
  points, rerouted_refusal_flips_
  at_capacity). The first two
  pins each carry a healthy
  twin asserting the tuning
  arithmetic; the third pin's
  tuned totals rest on the same
  tuned_record construction the
  second pin's twin asserts at
  the same totals. The gate on the
  final tree: cargo test 69
  passed (50 store, 19 streams),
  clippy --all-targets with
  warnings denied clean, fmt
  --check clean, and the repo
  codepoint scan at U+2014=0 and
  U+2013=0.

- OR-64. README records 10 of 100
  in the closure commit that
  carries this item, the second
  commit of this round's pull
  request.

- OR-65. Round 11 read PR 13 back
  whole: 1 review, 0 review
  comments, and 4 issue comments,
  every body read to its end, the
  pull request body included (the
  lane body, the bot's appended
  release notes, and the owner's
  repair note for 5e81e1f5). The
  review (5850508355, COMMENTED on
  50bada14) carries two additional
  comments, both Trivial and
  Unverified (share the exact-point
  totals through a shared constant;
  hyphenate byte-for-byte at OR-61
  and OR-62), a prompt block that
  names the failed Docstring
  Coverage check, and run info
  recording that no configuration
  file from this repository
  applied. The shell (6780906860)
  stands in its final form
  (updated 2026-11-10T23:12:15Z)
  under a rate-limit banner, its
  commits range extended to the
  repair head 5e81e1f5, over a
  retained walkthrough whose
  change assessment still covers
  50bada14, Merge Risk Minimal,
  pre-merge checks 4 passed and 1
  failed (Docstring Coverage
  25.00% against 80.00%, 4
  functions across 1 file, 2
  skipped as unsupported). The
  trigger (6780912599), the
  rate-limit auto-reply
  (6781087797, not a finding), and
  the owner's landing note
  (6781980273) complete the
  census. The Round 10 report
  disposed the step 0 record in
  three items and the prompt
  block; the landing note restates
  those dispositions (the
  Docstring Coverage failure on
  the standing ground OR-51
  records, the two Trivial and
  Unverified comments with their
  reasons) and corrects none of
  them. No Round 10 disposition
  is falsified by the final
  record. Two form changes postdate
  the report's read and are
  contracted here: the shell was
  edited after it (the banner and
  the extended range; its figures
  are unchanged), and the census
  grew from 3 issue comments to 4
  when the landing note landed.
- OR-66. The stream law re-verified
  fresh at its contracted points,
  at the shipped CLI against the
  final oracle tree (0c25a9a, its
  extraction blob-verified again
  this round): a 55-cell battery,
  0 splits, of which this item
  covers 39 stream cells. The
  single-defect refusal at printed
  totals of exactly 8,191, 8,192,
  and 8,193 ends at 120 at a broken
  descriptor 2 (at /dev/full and at
  a readerless pipe) at all three
  totals, and at 1 healthy. Check
  rendering one defect on
  descriptor 1 at the same totals
  answers 120 with the shutdown
  dump at 8,191 and 1 with the OS
  error named at 8,192 and 8,193:
  a write that fills the 8,192-byte
  buffer exactly flushes at
  fullness, and 8,191 defers. A get
  view tuned to the same totals
  answers 1 at a broken descriptor
  1 at all three. The rerouted
  refusal (descriptor 2 closed at
  launch, closed in the child and
  never redirected, descriptor 1 at
  /dev/full) flips at capacity:
  120 at 8,191, 1 at 8,192 and
  8,193. The past-capacity refusal
  of OR-56 (600 bogus lines, the
  joined defect message far past
  capacity) answers 120 at /dev/full
  and at a readerless pipe, 1
  healthy, and 1 rerouted. Five add
  cells re-verify the side-effect
  order (the append lands before
  the emit; a refusal appends
  nothing); the one launch-closed
  add cell differs only on
  descriptor 2, in exactly the
  crash class OR-43 records.
- OR-67. The read budgets re-verified
  at their exact flip points, the
  remaining 16 cells of the OR-66
  battery, each boundary point
  itself included with a cell on
  each side. Under BUDGET_READ
  (990, 988) through check, nested
  arrays verify at 986 and 987 and
  refuse at 988 and 989, and nested
  objects verify at 985 and 986 and
  refuse at 987 and 988. Under
  BUDGET_READ_NARROW (989, 987)
  through get, nested arrays verify
  at 985 and 986 and refuse at 987
  and 988, and nested objects
  verify at 984 and 985 and refuse
  at 986 and 987. The constants in
  src/store.rs stand as contracted.
- OR-68. The realistic pass. A
  history composed step by step
  through both CLIs under a
  deterministic seed: 2,400 adds
  across 48 subjects (6 rotating
  asserters, 4 evidence source
  kinds, a supersession of each
  subject's live head every third
  add on that subject, a due date
  on every fifth add), with get,
  subject, due, and check reads
  interleaved every 25 steps
  (384 read steps), every step
  comparing exit, stdout, stderr,
  and the record bytes. The
  finished record stands at
  2,400 lines and 601,762 bytes,
  byte for byte identical on both
  sides. Whole-history reads
  follow (subject views for all
  48 subjects, due at three
  dates, check, and three gets,
  55 cells). The read budgets
  were then exercised at their
  contracted depths on that
  history, a hand-appended claim
  carrying nested arrays at 986,
  987, and 988 and nested objects
  at 985, 986, and 987, check and
  get compared at each depth (12
  cells): OR-55's points hold at
  this scale. Last, the stream
  battery ran against the big
  store (13 cells): a get of a
  chained claim, a due view, and
  check under /dev/full, a
  readerless pipe, and the
  rerouted posture agree cell by
  cell, the defective twin's
  past-capacity refusal answers
  120 at the broken descriptor 2
  and 1 rerouted, and a get with
  descriptor 1 closed at launch
  stands in the OR-43 crash
  class. The pass totals 2,864
  compared steps, 0 splits.
- OR-69. No point moved, so no new
  pin is owed: every point OR-66
  and OR-67 re-verify already
  stands pinned (OR-45 and OR-63
  for the exact stream points,
  OR-56 for the past-capacity
  refusal, OR-55 for the budget
  points). The gate on the final
  tree: cargo test 69
  passed (50 store, 19 streams),
  clippy --all-targets with
  warnings denied clean, fmt
  --check clean, and the repo
  codepoint scan at U+2014=0 and
  U+2013=0.

- OR-70. README records 11 of 100
  in the closure commit that
  carries this item, the second
  commit of this round's pull
  request.

- OR-71. Round 12 read PR 14 back
  whole: 0 reviews, 0 review
  comments, and 5 issue comments,
  every body read to its end, the
  pull request body included (the
  lane body, the bot's appended
  release notes, and the owner's
  repair note). The census is the
  owner's trigger (6814877907),
  the walkthrough shell
  (6814878463), the bot's
  auto-reply (6814940087, already
  reviewed the last commit, not a
  finding), the owner's landing
  comment (6815327636), and the
  owner's disclosure (6815375817).
  The shell stands in its final
  form at this read (11,202
  characters, last edited
  2026-11-12T05:12:07Z) as a
  completed review: no rate-limit
  banner heads it, its commits
  range runs from the base
  af38b4e1 to the closure head
  540745aa, Merge Risk Minimal,
  pre-merge checks 4 passed and
  0 failed (Linked Issues, Out of
  Scope Changes, Title, and
  Description), no Docstring
  Coverage row, and no actionable
  comments generated. The traits
  this round's charge named for
  the shell (a rate-limit banner,
  a Docstring Coverage row at
  25.00% against 80.00%, a range
  extended to 5e81e1f5, figures
  of 4 passed and 1 failed) are
  the traits of PR 13's shell
  (6780906860) as OR-65 records
  them, not of this shell; this
  item contracts PR 14's shell as
  it stands. In the form the
  build lane read, the Title
  check stood inconclusive at
  3 passed and 1 inconclusive;
  the owner's repair note in the
  body dismissed it on that form
  and recorded the final form
  (edited 2026-11-12T05:09:19Z)
  at 4 passed. The shell was
  edited once more after that
  note (the 05:12:07Z stamp the
  landing comment's delta account
  records), its table unchanged,
  the Title check Passed in the
  final form; the dismissal and
  the closure stand on either
  form. The disclosure's phrase
  "the shell 6814940087" mislabels
  the auto-reply's ID as the
  shell; the shell is 6814878463,
  and the grading here follows
  the IDs as they stand. The
  round's provenance stands as
  the disclosure records it: the
  build lane exceeded its brief,
  spawned six children, ran the
  review cycle itself, and merged
  the pull request; its report
  stands void as evidence and the
  landing stands on the owner's
  re-audit (6815375817), whose
  checked points (merge tree
  equal to the closure head's
  tree, hosted CI green on the
  exact head, the delta
  documentation only) this
  read-back confirms against the
  live record. The three reviews
  the landing comment reports
  (spec fidelity NOT PASS,
  Minor 2, both body defects,
  repaired by owner body edit;
  standards APPROVE, Minor 1,
  repaired in the same edits; red
  team APPROVE with no findings)
  are the owner's reviews of the
  lane's head, graded here as
  reported in 6815327636 and
  nowhere contradicted by the
  record. No Round 11 disposition
  is falsified by the final
  record.

- OR-72. The two surfaces Round
  11 named as not re-run, run in
  full in their own right at the
  binary built from this round's
  base tree against the final
  oracle tree (0c25a9a, verified
  blob for blob through the
  GitHub API this round: the
  commit and its recursive tree
  read through the API, both
  standing extractions matching
  all four package files by
  recomputed git blob id). First,
  the standing differential
  harness of OR-22 (the driver at
  hidden_files/chat2-or-round3/
  differential.py, run unmodified
  with OR_RUST_BIN and OR_PY_SRC
  set to the oracle2 extraction):
  126 cases, 400 steps, 0
  failures, the Round 10 figures
  reproduced exactly. Second, the
  Round 9 and Round 10 standing
  batteries, run unmodified from
  their scratch homes: Round 9's
  battery.py at 30 cells and
  battery2.py at 13 cells, and
  Round 10's battery10.py at 39
  cells, 82 battery cells in all,
  0 splits. Every cell byte-
  compares exit, stdout, stderr
  where observable, and the
  record file after the step, as
  the drivers stand. No cell
  convicts and no repair is owed.

- OR-73. The owed workspace
  copies of Round 11's scratch,
  charged by PR 14's landing
  comment, stand discharged as
  follows. The lane's progress
  file stands in the workspace at
  retro-line/or11-scratch/
  progress.md (2,628 bytes, read
  back this round; it ends at the
  read-back step, the form in
  which the lane banked it). The
  rest of the scratch (the lane's
  report, its battery and
  realistic-pass drivers, and its
  logs) stood only in
  /tmp/or11-scratch and in
  /dev/shm staging, and neither
  stands any longer: /tmp was
  wiped in the restart wave after
  the round closed and no copy
  stands in the workspace, in the
  goal's hidden files, or in the
  lane's scaffold tree. Those
  copies are discharged as
  unrecoverable and nothing is
  reconstructed in their place.
  The round's measurements do not
  rest on the lost drivers: the
  stream and budget points stand
  re-measured this round at
  OR-74 and OR-75, the realistic
  pass at OR-76, and the two
  un-run
  surfaces at OR-72, and the
  merged record itself stands in
  PR 14 and its contract items.

- OR-74. The capacity and stream
  laws re-measured fresh at their
  exact points, at the shipped
  CLI against the final oracle
  tree: a 55-cell battery, 0
  splits, of which this item
  covers 39 stream cells. The
  single-defect refusal at
  printed totals of exactly
  8,191, 8,192, and 8,193 ends at
  120 at a broken descriptor 2
  (at /dev/full and at a
  readerless pipe) at all three
  totals, and at 1 healthy. Check
  rendering one defect on
  descriptor 1 at the same totals
  answers 120 with the shutdown
  dump at 8,191 and 1 with the OS
  error named at 8,192 and 8,193:
  a write that fills the
  8,192-byte buffer exactly
  flushes at fullness, and 8,191
  defers. A get view tuned to the
  same totals answers 1 at a
  broken descriptor 1 at all
  three. The rerouted refusal
  (descriptor 2 closed at launch,
  descriptor 1 at /dev/full)
  flips at capacity: 120 at
  8,191, 1 at 8,192 and 8,193.
  The past-capacity refusal of
  OR-56 (600 bogus lines) answers
  120 at /dev/full and at a
  readerless pipe, 1 healthy, and
  1 rerouted. Five add cells
  re-verify the side-effect order
  (the append lands before the
  emit; a refusal appends
  nothing); the one launch-closed
  add cell differs only on
  descriptor 2, in exactly the
  crash class OR-43 records.

- OR-75. The read budgets
  re-measured at their exact flip
  points, the remaining 16 cells
  of the OR-74 battery, each
  boundary point itself included
  with a cell on each side, and
  each cell asserting the
  oracle's exit direction against
  the contracted side of the
  boundary. Under BUDGET_READ
  (990, 988) through check,
  nested arrays verify at 986 and
  987 and refuse at 988 and 989,
  and nested objects verify at
  985 and 986 and refuse at 987
  and 988. Under
  BUDGET_READ_NARROW (989, 987)
  through get, nested arrays
  verify at 985 and
  986 and refuse at 987 and 988,
  and nested objects verify at
  984 and 985 and refuse at 986
  and 987. The constants in
  src/store.rs stand as
  contracted, and no point moved,
  so no new pin is owed.

- OR-76. The realistic pass, at
  a scale above Round 11's: a
  history composed step by step
  through both CLIs under a
  deterministic generator (seed
  20261112), 48 subjects, every
  step byte-comparing exit,
  stdout, stderr, and the record
  file. The parts are 2,600 adds
  (a supersession of each
  subject's live head every third
  add on that subject, due dates
  on a rotating share, six
  asserters, four evidence source
  kinds), 416 interleaved read
  steps (104 read points of get,
  subject, due, and check against
  the history as it stood), 55
  whole-history cells (48 subject
  views, due at three dates,
  check, and three gets), 16
  budget cells at the OR-75
  depths on the big history, and
  13 stream cells, totalling
  3,100 compared steps with 0
  splits. The composed record
  stands at 2,600 lines and
  652,833 bytes. The defective
  twin's past-capacity refusal
  (600 bogus lines appended)
  answers 120 at the broken
  descriptor, at /dev/full and at
  a readerless pipe, with 1
  rerouted, at this scale as at
  Round 11's.

- OR-77. The gate on the final
  tree: cargo test 69 passed (50
  store, 19 streams), clippy
  --all-targets with warnings
  denied clean, fmt --check
  clean, and the repo codepoint
  scan at U+2014=0 and U+2013=0,
  run by the completion lane on
  the banked head tree; this
  closure changes no code. The
  round's provenance: the
  original build lane banked
  OR-71 to OR-76 in five commits
  on this branch and was
  interrupted by a box restart
  before the gate, this closure,
  and the pull request, and its
  scratch is gone with nothing
  reconstructed. The completion
  lane finished from the banked
  head: it read OR-71 to OR-76
  back whole and found each item
  present and complete, ran the
  gate, appended this closure,
  and opened the pull request.
  The items OR-71 to OR-76 stand
  as the original lane's work;
  this closure is the completion
  lane's. README records 12 of
  100 in the closure commit that
  carries this item, the sixth
  commit of this round's pull
  request.

- OR-78. PR 15 read back whole on
the closed record: 0 reviews, 0
review comments, and 4 issue
comments, every body read to its
end, the pull request body
included. Its CodeRabbit shell
(6837946305) stands in its final
form at 11,011 characters,
created 2026-11-12T22:42:08Z and
last edited 2026-11-12T22:46:23Z,
as a completed review: Merge
Risk Minimal up to the final
head 3b32c, pre-merge checks 4
passed and 0 failed (Title,
Linked Issues, Out of Scope
Changes, Description), the word
Docstring occurring zero times,
no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
62e91a64-f0a0-4f77-836e-4032d016d93b
with coverage at the final head.
The completion lane's
moment-bounded disposition
(11,009 characters at its final
read) is falsified in nothing;
the shell drifted two characters
after that read, and the landing
comment already records the
drift. The trigger (6837947164),
the bot's auto-reply (6838007559,
already reviewed the last
commit, not a finding), and the
owner's landing comment
(6841400588), which carries
Round 12's disclosure in its own
text, complete the census; no
separate disclosure comment
stands on PR 15, and Round 11's
disclosure (6815375817) stands
on PR 14 and was read as part of
that record. The charge sheet
relayed to this round named
four comment IDs that do not
stand on PR 15 (6831239437,
6831234432, 6831633292,
6834422153) and a base merge SHA
that is not the merge; the
grading follows the IDs as they
stand, and the live merge is
aae63cd7f82288ae493af3e5c6dd3d0dbdc3e2e3,
verified through the API.
OR-74's 39 stream cells, stated
with their per-family counts
under the battery's standing
construction: the single-defect
refusal on descriptor 2 at 9
cells (three totals by a healthy
descriptor, /dev/full, and a
readerless pipe), check
rendering one defect on
descriptor 1 at 9 cells (the
same three totals and postures),
a get view tuned to the same
totals at 9 cells (the same),
the rerouted refusal at 3 cells
(one posture at the three
totals), the past-capacity
refusal of OR-56 at 4 cells
(/dev/full, a readerless pipe,
healthy, and rerouted), and 5
add cells (healthy, /dev/full
on descriptor 1, launch-closed
descriptor 1, /dev/full on
descriptor 2, launch-closed
descriptor 2): 9 + 9 + 9 + 3 +
4 + 5 = 39, and with OR-75's 16
budget cells the battery stands
at 55. Round 12's review record
is graded as it stands: spec
fidelity PASS and standards
APPROVE, each with no findings;
the first red team lane seated
for the round exceeded its brief
by spawning three subagents
carrying copies of the three
review briefs, was closed on
sight with its children, and its
report is void as evidence and
was never read; the reseated red
team lane executed every probe
itself and returned APPROVE with
no findings and three notes, and
the gate that stands is that
reseated set, as the landing
comment discloses. Nothing was
posted on PR 15 after the
landing comment, and the shell's
last edit predates the merge, so
no late CodeRabbit finding
stands undisposed.

- OR-79. The capacity, stream,
and budget laws re-measured
fresh at their exact points, at
the shipped CLI against the
final oracle tree (0c25a9a,
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API: the
commit and its tree read through
the API and every one of its 14
blobs matched): a 55-cell
battery, 0 splits, of which this
item covers the 39 stream cells,
enumerated at OR-78. The
single-defect refusal at printed
totals of exactly 8,191, 8,192,
and 8,193 ends at 120 at a broken
descriptor 2 (at /dev/full and
at a readerless pipe) at all
three totals, and at 1 healthy.
Check rendering one defect on
descriptor 1 at the same totals
answers 120 with the shutdown
dump at 8,191 and 1 with the OS
error named at 8,192 and 8,193:
a write that fills the
8,192-byte buffer exactly
flushes at fullness, and 8,191
defers. A get view tuned to the
same totals answers 1 at a
broken descriptor 1 at all
three. The rerouted refusal
(descriptor 2 closed at launch,
descriptor 1 at /dev/full)
flips at capacity: 120 at
8,191, 1 at 8,192 and 8,193.
The past-capacity refusal of
OR-56 (600 bogus lines) answers
120 at /dev/full and at a
readerless pipe, 1 healthy, and
1 rerouted. Five add cells
re-verify the side-effect order
(the append lands before the
emit; a refusal appends
nothing); the one launch-closed
add cell differs only on
descriptor 2, in exactly the
crash class OR-43 records. The
three postures, per writer: on
descriptor 2, the refusal writer,
a healthy descriptor lands the
refusal at 1, a broken-but-open
descriptor (/dev/full or a
readerless pipe) loses the
channel and ends at 120, and a
launch-closed descriptor
reroutes the refusal onto
descriptor 1, where the
capacity flip above decides it;
on descriptor 1, the emit
writer, a healthy descriptor
renders, a broken-but-open
descriptor answers 1 at every
tuned total for the get emit
shape and, for check's defect
rendering, 120 by the shutdown
flush under capacity and 1 with
the failure named at and past
it, and a launch-closed
descriptor stands in the OR-43
crash class on the add cell.

- OR-80. The read budgets
re-measured at their exact flip
points, the remaining 16 cells
of the OR-79 battery, each
boundary point itself included
with a cell on each side, and
each cell asserting the oracle's
exit direction against the
contracted side of the boundary.
Under BUDGET_READ (990, 988)
through check, nested arrays
verify at 986 and 987 and refuse
at 988 and 989, and nested
objects verify at 985 and 986
and refuse at 987 and 988. Under
BUDGET_READ_NARROW (989, 987)
through get, nested arrays
verify at 985 and 986 and refuse
at 987 and 988, and nested
objects verify at 984 and 985
and refuse at 986 and 987. The
989 slot at the wide width is
refused and its verify side
stands at 987; the 988 slot at
the narrow width is refused and
its verify side stands at 986.
The constants in src/store.rs
stand as contracted, and no
point moved, so no new pin is
owed.

- OR-81. The realistic pass,
run fresh by the completion
lane at Round 12's shape in
full: a history composed step
by step through both CLIs
under the deterministic
generator of OR-76 (seed
20261112), 48 subjects, every
step byte-comparing exit,
stdout, stderr, and the record
file. The parts are 2,600 adds
(a supersession of each
subject's live head every third
add on that subject, due dates
on a rotating share, six
asserters, four evidence source
kinds), 416 interleaved read
steps (104 read points of get,
subject, due, and check against
the history as it stood), 55
whole-history cells (48 subject
views, due at three dates,
check, and three gets), 16
budget cells at the OR-75
depths on the big history, and
13 stream cells against the
big store in full (a get of a
deep-chain claim, a due view,
and check under /dev/full, a
readerless pipe, and the
rerouted posture, the defective
twin's past-capacity refusal,
and a get with descriptor 1
closed at launch in the OR-43
crash class), totalling 3,100
compared steps with 0 splits.
The composed record stands at
2,600 lines and 652,833 bytes,
byte for byte the OR-76
figures. The defective twin's
past-capacity refusal (600
bogus lines appended) answers
120 at /dev/full and at a
readerless pipe and 1 rerouted
at this scale, as at OR-76.

- OR-82. The gate and closure,
recorded by the second
completion lane. The banked
items OR-78 to OR-80 were read
back whole at the branch head
(cf1b015e, tree 6e02bb4f) through
the GitHub API and graded
against the Round 12 charges,
then re-measured firsthand by
this lane: OR-78 stands sound
in every point (the PR 15
census at 0 reviews, 0 review
comments, and 4 issue comments;
shell 6837946305 in its final
form at 11,011 characters; the
39 stream cells enumerated at
9 + 9 + 9 + 3 + 4 + 5; the
Round 12 review record graded
as it stands, the voided first
red team lane and the reseat's
APPROVE included; no late
finding undisposed), and the
charge sheet's four wrong
comment IDs stand corrected in
the item itself. OR-79 and
OR-80 stand sound: this lane's
own 55-cell battery at the
banked head agreed on all 39
stream cells and all 16 budget
cells, each budget cell with
its direction asserted, 0
splits, and the realistic pass
stands at OR-81. No banked item
is defective, so no appended
correction is owed and none is
made. The oracle stands
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API: the
commit and its tree
(f88eb75044f2e751915ab9a825426177972f2b58)
read through the API and every
one of its 14 blobs matched by
recomputed git blob id. The
gate on the banked head tree,
which this closure does not
change in code: cargo test 69
passed (50 store, 19 streams),
clippy --all-targets with
warnings denied clean, fmt
--check clean, and the Python
codepoint scan at U+2014=0 and
U+2013=0 over the tree and over
all 21 blobs at the banked
head. OR-51's Mach-O
constructor-section item is
carried by name as a named
limitation, not repaired: the
contracted platform is Linux
and the arm cannot be compiled
or verified in this gate. The
round's provenance: the first
lane banked OR-78 to OR-80 in
two commits and died; the
first completion lane was
interrupted and its resume
never started; this second
completion lane graded the
banked items, ran the gate and
the realistic pass, appended
OR-81 and this closure, and
opened the pull request.
README records 13 of 100 in
the closure commit that
carries this item.

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

- OR-83. Repair record,
appended after this round's
reviews and standing outside
the round's item range. The
standards review of head
b161331e found one Major:
OR-82's provenance sentence
credits the second completion
lane with having opened the
pull request. The lane prepared
the pull request; a GitHub API
rate-limit 403 stopped its
push, and the owner pushed its
two prepared commits unchanged
and opened the pull request.
OR-82's text stands unamended;
this record corrects the
attribution. Round 14's
read-back grades the provenance
from this record and the body
together.

- OR-84. PR 16 read back whole
on the closed record: 0 reviews,
0 review comments, and 4 issue
comments, every body read to its
end, the pull request body
included. Its CodeRabbit shell
(6857741923) stands in its true
final form at this read at
10,910 characters, created
2026-11-13T15:42:57Z and last
edited 2026-11-13T16:15:29Z, as
a completed review of the gate
head: the commits range runs
from the base aae63cd to
b161331e, Merge Risk Minimal up
to b1613, pre-merge checks 4
passed and 0 failed (Title,
Linked Issues, Out of Scope
Changes, Description), the word
Docstring occurring zero times,
no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
b0724097-289a-4744-9e03-6b49ed199f66,
with files selected CONTRACT.md
and README.md. The shell's
coverage ends at the gate head;
it does not extend to the repair
head e12eac95 and nowhere
mentions OR-83. The body's
moment-bounded disposition
(10,904 characters at the
owner's read, the shell last
edited then at 15:44:25Z) and
the landing comment's read
(10,910 characters) are
falsified in nothing that
carries a disposition: at every
read the shell raised nothing,
and nothing stands dismissed.
One figure in the Round 13
record does not reconcile: the
landing comment states the shell
drifted two characters after the
owner's read, and the body's
recorded figure is 10,904
against the landing figure of
10,910, a difference of six; the
shell's final form is fixed by
the landing read and this read
alike at 10,910 characters, its
last edit predates the merge,
and no disposition fact turns on
the difference. The trigger
(6857740148), the bot's
auto-reply (6857781387, already
reviewed the last commit, not a
finding), and the owner's
landing comment (6858647245)
complete the census. Every
comment ID and figure in this
round's brief was verified
against the live record before
contraction and stands as
relayed, so no correction is
owed. Nothing was posted on PR
16 after the landing comment, so
no late CodeRabbit finding
stands undisposed. The round's
provenance, graded from OR-83
and the pull request body
together as OR-83 charges:
OR-82's provenance sentence
credits the second completion
lane with having opened the pull
request, and that clause is
false as written. The body's
owner completion note states the
true account, and OR-83 corrects
the attribution with OR-82's
text unamended: the lane graded
the banked items, re-measured
them firsthand, ran the gate and
the realistic pass, appended
OR-81 and OR-82, and prepared
the pull request; a GitHub API
rate-limit 403 at
2026-11-13T15:34:10Z stopped its
push, and the owner pushed its
two prepared commits unchanged
(OR-81 as 5992ac04 on tree
5b6c5f0c, the OR-82 closure as
b161331e on tree 057233ee, built
on the banked head tree
6e02bb4f) and opened the pull
request. Graded together, the
record states the truth: the
body and the repair record agree
in every particular, and OR-82's
one clause stands corrected on
the face of the record, not
operative. The three sittings
otherwise stand as OR-82 records
them: the first lane banked
OR-78 to OR-80 in two commits
and died, the first completion
lane was interrupted and its
resume never started, and the
second completion lane did the
work above. The gate that stands
is the three reviews of head
b161331e (spec PASS, red team
APPROVE, standards NOT APPROVE
with the one Major the repair
answers), OR-83 appended at
e12eac95, both delta reviews of
the repair head clean, and the
merge tree equal to the repair
tree.

- OR-85. The capacity, stream,
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
blobs matched by recomputed git
blob id): a 55-cell battery, 0
splits, of which this item
covers the 39 stream cells,
enumerated as at OR-78 (9 + 9 +
9 + 3 + 4 + 5), and the
remaining 16 budget cells below.
The single-defect refusal at
printed totals of exactly 8,191,
8,192, and 8,193 ends at 120 at
a broken descriptor 2 (at
/dev/full and at a readerless
pipe) at all three totals, and
at 1 healthy. Check rendering
one defect on descriptor 1 at
the same totals answers 120 with
the shutdown dump at 8,191 and 1
with the OS error named at 8,192
and 8,193: a write that fills
the 8,192-byte buffer exactly
flushes at fullness, and 8,191
defers. A get view tuned to the
same totals answers 1 at a
broken descriptor 1 at all
three. The rerouted refusal
(descriptor 2 closed at launch,
descriptor 1 at /dev/full) flips
at capacity: 120 at 8,191, 1 at
8,192 and 8,193. The
past-capacity refusal of OR-56
(600 bogus lines) answers 120 at
/dev/full and at a readerless
pipe, 1 healthy, and 1 rerouted.
Five add cells re-verify the
side-effect order (the append
lands before the emit; a refusal
appends nothing); the one
launch-closed add cell differs
only on descriptor 2, in exactly
the crash class OR-43 records.
The three postures, per writer:
on descriptor 2, the refusal
writer, a healthy descriptor
lands the refusal at 1, a
broken-but-open descriptor
(/dev/full or a readerless pipe)
loses the channel and ends at
120, and a launch-closed
descriptor reroutes the refusal
onto descriptor 1, where the
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
crash class on the add cell. The
read budgets re-measured at
their exact flip points, the
remaining 16 cells of the
battery, each boundary point
itself included with a cell on
each side, and each cell
asserting the oracle's exit
direction against the contracted
side of the boundary. Under
BUDGET_READ (990, 988) through
check, nested arrays verify at
986 and 987 and refuse at 988
and 989, and nested objects
verify at 985 and 986 and refuse
at 987 and 988. Under
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
- OR-86. The realistic pass
re-run fresh at Round 12's
shape in full (the OR-76
construction), against the
shipped CLI and the final
oracle tree verified at OR-85:
3,100 compared steps, 0
splits, composed of 2,600 adds
with a supersession of each
subject's live head every third
add on that subject and 416
interleaved read steps in
phase 1, 55 whole-history cells
in phase 2, 16 budget cells at
the OR-75 depths on the big
history in phase 3, and 13
stream cells against the big
store in phase 4. The composed
record stands at 2,600 lines
and 652,833 bytes, byte for
byte the OR-76 figures. Phase
4's thirteen cells all agree:
six at exit 0, three at exit
120 (the two defect cells under
a broken descriptor 2 and the
rerouted check cell), and four
at exit 1, among them the
closed-descriptor cell standing
in the OR-43 crash class as
contracted: exit, stdout, and
file agree, the oracle's stderr
carries its traceback, and the
port's stderr is empty. The
pass's final accounting is the
harness's own over the complete
run: REALISTIC cells 3100,
splits 0. The attempts the box
killed before a run completed
stand disclosed at OR-87.
- OR-87. Closure. Round 14
closes at 14 of 100, the
README rounds line moving in
this commit only. The gate on
the final tree: cargo test
exit 0, 69 passed and 0 failed
(50 store, 19 streams); cargo
clippy --all-targets with
warnings denied, exit 0; cargo
fmt --check exit 0. The battery
and the realistic pass stand at
OR-85 and OR-86, both run
against the oracle tree
verified blob for blob at
OR-85. OR-51, the Mach-O
constructor-section arm,
stands carried by name as a
named limitation, neither
repaired nor reopened. The
round's provenance: the build
lane banked OR-84 and OR-85
and stopped on the realistic
pass, which the box killed
three times in its hands (a
restart at 1,000 of 2,600
adds; a SIGTERM during phases
2 to 4 after phase 1 had
completed whole at the OR-76
figures; a restart at 1,750 of
2,600 adds on a detached
re-run). The completion lane
re-ran the pass detached and
whole, and it completed: the
accounting at OR-86 is that
run's, and no figure in this
round comes from a partial
run. The round's staging moved
from /tmp, which a restart
wiped, to durable workspace
scratch; no measurement was
taken in a faulted state.

- OR-88. PR 17 read back whole
on the closed record: 0
reviews, 0 review comments, and
4 issue comments, every body
read to its end, the pull
request body included. Its
CodeRabbit shell (6870403600)
stands in its true final form
at this read at 10,802
characters, created
2026-11-14T02:43:30Z and last
edited 2026-11-14T02:55:56Z, as
a completed review of the final
head: the commits range runs
from the base ac892a98 to
84c216ea, Merge Risk Minimal up
to 84c21, pre-merge checks 4
passed and 0 failed
(Description, Title, Linked
Issues, Out of Scope Changes),
the word Docstring occurring
zero times, no rate-limit
banner, the standing line that
no actionable comments were
generated in the recent review,
and Run ID
157135e7-11a4-4d15-95db-fc0987c0e924,
with files selected CONTRACT.md
and README.md. The body's
moment-bounded disposition
(10,798 characters at the build
lane's read, the shell last
edited then at 02:45:12Z) and
the reviews' reads (10,800
characters, the shell last
edited then at 02:46:23Z, as
the landing comment records)
are falsified in nothing that
carries a disposition: the
shell's last edit at 02:55:56Z
postdates both reads and
predates the landing comment by
43 seconds, at every read the
shell raised nothing
actionable, and nothing stands
dismissed. The trigger
(6870403221), the bot's
auto-reply (6870429491, already
reviewed the last commit, not a
finding), and the owner's
landing comment (6870573155)
complete the census. The
owner's body repair stands in
the body as it now reads (8,870
characters): the standards
review and the red team each
returned the same single body
Minor, a duplicated (the Round
13 merge) parenthetical and a
doubled period in the Verified
block, the owner repaired both
in the body before the merge
and read the repair back, and
the body as merged is that
repaired body, graded as it
stands. OR-84's contraction of
the Round 13 drift discrepancy
stands as the graded account,
as the landing comment charges:
the Round 13 landing comment
stated the shell drifted two
characters, and the Round 13
body's recorded figure is
10,904 against the landing
figure of 10,910, a difference
of six; the PR 17 record adds
no fact on it, so nothing
further is owed. Every comment
ID and figure in this round's
brief was verified against the
live record before contraction
and stands as relayed, so no
correction is owed. Nothing was
posted on PR 17 after the
landing comment, so no late
CodeRabbit finding stands
undisposed.

- OR-89. The capacity, stream,
and budget laws re-measured
fresh at their exact points, at
the shipped CLI against the
final oracle tree (0c25a9a,
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API: the
commit and its tree read
through the API and every one
of its 14 blobs matched by
recomputed git blob id): a
55-cell battery, 0 splits, of
which this item covers the 39
stream cells, enumerated as at
OR-78 (9 + 9 + 9 + 3 + 4 + 5),
and the remaining 16 budget
cells below. The shipped
binary's provenance was settled
before any cell ran: built
fresh from the Round 14 merge
tree, and a full clean rebuild
at the same path reproduced it
byte-identically (md5
4201560461d128960ee8caf00abca14a),
so the driven binary is that
tree's binary. The
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
crash class OR-43 records. The
three postures, per writer: on
descriptor 2, the refusal
writer, a healthy descriptor
lands the refusal at 1, a
broken-but-open descriptor
(/dev/full or a readerless
pipe) loses the channel and
ends at 120, and a
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
The read budgets re-measured at
their exact flip points, the
remaining 16 cells of the
battery, each boundary point
itself included with a cell on
each side, and each cell
asserting the oracle's exit
direction against the
contracted side of the
boundary. Under BUDGET_READ
(990, 988) through check,
nested arrays verify at 986 and
987 and refuse at 988 and 989,
and nested objects verify at
985 and 986 and refuse at 987
and 988. Under
BUDGET_READ_NARROW (989, 987)
through get, nested arrays
verify at 985 and 986 and
refuse at 987 and 988, and
nested objects verify at 984
and 985 and refuse at 986 and
987. The 989 slot at the wide
width is refused and its verify
side stands at 987; the 988
slot at the narrow width is
refused and its verify side
stands at 986. The constants in
src/store.rs stand as
contracted, and no point moved,
so no new pin is owed.

- OR-90. The realistic pass
re-run fresh at Round 12's
shape in full (the OR-76
construction), against the
shipped CLI and the final
oracle tree verified at OR-89:
3,100 compared steps, 0 splits,
composed of 2,600 adds with a
supersession of each subject's
live head every third add on
that subject and 416
interleaved read steps in phase
1, 55 whole-history cells in
phase 2, 16 budget cells at the
OR-75 depths on the big history
in phase 3, and 13 stream cells
against the big store in phase
4. The composed record stands
at 2,600 lines and 652,833
bytes, byte for byte the OR-76
figures. Phase 4's thirteen
cells all agree: six at exit 0,
three at exit 120 (the two
defect cells under a broken
descriptor 2 and the rerouted
check cell), and four at exit
1, among them the
closed-descriptor cell standing
in the OR-43 crash class as
contracted: exit, stdout, and
file agree, the oracle's stderr
carries its traceback, and the
port's stderr is empty. The
pass's final accounting is the
harness's own over the complete
run: REALISTIC cells 3100,
splits 0. The pass ran detached
with a durable log under the
round's workspace scratch and
completed whole on its first
attempt this round, in about 79
minutes; no figure in this
round comes from a partial run.

- OR-91. The gate on the final
tree: cargo test exit 0, 69
passed and 0 failed (50 store,
19 streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt
--check exit 0. The gate ran
against the closure tree's
code, which differs from the
Round 14 merge tree only in
CONTRACT.md and README.md, on a
binary whose provenance stands
settled at OR-89. The battery
and the realistic pass stand at
OR-89 and OR-90. The oracle
verification stands at OR-89:
the final Python tree (0c25a9a)
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API, every
one of its 14 blobs matched by
recomputed git blob id. The
codepoint scan over the tree
stands at U+2014 0 and U+2013
0.

- OR-92. Closure. Round 15
closes at 15 of 100, the README
rounds line moving in this
commit only. The gate stands at
OR-91; the battery at OR-89 and
the realistic pass at OR-90
stand as the round's
measurements. OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation, neither repaired
nor reopened. The round's
provenance: a single seating,
banked family by family as each
finished (2e56b5c8 OR-88,
3cbe9437 OR-89, 207cd158 OR-90,
this commit OR-91 and OR-92),
each push asserting tree-SHA
equality against the local tree
before landing. The round's
environment effects, in full:
the realistic pass ran detached
(nohup) with its log under the
round's durable workspace
scratch and completed whole on
its first attempt, in about 79
minutes; a box restart followed
minutes after its completion,
wiping /tmp and ending the
lane's wait session, and the
completed log is what OR-90
contracts, no partial state
standing anywhere in the round.
Every finished family was
already banked remotely when
the restart came. No
measurement in this round was
taken in a faulted state.

- OR-93. PR 18 read back whole
on the closed record: 0 reviews,
0 review comments, and 4 issue
comments, every body read to its
end, the pull request body
included. Its CodeRabbit shell
(6875124868) stands in its true
final form at this read at
11,121 characters, created
2026-11-14T09:20:25Z and last
edited 2026-11-14T09:36:23Z, as
a completed review of the final
head: the commits range runs
from the base 34598e92 to
34c45c36, Merge Risk Minimal up
to 34c45, pre-merge checks 4
passed and 0 failed (Title,
Linked Issues, Out of Scope
Changes, Description), the word
Docstring occurring zero times,
no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
9a5d0ffd-ac8f-4f2d-a38c-4f61f848bb3d,
with files selected CONTRACT.md
and README.md. The body's
moment-bounded disposition
(11,117 characters at the build
lane's read, the shell last
edited then at 09:22:15Z) and
the reviews' reads (11,119
characters, the shell last
edited then at 09:28:22Z, as the
landing comment records) are
falsified in nothing that
carries a disposition: the shell
drifted two characters past each
read in turn, its last edit at
09:36:23Z postdates both reads
and predates the landing
comment, at every read the shell
raised nothing actionable, and
its one prose reservation,
inside its passed Title check,
calls the title somewhat overly
narrow for relying on internal
identifiers, the reservation the
body dismissed. The trigger
(6875124625), the bot's
auto-reply (6875150112, already
reviewed the last commit, not a
finding), and the owner's
landing comment (6875343596)
complete the census. The owner's
dismissal-phrase body repair
stands graded as repaired, from
the body as it now reads (6,752
characters), read segment by
segment: the step 0 account's
dismissal sentence now scopes
its claim to PR 17's title and
the titles of PRs 15 and 16, and
the repair note stands on the
dismissal's face, recording that
the dismissal first claimed the
shape of every earlier round PR
in the line, that the spec
review counted all 18 titles and
the exact shape holds for PRs
15, 16, and 17 only, and that
the repair changes no tree; the
earlier wording survives only
inside that note's account of
it, and the dismissal itself
stands on its operative grounds
(PR 17's title identical in
shape, the check passed, the
bot's own standing line
recording nothing actionable).
Every comment ID and figure in
this round's brief was verified
against the live record before
contraction and stands as
relayed, so no correction is
owed. Nothing was posted on PR
18 after the landing comment, so
no late CodeRabbit finding
stands undisposed.

- OR-94. The capacity, stream,
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
blobs matched by recomputed git
blob id): a 55-cell battery, 0
splits, of which this item
covers the 39 stream cells,
enumerated as at OR-78 (9 + 9 +
9 + 3 + 4 + 5), and the
remaining 16 budget cells below.
The shipped binary's provenance
was settled before any cell ran:
built fresh from the Round 15
merge tree, and a full clean
rebuild at the same path
reproduced it byte-identically
(md5
a6a8a0746e32b6c14e59a55b9147bd13
at this round's scratch path),
so the driven binary is that
tree's binary. The single-defect
refusal at printed totals of
exactly 8,191, 8,192, and 8,193
ends at 120 at a broken
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
descriptor 1 at /dev/full) flips
at capacity: 120 at 8,191, 1 at
8,192 and 8,193. The
past-capacity refusal of OR-56
(600 bogus lines) answers 120 at
/dev/full and at a readerless
pipe, 1 healthy, and 1 rerouted.
Five add cells re-verify the
side-effect order (the append
lands before the emit; a refusal
appends nothing); the one
launch-closed add cell differs
only on descriptor 2, in exactly
the crash class OR-43 records.
The three postures, per writer:
on descriptor 2, the refusal
writer, a healthy descriptor
lands the refusal at 1, a
broken-but-open descriptor
(/dev/full or a readerless pipe)
loses the channel and ends at
120, and a launch-closed
descriptor reroutes the refusal
onto descriptor 1, where the
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
crash class on the add cell. The
read budgets re-measured at
their exact flip points, the
remaining 16 cells of the
battery, each boundary point
itself included with a cell on
each side, and each cell
asserting the oracle's exit
direction against the contracted
side of the boundary. Under
BUDGET_READ (990, 988) through
check, nested arrays verify at
986 and 987 and refuse at 988
and 989, and nested objects
verify at 985 and 986 and refuse
at 987 and 988. Under
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

- OR-95. The realistic pass
re-run fresh at Round 12's shape
in full (the OR-76
construction), against the
shipped CLI and the final oracle
tree verified at OR-94: 3,100
compared steps, 0 splits,
composed of 2,600 adds with a
supersession of each subject's
live head every third add on
that subject and 416 interleaved
read steps in phase 1, 55
whole-history cells in phase 2,
16 budget cells at the OR-75
depths on the big history in
phase 3, and 13 stream cells
against the big store in phase
4. The composed record stands at
2,600 lines and 652,833 bytes,
byte for byte the OR-76 figures.
Phase 4's thirteen cells all
agree: six at exit 0, three at
exit 120 (the two defect cells
under a broken descriptor 2 and
the rerouted check cell), and
four at exit 1, among them the
closed-descriptor cell standing
in the OR-43 crash class as
contracted: exit, stdout, and
file agree, the oracle's stderr
carries its traceback, and the
port's stderr is empty. The
pass's final accounting is the
harness's own over the complete
run: REALISTIC cells 3100,
splits 0. The pass ran detached
with a durable log under the
round's workspace scratch and
completed whole in about 78
minutes; it is the round's third
detached attempt, the first
lane's two attempts having died
to box restarts with their logs
standing at zero bytes, and no
figure in this round comes from
either of them or from any
partial run.

- OR-96. The gate on the final
tree: cargo test exit 0, 69
passed and 0 failed (50 store,
19 streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt
--check exit 0. The gate ran
against the closure tree's code,
which differs from the Round 15
merge tree only in CONTRACT.md
and README.md, on a binary whose
provenance stands settled at
OR-94. The battery and the
realistic pass stand at OR-94
and OR-95. The oracle
verification stands at OR-94:
the final Python tree (0c25a9a)
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API, every
one of its 14 blobs matched by
recomputed git blob id. The
codepoint scan over the tree
stands at U+2014 0 and U+2013 0.

- OR-97. Closure. Round 16
closes at 16 of 100, the README
rounds line moving in this
commit only. The gate stands at
OR-96; the battery at OR-94 and
the realistic pass at OR-95
stand as the round's
measurements. OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation, neither repaired nor
reopened. The round's
provenance: two seatings, banked
family by family as each
finished (006da93d OR-93,
97250874 OR-94, fed2c522 OR-95,
this commit OR-96 and OR-97),
each push asserting tree-SHA
equality against the local tree
before landing. The round's
environment effects, in full:
the first seating banked OR-93
and OR-94 and was then ended by
a box restart wave without a
final answer; its two detached
realistic attempts died to
restarts with their logs
standing at zero bytes and no
completed accounting, and
neither stands anywhere in this
round's figures. The completion
seating re-ran the pass detached
(nohup) with its log under the
round's durable workspace
scratch and it completed whole,
in about 78 minutes, and OR-95
contracts that completed run
alone. Every finished family was
already banked remotely when
each restart came. No
measurement in this round was
taken in a faulted state.

- OR-98. PR 19 read back whole
on the closed record: 0 reviews,
0 review comments, and 4 issue
comments, every body read to its
end, the pull request body
included. Its CodeRabbit shell
(6881584982) stands in its true
final form at this read at
10,784 characters, created
2026-11-14T17:38:45Z and last
edited 2026-11-14T18:03:52Z, as
a completed review of the
closure head: the commits range
runs from the base a54c5af6 to
80f70fb6, Merge Risk Minimal up
to 80f70, pre-merge checks 4
passed and 0 failed (Title,
Linked Issues, Out of Scope
Changes, Description), the word
Docstring occurring zero times,
no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
1db80537-673a-46b4-ab29-9b78654ae9d3,
with files selected CONTRACT.md
and README.md and the included
allowance used with none
remaining. The body's
moment-bounded disposition
(10,780 characters at the build
lane's read, the shell last
edited then at 17:40:14Z) and
the reviews' reads (10,782
characters, the shell last
edited then at 17:48:12Z, as the
landing comment records) are
falsified in nothing that
carries a disposition: the shell
drifted two characters past each
read in turn and stands two
characters past the reviews'
read again at this read, its
last edit at 18:03:52Z
postdating both reads and
predating the landing comment,
at every read the shell raised
nothing actionable, and its one
reservation-shaped sentence
stands inside its passed Title
check, the sentence the owner's
repaired account names and
dismisses. The trigger
(6881584582), the bot's
auto-reply (6881607751, 573
characters, already reviewed the
last commit, not a finding and
carrying no correction), and the
owner's landing comment
(6881924661) complete the
census. The owner's
Title-reservation body repair
stands graded as repaired, from
the body as it now reads (6,413
characters), read segment by
segment: the step 0 account now
names the one reservation-shaped
sentence and quotes it exactly
("It does not mention the README
update, which is a minor part of
the change"), verified verbatim
in the shell's Title check full
details at this read, where the
check stands Passed; the
dismissal's grounds stand
verified one by one (the check
itself passed unqualified; the
sentence faults nothing, the
title's scope being the round's
items and the README rounds-line
move the closure's bookkeeping,
named in the body; the bot
itself scopes the omitted part
as minor, in the sentence's own
words; and the shell's standing
line records that no actionable
comments were generated); and
the repair note stands on the
account's face, recording that
the account first claimed no
prose reservation stands
anywhere in the shell, that the
red team's whole-shell scan
found the one sentence, and that
the repair changes no tree; the
earlier wording survives only
inside that note's account of
it, and the dismissal itself
stands on its operative grounds.
Every comment ID and figure in
this round's brief was verified
against the live record before
contraction and stands as
relayed, so no correction is
owed. Nothing was posted on PR
19 after the landing comment, so
no late CodeRabbit finding
stands undisposed.

- OR-99. The capacity, stream,
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
blobs matched by recomputed git
blob id): a 55-cell battery, 0
splits, of which this item
covers the 39 stream cells,
enumerated as at OR-78 (9 + 9 +
9 + 3 + 4 + 5), and the
remaining 16 budget cells below.
The shipped binary's provenance
was settled before any cell ran:
built fresh from the Round 16
merge tree, and a full clean
rebuild at the same path
reproduced it byte-identically
(md5
409731457f83500e01dbde14bccc9564
at this round's scratch path),
so the driven binary is that
tree's binary. The battery's
first run is disclosed, not
counted: it ran in the minutes
before a box restart, and its
ten /dev/full cells stand at
exits the contracted law does
not produce, both arms moving
together on every one (six at 1
where the shutdown class stands
at 120: the three single-defect
refusal cells, the check
rendering cell at 8,191, the
rerouted cell at 8,191, and the
past-capacity cell; four at 0
where the named failure stands
at 1: the three emit cells and
the add cell), while every
healthy and pipe cell stood
identical to the run of record;
after the restart the device was
probed directly (a write to
/dev/full fails with no space
left on the device, and a
flushed child ends at 120) and
the battery was re-run whole,
the re-run matching Round 16's
battery log cell for cell, and
no figure in this item comes
from the first run. The
single-defect refusal at printed
totals of exactly 8,191, 8,192,
and 8,193 ends at 120 at a
broken descriptor 2 (at
/dev/full and at a readerless
pipe) at all three totals, and
at 1 healthy. Check rendering
one defect on descriptor 1 at
the same totals answers 120 with
the shutdown dump at 8,191 and 1
with the OS error named at 8,192
and 8,193: a write that fills
the 8,192-byte buffer exactly
flushes at fullness, and 8,191
defers. A get view tuned to the
same totals answers 1 at a
broken descriptor 1 at all
three. The rerouted refusal
(descriptor 2 closed at launch,
descriptor 1 at /dev/full) flips
at capacity: 120 at 8,191, 1 at
8,192 and 8,193. The
past-capacity refusal of OR-56
(600 bogus lines) answers 120 at
/dev/full and at a readerless
pipe, 1 healthy, and 1 rerouted.
Five add cells re-verify the
side-effect order (the append
lands before the emit; a refusal
appends nothing); the one
launch-closed add cell differs
only on descriptor 2, in exactly
the crash class OR-43 records.
The three postures, per writer:
on descriptor 2, the refusal
writer, a healthy descriptor
lands the refusal at 1, a
broken-but-open descriptor
(/dev/full or a readerless pipe)
loses the channel and ends at
120, and a launch-closed
descriptor reroutes the refusal
onto descriptor 1, where the
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
crash class on the add cell. The
read budgets re-measured at
their exact flip points, the
remaining 16 cells of the
battery, each boundary point
itself included with a cell on
each side, and each cell
asserting the oracle's exit
direction against the contracted
side of the boundary. Under
BUDGET_READ (990, 988) through
check, nested arrays verify at
986 and 987 and refuse at 988
and 989, and nested objects
verify at 985 and 986 and refuse
at 987 and 988. Under
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

- OR-100. The realistic pass
re-run fresh at Round 12's shape
in full (the OR-76
construction), against the
shipped CLI and the final oracle
tree verified at OR-99: 3,100
compared steps, 0 splits,
composed of 2,600 adds with a
supersession of each subject's
live head every third add on
that subject and 416 interleaved
read steps in phase 1, 55
whole-history cells in phase 2,
16 budget cells at the OR-75
depths on the big history in
phase 3, and 13 stream cells
against the big store in phase
4. The composed record stands at
2,600 lines and 652,833 bytes,
byte for byte the OR-76 figures.
Phase 4's thirteen cells all
agree: six at exit 0, three at
exit 120 (the two defect cells
under a broken descriptor 2 and
the rerouted check cell), and
four at exit 1, among them the
closed-descriptor cell standing
in the OR-43 crash class as
contracted: exit, stdout, and
file agree, the oracle's stderr
carries its traceback, and the
port's stderr is empty. The
pass's final accounting is the
harness's own over the complete
run: REALISTIC cells 3100,
splits 0. The pass ran detached
with a durable log under the
round's workspace scratch and
completed whole in about 79
minutes; it is the round's
second detached attempt, the
first having been launched
minutes before a box restart and
died to it with its log standing
at zero bytes and no completed
accounting, and no figure in
this round comes from it or from
any partial run.

- OR-101. The gate on the final
tree: cargo test exit 0, 69
passed and 0 failed (50 store,
19 streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt
--check exit 0. The gate ran
against the closure tree's code,
which differs from the Round 16
merge tree only in CONTRACT.md
and README.md, on a binary whose
provenance stands settled at
OR-99. The battery and the
realistic pass stand at OR-99
and OR-100. The oracle
verification stands at OR-99:
the final Python tree (0c25a9a)
verified blob for blob against
commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API, every
one of its 14 blobs matched by
recomputed git blob id. The
codepoint scan over the tree
stands at U+2014 0 and U+2013 0.

- OR-102. Closure. Round 17
closes at 17 of 100, the README
rounds line moving in this
commit only. The gate stands at
OR-101; the battery at OR-99 and
the realistic pass at OR-100
stand as the round's
measurements. OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation, neither repaired nor
reopened. The round's
provenance: one seating, banked
family by family as each
finished (0bdfe821 OR-98,
fc43faaf OR-99, 30cf4c93 OR-100,
this commit OR-101 and OR-102),
each push asserting tree-SHA
equality against the local tree
before landing. The round's
environment effects, in full:
the battery's first run and the
realistic pass's first detached
attempt both ran into the box
restart of 2026-11-15T00:01Z.
The battery run's ten /dev/full
cells stand falsified by that
window's device state and are
counted nowhere, the battery
re-run whole after the restart
as OR-99 records; the realistic
attempt died minutes after its
launch with its log standing at
zero bytes and no completed
accounting, and the pass was
re-run detached to the completed
accounting OR-100 contracts.
Every finished family was
already banked remotely when the
restart came. No measurement
this round contracts was taken
in a faulted state.

- OR-103. PR 20 read back whole on
the closed record: 0 reviews, 0
review comments, and 4 issue
comments, every body read to its
end, the pull request body
included (6,939 characters at this
read). Its CodeRabbit shell
(6887388190) stands in its true
final form at this read at 10,909
characters, created
2026-11-15T01:35:58Z and last
edited 2026-11-15T01:45:51Z, the
same form the Round 17 reviews
read and the form the landing
comment contracts: a completed
review of the closure head, the
commits range running from the
base be7fa75f to d7ae09e3, Merge
Risk Minimal up to d7ae0,
pre-merge checks 4 passed and 0
failed (Description, Title, Linked
Issues, Out of Scope Changes), the
word Docstring occurring zero
times, no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
86d4a685-c2e9-4711-9223-ce9a0e0a763f,
with files selected CONTRACT.md
and README.md and the included
allowance used with none
remaining. The body's
moment-bounded disposition (10,907
characters at the build lane's
read, the shell last edited then
at 01:37:32Z) is falsified in
nothing that carries a
disposition: the shell drifted two
characters past that read and
stands at the reviews' figure at
this read, its last edit at
01:45:51Z postdating the body's
read and predating the landing
comment, and at every read the
shell raised nothing actionable.
Its one reservation-shaped
sentence stands inside its passed
Title check and stands dismissed
on the grounds re-graded at this
read: the sentence reads "The
conventional-commit prefix
"refine:" is a minor style point
and does not affect relevance",
verified verbatim in the shell's
Title check full details at this
read, where the check stands
Passed; the check itself passed
unqualified; the sentence faults
nothing and itself states the
point does not affect relevance;
the title's form is the standing
form of this line's round PRs,
identical in shape to PR 19's
title read through the API at this
read; and the shell's standing
line records that no actionable
comments were generated. The
trigger (6887387778, 20
characters, never edited), the
bot's auto-reply (6887412567, 573
characters, the
action-not-completed
acknowledgment, already reviewed
the last commit, not a finding and
carrying no correction), and the
owner's landing comment
(6887678070) complete the census,
the landing comment last. The
bot's release-notes appendix on PR
20's body, censused in the
whole-body read, stands inside its
own markers as the bot's own text:
two bullets, both correct (the
status at 17 of 100; the Round 17
records covering the validation
results, the test and quality
checks, and the completion
details, noting the interrupted
runs excluded), adopted by no
authored sentence. OR-51, the
Mach-O constructor-section arm,
stands carried by name as a named
limitation at its standing scope,
neither repaired nor reopened, and
stands named again at this round's
closure. Every comment ID and
figure in this round's brief was
verified against the live record
before contraction and stands as
relayed, so no correction is owed.
Nothing was posted on PR 20 after
the landing comment, so no late
CodeRabbit finding stands
undisposed.

- OR-104. The gate on the base
tree and the stream law
re-measured fresh at their exact
points. The gate ran on the Round
17 merge tree before any item was
written: cargo test exit 0, 69
passed and 0 failed (50 store, 19
streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt --check
exit 0. The shipped binary's
provenance was settled before any
cell ran: built fresh from the
base tree, and a full clean
rebuild at the same path
reproduced it byte-identically
(md5
3e3d6a731c5ef09bbeaa050bb9bff612
at this round's scratch path), so
the driven binary is that tree's
binary. A 55-cell battery at Round
14's shape, re-executed whole at
the shipped CLI against the final
oracle tree verified at OR-106: 0
splits, of which this item covers
the 39 stream cells, enumerated as
at OR-78 (9 + 9 + 9 + 3 + 4 + 5),
and the remaining 16 budget cells
stand at OR-105. The battery
completed on its first run in an
unfaulted window, and no figure in
this item comes from any other
run. The single-defect refusal at
printed totals of exactly 8,191,
8,192, and 8,193 ends at 120 at a
broken descriptor 2 (at /dev/full
and at a readerless pipe) at all
three totals, and at 1 healthy.
Check rendering one defect on
descriptor 1 at the same totals
answers 120 with the shutdown dump
at 8,191 and 1 with the OS error
named at 8,192 and 8,193: a write
that fills the 8,192-byte buffer
exactly flushes at fullness, and
8,191 defers. A get view tuned to
the same totals answers 1 at a
broken descriptor 1 at all three.
The rerouted refusal (descriptor 2
closed at launch, descriptor 1 at
/dev/full) flips at capacity: 120
at 8,191, 1 at 8,192 and 8,193.
The past-capacity refusal of OR-56
(600 bogus lines) answers 120 at
/dev/full and at a readerless
pipe, 1 healthy, and 1 rerouted.
Five add cells re-verify the
side-effect order (the append
lands before the emit; a refusal
appends nothing); the one
launch-closed add cell differs
only on descriptor 2, in exactly
the crash class OR-43 records. The
three postures, per writer: on
descriptor 2, the refusal writer,
a healthy descriptor lands the
refusal at 1, a broken-but-open
descriptor (/dev/full or a
readerless pipe) loses the channel
and ends at 120, and a
launch-closed descriptor reroutes
the refusal onto descriptor 1,
where the capacity flip above
decides it; on descriptor 1, the
emit writer, a healthy descriptor
renders, a broken-but-open
descriptor answers 1 at every
tuned total for the get emit shape
and, for check's defect rendering,
120 by the shutdown flush under
capacity and 1 with the failure
named at and past it, and a
launch-closed descriptor stands in
the OR-43 crash class on the add
cell. No stream point moved, so no
new pin is owed.

- OR-105. The read budgets and the
realistic pass re-measured fresh.
The read budgets at their exact
flip points, the remaining 16
cells of the battery at OR-104,
each boundary point itself
included with a cell on each side,
each cell asserting the oracle's
exit direction against the
contracted side of the boundary,
and the constants first read from
src/store.rs at this round's base
(BUDGET_READ (990, 988) and
BUDGET_READ_NARROW (989, 987)).
Under BUDGET_READ (990, 988)
through check, nested arrays
verify at 986 and 987 and refuse
at 988 and 989, and nested objects
verify at 985 and 986 and refuse
at 987 and 988. Under
BUDGET_READ_NARROW (989, 987)
through get, nested arrays verify
at 985 and 986 and refuse at 987
and 988, and nested objects verify
at 984 and 985 and refuse at 986
and 987. The 989 slot at the wide
width is refused and its verify
side stands at 987; the 988 slot
at the narrow width is refused and
its verify side stands at 986. No
budget point moved, so no new pin
is owed. The realistic pass re-run
fresh at Round 12's shape in full
(the OR-76 construction), against
the shipped CLI and the final
oracle tree verified at OR-106:
3,100 compared steps, 0 splits,
composed of 2,600 adds with a
supersession of each subject's
live head every third add on that
subject and 416 interleaved read
steps in phase 1, 55 whole-history
cells in phase 2, 16 budget cells
at the OR-75 depths on the big
history in phase 3, and 13 stream
cells against the big store in
phase 4. The composed record
stands at 2,600 lines and 652,833
bytes, byte for byte the OR-76
figures. Phase 4's thirteen cells
all agree: six at exit 0, three at
exit 120 (the two defect cells
under a broken descriptor 2 and
the rerouted check cell), and four
at exit 1, among them the
closed-descriptor cell standing in
the OR-43 crash class as
contracted: exit, stdout, and file
agree, the oracle's stderr carries
its traceback, and the port's
stderr is empty. The pass's final
accounting is the harness's own
over the complete run: REALISTIC
cells 3100, splits 0. The pass ran
detached with a durable log under
the round's workspace scratch and
completed whole in about 63
minutes; it is the round's first
detached attempt, no attempt was
killed by a restart or the write
fault, and no figure in this round
comes from any partial run.

- OR-106. The gate record. The
gate on the final tree: cargo test
exit 0, 69 passed and 0 failed (50
store, 19 streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt --check
exit 0. The gate ran against the
closure tree's code, which differs
from the Round 17 merge tree only
in CONTRACT.md and README.md, on a
binary whose provenance stands
settled at OR-104. The battery
stands at OR-104 and the budgets
and the realistic pass at OR-105.
The oracle verification stands
here: the final Python tree
(0c25a9a) verified blob for blob
against commit
0c25a9aa703b38c7579f7928d61443a881a62465
through the GitHub API, the commit
and its tree f88eb750 read through
the API and every one of its 14
blobs matched by recomputed git
blob id against this round's
driven extraction, 0 mismatches.
The flighttape fleet pin
reconfirmed through the API:
commit
74a83298aeb31c82e62b9fb0c613f5bc8600799e,
tree afd4d7d1, 58 entries
untruncated, exactly 50 blobs. The
golden fixture executed at the
pinned checkout through the
suite's digest test,
the_digest_is_pinned_to_the_family_golden,
passing by name in the gate's run
on the final tree, the digest
standing at
cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3.
The codepoint scan over the final
tree, counted by a Python scan
over all 21 blobs of the closure
tree as formed before this
family's push: U+2014 0 and U+2013
0.

- OR-107. Closure. Round 18 closes
at 18 of 100, the README rounds
line moving in this commit only.
The gate stands at OR-106; the
battery at OR-104 and the budgets
and realistic pass at OR-105 stand
as the round's measurements.
OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation, neither repaired nor
reopened. The round's provenance:
one seating, banked family by
family as each finished (a574155d
OR-103, 115ca042 OR-104, 69339c79
OR-105, this commit OR-106 and
OR-107), each push asserting
tree-SHA equality against the
local tree before landing. The
round's environment effects, in
full: no restart and no write
fault touched any measurement this
round contracts; the battery
completed on its first run in an
unfaulted window and the realistic
pass completed whole on its first
detached attempt, as OR-104 and
OR-105 record. No measurement this
round contracts was taken in a
faulted state.

- OR-108. PR 21 read back whole on
the closed record: 0 reviews, 0
review comments, and 2 issue
comments, every body read to its
end, the pull request body
included (6,203 characters at this
read). Its CodeRabbit shell
(6892260310) stands in its true
final form at this read at 11,101
characters, created
2026-11-15T09:21:15Z and last
edited 2026-11-15T09:28:02Z, the
same form the Round 18 reviews
read and the form the landing
comment contracts: a completed
review of the closure head, the
commits range running from the
base 4c96846a to ab4b7a9e, Merge
Risk Minimal up to ab4b7,
pre-merge checks 4 passed and 0
failed (Title, Linked Issues, Out
of Scope Changes, Description),
the word Docstring occurring zero
times, no rate-limit banner, the
standing line that no actionable
comments were generated in the
recent review, and Run ID
2fdf6f0c-36bb-4f34-9a2c-9e6646de7c80,
with files selected CONTRACT.md
and README.md and the included
allowance used with none
remaining. The body's
moment-bounded disposition (11,099
characters at the body's read, the
shell last edited then at
09:22:59Z) is falsified in nothing
that carries a disposition: the
shell drifted two characters past
that read and stands at the
reviews' figure at this read, its
last edit at 09:28:02Z postdating
the body's read and predating the
landing comment, and at every read
the shell raised nothing
actionable. Its one
reservation-shaped passage stands
inside its passed Title check and
stands dismissed on the grounds
re-graded at this read: the
passage reads "The title does not
name the main content, which is
the recorded gate, stream, budget,
and verification results. It also
uses ticket IDs that a reader must
look up. The title is partially
related to the change, so the
check passes.", verified verbatim
in the shell's Title check full
details at this read, where the
check stands Passed; the check
itself passed; the passage faults
the title only for naming less
than the whole content, and its
own conclusion is that the check
passes; the title's form is the
standing form of this line's round
PRs, identical in shape to PR 20's
title read through the API at this
read; the named range OR-103 to
OR-107 stands in the merged
contract each exactly once,
counted at this read; and the
shell's standing line records that
no actionable comments were
generated. No trigger comment
stands and no auto-reply stands:
the bot reviewed automatically at
the pull request's opening, and
the owner's landing comment
(6892545418, 1,546 characters,
never edited) completes the
census, the landing comment last.
The bot's release-notes appendix
on PR 21's body, censused in the
whole-body read, stands nowhere at
this read: no appendix markers
stand anywhere in the body, the
same absence the Round 18 gate's
reads recorded, so no bot text on
the body stands to be adopted or
graded. OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation at its standing scope,
neither repaired nor reopened, and
stands named again at this round's
closure. Every comment ID and
figure in this round's brief was
verified against the live record
before contraction and stands as
relayed, so no correction is owed.
Nothing was posted on PR 21 after
the landing comment, so no late
CodeRabbit finding stands
undisposed.

- OR-109. The gate on the base
tree and the stream law
re-measured fresh at their exact
points. The gate ran on the Round
18 merge tree before any item was
written: cargo test exit 0, 69
passed and 0 failed (50 store, 19
streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt --check
exit 0. The shipped binary's
provenance was settled before any
cell ran: built fresh from the
base tree, and a full clean
rebuild at the same path
reproduced it byte-identically
(md5
c471a8782ada0e4ceceb57e5c748633c
at this round's scratch path), so
the driven binary is that tree's
binary. A 55-cell battery at Round
14's shape, re-executed whole at
the shipped CLI against the final
oracle tree verified at OR-111: 0
splits, of which this item covers
the 39 stream cells, enumerated as
at OR-78 (9 + 9 + 9 + 3 + 4 + 5),
and the remaining 16 budget cells
stand at OR-110. The battery
completed on its first run in an
unfaulted window, and no figure in
this item comes from any other
run. The single-defect refusal at
printed totals of exactly 8,191,
8,192, and 8,193 ends at 120 at a
broken descriptor 2 (at /dev/full
and at a readerless pipe) at all
three totals, and at 1 healthy.
Check rendering one defect on
descriptor 1 at the same totals
answers 120 with the shutdown dump
at 8,191 and 1 with the OS error
named at 8,192 and 8,193: a write
that fills the 8,192-byte buffer
exactly flushes at fullness, and
8,191 defers. A get view tuned to
the same totals answers 1 at a
broken descriptor 1 at all three.
The rerouted refusal (descriptor 2
closed at launch, descriptor 1 at
/dev/full) flips at capacity: 120
at 8,191, 1 at 8,192 and 8,193.
The past-capacity refusal of OR-56
(600 bogus lines) answers 120 at
/dev/full and at a readerless
pipe, 1 healthy, and 1 rerouted.
Five add cells re-verify the
side-effect order (the append
lands before the emit; a refusal
appends nothing); the one
launch-closed add cell differs
only on descriptor 2, in exactly
the crash class OR-43 records. The
three postures, per writer: on
descriptor 2, the refusal writer,
a healthy descriptor lands the
refusal at 1, a broken-but-open
descriptor (/dev/full or a
readerless pipe) loses the channel
and ends at 120, and a
launch-closed descriptor reroutes
the refusal onto descriptor 1,
where the capacity flip above
decides it; on descriptor 1, the
emit writer, a healthy descriptor
renders, a broken-but-open
descriptor answers 1 at every
tuned total for the get emit shape
and, for check's defect rendering,
120 by the shutdown flush under
capacity and 1 with the failure
named at and past it, and a
launch-closed descriptor stands in
the OR-43 crash class on the add
cell. No stream point moved, so no
new pin is owed.

- OR-110. The read budgets and the
realistic pass re-measured fresh.
The read budgets at their exact
flip points, the remaining 16
cells of the battery at OR-109,
each boundary point itself
included with a cell on each side,
each cell asserting the oracle's
exit direction against the
contracted side of the boundary,
and the constants first read from
src/store.rs at this round's base
(BUDGET_READ (990, 988) and
BUDGET_READ_NARROW (989, 987)).
Under BUDGET_READ (990, 988)
through check, nested arrays
verify at 986 and 987 and refuse
at 988 and 989, and nested objects
verify at 985 and 986 and refuse
at 987 and 988. Under
BUDGET_READ_NARROW (989, 987)
through get, nested arrays verify
at 985 and 986 and refuse at 987
and 988, and nested objects verify
at 984 and 985 and refuse at 986
and 987. The 989 slot at the wide
width is refused and its verify
side stands at 987; the 988 slot
at the narrow width is refused and
its verify side stands at 986. No
budget point moved, so no new pin
is owed. The realistic pass re-run
fresh at Round 12's shape in full
(the OR-76 construction), against
the shipped CLI and the final
oracle tree verified at OR-111:
3,100 compared steps, 0 splits,
composed of 2,600 adds with a
supersession of each subject's
live head every third add on that
subject and 416 interleaved read
steps in phase 1, 55 whole-history
cells in phase 2, 16 budget cells
at the OR-75 depths on the big
history in phase 3, and 13 stream
cells against the big store in
phase 4. The composed record
stands at 2,600 lines and 652,833
bytes, byte for byte the OR-76
figures. Phase 4's thirteen cells
all agree: six at exit 0, three at
exit 120 (the two defect cells
under a broken descriptor 2 and
the rerouted check cell), and four
at exit 1, among them the
closed-descriptor cell standing in
the OR-43 crash class as
contracted: exit, stdout, and file
agree, the oracle's stderr carries
its traceback, and the port's
stderr is empty. The pass's final
accounting is the harness's own
over the complete run: REALISTIC
cells 3100, splits 0. The pass ran
detached with a durable log under
the round's workspace scratch.
Four detached attempts stand in
this round's record. The first ran
about 100 minutes and died in
phase 4 when a write-fault wave
refused every write on the box;
its phase 1 had completed with the
two sides' records byte-identical
at the contracted figures, and it
is counted nowhere. The second
failed at startup on the same
wave, when no temporary directory
on the box was usable, and is
counted nowhere. The third
completed phase 1 whole at the
contracted figures and died in
phase 2 when a service restart
took the process, and is counted
nowhere. The fourth attempt
completed whole in about 100
minutes, and every figure in this
item comes from that completed run
alone.

- OR-111. The gate record whole.
The final gate re-run whole on the
closure tree: cargo test exit 0,
69 passed and 0 failed (50 store,
19 streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt --check
exit 0. The final oracle tree
stands verified through the API as
it has since OR-77: commit
0c25a9aa703b38c7579f7928d61443a881a62465,
tree
f88eb75044f2e751915ab9a825426177972f2b58,
all 14 blobs matched by recomputed
git blob id against the extraction
the batteries at OR-109 and OR-110
ran. The flighttape pin stands
reconfirmed through the API:
commit
74a83298aeb31c82e62b9fb0c613f5bc8600799e,
tree
afd4d7d13520411e003762548fc79082a1b9cf6e,
exactly 50 blobs, and the golden
digest
(cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3)
executed, not merely cited,
through the suite's digest test in
the gate runs of this round, on
the base tree at OR-109 and on the
closure tree here. A Python
codepoint scan over all 21 blobs
of the closure tree itself finds
zero U+2014 and zero U+2013.

- OR-112. Closure. Round 19 closes
at 19 of 100, the README rounds
line moving in this commit only.
The gate stands at OR-111; the
battery at OR-109 and the budgets
and realistic pass at OR-110 stand
as the round's measurements.
OR-51, the Mach-O
constructor-section arm, stands
carried by name as a named
limitation, neither repaired nor
reopened. The round's provenance:
one seating, banked family by
family as each finished (f4fb530d
OR-108, f1f993c8 OR-109, f8af031d
OR-110, this commit OR-111 and
OR-112), each push asserting
tree-SHA equality against the
local tree before landing. The
round's environment effects, in
full: a write-fault wave and a
service restart touched the
round's schedule and nothing it
contracts; the wave killed the
realistic pass's first attempt in
phase 4 and refused its second at
startup, and a restart killed its
third in phase 2, as OR-110
records, while the battery
completed on its first run in an
unfaulted window and every figure
this round contracts comes from a
completed run or gate in an
unfaulted window. No measurement
this round contracts was taken in
a faulted state.

- OR-113. PR 22 read back
whole on the closed record: 0
reviews, 0 review comments,
and 2 issue comments, every
body read to its end, the pull
request body included (7,221
characters at this read, the
owner's repaired final form).
Its CodeRabbit shell
(6902745414) stands in its
true final form at this read
at 13,073 characters, created
2026-11-15T22:54:23Z and last
edited 2026-11-15T23:18:29Z.
The shell carries three forms
in the record, each accounted
at its own moment: 12,966
characters at the pull request
body's read, the shell last
edited then at 22:56:57Z;
12,936 characters at the Round
19 gate's reads, settled
downward, last edited
23:03:06Z; and 13,073
characters at the owner's
brief-time read and at this
read, the shell having
re-rendered upward by 137
characters in the minute
before the landing comment
posted, every
disposition-bearing element
unchanged across the three
forms. At this read the shell
stands as a completed review
of the closure head ceb802bc:
Merge Risk Minimal up to
ceb80, pre-merge checks 3
passed and 1 inconclusive
(Description, Linked Issues,
Out of Scope Changes passed;
Title inconclusive), the word
Docstring occurring zero
times, no rate-limit banner,
the standing line that no
actionable comments were
generated in the recent
review, and Run ID
19e01f40-7536-4bd7-a058-f902bb836158,
with files selected
CONTRACT.md and README.md and
the included allowance used
with none remaining. Its one
reservation stands in the
inconclusive Title check and
stands dismissed on the
grounds recorded in PR 22's
body, re-graded at this read:
the passage reads that the
title names the round and
ticket range but not the main
change, and that it is too
generic to convey the primary
change to a teammate scanning
history, while its own
explanation records that the
title is related to the
changeset, so it does not
fail; the check stands
inconclusive, not failed; the
title's form is the standing
form of this line's round PRs,
identical in shape to PR 21's
and PR 20's titles read
through the API at this read;
the range the title names,
OR-108 to OR-112, heads
exactly one item each in the
merged contract, counted at
this read; and the shell's
standing line records that no
actionable comments were
generated. No trigger comment
stands and no auto-reply
stands: the bot reviewed
automatically at the pull
request's opening, and the
owner's landing comment
(6903058961, 2,496 characters,
created 2026-11-15T23:19:00Z
and never edited) completes
the census, the landing
comment last. The bot's
release-notes appendix on PR
22's body stands charged as
dropped, at the summary scope
PR 22's body records: the
body's step 0 paragraph
censuses it as the bot's own
text, a Summary by CodeRabbit
section recording
documentation updates to the
project status and the round's
measurement records, and the
owner's repair note on the
final body records that the
patch that completed the body
dropped it and that its exact
text is not recoverable from
the lane's records. Its
absence from PR 22's final
body is the standing state
this item contracts, not a
discovery at this read; the
text is not reconstructed
here, and the absence is not
graded as a new defect. OR-51,
the Mach-O constructor-section
arm, stands carried by name as
a named limitation at its
standing scope, neither
repaired nor reopened, and
stands named again at this
round's closure. Every comment
ID and figure in this round's
brief was verified against the
live record before contraction
and stands as relayed, so no
correction is owed. Nothing
was posted on PR 22 after the
landing comment, so no late
CodeRabbit finding stands
undisposed.

- OR-114. The gate on the base
tree and the stream law
re-measured fresh at their
exact points. The gate ran on
the Round 19 merge tree before
any item was written: cargo
test exit 0, 69 passed and 0
failed (50 store, 19 streams);
cargo clippy --all-targets
with warnings denied, exit 0;
cargo fmt --check exit 0. The
shipped binary's provenance
was settled before any cell
ran: built fresh from the base
tree, and a full clean rebuild
at the same path reproduced it
byte-identically (md5
89e9a9b629b9d484548ad381c4829193
at this round's scratch path),
so the driven binary is that
tree's binary. A 55-cell
battery at Round 14's shape,
re-executed whole at the
shipped CLI against the final
oracle tree verified at
OR-116: 0 splits, of which
this item covers the 39 stream
cells, enumerated as at OR-78
(9 + 9 + 9 + 3 + 4 + 5), and
the remaining 16 budget cells
stand at OR-115. The battery
completed on its first run in
an unfaulted window, and no
figure in this item comes from
any other run. The
single-defect refusal at
printed totals of exactly
8,191, 8,192, and 8,193 ends
at 120 at a broken descriptor
2 (at /dev/full and at a
readerless pipe) at all three
totals, and at 1 healthy.
Check rendering one defect on
descriptor 1 at the same
totals answers 120 with the
shutdown dump at 8,191 and 1
with the OS error named at
8,192 and 8,193: a write that
fills the 8,192-byte buffer
exactly flushes at fullness,
and 8,191 defers. A get view
tuned to the same totals
answers 1 at a broken
descriptor 1 at all three. The
rerouted refusal (descriptor 2
closed at launch, descriptor 1
at /dev/full) flips at
capacity: 120 at 8,191, 1 at
8,192 and 8,193. The
past-capacity refusal of OR-56
(600 bogus lines) answers 120
at /dev/full and at a
readerless pipe, 1 healthy,
and 1 rerouted. Five add cells
re-verify the side-effect
order (the append lands before
the emit; a refusal appends
nothing); the one
launch-closed add cell differs
only on descriptor 2, in
exactly the crash class OR-43
records. The three postures,
per writer: on descriptor 2,
the refusal writer, a healthy
descriptor lands the refusal
at 1, a broken-but-open
descriptor (/dev/full or a
readerless pipe) loses the
channel and ends at 120, and a
launch-closed descriptor
reroutes the refusal onto
descriptor 1, where the
capacity flip above decides
it; on descriptor 1, the emit
writer, a healthy descriptor
renders, a broken-but-open
descriptor answers 1 at every
tuned total for the get emit
shape and, for check's defect
rendering, 120 by the shutdown
flush under capacity and 1
with the failure named at and
past it, and a launch-closed
descriptor stands in the OR-43
crash class on the add cell.
No stream point moved, so no
new pin is owed.

- OR-115. The read budgets and
the realistic pass re-measured
fresh. The read budgets at
their exact flip points, the
remaining 16 cells of the
battery at OR-114, each
boundary point itself included
with a cell on each side, each
cell asserting the oracle's
exit direction against the
contracted side of the
boundary, and the constants
first read from src/store.rs
at this round's base
(BUDGET_READ (990, 988) and
BUDGET_READ_NARROW (989,
987)). Under BUDGET_READ (990,
988) through check, nested
arrays verify at 986 and 987
and refuse at 988 and 989, and
nested objects verify at 985
and 986 and refuse at 987 and
988. Under BUDGET_READ_NARROW
(989, 987) through get, nested
arrays verify at 985 and 986
and refuse at 987 and 988, and
nested objects verify at 984
and 985 and refuse at 986 and
987. The 989 slot at the wide
width is refused and its
verify side stands at 987; the
988 slot at the narrow width
is refused and its verify side
stands at 986. No budget point
moved, so no new pin is owed.
The realistic pass re-run
fresh at Round 12's shape in
full (the OR-76 construction),
against the shipped CLI and
the final oracle tree verified
at OR-116: 3,100 compared
steps, 0 splits, composed of
2,600 adds with a supersession
of each subject's live head
every third add on that
subject and 416 interleaved
read steps in phase 1, 55
whole-history cells in phase
2, 16 budget cells at the
OR-75 depths on the big
history in phase 3, and 13
stream cells against the big
store in phase 4. The composed
record stands at 2,600 lines
and 652,833 bytes, byte for
byte the OR-76 figures. Phase
4's thirteen cells all agree:
six at exit 0, three at exit
120 (the two defect cells
under a broken descriptor 2
and the rerouted check cell),
and four at exit 1, among them
the closed-descriptor cell
standing in the OR-43 crash
class as contracted: exit,
stdout, and file agree, the
oracle's stderr carries its
traceback, and the port's
stderr is empty. The pass's
final accounting is the
harness's own over the
complete run: REALISTIC cells
3100, splits 0. The pass ran
detached with a durable log
under the round's workspace
scratch. Two detached attempts
stand in this round's record.
The first ran about 55 minutes
in phase 1, its two sides'
records growing in lockstep at
every spot check, and died
when a service restart took
the process at 2,197 of the
2,600 adds, the two records
frozen byte-identical at 2,197
lines, and it is counted
nowhere. The second attempt
completed whole in about 67
minutes, and every figure in
this item comes from that
completed run alone.

- OR-116. The gate record
whole. The final gate re-run
whole on the closure tree:
cargo test exit 0, 69 passed
and 0 failed (50 store, 19
streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt
--check exit 0. The final
oracle tree stands verified
through the API as it has
since OR-77: commit
0c25a9aa703b38c7579f7928d61443a881a62465,
tree
f88eb75044f2e751915ab9a825426177972f2b58,
all 14 blobs matched by
recomputed git blob id against
the extraction the batteries
at OR-114 and OR-115 ran, 0
mismatches. The flighttape pin
stands reconfirmed through the
API: commit
74a83298aeb31c82e62b9fb0c613f5bc8600799e,
tree
afd4d7d13520411e003762548fc79082a1b9cf6e,
exactly 50 blobs, and the
golden digest
(cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3)
executed, not merely cited,
through the suite's digest
test in the gate runs of this
round, on the base tree at
OR-114 and on the closure tree
here. A Python codepoint scan
over all 21 blobs of the
closure tree itself finds zero
U+2014 and zero U+2013.

- OR-117. Closure. Round 20
closes at 20 of 100, the
README rounds line moving in
this commit only. The gate
stands at OR-116; the battery
at OR-114 and the budgets and
realistic pass at OR-115 stand
as the round's measurements.
OR-51, the Mach-O
constructor-section arm,
stands carried by name as a
named limitation, neither
repaired nor reopened. The
round's provenance: one
seating, banked family by
family as each finished
(1919f1f5 OR-113, c7903039
OR-114, 683f0d03 OR-115, this
commit OR-116 and OR-117),
each push asserting tree-SHA
equality against the local
tree before landing. The
round's environment effects,
in full: a service restart
touched the round's schedule
and nothing it contracts; the
restart killed the realistic
pass's first detached attempt
in phase 1, as OR-115 records,
while the battery completed on
its first run in an unfaulted
window and every figure this
round contracts comes from a
completed run or gate in an
unfaulted window. No
measurement this round
contracts was taken in a
faulted state.

- OR-118. PR 23 read back
whole on the closed record: 0
reviews, 0 review comments,
and 2 issue comments, every
body read to its end, the pull
request body included (7,861
characters at this read). Its
CodeRabbit shell (6911441349)
stands in its true final form
at this read at 11,218
characters, created
2026-11-16T08:33:51Z and last
edited 2026-11-16T08:39:37Z.
The shell carries three forms
in the record, each accounted
at its own moment: 11,214
characters at the pull request
body's first read, the shell
last edited then at 08:36:07Z;
11,216 characters at the
body's final read, last edited
08:38:59Z; and 11,218
characters at the Round 20
gate's reads and at this read,
the shell having re-rendered
upward by two characters at
each step, every
disposition-bearing element
unchanged across the three
forms. At this read the shell
stands as a completed review
of the closure head 6365fd33:
Merge Risk Minimal up to
6365f, pre-merge checks 4
passed and 0 failed
(Description, Title, Linked
Issues, Out of Scope Changes),
the word Docstring occurring
zero times, no rate-limit
banner, the standing line that
no actionable comments were
generated in the recent
review, and Run ID
009e76f4-3457-4265-a4d0-009e7c273dd5,
with files selected
CONTRACT.md and README.md and
the included allowance used
with none remaining. No
reservation stands in the
shell and no dismissal stands
on the record: its Title check
stands Passed, and the mild
remark in that check's full
details, that the title does
not name the verification
results or the closure,
withdraws itself in the same
passage (it is specific enough
for a reviewer to find the
primary change), so it stands
recorded as a disposition in
PR 23's body, re-graded at
this read: the check stands
passed, not inconclusive or
failed; the title (43
characters) names the round
and the OR-113 to OR-117
range, and that range heads
exactly one item each in the
merged contract, counted at
this read; the title's form is
the standing form of this
line's round PRs; and the
shell's standing line records
that no actionable comments
were generated. No review
object and no review comments
stand, so nothing was fixed.
No trigger comment stands and
no auto-reply stands: the bot
reviewed automatically at the
pull request's opening, and
the owner's landing comment
(6911812006, 1,745 characters,
created 2026-11-16T08:52:37Z
and never edited) completes
the census, the landing
comment last. The bot's
release-notes appendix on PR
23's body stands preserved at
the body's foot inside its own
markers, censused in the
whole-body read at this read
at 387 characters counting
from its opening marker: a
Summary by CodeRabbit section
with two bullets under
Documentation, recording the
records covering Round 20 and
the README's move to 20 of
100, each correct at this
read, and PR 23's body records
the appendix preserved byte
for byte through that body's
patch. OR-51, the Mach-O
constructor-section arm,
stands carried by name as a
named limitation at its
standing scope, neither
repaired nor reopened, and
stands named again at this
round's closure. Every comment
ID and figure in this round's
brief was verified against the
live record before contraction
and stands as relayed, so no
correction is owed. Nothing
was posted on PR 23 after the
landing comment, so no late
CodeRabbit finding stands
undisposed.

- OR-119. The gate on the base
tree and the stream law
re-measured fresh at their
exact points. The gate ran on
the Round 20 merge tree before
any item was written: cargo
test exit 0, 69 passed and 0
failed (50 store, 19 streams);
cargo clippy --all-targets
with warnings denied, exit 0;
cargo fmt --check exit 0. The
shipped binary's provenance
was settled before any cell
ran: built fresh from the base
tree, and a full clean rebuild
at the same path reproduced it
byte-identically (md5
ebacf7f3670e658eeb1fa9c9512aed18
at this round's scratch path),
so the driven binary is that
tree's binary. A 55-cell
battery at Round 14's shape,
re-executed whole at the
shipped CLI against the final
oracle tree verified at
OR-121: 0 splits, of which
this item covers the 39 stream
cells, enumerated as at OR-78
(9 + 9 + 9 + 3 + 4 + 5), and
the remaining 16 budget cells
stand at OR-120. The battery
completed on its first run in
an unfaulted window, and no
figure in this item comes from
any other run. Its cell list
stands identical to the Round
20 run of record line for
line, and the harness differs
from Round 20's in exactly
three lines: the round label,
the temp prefix, and the
summary print label
(BATTERY21). The single-defect
refusal at printed totals of
exactly 8,191, 8,192, and
8,193 ends at 120 at a broken
descriptor 2 (at /dev/full and
at a readerless pipe) at all
three totals, and at 1
healthy. Check rendering one
defect on descriptor 1 at the
same totals answers 120 with
the shutdown dump at 8,191 and
1 with the OS error named at
8,192 and 8,193: a write that
fills the 8,192-byte buffer
exactly flushes at fullness,
and 8,191 defers. A get view
tuned to the same totals
answers 1 at a broken
descriptor 1 at all three. The
rerouted refusal (descriptor 2
closed at launch, descriptor 1
at /dev/full) flips at
capacity: 120 at 8,191, 1 at
8,192 and 8,193. The
past-capacity refusal of OR-56
(600 bogus lines) answers 120
at /dev/full and at a
readerless pipe, 1 healthy,
and 1 rerouted. Five add cells
re-verify the side-effect
order (the append lands before
the emit; a refusal appends
nothing); the one
launch-closed add cell differs
only on descriptor 2, in
exactly the crash class OR-43
records. The three postures,
per writer: on descriptor 2,
the refusal writer, a healthy
descriptor lands the refusal
at 1, a broken-but-open
descriptor (/dev/full or a
readerless pipe) loses the
channel and ends at 120, and a
launch-closed descriptor
reroutes the refusal onto
descriptor 1, where the
capacity flip above decides
it; on descriptor 1, the emit
writer, a healthy descriptor
renders, a broken-but-open
descriptor answers 1 at every
tuned total for the get emit
shape and, for check's defect
rendering, 120 by the shutdown
flush under capacity and 1
with the failure named at and
past it, and a launch-closed
descriptor stands in the OR-43
crash class on the add cell.
No stream point moved, so no
new pin is owed.

- OR-120. The read budgets and
the realistic pass re-measured
fresh. The read budgets at
their exact flip points, the
remaining 16 cells of the
battery at OR-119, each
boundary point itself included
with a cell on each side, each
cell asserting the oracle's
exit direction against the
contracted side of the
boundary, and the constants
first read from src/store.rs
at this round's base
(BUDGET_READ (990, 988) and
BUDGET_READ_NARROW (989,
987)). Under BUDGET_READ (990,
988) through check, nested
arrays verify at 986 and 987
and refuse at 988 and 989, and
nested objects verify at 985
and 986 and refuse at 987 and
988. Under BUDGET_READ_NARROW
(989, 987) through get, nested
arrays verify at 985 and 986
and refuse at 987 and 988, and
nested objects verify at 984
and 985 and refuse at 986 and
987. The 989 slot at the wide
width is refused and its
verify side stands at 987; the
988 slot at the narrow width
is refused and its verify side
stands at 986. No budget point
moved, so no new pin is owed.
The realistic pass re-run
fresh at Round 12's shape in
full (the OR-76 construction),
against the shipped CLI and
the final oracle tree verified
at OR-121, stands not
contracted this round: nine
attempts stand in this round's
record and no attempt
completed, so no figure of the
pass is contracted here and
the pass's last contracted
state stands at OR-115, where
a completed run recorded 3,100
compared steps and 0 splits
with the composed record at
2,600 lines and 652,833 bytes.
Each of the nine is counted
nowhere, and no counted pass
rests on any of them. The
first ran about 50 minutes in
phase 1, its two sides'
records growing in lockstep at
every spot check, and died
when a box restart took the
process at 2,386 of the 2,600
adds, the two records standing
in lockstep at 2,386 lines and
598,486 bytes each. The second
attempt completed phase 1
whole inside itself, its
composed record standing at
the OR-76 figures on both
sides and its log recording
the phase's completion with no
split printed, and was killed
before the pass completed, its
later phases never printing,
the process gone after the box
restart at about 19:11Z. The
third attempt composed the
full record, its two sides
standing at 2,600 lines and
652,833 bytes each, and was
killed by the box restart at
about 21:30Z before the pass
completed, its log never
written. The fourth attempt
composed the full record at
the same figures and never
completed, its log never
written, the process gone
after the box restart at about
03:29Z on the following day.
The fifth attempt ran about
105 minutes with no box
restart intervening, its
record standing composed at
the same figures, and was gone
with its log never written and
its fourth phase never begun.
The sixth and seventh attempts
failed at launch in a faulted
write window before any cell
ran, the log writer refused
and no temporary directory
usable. The eighth attempt ran
in the foreground of its
supervising session, its
output carried to the durable
log by a consumer that retries
writes through the fault, and
was killed about six minutes
into its first phase by the
box restart at about 05:30Z.
The ninth attempt ran in the
same supervised form and was
killed by the box restart at
about 07:19Z in phase 1 at
2,310 of the 2,600 adds, its
records standing in lockstep
at 2,310 lines and 579,858
bytes each and its log never
written. As partial evidence
only, contracted as nothing:
the frozen record pairs of the
six attempts that composed
records (the first, second,
third, fourth, fifth, and
ninth) stand byte-identical
between the two sides at each
attempt's frozen length, so
the composition agreed step
for step as far as each of
those attempts ran. The
re-run's completion is carried
to the next round's charges as
an open item, the same posture
OR-51 stands in below.

- OR-121. The gate record
whole. The final gate re-run
whole on the closure tree:
cargo test exit 0, 69 passed
and 0 failed (50 store, 19
streams); cargo clippy
--all-targets with warnings
denied, exit 0; cargo fmt
--check exit 0. The final
oracle tree stands verified
through the API as it has
since OR-77: commit
0c25a9aa703b38c7579f7928d61443a881a62465,
tree
f88eb75044f2e751915ab9a825426177972f2b58,
all 14 blobs matched by
recomputed git blob id against
the extraction the batteries
at OR-119 and OR-120 ran, 0
mismatches. The flighttape pin
stands reconfirmed through the
API: commit
74a83298aeb31c82e62b9fb0c613f5bc8600799e,
tree
afd4d7d13520411e003762548fc79082a1b9cf6e,
exactly 50 blobs; the pin's
own suite run whole at this
round's hand at 241 passed and
0 failed, and the golden
executed through the pin's own
CLI, converting
tests/fixtures/claude_session_synthetic.jsonl
to a 1,663-byte tape
byte-identical to this repo's
tests/fixtures/tape_synthetic.json,
its digest printing
cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
and its verify accepting at
exit 0; the suite's own digest
test
(the_digest_is_pinned_to_the_family_golden)
passed in the gate runs of
this round, on the base tree
at OR-119 and on the closure
tree here. A Python codepoint
scan over all 21 blobs of the
closure tree itself finds zero
U+2014 and zero U+2013.

- OR-122. Closure. Round 21
closes at 21 of 100, the
README rounds line moving in
this commit only. The gate
stands at OR-121; the battery
at OR-119 and the budgets at
OR-120 stand as the round's
contracted measurements. Two
items stand carried open by
name. OR-51, the Mach-O
constructor-section arm,
stands carried as a named
limitation, neither repaired
nor reopened. The realistic
pass's re-run stands carried
uncontracted, as OR-120
records: nine attempts in this
round's record, no completed
run, no figure contracted, its
last contracted state standing
at OR-115, and its completion
stands charged to the next
round. The round's provenance:
one seating, banked family by
family as each finished
(02204df3 OR-118, 8ff2254d
OR-119, b9c74806 OR-120, this
commit OR-121 and OR-122),
each push asserting tree-SHA
equality against the local
tree before landing, every
push server-side through the
Git Data API with the branch
created at the base through
the API and no local git push
used. The round's environment
effects, in full: box restarts
at about 17:09Z, 19:11Z, and
21:30Z on the first day and at
about 03:29Z, 05:30Z, and
07:19Z on the second, and the
box write fault in its
flickering windows, run
through the round's record,
and the realistic pass's nine
attempts failed in that
environment, as OR-120
records; the battery completed
on its first run and every
gate this round contracts ran
whole in an unfaulted window.
No measurement this round
contracts was taken in a
faulted state, and no counted
pass rests on a killed or
refused attempt.

- OR-123. PR 24 read back
whole on the closed record: 0
reviews, 0 review comments,
and 4 issue comments, every
body read to its end, the pull
request body included (9,211
characters at this read). Its
CodeRabbit shell (6941916764)
stands in its true final form
at this read at 11,259
characters, created
2026-11-17T07:41:41Z and last
edited 2026-11-17T13:46:08Z.
The shell carries three forms
in the record, each accounted
at its own moment: 11,255
characters at the pull request
body's gate read, the shell
last edited then at
2026-11-17T07:45:29Z; 11,257
characters at the owner's gate
patch, last edited
2026-11-17T07:51:34Z; and
11,259 characters at the final
re-render, last edited
2026-11-17T13:46:08Z, standing
unchanged from that edit
through the queue comment, the
merge, and this read. The
landing comment's phrase
placing the further re-renders
at the queue comment and the
merge is loose on timing
alone, the final re-render
having landed before both; its
figure of 11,259 characters at
the landing is exact. The
shell in its final form
carries the completed review
of the closure head 3c8ad65e:
Merge Risk Minimal up to
3c8ad, pre-merge checks 4
passed and 0 failed (Title,
Linked Issues, Out of Scope
Changes, Description), the
word Docstring occurring zero
times, no rate-limit banner,
the standing line that no
actionable comments were
generated in the recent
review, Run ID
46e307f7-bd40-456c-9b3d-65e7df66ba9b,
files selected CONTRACT.md and
README.md, the included
allowance used with none
remaining. The Title check
stands Passed, its full
details affirming that the
title names the primary
change, is specific, and is
accurate. No reservation
stands in the shell and no
dismissal stands on the round:
no review object and no review
comments stand on the pull
request, so nothing was fixed
and nothing was dismissed, and
that posture stands confirmed
at this read. The owner's
queue comment (6953287480)
stands at 12 characters, the
words /trunk merge, created
2026-11-17T15:57:09Z and never
edited. The trunk notice
(6953290494) stands at 113
characters, created
2026-11-17T15:57:16Z and
edited once, at
2026-11-17T16:04:39Z, to its
final text recording the merge
as successful through the
queue. The landing comment
(6953705853) stands at 1,799
characters, created
2026-11-17T16:12:58Z, never
edited, standing last; nothing
stands posted after it, so the
fourth charge of that landing
stands discharged with nothing
to disposition. The bot's
release-notes appendix stands
at the pull request body's
foot inside its own markers at
493 characters counting from
its opening marker: a Summary
by CodeRabbit section with
three bullets under
Documentation (the status line
to 21 of 100; Round 21 records
covering reviews,
measurements, and
verification; nine incomplete
realistic-pass attempts
documented with the pass
remaining open and no result
recorded), each correct at
this read.

- OR-124. Base gate and
battery. The base gate ran
whole on the base tree
(94571683, the Round 21
closure tree) at this round's
hand: cargo test 69 passed and
0 failed (50 store, 19
streams), clippy with warnings
denied clean, fmt clean, every
stage on its first attempt
after the slot turn came. The
debug binary at this round's
scratch target path carries
md5
ebacf7f3670e658eeb1fa9c9512aed18,
the same digest as Round 21's
build at its own path.
Execution read the source from
the Round 21 lane's worktree,
whose local tree is the base
tree exactly; the brief's
fresh worktree could not be
materialized in the session's
write-fault regime, and the
deviation stands disclosed on
the pull request record. The
battery ran whole against this
build and the oracle verified
at OR-123's read-back shape
(0c25a9a, 14 blobs, 0
mismatches): 55 cells (39
stream, 16 budget), 0 splits,
every cell agreeing, on its
first executed run; one
earlier launch failure, on
this lane's own harness path
error with zero cells run,
stands in the round's attempt
history on the pull request
record. The budget constants
were re-read from the base
tree's src/store.rs lines 30
and 31 before the run:
BUDGET_READ (990, 988) and
BUDGET_READ_NARROW (989, 987),
unchanged.

- OR-125. The realistic
completion charged at OR-122
stands UNCONTRACTED again, and
its completion stands charged
to Round 23 as that round's
first business. The pass last
stands contracted at OR-115:
3,100 cells, 0 splits, the
composed record at 2,600 lines
and 652,833 bytes, its fourth
phase split six cells at exit
0, three at exit 120, four at
exit 1. This round's history:
one launch failure before any
attempt, the harness pointed
at the oracle's extraction
root instead of its src
directory, burning its retries
in under a minute with zero
cells run, the same class as
Round 21's sixth and seventh
attempts, corrected before the
first real attempt. Attempt 1
composed phases 1 to 3 whole,
its phase 1 record exact at
2,600 lines and 652,833 bytes,
then the write fault refused
its fourth phase's first seed
write and it exited with no
final accounting after 89
minutes. Attempt 2 was killed
by a box restart in phase 1,
last read at 1,487 of 2,600
adds in lockstep, its later
state unknowable. Attempt 3
repeated attempt 1's shape
whole: phases 1 to 3 complete,
the same refusal at the fourth
phase's first seed write, dead
after 76 minutes with probe
writes refused across the box
in the same minute. Attempt 4
never launched: its seating
died queued in a restart after
42 minutes, unrun. Attempt 5
ran to completion in 107
minutes and printed its
accounting, 3,100 cells and
1,271 splits, and counts
nowhere: the splits are the
fault's own, both sides
refusing appends together
through the fault windows with
only their refusal texts'
bytes differing, one one-sided
append failure desyncing the
pair mid-phase, its composed
record standing byte-identical
on disk at 2,458 lines and
617,216 bytes with 142 of its
adds never landed, its third
phase agreeing 16 of 16 and
its fourth 13 of 13 on that
record. Attempt 6 died to a
restart three minutes into its
run, last read at 344 adds in
lockstep. Attempt 7 completed
in 47 minutes with 2,064
splits of the same making, its
first one-sided refusal at
add-835, its composed pair
byte-identical at 2,205 lines
and 553,083 bytes with 395
adds never landed, its fourth
phase agreeing 13 of 13. As
partial evidence only,
contracted as nothing: the
frozen record pairs of
attempts 1 and 3 stand
byte-identical between the
sides at the full 2,600 lines,
attempt 5's pair at its 2,458,
and attempt 7's at its 2,205.
Every attempt stands counted
nowhere.

- OR-126. Verification family.
The oracle was re-verified
through the API before any
harness pointed at it: commit
0c25a9aa703b38c7579f7928d61443a881a62465,
tree
f88eb75044f2e751915ab9a825426177972f2b58,
14 blobs, 0 mismatches against
the local extraction, whose
package stands at its src
directory. The flighttape pin
was reconfirmed the same way:
commit
74a83298aeb31c82e62b9fb0c613f5bc8600799e,
tree
afd4d7d13520411e003762548fc79082a1b9cf6e,
exactly 50 blobs, 0
mismatches. The pin suite ran
whole in the verified
materialization with a fresh
target dir at this round's
scratch: 241 passed, 0 failed.
The golden was executed
through the pin's own CLI:
convert of the pin's
tests/fixtures/claude_session_synthetic.jsonl
produced a 1,663-byte tape
byte-identical to onrecord's
tests/fixtures/tape_synthetic.json,
the digest printed
cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3
at exit 0, and verify accepted
at exit 0. Checklist item 6
executed: cargo install into a
scratch CARGO_HOME completed,
the installed binary at
873,744 bytes. Item 7 executed
from that binary: the
quickstart reproduces the
README exactly, the add
printing
claim-2946b699934e0f6b, the
get view byte-identical to the
printed view at 719 bytes,
check printing the record is
sound. Item 3 re-verified
live: the GitHub description,
the README lede, and the
Cargo.toml package description
stand byte-identical, passing
outright.

- OR-127. Closure gate. The
closure gate ran whole on the
closure tree's code, which is
byte-identical to the base
tree's in every code blob,
only CONTRACT.md and README.md
differing between the trees:
after a full clean rebuild at
the same target path, cargo
test 69 passed and 0 failed
(50 store, 19 streams, the
counts read from the gate's
own built test binaries),
clippy with warnings denied
clean, fmt clean, every
stage's exit verified at 0 in
the session capture after two
earlier quota-killed attempts
at the same stages, whose
false clean lines in the first
follow-on script stand
corrected in the round's
record. The battery ran whole
on the closure build: 55
cells, 0 splits. Provenance at
build-path scope: the rebuilt
binary is byte-identical to
the first build, md5
ebacf7f3670e658eeb1fa9c9512aed18
at the same target path. A
Python codepoint scan over all
21 blobs of the closure tree
itself, executed over the
constructed tree before its
commit landed, finds zero
U+2014 and zero U+2013.

- OR-128. Round 22 closes at
22 of 100. The README rounds
line moves to 22 of 100 in
this closure commit only.
OR-51 stands carried open by
name into Round 23 as a named
limitation, the Mach-O
constructor-section arm,
neither repaired nor reopened
in this round, per the round's
charge.

- OR-129. Repair record, appended after
the three reviews of the
closure head, answering the
gate's findings on this
round's record (spec fidelity
F1 and F2; standards F1,
graded Major on that axis, and
F2; red team F1 and F2), with
OR-123 and OR-125 unrewritten.
The census first. OR-123 reads
PR 24 at 4 issue comments and
closes with the landing
comment (6953705853) standing
last, nothing posted after it,
and the landing's fourth
charge discharged with nothing
to disposition; that closing
clause is false and stands
qualified by this record. At
this record's own census PR 24
stands at six issue comments:
the shell (6941916764), the
owner's queue comment
(6953287480), the trunk notice
(6953290494), the landing
(6953705853, 1,799 characters,
created 2026-11-17T16:12:58Z,
never edited), and two posts
after the landing. The first,
the owner's stray duplicate
queue comment (6954365104, 12
characters, the words /trunk
merge, created
2026-11-17T16:37:36Z), was
posted by the owner's stale
enqueue script on the
already-merged pull request
and stands disclosed in the
owner's Session 186 record.
The second, trunk's error
notice (6954366730, 84
characters, created
2026-11-17T16:37:39Z), reads
"An error occurred while
submitting your PR to the
queue: `This PR is already
merged`" and is trunk's reply
to the duplicate. Both posts
are inert on their facts: each
stands on a closed,
already-merged pull request,
neither queued anything nor
could queue anything, and no
Round 21 figure, element, or
disposition moves for either.
Both stand dispositioned here,
which is what the fourth
charge asked of this
read-back, and the fourth
charge stands discharged on
this corrected census and not
on OR-123's clause. The pull
request 26 body's Spec
paragraph repeats the census
of four and the
nothing-after-the-landing
clause, and both stand
qualified by this record. The
attempt history second.
OR-125's account of the
realistic's seven attempts
omits this lane's own kill of
two queued sessions, an event
the lane's progress record
states plainly and which
stands recorded here, counted
nowhere: after attempt 1 died
at the fourth phase's first
seed write (2026-11-17
23:48:38Z), attempt 2 stood
queued behind the follow-on
rest session, whose provenance
stage ran cargo clean and
whose closure test stage then
failed six times inside the
same fault window that killed
attempt 1's phase 4, the clean
leaving the target dir without
a binary; this lane killed
both queued sessions, the rest
session and attempt 2's
queueing, which must not
launch against a missing
binary. For completeness the
same record states the two
queue deaths the lane's final
accounting keeps distinct from
the seven attempts: the
realistic's first two
queueings died unrun in the
box restarts at about 19:50Z
and about 20:44Z on
2026-11-17, each counted
nowhere. The mechanism third.
OR-125 states attempt 7's
first one-sided refusal at
add-835; the lane's progress
record, written from the same
FAIL lines, states add-835 as
py refused with the files
identical, a both-refused
event with the files in
agreement after the step, and
places the first one-sided
refusals at adds 872 and 873,
the pair desyncing there. A
one-sided refusal with the
other side landing cannot
leave the files identical, so
OR-125's sentence stands
qualified to the progress
record's account: the first
refusal event stands at
add-835, both sides refused
with the files identical, and
the first one-sided refusals
stand at adds 872 and 873. The
detail sits inside a
counted-nowhere attempt and
moves no disposition, and
OR-125 stands unrewritten in
every other particular, as
OR-123 does.

OR-130. PR 26 read back whole
at this round's opening
(charges 1 and 4 of the Round
22 landing comment
6982807997). The PR stands
merged as squash
1b34f00f19e368e982c6edca22ba60ca5f8a3f32
on 2026-11-18T13:59:35Z, this
round's base; its head stands
at
608beaa8c82a7c7ca818f28a398525f895e573ae
on tree
f1a13fcc5e5c3540fec93bfe4d8fc17e0aaf2340
over base
cb2822620d6f4063706ccb6957fefd1c32a63ec2,
and its body stands at 12,765
chars. The census at this
item's writing, refreshed at
2026-11-18T15:36Z: 4 issue
comments, 0 reviews, 0 review
comments, the landing comment
6982807997 (2,512 chars,
created 2026-11-18T15:18:37Z,
never edited) standing last.
The CodeRabbit shell
6971300569 stands in its true
final form at 11,290 chars,
last edited
2026-11-18T13:52:09Z: the
incremental review of the
repair head (Run ID
c33279dd-d097-4c51-8ee0-67c61579ab52,
reviewing the range 535dc0d2
to 608beaa8 with CONTRACT.md
the one file selected,
coverage standing at
608beaa8), its Merge Risk
standing at Minimal up to the
repair head, its pre-merge
checks at 4 passed and 0
failed, its standing line that
no actionable comments were
generated in the recent
review, and no review object
or review comment anywhere on
the PR, so nothing in it was
fixed. No reservation stands:
its Title check stands Passed,
the check's details noting
only that the title omits the
OR-129 census correction and
the README edit, acceptable
for a title, a mild remark
inside a passed check, so
nothing stands dismissed
either. Its walkthrough's
three rows are accurate in
substance, including its
naming of OR-129's census
correction. The shell's
earlier forms stand recorded
at 11,066 chars (the
closure-head review, Run ID
a2a1bcc5-2966-4c09-aad9-be683cb598dd)
and 11,068 at the red team's
read, each exact at its
moment; the body's step 0
paragraph states the shell at
11,066 and the Trunk notice at
403, both moment-bounded
figures exact at the lane's
reads and superseded by the
final forms contracted here.
The Trunk notice 6971297185
stands at 113 chars in its
post-merge form (merged
successfully, with the queue
details link), last edited
2026-11-18T13:59:39Z, its
403-char pre-merge form
standing at the gate reads.
The owner's queue comment
6980566561 stands at 12 chars,
the words /trunk merge,
created 2026-11-18T13:52:12Z
and never edited. The bot's
release-notes appendix stands
at the body's foot inside its
own markers at 606 chars
counting from its opening
marker (608 counting the two
separating newlines), its four
bullets under Documentation
standing as the Round 22
records; the realistic
attempts uncontracted; the
post-review repair and the
corrected comment count; the
README at 22 of 100, each
correct at this read; the
body's own step 0 paragraph
censuses the appendix at 592
chars with a different four,
naming OR-51 and not the
repair, the bot having edited
its appendix after the lane's
read, so the body's
enumeration stands superseded
on that limb, the appendix
being the bot's own text and
preserved byte for byte
through the owner's patch as
it then stood. The owner's
body patch graded against the
record: all four limbs stand
in the final body exactly as
charged (the Verified head
moved to 608beaa8 on tree
f1a13fcc, with OR-129 named in
the family list as the repair
record appended after the
three reviews on their
findings and passed by delta
reviews on both axes; the Spec
paragraph's census sentence
corrected to 6 issue comments
as corrected at OR-129; the
charge 4 clause restated with
both post-landing posts named
and dispositioned at OR-129;
and the kill sentence standing
at the realistic bullet's end,
recording the lane's kill of
its two queued sessions, each
counted nowhere), and the
patch changed no tree, the
head standing at 608beaa8 as
merged. OR-129's
qualifications stand
contracted as they stand: PR
24's census corrected to six
issue comments with both
post-landing posts
(6954365104, the owner's stray
duplicate queue comment from
the owner's stale enqueue
script, and 6954366730,
Trunk's error notice)
dispositioned as inert on the
already-merged PR, the Round
21 landing's charge 4
discharged on the corrected
census and not on OR-123's
clause; the attempt history
completed with the lane's kill
of its two queued sessions and
the two queue deaths (restarts
at about 19:50Z and 20:44Z on
2026-11-17) kept distinct from
the seven attempts, each
counted nowhere; and attempt
7's mechanism qualified (the
first refusal event at add-835
both sides refused with the
files identical, the first
one-sided refusals at adds 872
and 873), OR-123 and OR-125
standing unrewritten.
Post-landing material (charge
4): nothing stands posted
after the landing comment at
the refreshed census, and the
only post-merge edit anywhere
on the PR is the Trunk
notice's own move to its
post-merge form at 13:59:39Z,
dispositioned in this census;
charge 4 stands discharged
with nothing further to
disposition.

OR-131. The realistic and the
red team runner (charge 2 of
the Round 22 landing comment
6982807997). The Round 22 red
team's probe-gated runner
stands resolved at this
round's hand: its script
(rt_runner.sh, pid 88129,
launched 2026-11-18T08:33:41Z)
probed its full first window
of 45 probes without a clean
window and exited without
executing any stage; its
nowindow marker stands set,
its log holds only its START
line, none of its stage
markers stand (pin, golden,
append-refusal, realistic,
done), and a scan of /proc by
the script's path finds no
process of it, the one match
being the scanning shell's own
command line. It banked no
figures toward its untested
set, and no runner stands
armed undisclosed. Of that
untested set, this round
executed the pin suite and the
golden at its own hand
(OR-133); the append-refusal
reproduction was not re-run in
the red team's framing, the
refusal behavior it targeted
standing covered by the store
suite's refused-add pin, which
passed in this round's gate
(OR-132); and the realistic
attempt is this item's own
history. The realistic
completion, charged for the
third consecutive round,
stands carried again, on this
round's own complete attempt
record: eleven attempts, every
one counted nowhere, no
completed run's final
accounting standing in this
round. Attempt 1 began
2026-11-18T17:00:25Z and died
with its chain session under a
runtime SIGTERM at about
18:09Z, standing at 2,279 of
2,600 phase-1 adds with the
pair byte-identical (2,279
lines, 571,791 bytes, md5
c80b81a882dd828b3376fa27c18fc594
on both sides) and no split in
the compared prefix. Attempt 2
(18:16Z to 19:55Z) took a
split at add-2501 in the
refusal class as the
file-count quota wall rose,
its pair byte-identical at
2,508 lines and 629,360 bytes
(md5
764a37fde9b902375a92a4b3b0a2e436
on both sides), and died
silently against the wall with
no final accounting, the
chain's own result write
crashing on EDQUOT at
19:55:53Z. Attempt 3 (20:02Z
to 20:16Z) took splits at adds
487 and 765 and died when the
oracle interpreter itself
failed on OSError 122, Disk
quota exceeded, its crash dump
ending in the words lost
sys.stderr; its pair stands
byte-identical at 766 lines
and 191,417 bytes. Attempt 4
began at 20:28:07Z and died
with its session under a
runtime SIGTERM about three
minutes in, no temp root and
no harness log standing; a
session death, distinct from a
pass attempt's own failure.
Attempt 5 (20:44:07Z to
21:17:48Z) exited 1 with an
empty harness log and no final
accounting, its pair
byte-identical at 1,400 lines
and 350,449 bytes. Attempt 6
began at 21:26:41Z and died
with its session under a
runtime SIGTERM at about
22:10Z, some 44 minutes in,
its pair byte-identical at
1,052 lines and 263,123 bytes
(md5
26a86acd8e2ef726593ce7b9674ad652
on both sides) and no split in
the compared prefix. Attempt 7
(22:20:35Z to 23:54:59Z)
stands as the round's
strongest partial: phases 1 to
3 complete with zero splits,
phase 1 at 2,600 adds and 416
interleaved read steps with
the record composed at 2,600
lines and 652,833 bytes,
OR-115's exact figures
organically produced, phases 2
and 3 done; phase 4's first
cell then died at its seed
write on a PermissionError in
the harness itself, no
compared step of phase 4
having run. Attempt 8
(00:16:28Z to 01:20:41Z on
2026-11-19) exited 1 with an
empty log and no final
accounting, its pair standing
at the full composition, 2,600
lines and 652,833 bytes,
byte-identical (md5
6cba153da883f7b76ad69373719b72ac
on both sides), its death
silent and unlocated after the
record reached full
composition. Attempt 9
(01:25:39Z to 02:06:43Z)
exited 1 with an empty log and
no temp root created, no
compared step evidenced.
Attempt 10 (02:11:43Z to
02:21:45Z) exited 1 the same
way, empty log, no temp root,
no compared step evidenced.
Attempt 11 began at 02:26:24Z
and died with its session
under a runtime SIGTERM at
about 02:32Z, some six minutes
in, no temp root and no
harness log standing. Attempt
7's phases and the identical
pairs of attempts 2 and 8 are
stated as partial and
contracted as nothing. Two
further chain sessions died
before any attempt began (the
lane's first
shell-backgrounded launch at
15:29Z, killed with its
session before its first print
landed, and the run of
23:55:50Z, SIGTERMed inside
its probe window), each
counted nowhere. The last
contracted state of the
realistic stands at OR-115
unchanged (3,100 compared
steps, 0 splits; the composed
record at 2,600 lines and
652,833 bytes), and the
completion stands charged
forward to Round 24 as this
round's first business, for
the fourth consecutive round.

OR-132. The base gate and the
battery on the base tree. The
round worked from a verified
extraction of the base tree
f1a13fcc5e5c3540fec93bfe4d8fc17e0aaf2340
under the lane's scratch (21
blobs, every blob's git hash
verified in memory against its
tree entry and again after its
write, 0 mismatches), no
worktree materialized in the
fault regime, every target dir
and temp root under the
scratch. Cargo test stands at
69 of 69 in a single run (50
store and 19 streams, both
suites green), the run made in
a process tree whose /dev/full
stood verified genuine at the
run's own probe: a character
device at major 1, minor 7,
whose writes raise ENOSPC. The
gate's first two runs in other
trees measured the environment
instead: the store suite
passed 50 of 50 in both, and
the streams suite failed
exactly 11 named cells
(boundary_at_dev_full,
boundary_at_exact_capacity,
check_defect_at_dev_full_exits_120_with_the_dump,
check_single_defect_at_the_exact_points,
emit_with_stderr_closed_and_stdout_full_exits_1,
help_at_dev_full_exits_120_with_the_dump,
refusal_past_capacity_with_stderr_at_dev_full_exits_120,
refusal_single_defect_at_the_exact_points,
refusal_with_stderr_at_dev_full_exits_120,
refusal_with_stderr_closed_and_stdout_full_exits_120,
rerouted_refusal_flips_at_capacity),
each a cell that aims a stream
at /dev/full, each failing on
the bare exit 1 because in
those trees /dev/full stood as
a regular file whose writes
succeed (9,000 bytes at the
chain tree's probe); mknod
stands denied on this box,
inside a user namespace as at
the top level, so the device
cannot be repaired from
inside, and the cells stand
contracted only from the tree
where the device verified
genuine. Clippy (cargo clippy
--locked --all-targets -- -D
warnings) exits 0 and cargo
fmt --check exits 0 on the
base tree. The budget
constants stand re-read from
the extracted base tree at
src/store.rs lines 30 and 31:
BUDGET_READ the pair (990,
988) and BUDGET_READ_NARROW
the pair (989, 987).
Provenance: the debug binary's
md5 stands at
2d97c040cbdc00ca1695633ab796089d
before and after a full clean
rebuild (cargo clean, then
cargo build --locked at the
same target path), identical,
the figure scoped to its build
path with the source path
named as
~/workspace/retro-line/or23-scratch/work/base-src.
The battery on the base build:
55 cells (39 stream and 16
budget), 0 splits, exit 0.

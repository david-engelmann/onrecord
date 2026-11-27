# Onrecord Frequently Asked Questions

### Why does `onrecord` refuse two claims that both `--supersedes` the same earlier claim?
Supersession represents updating a specific line of reasoning (`A -> B`). If two agents independently disagree, they assert two separate claims on the same `--subject` (or one supersedes its own prior claim while the other asserts independently), which `onrecord subject` surfaces cleanly via `"conflict": true`. Allowing silent supersession forks inside one chain would blur sequential revision with parallel disagreement.

### Can multiple parallel agents append to the same `record.jsonl` simultaneously?
Because every claim ID hashes its `position` (line index), two concurrent writers racing without coordination could compute the same `position` index. Until native file locking lands in `Store::add`, serialize concurrent writes through a single lane lock (such as `flock record.jsonl.lock onrecord add ...`) or maintain per-lane ledgers.

### Why doesn't `onrecord add` default `--at` to the current wall clock?
Determinism and audit honesty: replaying the same `add` arguments on an empty store produces the exact same `claim-<16hex>` ID on any machine in any year, and callers cannot accidentally stamp delayed ingestion time in place of the actual observation timestamp.

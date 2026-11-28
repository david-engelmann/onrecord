//! `onrecord` is an append-only, cryptographically self-verifying claim store for cross-session
//! AI agent memory.
//!
//! Every stored claim carries `text`, `asserter`, ISO-8601 `asserted_at`, non-empty `evidence`,
//! linear `supersedes` provenance, optional `subject` and `due` (`YYYY-MM-DD`) fields, optional
//! downstream `extra` properties (such as `tetlock` confessions), and a position-bound Canonical
//! Form 1 identifier (`claim-<16hex>`, [`derive_id`]). Concurrent [`Store::add`] calls serialize
//! under an OS-level exclusive advisory file lock.
//!
//! ```text
//! let store = onrecord::Store::new(onrecord::RecPath::new("record.jsonl"));
//! let id = store.add(request)?;
//! let defects = store.check()?;
//! assert!(defects.is_empty());
//! ```

pub mod cli;
pub mod digest;
pub mod emit;
#[doc(hidden)]
pub mod helptext;
#[doc(hidden)]
pub mod pyrepr;
#[doc(hidden)]
pub mod repr_table;
pub mod store;
pub mod util;

pub use flighttape;
pub use store::{claim_defects, derive_id, load_record, AddClaim, RecPath, Store, StoreError};

static LAUNCH_FDS: std::sync::OnceLock<[i32; 3]> = std::sync::OnceLock::new();

#[doc(hidden)]
pub fn note_launch_fds(fds: [i32; 3]) {
    let _ = LAUNCH_FDS.set(fds);
}

pub(crate) fn launch_fd_closed(fd: i32) -> bool {
    match LAUNCH_FDS.get() {
        Some(fds) => fds[fd as usize] == -1,
        None => false,
    }
}

/// Execute the `onrecord` command-line interface on a dedicated 512 MiB stack thread
/// and return the process exit code (`0`, `1`, `2`, or `120`).
pub fn run_cli() -> i32 {
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}

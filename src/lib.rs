//! onrecord: an append-only claim store. Nothing is
//! stored as a bare fact: every claim carries who
//! asserted it, when, the evidence, and the chain it
//! superseded (OR-1 to OR-16).
//!
//! Round 3 is the Rust port. The JSON reader, the
//! value model, and the timestamp grammar are
//! consumed from the flighttape crate at a pinned
//! revision (OR-18), never re-implemented here.
//! Canonical form 1 comes from the same pin; the
//! digest module renders it with the Python digest's
//! float digit source, because the crate's renderer
//! is known to differ there (OR-18, and the flighttape
//! Round 12 dispositions).

pub mod cli;
pub mod digest;
pub mod emit;
pub mod helptext;
pub mod pyrepr;
pub mod repr_table;
pub mod store;
pub mod util;

pub use store::{claim_defects, derive_id, load_record, Store, StoreError};

/// The command line entry, in the cli module.
pub fn run_cli() -> i32 {
    // A generous stack, on the flighttape crate's own
    // precedent: parsed values nest as deep as their
    // files, and cloning or dropping a deep value
    // recurses with its depth.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}

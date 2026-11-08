//! The binary entry. Before the runtime's own init
//! can replace a launch-closed standard descriptor
//! with the null device, an .init_array constructor
//! records the fcntl F_GETFD result for descriptors
//! 0, 1, 2: -1 for a descriptor closed at launch.
//! No later observation can recover the fact (the
//! replacement is opened O_RDWR with no CLOEXEC,
//! indistinguishable from an explicit redirect), so
//! the record is handed to the library before the
//! command line runs (OR-43).

#[cfg(unix)]
use std::sync::atomic::{AtomicI32, Ordering};

#[cfg(unix)]
const F_GETFD: i32 = 1;

#[cfg(unix)]
extern "C" {
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
}

#[cfg(unix)]
static PRE_INIT_FDS: [AtomicI32; 3] = [
    AtomicI32::new(-99),
    AtomicI32::new(-99),
    AtomicI32::new(-99),
];

#[cfg(unix)]
extern "C" fn record_launch_fds() {
    for (index, fd) in (0..3i32).enumerate() {
        let result = unsafe { fcntl(fd, F_GETFD) };
        PRE_INIT_FDS[index].store(result, Ordering::SeqCst);
    }
}

#[cfg(unix)]
#[used]
#[link_section = ".init_array"]
static RECORD_LAUNCH_FDS: extern "C" fn() = record_launch_fds;

fn main() {
    #[cfg(unix)]
    onrecord::note_launch_fds([
        PRE_INIT_FDS[0].load(Ordering::SeqCst),
        PRE_INIT_FDS[1].load(Ordering::SeqCst),
        PRE_INIT_FDS[2].load(Ordering::SeqCst),
    ]);
    std::process::exit(onrecord::run_cli());
}

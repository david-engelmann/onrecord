//! OR-43 to OR-50. The closed-stream matrix, pinned
//! at the binary: launch-closed descriptors are
//! closed in a shell before exec, never redirected,
//! and the broken-stream cells write at /dev/full,
//! whose failure is deterministic. The readerless
//! pipe cells live in the standing differential
//! harness, whose section 17 asserts them against
//! the oracle in the same run.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_onrecord");
const DUMP_28: &str = "Exception ignored in: <_io.TextIOWrapper \
    name='<stdout>' mode='w' encoding='utf-8'>\n\
    OSError: [Errno 28] No space left on device\n";

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let dir =
            std::env::temp_dir().join(format!("onrecord-streams-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        Tmp(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Out {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run(args: &[&str], cwd: &Path) -> Out {
    let output = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run the binary");
    Out {
        code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

/// Run with descriptors closed at launch: a shell
/// closes them after its own startup, then execs
/// the binary, so the binary starts with the
/// descriptor closed exactly as a launching parent
/// would leave it.
fn run_closed(args: &[&str], cwd: &Path, close1: bool, close2: bool) -> Out {
    let mut script = String::new();
    if close1 {
        script.push_str("exec 1>&-; ");
    }
    if close2 {
        script.push_str("exec 2>&-; ");
    }
    script.push_str("exec \"$@\"");
    let output = Command::new("/bin/sh")
        .args(["-c", &script, "sh", BIN])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run the binary");
    Out {
        code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

/// Run with one standard stream aimed at /dev/full.
fn run_full(args: &[&str], cwd: &Path, full1: bool) -> Out {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let mut command = Command::new(BIN);
    command.args(args).current_dir(cwd);
    if full1 {
        command.stdout(Stdio::from(full));
    } else {
        command.stderr(Stdio::from(full));
    }
    let output = command.output().expect("run the binary");
    Out {
        code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

const ADD: &[&str] = &[
    "add",
    "r.jsonl",
    "--text",
    "The staging deploy is broken",
    "--asserter",
    "agent-a",
    "--at",
    "2026-01-01T09:00:00Z",
    "--evidence",
    "trace:run-17",
];

fn seed(tmp: &Tmp) -> String {
    let out = run(ADD, &tmp.0);
    assert_eq!(out.code, 0);
    String::from_utf8(out.stdout)
        .expect("id")
        .trim()
        .to_string()
}

#[test]
fn help_with_stdout_closed_lands_on_stderr() {
    let tmp = Tmp::new("help-closed1");
    let out = run_closed(&["--help"], &tmp.0, true, false);
    assert_eq!(out.code, 0);
    assert_eq!(out.stderr, onrecord::helptext::TOP_HELP.as_bytes());
    assert!(out.stdout.is_empty());
}

#[test]
fn add_with_stdout_closed_exits_1_and_the_append_lands() {
    let tmp = Tmp::new("add-closed1");
    let out = run_closed(ADD, &tmp.0, true, false);
    assert_eq!(out.code, 1);
    assert!(out.stderr.is_empty());
    let landed = std::fs::read(tmp.path("r.jsonl")).expect("the append landed");
    let other = Tmp::new("add-closed1-twin");
    let twin = run(ADD, &other.0);
    assert_eq!(twin.code, 0);
    assert_eq!(landed, std::fs::read(other.path("r.jsonl")).expect("twin"));
}

#[test]
fn get_with_stdout_closed_exits_1_silent() {
    let tmp = Tmp::new("get-closed1");
    let id = seed(&tmp);
    let out = run_closed(&["get", "r.jsonl", &id], &tmp.0, true, false);
    assert_eq!(out.code, 1);
    assert!(out.stderr.is_empty());
}

#[test]
fn check_with_stdout_closed_agrees_in_exit_only() {
    let tmp = Tmp::new("check-closed1");
    seed(&tmp);
    let sound = run_closed(&["check", "r.jsonl"], &tmp.0, true, false);
    assert_eq!(sound.code, 0);
    let mut defect = std::fs::read(tmp.path("r.jsonl")).expect("seed");
    defect.extend_from_slice(b"{\"bogus\": true}\n");
    std::fs::write(tmp.path("defect.jsonl"), defect).expect("defect");
    let out = run_closed(&["check", "defect.jsonl"], &tmp.0, true, false);
    assert_eq!(out.code, 1);
    assert!(out.stderr.is_empty());
}

#[test]
fn refusal_with_stderr_closed_lands_on_stdout() {
    let tmp = Tmp::new("refusal-closed2");
    seed(&tmp);
    let out = run_closed(&["get", "r.jsonl", "claim-nope"], &tmp.0, false, true);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, b"error, no claim with id claim-nope\n");
}

#[test]
fn usage_with_stderr_closed_lands_on_stdout_without_the_error_line() {
    let tmp = Tmp::new("usage-closed2");
    let out = run_closed(&[], &tmp.0, false, true);
    assert_eq!(out.code, 2);
    assert_eq!(out.stdout, onrecord::helptext::TOP_USAGE.as_bytes());
}

#[test]
fn both_closed_keeps_the_exits() {
    let tmp = Tmp::new("closed12");
    seed(&tmp);
    let missing = run_closed(&["get", "r.jsonl", "claim-nope"], &tmp.0, true, true);
    assert_eq!(missing.code, 1);
    let help = run_closed(&["--help"], &tmp.0, true, true);
    assert_eq!(help.code, 0);
    let usage = run_closed(&[], &tmp.0, true, true);
    assert_eq!(usage.code, 2);
}

#[test]
fn help_at_dev_full_exits_120_with_the_dump() {
    let tmp = Tmp::new("help-full1");
    let out = run_full(&["--help"], &tmp.0, true);
    assert_eq!(out.code, 120);
    assert_eq!(out.stderr, DUMP_28.as_bytes());
}

#[test]
fn check_defect_at_dev_full_exits_120_with_the_dump() {
    let tmp = Tmp::new("check-full1");
    seed(&tmp);
    let mut defect = std::fs::read(tmp.path("r.jsonl")).expect("seed");
    defect.extend_from_slice(b"{\"bogus\": true}\n");
    std::fs::write(tmp.path("defect.jsonl"), defect).expect("defect");
    let out = run_full(&["check", "defect.jsonl"], &tmp.0, true);
    assert_eq!(out.code, 120);
    assert_eq!(out.stderr, DUMP_28.as_bytes());
}

#[test]
fn refusal_with_stderr_at_dev_full_exits_120() {
    let tmp = Tmp::new("refusal-full2");
    seed(&tmp);
    let out = run_full(&["get", "r.jsonl", "claim-nope"], &tmp.0, false);
    assert_eq!(out.code, 120);
    assert!(out.stdout.is_empty());
}

#[test]
fn emit_with_stderr_closed_and_stdout_full_exits_1() {
    // The nested posture on the emit path: the emit
    // fails, the refusal reroutes into the stdout
    // buffer, and _emit_text's sink replacement
    // discards it, so the exit stays the refusal's 1
    // and the shutdown has nothing left to fail.
    let tmp = Tmp::new("nested-emit");
    let id = seed(&tmp);
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let output = Command::new("/bin/sh")
        .args([
            "-c",
            "exec 2>&-; exec \"$@\"",
            "sh",
            BIN,
            "get",
            "r.jsonl",
            &id,
        ])
        .current_dir(&tmp.0)
        .stdout(Stdio::from(full))
        .output()
        .expect("run the binary");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn refusal_with_stderr_closed_and_stdout_full_exits_120() {
    // The nested posture: the refusal reroutes into
    // the buffered stdout, whose terminal flush
    // fails, so the exit is the shutdown's 120.
    let tmp = Tmp::new("nested");
    seed(&tmp);
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let output = Command::new("/bin/sh")
        .args([
            "-c",
            "exec 2>&-; exec \"$@\"",
            "sh",
            BIN,
            "get",
            "r.jsonl",
            "claim-nope",
        ])
        .current_dir(&tmp.0)
        .stdout(Stdio::from(full))
        .output()
        .expect("run the binary");
    assert_eq!(output.status.code(), Some(120));
    let _ = std::io::stdout().flush();
}

#[test]
fn check_defect_echo_with_a_surrogate_id_exits_1_after_the_renderable_defects() {
    // A hand-written record whose claim id carries a
    // lone surrogate (a JSON escape in the line) makes
    // check's duplicate defect echo the id itself. The
    // interpreter's print of that defect raises
    // UnicodeEncodeError uncaught after the renderable
    // defects have landed; the port answers the crash
    // class with its silent exit 1. The view render
    // failure a surrogate was first aimed at is
    // unreachable through the CLI: add refuses such a
    // claim at its canonical render, and a stored one
    // makes the record defective at load, so no view
    // ever carries one.
    let tmp = Tmp::new("check-surrogate-id");
    let line: &[u8] = b"{\"asserter\": \"agent-a\", \"asserted_at\": \"2026-01-01T09:00:00Z\", \"evidence\": [{\"reference\": \"seed\", \"source\": \"note\"}], \"id\": \"claim-\\ud800x\", \"supersedes\": null, \"text\": \"clean text\"}\n";
    let mut seeded = line.to_vec();
    seeded.extend_from_slice(line);
    std::fs::write(tmp.path("dup.jsonl"), seeded).expect("seed the record");
    let out = run(&["check", "dup.jsonl"], &tmp.0);
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stdout,
        b"line 1: id does not match its derivation\nline 2: id does not match its derivation\n"
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn boundary_at_dev_full() {
    // A record of K bogus lines renders six defect
    // lines each; the totals cross the 8192-byte
    // buffer capacity between K=44 (8,174 bytes,
    // terminal flush only, exit 120 with the dump)
    // and K=45 (8,361 bytes, the mid-run flush fails
    // on the first defect write of line 45, exit 1
    // with the failure named on stderr). The
    // readerless-pipe twin is harness section 17's.
    for (k, want) in [(44usize, 120), (45, 1)] {
        let tmp = Tmp::new("boundary");
        seed(&tmp);
        let mut record = std::fs::read(tmp.path("r.jsonl")).expect("seed");
        for _ in 0..k {
            record.extend_from_slice(b"{\"bogus\": true}\n");
        }
        std::fs::write(tmp.path("many.jsonl"), &record).expect("many");
        let out = run_full(&["check", "many.jsonl"], &tmp.0, true);
        assert_eq!(out.code, want, "full1 K={k}");
        if k == 44 {
            assert_eq!(out.stderr, DUMP_28.as_bytes());
        } else {
            assert_eq!(out.stderr, b"error, [Errno 28] No space left on device\n");
        }
    }
}

#[test]
fn boundary_at_exact_capacity() {
    // The buffer law at its exact point, measured
    // against the oracle at /dev/full: a check whose
    // defect output totals 8,191 bytes defers its
    // failure to the shutdown flush (exit 120, the
    // dump on stderr), while exactly 8,192 bytes
    // flushes at fullness inside the run (exit 1,
    // the failure named on stderr). The construction
    // is the measured one: 43 bogus lines and a
    // duplicated claim whose id length tunes the
    // duplicate defect's echo to the byte.
    for (idlen, want) in [(87usize, 120), (88, 1)] {
        let tmp = Tmp::new("exact-capacity");
        let id = format!("claim-{}", "x".repeat(idlen));
        let mut line = String::from(
            "{\"asserter\": \"agent-a\", \"asserted_at\": \
             \"2026-01-01T09:00:00Z\", \"evidence\": [{\"reference\": \
             \"seed\", \"source\": \"note\"}], \"id\": \"",
        );
        line.push_str(&id);
        line.push_str("\", \"supersedes\": null, \"text\": \"clean text\"}\n");
        let mut record = Vec::new();
        for _ in 0..43 {
            record.extend_from_slice(b"{\"bogus\": true}\n");
        }
        record.extend_from_slice(line.as_bytes());
        record.extend_from_slice(line.as_bytes());
        std::fs::write(tmp.path("many.jsonl"), &record).expect("many");
        let out = run_full(&["check", "many.jsonl"], &tmp.0, true);
        assert_eq!(out.code, want, "exact capacity idlen={idlen}");
        if want == 120 {
            assert_eq!(out.stderr, DUMP_28.as_bytes());
        } else {
            assert_eq!(out.stderr, b"error, [Errno 28] No space left on device\n");
        }
    }
}

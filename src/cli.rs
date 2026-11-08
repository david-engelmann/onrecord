//! OR-11, OR-13. A thin command line over the store:
//! add, get, subject, check, due. Failures are named
//! errors with exit 1. The parse reproduces the
//! Python's argparse behavior for this grammar byte
//! for byte: its help texts, its error shapes, and
//! the order they fire in. Help fires at the end of
//! the token that carries it; a scan error (a missing
//! option value, an ignored explicit argument, an
//! ambiguous long option) fires when its token is
//! processed and outranks a pending help; extras are
//! only reported after the required check. Argument
//! tokens are carried as text in the operating
//! system's own decoding (fsdecode), so argument
//! bytes survive as the Python's do.

use std::io::Write;
use std::os::unix::io::FromRawFd;

use flighttape::value::{Text, Value};

use crate::emit::{dumps_view, Mode};
use crate::helptext::*;
use crate::pyrepr::repr_text;
use crate::store::{AddClaim, RecPath, Store, StoreError};
use crate::util::{fsdecode_units, stderr_render, Msg};

/// Whether a standard descriptor was closed at launch,
/// from the facts the binary's pre-init constructor
/// recorded before the runtime sanitized anything
/// (see main.rs): the Python in the same posture has
/// the corresponding sys.stdout or sys.stderr set to
/// None, which is the state the stream model below
/// reproduces. No post-init fact separates the
/// sanitized descriptor from an explicit /dev/null
/// redirect: the runtime opens the null device O_RDWR
/// with no CLOEXEC, byte-identical in every observable
/// to a shell's `<> /dev/null` (measured under strace
/// on this runtime), so the pre-init record is the
/// only exact witness.
fn fd_closed_at_launch(fd: i32) -> bool {
    crate::launch_fd_closed(fd)
}

/// Write bytes to a raw standard descriptor, ignoring
/// the descriptor's ownership.
fn raw_write(fd: i32, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(fd) });
    file.write_all(bytes)
}

/// The interpreter's buffered writer capacity: a
/// channel accumulates this many bytes before a write
/// forces a flush.
const BUFFER_CAPACITY: usize = 8192;

/// One emulated TextIOWrapper channel: an 8192-byte
/// buffer over a real standard descriptor. A write
/// that would overflow the buffer flushes it first; a
/// single write at or past the capacity flushes and
/// then writes directly. A failed flush discards the
/// buffered bytes, as the interpreter's does, and the
/// error surfaces to the writer class that catches it.
struct Channel {
    buffer: Vec<u8>,
    /// A channel replaced by the null sink, the shape
    /// _emit_text leaves behind: writes vanish and
    /// flushes succeed, so nothing the channel held
    /// can surface at the shutdown.
    sink: bool,
}

impl Channel {
    fn new() -> Channel {
        Channel {
            buffer: Vec::new(),
            sink: false,
        }
    }

    fn write(&mut self, fd: i32, bytes: &[u8]) -> std::io::Result<()> {
        if self.sink {
            return Ok(());
        }
        if self.buffer.len() + bytes.len() > BUFFER_CAPACITY {
            self.flush(fd)?;
            if bytes.len() >= BUFFER_CAPACITY {
                return raw_write(fd, bytes);
            }
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn flush(&mut self, fd: i32) -> std::io::Result<()> {
        if self.sink || self.buffer.is_empty() {
            return Ok(());
        }
        let result = raw_write(fd, &self.buffer);
        self.buffer.clear();
        result
    }
}

/// The two standard channels with their launch state.
/// A None channel is the Python's None stream: writes
/// routed to it follow the interpreter's routing rules
/// per writer class, spelled out on each method.
struct Streams {
    out: Option<Channel>,
    err: Option<Channel>,
}

impl Streams {
    fn new() -> Streams {
        Streams {
            out: if fd_closed_at_launch(1) {
                None
            } else {
                Some(Channel::new())
            },
            err: if fd_closed_at_launch(2) {
                None
            } else {
                Some(Channel::new())
            },
        }
    }

    fn stdout_closed(&self) -> bool {
        self.out.is_none()
    }

    /// Replace the stdout channel with the null sink,
    /// discarding whatever it held: _emit_text's own
    /// move after a failed emit, so the broken stream
    /// cannot mask the refusal at the shutdown.
    fn sink_stdout(&mut self) {
        self.out = Some(Channel {
            buffer: Vec::new(),
            sink: true,
        });
    }

    /// print() to stdout: a None stdout swallows the
    /// write silently; the exit stands.
    fn print_out(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        match &mut self.out {
            None => Ok(()),
            Some(channel) => channel.write(1, bytes),
        }
    }

    /// print(..., file=sys.stderr), and print_usage's
    /// chain with it: a None stderr reroutes the bytes
    /// to stdout, buffered there like any print.
    fn print_err(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if self.err.is_some() {
            let channel = self.err.as_mut().expect("checked");
            channel.write(2, bytes)
        } else {
            self.print_out(bytes)
        }
    }

    /// argparse's exit message through _print_message:
    /// addressed to sys.stderr with no reroute; a None
    /// stderr swallows it whole.
    fn message_err(&mut self, bytes: &[u8]) {
        if let Some(channel) = &mut self.err {
            let _ = channel.write(2, bytes);
        }
    }

    /// argparse's help chain (print_help then
    /// _print_message): addressed to stdout, falling
    /// back to stderr when stdout is None, and to
    /// nothing when both are.
    fn print_help_text(&mut self, bytes: &[u8]) {
        if self.out.is_some() {
            let channel = self.out.as_mut().expect("checked");
            let _ = channel.write(1, bytes);
        } else if let Some(channel) = &mut self.err {
            let _ = channel.write(2, bytes);
        }
    }

    /// The interpreter's shutdown: flush stdout, then
    /// stderr. A failed stdout flush writes the
    /// ignored-exception dump to the second
    /// descriptor's raw fd and the exit becomes 120;
    /// a failed stderr flush makes the exit 120 with
    /// nothing observable.
    fn shutdown(mut self, code: i32) -> i32 {
        let mut result = code;
        if let Some(channel) = &mut self.out {
            if let Err(error) = channel.flush(1) {
                let dump = format!(
                    "Exception ignored in: <_io.TextIOWrapper name='<stdout>' \
                     mode='w' encoding='utf-8'>\n{}: {}\n",
                    os_error_class(&error),
                    os_error_text(&error)
                );
                let _ = raw_write(2, dump.as_bytes());
                result = 120;
            }
        }
        if let Some(channel) = &mut self.err {
            if channel.flush(2).is_err() {
                result = 120;
            }
        }
        result
    }
}

/// The exception class name the interpreter's shutdown
/// dump prints for an OS error: the OSError subclass
/// CPython selects by errno, OSError itself when no
/// subclass applies.
fn os_error_class(error: &std::io::Error) -> &'static str {
    match error.raw_os_error() {
        Some(2) => "FileNotFoundError",
        Some(3) => "ProcessLookupError",
        Some(4) => "InterruptedError",
        Some(10) => "ChildProcessError",
        Some(11) => "BlockingIOError",
        Some(1) | Some(13) => "PermissionError",
        Some(17) => "FileExistsError",
        Some(20) => "NotADirectoryError",
        Some(21) => "IsADirectoryError",
        Some(32) => "BrokenPipeError",
        Some(103) => "ConnectionAbortedError",
        Some(104) => "ConnectionResetError",
        Some(110) => "TimeoutError",
        _ => "OSError",
    }
}

/// One argument token: its text, and its decoded
/// scalars for the parser's character work.
struct Token {
    text: Text,
    scalars: Vec<u32>,
}

impl Token {
    fn from_units(units: Vec<u16>) -> Token {
        let mut scalars = Vec::new();
        let mut index = 0;
        while index < units.len() {
            let unit = units[index];
            if (0xD800..=0xDBFF).contains(&unit)
                && index + 1 < units.len()
                && (0xDC00..=0xDFFF).contains(&units[index + 1])
            {
                scalars.push(crate::util::decode_pair(unit, units[index + 1]));
                index += 2;
            } else {
                scalars.push(unit as u32);
                index += 1;
            }
        }
        Token {
            text: Text::from_units(units),
            scalars,
        }
    }

    fn rendered(&self) -> String {
        stderr_render(self.text.units())
    }
}

/// Whether a token is shaped like an option: it starts
/// with a dash, is not a bare dash, and does not look
/// like a negative number (no option in this grammar
/// does, so argparse lets those through as values).
fn option_shaped(token: &Token) -> bool {
    let scalars = &token.scalars;
    if scalars.len() < 2 || scalars[0] != '-' as u32 {
        return false;
    }
    !is_negative_number(&scalars[1..])
}

/// argparse's negative number matcher, the full token
/// pattern ^-\d+$|^-\d*\.\d+$ with \d a Unicode
/// decimal digit: a token matching it whole is a
/// value, never an option.
fn is_negative_number(rest: &[u32]) -> bool {
    let digit = |c: &u32| crate::util::decimal_digit(*c).is_some();
    if !rest.is_empty() && rest.iter().all(digit) {
        return true;
    }
    match rest.iter().position(|c| *c == '.' as u32) {
        Some(dot) => {
            let (before, after) = (&rest[..dot], &rest[dot + 1..]);
            !after.is_empty() && before.iter().all(digit) && after.iter().all(digit)
        }
        None => false,
    }
}

fn scalars_of(text: &str) -> Vec<u32> {
    text.chars().map(|ch| ch as u32).collect()
}

/// Split a long-option token at its first '=': the
/// name's scalars, and the glued value's text when an
/// '=' is present.
fn split_long(token: &Token) -> (&[u32], Option<Text>) {
    match token.scalars.iter().position(|c| *c == '=' as u32) {
        Some(at) => {
            let units = token.text.units();
            let mut unit_at = 0usize;
            let mut scalar_at = 0usize;
            while scalar_at < at {
                if (0xD800..=0xDBFF).contains(&units[unit_at])
                    && unit_at + 1 < units.len()
                    && (0xDC00..=0xDFFF).contains(&units[unit_at + 1])
                {
                    unit_at += 2;
                } else {
                    unit_at += 1;
                }
                scalar_at += 1;
            }
            (
                &token.scalars[..at],
                Some(Text::from_units(units[unit_at + 1..].to_vec())),
            )
        }
        None => (&token.scalars[..], None),
    }
}

/// The text of a token from a scalar index on, for
/// the value glued behind an '=' in a short cluster.
fn text_from_scalar(token: &Token, scalar_index: usize) -> Text {
    let units = token.text.units();
    let mut unit_at = 0usize;
    let mut scalar_at = 0usize;
    while scalar_at < scalar_index && unit_at < units.len() {
        if (0xD800..=0xDBFF).contains(&units[unit_at])
            && unit_at + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[unit_at + 1])
        {
            unit_at += 2;
        } else {
            unit_at += 1;
        }
        scalar_at += 1;
    }
    Text::from_units(units[unit_at.min(units.len())..].to_vec())
}

/// The outcome of matching a long-option name against
/// a level's option set.
enum LongMatch {
    Exact(&'static str),
    Unique(&'static str),
    Ambiguous(Vec<&'static str>),
    None,
}

fn match_long(name: &[u32], options: &[&'static str]) -> LongMatch {
    let mut candidates: Vec<&'static str> = Vec::new();
    for option in options {
        let full = scalars_of(option);
        if full == name {
            return LongMatch::Exact(option);
        }
        if full.len() > name.len() && full[..name.len()] == *name {
            candidates.push(option);
        }
    }
    match candidates.len() {
        0 => LongMatch::None,
        1 => LongMatch::Unique(candidates[0]),
        _ => LongMatch::Ambiguous(candidates),
    }
}

enum ParseOutcome {
    Run(&'static str, Box<SubParsed>),
    Exit(i32),
}

fn parse_error(streams: &mut Streams, usage: &str, prog: &str, message: &str) -> ParseOutcome {
    // The Python's argparse error report is two writes
    // with different routing: the usage block goes
    // through print_usage, which reroutes to stdout
    // when sys.stderr is None, and the error line goes
    // through _print_message, which a None stderr
    // swallows whole. On an open stderr the two land
    // in order in its buffer, byte-identical to the
    // single report the port used to print.
    let _ = streams.print_err(usage.as_bytes());
    streams.message_err(format!("{prog}: error: {message}\n").as_bytes());
    ParseOutcome::Exit(2)
}

/// Print a help text the argparse way: the text is
/// addressed to stdout through the buffered channel,
/// falling back to stderr when stdout was closed at
/// launch. A broken stream surfaces only at the
/// shutdown flush, as exit 120 with the
/// ignored-exception dump, never as a caught refusal.
fn print_help(streams: &mut Streams, text: &str) -> ParseOutcome {
    streams.print_help_text(text.as_bytes());
    ParseOutcome::Exit(0)
}

struct SubParsed {
    positionals: Vec<Text>,
    values: Vec<(&'static str, Text)>,
}

impl SubParsed {
    fn get_text(&self, canonical: &str) -> Option<Text> {
        self.values
            .iter()
            .rev()
            .find_map(|(name, value)| (*name == canonical).then(|| value.clone()))
    }

    fn get_all(&self, canonical: &str) -> Vec<Text> {
        self.values
            .iter()
            .filter(|(name, _)| *name == canonical)
            .map(|(_, value)| value.clone())
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OptKind {
    Help,
    Value,
    Append,
}

struct OptSpec {
    canonical: &'static str,
    label: &'static str,
    kind: OptKind,
}

struct SubSpec {
    usage: &'static str,
    prog: &'static str,
    help: &'static str,
    positionals: &'static [&'static str],
    options: &'static [OptSpec],
    /// The required arguments in definition order:
    /// (label, is_positional).
    required: &'static [(&'static str, bool)],
}

const HELP_OPT: OptSpec = OptSpec {
    canonical: "--help",
    label: "-h/--help",
    kind: OptKind::Help,
};

const fn value_opt(canonical: &'static str) -> OptSpec {
    OptSpec {
        canonical,
        label: canonical,
        kind: OptKind::Value,
    }
}

const ADD_SPEC: SubSpec = SubSpec {
    usage: ADD_USAGE,
    prog: "onrecord add",
    help: ADD_HELP,
    positionals: &["record"],
    options: &[
        HELP_OPT,
        value_opt("--text"),
        value_opt("--asserter"),
        value_opt("--at"),
        OptSpec {
            canonical: "--evidence",
            label: "--evidence",
            kind: OptKind::Append,
        },
        value_opt("--supersedes"),
        value_opt("--subject"),
        value_opt("--due"),
    ],
    required: &[
        ("record", true),
        ("--text", false),
        ("--asserter", false),
        ("--at", false),
    ],
};

const GET_SPEC: SubSpec = SubSpec {
    usage: GET_USAGE,
    prog: "onrecord get",
    help: GET_HELP,
    positionals: &["record", "id"],
    options: &[HELP_OPT],
    required: &[("record", true), ("id", true)],
};

const SUBJECT_SPEC: SubSpec = SubSpec {
    usage: SUBJECT_USAGE,
    prog: "onrecord subject",
    help: SUBJECT_HELP,
    positionals: &["record", "key"],
    options: &[HELP_OPT],
    required: &[("record", true), ("key", true)],
};

const DUE_SPEC: SubSpec = SubSpec {
    usage: DUE_USAGE,
    prog: "onrecord due",
    help: DUE_HELP,
    positionals: &["record", "date"],
    options: &[HELP_OPT],
    required: &[("record", true), ("date", true)],
};

const CHECK_SPEC: SubSpec = SubSpec {
    usage: CHECK_USAGE,
    prog: "onrecord check",
    help: CHECK_HELP,
    positionals: &["record"],
    options: &[HELP_OPT],
    required: &[("record", true)],
};

fn spec_for(command: &str) -> &'static SubSpec {
    match command {
        "add" => &ADD_SPEC,
        "get" => &GET_SPEC,
        "subject" => &SUBJECT_SPEC,
        "due" => &DUE_SPEC,
        _ => &CHECK_SPEC,
    }
}

/// Take the value for an option: a glued form wins,
/// else the next token, which an option-shaped token
/// can never be. On success `index` still points at
/// the option token; the caller advances past both.
fn take_value(
    streams: &mut Streams,
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    glued: Option<Text>,
    opt: &OptSpec,
) -> Result<Text, ParseOutcome> {
    if let Some(value) = glued {
        return Ok(value);
    }
    match tokens.get(*index + 1) {
        Some(next) if !option_shaped(next) => {
            *index += 1;
            Ok(next.text.clone())
        }
        _ => Err(parse_error(
            streams,
            spec.usage,
            spec.prog,
            &format!("argument {}: expected one argument", opt.label),
        )),
    }
}

/// Parse one subcommand's tokens.
/// A short cluster token's outcome: the token
/// becomes an extra, it is consumed silently, or it
/// ends the parse.
enum ClusterFlow {
    Extra,
    Skip,
    Exit(ParseOutcome),
}

/// Read a short cluster: only h is known here. A
/// first letter the command does not know makes the
/// whole token an extra; help fires when the token
/// ends, unless a scan error in it fired first.
fn short_cluster(streams: &mut Streams, spec: &SubSpec, token: &Token) -> ClusterFlow {
    let cluster = &token.scalars[1..];
    if cluster[0] != 'h' as u32 {
        return ClusterFlow::Extra;
    }
    let mut help_pending = false;
    let mut at = 0usize;
    while at < cluster.len() {
        if cluster[at] == 'h' as u32 {
            if cluster.get(at + 1) == Some(&('=' as u32)) {
                // A flag that takes no value, given
                // one: argparse refuses the explicit
                // argument.
                let value = text_from_scalar(token, at + 3);
                return ClusterFlow::Exit(parse_error(
                    streams,
                    spec.usage,
                    spec.prog,
                    &format!(
                        "argument -h/--help: ignored explicit argument {}",
                        repr_text(&value)
                    ),
                ));
            }
            help_pending = true;
            at += 1;
        } else {
            break;
        }
    }
    if help_pending {
        return ClusterFlow::Exit(print_help(streams, spec.help));
    }
    ClusterFlow::Skip
}

fn parse_sub(
    streams: &mut Streams,
    command: &'static str,
    tokens: &[Token],
    top_extras: &[Text],
) -> ParseOutcome {
    let spec = spec_for(command);
    let candidates: Vec<&'static str> = spec.options.iter().map(|opt| opt.canonical).collect();
    let mut positionals: Vec<Text> = Vec::new();
    let mut extras: Vec<Text> = top_extras.to_vec();
    let mut values: Vec<(&'static str, Text)> = Vec::new();
    let mut separated = false;
    let mut index = 0usize;

    fn push_positional(
        token: &Text,
        spec: &SubSpec,
        positionals: &mut Vec<Text>,
        extras: &mut Vec<Text>,
    ) {
        if positionals.len() < spec.positionals.len() {
            positionals.push(token.clone());
        } else {
            extras.push(token.clone());
        }
    }

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(&token.text, spec, &mut positionals, &mut extras);
            index += 1;
            continue;
        }
        if token.scalars == scalars_of("--") {
            // The first "--" always ends option parsing,
            // but when every positional is already
            // filled argparse keeps the "--" itself
            // among the unrecognized arguments.
            separated = true;
            if positionals.len() == spec.positionals.len() {
                extras.push(token.text.clone());
            }
            index += 1;
            continue;
        }
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
        {
            let (name, glued) = split_long(token);
            let chosen = match match_long(name, &candidates) {
                LongMatch::Exact(option) | LongMatch::Unique(option) => option,
                LongMatch::Ambiguous(matches) => {
                    return parse_error(
                        streams,
                        spec.usage,
                        spec.prog,
                        &format!(
                            "ambiguous option: {} could match {}",
                            token.rendered(),
                            matches.join(", ")
                        ),
                    );
                }
                LongMatch::None => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            };
            let opt = spec
                .options
                .iter()
                .find(|opt| opt.canonical == chosen)
                .expect("a spec option");
            if opt.kind == OptKind::Help {
                if let Some(value) = glued {
                    return parse_error(
                        streams,
                        spec.usage,
                        spec.prog,
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                return print_help(streams, spec.help);
            }
            match take_value(streams, spec, tokens, &mut index, glued, opt) {
                Ok(value) => values.push((opt.canonical, value)),
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            match short_cluster(streams, spec, token) {
                ClusterFlow::Extra => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
                ClusterFlow::Skip => {
                    index += 1;
                    continue;
                }
                ClusterFlow::Exit(outcome) => return outcome,
            }
        }
        push_positional(&token.text, spec, &mut positionals, &mut extras);
        index += 1;
    }

    let mut missing: Vec<&str> = Vec::new();
    for (label, is_positional) in spec.required {
        if *is_positional {
            let at = spec
                .positionals
                .iter()
                .position(|name| name == label)
                .expect("a spec positional");
            if positionals.len() <= at {
                missing.push(label);
            }
        } else if !values.iter().any(|(name, _)| name == label) {
            missing.push(label);
        }
    }
    if !missing.is_empty() {
        return parse_error(
            streams,
            spec.usage,
            spec.prog,
            &format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        );
    }
    if !extras.is_empty() {
        let rendered: Vec<String> = extras.iter().map(|t| stderr_render(t.units())).collect();
        return parse_error(
            streams,
            TOP_USAGE,
            "onrecord",
            &format!("unrecognized arguments: {}", rendered.join(" ")),
        );
    }
    ParseOutcome::Run(
        command,
        Box::new(SubParsed {
            positionals,
            values,
        }),
    )
}

const COMMANDS: [&str; 5] = ["add", "get", "subject", "due", "check"];

/// Parse the top level: help, the command word, and
/// the held extras an unknown token becomes.
fn parse_top(streams: &mut Streams, tokens: &[Token]) -> ParseOutcome {
    let mut extras: Vec<Text> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
            && token.scalars != scalars_of("--")
        {
            let (name, glued) = split_long(token);
            match match_long(name, &["--help"]) {
                LongMatch::Exact("--help") | LongMatch::Unique("--help") => {
                    if let Some(value) = glued {
                        return parse_error(
                            streams,
                            TOP_USAGE,
                            "onrecord",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    return print_help(streams, TOP_HELP);
                }
                _ => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            }
        }
        if token.scalars == scalars_of("--") || !option_shaped(token) {
            if token.scalars == scalars_of("--") && index == tokens.len() - 1 {
                // A separator with nothing after it:
                // argparse has consumed it, and the
                // command is simply missing.
                break;
            }
            for command in COMMANDS {
                if token.text.units() == Text::from_str(command).units() {
                    let command: &'static str = command;
                    return parse_sub(streams, command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                streams,
                TOP_USAGE,
                "onrecord",
                &format!(
                    "argument command: invalid choice: {} (choose from 'add', 'get', 'subject', 'due', 'check')",
                    repr_text(&token.text)
                ),
            );
        }
        if token.scalars[1] == 'h' as u32 {
            return print_help(streams, TOP_HELP);
        }
        extras.push(token.text.clone());
        index += 1;
    }
    parse_error(
        streams,
        TOP_USAGE,
        "onrecord",
        "the following arguments are required: command",
    )
}

/// The named refusal for a stdout the emit cannot
/// use, printed the way the Python's except arm
/// prints it: through stderr's channel, rerouting
/// to stdout when stderr was closed at launch. The
/// Python then replaces its stdout with a sink, so
/// whatever the rerouted refusal joined in the
/// stdout buffer is discarded with it.
fn cannot_write(streams: &mut Streams) -> i32 {
    let _ = streams.print_err(b"error, cannot write to stdout\n");
    // The Python then closes the old stdout before
    // replacing it: the close flushes whatever the
    // wrapper holds, so a refusal rerouted into the
    // stdout buffer (stderr closed at launch, stdout
    // itself working, the render-failure cell) lands
    // before the sink replaces the channel. A failed
    // close-flush discards, as the channel's flush
    // already does.
    if let Some(channel) = &mut streams.out {
        let _ = channel.flush(1);
    }
    streams.sink_stdout();
    1
}

/// Write text to stdout the way the Python's
/// _emit_text does: an explicit write and flush, a
/// failure the named refusal "error, cannot write to
/// stdout" and exit 1. When stdout was closed at
/// launch the Python's write meets a None stream and
/// the failure is its uncaught crash class instead: a
/// bare exit 1 after the operation, the append for
/// add already landed, with no refusal printed.
fn emit_text(streams: &mut Streams, bytes: &[u8]) -> i32 {
    if streams.stdout_closed() {
        return 1;
    }
    let step = {
        let channel = streams.out.as_mut().expect("stdout open");
        channel.write(1, bytes).and_then(|_| channel.flush(1))
    };
    match step {
        Ok(()) => 0,
        Err(_) => cannot_write(streams),
    }
}

/// Emit a view document: json.dumps at an indent of
/// two plus a newline, through emit_text. A document
/// the stream cannot take (a lone surrogate outside
/// the escape band) fails the same way.
fn emit_document(streams: &mut Streams, document: &Value) -> i32 {
    match dumps_view(document, Mode::Stdout) {
        Ok(bytes) => emit_text(streams, &bytes),
        Err(_) => {
            if streams.stdout_closed() {
                1
            } else {
                cannot_write(streams)
            }
        }
    }
}

/// Report a store failure the way the Python's main
/// does: a named refusal after "error, " printed to
/// stderr's channel, and the crash class as a bare
/// exit 1.
fn report(streams: &mut Streams, result: Result<i32, StoreError>) -> i32 {
    match result {
        Ok(code) => code,
        Err(StoreError::Named(error)) => {
            let _ = streams.print_err(format!("error, {}\n", error.render_stderr()).as_bytes());
            1
        }
        Err(StoreError::Crash) => 1,
    }
}

fn text_arg(parsed: &SubParsed, canonical: &str) -> Text {
    parsed.get_text(canonical).expect("a required argument")
}

fn parse_evidence(items: &[Text]) -> Result<Value, StoreError> {
    let mut evidence = Vec::new();
    for item in items {
        let units = item.units();
        let colon = units.iter().position(|unit| *unit == b':' as u16);
        let (source, reference) = match colon {
            Some(at) if at > 0 && at + 1 < units.len() => (
                Text::from_units(units[..at].to_vec()),
                Text::from_units(units[at + 1..].to_vec()),
            ),
            _ => {
                let mut message = Msg::str("evidence must name a source and a reference, ");
                message.push_text(item);
                return Err(StoreError::Named(message));
            }
        };
        evidence.push(Value::Object(vec![
            (Text::from_str("source"), Value::Str(source)),
            (Text::from_str("reference"), Value::Str(reference)),
        ]));
    }
    Ok(Value::Array(evidence))
}

fn execute(streams: &mut Streams, command: &str, parsed: &SubParsed) -> i32 {
    let store = Store::new(RecPath::from_units(parsed.positionals[0].units()));
    match command {
        "add" => report(streams, (|| {
            let evidence = parse_evidence(&parsed.get_all("--evidence"))?;
            let claim_id = store.add(AddClaim {
                text: text_arg(parsed, "--text"),
                asserter: text_arg(parsed, "--asserter"),
                asserted_at: text_arg(parsed, "--at"),
                evidence,
                supersedes: parsed.get_text("--supersedes"),
                subject: parsed.get_text("--subject"),
                due: parsed.get_text("--due"),
                extra: Vec::new(),
            })?;
            let mut bytes = claim_id.into_bytes();
            bytes.push(b'\n');
            Ok(emit_text(streams, &bytes))
        })()),
        "get" => report(streams, (|| {
            let document = store.get(&parsed.positionals[1])?;
            Ok(emit_document(streams, &document))
        })()),
        "subject" => report(streams, (|| {
            let document = store.subject(&parsed.positionals[1])?;
            Ok(emit_document(streams, &document))
        })()),
        "due" => report(streams, (|| {
            let document = store.due(&parsed.positionals[1])?;
            Ok(emit_document(streams, &document))
        })()),
        _ => {
            // check: defects print to stdout through
            // its buffered channel, one per line. A
            // flush failure partway is the Python's
            // caught OSError, reported with its OS
            // text and exit 1; a failure of the final
            // flush is the shutdown's, exit 120 with
            // the dump, decided in Streams::shutdown.
            let defects = match store.check() {
                Ok(defects) => defects,
                Err(error) => return report(streams, Err(error)),
            };
            for defect in &defects {
                let bytes = match defect.render_stdout() {
                    Some(bytes) => bytes,
                    None => return 1,
                };
                // print() writes the text and its
                // newline as two buffer writes; the
                // pair is the flush boundary's unit.
                let written = streams
                    .print_out(&bytes)
                    .and_then(|_| streams.print_out(b"\n"));
                if let Err(error) = written {
                    let _ = streams
                        .print_err(format!("error, {}\n", os_error_text(&error)).as_bytes());
                    return 1;
                }
            }
            if defects.is_empty() {
                let _ = streams.print_out(b"the record is sound\n");
                0
            } else {
                1
            }
        }
    }
}

/// An operating system error's CPython rendering,
/// "[Errno N] message", for the CLI's stream
/// failures; the strerror table itself lives in
/// util, shared with the store's phrases.
fn os_error_text(err: &std::io::Error) -> String {
    match err.raw_os_error() {
        Some(errno) => {
            let bare = err.to_string();
            let text = crate::util::strerror_text(err);
            if text == bare {
                // Outside the table, the library's own
                // rendering already carries the errno.
                bare
            } else {
                format!("[Errno {errno}] {text}")
            }
        }
        None => err.to_string(),
    }
}

pub fn run_cli() -> i32 {
    let mut streams = Streams::new();
    let tokens: Vec<Token> = std::env::args_os()
        .skip(1)
        .map(|arg| {
            #[cfg(unix)]
            let bytes = arg.as_encoded_bytes().to_vec();
            #[cfg(not(unix))]
            let bytes = arg.to_string_lossy().into_owned().into_bytes();
            Token::from_units(fsdecode_units(&bytes))
        })
        .collect();
    let code = match parse_top(&mut streams, &tokens) {
        ParseOutcome::Run(command, parsed) => execute(&mut streams, command, &parsed),
        ParseOutcome::Exit(code) => code,
    };
    streams.shutdown(code)
}

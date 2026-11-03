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

use flighttape::value::{Text, Value};

use crate::emit::{dumps_view, Mode};
use crate::helptext::*;
use crate::pyrepr::repr_text;
use crate::store::{AddClaim, RecPath, Store, StoreError};
use crate::util::{fsdecode_units, stderr_render, Msg};

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

fn parse_error(usage: &str, prog: &str, message: &str) -> ParseOutcome {
    // The usage constants carry their trailing
    // newline; the Python prints the error line
    // directly under the usage block.
    eprintln!("{usage}{prog}: error: {message}");
    ParseOutcome::Exit(2)
}

/// Print a help text the guarded way: the Python's
/// argparse prints help to a buffered stdout whose
/// failure surfaces only at interpreter shutdown
/// (exit 120 with an ignored exception dump), so a
/// broken stream here is the port's named shape for
/// that class, a caught write error with exit 1,
/// never a panic.
fn print_help(text: &str) -> ParseOutcome {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    match handle
        .write_all(text.as_bytes())
        .and_then(|_| handle.flush())
    {
        Ok(()) => ParseOutcome::Exit(0),
        Err(error) => {
            eprintln!("error, {}", os_error_text(&error));
            ParseOutcome::Exit(1)
        }
    }
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
fn short_cluster(spec: &SubSpec, token: &Token) -> ClusterFlow {
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
        return ClusterFlow::Exit(print_help(spec.help));
    }
    ClusterFlow::Skip
}

fn parse_sub(command: &'static str, tokens: &[Token], top_extras: &[Text]) -> ParseOutcome {
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
                        spec.usage,
                        spec.prog,
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                return print_help(spec.help);
            }
            match take_value(spec, tokens, &mut index, glued, opt) {
                Ok(value) => values.push((opt.canonical, value)),
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            match short_cluster(spec, token) {
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
fn parse_top(tokens: &[Token]) -> ParseOutcome {
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
                            TOP_USAGE,
                            "onrecord",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    return print_help(TOP_HELP);
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
                    return parse_sub(command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                TOP_USAGE,
                "onrecord",
                &format!(
                    "argument command: invalid choice: {} (choose from 'add', 'get', 'subject', 'due', 'check')",
                    repr_text(&token.text)
                ),
            );
        }
        if token.scalars[1] == 'h' as u32 {
            return print_help(TOP_HELP);
        }
        extras.push(token.text.clone());
        index += 1;
    }
    parse_error(
        TOP_USAGE,
        "onrecord",
        "the following arguments are required: command",
    )
}

/// Write text to stdout the way the Python's
/// _emit_text does: a failure is the named refusal
/// "error, cannot write to stdout" and exit 1.
fn emit_text(bytes: &[u8]) -> i32 {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    match handle.write_all(bytes).and_then(|_| handle.flush()) {
        Ok(()) => 0,
        Err(_) => {
            eprintln!("error, cannot write to stdout");
            1
        }
    }
}

/// Emit a view document: json.dumps at an indent of
/// two plus a newline, through emit_text. A document
/// the stream cannot take (a lone surrogate outside
/// the escape band) fails the same way.
fn emit_document(document: &Value) -> i32 {
    match dumps_view(document, Mode::Stdout) {
        Ok(bytes) => emit_text(&bytes),
        Err(_) => {
            eprintln!("error, cannot write to stdout");
            1
        }
    }
}

/// Report a store failure the way the Python's main
/// does: a named refusal after "error, ", and the
/// crash class as a bare exit 1.
fn report(result: Result<i32, StoreError>) -> i32 {
    match result {
        Ok(code) => code,
        Err(StoreError::Named(error)) => {
            eprintln!("error, {}", error.render_stderr());
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

fn execute(command: &str, parsed: &SubParsed) -> i32 {
    let store = Store::new(RecPath::from_units(parsed.positionals[0].units()));
    match command {
        "add" => report((|| {
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
            Ok(emit_text(&bytes))
        })()),
        "get" => report((|| {
            let document = store.get(&parsed.positionals[1])?;
            Ok(emit_document(&document))
        })()),
        "subject" => report((|| {
            let document = store.subject(&parsed.positionals[1])?;
            Ok(emit_document(&document))
        })()),
        "due" => report((|| {
            let document = store.due(&parsed.positionals[1])?;
            Ok(emit_document(&document))
        })()),
        _ => {
            // check: defects print to stdout, one per
            // line; a stream failure writing them is
            // the Python's caught OSError, reported
            // with its OS text.
            let defects = match store.check() {
                Ok(defects) => defects,
                Err(error) => return report(Err(error)),
            };
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            for defect in &defects {
                let mut bytes = match defect.render_stdout() {
                    Some(bytes) => bytes,
                    None => return 1,
                };
                bytes.push(b'\n');
                if let Err(err) = handle.write_all(&bytes) {
                    eprintln!("error, {}", os_error_text(&err));
                    return 1;
                }
            }
            if defects.is_empty() {
                match handle
                    .write_all(b"the record is sound\n")
                    .and_then(|_| handle.flush())
                {
                    Ok(()) => 0,
                    Err(err) => {
                        eprintln!("error, {}", os_error_text(&err));
                        1
                    }
                }
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
    match parse_top(&tokens) {
        ParseOutcome::Run(command, parsed) => execute(command, &parsed),
        ParseOutcome::Exit(code) => code,
    }
}

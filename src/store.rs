//! OR-5 to OR-16. The append-only claim store,
//! mirroring the Python store of Rounds 1 and 2
//! phrase for phrase.
//!
//! Three derive sites exist in the Python, each
//! with the interpreter stack of its own call chain,
//! and each was probed for its render budget (see
//! the digest module): claim_defects at read time
//! (990, 988), claim_defects inside add (993, 991),
//! and add's own derive_id (994, 992). A line's
//! parse has its own pair of caps, (9994 total,
//! 9991 to the deepest object), probed through the
//! shipped CLI: the loader's duplicate key hook
//! spends stack as each object closes.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use flighttape::value::{Text, Value};

use crate::digest::{self, Budget};
use crate::emit::{self, Mode};
use crate::util::{self, Msg};

pub const BUDGET_READ: Budget = (990, 988);
pub const BUDGET_ADD_CHECK: Budget = (993, 991);
pub const BUDGET_ADD_DERIVE: Budget = (994, 992);
const LINE_DEPTH_CAP: usize = 9994;
const LINE_OBJECT_DEPTH_CAP: usize = 9991;

const REQUIRED: [&str; 6] = [
    "id",
    "text",
    "asserter",
    "asserted_at",
    "evidence",
    "supersedes",
];

/// A named refusal, or the crash class: the Python
/// dies with an uncaught exception and the port
/// answers with a silent exit 1 (named in the
/// dossier).
#[derive(Debug)]
pub enum StoreError {
    Named(Msg),
    Crash,
}

impl StoreError {
    pub fn named(text: &str) -> StoreError {
        StoreError::Named(Msg::str(text))
    }

    pub fn text(&self) -> String {
        match self {
            StoreError::Named(msg) => msg.render_stderr(),
            StoreError::Crash => String::new(),
        }
    }

    pub fn is_crash(&self) -> bool {
        matches!(self, StoreError::Crash)
    }
}

fn named(message: Msg) -> StoreError {
    StoreError::Named(message)
}

fn msg_with(prefix: &str, text: &Text, suffix: &str) -> Msg {
    let mut out = Msg::str(prefix);
    out.push_text(text);
    out.push_str(suffix);
    out
}

/// The bare strerror text for an OS error, as the
/// Python's OSError.strerror reads (the append
/// phrase interpolates it). Errnos outside the
/// table fall back to the standard library's
/// rendering, a named deviation.
fn strerror_text(err: &std::io::Error) -> String {
    match err.raw_os_error() {
        Some(errno) => {
            let text = match errno {
                1 => "Operation not permitted",
                2 => "No such file or directory",
                13 => "Permission denied",
                20 => "Not a directory",
                21 => "Is a directory",
                24 => "Too many open files",
                28 => "No space left on device",
                30 => "Read-only file system",
                32 => "Broken pipe",
                36 => "File name too long",
                40 => "Too many levels of symbolic links",
                _ => "",
            };
            if text.is_empty() {
                err.to_string()
            } else {
                text.to_string()
            }
        }
        None => err.to_string(),
    }
}

/// The record's path: the OS spelling the syscalls
/// take, and the display spelling the phrases print
/// (the Python's pathlib rendering of the argument).
#[derive(Clone)]
pub struct RecPath {
    pub os: PathBuf,
    pub display: Msg,
}

impl RecPath {
    pub fn from_units(units: &[u16]) -> RecPath {
        let normalized = util::normalize_path_display(units);
        RecPath {
            os: util::os_path_from_units(&normalized),
            display: Msg(normalized),
        }
    }

    pub fn new(text: &str) -> RecPath {
        RecPath::from_units(&util::fsdecode_units(text.as_bytes()))
    }
}

pub fn field<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    if let Value::Object(entries) = value {
        entries
            .iter()
            .find(|(key, _)| text_is(key, name))
            .map(|(_, item)| item)
    } else {
        None
    }
}

pub fn text_is(text: &Text, name: &str) -> bool {
    text.units() == Text::from_str(name).units()
}

fn as_text(value: &Value) -> Option<&Text> {
    if let Value::Str(text) = value {
        Some(text)
    } else {
        None
    }
}

fn non_empty_text(value: &Value) -> bool {
    matches!(value, Value::Str(text) if !text.units().is_empty())
}

/// The id of a claim without its id, together with
/// its position (OR-6a), under the budget of the
/// calling site.
pub fn derive_id(
    claim: &Value,
    position: usize,
    budget: Budget,
) -> Result<String, digest::DigestError> {
    let payload = Value::Object(vec![
        (Text::from_str("claim"), claim.clone()),
        (Text::from_str("position"), Value::Int(position.to_string())),
    ]);
    let hex = digest::digest_of(&payload, budget)?;
    Ok(format!("claim-{}", &hex[..16]))
}

fn is_iso_timestamp(value: &Value) -> bool {
    match as_text(value) {
        Some(text) => match text.as_str() {
            Some(plain) => flighttape::timeparse::parse_timestamp(&plain).is_some(),
            None => false,
        },
        None => false,
    }
}

/// OR-13a: exactly YYYY-MM-DD and a real calendar
/// day. The Python's digit test accepts every
/// Unicode decimal digit, but its calendar parse
/// then refuses the non-ASCII ones (probed), so the
/// net form is ASCII digits only.
pub fn is_iso_date_text(text: &Text) -> bool {
    let units = text.units();
    if units.len() != 10 {
        return false;
    }
    let digit = |unit: u16| -> Option<u32> {
        if (b'0' as u16..=b'9' as u16).contains(&unit) {
            Some((unit - b'0' as u16) as u32)
        } else {
            None
        }
    };
    if units[4] != b'-' as u16 || units[7] != b'-' as u16 {
        return false;
    }
    let mut digits = [0u32; 8];
    for (slot, index) in [0usize, 1, 2, 3, 5, 6, 8, 9].iter().enumerate() {
        match digit(units[*index]) {
            Some(d) => digits[slot] = d,
            None => return false,
        }
    }
    let year = digits[0] * 1000 + digits[1] * 100 + digits[2] * 10 + digits[3];
    let month = digits[4] * 10 + digits[5];
    let day = digits[6] * 10 + digits[7];
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    (1..=days[(month - 1) as usize]).contains(&day)
}

fn is_iso_date(value: &Value) -> bool {
    match as_text(value) {
        Some(text) => is_iso_date_text(text),
        None => false,
    }
}

/// Shape defects of one parsed line (OR-5, OR-5a,
/// OR-11a). Every defect is named; one bad field
/// never hides another.
pub fn claim_defects(claim: &Value, line_no: usize, position: usize, budget: Budget) -> Vec<Msg> {
    let where_ = format!("line {line_no}");
    let mut defects: Vec<Msg> = Vec::new();
    if !matches!(claim, Value::Object(_)) {
        return vec![Msg::str(&format!("{where_}: not a claim object"))];
    }
    for name in REQUIRED {
        if field(claim, name).is_none() {
            defects.push(Msg::str(&format!("{where_}: missing field {name}")));
        }
    }
    if let Some(id) = field(claim, "id") {
        if !non_empty_text(id) {
            defects.push(Msg::str(&format!(
                "{where_}: id must be a non-empty string"
            )));
        }
    }
    if let Some(text) = field(claim, "text") {
        if !non_empty_text(text) {
            defects.push(Msg::str(&format!(
                "{where_}: text must be a non-empty string"
            )));
        }
    }
    if let Some(asserter) = field(claim, "asserter") {
        if !non_empty_text(asserter) {
            defects.push(Msg::str(&format!(
                "{where_}: asserter must be a non-empty string"
            )));
        }
    }
    if let Some(asserted_at) = field(claim, "asserted_at") {
        if !is_iso_timestamp(asserted_at) {
            defects.push(Msg::str(&format!(
                "{where_}: asserted_at is not an ISO timestamp"
            )));
        }
    }
    if let Some(evidence) = field(claim, "evidence") {
        match evidence {
            Value::Array(items) => {
                if items.is_empty() {
                    defects.push(Msg::str(&format!(
                        "{where_}: evidence must hold at least one entry"
                    )));
                } else {
                    for (index, entry) in items.iter().enumerate() {
                        let sound = matches!(entry, Value::Object(_))
                            && field(entry, "source").map(non_empty_text).unwrap_or(false)
                            && field(entry, "reference")
                                .map(non_empty_text)
                                .unwrap_or(false);
                        if !sound {
                            defects.push(Msg::str(&format!(
                                "{where_}: evidence[{index}] must name a source and a reference"
                            )));
                        }
                    }
                }
            }
            _ => defects.push(Msg::str(&format!("{where_}: evidence must be a list"))),
        }
    }
    if let Some(supersedes) = field(claim, "supersedes") {
        if !matches!(supersedes, Value::Null) && as_text(supersedes).is_none() {
            defects.push(Msg::str(&format!(
                "{where_}: supersedes must be an id or null"
            )));
        }
    }
    if let Some(subject) = field(claim, "subject") {
        if !matches!(subject, Value::Null) && as_text(subject).is_none() {
            defects.push(Msg::str(&format!("{where_}: subject must be a string")));
        }
    }
    if let Some(due) = field(claim, "due") {
        if !matches!(due, Value::Null) && !is_iso_date(due) {
            defects.push(Msg::str(&format!("{where_}: due is not an ISO date")));
        }
    }
    if REQUIRED.iter().all(|name| field(claim, name).is_some())
        && matches!(field(claim, "id"), Some(Value::Str(_)))
    {
        let without = Value::Object(match claim {
            Value::Object(entries) => entries
                .iter()
                .filter(|(key, _)| !text_is(key, "id"))
                .cloned()
                .collect(),
            _ => Vec::new(),
        });
        match derive_id(&without, position, budget) {
            Err(_) => defects.push(Msg::str(&format!(
                "{where_}: the claim cannot be rendered under canonical form 1, \
                 so its id cannot be verified"
            ))),
            Ok(expected) => {
                if let Some(Value::Str(id)) = field(claim, "id") {
                    if id.as_str().as_deref() != Some(expected.as_str()) {
                        defects.push(Msg::str(&format!(
                            "{where_}: id does not match its derivation"
                        )));
                    }
                }
            }
        }
    }
    defects
}

/// The duplicate key a line's parse reports, if
/// any. The Python's object pairs hook raises the
/// repeated key as the object holding it closes, so
/// the scan mirrors close order: a nested object's
/// duplicate fires inside its value, ahead of a
/// pending duplicate in any enclosing object, and a
/// structural error met before the close hides the
/// duplicate behind the parse failure, exactly as
/// the interpreter's does.
enum Scan {
    Clean,
    Duplicate(Text),
    Bad,
}

fn first_duplicate_key(text: &str) -> Option<Text> {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0usize;
    match scan_value(&chars, &mut pos) {
        Scan::Duplicate(key) => Some(key),
        _ => None,
    }
}

fn scan_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && matches!(chars[*pos], ' ' | '\t' | '\n' | '\r') {
        *pos += 1;
    }
}

fn scan_string(chars: &[char], pos: &mut usize) -> Option<Text> {
    // chars[*pos] is the opening quote; returns the
    // decoded key text, or None if the string does
    // not close.
    *pos += 1;
    let mut units: Vec<u16> = Vec::new();
    while *pos < chars.len() {
        let ch = chars[*pos];
        match ch {
            '"' => {
                *pos += 1;
                return Some(Text::from_units(units));
            }
            '\\' => {
                *pos += 1;
                if *pos >= chars.len() {
                    return None;
                }
                let esc = chars[*pos];
                *pos += 1;
                match esc {
                    'u' => {
                        let mut value: u32 = 0;
                        for _ in 0..4 {
                            if *pos >= chars.len() {
                                return None;
                            }
                            value = value * 16 + chars[*pos].to_digit(16)?;
                            *pos += 1;
                        }
                        units.push(value as u16);
                    }
                    'n' => units.push(b'\n' as u16),
                    't' => units.push(b'\t' as u16),
                    'r' => units.push(b'\r' as u16),
                    'b' => units.push(8),
                    'f' => units.push(12),
                    _ => {
                        let mut buf = [0u16; 2];
                        units.extend_from_slice(esc.encode_utf16(&mut buf));
                    }
                }
            }
            _ => {
                let mut buf = [0u16; 2];
                units.extend_from_slice(ch.encode_utf16(&mut buf));
                *pos += 1;
            }
        }
    }
    None
}

fn scan_value(chars: &[char], pos: &mut usize) -> Scan {
    scan_ws(chars, pos);
    if *pos >= chars.len() {
        return Scan::Bad;
    }
    match chars[*pos] {
        '{' => {
            *pos += 1;
            let mut seen: Vec<Text> = Vec::new();
            let mut pending: Option<Text> = None;
            scan_ws(chars, pos);
            if *pos < chars.len() && chars[*pos] == '}' {
                *pos += 1;
                return Scan::Clean;
            }
            loop {
                scan_ws(chars, pos);
                if *pos >= chars.len() || chars[*pos] != '"' {
                    return Scan::Bad;
                }
                let key = match scan_string(chars, pos) {
                    Some(key) => key,
                    None => return Scan::Bad,
                };
                scan_ws(chars, pos);
                if *pos >= chars.len() || chars[*pos] != ':' {
                    return Scan::Bad;
                }
                *pos += 1;
                match scan_value(chars, pos) {
                    Scan::Clean => {}
                    other => return other,
                }
                if seen.iter().any(|held: &Text| held.units() == key.units()) {
                    if pending.is_none() {
                        pending = Some(key);
                    }
                } else {
                    seen.push(key);
                }
                scan_ws(chars, pos);
                if *pos >= chars.len() {
                    return Scan::Bad;
                }
                if chars[*pos] == ',' {
                    *pos += 1;
                    continue;
                }
                if chars[*pos] == '}' {
                    *pos += 1;
                    return match pending {
                        Some(key) => Scan::Duplicate(key),
                        None => Scan::Clean,
                    };
                }
                return Scan::Bad;
            }
        }
        '[' => {
            *pos += 1;
            scan_ws(chars, pos);
            if *pos < chars.len() && chars[*pos] == ']' {
                *pos += 1;
                return Scan::Clean;
            }
            loop {
                match scan_value(chars, pos) {
                    Scan::Clean => {}
                    other => return other,
                }
                scan_ws(chars, pos);
                if *pos >= chars.len() {
                    return Scan::Bad;
                }
                if chars[*pos] == ',' {
                    *pos += 1;
                    continue;
                }
                if chars[*pos] == ']' {
                    *pos += 1;
                    return Scan::Clean;
                }
                return Scan::Bad;
            }
        }
        '"' => match scan_string(chars, pos) {
            Some(_) => Scan::Clean,
            None => Scan::Bad,
        },
        _ => {
            // A literal or number: skip its token. The
            // scan only arbitrates duplicate against
            // error; token validity is the reader's.
            let start = *pos;
            while *pos < chars.len()
                && !matches!(chars[*pos], ',' | ']' | '}' | ' ' | '\t' | '\n' | '\r')
            {
                *pos += 1;
            }
            if *pos == start {
                Scan::Bad
            } else {
                Scan::Clean
            }
        }
    }
}

/// The claims superseding one target, in record
/// order: (line number, the superseding claim's id).
type ForkSources = Vec<(usize, Option<Text>)>;

fn is_blank_line(line: &str) -> bool {
    line.chars()
        .all(|ch| ch.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&ch))
}

/// Return (claims, defects): every claim in order
/// and every defect in the record, line numbers
/// attached.
pub fn load_record(path: &RecPath) -> (Vec<Value>, Vec<Msg>) {
    let raw = match std::fs::read(&path.os) {
        Ok(raw) => raw,
        Err(_) => {
            let mut message = Msg::str("cannot read the record, ");
            message.push_msg(&path.display);
            return (Vec::new(), vec![message]);
        }
    };
    if raw.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let body = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw[..]
    };
    let text = match String::from_utf8(body.to_vec()) {
        Ok(text) => text,
        Err(_) => return (Vec::new(), vec![Msg::str("the record is not valid UTF-8")]),
    };
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut defects: Vec<Msg> = Vec::new();
    let mut parsed: Vec<Option<(usize, Value)>> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;
        if is_blank_line(line) {
            defects.push(Msg::str(&format!("line {line_no}: blank line")));
            parsed.push(None);
            continue;
        }
        let (max_depth, max_object) = util::json_text_depths(line);
        if max_depth > LINE_DEPTH_CAP || max_object > LINE_OBJECT_DEPTH_CAP {
            defects.push(Msg::str(&format!(
                "line {line_no}: nesting too deep to read"
            )));
            parsed.push(None);
            continue;
        }
        match flighttape::json::parse(line) {
            Ok(claim) => {
                if let Some(dup) = first_duplicate_key(line) {
                    defects.push(msg_with(
                        &format!("line {line_no}: duplicate key "),
                        &dup,
                        "",
                    ));
                    parsed.push(None);
                    continue;
                }
                defects.extend(claim_defects(&claim, line_no, index, BUDGET_READ));
                parsed.push(Some((line_no, claim)));
            }
            Err(_) => {
                if let Some(dup) = first_duplicate_key(line) {
                    defects.push(msg_with(
                        &format!("line {line_no}: duplicate key "),
                        &dup,
                        "",
                    ));
                } else {
                    defects.push(Msg::str(&format!("line {line_no}: not JSON")));
                }
                parsed.push(None);
            }
        }
    }
    let numbered: Vec<(usize, &Value)> = parsed
        .iter()
        .filter_map(|entry| entry.as_ref())
        .filter(|(_, claim)| matches!(claim, Value::Object(_)))
        .map(|(line_no, claim)| (*line_no, claim))
        .collect();
    let claims: Vec<Value> = numbered.iter().map(|(_, claim)| (*claim).clone()).collect();
    let mut line_of: HashMap<Vec<u16>, usize> = HashMap::new();
    for (line_no, claim) in &numbered {
        if let Some(Value::Str(cid)) = field(claim, "id") {
            if line_of.contains_key(cid.units()) {
                defects.push(msg_with(
                    &format!("line {line_no}: id "),
                    cid,
                    " appears twice",
                ));
            }
            line_of.insert(cid.units().to_vec(), *line_no);
        }
    }
    let mut superseded_by: Vec<(Text, ForkSources)> = Vec::new();
    for (line_no, claim) in &numbered {
        if let Some(Value::Str(target)) = field(claim, "supersedes") {
            if !line_of.contains_key(target.units()) {
                defects.push(msg_with(
                    &format!("line {line_no}: supersedes names no claim in the record, "),
                    target,
                    "",
                ));
            }
            let source = (
                *line_no,
                as_text(field(claim, "id").unwrap_or(&Value::Null)).cloned(),
            );
            match superseded_by
                .iter_mut()
                .find(|(held, _)| held.units() == target.units())
            {
                Some((_, sources)) => sources.push(source),
                None => superseded_by.push((target.clone(), vec![source])),
            }
        }
    }
    for (target, sources) in &superseded_by {
        if sources.len() > 1 {
            let target_line = line_of
                .get(target.units())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string());
            let mut message = Msg::str(&format!("line {}: claim ", sources[1].0));
            message.push_text(target);
            message.push_str(&format!(
                " (line {target_line}) is superseded by two claims, "
            ));
            match &sources[0].1 {
                Some(id) => message.push_text(id),
                None => message.push_str("None"),
            }
            message.push_str(" and ");
            match &sources[1].1 {
                Some(id) => message.push_text(id),
                None => message.push_str("None"),
            }
            message.push_str(", a fork");
            defects.push(message);
        }
    }
    (claims, defects)
}

fn require_sound(defects: &[Msg]) -> Result<(), StoreError> {
    if defects.is_empty() {
        Ok(())
    } else {
        let mut message = Msg::str("the record has defects: ");
        message.push_msg(&util::join_msgs(defects, "; "));
        Err(named(message))
    }
}

/// OR-8b: the subjects a live head answers for: its
/// own subject, or every subject along its chain
/// when it carries none.
fn answers(claim: &Value, chain: &[Value]) -> Vec<Text> {
    if let Some(Value::Str(own)) = field(claim, "subject") {
        return vec![own.clone()];
    }
    let mut out: Vec<Text> = Vec::new();
    for link in chain {
        if let Some(Value::Str(subject)) = field(link, "subject") {
            if !out.iter().any(|held| held.units() == subject.units()) {
                out.push(subject.clone());
            }
        }
    }
    out
}

fn build_chain(claims: &[Value], claim: &Value) -> Result<Vec<Value>, StoreError> {
    let mut by_id: HashMap<Vec<u16>, &Value> = HashMap::new();
    for item in claims {
        if let Some(Value::Str(cid)) = field(item, "id") {
            by_id.insert(cid.units().to_vec(), item);
        }
    }
    let mut chain = vec![claim.clone()];
    let mut seen: HashSet<Vec<u16>> = HashSet::new();
    if let Some(Value::Str(cid)) = field(claim, "id") {
        seen.insert(cid.units().to_vec());
    }
    let mut current = claim;
    while let Some(Value::Str(target)) = field(current, "supersedes") {
        let parent = by_id.get(target.units()).ok_or(StoreError::Crash)?;
        if let Some(Value::Str(pid)) = field(parent, "id") {
            if !seen.insert(pid.units().to_vec()) {
                break;
            }
        }
        chain.push((*parent).clone());
        current = parent;
    }
    chain.reverse();
    Ok(chain)
}

fn retrieval(claim: &Value, chain: Vec<Value>) -> Value {
    Value::Object(vec![
        (Text::from_str("claim"), claim.clone()),
        (Text::from_str("chain"), Value::Array(chain)),
    ])
}

fn superseded_set(claims: &[Value]) -> HashSet<Vec<u16>> {
    let mut set = HashSet::new();
    for claim in claims {
        if let Some(Value::Str(target)) = field(claim, "supersedes") {
            set.insert(target.units().to_vec());
        }
    }
    set
}

fn is_superseded(set: &HashSet<Vec<u16>>, claim: &Value) -> bool {
    match field(claim, "id") {
        Some(Value::Str(id)) => set.contains(id.units()),
        _ => false,
    }
}

pub struct AddClaim {
    pub text: Text,
    pub asserter: Text,
    pub asserted_at: Text,
    pub evidence: Value,
    pub supersedes: Option<Text>,
    pub subject: Option<Text>,
    pub due: Option<Text>,
    pub extra: Vec<(Text, Value)>,
}

pub struct Store {
    pub path: RecPath,
}

impl Store {
    pub fn new(path: RecPath) -> Store {
        Store { path }
    }

    fn exists(&self) -> bool {
        std::fs::metadata(&self.path.os).is_ok()
    }

    fn load(&self) -> Result<Vec<Value>, StoreError> {
        if !self.exists() {
            return Ok(Vec::new());
        }
        let (claims, defects) = load_record(&self.path);
        require_sound(&defects)?;
        Ok(claims)
    }

    pub fn add(&self, request: AddClaim) -> Result<String, StoreError> {
        let claims = self.load()?;
        if let Some(supersedes) = &request.supersedes {
            let known = claims.iter().any(|claim| {
                matches!(field(claim, "id"), Some(Value::Str(id)) if id.units() == supersedes.units())
            });
            if !known {
                return Err(named(msg_with(
                    "supersedes names no claim in the record, ",
                    supersedes,
                    "",
                )));
            }
            let already = claims.iter().any(|claim| {
                matches!(field(claim, "supersedes"), Some(Value::Str(id)) if id.units() == supersedes.units())
            });
            if already {
                return Err(named(msg_with(
                    "claim ",
                    supersedes,
                    " has already been superseded",
                )));
            }
        }
        let mut entries: Vec<(Text, Value)> = vec![
            (Text::from_str("text"), Value::Str(request.text.clone())),
            (
                Text::from_str("asserter"),
                Value::Str(request.asserter.clone()),
            ),
            (
                Text::from_str("asserted_at"),
                Value::Str(request.asserted_at.clone()),
            ),
            (Text::from_str("evidence"), request.evidence.clone()),
            (
                Text::from_str("supersedes"),
                match &request.supersedes {
                    Some(id) => Value::Str(id.clone()),
                    None => Value::Null,
                },
            ),
        ];
        if let Some(subject) = &request.subject {
            entries.push((Text::from_str("subject"), Value::Str(subject.clone())));
        }
        if let Some(due) = &request.due {
            entries.push((Text::from_str("due"), Value::Str(due.clone())));
        }
        for (key, value) in &request.extra {
            if text_is(key, "id") || entries.iter().any(|(held, _)| held.units() == key.units()) {
                return Err(named(msg_with(
                    "extra field ",
                    key,
                    " collides with a claim field",
                )));
            }
            entries.push((key.clone(), value.clone()));
        }
        let claim = Value::Object(entries);
        // Position is the count of lines in the record;
        // a sound record holds one claim per line, so
        // the loaded claims are that count (the
        // invariant OR-6a stands on).
        let position = claims.len();
        let claim_id = derive_id(&claim, position, BUDGET_ADD_DERIVE).map_err(|_| {
            StoreError::named("the claim cannot be rendered under canonical form 1")
        })?;
        let mut stored_entries =
            vec![(Text::from_str("id"), Value::Str(Text::from_str(&claim_id)))];
        if let Value::Object(entries) = &claim {
            stored_entries.extend(entries.iter().cloned());
        }
        let stored = Value::Object(stored_entries);
        let defects = claim_defects(&stored, position + 1, position, BUDGET_ADD_CHECK);
        if let Some(first) = defects.into_iter().next() {
            return Err(StoreError::Named(strip_line_prefix(&first)));
        }
        self.append(&stored)?;
        Ok(claim_id)
    }

    fn append(&self, stored: &Value) -> Result<(), StoreError> {
        let mut prefix: &[u8] = b"";
        let opened = (|| -> Result<std::fs::File, std::io::Error> {
            if self.exists() {
                let metadata = std::fs::metadata(&self.path.os)?;
                if metadata.len() > 0 {
                    let mut handle = std::fs::File::open(&self.path.os)?;
                    handle.seek(SeekFrom::End(-1))?;
                    let mut tail = [0u8; 1];
                    handle.read_exact(&mut tail)?;
                    if tail != *b"\n" {
                        prefix = b"\n";
                    }
                }
            }
            std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.path.os)
        })();
        let mut handle = match opened {
            Ok(handle) => handle,
            Err(error) => return Err(self.append_error(&error)),
        };
        // The Python renders its line before opening
        // the file but writes through the text layer,
        // so a line holding a lone surrogate fails at
        // the write, after the open has created the
        // file, and nothing lands. The port renders at
        // this point, under the strict mode.
        let mut line = match emit::dumps_line(stored, Mode::Strict) {
            Ok(line) => line,
            Err(_) => return Err(StoreError::Crash),
        };
        line.push(b'\n');
        let written = (|| -> Result<(), std::io::Error> {
            handle.write_all(prefix)?;
            handle.write_all(&line)?;
            handle.flush()
        })();
        match written {
            Ok(()) => Ok(()),
            Err(error) => Err(self.append_error(&error)),
        }
    }

    fn append_error(&self, error: &std::io::Error) -> StoreError {
        let mut message = Msg::str("cannot append to the record, ");
        message.push_msg(&self.path.display);
        message.push_str(": ");
        message.push_str(&strerror_text(error));
        named(message)
    }

    pub fn get(&self, claim_id: &Text) -> Result<Value, StoreError> {
        let claims = self.load()?;
        let claim = claims
            .iter()
            .find(|claim| {
                matches!(field(claim, "id"), Some(Value::Str(id)) if id.units() == claim_id.units())
            })
            .ok_or_else(|| named(msg_with("no claim with id ", claim_id, "")))?;
        let chain = build_chain(&claims, claim)?;
        Ok(retrieval(claim, chain))
    }

    pub fn subject(&self, key: &Text) -> Result<Value, StoreError> {
        let claims = self.load()?;
        let superseded = superseded_set(&claims);
        let mut live: Vec<(&Value, Vec<Value>)> = Vec::new();
        for claim in &claims {
            if is_superseded(&superseded, claim) {
                continue;
            }
            let chain = build_chain(&claims, claim)?;
            if answers(claim, &chain)
                .iter()
                .any(|s| s.units() == key.units())
            {
                live.push((claim, chain));
            }
        }
        match live.len() {
            0 => Err(named(msg_with("no live claim on subject ", key, ""))),
            1 => {
                let (claim, chain) = live.into_iter().next().expect("one live claim");
                Ok(retrieval(claim, chain))
            }
            _ => Ok(Value::Object(vec![
                (Text::from_str("conflict"), Value::Bool(true)),
                (Text::from_str("subject"), Value::Str(key.clone())),
                (
                    Text::from_str("claims"),
                    Value::Array(
                        live.into_iter()
                            .map(|(claim, chain)| retrieval(claim, chain))
                            .collect(),
                    ),
                ),
            ])),
        }
    }

    pub fn due(&self, as_of: &Text) -> Result<Value, StoreError> {
        if !is_iso_date_text(as_of) {
            return Err(named(msg_with(
                "the supplied date is not an ISO date, ",
                as_of,
                "",
            )));
        }
        // OR-13b, without a race: the load itself
        // reports an unreadable record as a defect, so
        // a record missing at load time refuses here
        // by name instead of answering empty.
        let (claims, defects) = load_record(&self.path);
        require_sound(&defects)?;
        let superseded = superseded_set(&claims);
        let mut due: Vec<(&Text, usize, &Value)> = Vec::new();
        for (position, claim) in claims.iter().enumerate() {
            if is_superseded(&superseded, claim) {
                continue;
            }
            if let Some(Value::Str(due_date)) = field(claim, "due") {
                if due_date.units() <= as_of.units() {
                    due.push((due_date, position, claim));
                }
            }
        }
        due.sort_by(|a, b| a.0.units().cmp(b.0.units()).then(a.1.cmp(&b.1)));
        let mut documents = Vec::new();
        for (_, _, claim) in &due {
            let chain = build_chain(&claims, claim)?;
            documents.push(retrieval(claim, chain));
        }
        let mut counts: Vec<(Text, usize)> = Vec::new();
        for document in &documents {
            if let Some(claim) = field(document, "claim") {
                if let Some(Value::Array(chain)) = field(document, "chain") {
                    for subject in answers(claim, chain) {
                        match counts
                            .iter_mut()
                            .find(|(held, _)| held.units() == subject.units())
                        {
                            Some((_, count)) => *count += 1,
                            None => counts.push((subject, 1)),
                        }
                    }
                }
            }
        }
        let mut conflicts: Vec<Text> = counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(subject, _)| subject)
            .collect();
        conflicts.sort_by(|a, b| a.units().cmp(b.units()));
        let mut document = vec![
            (Text::from_str("as_of"), Value::Str(as_of.clone())),
            (Text::from_str("due"), Value::Array(documents)),
        ];
        if !conflicts.is_empty() {
            document.push((
                Text::from_str("conflicts"),
                Value::Array(conflicts.into_iter().map(Value::Str).collect()),
            ));
        }
        Ok(Value::Object(document))
    }

    pub fn check(&self) -> Result<Vec<Msg>, StoreError> {
        if !self.exists() {
            let mut message = Msg::str("no such record, ");
            message.push_msg(&self.path.display);
            return Err(named(message));
        }
        let (_, defects) = load_record(&self.path);
        Ok(defects)
    }
}

fn strip_line_prefix(defect: &Msg) -> Msg {
    // The Python raises the first defect's text past
    // its "line N: " prefix: a split on the first
    // colon space, keeping the rest.
    let units = &defect.0;
    let mut index = 0;
    while index + 1 < units.len() {
        if units[index] == b':' as u16 && units[index + 1] == b' ' as u16 {
            return Msg(units[index + 2..].to_vec());
        }
        index += 1;
    }
    defect.clone()
}

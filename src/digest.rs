//! Canonical form 1 digests (OR-6a), consumed pinned
//! (OR-18). The form is flighttape's: object keys
//! sorted by UTF-16 code units, numbers rendered as
//! ECMAScript renders them, no whitespace. This
//! module renders it from the crate's value model,
//! with one deliberate difference from the crate's
//! own renderer: the float digits come from the
//! Python digest's source (CPython repr digits under
//! the ECMAScript case split), because the crate's
//! renderer runs one digit long on exactly
//! terminating expansions (the 2^-24 family, probed
//! against the oracle for this round) and claim ids
//! are observable behavior. The difference is owed
//! to flighttape's Round 12 dispositions.
//!
//! A value that cannot be rendered is a DigestError:
//! a non-finite number, or text that cannot be
//! encoded (a lone surrogate in a string or a key,
//! which the Python's UTF-16BE sort key refuses the
//! same way). Rendering also refuses past a depth
//! budget, standing in for the interpreter recursion
//! limit the Python hits in its renderer: the budget
//! is a pair (deepest value node, deepest object
//! node, both counted from the rendered root at
//! depth 1), and each call site passes the budget
//! probed for its own stack (the store module owns
//! the three shipped sites).

use flighttape::value::{Text, Value};
use sha2::{Digest, Sha256};

use crate::emit::{escape_string, py_float, Mode};

/// (deepest value node, deepest object node) a
/// render may reach.
pub type Budget = (usize, usize);

#[derive(Debug)]
pub struct DigestError;

fn render_number(value: f64) -> String {
    // The Python digest's renderer: repr gives the
    // shortest digits that round trip; from them,
    // point is the count of digits before the decimal
    // point when the value is written out in decimal,
    // and the four cases are the ECMAScript number to
    // string cases.
    if value == 0.0 {
        return "0".to_string();
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let text = py_float(value.abs());
    let (mantissa, exp_part) = match text.split_once('e') {
        Some((m, e)) => (
            m.to_string(),
            Some(e.parse::<i64>().expect("repr exponent")),
        ),
        None => (text, None),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((w, f)) => (w.to_string(), f.to_string()),
        None => (mantissa, String::new()),
    };
    let raw = format!("{whole}{fraction}");
    let digits_trimmed = raw.trim_start_matches('0');
    let exp10 = match exp_part {
        Some(e) => e,
        None => {
            let leading = (raw.len() - digits_trimmed.len()) as i64;
            whole.len() as i64 - leading - 1
        }
    };
    let mut digits = digits_trimmed.trim_end_matches('0').to_string();
    if digits.is_empty() {
        digits = "0".to_string();
    }
    let point = exp10 + 1;
    let count = digits.len() as i64;
    if count <= point && point <= 21 {
        format!("{sign}{digits}{}", "0".repeat((point - count) as usize))
    } else if 0 < point && point <= 21 {
        let at = point as usize;
        format!("{sign}{}.{}", &digits[..at], &digits[at..])
    } else if -6 < point && point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else {
        let head = if count == 1 {
            digits.clone()
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        format!(
            "{sign}{head}e{}{}",
            if exp10 >= 0 { "+" } else { "-" },
            exp10.abs()
        )
    }
}

struct Renderer<'a> {
    budget: Budget,
    out: &'a mut Vec<u8>,
}

impl Renderer<'_> {
    /// Render one value sitting at node depth
    /// `depth`. The budget's two measures are checked
    /// on entry: every node against the first, every
    /// object against the second.
    fn render(&mut self, value: &Value, depth: usize) -> Result<(), DigestError> {
        if depth > self.budget.0 {
            return Err(DigestError);
        }
        match value {
            Value::Null => self.out.extend_from_slice(b"null"),
            Value::Bool(true) => self.out.extend_from_slice(b"true"),
            Value::Bool(false) => self.out.extend_from_slice(b"false"),
            Value::Str(text) => {
                escape_string(text, Mode::Strict, self.out).map_err(|_| DigestError)?;
            }
            Value::Int(digits) => {
                // The Python renders an int through str,
                // which refuses past the interpreter's
                // digit limit; the crate reader accepts
                // integer literals only to the same
                // limit, so a longer digit string can
                // only arrive built by hand.
                if digits.trim_start_matches('-').len() > 4300 {
                    return Err(DigestError);
                }
                self.out.extend_from_slice(digits.as_bytes());
            }
            Value::Float(number) => {
                if !number.is_finite() {
                    return Err(DigestError);
                }
                self.out
                    .extend_from_slice(render_number(*number).as_bytes());
            }
            Value::Array(items) => {
                self.out.push(b'[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        self.out.push(b',');
                    }
                    self.render(item, depth + 1)?;
                }
                self.out.push(b']');
            }
            Value::Object(entries) => {
                if depth > self.budget.1 {
                    return Err(DigestError);
                }
                // The Python sorts keys by their UTF-16BE
                // encoding; a lone surrogate key fails
                // that encode, which is its cannot be
                // encoded refusal.
                let mut keyed: Vec<(&Text, &Value)> = Vec::with_capacity(entries.len());
                for (key, item) in entries {
                    if !key_is_encodable(key) {
                        return Err(DigestError);
                    }
                    keyed.push((key, item));
                }
                keyed.sort_by(|a, b| a.0.units().cmp(b.0.units()));
                self.out.push(b'{');
                for (index, (key, item)) in keyed.iter().enumerate() {
                    if index > 0 {
                        self.out.push(b',');
                    }
                    escape_string(key, Mode::Strict, self.out).map_err(|_| DigestError)?;
                    self.out.push(b':');
                    self.render(item, depth + 1)?;
                }
                self.out.push(b'}');
            }
        }
        Ok(())
    }
}

fn key_is_encodable(text: &Text) -> bool {
    let units = text.units();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if index + 1 >= units.len() || !(0xDC00..=0xDFFF).contains(&units[index + 1]) {
                return false;
            }
            index += 2;
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            return false;
        } else {
            index += 1;
        }
    }
    true
}

/// The value's canonical form 1 as UTF-8 bytes,
/// under the budget the caller probed for its site.
pub fn canonical_bytes(value: &Value, budget: Budget) -> Result<Vec<u8>, DigestError> {
    let mut out = Vec::new();
    let mut renderer = Renderer {
        budget,
        out: &mut out,
    };
    renderer.render(value, 1)?;
    Ok(out)
}

/// The SHA-256 hex digest of a value under
/// canonical form 1.
pub fn digest_of(value: &Value, budget: Budget) -> Result<String, DigestError> {
    let bytes = canonical_bytes(value, budget)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

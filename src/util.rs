//! Shared constructions: argument and path text in
//! CPython's filesystem decoding, the messages the
//! store's phrases are built from, the strerror
//! table the OS error phrases read, and the Unicode
//! decimal digit table the argument matcher uses.

use flighttape::value::Text;

/// Decode OS bytes the way CPython decodes file system
/// and argument strings: valid UTF-8 decodes normally,
/// and each byte that cannot decode becomes the
/// surrogateescape unit 0xDC00 plus the byte.
pub fn fsdecode_units(bytes: &[u8]) -> Vec<u16> {
    let mut units = Vec::new();
    let mut rest = bytes;
    loop {
        match std::str::from_utf8(rest) {
            Ok(text) => {
                units.extend(text.encode_utf16());
                return units;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                let head = std::str::from_utf8(&rest[..valid]).expect("the valid prefix");
                units.extend(head.encode_utf16());
                match error.error_len() {
                    Some(len) => {
                        for byte in &rest[valid..valid + len] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        rest = &rest[valid + len..];
                    }
                    None => {
                        for byte in &rest[valid..] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        return units;
                    }
                }
            }
        }
    }
}

/// The scalar a surrogate pair encodes.
pub fn decode_pair(high: u16, low: u16) -> u32 {
    0x10000 + (((high as u32) - 0xD800) << 10) + ((low as u32) - 0xDC00)
}

/// The inverse: encode text units back to OS bytes,
/// mapping surrogateescape units back to their bytes.
/// A lone surrogate outside the escape range cannot
/// be encoded; it renders as the UTF-8 replacement,
/// a case argument strings from the OS never reach
/// (their surrogates come only from fsdecode).
#[cfg(unix)]
pub fn fsencode_bytes(units: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
        } else if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(scalar)
                    .expect("a valid pair")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.extend_from_slice("\u{FFFD}".as_bytes());
            index += 1;
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(unit as u32)
                    .expect("a BMP scalar")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 1;
        }
    }
    out
}

/// Render text for a stderr message the way CPython's
/// stderr does (its error handler is backslashreplace):
/// valid text passes through, and every lone surrogate
/// becomes its \uXXXX escape.
pub fn stderr_render(units: &[u16]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            out.push(char::from_u32(scalar).expect("a valid pair"));
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.push_str(&format!("\\u{unit:04x}"));
            index += 1;
        } else {
            out.push(char::from_u32(unit as u32).expect("a BMP scalar"));
            index += 1;
        }
    }
    out
}

/// Render text the way CPython's stdout does (its
/// error handler is surrogateescape): a lone
/// surrogate in DC80 to DCFF writes as its raw byte,
/// and any other lone surrogate fails the write.
pub fn stdout_render(units: &[u16]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(scalar)
                    .expect("a valid pair")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 2;
        } else if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            return None;
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(unit as u32)
                    .expect("a BMP scalar")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 1;
        }
    }
    Some(out)
}

/// Normalize a path's display form the way pathlib
/// does when a Path is printed: empty and dot
/// components drop, duplicate slashes collapse (a
/// leading pair of slashes is preserved), a trailing
/// slash drops, and an empty result is a dot. Dot-dot
/// components are kept; nothing is resolved.
pub fn normalize_path_display(units: &[u16]) -> Vec<u16> {
    let slash = '/' as u16;
    let absolute = units.first() == Some(&slash);
    let double = absolute && units.get(1) == Some(&slash) && units.get(2) != Some(&slash);
    let mut parts: Vec<&[u16]> = Vec::new();
    for part in units.split(|unit| *unit == slash) {
        if part.is_empty() || part == ['.' as u16] {
            continue;
        }
        parts.push(part);
    }
    let mut out: Vec<u16> = Vec::new();
    if double {
        out.extend_from_slice(&[slash, slash]);
    } else if absolute {
        out.push(slash);
    }
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(slash);
        }
        out.extend_from_slice(part);
    }
    if out.is_empty() {
        out.push('.' as u16);
    }
    out
}

/// A message under construction, held as text units
/// so a name the Python inserts raw (a claim id, a
/// path) keeps its exact units until the message is
/// rendered for its destination: stderr renders
/// backslashreplace, stdout surrogateescape.
#[derive(Clone, Debug, PartialEq)]
pub struct Msg(pub Vec<u16>);

impl Default for Msg {
    fn default() -> Msg {
        Msg::new()
    }
}

impl Msg {
    pub fn new() -> Msg {
        Msg(Vec::new())
    }

    pub fn str(text: &str) -> Msg {
        let mut msg = Msg::new();
        msg.push_str(text);
        msg
    }

    pub fn push_str(&mut self, text: &str) {
        self.0.extend(text.encode_utf16());
    }

    pub fn push_text(&mut self, text: &Text) {
        self.0.extend_from_slice(text.units());
    }

    pub fn push_msg(&mut self, other: &Msg) {
        self.0.extend_from_slice(&other.0);
    }

    pub fn render_stderr(&self) -> String {
        stderr_render(&self.0)
    }

    pub fn render_stdout(&self) -> Option<Vec<u8>> {
        stdout_render(&self.0)
    }
}

/// Join messages with a separator, the way the
/// Python joins defect strings.
pub fn join_msgs(items: &[Msg], separator: &str) -> Msg {
    let mut out = Msg::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(separator);
        }
        out.push_msg(item);
    }
    out
}

/// The bare strerror text for an OS error, as the
/// Python's OSError.strerror reads. One table serves
/// the store's phrases and the CLI's "[Errno N]"
/// renderings. Errnos outside the table fall back
/// to the standard library's rendering, a named
/// deviation.
pub fn strerror_text(err: &std::io::Error) -> String {
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

/// The OS path for a normalized display path: the
/// fsencoding of its units.
pub fn os_path_from_units(units: &[u16]) -> std::path::PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(fsencode_bytes(units)))
    }
    #[cfg(not(unix))]
    {
        std::path::PathBuf::from(String::from_utf16_lossy(units))
    }
}

/// The first code point of each block of ten
/// Unicode decimal digits (the Nd category): every
/// block runs 0 to 9 from its start.
const DECIMAL_DIGIT_STARTS: [u32; 68] = [
    0x30, 0x660, 0x6F0, 0x7C0, 0x966, 0x9E6, 0xA66, 0xAE6, 0xB66, 0xBE6, 0xC66, 0xCE6, 0xD66,
    0xDE6, 0xE50, 0xED0, 0xF20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90,
    0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10,
    0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650,
    0x116C0, 0x11730, 0x118E0, 0x11950, 0x11C50, 0x11D50, 0x11DA0, 0x11F50, 0x16A60, 0x16AC0,
    0x16B50, 0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E4F0, 0x1E950,
    0x1FBF0,
];

/// A scalar's Unicode decimal digit value, where it
/// has one: the digit set CPython's float() and
/// argparse's negative number matcher read.
pub fn decimal_digit(scalar: u32) -> Option<u32> {
    let at = DECIMAL_DIGIT_STARTS.partition_point(|start| *start <= scalar);
    if at == 0 {
        return None;
    }
    let start = DECIMAL_DIGIT_STARTS[at - 1];
    let offset = scalar - start;
    if offset < 10 {
        Some(offset)
    } else {
        None
    }
}

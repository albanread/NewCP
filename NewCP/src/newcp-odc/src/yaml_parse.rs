//! Tiny YAML parser for the subset emitted by [`yaml_lossless`].
//!
//! Only handles the constructs the lossless emitter uses:
//!
//! - block mappings with `key: value` or `key:` followed by indented
//!   subnodes
//! - block sequences with `- item` (indented under the parent key)
//! - scalars: integers (decimal or `0xNN` hex), `true`/`false`, bare
//!   identifiers (made of `[A-Za-z0-9._/-]`), double-quoted strings
//!   with the standard escape set (`\\`, `\"`, `\n`, `\t`, `\r`,
//!   `\xNN`)
//! - `!!binary |` followed by one or more indented base64 lines
//!
//! Comments (`#` to end of line) and blank lines are skipped. Block
//! scalar styles other than the two we use (`|` for binary) are not
//! supported. Flow mappings (`{ k: v }`) and flow sequences (`[a, b]`)
//! are *not* handled — the lossless emitter never produces them.
//!
//! The parser is deliberately strict and indentation-sensitive: it
//! cannot read arbitrary YAML, only the canonical output our emitter
//! produces.

use crate::error::{OdcError, Result};

#[derive(Debug, Clone)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
    Mapping(Vec<(String, YamlValue)>),
    Sequence(Vec<YamlValue>),
}

impl YamlValue {
    pub fn as_mapping(&self) -> Result<&Vec<(String, YamlValue)>> {
        match self {
            YamlValue::Mapping(m) => Ok(m),
            _ => Err(OdcError::Inconsistent("expected YAML mapping")),
        }
    }
    pub fn as_sequence(&self) -> Result<&Vec<YamlValue>> {
        match self {
            YamlValue::Sequence(s) => Ok(s),
            _ => Err(OdcError::Inconsistent("expected YAML sequence")),
        }
    }
    pub fn as_str(&self) -> Result<&str> {
        match self {
            YamlValue::String(s) => Ok(s.as_str()),
            _ => Err(OdcError::Inconsistent("expected YAML string")),
        }
    }
    pub fn as_int(&self) -> Result<i64> {
        match self {
            YamlValue::Int(i) => Ok(*i),
            _ => Err(OdcError::Inconsistent("expected YAML integer")),
        }
    }
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            YamlValue::Bool(b) => Ok(*b),
            _ => Err(OdcError::Inconsistent("expected YAML bool")),
        }
    }
    pub fn as_bytes(&self) -> Result<&[u8]> {
        match self {
            YamlValue::Bytes(b) => Ok(b.as_slice()),
            _ => Err(OdcError::Inconsistent("expected YAML !!binary")),
        }
    }
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        if let YamlValue::Mapping(m) = self {
            for (k, v) in m {
                if k == key {
                    return Some(v);
                }
            }
        }
        None
    }
    pub fn require(&self, key: &str) -> Result<&YamlValue> {
        self.get(key).ok_or(OdcError::Inconsistent("missing YAML key"))
    }
}

#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    indent: usize,
    body: &'a str,
}

fn split_lines(input: &str) -> Vec<Line<'_>> {
    let mut out = Vec::new();
    for raw in input.lines() {
        // Strip trailing CR for files saved with CRLF.
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - trimmed.len();
        out.push(Line { indent, body: trimmed });
    }
    out
}

pub fn parse(input: &str) -> Result<YamlValue> {
    let lines = split_lines(input);
    let mut idx = 0;
    if lines.is_empty() {
        return Ok(YamlValue::Null);
    }
    let base_indent = lines[0].indent;
    let value = parse_node(&lines, &mut idx, base_indent)?;
    Ok(value)
}

/// Parse a node whose first significant line sits at indent
/// `expected_indent`. The node is either a mapping or a sequence
/// depending on whether the first line begins with `- `. A scalar at
/// the top level isn't part of our emitter's grammar.
fn parse_node(lines: &[Line<'_>], idx: &mut usize, expected_indent: usize) -> Result<YamlValue> {
    if *idx >= lines.len() {
        return Ok(YamlValue::Null);
    }
    let line = lines[*idx];
    if line.indent < expected_indent {
        return Ok(YamlValue::Null);
    }
    if line.body.starts_with("- ") || line.body == "-" {
        parse_sequence(lines, idx, expected_indent)
    } else {
        parse_mapping(lines, idx, expected_indent)
    }
}

fn parse_mapping(
    lines: &[Line<'_>],
    idx: &mut usize,
    expected_indent: usize,
) -> Result<YamlValue> {
    let mut entries: Vec<(String, YamlValue)> = Vec::new();
    while *idx < lines.len() {
        let line = lines[*idx];
        if line.indent < expected_indent {
            break;
        }
        if line.indent > expected_indent {
            return Err(OdcError::Inconsistent("unexpected indent in YAML mapping"));
        }
        if line.body.starts_with("- ") || line.body == "-" {
            return Err(OdcError::Inconsistent("sequence item where mapping expected"));
        }
        let (key, rest) = split_key(line.body)?;
        *idx += 1;
        let value = if rest.is_empty() {
            // Block-style child node — could be mapping, sequence, or
            // empty scalar (null).
            if *idx < lines.len() && lines[*idx].indent > expected_indent {
                parse_node(lines, idx, lines[*idx].indent)?
            } else {
                YamlValue::Null
            }
        } else if rest.starts_with("!!binary") {
            parse_binary(lines, idx, expected_indent)?
        } else {
            parse_inline_scalar(rest)?
        };
        entries.push((key.to_string(), value));
    }
    Ok(YamlValue::Mapping(entries))
}

fn parse_sequence(
    lines: &[Line<'_>],
    idx: &mut usize,
    expected_indent: usize,
) -> Result<YamlValue> {
    let mut items: Vec<YamlValue> = Vec::new();
    while *idx < lines.len() {
        let line = lines[*idx];
        if line.indent < expected_indent {
            break;
        }
        if line.indent > expected_indent {
            return Err(OdcError::Inconsistent("unexpected indent in YAML sequence"));
        }
        if !(line.body.starts_with("- ") || line.body == "-") {
            break;
        }
        let after_dash = if line.body == "-" { "" } else { &line.body[2..] };
        *idx += 1;
        let item = if after_dash.is_empty() {
            if *idx < lines.len() && lines[*idx].indent > expected_indent {
                parse_node(lines, idx, lines[*idx].indent)?
            } else {
                YamlValue::Null
            }
        } else if let Some((k, rest)) = try_split_key(after_dash) {
            // `- key: value` or `- key:` introduces a mapping whose
            // first entry is on this same line. Subsequent entries sit
            // at indent = expected_indent + 2 (after the `- `).
            let inner_indent = expected_indent + 2;
            let mut entries: Vec<(String, YamlValue)> = Vec::new();
            let first_value = if rest.is_empty() {
                if *idx < lines.len() && lines[*idx].indent > inner_indent {
                    parse_node(lines, idx, lines[*idx].indent)?
                } else {
                    YamlValue::Null
                }
            } else if rest.starts_with("!!binary") {
                parse_binary(lines, idx, inner_indent)?
            } else {
                parse_inline_scalar(rest)?
            };
            entries.push((k.to_string(), first_value));
            // Parse the rest of the mapping at inner_indent.
            while *idx < lines.len() {
                let l = lines[*idx];
                if l.indent < inner_indent {
                    break;
                }
                if l.indent > inner_indent {
                    return Err(OdcError::Inconsistent("unexpected indent inside mapping-in-sequence"));
                }
                if l.body.starts_with("- ") || l.body == "-" {
                    break;
                }
                let (k2, rest2) = split_key(l.body)?;
                *idx += 1;
                let v2 = if rest2.is_empty() {
                    if *idx < lines.len() && lines[*idx].indent > inner_indent {
                        parse_node(lines, idx, lines[*idx].indent)?
                    } else {
                        YamlValue::Null
                    }
                } else if rest2.starts_with("!!binary") {
                    parse_binary(lines, idx, inner_indent)?
                } else {
                    parse_inline_scalar(rest2)?
                };
                entries.push((k2.to_string(), v2));
            }
            YamlValue::Mapping(entries)
        } else {
            // Scalar item in a sequence.
            parse_inline_scalar(after_dash)?
        };
        items.push(item);
    }
    Ok(YamlValue::Sequence(items))
}

fn try_split_key(s: &str) -> Option<(&str, &str)> {
    // A key is `[A-Za-z_][A-Za-z0-9_]*` followed by `:`. Anything else
    // (including a quoted string) is a scalar.
    let mut iter = s.char_indices();
    let first = iter.next()?;
    if !(first.1.is_ascii_alphabetic() || first.1 == '_') {
        return None;
    }
    let mut end = first.0 + first.1.len_utf8();
    for (i, c) in iter {
        if c.is_ascii_alphanumeric() || c == '_' {
            end = i + c.len_utf8();
            continue;
        }
        if c == ':' {
            // Found `key:` — what follows must be empty or whitespace.
            let after = &s[i + 1..];
            let rest = after.trim_start();
            // Distinguish `foo:bar` (no key) from `foo: bar` (key + value).
            if !after.is_empty() && !after.starts_with(' ') {
                return None;
            }
            return Some((&s[..end], rest));
        }
        return None;
    }
    None
}

fn split_key(s: &str) -> Result<(&str, &str)> {
    try_split_key(s).ok_or(OdcError::Inconsistent("expected `key:` at line start"))
}

fn parse_inline_scalar(rest: &str) -> Result<YamlValue> {
    let trimmed = strip_trailing_comment(rest);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return Ok(YamlValue::Null);
    }
    if trimmed == "null" || trimmed == "~" {
        return Ok(YamlValue::Null);
    }
    if trimmed == "true" {
        return Ok(YamlValue::Bool(true));
    }
    if trimmed == "false" {
        return Ok(YamlValue::Bool(false));
    }
    if trimmed.starts_with('"') {
        return Ok(YamlValue::String(parse_double_quoted(trimmed)?));
    }
    if trimmed.starts_with("[]") {
        return Ok(YamlValue::Sequence(Vec::new()));
    }
    if trimmed.starts_with("{}") {
        return Ok(YamlValue::Mapping(Vec::new()));
    }
    if let Some(n) = parse_int(trimmed) {
        return Ok(YamlValue::Int(n));
    }
    // Bare identifier — store as string.
    Ok(YamlValue::String(trimmed.to_string()))
}

fn strip_trailing_comment(s: &str) -> &str {
    // YAML allows `# comment` after a value but not inside a string.
    // We're conservative: strip ` #` if it's outside a quoted segment.
    let mut in_quote = false;
    let mut prev_space = false;
    for (i, c) in s.char_indices() {
        if c == '"' {
            in_quote = !in_quote;
        } else if c == '#' && !in_quote && prev_space {
            return &s[..i];
        }
        prev_space = c == ' ';
    }
    s
}

fn parse_int(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x") {
        return i64::from_str_radix(hex, 16).ok().or_else(|| {
            u64::from_str_radix(hex, 16).ok().map(|u| u as i64)
        });
    }
    if let Some(hex) = s.strip_prefix("-0x") {
        return i64::from_str_radix(hex, 16).ok().map(|n| -n);
    }
    s.parse::<i64>().ok()
}

fn parse_double_quoted(s: &str) -> Result<String> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(OdcError::Inconsistent("string not double-quoted"));
    }
    let inner = &s[1..s.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('0') => out.push('\0'),
                Some('x') => {
                    let h1 = chars.next().ok_or(OdcError::Inconsistent("\\x truncated"))?;
                    let h2 = chars.next().ok_or(OdcError::Inconsistent("\\x truncated"))?;
                    let hi = h1.to_digit(16).ok_or(OdcError::Inconsistent("\\x bad hex"))?;
                    let lo = h2.to_digit(16).ok_or(OdcError::Inconsistent("\\x bad hex"))?;
                    out.push(char::from_u32(hi * 16 + lo).unwrap_or('?'));
                }
                Some(other) => {
                    return Err(OdcError::Inconsistent("unknown escape in YAML string"))
                        .map_err(|e| { let _ = other; e });
                }
                None => return Err(OdcError::Inconsistent("trailing backslash in YAML string")),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

fn parse_binary(lines: &[Line<'_>], idx: &mut usize, expected_indent: usize) -> Result<YamlValue> {
    // The line that introduced `!!binary` (and possibly a `|`) has
    // already been consumed by the caller (which advanced *idx). We
    // accumulate base64 data from indented continuation lines.
    let mut b64 = String::new();
    let inner_indent = expected_indent + 2;
    while *idx < lines.len() {
        let line = lines[*idx];
        if line.indent < inner_indent {
            break;
        }
        b64.push_str(line.body.trim());
        *idx += 1;
    }
    Ok(YamlValue::Bytes(decode_base64(&b64)?))
}

fn decode_base64(s: &str) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in s.chars() {
        if c.is_whitespace() {
            continue;
        }
        if c == '=' {
            // Padding ends the stream; consume any trailing `=` and stop.
            break;
        }
        let v = match c {
            'A'..='Z' => (c as u32) - ('A' as u32),
            'a'..='z' => 26 + (c as u32) - ('a' as u32),
            '0'..='9' => 52 + (c as u32) - ('0' as u32),
            '+' => 62,
            '/' => 63,
            _ => return Err(OdcError::Inconsistent("invalid base64 character")),
        };
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Ok(out)
}

/// Encode bytes to base64. Standard RFC-4648 alphabet, no line wrapping
/// (the emitter wraps lines on its own).
pub fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(ALPHABET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        out.push(ALPHABET[(b2 & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b0 = bytes[i];
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(ALPHABET[((b1 & 0x0F) << 2) as usize] as char);
        out.push('=');
    }
    out
}

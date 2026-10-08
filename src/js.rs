//! JavaScript semantics, exactly. Every helper here exists so the Rust port
//! produces byte-identical output to peek-vanilla: the same doubles, printed
//! the same way, hashed the same way.

/// JS Math.round: ties toward +Infinity. Exact for the magnitudes we use.
pub fn js_round(v: f64) -> i64 {
    (v + 0.5).floor() as i64
}

/// JS String(number): integral values print bare ("2", not "2.0"), the rest
/// use the shortest round-trip decimal. Also maps -0 to "0" like JS.
pub fn js_num_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v == 0.0 {
        return "0".to_string(); // catches -0.0: JS String(-0) === "0"
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    // Rust's {} is the shortest round-trip decimal, like JS, for our range.
    format!("{}", v)
}

/// String(Math.round(v * 100) / 100)
pub fn f2(v: f64) -> String {
    js_num_to_string(js_round(v * 100.0) as f64 / 100.0)
}

/// String(Math.round(v * 1000) / 1000)
pub fn f3(v: f64) -> String {
    js_num_to_string(js_round(v * 1000.0) as f64 / 1000.0)
}

/// JS /\s+/ without the /u flag: the WhiteSpace + LineTerminator list.
fn is_js_space(c: char) -> bool {
    matches!(c,
        '\u{9}'..='\u{D}' | '\u{20}' | '\u{A0}' | '\u{1680}'
        | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}'
        | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}')
}

/// s.normalize('NFC').trim().replace(/\s+/g, ' ').toLowerCase()
pub fn tidy(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let nfc: String = s.nfc().collect();
    let trimmed = nfc.trim_matches(is_js_space);
    // collapse runs of JS whitespace to a single space
    let mut out = String::with_capacity(trimmed.len());
    let mut in_ws = false;
    for c in trimmed.chars() {
        if is_js_space(c) {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out.to_lowercase()
}

/* ---------- minimal JSON.stringify ---------- */

/// A JSON value with significant key order (like a JS object literal).
#[derive(Clone, Debug)]
pub enum JVal {
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
    Arr(Vec<JVal>),
    /// Insertion order is significant.
    Obj(Vec<(String, JVal)>),
}

fn append_json_string(out: &mut String, s: &str) {
    out.push('"');
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u00{:02x}", c as u32));
            }
            c if (0xD800..=0xDBFF).contains(&(c as u32)) => {
                // a valid pair goes out raw, like JSON.stringify; a lone one is escaped
                // (unreachable: a char from a valid &str is never a surrogate)
                if let Some(&d) = chars.peek() {
                    if (0xDC00..=0xDFFF).contains(&(d as u32)) {
                        out.push(c);
                        out.push(chars.next().unwrap());
                    } else {
                        out.push_str(&format!("\\u{:04x}", c as u32));
                    }
                } else {
                    out.push_str(&format!("\\u{:04x}", c as u32));
                }
            }
            // Note: a Rust &str can never hold a lone low surrogate or any
            // other unpaired surrogate; only lone highs are reachable here,
            // and only via char::from_u32_unchecked-style construction, which
            // we never do. Valid lows only appear inside pairs, handled above.
            c => out.push(c),
        }
    }
    out.push('"');
}

fn append_json(out: &mut String, v: &JVal) {
    match v {
        JVal::Str(s) => append_json_string(out, s),
        JVal::Num(n) => out.push_str(&js_num_to_string(*n)),
        JVal::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JVal::Null => out.push_str("null"),
        JVal::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                append_json(out, item);
            }
            out.push(']');
        }
        JVal::Obj(entries) => {
            out.push('{');
            for (i, (k, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                append_json_string(out, k);
                out.push(':');
                append_json(out, item);
            }
            out.push('}');
        }
    }
}

/// JSON.stringify for the narrow shapes settle() hashes. No spaces, like JS.
pub fn js_json_stringify(v: &JVal) -> String {
    let mut out = String::new();
    append_json(&mut out, v);
    out
}

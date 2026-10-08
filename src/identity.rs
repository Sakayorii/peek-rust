//! A name in, an identity out. Port of peek-vanilla's identity.js.
//!
//! Every axis hashes on its own seed, so changing one list never moves
//! another axis: seed(axis) = fnv1a("peek@1:axis:tidy(name)").

use crate::js::js_round;
use crate::tables::{self, LATEST};

pub const STYLE: &str = "peek";

/// FNV-1a over the UTF-8 bytes, with WHATWG TextEncoder semantics.
///
/// Note: a Rust `&str` is always valid UTF-8, so lone surrogates cannot
/// occur here at all (JS callers would see them become U+FFFD before the
/// hash; there is no such input on this side).
pub fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for b in s.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

/// Seeded PRNG (stateful, like the JS closure).
pub struct Mulberry32 {
    a: i32,
}

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self { a: seed as i32 }
    }

    pub fn next(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6d2b79f5u32 as i32);
        // NOTE: JS >>> is a logical shift; Rust >> on i32 is arithmetic,
        // so shift through u32.
        let mut t =
            (self.a ^ ((self.a as u32 >> 15) as i32)).wrapping_mul(1 | self.a);
        t = (t.wrapping_add(
            (t ^ ((t as u32 >> 7) as i32)).wrapping_mul(61 | t),
        )) ^ t;
        (((t ^ ((t as u32 >> 14) as i32)) as u32) as f64) / 4294967296.0
    }
}

/// What each axis picks from, in list order. Append-only.
pub fn lists() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("face", tables::face_names()),
        ("color", tables::color_names()),
        ("eyes", tables::part_list("eyes").to_vec()),
        ("brows", tables::part_list("brows").to_vec()),
        ("mouth", tables::part_list("mouth").to_vec()),
        ("cheeks", tables::part_list("cheeks").to_vec()),
        ("trait", tables::part_list("trait").to_vec()),
    ]
}

/// The axes in readout order.
pub fn axes() -> Vec<&'static str> {
    lists().into_iter().map(|(axis, _)| axis).collect()
}

pub fn seed(name: &str, axis: &str, version: u32) -> u32 {
    fnv1a(&format!(
        "{}@{}:{}:{}",
        STYLE,
        version,
        axis,
        crate::js::tidy(name)
    ))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Persona {
    pub spread: f64,
    pub blink: f64,
    /// Insertion order is significant (draw order / JSON order).
    pub add: Vec<(String, f64)>,
    pub mul: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    pub key: String,
    pub hash: u32,
    pub version: u32,
    pub face: String,
    pub color: String,
    pub eyes: String,
    pub brows: String,
    pub mouth: String,
    pub cheeks: String,
    pub trait_: String,
    pub persona: Persona,
}

pub fn identify(name: &str, version: u32) -> Identity {
    let lengths = tables::version_lengths(version)
        .unwrap_or_else(|| panic!("peek@{} does not exist", version));
    let pick = |axis: &str| -> String {
        let list: Vec<&str> = lists()
            .into_iter()
            .find(|(a, _)| *a == axis)
            .map(|(_, l)| l)
            .unwrap();
        let len = *lengths
            .iter()
            .find(|(a, _)| *a == axis)
            .map(|(_, n)| n)
            .unwrap();
        list[(seed(name, axis, version) % len) as usize].to_string()
    };
    let mut rnd = Mulberry32::new(seed(name, "persona", version));
    let mut r = |a: f64, b: f64| -> f64 { js_round((a + rnd.next() * (b - a)) * 100.0) as f64 / 100.0 };
    // order is the draw order: never reorder these entries
    let persona = Persona {
        spread: r(-7.0, 7.0),
        blink: r(0.75, 1.4),
        add: vec![
            ("browY".to_string(), r(-4.0, 4.0)),
            ("browTilt".to_string(), r(-5.0, 5.0)),
            ("browArch".to_string(), r(-0.2, 0.3)),
            ("rot".to_string(), r(-3.0, 3.0)),
            ("hair".to_string(), r(-0.2, 0.35)),
            ("gx".to_string(), r(-0.12, 0.12)),
            ("my".to_string(), r(-2.0, 3.0)),
        ],
        mul: vec![
            ("eyeS".to_string(), r(0.92, 1.08)),
            ("pupil".to_string(), r(0.9, 1.12)),
            ("browW".to_string(), r(0.85, 1.2)),
            ("mw".to_string(), r(0.85, 1.18)),
        ],
    };
    let key = crate::js::tidy(name);
    Identity {
        key: key.clone(),
        hash: fnv1a(&key),
        version,
        face: pick("face"),
        color: pick("color"),
        eyes: pick("eyes"),
        brows: pick("brows"),
        mouth: pick("mouth"),
        cheeks: pick("cheeks"),
        trait_: pick("trait"),
        persona,
    }
}

pub fn identify_latest(name: &str) -> Identity {
    identify(name, LATEST)
}

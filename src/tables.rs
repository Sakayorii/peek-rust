//! The peek style's tables. Generated from the verified Kotlin Tables.kt.
//!
//! Every list is append-only and ordered: the hash indexes into list order.
//! Never reorder or remove an entry. A version pins how much of each list
//! it may pick from (VERSIONS).

pub struct FaceBody {
    pub d: Option<&'static str>,
    pub circle: Option<&'static [f64]>,
    pub stroke: Option<f64>,
}
pub struct BrowDef { pub y: f64, pub hw: f64, pub arch: f64, pub w: f64 }
pub struct MouthDef { pub x: f64, pub y: f64, pub w: f64, pub d: f64, pub sw: f64 }
pub struct CrownDef { pub poly: &'static [&'static [f64]], pub r: f64 }
pub struct FaceDef {
    pub body: FaceBody,
    pub bottom: f64,
    pub sink: f64,
    pub eyes: &'static [&'static [f64]],
    pub rx: f64,
    pub ry: f64,
    pub pr: f64,
    pub brow: BrowDef,
    pub cheeks: &'static [&'static [f64]],
    pub crx: f64,
    pub cry: f64,
    pub mouth: MouthDef,
    pub crown: CrownDef,
}
pub struct ColorDef { pub body: &'static str, pub deep: &'static str }

pub static INK: &[(&str, &str)] = &[("ink", "#1A1918"), ("bone", "#F3F0E8"), ("paper", "#FFFDF8")];

pub static FACES: &[(&str, FaceDef)] = &[("diamond", FaceDef { body: FaceBody { d: Some("M170 20L320 170L170 320L20 170Z"), circle: None, stroke: None }, bottom: 320.0, sink: 30.0, eyes: &[&[132.5, 155.0], &[207.5, 155.0]], rx: 20.0, ry: 27.5, pr: 10.0, brow: BrowDef { y: 102.5, hw: 18.75, arch: 15.0, w: 5.62 }, cheeks: &[&[117.5, 200.0], &[222.5, 200.0]], crx: 11.25, cry: 5.62, mouth: MouthDef { x: 170.0, y: 218.75, w: 15.0, d: 16.25, sw: 4.38 }, crown: CrownDef { poly: &[&[170.0, 20.0], &[320.0, 170.0], &[170.0, 320.0], &[20.0, 170.0]], r: 0.0 } }), ("semicircle", FaceDef { body: FaceBody { d: Some("M0 255A170 170 0 0 1 340 255Z"), circle: None, stroke: None }, bottom: 255.0, sink: 0.0, eyes: &[&[102.5, 175.0], &[237.5, 175.0]], rx: 30.0, ry: 25.0, pr: 11.25, brow: BrowDef { y: 127.5, hw: 25.0, arch: 15.0, w: 5.62 }, cheeks: &[&[55.0, 212.5], &[285.0, 212.5]], crx: 11.25, cry: 5.62, mouth: MouthDef { x: 170.0, y: 221.25, w: 16.25, d: 16.25, sw: 4.38 }, crown: CrownDef { poly: &[&[170.0, 255.0]], r: 170.0 } }), ("circle", FaceDef { body: FaceBody { d: None, circle: Some(&[170.0, 170.0, 150.0]), stroke: None }, bottom: 320.0, sink: 20.0, eyes: &[&[112.8, 162.2], &[227.2, 162.2]], rx: 29.9, ry: 29.9, pr: 13.0, brow: BrowDef { y: 110.2, hw: 23.4, arch: 15.6, w: 5.85 }, cheeks: &[&[84.2, 203.8], &[255.8, 203.8]], crx: 11.7, cry: 5.85, mouth: MouthDef { x: 170.0, y: 223.3, w: 16.9, d: 16.9, sw: 4.55 }, crown: CrownDef { poly: &[&[170.0, 170.0]], r: 150.0 } }), ("triangle", FaceDef { body: FaceBody { d: Some("M170 40.8L312.8 299.2L27.2 299.2Z"), circle: None, stroke: Some(27.2) }, bottom: 312.8, sink: 0.0, eyes: &[&[132.6, 210.8], &[207.4, 210.8]], rx: 20.4, ry: 27.2, pr: 10.0, brow: BrowDef { y: 166.6, hw: 18.7, arch: 15.3, w: 5.62 }, cheeks: &[&[98.6, 238.0], &[241.4, 238.0]], crx: 11.5, cry: 5.75, mouth: MouthDef { x: 170.0, y: 254.0, w: 17.0, d: 16.25, sw: 4.38 }, crown: CrownDef { poly: &[&[170.0, 40.8], &[312.8, 299.2], &[27.2, 299.2]], r: 13.6 } })];

pub static COLORS: &[(&str, ColorDef)] = &[("lavender", ColorDef { body: "#D8CDF0", deep: "#BDAEE6" }), ("fog", ColorDef { body: "#C8D6E8", deep: "#A8BCDC" }), ("clay", ColorDef { body: "#F1CDBF", deep: "#E2A893" }), ("mint", ColorDef { body: "#CFE7D6", deep: "#A5CDB1" }), ("butter", ColorDef { body: "#ECE2B9", deep: "#D9C284" }), ("rose", ColorDef { body: "#EFCAD7", deep: "#DFA4BA" }), ("aqua", ColorDef { body: "#BDE1E5", deep: "#8FC7D1" })];

pub static PARTS: &[(&str, &[&str])] = &[("eyes", &["oval", "bead", "ring"]), ("brows", &["arch", "bar", "wedge", "dash"]), ("mouth", &["poly", "round", "line", "box"]), ("cheeks", &["oval", "dots", "lines"]), ("trait", &["square", "fin", "ring", "dot", "peak"])];

pub static VERSIONS: &[(u32, &[(&str, u32)])] = &[(1, &[("face", 4), ("color", 7), ("eyes", 3), ("brows", 4), ("mouth", 4), ("cheeks", 3), ("trait", 5)])];

pub const LATEST: u32 = 1;

pub static GROUPS: &[(&str, &[&str])] = &[("eyes", &["lid", "lower", "lidTilt", "eyeS", "lidAsym"]), ("pupil", &["pupil", "gx", "gy", "shine"]), ("brows", &["browY", "browTilt", "browArch", "browW", "browAsym"]), ("mouth", &["mw", "mt", "mb", "mx", "my", "mk"]), ("body", &["alt", "rot", "x", "sx", "sy"]), ("extra", &["blush", "hair", "dim", "thing"])];

pub static EXPRESSIONS: &[(&str, &[(&str, f64)])] = &[
    ("normal", &[("alt", 0.0), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.1), ("lower", 0.0), ("lidTilt", 0.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 1.0), ("gx", 0.0), ("gy", 0.0), ("shine", 0.7), ("browY", 0.0), ("browTilt", 0.0), ("browArch", 0.4), ("browW", 1.0), ("browAsym", 0.0), ("mw", 0.72), ("mt", 0.0), ("mb", 0.5), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 0.0), ("hair", 0.0), ("dim", 0.0), ("thing", 0.0)]),
    ("happy", &[("alt", 0.05), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.0), ("lower", 0.42), ("lidTilt", 0.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 1.0), ("gx", 0.0), ("gy", -0.05), ("shine", 1.0), ("browY", -5.0), ("browTilt", 0.0), ("browArch", 1.0), ("browW", 1.0), ("browAsym", 0.0), ("mw", 1.0), ("mt", 0.0), ("mb", 1.0), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 1.0), ("hair", 0.45), ("dim", 0.0), ("thing", 0.0)]),
    ("sad", &[("alt", -0.16), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 0.985), ("lid", 0.42), ("lower", 0.0), ("lidTilt", 16.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 0.95), ("gx", 0.0), ("gy", 0.8), ("shine", 0.0), ("browY", 3.0), ("browTilt", 14.0), ("browArch", 0.0), ("browW", 1.0), ("browAsym", 0.0), ("mw", 0.8), ("mt", -0.55), ("mb", -0.55), ("mx", 0.0), ("my", 6.0), ("mk", 0.0), ("blush", 0.0), ("hair", -0.75), ("dim", 0.0), ("thing", 0.0)]),
    ("angry", &[("alt", 0.0), ("rot", 0.0), ("x", 0.0), ("sx", 1.02), ("sy", 0.98), ("lid", 0.34), ("lower", 0.12), ("lidTilt", -22.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 0.82), ("gx", 0.0), ("gy", 0.1), ("shine", 0.0), ("browY", 5.0), ("browTilt", -21.0), ("browArch", 0.0), ("browW", 1.3), ("browAsym", 0.0), ("mw", 0.8), ("mt", -0.08), ("mb", -0.08), ("mx", 0.0), ("my", 3.0), ("mk", 0.0), ("blush", 0.0), ("hair", 0.25), ("dim", 0.0), ("thing", 0.0)]),
    ("sleepy", &[("alt", -1.12), ("rot", -3.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 1.0), ("lower", 0.0), ("lidTilt", 4.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 0.9), ("gx", 0.0), ("gy", 0.5), ("shine", 0.0), ("browY", 6.0), ("browTilt", 3.0), ("browArch", 0.15), ("browW", 0.9), ("browAsym", 0.0), ("mw", 0.35), ("mt", -0.3), ("mb", 0.3), ("mx", 0.0), ("my", 2.0), ("mk", 0.0), ("blush", 0.0), ("hair", -1.0), ("dim", 0.0), ("thing", 0.0)]),
    ("curious", &[("alt", -0.32), ("rot", 6.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.0), ("lower", 0.0), ("lidTilt", 0.0), ("eyeS", 1.06), ("lidAsym", 0.0), ("pupil", 1.05), ("gx", 0.0), ("gy", 0.0), ("shine", 1.0), ("browY", -3.0), ("browTilt", 0.0), ("browArch", 0.8), ("browW", 1.0), ("browAsym", 9.0), ("mw", 0.38), ("mt", -0.36), ("mb", 0.36), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 0.0), ("hair", 0.3), ("dim", 0.0), ("thing", 1.0)]),
    ("surprised", &[("alt", 0.36), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.0), ("lower", 0.0), ("lidTilt", 0.0), ("eyeS", 1.2), ("lidAsym", 0.0), ("pupil", 0.66), ("gx", 0.0), ("gy", 0.0), ("shine", 0.8), ("browY", -13.0), ("browTilt", 0.0), ("browArch", 1.0), ("browW", 1.0), ("browAsym", 0.0), ("mw", 0.55), ("mt", -0.62), ("mb", 0.62), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 0.0), ("hair", 1.0), ("dim", 0.0), ("thing", 0.0)]),
    ("excited", &[("alt", 0.12), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.0), ("lower", 0.5), ("lidTilt", 0.0), ("eyeS", 1.08), ("lidAsym", 0.0), ("pupil", 1.08), ("gx", 0.0), ("gy", 0.0), ("shine", 1.0), ("browY", -9.0), ("browTilt", 0.0), ("browArch", 1.0), ("browW", 1.0), ("browAsym", 0.0), ("mw", 1.15), ("mt", 0.0), ("mb", 1.3), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 1.0), ("hair", 1.0), ("dim", 0.0), ("thing", 0.0)]),
    ("confused", &[("alt", -0.05), ("rot", -7.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.22), ("lower", 0.0), ("lidTilt", -4.0), ("eyeS", 1.0), ("lidAsym", 0.28), ("pupil", 0.92), ("gx", -0.35), ("gy", -0.45), ("shine", 0.3), ("browY", -2.0), ("browTilt", -5.0), ("browArch", 0.3), ("browW", 1.0), ("browAsym", 11.0), ("mw", 0.6), ("mt", -0.1), ("mb", 0.12), ("mx", 5.0), ("my", 0.0), ("mk", 13.0), ("blush", 0.0), ("hair", 0.1), ("dim", 0.0), ("thing", 0.0)]),
    ("bored", &[("alt", -0.34), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.0), ("lid", 0.5), ("lower", 0.0), ("lidTilt", 0.0), ("eyeS", 1.0), ("lidAsym", 0.0), ("pupil", 0.9), ("gx", -0.55), ("gy", 0.25), ("shine", 0.0), ("browY", 3.0), ("browTilt", 0.0), ("browArch", 0.1), ("browW", 1.0), ("browAsym", 0.0), ("mw", 0.55), ("mt", 0.0), ("mb", 0.04), ("mx", -5.0), ("my", 0.0), ("mk", 0.0), ("blush", 0.0), ("hair", -0.45), ("dim", 1.0), ("thing", 0.0)]),
    ("attentive", &[("alt", 0.15), ("rot", 0.0), ("x", 0.0), ("sx", 1.0), ("sy", 1.03), ("lid", 0.0), ("lower", 0.0), ("lidTilt", 0.0), ("eyeS", 1.1), ("lidAsym", 0.0), ("pupil", 0.8), ("gx", 0.0), ("gy", -0.1), ("shine", 1.0), ("browY", -7.0), ("browTilt", 0.0), ("browArch", 0.7), ("browW", 1.0), ("browAsym", 0.0), ("mw", 0.55), ("mt", 0.0), ("mb", 0.4), ("mx", 0.0), ("my", 0.0), ("mk", 0.0), ("blush", 0.0), ("hair", 0.65), ("dim", 0.0), ("thing", 0.0)]),
];

pub fn face(name: &str) -> &'static FaceDef {
    FACES.iter().find(|(n, _)| *n == name).map(|(_, f)| f)
        .unwrap_or_else(|| panic!("unknown face {}", name))
}
pub fn face_names() -> Vec<&'static str> { FACES.iter().map(|(n, _)| *n).collect() }
pub fn color(name: &str) -> &'static ColorDef {
    COLORS.iter().find(|(n, _)| *n == name).map(|(_, c)| c)
        .unwrap_or_else(|| panic!("unknown color {}", name))
}
pub fn color_names() -> Vec<&'static str> { COLORS.iter().map(|(n, _)| *n).collect() }
pub fn part_list(axis: &str) -> &'static [&'static str] {
    PARTS.iter().find(|(n, _)| *n == axis).map(|(_, l)| *l)
        .unwrap_or_else(|| panic!("unknown part axis {}", axis))
}
pub fn expression(name: &str) -> &'static [(&'static str, f64)] {
    EXPRESSIONS.iter().find(|(n, _)| *n == name).map(|(_, e)| *e)
        .unwrap_or_else(|| panic!("unknown expression {}", name))
}
pub fn expression_names() -> Vec<&'static str> { EXPRESSIONS.iter().map(|(n, _)| *n).collect() }
pub fn version_lengths(v: u32) -> Option<&'static [(&'static str, u32)]> {
    VERSIONS.iter().find(|(n, _)| *n == v).map(|(_, l)| *l)
}
pub fn ink(name: &str) -> &'static str {
    INK.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
        .unwrap_or_else(|| panic!("unknown ink {}", name))
}

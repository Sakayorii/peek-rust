//! toSvg: a name in, a standalone SVG string out. Same input, same bytes, on
//! any machine. Port of peek-vanilla's svg.js.

use crate::draw::{draw, rest_pose, Pose};
use crate::identity::{fnv1a, identify, Identity};
use crate::js::{js_json_stringify, JVal};
use crate::scene::SNode;
use crate::tables::LATEST;

/// Render options. An explicit face/color/part wins over the hash; every
/// other axis stays put.
#[derive(Clone, Debug)]
pub struct PeekOptions {
    pub face: Option<String>,
    pub color: Option<String>,
    pub eyes: Option<String>,
    pub brows: Option<String>,
    pub mouth: Option<String>,
    pub cheeks: Option<String>,
    pub trait_: Option<String>,
    pub expression: String,
    /// [x, y] in -1..1, overrides the expression's gaze.
    pub gaze: Option<[f64; 2]>,
    pub size: f64,
    pub frame: String,
    pub square: bool,
    pub riso: bool,
    pub id: Option<String>,
    /// Defaults to the name. Set hide_title to drop the aria label.
    pub title: Option<String>,
    pub hide_title: bool,
    pub version: u32,
}

impl Default for PeekOptions {
    fn default() -> Self {
        Self {
            face: None,
            color: None,
            eyes: None,
            brows: None,
            mouth: None,
            cheeks: None,
            trait_: None,
            expression: "normal".to_string(),
            gaze: None,
            size: 64.0,
            frame: "ink".to_string(),
            square: true,
            riso: false,
            id: None,
            title: None,
            hide_title: false,
            version: LATEST,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawOpts {
    pub size: f64,
    pub frame: String,
    pub square: bool,
    pub riso: bool,
    pub live: bool,
    pub id: String,
    /// Resolved title: the name, an override, or None when hidden.
    pub title: Option<String>,
}

fn who_to_json(who: &Identity) -> JVal {
    JVal::Obj(vec![
        ("key".to_string(), JVal::Str(who.key.clone())),
        ("hash".to_string(), JVal::Num(who.hash as f64)),
        ("version".to_string(), JVal::Num(who.version as f64)),
        ("face".to_string(), JVal::Str(who.face.clone())),
        ("color".to_string(), JVal::Str(who.color.clone())),
        ("eyes".to_string(), JVal::Str(who.eyes.clone())),
        ("brows".to_string(), JVal::Str(who.brows.clone())),
        ("mouth".to_string(), JVal::Str(who.mouth.clone())),
        ("cheeks".to_string(), JVal::Str(who.cheeks.clone())),
        ("trait".to_string(), JVal::Str(who.trait_.clone())),
        (
            "persona".to_string(),
            JVal::Obj(vec![
                ("spread".to_string(), JVal::Num(who.persona.spread)),
                ("blink".to_string(), JVal::Num(who.persona.blink)),
                (
                    "add".to_string(),
                    JVal::Obj(
                        who.persona
                            .add
                            .iter()
                            .map(|(k, v)| (k.clone(), JVal::Num(*v)))
                            .collect(),
                    ),
                ),
                (
                    "mul".to_string(),
                    JVal::Obj(
                        who.persona
                            .mul
                            .iter()
                            .map(|(k, v)| (k.clone(), JVal::Num(*v)))
                            .collect(),
                    ),
                ),
            ]),
        ),
    ])
}

fn pose_to_json(pose: &Pose) -> JVal {
    let mut entries: Vec<(String, JVal)> = pose
        .channels
        .iter()
        .map(|(k, v)| (k.clone(), JVal::Num(*v)))
        .collect();
    entries.push(("expression".to_string(), JVal::Str(pose.expression.clone())));
    JVal::Obj(entries)
}

fn look_to_json(o: &PeekOptions, live: bool) -> JVal {
    JVal::Obj(vec![
        ("size".to_string(), JVal::Num(o.size)),
        ("frame".to_string(), JVal::Str(o.frame.clone())),
        ("square".to_string(), JVal::Bool(o.square)),
        ("riso".to_string(), JVal::Bool(o.riso)),
        ("live".to_string(), JVal::Bool(live)),
    ])
}

fn to_base36(mut v: u32) -> String {
    // Kotlin's UInt.toString(36): lowercase, no padding.
    if v == 0 {
        return "0".to_string();
    }
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while v > 0 {
        out.push(digits[(v % 36) as usize] as char);
        v /= 36;
    }
    out.iter().rev().collect()
}

pub fn settle(name: &str, o: &PeekOptions, live: bool) -> (Identity, Pose, DrawOpts) {
    let base = identify(name, o.version);
    let who = Identity {
        face: o.face.clone().unwrap_or(base.face),
        color: o.color.clone().unwrap_or(base.color),
        eyes: o.eyes.clone().unwrap_or(base.eyes),
        brows: o.brows.clone().unwrap_or(base.brows),
        mouth: o.mouth.clone().unwrap_or(base.mouth),
        cheeks: o.cheeks.clone().unwrap_or(base.cheeks),
        trait_: o.trait_.clone().unwrap_or(base.trait_),
        ..base
    };
    let pose = rest_pose(&who, &o.expression, o.gaze);
    let id = o.id.clone().unwrap_or_else(|| {
        format!(
            "peek-{}",
            to_base36(fnv1a(&js_json_stringify(&JVal::Arr(vec![
                who_to_json(&who),
                pose_to_json(&pose),
                look_to_json(o, live),
            ]))))
        )
    });
    let opts = DrawOpts {
        size: o.size,
        frame: o.frame.clone(),
        square: o.square,
        riso: o.riso,
        live,
        id,
        title: if o.hide_title {
            None
        } else {
            Some(o.title.clone().unwrap_or_else(|| name.to_string()))
        },
    };
    (who, pose, opts)
}

// Characters XML 1.0 cannot hold at all: C0 controls (a Rust &str never holds
// lone surrogates, so only the C0 + FFFE/FFFF arms are reachable here).
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let c = match c {
            '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}' => {
                '\u{FFFD}'
            }
            _ => c,
        };
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn append_node(out: &mut String, node: &SNode) {
    out.push('<');
    out.push_str(&node.tag);
    for (k, v) in &node.attrs {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(&esc(v));
        out.push('"');
    }
    if node.children.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for c in &node.children {
        append_node(out, c);
    }
    out.push_str("</");
    out.push_str(&node.tag);
    out.push('>');
}

pub fn serialize(node: &SNode) -> String {
    let mut out = String::new();
    append_node(&mut out, node);
    out
}

/// A name in, a standalone SVG string out.
pub fn to_svg(name: &str, opts: &PeekOptions) -> String {
    let (who, pose, draw_opts) = settle(name, opts, false);
    serialize(&draw(&who, &pose, &draw_opts))
}

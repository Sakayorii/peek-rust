//! One pure geometry function: an identity and a pose in, a plain node tree
//! out. Port of peek-vanilla's draw.js.
//!
//! Structure depends only on the identity and the options, never the pose,
//! so a live tree keeps the same nodes frame to frame. A static tree drops
//! what is invisible (opacity 0) to stay small.

use crate::identity::Identity;
use crate::js::{f2, f3, js_num_to_string, js_round};
use crate::scene::{n, prune, AttrVal, SNode};
use crate::svg::DrawOpts;
use crate::tables::{self, CrownDef, FaceDef};

/// The frame: viewBox -30 -30 400 400, the floor on its bottom edge.
const VB: [f64; 4] = [-30.0, -30.0, 400.0, 400.0];
const FLOOR: f64 = 370.0;
const THING: [f64; 2] = [330.0, 20.0];
const RAD: f64 = std::f64::consts::PI / 180.0;

pub fn clamp(v: f64, a: f64, b: f64) -> f64 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}

pub fn smooth(a: f64, b: f64, v: f64) -> f64 {
    let t = clamp((v - a) / (b - a), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn stroke_for(px: f64) -> f64 {
    if px >= 96.0 {
        1.0
    } else if px >= 56.0 {
        1.25
    } else if px >= 40.0 {
        1.6
    } else if px >= 30.0 {
        2.0
    } else {
        2.4
    }
}

/// Pose: expression channels in BASE order, plus blink/lag, plus the expression name.
#[derive(Clone, Debug)]
pub struct Pose {
    /// Insertion order is significant (JSON order).
    pub channels: Vec<(String, f64)>,
    pub expression: String,
}

impl Pose {
    pub fn get(&self, c: &str) -> f64 {
        self.channels
            .iter()
            .find(|(k, _)| k == c)
            .map(|(_, v)| *v)
            .unwrap_or(0.0)
    }
    fn set(&mut self, c: &str, v: f64) {
        if let Some(e) = self.channels.iter_mut().find(|(k, _)| k == c) {
            e.1 = v;
        } else {
            self.channels.push((c.to_string(), v));
        }
    }
}

/// Where a face rests: the body offset, the eye line, one altitude unit.
pub struct Frame {
    pub base_y: f64,
    pub eye_y: f64,
    pub r: f64,
}

pub fn frame_of(face: &FaceDef) -> Frame {
    let base_y = FLOOR + face.sink - face.bottom;
    let eye_y = face.eyes[0][1];
    // altitude -1 puts the eye line on the floor
    Frame {
        base_y,
        eye_y,
        r: FLOOR - (eye_y + base_y),
    }
}

/// The eyes' aim at the thing in the corner, from wherever they are.
pub fn thing_gaze(face: &FaceDef, x: f64, alt: f64) -> [f64; 2] {
    let fr = frame_of(face);
    let ey = fr.eye_y + fr.base_y - alt * fr.r;
    [
        clamp((THING[0] - 170.0 - x) / 150.0, -1.0, 1.0),
        clamp((THING[1] - ey) / 150.0, -1.0, 1.0),
    ]
}

/// The pose an expression settles to. A curious face ends up looking at the
/// thing in the corner; an explicit gaze wins over both.
pub fn rest_pose(who: &Identity, expression: &str, gaze: Option<[f64; 2]>) -> Pose {
    let base = tables::expression(expression);
    let mut channels: Vec<(String, f64)> =
        base.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    let mut pose = Pose {
        channels,
        expression: expression.to_string(),
    };
    // push blink/lag after the expression channels, like the JS does
    pose.channels.push(("blink".to_string(), 0.0));
    pose.channels.push(("lag".to_string(), 0.0));
    // the choreography's last key leaves curious leaning 7, not the table's 6
    if expression == "curious" {
        pose.set("rot", 7.0);
    }
    let g = match gaze {
        Some(g) => Some(g),
        None => {
            if expression == "curious" {
                Some(thing_gaze(tables::face(&who.face), 0.0, pose.get("alt")))
            } else {
                None
            }
        }
    };
    if let Some(g) = g {
        pose.set("gx", g[0]);
        pose.set("gy", g[1]);
    }
    pose
}

pub struct Seat {
    pub x: f64,
    pub y: f64,
    pub a: f64,
}

/// A point on the crown outline grown by `d`, reached by walking `u` along it
/// from the top (where the outward normal points straight up); u > 0 walks
/// right. Returns the point and the normal's bearing in degrees.
pub fn seat(crown: &CrownDef, d: f64, u: f64) -> Seat {
    let r = 0.0f64.max(crown.r + d);
    let at = |p: &[f64], b: f64| Seat {
        x: p[0] + r * b.sin(),
        y: p[1] - r * b.cos(),
        a: b / RAD,
    };
    let p = crown.poly;
    if p.len() == 1 {
        return at(p[0], if r != 0.0 { u / r } else { 0.0 });
    }
    let n = p.len() as i32;
    let dir: i32 = if u >= 0.0 { 1 } else { -1 };
    let mut left = u.abs();
    let mut i: i32 = 0;
    let mut b = 0.0;
    for _ in 0..n {
        let j = (i + dir + n) % n;
        let (x0, y0) = (p[i as usize][0], p[i as usize][1]);
        let (x1, y1) = (p[j as usize][0], p[j as usize][1]);
        let ex = x1 - x0;
        let ey = y1 - y0;
        let l = ex.hypot(ey);
        let nx = (dir as f64 * ey) / l;
        let ny = (-dir as f64 * ex) / l;
        let be = nx.atan2(-ny);
        let mut db = be - b;
        while dir as f64 * db < 0.0 {
            db += dir as f64 * 2.0 * std::f64::consts::PI;
        }
        let arc = r * db.abs();
        if left <= arc {
            return at(p[i as usize], if r != 0.0 { b + (dir as f64 * left) / r } else { b });
        }
        left -= arc;
        if left <= l {
            return Seat {
                x: x0 + (ex / l) * left + r * nx,
                y: y0 + (ey / l) * left + r * ny,
                a: be / RAD,
            };
        }
        left -= l;
        i = j;
        b = be;
    }
    at(p[i as usize], b)
}

/// Signed distance from a point to the crown outline, negative inside.
pub fn depth(crown: &CrownDef, x: f64, y: f64) -> f64 {
    let p = crown.poly;
    if p.len() == 1 {
        return (x - p[0][0]).hypot(y - p[0][1]) - crown.r;
    }
    let mut out = f64::NEG_INFINITY;
    let mut near = f64::INFINITY;
    for i in 0..p.len() {
        let a = p[i];
        let b = p[(i + 1) % p.len()];
        let ex = b[0] - a[0];
        let ey = b[1] - a[1];
        let l2 = ex * ex + ey * ey;
        let l = l2.sqrt();
        out = out.max(((x - a[0]) * ey - (y - a[1]) * ex) / l);
        let t = clamp(((x - a[0]) * ex + (y - a[1]) * ey) / l2, 0.0, 1.0);
        near = near.min((x - a[0] - t * ex).hypot(y - a[1] - t * ey));
    }
    (if out <= 0.0 { out } else { near }) - crown.r
}

const RING_U: f64 = 158.0 * 35.0 * RAD;

/// The ring's rest position and how far up it may ride, as (rest, min).
/// Port of ringWalk in draw.js.
pub fn ring_walk(sh: &FaceDef) -> (f64, f64) {
    let ex = sh.eyes[1][0] + sh.brow.hw + 7.0;
    let top = sh.brow.y - 30.0;
    let bottom = sh.brow.y + 8.0;
    let mut u = RING_U;
    while u < 400.0 {
        let s = seat(&sh.crown, 8.0, u);
        let dx = s.x - ex;
        let dy = s.y - clamp(s.y, top, bottom);
        if if s.x > ex {
            dx.hypot(dy) > 27.0
        } else {
            s.y < top - 27.0
        } {
            break;
        }
        u += 2.0;
    }
    if u > RING_U {
        (u + 20.0, u)
    } else {
        (u, f64::NEG_INFINITY)
    }
}

pub fn draw(who: &Identity, pose: &Pose, o: &DrawOpts) -> SNode {
    let sh = tables::face(&who.face);
    let color = tables::color(&who.color);
    let fill = color.body;
    let deep = color.deep;
    let p = &who.persona;
    let mul = &p.mul;
    let add = &p.add;
    let g = |c: &str| -> f64 {
        pose.get(c)
            * mul.iter().find(|(k, _)| k == c).map(|(_, v)| *v).unwrap_or(1.0)
            + add.iter().find(|(k, _)| k == c).map(|(_, v)| *v).unwrap_or(0.0)
    };
    let spread = p.spread;
    let id = &o.id;
    let st = stroke_for(o.size);
    let fr = frame_of(sh);
    let base_y = fr.base_y;
    let r_ = fr.r;
    let mut defs: Vec<SNode> = Vec::new();
    // the frame
    defs.push(n(
        "clipPath",
        "frame.clip",
        vec![("id", format!("{}-f", id).into())],
        vec![if o.square {
            n(
                "rect",
                "frame.clip.shape",
                vec![
                    ("x", VB[0].into()),
                    ("y", VB[1].into()),
                    ("width", VB[2].into()),
                    ("height", VB[3].into()),
                ],
                vec![],
            )
        } else {
            n(
                "circle",
                "frame.clip.shape",
                vec![("cx", 170.into()), ("cy", 170.into()), ("r", 200.into())],
                vec![],
            )
        }],
    ));
    // riso: registration error grows with size; a different seed per plate
    let k = if o.riso { smooth(120.0, 480.0, o.size) } else { 0.0 };
    let riso = k > 0.01;
    let upp = VB[2] / o.size.max(1.0);
    let mk = k * k;
    let plate = |pl: &str, seed_a: i32, seed_b: i32, sc: f64| -> SNode {
        n(
            "filter",
            &format!("riso.{}", pl),
            vec![
                ("id", format!("{}-p{}", id, pl).into()),
                ("x", "-20%".into()),
                ("y", "-20%".into()),
                ("width", "140%".into()),
                ("height", "140%".into()),
                ("color-interpolation-filters", "sRGB".into()),
            ],
            vec![
                n(
                    "feTurbulence",
                    &format!("riso.{}.warp", pl),
                    vec![
                        ("type", "fractalNoise".into()),
                        ("baseFrequency", "0.032".into()),
                        ("numOctaves", 2.into()),
                        ("seed", seed_a.into()),
                        ("result", "warp".into()),
                    ],
                    vec![],
                ),
                n(
                    "feDisplacementMap",
                    &format!("riso.{}.disp", pl),
                    vec![
                        ("in", "SourceGraphic".into()),
                        ("in2", "warp".into()),
                        ("scale", f2(sc * k).into()),
                        ("xChannelSelector", "R".into()),
                        ("yChannelSelector", "G".into()),
                        ("result", "moved".into()),
                    ],
                    vec![],
                ),
                n(
                    "feTurbulence",
                    &format!("riso.{}.grain", pl),
                    vec![
                        ("type", "fractalNoise".into()),
                        ("baseFrequency", "0.85".into()),
                        ("numOctaves", 2.into()),
                        ("seed", seed_b.into()),
                        ("result", "grain".into()),
                    ],
                    vec![],
                ),
                n(
                    "feColorMatrix",
                    &format!("riso.{}.mask", pl),
                    vec![
                        ("in", "grain".into()),
                        ("type", "matrix".into()),
                        (
                            "values",
                            format!(
                                "0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  {} 0 0 0 {}",
                                f3(-8.0 * mk),
                                f3(1.0 + 5.85 * mk)
                            )
                            .into(),
                        ),
                        ("result", "mask".into()),
                    ],
                    vec![],
                ),
                n(
                    "feComposite",
                    &format!("riso.{}.out", pl),
                    vec![
                        ("in", "moved".into()),
                        ("in2", "mask".into()),
                        ("operator", "in".into()),
                    ],
                    vec![],
                ),
            ],
        )
    };
    let grainy = riso && o.frame != "none";
    if riso {
        defs.push(plate("A", 4, 11, 3.2));
        defs.push(plate("B", 9, 23, 2.2));
    }
    if grainy {
        // bone speckle on ink, ink speckle on light grounds
        let (rr, gg, bb) = if o.frame == "ink" {
            (0.953, 0.941, 0.91)
        } else {
            (0.102, 0.098, 0.094)
        };
        defs.push(n(
            "filter",
            "grain.filter",
            vec![
                ("id", format!("{}-grain", id).into()),
                ("x", 0.into()),
                ("y", 0.into()),
                ("width", 1.into()),
                ("height", 1.into()),
                ("color-interpolation-filters", "sRGB".into()),
            ],
            vec![
                n(
                    "feTurbulence",
                    "grain.noise",
                    vec![
                        ("type", "fractalNoise".into()),
                        ("baseFrequency", "0.9".into()),
                        ("numOctaves", 2.into()),
                        ("seed", 5.into()),
                    ],
                    vec![],
                ),
                n(
                    "feColorMatrix",
                    "grain.matrix",
                    vec![
                        ("type", "matrix".into()),
                        (
                            "values",
                            format!(
                                "0 0 0 0 {}  0 0 0 0 {}  0 0 0 0 {}  7 0 0 0 -5.1",
                                js_num_to_string(rr),
                                js_num_to_string(gg),
                                js_num_to_string(bb)
                            )
                            .into(),
                        ),
                    ],
                    vec![],
                ),
            ],
        ));
    }
    // colour drains a little when bored
    let dim = clamp(g("dim"), 0.0, 1.0);
    if o.live || dim > 0.01 {
        defs.push(n(
            "filter",
            "dim.filter",
            vec![
                ("id", format!("{}-dim", id).into()),
                ("color-interpolation-filters", "sRGB".into()),
            ],
            vec![n(
                "feColorMatrix",
                "dim.matrix",
                vec![
                    ("type", "saturate".into()),
                    ("values", f3(1.0 - dim * 0.55).into()),
                ],
                vec![],
            )],
        ));
    }
    // the body
    let alt = g("alt");
    let rot = g("rot");
    let x = g("x");
    let sx = g("sx");
    let sy = g("sy");
    let b_ = sh.bottom;
    let ty = base_y - alt * r_;
    let sc = |v: f64| js_round(v * 100000.0) as f64 / 100000.0;
    let body_node = if let Some(d) = sh.body.d {
        n(
            "path",
            "body",
            vec![
                ("d", d.into()),
                ("fill", fill.into()),
                (
                    "stroke",
                    if sh.body.stroke.is_some() {
                        Some(fill)
                    } else {
                        None
                    }
                    .into(),
                ),
                ("stroke-width", sh.body.stroke.into()),
                (
                    "stroke-linejoin",
                    if sh.body.stroke.is_some() {
                        Some("round")
                    } else {
                        None
                    }
                    .into(),
                ),
            ],
            vec![],
        )
    } else {
        let c = sh.body.circle.expect("face body needs d or circle");
        n(
            "circle",
            "body",
            vec![
                ("cx", c[0].into()),
                ("cy", c[1].into()),
                ("r", c[2].into()),
                ("fill", fill.into()),
            ],
            vec![],
        )
    };
    // eyes
    let eye_s = g("eyeS");
    let rx = sh.rx * eye_s;
    let ry = sh.ry * eye_s;
    let lid0 = g("lid");
    let lid_asym = g("lidAsym");
    let lower = clamp(g("lower"), 0.0, 1.0);
    let blink = pose.get("blink");
    let pupil = g("pupil");
    let gx = clamp(g("gx"), -1.1, 1.1);
    let gy = clamp(g("gy"), -1.1, 1.1);
    let tilt = g("lidTilt");
    let shine = clamp(g("shine"), 0.0, 1.0);
    let sw = sh.brow.w * st;
    let bead = who.eyes == "bead";
    let mut eyes: Vec<SNode> = Vec::new();
    for (i, e) in sh.eyes.iter().enumerate() {
        let ex = e[0];
        let ey = e[1];
        let side: f64 = if i == 0 { -1.0 } else { 1.0 };
        let kk = format!("eye{}", i);
        let mut lid = clamp(
            lid0 + if i == 0 {
                lid_asym
            } else {
                -lid_asym * 0.3
            },
            0.0,
            1.0,
        );
        lid = lid + (1.0 - lid) * blink;
        defs.push(n(
            "clipPath",
            &format!("{}.clip", kk),
            vec![("id", format!("{}-e{}", id, i).into())],
            vec![n(
                "ellipse",
                &format!("{}.clip.shape", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("rx", f2(rx).into()),
                    ("ry", f2(ry).into()),
                ],
                vec![],
            )],
        ));
        defs.push(n(
            "clipPath",
            &format!("{}.lidclip", kk),
            vec![("id", format!("{}-l{}", id, i).into())],
            vec![n(
                "ellipse",
                &format!("{}.lidclip.shape", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("rx", f2(rx + 1.6).into()),
                    ("ry", f2(ry + 1.6).into()),
                ],
                vec![],
            )],
        ));
        // what sits in the socket: a pupil on paper, a bare bead, or a rimmed eye
        let br = sh.rx.min(sh.ry) * 0.62;
        let pr = if bead {
            br * eye_s * (0.4 + 0.6 * pupil)
        } else {
            sh.pr * pupil
        };
        let base = if bead { br } else { sh.pr };
        let px = gx * 0.0f64.max(sh.rx - base) * 0.9;
        let py = gy * 0.0f64.max(sh.ry - base) * 0.8;
        // capped so a small rimmed eye stays an eye, not an ink blot
        let rim_w = (sw * 0.8).min(sh.rx.min(sh.ry) * 0.28);
        let mut inner: Vec<SNode> = Vec::new();
        if !bead {
            inner.push(n(
                "ellipse",
                &format!("{}.white", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("rx", f2(rx).into()),
                    ("ry", f2(ry).into()),
                    ("fill", tables::ink("paper").into()),
                ],
                vec![],
            ));
        }
        inner.push(n(
            "circle",
            &format!("{}.pupil", kk),
            vec![
                ("cx", f2(px).into()),
                ("cy", f2(py).into()),
                ("r", f2(pr).into()),
                ("fill", tables::ink("ink").into()),
            ],
            vec![],
        ));
        inner.push(n(
            "circle",
            &format!("{}.shine", kk),
            vec![
                ("cx", f2(px + pr * 0.44).into()),
                ("cy", f2(py - pr * 0.44).into()),
                ("r", f2(pr * if bead { 0.26 } else { 0.3 }).into()),
                ("fill", tables::ink("paper").into()),
                ("opacity", f2(shine).into()),
            ],
            vec![],
        ));
        if who.eyes == "ring" {
            inner.push(n(
                "ellipse",
                &format!("{}.rim", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("rx", f2(rx - rim_w / 2.0).into()),
                    ("ry", f2(ry - rim_w / 2.0).into()),
                    ("fill", "none".into()),
                    ("stroke", tables::ink("ink").into()),
                    ("stroke-width", f2(rim_w).into()),
                ],
                vec![],
            ));
        }
        // upper lid: a straight cut, tilted, same as the v1.0 sad and angry lids
        let edge = -ry - 1.6 + lid * (2.0 * ry + 3.2);
        let a = side * tilt;
        // lower lid: a wide arc pushing up from below, the smiling squint
        let lrx = rx * 1.55;
        let lry = ry * 1.15;
        let top = ry + 1.6 - lower * ry * 1.5;
        let mut parts: Vec<SNode> = vec![
            n(
                "g",
                &format!("{}.inner", kk),
                vec![("clip-path", format!("url(#{}-e{})", id, i).into())],
                inner,
            ),
            n(
                "g",
                &format!("{}.lids", kk),
                vec![("clip-path", format!("url(#{}-l{})", id, i).into())],
                vec![
                    n(
                        "rect",
                        &format!("{}.lid", kk),
                        vec![
                            ("x", f2(-rx * 2.2).into()),
                            ("y", f2(-ry * 3.0).into()),
                            ("width", f2(rx * 4.4).into()),
                            ("height", f2(0.0f64.max(edge + ry * 3.0)).into()),
                            ("transform", format!("rotate({})", f2(a)).into()),
                            ("fill", fill.into()),
                        ],
                        vec![],
                    ),
                    n(
                        "ellipse",
                        &format!("{}.lower", kk),
                        vec![
                            ("cx", 0.into()),
                            ("cy", f2(top + lry).into()),
                            ("rx", f2(lrx).into()),
                            ("ry", f2(lry).into()),
                            ("fill", fill.into()),
                        ],
                        vec![],
                    ),
                ],
            ),
        ];
        // a rimmed eye inks its lid edges too, so the outline follows the lids
        if who.eyes == "ring" {
            parts.push(n(
                "g",
                &format!("{}.edges", kk),
                vec![
                    ("clip-path", format!("url(#{}-e{})", id, i).into()),
                    ("fill", "none".into()),
                    ("stroke", tables::ink("ink").into()),
                    ("stroke-width", f2(rim_w).into()),
                ],
                vec![
                    n(
                        "path",
                        &format!("{}.edge", kk),
                        vec![
                            (
                                "d",
                                format!(
                                    "M{} {}H{}",
                                    f2(-rx * 2.2),
                                    f2(edge),
                                    f2(rx * 2.2)
                                )
                                .into(),
                            ),
                            ("transform", format!("rotate({})", f2(a)).into()),
                            ("opacity", if lid > 0.02 { 1 } else { 0 }.into()),
                        ],
                        vec![],
                    ),
                    n(
                        "ellipse",
                        &format!("{}.edge.lower", kk),
                        vec![
                            ("cx", 0.into()),
                            ("cy", f2(top + lry).into()),
                            ("rx", f2(lrx).into()),
                            ("ry", f2(lry).into()),
                            ("opacity", if lower > 0.02 { 1 } else { 0 }.into()),
                        ],
                        vec![],
                    ),
                ],
            ));
        }
        // closed eye: a hairline drawn once the lid is down
        let co = smooth(0.86, 1.0, lid);
        let yy = edge.min(ry) - 1.0;
        let hw = rx * 0.92;
        parts.push(n(
            "path",
            &format!("{}.closed", kk),
            vec![
                (
                    "d",
                    format!(
                        "M{} {}Q0 {} {} {}",
                        f2(-hw),
                        f2(yy - 2.0),
                        f2(yy + 6.0),
                        f2(hw),
                        f2(yy - 2.0)
                    )
                    .into(),
                ),
                ("transform", format!("rotate({})", f2(a * 0.6)).into()),
                ("fill", "none".into()),
                ("stroke", tables::ink("ink").into()),
                ("stroke-linecap", "round".into()),
                ("stroke-width", f2(sw * 0.85).into()),
                (
                    "opacity",
                    if co > 0.01 {
                        f2(co).into()
                    } else {
                        AttrVal::from(0)
                    },
                ),
            ],
            vec![],
        ));
        eyes.push(n(
            "g",
            &kk,
            vec![(
                "transform",
                if bead {
                    format!(
                        "translate({} {})",
                        f2(ex + side * spread + gx * 6.0),
                        f2(ey + gy * 6.0)
                    )
                } else {
                    format!("translate({} {})", f2(ex + side * spread), js_num_to_string(ey))
                }
                .into(),
            )],
            parts,
        ));
    }
    // brows
    let brow_y = g("browY") - (eye_s - 1.0) * sh.ry * 0.9;
    let btilt = g("browTilt");
    let arch = g("browArch");
    let basym = g("browAsym");
    let bw = sw * g("browW");
    let mut brows: Vec<SNode> = Vec::new();
    for (i, e) in sh.eyes.iter().enumerate() {
        let ex = e[0];
        let side: f64 = if i == 0 { -1.0 } else { 1.0 };
        let hw = sh.brow.hw;
        let c = -sh.brow.arch * arch;
        // half-length, arch depth and stroke weight per type
        let (w, ca, k2): (f64, f64, f64) = match who.brows.as_str() {
            "bar" => (hw * 0.9, c * 0.3, 1.75),
            "dash" => (hw * 0.42, c * 0.25, 1.9),
            _ => (hw, c, 1.0),
        };
        // wedge: tapered, heavy at the inner end, toward the nose
        let ti = bw * 1.7;
        let to = bw * 0.45;
        let (tl, tr) = if side < 0.0 { (to, ti) } else { (ti, to) };
        let wedge = who.brows == "wedge";
        let (hl, hr) = if wedge {
            (tl / 2.0 + to / 2.0, tr / 2.0 + to / 2.0)
        } else {
            (0.0, 0.0)
        };
        let half = |t: f64| -> f64 {
            if wedge {
                hl + (hr - hl) * t
            } else {
                (bw * k2) / 2.0
            }
        };
        // a brow never leaves the face: raised into the outline, it stops there
        let bx = ex + side * spread;
        let by = sh.brow.y + brow_y + if i == 1 { -basym } else { basym * 0.3 };
        let cos = (side * btilt * RAD).cos();
        let sin = (side * btilt * RAD).sin();
        let inside = |dy: f64| -> bool {
            for tt in 0..=4 {
                let t = tt as f64 / 4.0;
                let lx = w * (2.0 * t - 1.0);
                let ly = 2.0 * t * (1.0 - t) * ca;
                let px = bx + lx * cos - ly * sin;
                let py = by + dy + lx * sin + ly * cos;
                if depth(&sh.crown, px, py) + half(t) + 2.0 > 0.0 {
                    return false;
                }
            }
            true
        };
        let mut lo = 0.0;
        let mut hi = 0.0;
        if !inside(0.0) {
            hi = 40.0;
            for _ in 0..12 {
                let mid = (lo + hi) / 2.0;
                if inside(mid) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
        }
        let transform = format!(
            "translate({} {}) rotate({})",
            f2(bx),
            f2(by + hi),
            f2(side * btilt)
        );
        let key = format!("brow{}", i);
        brows.push(if wedge {
            let dm = ((tl + tr) / 2.0) * 0.75;
            n(
                "path",
                &key,
                vec![
                    (
                        "d",
                        format!(
                            "M{} {}Q0 {} {} {}L{} {}Q0 {} {} {}Z",
                            f2(-hw),
                            f2(-tl / 2.0),
                            f2(c - dm),
                            f2(hw),
                            f2(-tr / 2.0),
                            f2(hw),
                            f2(tr / 2.0),
                            f2(c + dm),
                            f2(-hw),
                            f2(tl / 2.0)
                        )
                        .into(),
                    ),
                    ("transform", transform.into()),
                    ("fill", tables::ink("ink").into()),
                    ("stroke", tables::ink("ink").into()),
                    ("stroke-linejoin", "round".into()),
                    ("stroke-width", f2(to).into()),
                ],
                vec![],
            )
        } else {
            n(
                "path",
                &key,
                vec![
                    (
                        "d",
                        format!("M{} 0Q0 {} {} 0", f2(-w), f2(ca), f2(w)).into(),
                    ),
                    ("transform", transform.into()),
                    ("fill", "none".into()),
                    ("stroke", tables::ink("ink").into()),
                    ("stroke-linecap", "round".into()),
                    ("stroke-linejoin", "round".into()),
                    ("stroke-width", f2(bw * k2).into()),
                ],
                vec![],
            )
        });
    }
    // mouth
    let m = &sh.mouth;
    let mw = m.w * 0.05f64.max(g("mw"));
    let mt = g("mt") * m.d;
    let mb = g("mb") * m.d;
    let kb = 4.0 / 3.0;
    let w7 = mw * 0.72;
    // line: one open stroke along the middle of the lips, so it stays a line
    // in every state: a smile, a frown, or flat where the others open
    let mid = ((mt + mb) / 2.0) * kb;
    let d = match who.mouth.as_str() {
        "poly" => format!("M{} 0L0 {}L{} 0L0 {}Z", f2(-mw), f2(mt), f2(mw), f2(mb)),
        "box" => format!(
            "M{} 0L{} {}L{} {}L{} 0L{} {}L{} {}Z",
            f2(-mw),
            f2(-w7),
            f2(mb),
            f2(w7),
            f2(mb),
            f2(mw),
            f2(w7),
            f2(mt),
            f2(-w7),
            f2(mt)
        ),
        "line" => format!(
            "M{} 0C{} {} {} {} {} 0",
            f2(-mw),
            f2(-mw * 0.5),
            f2(mid),
            f2(mw * 0.5),
            f2(mid),
            f2(mw)
        ),
        _ => format!(
            "M{} 0C{} {} {} {} {} 0C{} {} {} {} {} 0Z",
            f2(-mw),
            f2(-mw),
            f2(mb * kb),
            f2(mw),
            f2(mb * kb),
            f2(mw),
            f2(mw),
            f2(mt * kb),
            f2(-mw),
            f2(mt * kb),
            f2(-mw)
        ),
    };
    let open = who.mouth == "line";
    let mouth = n(
        "path",
        "mouth",
        vec![
            ("d", d.into()),
            (
                "transform",
                format!(
                    "translate({} {}) rotate({})",
                    f2(m.x + g("mx")),
                    f2(m.y + g("my")),
                    f2(g("mk"))
                )
                .into(),
            ),
            ("fill", if open { "none" } else { tables::ink("ink") }.into()),
            ("stroke", tables::ink("ink").into()),
            ("stroke-linecap", "round".into()),
            ("stroke-linejoin", "round".into()),
            ("stroke-width", f2(if open { sw } else { m.sw * st }).into()),
        ],
        vec![],
    );
    // cheeks
    let blush = clamp(g("blush"), 0.0, 1.0);
    let mut cheeks: Vec<SNode> = Vec::new();
    for (i, cc) in sh.cheeks.iter().enumerate() {
        let cx = cc[0];
        let cy = cc[1];
        let key = format!("cheek{}", i);
        let side_s: f64 = if i == 0 { -1.0 } else { 1.0 };
        let at: Vec<(&str, AttrVal)> = vec![
            ("opacity", f2(blush).into()),
            (
                "transform",
                format!(
                    "translate({} {}) scale({}) translate({} {})",
                    f2(cx + side_s * spread * 0.6),
                    f2(cy - lower * 4.0),
                    f2(0.6 + 0.4 * blush),
                    js_num_to_string(-cx),
                    js_num_to_string(-cy)
                )
                .into(),
            ),
        ];
        cheeks.push(match who.cheeks.as_str() {
            "dots" => {
                let r = sh.cry * 0.9;
                let mut attrs = at;
                attrs.push(("fill", deep.into()));
                n(
                    "g",
                    &key,
                    attrs,
                    vec![
                        n(
                            "circle",
                            &format!("{}.0", key),
                            vec![
                                ("cx", f2(cx - sh.crx * 1.05).into()),
                                ("cy", js_num_to_string(cy).into()),
                                ("r", f2(r).into()),
                            ],
                            vec![],
                        ),
                        n(
                            "circle",
                            &format!("{}.1", key),
                            vec![
                                ("cx", js_num_to_string(cx).into()),
                                ("cy", f2(cy + sh.cry * 0.5).into()),
                                ("r", f2(r).into()),
                            ],
                            vec![],
                        ),
                        n(
                            "circle",
                            &format!("{}.2", key),
                            vec![
                                ("cx", f2(cx + sh.crx * 1.05).into()),
                                ("cy", js_num_to_string(cy).into()),
                                ("r", f2(r).into()),
                            ],
                            vec![],
                        ),
                    ],
                )
            }
            "lines" => {
                let h = sh.cry * 1.3;
                let mut attrs = at;
                attrs.push(("fill", "none".into()));
                attrs.push(("stroke", deep.into()));
                attrs.push(("stroke-linecap", "round".into()));
                attrs.push(("stroke-width", f2(sh.cry * 0.78).into()));
                n(
                    "g",
                    &key,
                    attrs,
                    [-1, 0, 1]
                        .iter()
                        .map(|j| {
                            let x0 = cx + *j as f64 * sh.crx * 0.8;
                            n(
                                "path",
                                &format!("{}.{}", key, j + 1),
                                vec![(
                                    "d",
                                    format!(
                                        "M{} {}L{} {}",
                                        f2(x0 + h * 0.5),
                                        f2(cy - h),
                                        f2(x0 - h * 0.5),
                                        f2(cy + h)
                                    )
                                    .into(),
                                )],
                                vec![],
                            )
                        })
                        .collect(),
                )
            }
            _ => {
                let mut attrs: Vec<(&str, AttrVal)> = vec![
                    ("cx", cx.into()),
                    ("cy", cy.into()),
                    ("rx", sh.crx.into()),
                    ("ry", sh.cry.into()),
                    ("fill", deep.into()),
                ];
                attrs.extend(at);
                n("ellipse", &key, attrs, vec![])
            }
        });
    }
    // signature trait: each one keeps its v1.2 motion, measured from a seat on
    // the crown outline instead of a spot on its old face
    let hair = g("hair");
    let lag = pose.get("lag");
    let (trait_t, shape): (String, SNode) = match who.trait_.as_str() {
        "square" => {
            let s = seat(&sh.crown, 22.0 + hair * 10.0, 0.0);
            (
                format!(
                    "translate({} {}) rotate({})",
                    f2(s.x),
                    f2(s.y + lag),
                    f2(s.a + 22.0 + hair * 23.0)
                ),
                n(
                    "rect",
                    "trait.shape",
                    vec![
                        ("x", (-11).into()),
                        ("y", (-11).into()),
                        ("width", 22.into()),
                        ("height", 22.into()),
                        ("fill", deep.into()),
                    ],
                    vec![],
                ),
            )
        }
        "fin" => {
            let s = seat(&sh.crown, -6.0, -164.0 * 22.0 * RAD);
            let r = s.a + (if hair >= 0.0 { hair * 20.0 } else { hair * 55.0 }) - lag * 0.8;
            (
                format!("translate({} {}) rotate({})", f2(s.x), f2(s.y + lag * 0.25), f2(r)),
                n(
                    "path",
                    "trait.shape",
                    vec![("d", "M0 0L0 -40A40 40 0 0 1 40 0Z".into()), ("fill", deep.into())],
                    vec![],
                ),
            )
        }
        "ring" => {
            // worn at one o'clock; rides up when excited, slides when sleepy
            let b = 35.0 - hair * 14.0 - (if hair < 0.0 { hair * -21.0 } else { 0.0 }) + lag * 0.9;
            let w = ring_walk(sh);
            let s = seat(&sh.crown, 8.0, w.1.max(w.0 + 158.0 * (b - 35.0) * RAD));
            (
                format!(
                    "translate({} {}) scale({})",
                    f2(s.x),
                    f2(s.y),
                    f2(1.0 + 0.0f64.max(hair) * 0.1)
                ),
                n(
                    "circle",
                    "trait.shape",
                    vec![
                        ("cx", 0.into()),
                        ("cy", 0.into()),
                        ("r", 12.5.into()),
                        ("fill", "none".into()),
                        ("stroke", deep.into()),
                        (
                            "stroke-width",
                            f2(7.0 * if pose.expression == "angry" { 1.25 } else { 1.0 }).into(),
                        ),
                    ],
                    vec![],
                ),
            )
        }
        "dot" => {
            // rests on the top; lifts clear when perky, rolls over and down the
            // right slope when low
            let ss = clamp(-hair, 0.0, 1.2);
            let th = 1.0f64.min(ss / 0.35) * 61.05 * RAD;
            let travel = (0.0f64.max(ss - 0.35) / 0.65) * 92.0;
            let p = seat(&sh.crown, 12.5, 26.1 * th + travel);
            let mut cy = p.y;
            if hair > 0.0 {
                cy -= hair * 18.0;
            }
            cy += lag * if ss > 0.35 { 0.3 } else { 1.0 };
            (
                format!(
                    "translate({} {}) scale({})",
                    f2(p.x),
                    f2(cy),
                    f2(1.0 + 0.0f64.max(hair) * 0.1)
                ),
                n(
                    "circle",
                    "trait.shape",
                    vec![
                        ("cx", 0.into()),
                        ("cy", 0.into()),
                        ("r", 12.5.into()),
                        ("fill", deep.into()),
                    ],
                    vec![],
                ),
            )
        }
        _ => {
            // peak: the mark's triangle worn on the crown; lifts when perky, keels
            // over sideways when low
            let s = seat(&sh.crown, 4.0 + 0.0f64.max(hair) * 9.0, 0.0);
            let r = s.a + (if hair < 0.0 { -hair * 38.0 } else { 0.0 }) - lag * 0.5;
            (
                format!(
                    "translate({} {}) rotate({}) scale({})",
                    f2(s.x),
                    f2(s.y + lag),
                    f2(r),
                    f2(1.0 + 0.0f64.max(hair) * 0.1)
                ),
                n(
                    "path",
                    "trait.shape",
                    vec![
                        ("d", "M0 -27L13 -3L-13 -3Z".into()),
                        ("fill", deep.into()),
                        ("stroke", deep.into()),
                        ("stroke-width", 6.into()),
                        ("stroke-linejoin", "round".into()),
                    ],
                    vec![],
                ),
            )
        }
    };
    let trait_node = n("g", "trait", vec![("transform", trait_t.into())], vec![shape]);
    // the plates sit a hair out of register at large sizes
    let off = |dx: f64, dy: f64| -> Option<String> {
        if riso {
            Some(format!("translate({} {})", f2(dx * k * upp), f2(dy * k * upp)))
        } else {
            None
        }
    };
    let mut plate_a_children = vec![body_node];
    plate_a_children.extend(cheeks);
    plate_a_children.push(trait_node);
    let mut plate_b_children = eyes;
    plate_b_children.extend(brows);
    plate_b_children.push(mouth);
    let av = n(
        "g",
        "av",
        vec![
            (
                "transform",
                format!(
                    "translate({} {}) rotate({} 170 {}) translate(170 {}) scale({} {}) translate(-170 {})",
                    f2(x),
                    f2(ty),
                    f2(rot),
                    js_num_to_string(b_),
                    js_num_to_string(b_),
                    js_num_to_string(sc(sx)),
                    js_num_to_string(sc(sy)),
                    js_num_to_string(-b_)
                )
                .into(),
            ),
            (
                "filter",
                if dim > 0.01 {
                    Some(format!("url(#{}-dim)", id))
                } else {
                    None
                }
                .into(),
            ),
        ],
        vec![
            n(
                "g",
                "plateA",
                vec![
                    ("transform", off(-0.55, 0.45).into()),
                    (
                        "filter",
                        if riso {
                            Some(format!("url(#{}-pA)", id))
                        } else {
                            None
                        }
                        .into(),
                    ),
                ],
                plate_a_children,
            ),
            n(
                "g",
                "plateB",
                vec![
                    ("transform", off(0.6, -0.5).into()),
                    (
                        "filter",
                        if riso {
                            Some(format!("url(#{}-pB)", id))
                        } else {
                            None
                        }
                        .into(),
                    ),
                ],
                plate_b_children,
            ),
        ],
    );
    // the thing in the corner: a dot lattice from the mark's vocabulary
    let mut scene: Vec<SNode> = Vec::new();
    if o.frame != "none" {
        let bg = match o.frame.as_str() {
            "ink" => tables::ink("ink"),
            "bone" => tables::ink("bone"),
            _ => tables::ink("paper"),
        };
        scene.push(n(
            "rect",
            "bg",
            vec![
                ("x", VB[0].into()),
                ("y", VB[1].into()),
                ("width", VB[2].into()),
                ("height", VB[3].into()),
                ("fill", bg.into()),
            ],
            vec![],
        ));
        let th = clamp(g("thing"), 0.0, 1.0);
        let mut dots: Vec<SNode> = Vec::new();
        for j in 0..=2 {
            for i in 0..=2 {
                let q = j * 3 + i;
                let kk = smooth(q as f64 / 12.0, q as f64 / 12.0 + 0.35, th);
                dots.push(n(
                    "circle",
                    &format!("lattice.{}", q),
                    vec![
                        ("cx", (312 + i * 18).into()),
                        ("cy", (2 + j * 18).into()),
                        ("r", f2(1.2 + 2.2 * kk).into()),
                        ("opacity", f3(kk * 0.55).into()),
                    ],
                    vec![],
                ));
            }
        }
        scene.push(n(
            "g",
            "lattice",
            vec![(
                "fill",
                if o.frame == "ink" {
                    tables::ink("bone")
                } else {
                    tables::ink("ink")
                }
                .into(),
            )],
            dots,
        ));
    }
    scene.push(av);
    if grainy {
        scene.push(n(
            "rect",
            "grain",
            vec![
                ("x", VB[0].into()),
                ("y", VB[1].into()),
                ("width", VB[2].into()),
                ("height", VB[3].into()),
                ("filter", format!("url(#{}-grain)", id).into()),
                ("opacity", if o.frame == "ink" { 0.3 } else { 0.16 }.into()),
                ("pointer-events", "none".into()),
            ],
            vec![],
        ));
    }
    let root = n(
        "svg",
        "root",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg".into()),
            (
                "viewBox",
                VB.iter()
                    .map(|v| js_num_to_string(*v))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .into(),
            ),
            ("width", o.size.into()),
            ("height", o.size.into()),
            (
                "role",
                if o.title.is_none() {
                    None
                } else {
                    Some("img")
                }
                .into(),
            ),
            ("aria-label", o.title.clone().into()),
            (
                "aria-hidden",
                if o.title.is_none() {
                    Some("true")
                } else {
                    None
                }
                .into(),
            ),
        ],
        vec![
            n("defs", "defs", vec![], defs),
            n(
                "g",
                "frame",
                vec![("clip-path", format!("url(#{}-f)", id).into())],
                scene,
            ),
        ],
    );
    if o.live {
        root
    } else {
        prune(&root).unwrap_or(root)
    }
}

//! Bloc, from the Bloc sheet (direction A, palette P2): every face rebuilt
//! from a square, a circle and a bar. The same four silhouettes, the body cut
//! in two down the middle (strong on the left, light on the right), one ink
//! bar for both brows, and the trait as a block over the upper right outline.
//! No round corners, no gradients, no outlines.
//!
//! Expressions come from the same channels as peek, read as three moulds:
//! round eyes and a whole bar at rest, a level cut from below or above for
//! the smiling and the drowsy, a tilted cut and a bar broken in two for the
//! angry, the sad and the confused. The bar is always two halves, so it
//! breaks and mends continuously when animated.

use crate::draw::{clamp, depth, seat, smooth, Pose};
use crate::identity::Identity;
use crate::js::f2;
use crate::scene::{n, SNode};
use crate::svg::DrawOpts;
use crate::tables::{self, CrownDef, FaceBody};

const RAD: f64 = std::f64::consts::PI / 180.0;

/* Peek's silhouettes; the triangle's round joins are cut sharp and it grows
 * to the 300 box the circle and the diamond fill, its base unmoved. */
const TRI: &[&[f64]] = &[&[170.0, 12.8], &[320.0, 312.8], &[20.0, 312.8]];

fn bloc_body(face: &str) -> FaceBody {
    match face {
        "triangle" => FaceBody { d: Some("M170 12.8L320 312.8L20 312.8Z"), circle: None, stroke: None },
        _ => {
            let b = &tables::face(face).body;
            FaceBody { d: b.d, circle: b.circle, stroke: b.stroke }
        }
    }
}

fn bloc_crown(face: &str) -> CrownDef {
    match face {
        "triangle" => CrownDef { poly: TRI, r: 0.0 },
        _ => {
            let c = &tables::face(face).crown;
            CrownDef { poly: c.poly, r: c.r }
        }
    }
}

// Bloc's own ramps: lines thicken 2.6x below 30px, eyes and trait grow
fn bloc_stroke_for(px: f64) -> f64 {
    if px >= 96.0 {
        1.0
    } else if px >= 56.0 {
        1.25
    } else if px >= 40.0 {
        1.6
    } else if px >= 30.0 {
        2.0
    } else {
        2.6
    }
}

fn bloc_boost_for(px: f64) -> f64 {
    if px < 30.0 {
        1.25
    } else if px < 56.0 {
        1.12
    } else {
        1.0
    }
}

/// A point turned by `deg` about the origin, y down.
fn turn(x: f64, y: f64, deg: f64) -> [f64; 2] {
    let c = (deg * RAD).cos();
    let s = (deg * RAD).sin();
    [x * c - y * s, x * s + y * c]
}

fn poly(pts: &[[f64; 2]]) -> String {
    let mut out = String::from("M");
    for (i, pt) in pts.iter().enumerate() {
        if i > 0 {
            out.push('L');
        }
        out.push_str(&format!("{} {}", f2(pt[0]), f2(pt[1])));
    }
    out.push('Z');
    out
}

pub fn bloc_parts(
    who: &Identity,
    pose: &Pose,
    o: &DrawOpts,
    defs: &mut Vec<SNode>,
    g: &dyn Fn(&str) -> f64,
) -> (Vec<SNode>, Vec<SNode>) {
    let sh = tables::face(&who.face);
    let body = bloc_body(&who.face);
    let crown = bloc_crown(&who.face);
    let id = &o.id;
    let p = &who.persona;
    let spread = p.spread;
    let st = bloc_stroke_for(o.size);
    let up = bloc_boost_for(o.size);
    let strong = tables::bloc_ink(&who.color);
    // the light half goes deep on light grounds, so the body never sinks in
    let light = if o.frame == "bone" || o.frame == "paper" {
        tables::color(&who.color).deep
    } else {
        tables::color(&who.color).body
    };
    let line_w = sh.brow.w * st;

    // the body: one block, its right half cut out in the light ink
    defs.push(n(
        "clipPath",
        "half",
        vec![("id", format!("{}-r", id).into())],
        vec![n(
            "rect",
            "half.shape",
            vec![
                ("x", 170.into()),
                ("y", (-30).into()),
                ("width", 200.into()),
                ("height", 400.into()),
            ],
            vec![],
        )],
    ));
    let block = |key: &str, fill: &str, clip: Option<String>| -> SNode {
        if let Some(d) = body.d {
            n(
                "path",
                key,
                vec![
                    ("d", d.into()),
                    ("fill", fill.into()),
                    ("clip-path", clip.into()),
                ],
                vec![],
            )
        } else {
            let c = body.circle.expect("face body needs d or circle");
            n(
                "circle",
                key,
                vec![
                    ("cx", c[0].into()),
                    ("cy", c[1].into()),
                    ("r", c[2].into()),
                    ("fill", fill.into()),
                    ("clip-path", clip.into()),
                ],
                vec![],
            )
        }
    };

    // eyes: a paper disc with its pupil, a solid bead, or a rimmed ring
    let eye_s = g("eyeS");
    let r0 = (sh.rx * sh.ry).sqrt() * up;
    let lid0 = g("lid");
    let lid_asym = g("lidAsym");
    let lower = clamp(g("lower"), 0.0, 1.0);
    let pupil = g("pupil");
    let gx = clamp(g("gx"), -1.1, 1.1);
    let gy = clamp(g("gy"), -1.1, 1.1);
    let tilt = g("lidTilt");
    let bead = who.eyes == "bead";
    let mut eyes: Vec<SNode> = Vec::new();
    for (i, e) in sh.eyes.iter().enumerate() {
        let ex = e[0];
        let ey = e[1];
        let side: f64 = if i == 0 { -1.0 } else { 1.0 };
        let kk = format!("eye{}", i);
        let mut lid = clamp(
            lid0 + if i == 0 { lid_asym } else { -lid_asym * 0.3 },
            0.0,
            1.0,
        );
        lid = lid + (1.0 - lid) * pose.get("blink");
        // peek rests its lid a tenth down; a Bloc eye rests whole
        let cut = clamp((lid - 0.1) / 0.9, 0.0, 1.0);
        let r = if bead {
            r0 * 0.7 * eye_s * (0.4 + 0.6 * pupil)
        } else {
            r0 * eye_s
        };
        let (tx, ty) = if bead {
            (ex + side * spread + gx * 6.0, ey + gy * 6.0)
        } else {
            (ex + side * spread, ey)
        };
        // the cuts: a straight lid from above, tilted, and a level one below
        let a = side * tilt;
        let edge = -r - 1.0 + cut * (2.0 * r + 2.0);
        let floor = r + 1.0 - lower * r * 2.2;
        let w_ = r + 2.0;
        let lid_at = |x: f64| x * (a * RAD).tan() + edge / (a * RAD).cos();
        let t0 = lid_at(-w_);
        let t1 = lid_at(w_);
        defs.push(n(
            "clipPath",
            &format!("{}.clip", kk),
            vec![("id", format!("{}-e{}", id, i).into())],
            vec![n(
                "path",
                &format!("{}.clip.shape", kk),
                vec![(
                    "d",
                    poly(&[
                        [-w_, t0],
                        [w_, t1],
                        [w_, floor.max(t1)],
                        [-w_, floor.max(t0)],
                    ])
                    .into(),
                )],
                vec![],
            )],
        ));
        let pr = sh.pr * pupil * up;
        let px = gx * (0.0f64).max(r - pr) * 0.85;
        let py = gy * (0.0f64).max(r - pr) * 0.75;
        let rim = (r * 0.27 * st.min(1.6)).min(r * 0.4);
        let mut inner: Vec<SNode> = Vec::new();
        if bead {
            inner.push(n(
                "circle",
                &format!("{}.bead", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("r", f2(r).into()),
                    ("fill", tables::ink("ink").into()),
                ],
                vec![],
            ));
        } else {
            inner.push(n(
                "circle",
                &format!("{}.white", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("r", f2(r).into()),
                    ("fill", tables::ink("paper").into()),
                ],
                vec![],
            ));
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
        }
        if who.eyes == "ring" {
            inner.push(n(
                "circle",
                &format!("{}.rim", kk),
                vec![
                    ("cx", 0.into()),
                    ("cy", 0.into()),
                    ("r", f2(r - rim / 2.0).into()),
                    ("fill", "none".into()),
                    ("stroke", tables::ink("ink").into()),
                    ("stroke-width", f2(rim).into()),
                ],
                vec![],
            ));
        }
        // closed: a bar where the eye was
        let co = smooth(0.86, 1.0, lid);
        let yy = edge.min(r) - line_w / 2.0;
        eyes.push(n(
            "g",
            &kk,
            vec![("transform", format!("translate({} {})", f2(tx), f2(ty)).into())],
            vec![
                n(
                    "g",
                    &format!("{}.inner", kk),
                    vec![("clip-path", format!("url(#{}-e{})", id, i).into())],
                    inner,
                ),
                n(
                    "path",
                    &format!("{}.closed", kk),
                    vec![
                        ("d", format!("M{} {}H{}", f2(-r * 0.95), f2(yy), f2(r * 0.95)).into()),
                        ("transform", format!("rotate({})", f2(a * 0.6)).into()),
                        ("stroke", tables::ink("ink").into()),
                        ("stroke-width", f2(line_w * 1.4).into()),
                        (
                            "opacity",
                            if co > 0.01 {
                                f2(co).into()
                            } else {
                                crate::scene::AttrVal::from(0)
                            },
                        ),
                    ],
                    vec![],
                ),
            ],
        ));
    }

    // the bar: both brows in one, from one eye's outer edge to the other's.
    // The persona slants it; the expression's tilt breaks it into a V or a
    // caret; asymmetry lifts its right end. It never leaves the body.
    let bt = pose.get("browTilt");
    let basym = g("browAsym");
    let th = line_w * 2.0 * g("browW");
    let mul_eye_s = p
        .mul
        .iter()
        .find(|(k, _)| k == "eyeS")
        .map(|(_, v)| *v)
        .unwrap_or(1.0);
    let hw = sh.eyes[1][0] - 170.0 + spread + (sh.rx * sh.ry).sqrt() * mul_eye_s * up;
    let gap = smooth(0.0, 8.0, bt.abs()) * th * 0.9;
    let add_brow_tilt = p
        .add
        .iter()
        .find(|(k, _)| k == "browTilt")
        .map(|(_, v)| *v)
        .unwrap_or(0.0);
    let slant = add_brow_tilt * 0.3 - ((1.3 * basym) / (2.0 * hw)).atan() / RAD;
    let y0 = sh.brow.y + g("browY") - (eye_s - 1.0) * sh.ry * 0.9 - 0.35 * basym;
    // `len` shortens the bar from its outer ends, 1 is full length
    let halves = |dy: f64, len: f64| -> Vec<[[f64; 2]; 4]> {
        [-1.0f64, 1.0]
            .iter()
            .map(|s| {
                let s = *s;
                let o2 = turn((s * gap) / 2.0, 0.0, slant);
                let x0 = 170.0 + o2[0];
                let yy = y0 + dy + o2[1];
                let ang = slant + s * bt;
                let u = turn(s * (hw * len - gap / 2.0), 0.0, ang);
                let nrm = turn(0.0, th / 2.0, ang);
                [
                    [x0 - nrm[0], yy - nrm[1]],
                    [x0 + u[0] - nrm[0], yy + u[1] - nrm[1]],
                    [x0 + u[0] + nrm[0], yy + u[1] + nrm[1]],
                    [x0 + nrm[0], yy + nrm[1]],
                ]
            })
            .collect()
    };
    let inside = |dy: f64, len: f64| -> bool {
        halves(dy, len)
            .iter()
            .all(|q| q.iter().all(|pt| depth(&crown, pt[0], pt[1]) + 2.0 < 0.0))
    };
    // pushed down until it fits; if even that fails, it grows shorter
    let mut lo = 0.0;
    let mut hi = 0.0;
    if !inside(0.0, 1.0) {
        hi = 60.0;
        for _ in 0..12 {
            let mid = (lo + hi) / 2.0;
            if inside(mid, 1.0) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
    }
    let mut len = 1.0;
    if !inside(hi, 1.0) {
        let mut short = 0.0;
        for _ in 0..12 {
            let mid = (short + len) / 2.0;
            if inside(hi, mid) {
                short = mid;
            } else {
                len = mid;
            }
        }
        len = short;
    }
    let bars: Vec<SNode> = halves(hi, len)
        .iter()
        .enumerate()
        .map(|(i, q)| {
            n(
                "path",
                &format!("bar{}", i),
                vec![
                    ("d", poly(q).into()),
                    ("fill", tables::ink("ink").into()),
                ],
                vec![],
            )
        })
        .collect();

    // mouth: poly a turned square, round a half disc, line a bar, box a
    // block. Diamonds and triangles get a bigger one, it vanished at 16px.
    let m = &sh.mouth;
    let big = (if who.face == "diamond" || who.face == "triangle" {
        1.25
    } else {
        1.0
    }) * up;
    let mw = m.w * (0.05f64).max(g("mw")) * big;
    let mt = g("mt") * m.d * big;
    let mb = g("mb") * m.d * big;
    let mid = (mt + mb) / 2.0;
    let kb = 4.0 / 3.0;
    let d: String = match who.mouth.as_str() {
        "poly" => {
            let s2 = (mw * 0.9).max((mb - mt) / 2.0);
            poly(&[[-s2, mid], [0.0, mid - s2], [s2, mid], [0.0, mid + s2]])
        }
        "box" => {
            let h = ((mb - mt).max(line_w * 1.6)) / 2.0;
            let w = mw * 0.85;
            poly(&[[-w, mid - h], [w, mid - h], [w, mid + h], [-w, mid + h]])
        }
        "line" => {
            let h = (line_w * 1.4) / 2.0;
            let w = mw * 1.5;
            let y = mid * 0.5;
            poly(&[[-w, y - h], [w, y - h], [w, y + h], [-w, y + h]])
        }
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
            ("fill", tables::ink("ink").into()),
        ],
        vec![],
    );

    // the trait, as a block over the upper right outline. Perky lifts it
    // clear, low slides it down the side.
    let hair = g("hair");
    let s = seat(&crown, (0.0f64).max(hair) * 8.0, 110.0 + (0.0f64).max(-hair) * 40.0);
    let shape: SNode = match who.trait_.as_str() {
        "square" => n(
            "rect",
            "trait.shape",
            vec![
                ("x", (-21).into()),
                ("y", (-21).into()),
                ("width", 42.into()),
                ("height", 42.into()),
            ],
            vec![],
        ),
        "dot" => n(
            "circle",
            "trait.shape",
            vec![("cx", 0.into()), ("cy", 0.into()), ("r", 22.into())],
            vec![],
        ),
        "ring" => n(
            "circle",
            "trait.shape",
            vec![
                ("cx", 0.into()),
                ("cy", 0.into()),
                ("r", 17.5.into()),
                ("fill", "none".into()),
                ("stroke", strong.into()),
                ("stroke-width", 13.into()),
            ],
            vec![],
        ),
        "fin" => n(
            "path",
            "trait.shape",
            vec![("d", "M-27 27V-27A54 54 0 0 1 27 27Z".into())],
            vec![],
        ),
        _ => n(
            "rect",
            "trait.shape",
            vec![
                ("x", (-16).into()),
                ("y", (-30).into()),
                ("width", 32.into()),
                ("height", 60.into()),
            ],
            vec![],
        ),
    };
    let trait_node = n(
        "g",
        "trait",
        vec![
            (
                "transform",
                format!(
                    "translate({} {}) scale({})",
                    f2(s.x),
                    f2(s.y + pose.get("lag")),
                    f2(up)
                )
                .into(),
            ),
            ("fill", strong.into()),
        ],
        vec![shape],
    );

    let front = vec![
        block("body", strong, None),
        block("body.light", light, Some(format!("url(#{}-r)", id))),
        trait_node,
    ];
    let mut back = eyes;
    back.extend(bars);
    back.push(mouth);
    (front, back)
}

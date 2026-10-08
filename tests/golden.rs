//! Golden tests: the Rust port must reproduce peek-vanilla's SVGs byte-for-byte.
#[path = "corpus_data.rs"]
mod corpus_data;

use sakayori_peek::{identify, to_svg, PeekOptions};

fn opts_for(c: &corpus_data::GoldenCase) -> PeekOptions {
    PeekOptions {
        face: c.face.map(|s| s.to_string()),
        color: c.color.map(|s| s.to_string()),
        eyes: c.eyes.map(|s| s.to_string()),
        brows: c.brows.map(|s| s.to_string()),
        mouth: c.mouth.map(|s| s.to_string()),
        cheeks: c.cheeks.map(|s| s.to_string()),
        trait_: c.trait_.map(|s| s.to_string()),
        expression: c.expression.to_string(),
        gaze: c.gaze,
        size: c.size,
        frame: c.frame.to_string(),
        square: c.square,
        riso: c.riso,
        id: c.id.map(|s| s.to_string()),
        title: c.title.map(|s| s.to_string()),
        hide_title: c.hide_title,
        version: 1,
    }
}

#[test]
fn golden_corpus() {
    let mut fails = 0;
    for (i, c) in corpus_data::CORPUS.iter().enumerate() {
        let actual = to_svg(c.name, &opts_for(c));
        if actual != c.svg {
            fails += 1;
            if fails <= 5 {
                let pos = actual
                    .char_indices()
                    .zip(c.svg.char_indices())
                    .find(|((_, a), (_, b))| a != b)
                    .map(|((p, _), _)| p);
                println!(
                    "MISMATCH case {} name={:?} first diff at {:?} (lengths {} vs {})",
                    i,
                    c.name,
                    pos,
                    actual.len(),
                    c.svg.len()
                );
                if let Some(p) = pos {
                    let a0 = p.saturating_sub(60);
                    println!("  actual  : {:?}", &actual[a0..(p + 60).min(actual.len())]);
                    println!("  expected: {:?}", &c.svg[a0..(p + 60).min(c.svg.len())]);
                }
            }
        }
    }
    assert_eq!(fails, 0, "{} cases mismatched", fails);
}

#[test]
fn determinism() {
    // the same name always yields the same bytes, on any machine
    let a = to_svg("Sakayori", &PeekOptions::default());
    let b = to_svg("Sakayori", &PeekOptions::default());
    assert_eq!(a, b);
    assert!(a.starts_with("<svg "));
}

#[test]
fn identify_is_stable() {
    let x = identify("Nguyễn Văn An", 1);
    let y = identify("Nguyễn Văn An", 1);
    assert_eq!(x, y);
    assert_eq!(x.face, y.face);
}

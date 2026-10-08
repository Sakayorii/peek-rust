# peek-rust

Rust port of [peek-vanilla](https://github.com/Sakayorii/peek-vanilla) — itself a
verified 1:1 port of [Peek](https://github.com/doan-labs/peek) by Doan Labs (MIT).
A name in, a face out: deterministic avatars, no network, no database.

Part of the peek family: `peek` (original, React) · `peek-vanilla` (JS) ·
`peek-kotlin` (JVM) · `peek-rust` (Rust).

```
src/
├── js.rs       — JavaScript semantics, exactly (number printing, tidy, JSON)
├── identity.rs — tidy / FNV-1a / mulberry32 / identify
├── tables.rs   — faces, colors, parts, expressions (generated from the Kotlin port's tables)
├── scene.rs    — the node tree draw() builds
├── draw.rs     — one pure geometry function (port of draw.js)
├── svg.rs      — settle / serialize / to_svg (port of svg.js)
└── lib.rs
```

## Use

```rust
use sakayori_peek::{to_svg, PeekOptions};

let svg = to_svg("Sakayori", &PeekOptions::default());
// or with options:
let svg = to_svg("Sakayori", &PeekOptions {
    expression: "happy".to_string(),
    size: 128.0,
    ..Default::default()
});
```

## Verification

The port is verified byte-identical against peek-vanilla on a 445-case golden
corpus: curated names (unicode, emoji, whitespace, NFC, control chars, XML
escapes), all 11 expressions, every axis override, sizes, frames, riso, titles,
explicit ids, gaze, plus 300 seeded fuzz names.

```sh
node gen-corpus-rs.mjs   # regenerate tests/corpus_data.rs from peek-vanilla
cargo test               # 445 golden cases + determinism checks
```

One deliberate difference from the JS: a Rust `&str` cannot hold lone
surrogates, so that corpus case uses the U+FFFD-sanitized name on both sides —
the same thing `TextEncoder` feeds the hash on the JS side.

## License

MIT — see [LICENSE](LICENSE). Original Peek by Doan Labs.

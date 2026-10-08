//! peek-rust: deterministic avatars for Rust. A name in, a face out.
//!
//! Rust port of [peek-vanilla](https://github.com/Sakayorii/peek-vanilla) —
//! itself a verified 1:1 port of [Peek](https://github.com/doan-labs/peek) by
//! Doan Labs (MIT). Same name, same bytes, on any machine: no network, no
//! database, no RNG outside the seeded one.
//!
//! Part of the peek family: `peek` (original, React) · `peek-vanilla` (JS) ·
//! `peek-kotlin` (JVM) · `peek-rust` (Rust).

pub mod draw;
pub mod identity;
pub mod js;
pub mod scene;
pub mod svg;
pub mod tables;

pub use draw::{draw, rest_pose, Pose};
pub use identity::{identify, Identity};
pub use svg::{to_svg, DrawOpts, PeekOptions};

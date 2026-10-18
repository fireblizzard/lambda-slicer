//! Code copied from Xenoxide (`fireblizzard/halflife-xenoxide` at
//! `33eac809dfd73b0907908f9c63cce5d66efdd2e6`), under this workspace's
//! license by its owner's leave (docs/dependency-audit.md, "Viewer
//! references")
//!
//! The files keep Xenoxide's own comments and layout, so they can be
//! compared with their source, and the comment check skips this folder.
//! Changed here, each marked `Lambda Slicer` where it is not plain
//!
//! - `bsp30` is `crates/xenoxide-bsp30/src`, unchanged
//! - `studio` is `crates/xenoxide-assets/src/studio.rs` and its folder,
//!   reading attachments as well
//! - `studio_pose` is `crates/xenoxide-game/src/studio_pose.rs` and its
//!   tests, reading `studio` here and calling `libm` itself for the sines,
//!   cosines and arc cosines `xenoxide-physics` takes from it

// Left as Xenoxide wrote them, for comparing with the source
#[allow(clippy::style, clippy::complexity, clippy::perf)]
pub mod bsp30;
#[allow(clippy::style, clippy::complexity, clippy::perf)]
pub mod studio;
#[allow(clippy::style, clippy::complexity, clippy::perf)]
pub mod studio_pose;

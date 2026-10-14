# Dependency audit

Toolchain: Rust **1.98.1** (`rust-toolchain.toml`), edition 2024. `Cargo.lock` is committed.

## Direct dependencies

| Crate | Version | License | Use |
|---|---|---|---|
| thiserror | 2.0.21 | MIT OR Apache-2.0 | error types |
| serde | 1.0.229 | MIT OR Apache-2.0 | reports, manifest |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | JSON output |
| blake3 | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | source/output hashes |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | temp output + `persist_noclobber` |
| same-file | 1.0.6 | Unlicense OR MIT | source/destination alias detection |
| clap | 4.6.7 | MIT OR Apache-2.0 | CLI |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | CLI errors |
| proptest (dev) | 1.12.0 | MIT OR Apache-2.0 | malformed-input property tests |

Not yet added (later milestones): `eframe`/`egui` and `rfd` (GUI, M6), `rusqlite` (cache, M5),
`walkdir` (scan, M5), `tracing`.

## Codec decision

No demo codec crate is linked. The container reader/writer in `lambda-slicer-core::container` is an
independent implementation from the documented format. Layout facts were cross-checked against:

- `dem` 0.2.2, `src/types.rs` at `khanghugo/dem` master (LGPL-3.0) — read for field order and
  sizes of `DemoInfo`, `RefParams`, `UserCmd`, `MoveVars`, `Event`, `Sound`. **No code copied.**
  The upstream commit hash was not recorded during planning; record it before adapting any of
  its network-message code in M2.

Every layout claim was then verified by parsing real recordings to their terminal records and by
byte-identical S0b roundtrips (see `engine-validation.md`).

## Open license decision

The workspace is marked `UNLICENSED` until a project license is chosen. If M2 adapts `dem`
(LGPL-3.0) or ReHLDS (GPL-3.0) code rather than reimplementing from behavior, that choice
constrains the project license and must be recorded here first.

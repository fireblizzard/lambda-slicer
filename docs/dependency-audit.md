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
| rusqlite (`bundled`) | 0.40.2 | MIT, bundled SQLite public domain | library cache |
| ctrlc | 3.5.2 | MIT OR Apache-2.0 | CLI scan cancellation |
| eframe | 0.37.0 | MIT OR Apache-2.0 | GUI (egui, harfrust shaping, system font discovery) |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | right-to-left text order in the GUI |
| image (`png`) | 0.25.10 | MIT OR Apache-2.0 | GUI screenshots for checking languages |
| fluent-bundle | 0.16.0 | Apache-2.0 OR MIT | interface catalogs and plurals |
| intl-memoizer | 0.5.3 | Apache-2.0 OR MIT | Fluent number formatter signature |
| unic-langid | 0.9.6 | MIT OR Apache-2.0 | language tags |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 | system language on first run |
| egui_extras | 0.37.0 | MIT OR Apache-2.0 | virtualized library table |
| rfd | 0.17.2 | MIT | native file and folder dialogs |
| chrono (`clock`) | 0.4.45 | MIT OR Apache-2.0 | recording dates in local time |
| fluent-syntax, intl_pluralrules, fluent-langneg, tempfile (dev) | 0.12.0, 7.0.2, 0.13.1, 3.27.0 | Apache-2.0 OR MIT | catalog and settings tests |

Not yet added (later milestones): `tracing`. Discovery uses
`std::fs` instead of `walkdir`, to control which links are followed.

## Codec decision

No demo codec crate is linked. The container reader/writer in `lambda-slicer-core::container` is an
independent implementation from the documented format. Layout facts were cross-checked against:

- `dem` 0.2.2, `src/types.rs` at `khanghugo/dem` master (LGPL-3.0) — read for field order and
  sizes of `DemoInfo`, `RefParams`, `UserCmd`, `MoveVars`, `Event`, `Sound`. **No code copied.**
  The upstream commit hash was not recorded during planning; record it before adapting any of
  its network-message code in M2.

Every layout claim was then verified by parsing real recordings to their terminal records and by
byte-identical S0b roundtrips (see `engine-validation.md`).

### Network layer (M2)

The bit reader, delta decoder, message grammar, and state reconstruction in
`lambda-slicer-core::protocol` and `::state` are also an independent implementation. They were
written from the following references, which were **read for behavior only; no code was copied
or translated line by line**:

| Reference | Revision | License | Used for |
|---|---|---|---|
| ReHLDS engine `sv_main.cpp`, `delta.cpp`, `pr_cmds.cpp` | `550f2d62f13f4ebeb029c1d9d1c212133202611d` | GPL-2.0-or-later | Authoritative writer layouts: serverinfo, resources, clientdata/weapondata, delta header, packet entities, baselines, events, sound, user messages, static entities; delta field encoding and client-side time-window decode |
| ReHLDS HLTV `Server.cpp`, `World.cpp`, `Delta.cpp`, `BitBuffer.cpp` | same | GPL-2.0-or-later | Reader-side cross-check, temp-entity length table, sign-magnitude reads |
| hlviewer.js `src/Replay/*`, `src/BitReader.ts` | `7b002358b6863c2f7e94b531fa9b599a28cefeba` | MIT | Independent reader cross-check |
| Valve Half-Life SDK `common/const.h` | master | Half-Life SDK license | Temp-entity body layouts |

Where references disagreed, the engine writer (what actually produced the bytes) or the SDK
layout was followed; the disagreements are listed in `protocol-profile-5-48.md`. All grammar was
then checked by decoding the local corpus (see the scan there).

The planning document called ReHLDS GPL-3.0; its file headers say GPL version 2 or later.

## Open license decision

The workspace is marked `UNLICENSED` until a project license is chosen. No `dem` (LGPL-3.0) or
ReHLDS (GPL-2.0-or-later) code has been adapted; adapting any would constrain the project license
and must be recorded here first.

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

`intl_pluralrules` 7.0.2, which `fluent-bundle` uses for plurals, is replaced through
`[patch.crates-io]` by a copy in `vendor/intl_pluralrules` with the Lithuanian and Arabic cardinal
rules corrected to CLDR ([vendor](../vendor/README.md)). Same license, same version number.

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

## Viewer references (M8)

The integrated viewer (implementation plan §13) draws on two references, recorded here before any
of their code is used.

| Reference | Revision | License | Use |
|---|---|---|---|
| Xenoxide (`fireblizzard/halflife-xenoxide`, local checkout `C:\Users\naz\source\repos\xenoxide`) | `33eac809dfd73b0907908f9c63cce5d66efdd2e6` | none in the repository. Its owner, who also owns Lambda Slicer, allowed on 2026-10-17 that the modules Lambda Slicer needs be copied under MIT OR Apache-2.0 | WAD3 texture parsing (`xenoxide-assets/src/lib.rs`), the worldspawn WAD list (`xenoxide-map`), and later BSP30 reading, studio models and pose evaluation (`xenoxide-bsp30`, `xenoxide-assets/src/studio`, `xenoxide-game/src/studio_pose.rs`) |
| webhl (`x8BitRain/webhl`) | `1e174d088f80a276079b0293661e0de6f95da0cd` | none declared. A fork of hlviewer.js (MIT, above), whose own changes carry no license | reading only, for how a browser player loads local assets and orders sounds. No code is copied, and its README lists sound-ordering and map-change bugs |

Copies live in `crates/lambda-slicer-assets/src/xenoxide`, whose `mod.rs` lists every file's source
and what changed. They keep Xenoxide's own comments and layout, so they can be compared with their
source and updated from it, and the comment check skips that folder. The only changes so far are
import paths, `libm` called directly for the pose's sines, cosines and arc cosines, studio
attachments read, and an unresolved model's sequences readable before its sequence group files
are, each marked `Lambda Slicer`. Copies are
made only of what a milestone uses. Xenoxide's working tree had uncommitted changes in its
interface crates at that revision, none in the files copied. Its tests came along and pass, the
six that read the installed game included (`XENOXIDE_GAME_DIRECTORY`), among them the stock
player pose matching Xenoxide's golden transforms bit for bit.

| Crate | Version | License | Use |
|---|---|---|---|
| sha2 | 0.10.9 | MIT OR Apache-2.0 | studio model identities, as Xenoxide computes them |
| libm | 0.2.16 | MIT OR Apache-2.0 | the same sines, cosines and arc cosines on every platform, for poses |
| wgpu (`wgsl` only) | 30.0.1 | MIT OR Apache-2.0 | the viewer's drawing, on the device eframe opens, the version egui-wgpu 0.37 uses ([viewer backend](viewer-backend.md)) |
| bytemuck (`derive`) | 1.25.2 | Zlib OR Apache-2.0 OR MIT | vertex and uniform bytes for the GPU |
| cpal | 0.18.2 | Apache-2.0 | sound output to the default device, behind `lambda-slicer-audio`'s `device` feature. On Linux it links the system's ALSA library through `alsa` 0.11.0 (Apache-2.0 OR MIT) and `alsa-sys` 0.4.0 (MIT) |
| pollster, wgpu, image (`png`) (dev) | 0.4.0, 30.0.1, 0.25.10 | Apache-2.0 OR MIT, MIT OR Apache-2.0, MIT OR Apache-2.0 | the headless render bench: a device of its own and a saved frame |

## Project license

Chosen 2026-10-17: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`), like most of the Rust
ecosystem the project builds on. No `dem` (LGPL-3.0) or ReHLDS (GPL-2.0-or-later) code has been
adapted, as the references above were read to learn layouts only. Adapting any would constrain
the license and must be recorded here first. Every shipped dependency is under a license
compatible with MIT OR Apache-2.0 (MIT, Apache-2.0, BSD, ISC, Zlib, BSL-1.0, Unicode-3.0, and
OFL-1.1 and the Ubuntu Font Licence for egui's bundled fonts), and the release carries their
notices ([release](release.md)).

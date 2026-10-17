# Lambda Slicer implementation plan

**Engineering handoff created 14 October 2026; viewer/audio follow-on added 17 October 2026.** Implementation has progressed since the original handoff. Use `README.md` and the milestone validation documents for current completion status; retain the historical observations below as dated evidence.

Build Lambda Slicer as a Rust desktop application with a native GUI, a shared analysis/export core, and a small diagnostic CLI. It will find recorded weapon activity, identify the actor when the recording provides sufficient evidence, and export selected intervals into new GoldSrc `.dem` files. The difficult part is producing an independently playable slice starting midway through a recording: this requires reconstructing protocol and game state, not copying frames between two timestamps. Prove that operation in the installed Windows game **before building the full GUI**. Analysis can be useful earlier, but an analyzer alone does not complete this project.

This document specifies the implementation, staged acceptance gates, and remaining protocol experiments. Statements marked **Observed** were established by read-only inspection or cited source code. **Design** means the chosen implementation approach. **Gate** means work that must produce evidence before the corresponding feature can be called supported. The codec and engine experiments below are necessary implementation tasks, not optional future research.

## 1. Required outcome and boundaries

The user must be able to open or drag/drop a demo, see a timeline and table of different weapons' recorded actions and actors, select one or more intervals, and export new `.dem` files without altering the input. Recursive folder scanning and cached gauss search are first-class features because the local collection contains tens of thousands of files. A clip requested as `1:36–4:48` means the half-open source interval **[96.000, 288.000)**, approximately 192 seconds after documented snapping to recorded frame boundaries. It must start at the selected scene when played with ordinary `playdemo`, using the installed mod/map resources, with no dependence on the original recording.

M0–M7 deliver the timeline, analysis and independent demo-export application. After those milestones, M8–M12 add integrated 3D demo playback with synchronized sound, timeline-controlled seeking, rewind and fast-forward, as specified in §13. Video-file encoding and automatic camera editing remain outside the requested scope. Game playback remains the compatibility and fidelity reference.

The required target includes multiplayer POV recordings with two or more players and singleplayer recordings. Singleplayer is a required fixture and release gate, even if none is found in the supplied corpus. Spectator recordings need explicit actor handling. HLTV is a discovery-driven compatibility extension: recognize it and avoid misdecoding it, then certify it with fixtures if present or requested. Do not silently treat HLTV as ordinary POV.

Honest output is essential. A POV recording does not necessarily contain every remote player's action. Report **“No observed gauss use”**, together with coverage, instead of claiming nobody used gauss. Some weapons have direct shot events; others expose attempts, sustained activity, selection, or indirect evidence. Show those distinctions. Unsupported parsing or export must identify the reason and location rather than guess.

### 1.1 Local paths and observed corpus

These are development inputs, not production constants:

| Purpose | Path |
|---|---|
| Repository | `C:\Users\naz\source\repos\lambda-slicer` |
| Archive root | `F:\HLScreeniesDemos` |
| Mod and second demo root | `F:\SteamLibrary\steamapps\common\Half-Life\ag` |
| Game installation | `F:\SteamLibrary\steamapps\common\Half-Life` |

**Observed:** recursive read-only enumeration found 1,129 archive paths totaling 20,794,389,276 bytes and 53,920 paths under `ag` totaling 53,069,260,123 bytes: **55,049 paths and 73,863,649,399 bytes, approximately 68.79 GiB**, before hardlink/content deduplication. Enumeration suppressed access errors, so these are observed entries, not proof of complete access. No directory reparse points were found. Most recordings under `ag` are small: 46,576 below 1 MiB, 6,847 from 1–10 MiB, 455 from 10–100 MiB, and 42 at least 100 MiB. Do not discard short recordings; make duration filtering available.

A systematic path-spaced sample, supplemented with size extremes and selected filename matches, read only headers and directory tables from 223 distinct demos. All sampled headers were GoldSrc demo protocol 5, network protocol 48, game directory `ag`. This was not a random sample and did not establish the format distribution or payload validity of all 55,049 files. Initial implementation should target **5/48**, while a full metadata scan reports other protocols honestly.

The installed `ag/liblist.gam` labels AG 6.7/openag.pro, but the active server DLL is Metamod and alternate DLL backups exist. This is a customized installation; the exact engine/client build has not been established. A campaign map name or the alternate filename `liblist-singleplayer.gam` does not prove singleplayer mode. Event resources missing from `ag/events` can reside under `valve/events`; the recorded resource table remains authoritative for numeric event identifiers.

### 1.2 Local fixture manifest to create during implementation

Keep original demos outside Git. Introduce an optional local fixture manifest, populated by the user/developer, with paths, full BLAKE3 hashes, expected profile, and verified observations. The following are suitable starting references; **none is currently a known-positive gauss fixture**:

| Role | Exact path | Bytes | Advisory directory duration |
|---|---|---:|---:|
| Required long interval test | `F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem` | 97,633,668 | 1206.301 s |
| Campaign-map candidate, mode unverified | `F:\HLScreeniesDemos\01apr2020\run_108.dem` | 110,210 | 0.55737305 s |
| Minimal local recording | `F:\HLScreeniesDemos\01apr2020\qe_9.dem` | 39,812 | 0.04156494 s |
| Streaming stress | `F:\SteamLibrary\steamapps\common\Half-Life\ag\20220319_194739_fastrun.dem` | 415,096,974 | 6078.867 s |
| Short movement attempt | `F:\SteamLibrary\steamapps\common\Half-Life\ag\20220413_171130_prochallenge_bhop_skip.dem` | 222,147 | 2.0216064 s |

All five sampled headers have two directory entries, `LOADING` type 0 and `Playback` type 1. The long fixture has loading offset 544, length 28,793; playback offset 29,337, length 97,604,143; directory offset 97,633,480; and 151,207 reported playback frames. These are useful initial container assertions, not proof that frame payloads or playback are correct. Calculate actual elapsed time from the decoded stream instead of trusting the advisory duration.

## 2. Technical choices and repository layout

Use Rust stable, edition 2024, with the exact tested toolchain recorded in `rust-toolchain.toml` and `Cargo.lock` committed. Resolve compatible dependencies once, verify their MSRVs, and lock them. The plan intentionally does not guess a stable compiler version that has not been built against the chosen dependency graph.

Use `eframe`/`egui` for the native GUI and `rfd` for dialogs. Research inspected **eframe 0.36.2**, whose `App` trait uses `ui` and `logic`; use the API documentation matching the version actually pinned. Use `eframe::egui` to avoid mismatched egui versions. Research inspected `rfd` **0.17.2**. Retain Linux X11/Wayland support; document the selected renderer's build prerequisites and the XDG portal/fallback requirements. GUI worker completion can trigger `Context::request_repaint`. ([eframe](https://docs.rs/eframe/0.36.2/eframe/), [App trait](https://docs.rs/eframe/0.36.2/eframe/trait.App.html), [rfd](https://docs.rs/rfd/0.17.2/rfd/))

Other intended dependencies: `serde`, `serde_json`, `thiserror`, `tracing`, `blake3`, `rusqlite` with bundled SQLite, `walkdir`, `same-file`, `tempfile`, an OS application-directory helper, and `clap` for the CLI. Use `std::thread`, bounded channels, and `AtomicBool` cancellation; Tokio is unnecessary for this local application. Use `proptest` and targeted fuzz harnesses for binary/bit parsers.

### 2.1 Codec choice requires an explicit audit

Evaluate `dem` **0.2.2** as a network codec starting point. It advertises GoldSrc parsing/writing and has raw/parsed network representations. Its public file parser reads the whole file, its parse errors lose detail, and its `DemoState` is a schema/registration context, not a reconstructed world. It is **LGPL-3.0**, not MIT/Apache. `hldemo` **0.3.0** is MIT OR Apache-2.0 and useful as an independent container reference, but does not advertise the needed network state reconstruction or writer. ([dem metadata](https://docs.rs/crate/dem/0.2.2/source/Cargo.toml), [dem API source](https://raw.githubusercontent.com/khanghugo/dem/master/src/lib.rs), [dem types](https://raw.githubusercontent.com/khanghugo/dem/master/src/types.rs), [hldemo](https://docs.rs/hldemo/latest/hldemo/))

Do not scatter upstream types throughout the app. Put an audited codec/fork behind internal interfaces. The implementer must record an upstream commit hash and audit results before relying on its writer. The complete `demo_writer.rs` was not inspected during this planning work. The container parser has unchecked slices that need correction or replacement before arbitrary user files are accepted. The delta implementation has specific semantic gaps described below.

Create `docs/dependency-audit.md` during implementation, recording commit/version, license, files adapted, patches, known gaps, and fixture evidence. Preserve required notices and choose a compatible project/distribution license deliberately. Valve SDK, ReHLDS, OpenAG, and codec source licenses are not interchangeable permissions to copy code into an arbitrary license. ReHLDS's cited implementation is a behavior reference; resolve its GPL obligations before copying or linking code. Independent implementation from the documented format is an alternative, with provenance recorded.

### 2.2 File ownership and boundaries

```text
Cargo.toml                         workspace and dependency versions
Cargo.lock
rust-toolchain.toml
crates/
  lambda-slicer-core/src/
    lib.rs                         public service API and result types
    container/{reader,index,writer}.rs
    protocol/{profile,bit,delta,messages,codec}.rs
    state/{world,snapshots,identity,persistent,clock}.rs
    analysis/{events,weapons,coverage}.rs
    timeline/{time,groups,selection}.rs
    slice/{plan,bootstrap,transform,write}.rs
    validate/{structural,semantic}.rs
    library/{discover,cache,fingerprint}.rs
    jobs.rs
  lambda-slicer-cli/src/main.rs     inspect/analyze/scan/slice/validate
  lambda-slicer-gui/src/
    main.rs
    app.rs                         UI state, generation IDs, worker messages
    library.rs                     paged/virtualized library
    timeline.rs                    lanes, aggregation, selection handles
    details.rs                     event evidence and diagnostics
    export.rs                      clip list and export jobs
    settings.rs                    roots, game paths, cache, worker count
tests/fixtures/                    small redistributable synthetic fixtures
crates/lambda-slicer-core/tests/local_corpus.rs
                                   opt-in external fixture manifest harness
fuzz/                              readers, bit/delta/message decoding
docs/
  dependency-audit.md
  protocol-profile-5-48.md
  engine-validation.md
  fixture-recording.md
.github/workflows/ci.yml
```

The core must not depend on egui or rfd. CLI and GUI use the same analysis, planning, export, and validation services. Export internals do not accept UI indices as byte offsets. Database availability must not be required to open and analyze a single demo.

### 2.3 Public contracts

Types below are design contracts, not a complete compiling implementation. Use explicit newtypes and checked conversion functions.

```rust
struct DemoTime(i64); // checked integer microseconds, normalized display time
struct SegmentId(u32);
struct SourcePosition {
    segment: SegmentId,
    directory_index: u32,
    frame_ordinal: u64,
    byte_offset: u64,
    message_ordinal: Option<u32>,
}
struct PlayerKey { segment: SegmentId, slot: u16, incarnation: u32 }
struct SourceFingerprint {
    size: u64,
    modified_ns: Option<i128>,
    file_identity: Option<FileIdentity>,
    quick_hash: Hash,
    full_hash: Option<Hash>,
}
struct SliceRequest {
    source: PathBuf,
    fingerprint: SourceFingerprint,
    segment: SegmentId,
    start: DemoTime,
    end: DemoTime, // exclusive
    destination: PathBuf,
}
struct SlicePlan {
    requested: TimeRange,
    actual: TimeRange,
    first_group: GroupId,
    end_group_exclusive: GroupId,
    bootstrap: BootstrapRecipe,
    clock_policy: ClockPolicy,
    sequence_policy: SequencePolicy,
    blockers: Vec<ExportBlocker>,
}
```

Expose `inspect_header`, `analyze`, `plan_slice`, `export_slice`, and `validate`. Long operations take cancellation and progress sinks, produce typed errors, and can operate without the GUI. A plan is bound to a full source fingerprint and profile version; exporting it after source change must fail. Internal snapshot handles may be ephemeral, but no exported `.dem` depends on a cache or sidecar.

Separate `AnalysisStatus` (`Queued`, `Running`, `Complete`, `Partial`, `Unsupported`, `Failed`, `Cancelled`), recording visibility, action evidence, actor evidence, and export compatibility. “Complete” parsing does not mean complete visibility of the match or proven export compatibility.

## 3. Read the GoldSrc container safely and preserve raw data

### 3.1 Initial binary contract

GoldSrc container integers are little-endian. Header layout is 544 bytes:

| Offset | Field | Size |
|---:|---|---:|
| 0 | `HLDEMO\0\0` magic | 8 |
| 8 | demo protocol, signed i32 | 4 |
| 12 | network protocol, signed i32 | 4 |
| 16 | map name, fixed raw bytes | 260 |
| 276 | game directory, fixed raw bytes | 260 |
| 536 | map checksum, u32 | 4 |
| 540 | directory offset, signed i32 | 4 |

At a valid directory offset, read the signed entry count followed by 92-byte records: type i32, description[64], flags i32, CD track i32, track time f32, frame count i32, offset i32, length i32. Preserve raw string buffers, flags, and float bits. Validate all signed values before converting to offsets or sizes, and cross-check every region against file length. A zero directory offset is a possible incomplete recording; support it only through a separate sequential recovery mode with explicit partial status. Never reinterpret it as an ordinary complete recording. ([dem container parser](https://raw.githubusercontent.com/khanghugo/dem/master/src/demo_parser.rs), [format reference](https://github.com/cgdangelo/talent/blob/main/libraries/parser-goldsrc/docs/demo-structure.md))

Each outer frame begins with type u8, time f32, and frame number i32. Tags 2–9 are DemoStart, ConsoleCommand, ClientData, NextSection, Event, WeaponAnimation, Sound, and DemoBuffer respectively. Known network record forms include tags 0/1; audit their exact layouts against the pinned codec and engine. Do not treat every unknown tag as a network packet merely because an upstream parser does. A network record includes recorded demo/view information, seven sequence integers, a message length, and the network payload. Parse fields explicitly; Rust/C struct padding is not the disk layout. The full nested network record grammar must be transcribed into `docs/protocol-profile-5-48.md` from the pinned source and verified fixtures before implementing its writer. ([dem types](https://raw.githubusercontent.com/khanghugo/dem/master/src/types.rs), [ReHLDS DemoFile](https://github.com/rehlds/ReHLDS/blob/master/rehlds/HLTV/common/DemoFile.cpp))

Use a buffered `Read + Seek` reader and a compact `ContainerIndex`. Each `FrameSpan` records entry, byte range, tag, original time bits, frame number, optional sequence metadata, and continuity segment. Do not load every file into a full `dem::Demo` tree. Preserve padding and raw spans for opaque payloads. Directory counts/durations are advisory cross-checks; terminal records and bounded payload parsing establish real structure.

### 3.2 Failure behavior and limits

Every addition, multiplication, range calculation, and allocation derived from file contents is checked. Reject nonfinite scheduling times, impossible counts, truncated fixed buffers, and lengths beyond remaining bytes. Apply documented resource limits, with measured exceptions configurable for legitimate files. A precise error contains source path, entry, frame ordinal, byte offset, opcode/field, and cause.

An engine message with an unknown unframed opcode cannot safely be skipped. Stop decoding that stream and mark the affected segment partial/unsupported; do not scan for a plausible next opcode. A registered user message with a known fixed or variable length can be preserved as opaque, but its unknown semantics can still block slicing. A corrupt file must not crash a scan or contaminate the next file's state.

Do not execute ConsoleCommand records, `stufftext`, client DLL demo buffers, scripts, or any other embedded instructions during analysis. Display escaped/plain text only when useful. The application is a parser, not an interpreter of recorded game commands. Engine validation is a separate deliberate test using trusted fixtures.

## 4. Reconstruct protocol state before interpreting events or slicing

### 4.1 Decode state and snapshot identities

Maintain the following per continuity segment:

```text
DecodeState
  protocol and mod profile
  dynamic delta schemas, preserving field order/type/scale/bit widths
  registered user messages: id -> raw name and fixed/variable length
  resources: (type, index) -> exact path bytes and metadata
  entity baselines and instanced baselines
  sequence-indexed entity/clientdata/weapondata snapshot history
  low-byte reference -> full source sequence/epoch resolution
  server time, max_clients, recorder identity, view entity, HLTV flag
  signon lifecycle, persistent engine state, ordered mod reducers
  player connections and name history
```

A `PacketSnapshot` contains the full ordered entity set, full clientdata, all supported weapondata slots, source sequence identity, source server time, and the associated original message spans. A delta describes changes, not a complete entity. Missing fields inherit from the proper base.

Process physical message order, including loading/signon messages. Detect new server/signon/map continuity boundaries; invalidate previous schemas, resources, identities, and sequence bases as required. Directory boundaries alone are not reliable session boundaries. Snapshot keys use netchannel source sequence identity, not the outer demo frame number. Track wrapping and duplicate sequence circumstances without silently overwriting a different snapshot.

For `svc_deltapacketentities`, resolve the low-byte reference through the actual sequence mapping. It is not “current frame minus this byte.” Copy/reference the referenced active set, apply patches, remove deleted entities, and retain unchanged entities. New entities use the correct baseline rules. Resolve `svc_clientdata` against its own base and retain all weapondata deltas. In HLTV mode its body can be empty; that is not automatically corruption. ([ReHLDS Server](https://raw.githubusercontent.com/rehlds/ReHLDS/master/rehlds/HLTV/Core/src/Server.cpp), [dem clientdata](https://raw.githubusercontent.com/khanghugo/dem/master/src/netmsg_doer/client_data.rs))

### 4.2 Mandatory codec audit before export

| Topic | Required work and proof |
|---|---|
| Delta schema decoding | Decode dynamic descriptions through the protocol's meta-schema; validate field count, masks, bit widths, strings, and scales. Preserve original quantized representations. |
| Entity header baselines | Audit full/delta packet grammar for entity baseline, instanced baseline, and backward reference to a prior decoded entity. The inspected Rust representation does not fully express these distinctions. Test zero instanced baselines with a nonzero backward offset, nonzero instanced baseline count with selection false/true, and delta packets with instanced baselines. |
| Reference resolution | Implement the source sequence map and wrap epochs. Test missing bases, low-byte wrap, repeated outer frame counters, removals, and newly visible entities. |
| Time windows | Represent `TimeWindow8` and `TimeWindowBig` separately from floats. Preserve source server time, quantized signed offset, and scale. Test exact truncation/scaling semantics instead of blanket rounding. |
| Client/weapon data | Reconstruct every referenced slot; do not adopt a reference implementation that discards weapondata for POV export. |
| Container writer | Audit header patches, directory lengths/counts, terminal records, raw mode, all frame payloads, message ordering, and fixed-string preservation. |
| Output limits | Establish the actual profile's maximum encoded packet/message sizes from engine behavior/source and fixtures; do not substitute a parser allocation cap. |

The inspected `dem` delta parser groups time-window fields with floats and its writer rounds scaled values. ReHLDS uses server-time-dependent decoding; for normal TimeWindow8 it is `(server_time * 100 - signed_offset) / 100`, with encoding based on separately truncated scaled values. Large time windows use schema scaling. Audit the exact bit/sign behavior rather than copying this formula into a generic float encoder. ([dem delta](https://raw.githubusercontent.com/khanghugo/dem/master/src/delta.rs), [ReHLDS Delta](https://raw.githubusercontent.com/rehlds/ReHLDS/master/rehlds/HLTV/Core/src/Delta.cpp))

The inspected full entity packet implementation has an unconditional `has_baseline_index` concept, while the reference engine distinguishes conditional instanced baseline selection and prior-entity offsets. The delta packet implementation also requires this audit. Simply renaming a field will not fix an incorrect bit grammar. ([dem packet entities](https://raw.githubusercontent.com/khanghugo/dem/master/src/netmsg_doer/packet_entities.rs), [dem delta packet entities](https://raw.githubusercontent.com/khanghugo/dem/master/src/netmsg_doer/delta_packet_entities.rs))

ReHLDS World is useful for reconstructing baselines, snapshots, resources, and new-data output, but its intermediate clientdata path discards weapondata. It is not a drop-in correct AG POV slicing implementation. Its surrounding DemoPlayer's `RemoveFrames` is empty. ([ReHLDS World](https://raw.githubusercontent.com/rehlds/ReHLDS/master/rehlds/HLTV/Core/src/World.cpp), [ReHLDS DemoPlayer](https://raw.githubusercontent.com/rehlds/ReHLDS/master/rehlds/HLTV/DemoPlayer/src/DemoPlayer.cpp))

### 4.3 Clock domains and atomic frame groups

Keep these clocks distinct in code and tests:

| Domain | Meaning | Initial export policy to test |
|---|---|---|
| Outer demo time | Playback scheduling | Rebase retained groups to near zero |
| Outer frame counter | Container sequencing/group association | Regenerate coherently; never use as net sequence |
| `svc_time` | Server simulation coordinate | Preserve original coordinate initially |
| Demo/refparam simulation fields | Camera/client simulation and interpolation | Preserve until each field's semantics is audited |
| Entity/weapon absolute timers | Animation, attacks, related state | Preserve source domain or remap by declared schema |
| Time-window wire offsets | Values relative to server time | Re-encode against the server time of the emitted packet |
| Event delay | Relative delayed dispatch | Preserve as a duration; account for pending boundary events |

Normalize the UI timeline from the actual playback lifecycle, respecting DemoStart and discontinuities. Keep raw f32 bits. Do not subtract the requested start from every floating field. A nonmonotonic clock is a diagnostic/segment decision, not an instruction to sort the original stream by time.

Define `FrameGroup` as an indivisible, source-ordered playback unit containing its network packet and associated outer records required by that recorded frame. Establish exact grouping using tag/frame-number/time relationships from the protocol reader and controlled fixtures; equal timestamps alone do not establish one frame, and one frame can contain multiple commands. Preserve order within and between groups. Boundary tests must include multiple records sharing a timestamp and event records adjacent to network records. This grouping definition is a required S0/S1 audit result, not a guessed universal rule.

For selection, choose the first complete group at or after `start`, and exclude the first group at or after `end`. If end exceeds the segment duration, report a validation error or require an explicit UI adjustment. A selection with no retained groups is an error. Display requested and effective times, including snapping. A changelevel/signon boundary initially requires separate clips; do not merge incompatible sessions into one apparently ordinary slice.

## 5. Weapon activity and actor attribution

### 5.1 Preserve evidence before classification

Consume standalone demo `Event` records, `svc_event`, and `svc_event_reliable`. Resolve numeric event IDs through the segment's resource table and dynamic event delta schema. An event resource index is not a weapon ID, and a resource being precached does not prove use.

```text
RecordedEvent
  stable raw record ID and SourcePosition
  source kind: DemoEvent / SvcEvent / SvcEventReliable
  raw event index and resolved resource path, if known
  outer dispatch flags, if available, separately from event_args.flags
  complete decoded event args and raw quantized fields
  packet_index, explicit entindex, resolved entity, resolution reason
  outer presentation time, source server time, relative delay
  derived scheduled time if its mapping is established

WeaponOccurrence
  occurrence ID, raw evidence IDs, segment and time
  weapon, action kind, mode, optional PlayerKey/entity identity
  action evidence label, attribution evidence label
  optional interval endpoints with known/unknown endpoint flags
```

Preserve `event_args` fields including flags, entity, origin/angles/velocity, ducking, floating/integer/boolean parameters, even when a weapon rule uses only a subset. Keep dispatch flags (`FEV_NOTHOST`, `FEV_RELIABLE`, etc.) separate from argument flags. Recorded dispatch time and delayed presentation time are different; index both, and document which the timeline shows. Retain immutable raw references so detector improvements can be audited. ([Valve event flags](https://github.com/ValveSoftware/halflife/blob/master/common/event_flags.h), [ReHLDS event construction](https://github.com/rehlds/ReHLDS/blob/master/rehlds/engine/pr_cmds.cpp#L1120))

### 5.2 Resolve actors correctly

Reliable events carry `args.entindex`. Unreliable event `packet_index` is an **array position in that packet's complete entity array**, not an edict or player number. Resolve it only after reconstructing that packet's full entity array, including unchanged delta entities. When the invoker is absent from the packet, the server can use `packet_index == entity_count` and provide the actual entity in the event args. Handle this fallback explicitly; out-of-range indices never index an arbitrary cached array. Defer event resolution until packet context is available while retaining original message order for dispatch. ([ReHLDS event emission](https://github.com/rehlds/ReHLDS/blob/master/rehlds/engine/sv_main.cpp#L4115))

Use `svc_updateuserinfo` for connection identity: slot, userID, info string, and associated raw data. Convert zero-based userinfo slots to one-based player entities in named tested functions. A reused slot with changed userID/disconnect/reconnect begins a new incarnation; a name change updates a time-bounded name history. Do not key a player by display name. Recorder, view entity, spectator target, and shooter are different concepts.

For standalone events, use valid explicit entity identity. A missing/zero entity remains unknown unless a profile-specific local-recorder rule is proven with a fixture. Do not assign it to the current camera player. Label unresolved cases `Unknown player (entity N)` or `Unknown actor`; separate nonplayer entities/NPCs from player records in singleplayer.

Visibility is independent of decoder correctness. ReHLDS filters nonglobal events by recipient visibility and `FEV_NOTHOST` can suppress a local predicted event on the network path; local demo events can provide another channel. Proxy/singleplayer multicast behavior differs, but neither implies unlimited completeness under packet loss, queue limits, or mod behavior. Input processing also gates firing on cooldown/ammo. `IN_ATTACK` alone is **input intent**, not proof of a shot. ([ReHLDS playback filtering](https://github.com/rehlds/ReHLDS/blob/master/rehlds/engine/pr_cmds.cpp#L1120), [Valve prediction events](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/com_weapons.cpp#L121), [Valve weapon input handling](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/hl/hl_weapons.cpp#L308))

Deduplicate only duplicate processing of the same raw source record initially. Correlate local/network records into one occurrence only after fixtures establish a reliable same-dispatch relation; retain both evidence IDs. Do not merge by a broad time threshold, which would destroy rapid-fire or simultaneous-player events. Counts must state whether they are raw recorded events or validated normalized occurrences.

### 5.3 Gauss rules: firing is different from charging and stopping

Implement a versioned `HlAgWeaponProfile` with these rules:

| Resource and fields | Action | Adds a shot? |
|---|---|---|
| `events/gauss.sc`, `bparam2 != 0` | Charge/spin stop | No |
| `events/gauss.sc`, `bparam2 == 0`, `bparam1 != 0` | Primary fire | Yes |
| `events/gauss.sc`, `bparam2 == 0`, `bparam1 == 0` | Charged secondary fire | Yes |
| `events/gaussspin.sc` | Charge activity/pitch update | No |

OpenAG emits a reliable delayed stop after firing, and holstering can emit the stop without a shot. The client gauss handler returns after handling `bparam2`; therefore counting every `gauss.sc` record would be wrong. Preserve `fparam1` as `event_damage_parameter`, not actual inflicted damage. `gaussspin.sc` uses `iparam1` for pitch and `bparam1` for sound-state change. ([OpenAG gauss](https://github.com/YaLTeR/OpenAG/blob/master/dlls/gauss.cpp#L70), [Valve gauss event handlers](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/ev_hldm.cpp#L777), [Valve gauss logic](https://github.com/ValveSoftware/halflife/blob/master/dlls/gauss.cpp))

Track charge activity per actor: first observed spin opens an observed interval, later spin updates extend it, and fire/stop closes it. Missing start/end remains an uncertain endpoint. Do not infer a fixed full-charge percentage or a firing event from ammo loss. Sound evidence such as `weapons/gauss2.wav` can support a candidate, but is not automatically an additional shot. A stop without a recorded fire may reflect missing visibility/loss; keep it as a stop.

Gauss search should separately expose recorded fires, charge activity, kills, and possible/unknown candidates. Suggested scenes may merge nearby activity with configurable preroll/postroll, but shot counts use only firing occurrences. Unknown shooter does not invalidate a correctly decoded gauss firing event.

### 5.4 Standard weapon catalog and action semantics

Match recorded resource paths with a profile-controlled normalization for separators/case, while preserving exact raw bytes. Do not normalize filesystem paths by lowercasing them. Standard HL IDs and event mappings are starting semantics, refined by recorded `WeaponList` and profile fixtures. ID 5 is the legacy chaingun define, not a normal supported Half-Life weapon. ([Valve IDs](https://github.com/ValveSoftware/halflife/blob/master/dlls/weapons.h#L59), [Valve event hooks](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/hl/hl_events.cpp#L53))

| ID | Weapon | Event suffix under `events/` | Display action |
|---:|---|---|---|
| 1 | Crowbar | `crowbar.sc` | Swing; not confirmed hit |
| 2 | Glock | `glock1.sc`, `glock2.sc` | Fire, profile mode |
| 3 | Python/.357 | `python.sc` | Fire |
| 4 | MP5 | `mp5.sc`, `mp52.sc` | Bullet fire / grenade launch |
| 6 | Crossbow | `crossbow1.sc`, `crossbow2.sc` | Fire; latter is zoomed multiplayer shot |
| 7 | Shotgun | `shotgun1.sc`, `shotgun2.sc` | Single / double fire |
| 8 | RPG | `rpg.sc` | Launch |
| 9 | Gauss | `gauss.sc`, `gaussspin.sc` | Fire / charge / stop as above |
| 10 | Egon | `egon_fire.sc`, `egon_stop.sc` | Start / sustained activity / end |
| 11 | Hivehand | `firehornet.sc` | Fire; tracked/dart mode from validated parameters |
| 12 | Handgrenade | No dedicated mapping established | Equipped, animation, inferred throw, kill evidence |
| 13 | Tripmine | `tripfire.sc` | Placement attempt unless success corroborated |
| 14 | Satchel | No dedicated weapon event script | Equipped, animation, inferred throw/detonate, kill evidence |
| 15 | Snark | `snarkfire.sc` | Release attempt unless success corroborated |

Crossbow zoom toggle is not a `crossbow2.sc` firing event. Egon startup and sustained events are not damage-tick counts. Tripmine and snark dispatch their events before checking whether deployment succeeds. Handgrenade pin pull precedes throwing in `WeaponIdle`; satchel throwing and radio action require different evidence. Do not invent actor ownership from a projectile's proximity or its first appearance in a visibility-limited recording. Fine-grained animation/projectile rules stay disabled until profile fixtures establish them. ([Crossbow](https://github.com/ValveSoftware/halflife/blob/master/dlls/crossbow.cpp#L299), [Egon](https://github.com/ValveSoftware/halflife/blob/master/dlls/egon.cpp#L165), [Tripmine](https://github.com/ValveSoftware/halflife/blob/master/dlls/tripmine.cpp#L403), [Snark](https://github.com/ValveSoftware/halflife/blob/master/dlls/squeakgrenade.cpp#L494), [Handgrenade](https://github.com/ValveSoftware/halflife/blob/master/dlls/handgrenade.cpp#L105), [Satchel](https://github.com/ValveSoftware/halflife/blob/master/dlls/satchel.cpp#L232), [Hivehand](https://github.com/ValveSoftware/halflife/blob/master/dlls/hornetgun.cpp#L133))

`CurWeapon` is state byte, signed weapon ID, signed clip count; zero state does not select that weapon. `WeaponList` supplies weapon/ammo metadata. Neither message carries shooter identity; scope it to the recipient/client context proven by the profile. Half-Life `DeathMsg` is killer byte, victim byte, weapon string, without Counter-Strike's headshot byte. A kill timestamp is not a launch timestamp and does not increment shot count. Singleplayer cannot depend on a multiplayer death feed. ([Valve HUD ammo](https://github.com/ValveSoftware/halflife/blob/master/cl_dll/ammo.cpp#L539), [Valve DeathMsg](https://github.com/ValveSoftware/halflife/blob/master/dlls/multiplay_gamerules.cpp#L600))

## 6. Independent slice export: mandatory feasibility sequence

### 6.1 Establish controls before reconstructing a start state

Use a real recording that already plays in the installed AG. Save engine/mod identification, commands, logs/screenshots, source hash, and observations in `docs/engine-validation.md`.

1. **S0a: exact copy under another name.** Confirm the test environment, resource availability, and ordinary `playdemo` invocation.
2. **S0b: container roundtrip with raw payloads and unchanged clocks.** Rebuild the container while preserving all packet/opaque payload bytes. Compare logical structure and play it. Fix writer defects before continuing.
3. **S0c: end-only truncation from time zero.** Rebuild the terminal records/directory for `[0,end)`. Play it and verify a clean stop.
4. **S1: nonzero-start restart.** First use a short deterministic fixture with no complex effects, then a two-player gauss fixture, then the real long file's `[96,288)` interval.

Do not spend weeks polishing a GUI while S1 is unproven. If the simple restart fails, isolate clock/signon/sequence/state issues with trace comparisons and controlled variants. Maintain a hypothesis/result table. A repeated failure is a blocker to resolve or report; it is not permission to silently change the product into an analyzer or full-demo seek helper.

### 6.2 Full sequential decode and boundary snapshot

Rescan from the original segment's signon to the boundary, even if the user selected a time near the end. Checkpoints may accelerate later runs only after they are validated and versioned. The export must be able to operate from the original file alone.

Capture state **immediately before** the first retained atomic group. Applying that group during bootstrap and replaying it again would duplicate shots, HUD transitions, and effects. Capture static initialization plus durable dynamic state:

| State class | Bootstrap treatment |
|---|---|
| Protocol, map, checksum, mod, resources | Preserve legal initialization and exact resource identities |
| Delta schemas, entity/instanced baselines | Emit in required order before dependent messages |
| Registered user messages | Register before emitting their bodies |
| Server/player information and identity | Latest state at the boundary, with correct slots/incarnations |
| Entities, clientdata, weapondata | Complete reconstructed pre-boundary state |
| Light styles, view entity, movevars, pause/intermission, room/CD settings | Ordered persistent reducers and profile-specific restart emission |
| Score/HUD/weapon selection | Known mod reducers respecting reset/spawn/death lifecycle |
| POV view/refparams/viewmodel/animation | Audited restart state and required interpolation sample |
| Looping sounds, long-lived effects, delayed events | Reconstruct only through tested handlers, otherwise block affected state |
| Historical chat, deaths, one-shot sounds/shots | Do not replay merely to rebuild state |

“Keep the last message of every opcode” is not a correct persistent-state reducer: resets and lifecycle messages change the meaning of subsequent updates. Build typed reducers with reset behavior and tests. Unknown registered messages and DemoBuffer payloads can carry client DLL private state. Valve exposes a separate opaque demo buffer API, so generic networking alone cannot prove their restart behavior. ([Valve demo API](https://raw.githubusercontent.com/ValveSoftware/halflife/blob/master/common/demo_api.h))

#### Which reconstructed state an export may use

The decoder reports, per snapshot, whether its state is proven (every base back to the last full update resolved strictly) or inferred, and M2 cross-checks each source against client-recorded values (`verified`, `inconclusive`, `failed`; see `docs/protocol-profile-5-48.md`). An arbitrary-start export writes reconstructed state, so it may use only state that is proven:

- **The source must not be `failed`.** A disagreement on proven state means the decoder misread something it believed, so none of its proven flags can be trusted for that file. Refuse with `SourceFailedCrossCheck`.
- **The boundary snapshot must be proven.** The state captured immediately before the first retained group, and every snapshot in its base chain back to the last full update, must not be inferred. Otherwise refuse with `UnprovenState`, naming the first inferred location.
- **Every reconstructed state the export emits must be proven too.** The first prototype (§6.4) replaces each retained entity update with a complete packet from reconstructed state, so an inferred snapshot anywhere inside the interval blocks the export the same way. A later delta-preserving encoder must apply the same rule to every state it derives.
- **No boundary or retained packet may fall in a run the client dropped as too old.** During such a run the client's state is frozen at an older frame while the source keeps carrying packets it ignored. Refuse with `ClientDroppedPackets` until the export reproduces that behaviour and it is validated in the game.

An `inconclusive` source is therefore usable wherever these conditions hold. Typically that is everything after the first full update that follows a recording's opening frames. Exact copies and end-only trims rebuild no state and are not subject to this rule. `plan_slice` reports which of the requested range is eligible, so the GUI can explain a refusal and suggest the nearest eligible start.

Pending delayed events need a queue with source schedule identity: events dispatched before the boundary but due inside the interval require tested reconstruction; events scheduled beyond the exclusive end must not extend the requested clip. Do not silently drop them or replay them immediately. Likewise, charging that started before the cut needs the correct observed ongoing state, not a fabricated new shot at zero.

### 6.3 Synthesize legal startup and first state

Create a fresh legal loading entry using required original static initialization in engine order, followed by boundary durable state. Preserve header map/game/protocol/checksum. Establish the relevant server time before decoding time-relative fields. Emit a bootstrap state packet before the first retained visible group, with scheduling that does not add historical gameplay duration.

Derive signon stages/order and whether a zero-time bootstrap/interpolation sample is needed from S0 controls and actual engine behavior. Do not invent a `signonnum = 4` shortcut. ReHLDS's startup writer emits new data, signon, time, player updates, and light styles, but this is a reference to audit against POV requirements. ([ReHLDS DemoFile](https://github.com/rehlds/ReHLDS/blob/master/rehlds/HLTV/common/DemoFile.cpp))

The output must contain all bootstrap information necessary for playback and must still work when the original file is renamed/unavailable. Matching game/mod/map assets are normal dependencies. A JSON report, original path, or cached snapshot must never be a playback dependency.

### 6.4 Encode output-consistent state, then optimize

For the first proof, replace each retained stateful entity update with a complete `svc_packetentities`, based on the correct output baselines; emit clientdata without a delta base and include complete supported weapondata. Preserve ordered non-stateful messages and outer records that fall within the selected interval. Preserve entity ordering because unreliable event indices refer to the packet's ordered entity array. If ordering changes, regenerate those references consistently.

This removes cross-packet reference dependencies but is a **prototype strategy**, not a guaranteed final solution. Full packets can exceed the engine limit or make time-window values unrepresentable against the selected baseline. Measure encoded sizes and representability. If repeated full updates overflow, retain the first complete restart state and encode later updates as bounded deltas against snapshots actually emitted in the output. Regenerate every affected sequence/reference and validate with a fresh decoder. Never repair only the first delta while leaving later references pointing into discarded history.

If the mandatory initial full state cannot be represented within the profile's legal limits, return `RestartStateUnrepresentable`/`PacketOverflow` and resolve the encoding/bootstrap design. Never split an entity message at an arbitrary byte position, silently discard entities/weapondata, or zero fields to make it fit. Any legal multistep initialization must be established against the engine grammar and tested without an added visible historical prefix.

Preserve all seven source sequence-header fields for the initial experiment only if that is compatible with self-contained messages. If fresh sequences are necessary, document each field's role and regenerate them coherently, including low-byte delta references, acknowledgement/reliable state, and history epochs. Numeric renumbering of one field alone is unacceptable. Record the selected `SequencePolicy` in the profile and export report.

### 6.5 Resolve clocks with a bounded experiment

First test rebased outer schedule/frame numbers with preserved source server-time coordinates, preserved simulation fields, and correctly re-encoded time-window values. This avoids confusing absolute clocks with durations. The startup must establish the boundary's original server time.

Compare this prototype against the original with packet/state traces and actual game playback. If the client rejects it or camera/animation/audio timing is wrong, enumerate affected clock fields and implement a schema-aware transformation with dedicated fixtures. The exact retail playback clock interaction remains unproven. Record a field-level policy table, never a blanket `float -= start` transformation.

A slice that plays 96 seconds of invisible/fast-forwarded source prefix before showing the requested scene is not the required output. Optional user-selected visible preroll is a different explicit interval. No engine modifications, special seek commands, external original file, or custom playback launcher may be required for the ordinary export path.

### 6.6 Message export policy and blockers

Every parsed message/frame type receives one profile policy: static initialization, durable reducer/re-emitter, state snapshot transformation, retained transient, proven-safe opaque pass-through, or unsupported. Bounded length alone does not prove an opaque payload is safe to copy after a time/sequence transformation. Unknown semantics that can affect restart state or transformed fields block the affected export, even if analysis is complete.

Use typed blockers with context, including `UnsupportedProtocol`, `MissingDeltaBase`, `UnknownOpcode`, `UnknownPersistentMessage`, `UnknownDemoBufferSemantics`, `MissingRestartHandler`, `UnrepresentableTimeField`, `PacketOverflow`, `CrossSegmentRange`, `SourceChanged`, `EmptyRange`, and the reconstructed-state blockers `SourceFailedCrossCheck`, `UnprovenState`, and `ClientDroppedPackets` (§6.2). Explain them in the GUI. Do not replace this list with a generic “failed” toast.

```text
plan_slice(request):
  open immutable source; validate requested segment/range/profile
  index and sequentially decode to the exclusive end
  resolve atomic groups; snapshot state before first retained group
  require a non-failed source and proven, undropped state at the boundary and throughout
  build bootstrap recipe and message transformation policy
  check all restart/time/sequence/opaque-state coverage and output limits
  return requested/actual bounds, recipe, source fingerprint, blockers

export_slice(plan):
  verify source identity and full content fingerprint
  create unique sibling temporary output with no overwrite
  emit header placeholder and synthesized loading/bootstrap
  decode/re-encode retained groups with output-consistent state history
  write terminal records and directory; patch header and metadata
  flush/sync; independently reparse; compare semantic traces
  verify source stability; publish with no-clobber operation
  return final path, actual range, hashes, diagnostics, validation levels
```

### 6.7 Three distinct validation levels

**Structural:** a fresh reader consumes the intended entries; offsets, lengths, counts, times, terminal records, and message boundaries are valid; all resources and delta bases resolve; no reference requires discarded source history. Writer/parser roundtrip alone is insufficient because they can share a bug.

**Semantic:** independently decode source and output through reducers. Compare complete entity/client/weapon states at corresponding visible samples, player identity/name at time, event order and action counts, delayed dispatches, and preserved opaque bytes. Normalize only declared clock domains. Permit tolerances only for demonstrated wire quantization and document them; broad position/time tolerances can hide a broken transform.

**Engine:** play both original and output in the installed Windows AG. Capture actual screenshots/logs and a test record with game/mod identification, source/output hashes, requested/effective times, duration, and observations. For `[96,288)`, verify the correct initial scene, approximately 3:12 at normal playback rate, the last included scene and clean endpoint, correct players/camera/models/HUD, gauss charge/discharge audio and effects, no frozen entities, overflow, unexpected disconnect, or desynchronization. Copy the output to a simple valid relative demo name in an appropriate game directory and test it with ordinary `playdemo`; do not interpolate untrusted strings into shell commands.

Engine validation certifies a tested codec/profile and fixture matrix. Each routine export can report structural/semantic validation plus the profile's engine-tested status; do not claim every newly exported file was game-tested automatically. A parser-valid output under an unproven profile remains experimental, and normal export support must not be advertised before the engine gate passes.

## 7. Batch library, cache, and responsiveness

### 7.1 Discovery and worker model

Discovery streams paths into a bounded queue. Do not wait for all 55,049 paths or full hashes before showing results. Skip directory symlinks/junction traversal by default, deduplicate overlapping roots and file identities, and record inaccessible-directory errors. Use one parser worker by default on the supplied `F:` drive; make 1–4 workers configurable after measuring disk behavior. User-opened files take priority over background scanning.

Jobs carry IDs and UI generation IDs. Cancel at file/frame/message boundaries and during long bounded loops; cancellation is not corruption. Send aggregated progress at no more than about 10 Hz. Limit the GUI's per-frame message draining. Store immutable summaries behind `Arc`; keep complete packet trees out of UI state. Pausing/cancelling discovery preserves completed results. A corrupt file yields its own result and the scan continues.

The initial full library analysis is necessarily I/O work over tens of GiB; do not promise instant first-pass results. Subsequent scans should be dominated by metadata checks/cache hits. Measure discovered/queued/completed counts, bytes read, cache hits, current file, and parse errors separately so progress is meaningful.

### 7.2 Cache schema and invalidation

Store SQLite in the OS app cache/data directory, not beside every source file. Use one DB writer and per-file transactions. Index `(file_id,time_us)` and `(weapon,file_id)`. Suggested tables:

```text
files(id, platform_path_blob, display_path, size, mtime_ns, file_identity,
      quick_hash, full_hash, parser_version, detector_version, profile_version,
      status, map, mod, demo_protocol, network_protocol, duration_us,
      coverage_json, gauss_fire_count, gauss_charge_count, candidate_count,
      diagnostics_json)
players(file_id, segment_id, slot, incarnation, user_id,
        start_us, end_us, name_bytes, name_display)
events(file_id, event_id, segment_id, time_us, frame_ordinal, byte_offset,
       weapon, action, mode, slot, incarnation, action_evidence,
       actor_evidence, evidence_json)
clips(file_id, clip_id, segment_id, start_us, end_us, label)
```

Keep detailed frame indices/checkpoints on demand for selected files or in a separately versioned disk cache. Do not retain giant world histories for the entire library. SQLite transactions/queries are supported by rusqlite; benchmark paging and transaction batching with this corpus. ([rusqlite](https://docs.rs/rusqlite/latest/rusqlite/))

Fingerprint metadata uses size, modification time, file identity, and a quick hash of bounded header/first/last regions; stream full BLAKE3 during full analysis. Quick matches are hints, not proof of unchanged contents. Parser/detector/profile/schema version changes invalidate affected records. Export verifies the full content fingerprint through the source read used for export before publication, with metadata/identity checks before and after. A concurrent mutation yields `SourceChanged`, never a trusted clip from mixed bytes. Offer Force rescan.

Cancelled/partial results stay explicitly marked and cannot overwrite a previous successful analysis as though complete. Store diagnostics and coverage with counts so a cached zero cannot become a false negative. Cache failure must degrade to direct analysis. Preserve raw player strings and native path encoding separately from display text.

### 7.3 Performance acceptance

Test discovery and cached listing with the observed corpus scale or generated metadata equivalent. Library rows must be paged/virtualized, and opening a cached selection must not instantiate 55,049 rows or parse all files. During a scan/export, window input, scrolling, and cancellation must remain responsive; target sub-250 ms UI feedback and report measured results on the development machine. This is a target to test, not a claimed benchmark.

Stress analysis with the 415 MB fixture and record peak memory. Memory should track one active file's compact index, bounded snapshot history, and selected result data, not all files or an unbounded decoded tree. Establish actual limits after the codec audit; do not discard required delta history merely to meet an arbitrary memory number.

## 8. GUI workflow

Use a toolbar with Open demo(s), Add folder, Rescan, Cancel/Pause, and Settings. Drag/drop accepts files and folders through native dropped paths. Inspect header magic, not just extension. Show per-item errors for unrelated or unsupported files. `egui` exposes hovered/dropped files through input state. ([RawInput](https://docs.rs/egui/0.36.2/egui/struct.RawInput.html))

The left library has status, name/path, map, duration, gauss fire count, charge/candidate counts, and coverage. Provide text/map/player/weapon filters and a minimum duration filter, with gauss highlighted by default. Show “not analyzed,” “partial,” and “no observed use” distinctly. Search results should still include unknown-actor gauss events. Selecting a result prioritizes its detailed analysis.

The right pane contains metadata/coverage, a zoomable timeline, event table, and clip list. Use painter-backed lanes per player with weapon/action filters; selection, input intent, firing, charging, and kill tracks remain distinguishable. At wide zoom, aggregate markers into bins based on visible pixel width. Time-sorted indices support hit testing and event navigation. Show historical actor names at the event time, plus stable connection identity in details.

Clicking an event centers the timeline and proposes a configurable pre/postroll range. From M11, it also seeks the integrated viewer through the shared playback controller (§13.4). Drag handles or edit start/end fields accepting `96`, `1:36`, and `00:01:36.250`; display `HH:MM:SS.mmm`. Permit multiple named clips. Show the requested interval, effective snapped interval, and duration before export. The example must visibly read requested 01:36–04:48, duration 03:12, plus any adjustment.

The export pane shows destination, compatibility/profile status, blockers, progress, structural/semantic validation, and final file path. Do not display success until no-clobber publication succeeds. A failed export leaves the original untouched and does not put a broken final `.dem` in the destination. Provide a safe copyable `playdemo` instruction using the final basename and explain any placement requirement; automatic game launching is optional later and must use explicit executable/argument APIs.

Settings hold configurable roots, game/mod location, worker count, cache location/size policy, scene preroll/postroll, and diagnostic verbosity. Persist ordinary settings in the OS configuration directory. No production path may be hardcoded to this developer's Windows installation. Linux uses the same core and GUI; path/case/portal behavior is tested separately.

### 8.1 Languages

The interface ships in English (`en`), Latin American Spanish (`es-419`), Brazilian Portuguese (`pt-BR`), Russian (`ru`), Ukrainian (`uk`), Turkish (`tr`), Polish (`pl`), German (`de`), French (`fr`), Italian (`it`), Finnish (`fi`), Lithuanian (`lt`), Arabic (`ar`) and Simplified Chinese (`zh-Hans`), selectable in Settings and applied without a restart. The first run picks the operating system's language when it is one of these, otherwise English, and the choice persists with the other settings. Adding a language later means adding one catalog file.

- **Catalogs.** Use Project Fluent (`fluent-bundle`), one `.ftl` file per locale embedded in the binary. Russian, Ukrainian and Polish need the one/few/many plural forms (1 убийство, 2 убийства, 5 убийств, or 1 zabójstwo, 2 zabójstwa, 5 zabójstw), which Fluent selects from CLDR rules. Lithuanian has its own one/few/many/other forms, Arabic has six (zero, one, two, few, many, other), French and Italian add a separate form for large round numbers, and Chinese has a single form. Never build a sentence from separately translated fragments. Each sentence is one message with named arguments, so translators can reorder them.
- **Fallback and completeness.** English is the source and the fallback. A message missing from another locale falls back to English for that message, never to its key. A test checks that every locale has every English message with the same arguments, and that every message id used in code exists in English.
- **The core stays language-neutral.** Everything the core reports to a user (errors, export blockers, coverage notes, structural findings, scan outcomes) carries a stable code and typed arguments. Its English text remains for the CLI, logs and JSON, which stay English. The GUI translates by code and keeps the English detail one click away for bug reports. Coverage notes, now English strings, become structured codes before the GUI uses them.
- **Not translated.** Player names, map names, paths, the death notice's weapon strings, protocol and profile identifiers, and the `playdemo` command. Weapon names use the game's own names in every locale.
- **Formatting.** Numbers, decimals and sizes use the locale's separators as CLDR gives them (a decimal comma in `pt-BR`, `ru`, `uk`, `tr`, `pl`, `de`, `fr`, `it`, `fi` and `lt`, a decimal point in `en`, `es-419`, `ar` and `zh-Hans`). Arabic uses the digits 0 to 9 like the game and the CLI, with the Arabic decimal separator, and dates use the locale's order. The timecode display `HH:MM:SS.mmm` is the same in every locale, matching the CLI and the game. Time entry accepts both `.` and `,` before the milliseconds.
- **Text.** Cyrillic, Turkish (`İ ı Ş ş Ğ ğ`), Polish (`ą ć ę ł ń ś ź ż`), German (`ß ä ö ü`), Portuguese (`ã õ ç`), French (`é è ê ç œ`), Italian (`à è ì ò ù`), Finnish (`ä ö`) and Lithuanian (`ą č ę ė į š ų ū ž`) letters must render with the bundled fonts. Check egui's default fonts and bundle a covering font if any glyph is missing. egui's default fonts have no Chinese glyphs, and player names can contain Chinese, Japanese or Korean in any locale, so a CJK fallback font is needed whatever the interface language. eframe's system font discovery covers this on Windows without a bundled CJK font, as the spike showed. Linux discovery through fontconfig still needs checking, and a subset is bundled only where no system font is found. Chinese text has no spaces, so labels must wrap between characters. Case-insensitive search uses Unicode case folding. In the Turkish locale `I` matches `ı` and `İ` matches `i`. Player name search works the same whatever the interface language.
- **Arabic and right-to-left text.** Arabic letters change shape with their neighbours and are written right to left, so Arabic needs text shaping and the Unicode bidirectional algorithm, not just glyphs. Arabic player names need this in every locale. The M6 text spike ([gui-text](docs/gui-text.md)) found that egui 0.37 shapes Arabic words correctly with harfrust but places text in logical order, so the GUI orders text with `unicode-bidi` and lays out each word of a right-to-left run separately. Text fields stay in logical order while editing, a known limitation. In the Arabic interface the layout mirrors (the library pane on the right, labels aligned right), while the timeline's time axis, timecodes, paths and player names keep their own direction. Names, paths and numbers inside Arabic sentences are isolated so they do not reorder the sentence. Fluent inserts isolation marks around arguments, which the text renderer must handle or the GUI must strip after isolating by layout.
- **Layout.** German, Finnish, Russian, Ukrainian, Polish and French text often runs about a third longer than English, and Finnish compound words can be long enough to need hyphenation or wider controls. Controls wrap or grow instead of truncating. A pseudo-locale that lengthens every message and adds accents is used to check each screen.
- **Translations.** First drafts may be machine-assisted but are marked unreviewed until a native-speaking player reviews the locale. Translators only edit `.ftl` files, and the About screen credits reviewers.

## 9. File safety and malformed input policy

Open originals read-only and never rewrite their directories or repair them in place. All path APIs use `Path`, `PathBuf`, and `OsStr`; Linux paths may be non-UTF-8, and Windows paths may be long/UNC. Encode native paths with a platform marker in the cache. Display lossy strings only at the UI boundary. Do not globally lowercase paths.

Before export, reject the source itself and aliases through canonical identity/hardlinks/symlinks. Reject every existing destination; version one has no overwrite mode. Sanitized suggested basenames must avoid separators, Windows reserved names, and invalid filename characters, for example `source__01m36s000-04m48s000.dem`. Do not use an exists-then-create check as race protection. `same-file` helps with identity, and `OpenOptions::create_new(true)` refuses existing files/symlinks. ([same-file](https://docs.rs/same-file/latest/same_file/), [Rust create_new](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new))

Create an unpredictable temporary file in the destination directory using `NamedTempFile::new_in`; write/patch/flush/sync and validate through its handle. Publish with a no-clobber operation such as `persist_noclobber`. Its documentation does not promise universal atomicity and describes possible leftover links after failure; handle/report that accurately. Never fall back to overwrite/replace. Clean only the temporary file created by this export. A collision can choose a new suffix or return a clear error. Treat an optional provenance JSON sidecar as a separate output, not an atomic pair with the demo. ([tempfile persistence](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist_noclobber))

Inject disk-full, permission-denied, cancellation, and commit-collision failures in tests. Verify original hashes before/after and no successful-looking target on failure. Recovery of truncated recordings is a separate explicit mode with partial coverage; a recoverable header is not authorization to export uncertain state as a normal valid clip.

## 10. Milestones, owned files, and acceptance gates

Implement in order. Each milestone ends with a committed, reproducible state and documented limitations. Do not skip the early game test to claim faster progress.

| Milestone | Primary files/ownership | Dependencies | Acceptance |
|---|---|---|---|
| M0: scaffold and evidence | Workspace manifests, CLI skeleton, `docs/dependency-audit.md`, fixture manifest format, CI | None | Windows/Linux core build; licenses/version choice recorded; local sources untouched; fixture metadata/hash inspection works |
| M1: bounded container and controls | `container/*`, CLI inspect/validate, container tests | M0 | Known fixture header/table assertions; malformed input safe; S0a exact copy, S0b raw roundtrip, S0c end-only export play in Windows game |
| M2: audited state decoder | `protocol/*`, `state/*`, profile documentation | M1 | Dynamic schemas, resources, baselines, source sequence map, complete entity/client/weapon state; codec audit fixtures and trace comparisons pass |
| M3: arbitrary-start feasibility | `slice/*`, `validate/*`, clock/groups, engine-validation record | M2 | Simple restart then two-player gauss restart; actual `[96,288)` long-file test; no hidden prefix/original dependency; packet-limit/clock/signon decisions documented; exports use only proven, undropped state from non-failed sources (§6.2), with tests for each refusal |
| M4: action/actor analysis | `analysis/*`, identity history, JSON CLI | M2; M3 remains release prerequisite | Gauss false-positive fixtures, full weapon catalog action types, unknown actor/visibility labels, singleplayer analysis fixtures |
| M5: library and cache | `library/*`, jobs, scan CLI | M4 | Recursive bounded scan at corpus scale; cache invalidation, cancellation, failure isolation; selected file priority |
| M6: desktop workflow | GUI modules, settings, locale catalogs | M3–M5 | Open/drop file/folder, filters/timeline/evidence, multiple clips, no-clobber export, responsive UI; all fourteen languages switchable without restart, catalog completeness test, CJK glyphs in names and the Chinese interface, shaped right-to-left Arabic with a mirrored layout and Arabic names shaped in every locale, correct ru/uk/pl/lt/ar plurals for 0, 1, 2, 5, 11, 21, 22 and 25, Turkish search casing, no truncation under the pseudo-locale |
| M7: compatibility and release | CI, fixture matrix, packaging, docs | M6 | Required Windows multiplayer and singleplayer engine matrix; source hashes unchanged; Linux build/core CI and documented pending live smoke tests |
| M8: playback foundation | Playback/assets crates, backend spike, viewer profile | M7 | Shared clock, asset resolution, backend decision and checkpoint/seek equivalence (§13) |
| M9: world and models | Render crate, BSP/studio/sprite adapters | M8 | Embedded world and animated models match recorded camera/state and game references |
| M10: effects and audio | Effect handlers, audio crate, fidelity fixtures | M9 | Weapon/death effects and synchronized sound survive seeking, including active loops and delayed events |
| M11: viewer editing workflow | Viewer panel and all timeline integrations | M10 | Event-click seek, scrub, step, reverse, rates and clip loops use one playback controller |
| M12: viewer qualification | Fidelity matrix, benchmarks, platform validation | M11 | Visual/audio and seek equivalence gates, measured performance and honest Windows/Linux support |

M3 is a feasibility gate, not a vague spike. Its output includes a working CLI slice path, saved trace comparison, selected clock/sequence/bootstrap policy, and game evidence. If the profile still cannot start independently, keep work focused on the failing state class and report the unmet requirement. No parser-only completion claim is acceptable.

## 11. Test plan and commands

### 11.1 Deterministic fixtures

Record short controlled fixtures with known action order, input settings, participants, map, mod/client identity, and expected observations. Prefer a small controlled multiplayer server and a documented singleplayer session. Store redistributable minimal fixtures in Git only when rights and size permit; otherwise use hashes and the local manifest. Each important semantic expectation must be checked against actual playback, not only generated by the same parser being tested.

Required gauss cases: primary; partial/full secondary charge; many pitch updates then one release; empty/underwater trigger; holster without fire; stop-only event; overcharge; charge at file/cut boundary; prediction enabled/disabled; two simultaneous actors; remote events in/out of visibility. Required identity/mode cases: POV, in-eye spectator, slot reuse, name change, unresolved entity, and singleplayer. Add HLTV fixtures when discovered or intentionally supported.

Required other weapons: Glock modes, Python, MP5 bullet/grenade, shotgun modes, crossbow zoom-only versus zoomed fire, RPG launch, crowbar swing without hit, Egon sustained activity/end, hivehand modes, blocked and successful tripmine/snark attempts, grenade pin pull versus throw, and satchel throw versus radio action. This is required action coverage, even where the correct output is “attempt” or “inferred” rather than “observed shot.”

Required slice states: no-effect basic scene, ongoing gauss charge, death/respawn, pause/unpause, player join/name change, entity appear/remove, sequence low-byte wrap, nonzero server uptime, pending delayed event, sustained Egon/looping sound, view change, cut near disconnect, and a map transition rejected or split explicitly. Test multiple slices from one unchanged source.

### 11.2 Meaningful automated assertions

- Stop-only gauss adds zero shots; fire plus stop adds one; ten spin updates add zero; two actors do not collapse; primary/charged modes differ.
- Unreliable actor mapping uses the reconstructed entity array; reliable explicit identity works; invalid identity remains unknown; resource IDs can vary between files.
- Attack buttons, ammo loss, selection, and kills do not increment observed firing counts by themselves.
- Name history and slot incarnation resolve at event time; singleplayer mode does not depend on map naming.
- Full and delta entity reconstruction matches expected snapshots, including baselines, removals, wrap, clientdata, and weapondata.
- Delta/time-window encode/decode respects verified quantization and clock domain behavior; missing bases are errors.
- Structural output has no pre-cut sequence dependency; semantic state/event traces align at first and later retained groups.
- Range parsing/group snapping handles shared timestamps, empty ranges, end exclusivity, pauses, and segment boundaries.
- Truncated/overflow/nonfinite/unknown-opcode inputs return bounded diagnostics without panic or uncontrolled allocation.
- No-clobber safety holds for same path, hardlink, symlink, Windows aliases where applicable, existing output, injected write failure, and a concurrent collision.
- Cache version/source changes invalidate results; stale worker generations cannot replace the active selection; cancellation preserves correct prior results.

Fuzz container readers, bit readers, schema/delta codecs, and message boundary decoding with time/memory limits. Add regression inputs for every discovered crash. Do not treat fuzzing as proof of playback correctness.

### 11.3 CLI contract and reproducible commands

Implement these subcommands/options consistently; JSON schemas must be versioned. `--json -` writes stdout, while a path writes a separate result file. `scan` takes one or more roots. `validate` reports structural/semantic capability without pretending to launch the game.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --workspace --locked

cargo run -p lambda-slicer-cli -- inspect 'F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem' --json -
cargo run -p lambda-slicer-cli -- analyze 'F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem' --json 'work\analysis.json'
cargo run -p lambda-slicer-cli -- scan 'F:\HLScreeniesDemos' 'F:\SteamLibrary\steamapps\common\Half-Life\ag' --recursive --jobs 1
cargo run -p lambda-slicer-cli -- slice 'F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem' --start 00:01:36 --end 00:04:48 --output 'work\naz_crossfire_96_288.dem'
cargo run -p lambda-slicer-cli -- validate 'work\naz_crossfire_96_288.dem'

$env:LAMBDA_SLICER_FIXTURE_MANIFEST = 'C:\path\to\local-fixtures.json'
cargo test -p lambda-slicer-core --test local_corpus -- --ignored
```

Create output parent directories deliberately; commands must not overwrite an existing file. Support a `slice --dry-run --json -` planning mode before writing. Test semantic source/output comparison through a dedicated validation option or integration harness; include its exact command in the implemented engine-validation record once the interface is finalized. Fuzz commands should name actual targets, for example `cargo fuzz run delta_decode`, after those targets exist.

CI runs locked builds/tests on Windows x86_64 MSVC and Linux x86_64 GNU from M0. Install and document native Linux prerequisites for the pinned GUI stack, run format/clippy, and keep game binaries/private corpus out of CI. Linux automated build/core coverage is required now even though human Linux testing is deferred. Mark Linux GUI dialogs/drag/drop/rendering and actual game playback **pending** until exercised on Linux; do not claim those passed from cross-compilation alone.

## 12. Release support matrix and completion checklist

| Capability/profile | Target | Evidence required before supported |
|---|---|---|
| GoldSrc 5/48 metadata | Initial target | Full bounded container tests and corpus header scan |
| AG 5/48 multiplayer POV analysis | Required | Known-positive/negative controlled events and real corpus samples |
| AG 5/48 independent slicing | Required | Structural + semantic + Windows game gates, including `[96,288)` |
| Half-Life/AG singleplayer analysis and slicing | Required | Actual singleplayer fixture, lifecycle/HUD/weapon and engine tests; separate profile if needed |
| Spectator actor handling | Required when encountered | Camera-versus-actor tests, explicit unknowns where unavailable |
| HLTV | Discover and diagnose; extend with fixtures | Correct empty clientdata/profile handling and dedicated playback tests |
| Network 46/47, other forks/mods | Recognize metadata where safe | Separate audited grammar/profile/fixtures before analysis/export support |
| Truncated/corrupt demos | Diagnose; optional recovery | Explicit partial coverage and no normal export from unknown state |
| Windows GUI/runtime | Required release target | Interactive smoke tests, no-clobber export, actual game validation |
| Interface languages `en`, `es-419`, `pt-BR`, `ru`, `uk`, `tr`, `pl`, `de`, `fr`, `it`, `fi`, `lt`, `ar`, `zh-Hans` | Required | Complete catalogs, each reviewed by a native speaker, screens checked in every locale |
| Linux | Architectural/CI target now | Build/core tests now; live GUI/game status explicitly pending until tested |

At completion the user can find gauss-relevant recordings in a large folder, inspect exact recorded actions and actors/unknowns, select clips, and play independently exported files in the intended game. “Every weapon” means every supported standard weapon appears with the strongest evidence actually present; the app must not fabricate universally observable shot counts. “Arbitrary interval” means requested time selection is supported with explicit group snapping and profile/state compatibility checks; a blocked state is an unmet export case, not a successful empty or prefix-preserving slice.

Before declaring release-ready, verify originals' full hashes are unchanged; independent output works without the original file; all engine-validation results include actual observations; all unresolved codec/private-state cases produce actionable blockers; the UI remains responsive at library scale; and the repository documents exact tested dependency/profile versions.

### Next implementer's start checklist

1. Read this document, repository instructions, `README.md` and milestone records; inspect the existing implementation and working tree. Preserve completed work and unrelated edits rather than recreating the scaffold. Resume at the first unfinished milestone; M8–M12 follow the current M0–M7 scope.
2. Choose/lock the Rust toolchain and dependency/license strategy; inspect the exact codec source locally and record its commit.
3. Create the workspace, CLI, bounded container reader, and optional local fixture manifest. Hash originals without changing them.
4. Run the three S0 controls in the installed Windows game and save evidence before touching arbitrary-start transformations.
5. Complete the delta/baseline/time-window/sequence/clientdata audit, then implement state reducers and M3 restart export.
6. Prove a real standalone nonzero-start clip, including the long `[96,288)` test. Resolve concrete failing state classes before GUI expansion.
7. Implement remaining detectors, batch/cache services, and the GUI against the same core APIs; complete required singleplayer and weapon fixtures.

The principal unresolved risk is exact restart fidelity for the installed client/mod's private and transient state, followed by clock/sequence behavior and packet representability. The proposed implementation makes those risks testable early. It does not claim that research alone has already solved or verified them.

## 13. Follow-on phase: integrated demo viewer and sound

**Added 17 October 2026. Implement after the current M0–M7 scope.** M8–M12 add an embedded, interactive Half-Life/AG demo viewer with synchronized audio. This phase is required follow-on work, not a replacement for independent demo export. Game playback remains the reference for fidelity and exported-file compatibility. Video-file encoding and automatic camera editing are separate future features.

The viewer must display the recorded world, players, weapons, animations, sprites, textures, effects and HUD, and play the corresponding sounds. A player killing another player is the acceptance example: the visible killer and victim have the correct models, weapon and poses; firing, muzzle flash, impacts/blood, death animation and audio occur in their recorded order. Support multiplayer POV and singleplayer, with separately certified spectator/HLTV behavior. Do not call a textured BSP with moving player markers a complete viewer.

### 13.1 Reuse and backend decision

Keep the existing Lambda Slicer decoder authoritative. Rendering consumes decoded state and ordered raw playback evidence; the compact weapon-analysis event table alone cannot drive a faithful viewer. Do not simulate combat again from attack inputs or synthesize a death animation merely because a kill-feed entry exists.

References and reusable starting points identified for this extension:

| Source | Useful starting point | Integration constraint |
|---|---|---|
| [webhl](https://github.com/x8BitRain/webhl), especially `src/hlviewerjs/{ReplayPlayer,SoundSystem}.ts`, `Parsers/`, `Graphics/` and `Replay/` | Local asset loading, demo-player organization, browser playback and audio reference | Its README documents sound-ordering and map-switch playback bugs. Audit the actual implementation and license before adaptation; it is not the fidelity oracle |
| `C:\Users\naz\source\repos\xenoxide\crates\xenoxide-bsp30` and `xenoxide-map` | Validated BSP30 reading, world geometry, visibility and material preparation | Extract only needed interfaces; Lambda Slicer must build without this developer-specific checkout |
| Xenoxide `crates/xenoxide-assets/src/studio.rs` and `studio/render.rs` | Studio-model data, textures, mesh and external dependency handling | Keep asset identity and model/mesh compatibility checks |
| Xenoxide `crates/xenoxide-game/src/studio_pose.rs`, documented in `STUDIO_POSE.md` | Explicit-time skeletal pose evaluation, controllers, blends, transitions and gait inputs | Playback supplies the animation history and movement-derived inputs; the renderer must not advance them independently |
| Xenoxide `crates/xenoxide-render/src/{viewer,studio,studio_pipeline,vulkan}.rs` | World and model GPU rendering | Current renderer owns window/swapchain/UI integration and uses a different egui version; current studio shading lacks some Half-Life lighting/material behavior |

**Default design:** render into an offscreen GPU texture using `wgpu`, composed into the existing eframe/egui window through the pinned version's texture/callback API. Reuse Xenoxide's asset and pose logic first, then port applicable world/model rendering algorithms. Keep one GPU device and avoid per-frame GPU-to-CPU readback. Backend selection alone does not establish performance.

**M8 gate:** compare a small wgpu integration with the work needed to expose Xenoxide's Vulkan renderer as an offscreen renderer. Use the same map, animated models, viewport sizes and measurement procedure. Record frame time, GPU/CPU memory, resize behavior and Windows/Linux integration cost in `docs/viewer-backend.md`. Retain wgpu unless evidence justifies switching. A Vulkan choice must specify egui version alignment, device/image ownership, synchronization and Linux window integration; do not embed an entire second application or add an unmeasured image-copy bridge.

Record exact source commits, applicable licenses, provenance and imported patches in `docs/dependency-audit.md`. Extract shared code into pinned, independently buildable crates or maintain an explicitly recorded copy. Do not import Xenoxide's gameplay/networking simulation just to evaluate a pose, or make mutable sibling paths production dependencies. Load the user's installed assets; do not bundle game content.

### 13.2 Crates and contracts

Suggested new ownership, adjusted to the existing tree rather than rewriting working modules:

```text
lambda-slicer-assets/src/       search paths, archives, bsp, wad, studio, sprite, audio assets
lambda-slicer-playback/src/     controller, clock, decode adapter, checkpoints, scene, effects
lambda-slicer-render/src/       device integration, world, models, sprites, lighting, effects
lambda-slicer-audio/src/        mixer, channels, spatialization, resampling, device output
lambda-slicer-gui/src/viewer.rs viewport, controls and playback-status presentation
docs/viewer-backend.md          measured backend decision and integration contract
docs/viewer-profile.md          message/effect coverage, clocks and asset precedence
docs/viewer-validation.md       fixture hashes, game comparisons and performance results
```

Playback depends on core and asset data; rendering and audio consume immutable presentation data, not GUI state. Keep core analysis/export usable without a GPU, audio device, installed game or viewer dependency. Extract shared reducers where useful, but preserve export-specific refusal rules: the viewer's ability to show a state does not certify that a sliced demo reproduces it.

Define commands equivalent to `Load`, `Play`, `Pause`, `SeekTo`, `StepGroup`, `SetRate`, `SetLoop` and `SetCamera`. A playback position contains demo identity, session/segment and source time; repeated server times across map changes must not collide. Commands and responses carry a load generation and monotonically increasing seek revision.

Publish `PresentedFrame` with that identity, resolved time, scene revision, camera, renderable entities, effects and HUD; publish matching audio-state changes with the same generation. A stale worker result must never move the playhead, display a frame, or start sounds after a later seek/load. Keep source time, network/server time, presentation time and audio sample time distinct, with explicit mappings.

### 13.3 Asset resolution and scene construction

Use a configurable Half-Life installation and mod root. Resolve recorded resource indices through each session's resource table. Implement and document tested mod/fallback directory, loose-file, PAK and WAD precedence; preserve custom AG models and sounds. Support embedded BSP textures, external WAD textures, studio texture companions/sequence groups, sprite palettes/frames and sky textures. Match Linux asset names deliberately without globally lowercasing filesystem paths; diagnose ambiguous case matches. Reject traversal outside approved asset roots.

Validate lengths, indices, dimensions and decoded-memory budgets before allocating or uploading. Cache assets by content identity and conversion version; changing an asset or profile invalidates affected derived data. Missing or incompatible assets produce an actionable resource/path diagnostic and visibly incomplete playback, not an unnoticed substitute claimed as accurate.

Build world meshes, lightmaps and visibility data once per map. Render moving brush submodels separately from the static world. Implement dynamic light styles/lights, animated and masked textures, translucent/additive/glow modes, sky and water behavior. Use BSP visibility plus frustum culling and measured batching.

For studio models, handle skin/body groups, controllers, sequence/frame/framerate, gait, transitions, attachments, held weapons, viewmodels and material flags. Interpolate between recorded samples according to the validated client rules; do not interpolate through teleports, entity removal/reuse, deaths that reset state or map changes. Entity identity includes its lifecycle, not just an integer slot. Preserve recorder camera, view/FOV and viewmodel animation history. Use the installed model's sequence data, not assumed sequence numbers shared by every model.

### 13.4 One clock and timeline behavior

All controls that change the selected playback time use the controller: timeline background clicks, event/kill selection, time entry, next/previous event, frame stepping, rewind, fast-forward and clip preview. Timeline zoom/pan only changes the visible time range. Editing a clip boundary can preview that boundary while leaving the stored source interval independent of the current playhead.

Clicking an event at **00:05:00** seeks directly to that recorded moment, retaining the prior paused/playing state. A separate “Play event” action seeks to configurable preroll and plays through postroll. Seeking reconstructs the skipped interval internally; the user never waits for five minutes of real-time playback. Discrete changes use a documented before/after-event ordering at identical timestamps so the selected event is not omitted or fired twice.

Provide play/pause, source-time display, single recorded-group stepping, rate presets (initially 0.25×, 0.5×, 1×, 2× and 4×), continuous backward playback, and looping a selected clip. Distinguish a recorded group from a display refresh. Animation/effect simulation advances with playback time, not the number of rendered frames. Pausing freezes scene and audio progression. Use monotonic wall time and audio-device latency measurements to maintain A/V synchronization; UI work must not independently drive either clock.

Recorder POV is the initial fidelity target. Free camera/follow-player views can be exposed with explicit recording-coverage limitations and a chosen audio listener. A POV demo cannot recover unseen entities or events it never received. Do not invent missing activity or promise that another player's full first-person view is available. Recreated visual randomness must be stable across seeks; original unrecorded random choices cannot be promised pixel-identical.

### 13.5 Checkpoints, arbitrary seeking and rewind

Start with a correct sequential playback implementation and use it as the reference. Then add versioned checkpoints indexed by source position and segment. Begin with checkpoints roughly every two seconds and at session boundaries; tune spacing against measured seek cost and memory, not a fixed performance claim.

A checkpoint must contain enough state to continue the decoder and all presentation reducers, including:

- Dynamic schemas, resource/user-message registrations, baselines, sequence/delta history and confidence flags.
- Entity/client/weapon state and identities, previous interpolation samples, camera/viewmodel, animation transitions, gait accumulators and animation-event cursors.
- Light styles, HUD values with expiry, decals, active particles/temp entities/beams and their simulation age, plus pending delayed events.
- Active audio channels with sample identity, position, pitch/volume envelopes and loop state.
- Effect RNG state or stable event-derived seeds, source cursor and clock mappings.

A current entity snapshot is not a complete checkpoint. In particular, do not reconstruct playback by exporting a temporary slice: export bootstraps and their restrictions are different from restoring the viewer's full state.

For a seek: cancel obsolete work; restore the closest compatible earlier checkpoint; replay every intervening message/group and advance active effects in chronological order; evaluate presentation at the destination; atomically replace visible and audible state. During this work suppress screen presentation and audio output, but still update effect, RNG and mixer state. Do not discard skipped events merely because they are not displayed. Long-lived effects must remain represented even if their creation precedes the checkpoint.

Rewind restores earlier state and simulates forward into a bounded recent-frame cache. Do not integrate particles, physics or animations using a negative time step. Keep assets GPU-resident and upload only changes required by the chosen frame. During dragging, coalesce obsolete seeks and publish only the newest completed revision; release must resolve the exact requested time.

Build checkpoints lazily for the selected demo, with background continuation and progress. A first jump into an unindexed region may require a longer decode; show seeking status and keep cancellation responsive. Store bounded, compressed disk caches keyed by full demo fingerprint, checkpoint schema, decoder/effect/profile versions and relevant asset hashes. Never reuse a checkpoint from another session, changed source or incompatible implementation.

### 13.6 Effects and presentation fidelity

Implement typed handlers for network events, standalone demo events, temp entities, weapon animations and supported client-private demo buffers. Recognizing a message length is not implementing its effect. Inventory actual supported AG and singleplayer behavior in `docs/viewer-profile.md`, with fixtures for each stateful handler.

Cover gauss charging/firing/stopping and its beams; sustained Egon fire/stop; muzzle flashes; shell ejection; projectile models and trails; impacts, blood and decals; explosions, smoke and sprites; death/corpse/gib presentation; and HUD/camera effects. Preserve delayed dispatch times and event ordering. Feed the same stable identity to render and audio effects. Correlate local/network duplicates only when the profile proves they represent one dispatch.

Track creation, update, expiry and explicit stop/removal for every persistent effect. Keep event-triggered and model-animation-triggered effects from firing twice during replay or pose reevaluation. Do not recreate all historical one-shot effects at the destination; advance their ages and retain only those still active. HUD notifications likewise retain their remaining lifetime instead of restarting an expired vote or countdown.

Visual effect randomness uses deterministic per-event seeds or checkpointed state. Collision-dependent effects such as shells use the relevant static/moving collision geometry and a deterministic update policy. Avoid deriving particles and sounds from render FPS. Unknown active behavior is reported as incomplete/unsupported fidelity with context; the basic viewer can remain usable, but that fixture cannot pass the fidelity gate.

### 13.7 Audio playback and seeking

Sound is required, including weapon fire and charging, ongoing weapon loops, footsteps, impacts, ambient/static sounds and game speech. Handle recorded voice streams and sentence-based sounds only through explicitly supported codecs/resolvers; diagnose unsupported ones. Audit audio backend/decoder dependencies for Windows/Linux support and licenses, then pin the selected versions. Backend choice does not implement GoldSrc channel semantics.

Use a software mixer with deterministic, inspectable state, separated from the real-time device callback. The callback consumes bounded preallocated buffers; it must not read files, decode demos, allocate unbounded memory or take locks held by asset-loading workers. Missing devices permit silent preview with a visible status. Device changes must not alter the selected demo time.

Represent channels by validated entity/channel replacement rules and retain resource, source position, start time, playback cursor, loop points, pitch, gain, attenuation and envelopes. Implement start/stop/replace and volume/pitch changes for network sounds, demo sound records, ambient sounds and event-driven sounds. Integrate sample position over pitch/rate history, rather than multiplying total elapsed time by only the latest pitch. Handle spatial panning/attenuation relative to the chosen listener and tested room effects. Use a generation tag to discard queued samples from an old seek.

Seeking into a gauss charge restores the live loop at its correct position and pitch. Seeking into the tail of a one-shot restores only its remaining portion; an already finished gunshot stays silent. Seeking backwards removes sounds/effects that had not happened yet. Pause preserves channel cursors and resume continues them. Default scrub and reverse playback are muted, while logical audio state continues to reconstruct; resuming forward playback starts from the destination state without a burst of skipped sounds. At non-unit forward rates, initially resample with the playback rate so pitch changes accordingly; pitch-preserving time stretch is optional and must be labeled if later added.

At clip loop boundaries, restore the start state and flush queued future audio before replaying. A short output de-click ramp may prevent device discontinuities but must not conceal incorrect sound timing or restart loops from zero. Compare logical mixer events and offline mixed samples independently of live device latency.

### 13.8 GUI integration and responsiveness

Embed the viewport beside the existing timeline/event table and preserve all current clip/export workflows. Add asset-root selection, missing-resource details, playback coverage, mute/master volume, rate, camera mode and renderer diagnostics. Localize every new control/status in the existing locale system; preserve the timeline's time direction in RTL layouts.

Playback decoding, checkpoint construction, asset conversion and audio decoding run outside the UI thread. Use bounded queues and immutable shared scene data. Renderer uploads follow the GPU owner's threading rules; do not spawn a separate device per demo. Reuse geometry/textures and skin model vertices on the GPU where measured to help. Size model/effect capacity for the scene, not a fixed limit of 32 total studio objects.

Only the selected demo needs detailed playback state. Prioritize explicit seeks over background library scans, cap cache and GPU/audio memory, and evict unused assets/checkpoints. Resizing, changing demos, losing an audio device or encountering unsupported content must leave analysis/export usable.

### 13.9 Milestones and acceptance evidence

| Milestone | Required result and gate |
|---|---|
| M8 playback foundation | Document backend decision; establish asset resolution, one controller/clock and sequential scene/audio state; checkpoint restore and random-seek equivalence pass before optimized seeking |
| M9 world and models | Embedded textured/lightmapped BSP, brush entities, animated studio models/attachments/viewmodel and sprites; compare recorder-view frames against the installed game |
| M10 effects and audio | Implement the weapon/effect lifecycle and sound mixer matrix, including gauss/Egon, blood/death, delayed events and active sound restoration; record A/V comparisons |
| M11 editing integration | Every time-selection path reaches the same viewer position; play/pause/rates, reverse, stepping, clip loops and cancellable scrubbing work with synchronized audio |
| M12 fidelity and performance | Pass the fixture matrix, benchmark long demos and random seeks, document exact Windows evidence and Linux build/headless coverage; live Linux claims require an actual Linux test |

Keep deterministic synthetic redistributable assets/streams for unit and headless tests, and use the external manifest for game assets and real recordings. Test both ordinary demos and independently exported slices.

Required equivalence test: reach time T by sequential playback, direct seek, seek backward from later, repeated scrub and clip looping; compare entity/pose inputs, camera, effects, pending events, HUD expiry, RNG and logical audio channels. Compare selected GPU captures and offline audio output where deterministic. Test pauses, changes in playback speed, stale worker responses, entity/slot reuse, resource changes and session boundaries.

For the killer/victim fixture, record a short controlled multiplayer sequence with all relevant actors visible, then compare original game playback and the viewer at shot, impact, death and post-death times. Include camera/viewmodel, muzzle flash, blood, sound timing and victim animation; repeat by direct seek into each moment. Add fixtures for a mid-gauss charge, sustained Egon, delayed event across a seek, reload, moving brush entity, transparent sprite, expiring HUD notification, ambient loop and singleplayer NPC death. Missing received data remains an explicit coverage limitation, not a fabricated visual.

Initial performance targets on a recorded development-machine configuration: sustained 60 FPS at 1080p on representative AG scenes, p95 warm indexed seeks within 200 ms, visible input/cancel feedback within 100 ms, and steady-state measured A/V skew within 40 ms. These are acceptance targets to measure, not achieved claims. Report cold load/index time separately, plus p50/p95 frame/seek times, peak RAM/VRAM and audio underruns. Exercise the 415 MB fixture and dense scenes. If a target is missed, profile and record the cause rather than quietly reducing effect fidelity.

### 13.10 Completion and implementation order

Finish current M0–M7 work first. At the start of M8, inspect the current code and working tree, preserve ongoing edits, pin the reference revisions, and write the backend decision before integrating borrowed rendering code. Implement sequential state and deterministic tests before checkpoint optimization; add fidelity handlers alongside their fixtures; integrate all timeline actions only through the shared controller.

This phase is complete when the user can watch and hear supported demos inside Lambda Slicer, select an event at 05:00 and reach that scene, rewind/fast-forward/jump repeatedly without stale models, missing active effects or stray audio, and preview selected clips without changing their originals. A successful in-app preview does not replace the existing in-game export tests. Release notes must distinguish certified profiles, missing assets, unrecorded activity and unsupported client effects.

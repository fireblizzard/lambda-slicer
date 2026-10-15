# Protocol profile 5/48

Status: **container layer and network decoder (M2) implemented and checked against real
recordings.** Sections: container (header, directory, outer records), then network messages and
reconstructed state. Sources and licenses are in `dependency-audit.md`.

All integers are little-endian. Floats are IEEE-754 binary32 and are preserved as raw bits.

## Header (544 bytes)

| Offset | Field | Size |
|---:|---|---:|
| 0 | `HLDEMO\0\0` | 8 |
| 8 | demo protocol, i32 | 4 |
| 12 | network protocol, i32 | 4 |
| 16 | map name, raw bytes | 260 |
| 276 | game directory, raw bytes | 260 |
| 536 | map checksum, u32 | 4 |
| 540 | directory offset, i32 | 4 |

## Directory

At the directory offset: entry count (i32), then 92-byte records: type i32, description[64],
flags i32, CD track i32, track time f32, frame count i32, offset i32, length i32.

**Observed** on every recording inspected so far (see the corpus scan below):

- Two entries: `LOADING` (type 0) then `Playback` (type 1), contiguous, starting at 544, with the
  directory immediately after the last entry.
- For the Playback entry, **frame count = number of network records** (tags 0/1) in the entry,
  not the number of records and not the last frame number. Examples: `naz_crossfire_pb_01.dem`
  151,207 network records, last frame number 151,206; `20220319_194739_fastrun.dem` 578,699 network
  records, last frame number 1,105,205.
- For the Playback entry, **track time = time of the NextSection record**.
- LOADING has frame count 0 and track time 0.

## Outer records

Each record starts with tag u8, time f32, frame number i32 (9 bytes).

| Tag | Name | Body after the 9-byte prefix |
|---:|---|---|
| 0 | Network (signon/loading) | demo info (436) + 7 × i32 sequence + i32 length + payload |
| 1 | Network | same as tag 0 |
| 2 | DemoStart | none |
| 3 | ConsoleCommand | 64 raw bytes |
| 4 | ClientData | origin 3f, viewangles 3f, weaponbits i32, fov f32 (32 bytes) |
| 5 | NextSection | none |
| 6 | Event | flags i32, index i32, delay f32, event_args_t (72 bytes) |
| 7 | WeaponAnim | sequence i32, body i32 |
| 8 | Sound | channel i32, i32 length + sample bytes, attenuation f32, volume f32, flags i32, pitch i32 |
| 9 | DemoBuffer | i32 length + bytes |

Any other tag stops decoding of that entry with `UnknownTag`; it is never guessed to be a network
record.

The 436-byte demo info is timestamp f32, ref_params (232), usercmd (52), movevars (132),
view 3f, viewmodel i32 — field layout from the `dem` crate's `types.rs` (see
`dependency-audit.md`). It is kept as a raw block at this layer.

`event_args_t` (72 bytes): flags i32, entindex i32, origin 3f, angles 3f, velocity 3f, ducking i32,
fparam1 f32, fparam2 f32, iparam1 i32, iparam2 i32, bparam1 i32, bparam2 i32.

## Terminal records

**Observed:** an entry may contain **more than one NextSection record**.
`20220413_171130_prochallenge_bhop_skip.dem` has 89 consecutive NextSection records at the end of
its LOADING entry (792 bytes after the first one), with non-monotonic times. The reader reads the
whole entry region, yields every record, and reports `repeated_terminal` (info). Non-terminal
records after the first NextSection are reported as `records_after_terminal` (warning). The
LOADING NextSection's time and frame number can be far beyond the loading records (fastrun:
15770.47 s, frame 3,284,060) and are preserved as recorded.

## Frame groups (provisional)

**Observed:** records belonging to one client frame share a frame number and time, e.g.
`[ConsoleCommand…][DemoBuffer][ClientData][Network…]`. A frame number can carry several network
records (crossfire frame 1) or none (fastrun records client frames at ~256 Hz with fewer network
records). The terminal NextSection carries the time and frame number of the frame that would
have come next.

S0c truncation therefore cuts **before the first record whose frame number differs from the
previous record and whose time is ≥ end**, and writes a NextSection with that record's time and
frame number. This grouping is provisional until confirmed by the S0c in-game test and by the
network-layer audit (plan §4.3).

## Corpus scan (2026-10-14)

`corpus_scan` example, read-only, over `F:\HLScreeniesDemos` and
`F:\SteamLibrary\steamapps\common\Half-Life\ag`: **55,049 files, 73,790,776,891 bytes, 240 s**.

| Result | Files |
|---|---:|
| Container complete, no warnings | 55,045 |
| Directory offset zero (incomplete recording, recovery not implemented) | 4 |
| Profile 5/48 `ag` | 55,044 |
| Profile 5/48 `valve` | 1 |
| Repeated NextSection records (info) | 53,975 |

No file had an unknown tag, a truncated record, a region gap or overlap, a directory frame-count
mismatch, or a track-time mismatch. The Playback-entry frame-count and track-time rules above
therefore hold for every readable file in this corpus. This is container-level evidence only; it
does not show that network payloads decode or play correctly.

# Network layer (M2)

## Bit stream

Network payloads mix byte-aligned fields and bit sections. Bits are read LSB first. A bit
section starts at a byte boundary after the opcode and ends by rounding up to the next byte
(`MSG_EndBitReading`). `ReadSBits(n)` is **sign-magnitude**: one sign bit, then `n − 1`
magnitude bits; `−0` is representable and is preserved. Strings are NUL-terminated bytes.

## Delta encoding

A delta is a 3-bit mask byte count, that many mask bytes, then each field whose mask bit is set,
in schema order. A field not sent copies the base value **at the same struct offset**; struct
offsets outside the current schema become zero (the engine parses into a zeroed struct).

| Type | Wire | Converted value |
|---|---|---|
| BYTE/SHORT/INTEGER | `bits` (unsigned) or `sbits` (signed) | `/ premultiply`, `× postmultiply` if ≠ 1, cast to the C type |
| FLOAT | same | same, as f32 |
| ANGLE | `bits` | `raw × 360 / 2^bits` |
| TIMEWINDOW_8 | `sbits(8)` regardless of declared bits | `(svc_time × 100 − raw) / 100` |
| TIMEWINDOW_BIG | `sbits(bits)` | `(svc_time × pre − raw) / pre`, or `svc_time − raw` if pre ≈ 1 |
| STRING | bytes to NUL | bytes |

Values are stored in wire form, with each time-window value carrying the f32 bits of the
`svc_time` it was decoded against, so a writer can re-encode them exactly (plan §4.2).

Schemas arrive in `svc_deltadescription`: name string, 16-bit field count, then each field as a
delta of the fixed meta-schema `fieldType (32) | fieldName (string) | fieldOffset (16) |
fieldSize (8) | significant_bits (8) | premultiply (32, ×4000) | postmultiply (32, ×4000)`.

**Observed** in the AG recordings: `entity_state_t` 52 fields, `entity_state_player_t` 49,
`custom_entity_state_t` 19, `clientdata_t` 50, `weapon_data_t` 20, `event_t` 14, `usercmd_t` 15.

## Messages

All opcodes 1–58 have a layout implemented. Opcode 0 (`svc_bad`), 19 (`svc_damage`, unused by
the engine), 59–63, and unregistered user message ids stop decoding with an error naming the
frame and payload byte; nothing is skipped by guessing. User messages are ids ≥ 64 registered by
`svc_newusermsg` (size 255 means a byte length prefix).

Layouts that differ between references, and the choice made:

| Item | Choice | Disagreeing reference |
|---|---|---|
| `svc_event` args-delta bit | Present only when a packet index was sent (engine `SV_EmitEvents`) | ReHLDS HLTV reader reads it unconditionally |
| TE_PLAYERSPRITES (121) | 5 bytes (SDK layout) | ReHLDS HLTV table: 6 |
| TE_FIREFIELD (123) | 13 bytes (SDK layout, ReHLDS) | hlviewer.js: 9 |
| TE_TEXTMESSAGE | `fxtime` only when effect = 2 (SDK, ReHLDS) | hlviewer.js: when effect ≠ 0 |
| `svc_serverinfo` trailing byte | Must be 0; nonzero is reported as unsupported | ReHLDS HLTV reads an extra block when nonzero |
| `svc_hltv` | Only command 0 (HLTV_ACTIVE) accepted | — |
| Delta packet player range | `1 ≤ number ≤ max_clients` | hlviewer.js uses `< 32` |

## Entity packets and snapshots

Each network record with an entity or clientdata update stores a snapshot under its recorded
`incoming_sequence`. Server and client delta from `frames[delta_sequence & mask]`, a ring of
**8 frames when `max_clients == 1`, else 64** (ReHLDS `SV_UPDATE_BACKUP`). The base is the newest
frame occupying the reference's slot. Usually that is the frame carrying the reference's low byte.

**Observed: references past the ring.** In `20221127_173627_bhop_crossfire_cup_us.dem` four packets
in one burst all reference byte 76. For the first three, the frame with that byte is 62–64
packets back and still in its slot. The 64-back case has 0 updates, because the server's slot is
the one it is writing. For the fourth, it is 65 back, so slot `76 & 63` already holds the frame
received just before, and that newer occupant is the base. A rule requiring the exact byte (tried
during development) rejected these, fell into inference, and failed later in the file.

**Observed: recorded sequences can drift from the server's numbering.** In some recordings
(44 files in the corpus, mostly 2023–2026 bhop/kz demos, plus a few 2019–2022 matches), the
recorded `incoming_sequence` rises by exactly 1 per recorded packet while the server's numbering,
visible through the delta bytes, sometimes rises by more. The offset starts at a small constant
(2–5) and shifts later, consistent with lost packets. In those files the playback segment starts
at a sequence unrelated to the loading segment (e.g. 134 → 59138), so the recorder evidently
stores a counter rather than the live netchan sequence. Which client builds do this is not
established.

### Base selection: strict versus inferred

| Resolution | Rule | Proven? |
|---|---|---|
| `strict` | the newest snapshot in ring slot `from & (backup − 1)`, if it is 1 to `backup` packets old, and (for entity packets) its rebuild reproduces the declared entity count. An older occupant means the slot's real content was never recorded | yes, unless the session's numbering is already suspect |
| `offset` | matched under the sequence offset inferred earlier in the session, rebuild reproduces the declared count | no |
| `count_search` | the nearest of the last 64 snapshots whose rebuild reproduces the declared count; its offset is adopted | no |
| `empty` | nothing received yet, so an empty base | no |

- **Equal counts prove nothing.** Two snapshots with the same entities but different values both
  pass the count check, so a count-search or offset base is a guess.
- **Once any count search happens, the session's numbering is suspect** and strict resolution is
  disabled until the next `svc_serverinfo`. A same-byte match would name a frame offset by the
  drift; it passes the count check and goes unnoticed by the client-recorded checks, because
  origin is resent every frame. In `20230921_185958_bkz_goldbhop.dem` that path would have
  claimed 633 strict bases.
- **Uncertainty propagates.** Every snapshot carries `inferred`: set when its base was not
  strict or was itself inferred, or when clientdata had no base clientdata to delta from. Only a
  full update (no delta base) clears it. Reports give `inferred_snapshots`, the first location
  needing inference, and counts per resolution.
- Clientdata in the same packet carries the same delta byte, so it uses the entity packet's choice.
  Clientdata in a packet without entities uses strict or offset resolution only, else an empty base.
- The cross-check counts disagreements on inferred state separately. They make a file
  *inconclusive*, not *failed*, and disagreements on proven state make it *failed*.

**History of this rule.** An earlier version resolved the reference like the engine ring
(`delta & mask`, newest snapshot in that slot) without limiting how old that occupant was, and
accepted any base that reproduced the declared count. In goldbhop that accepted a stale
loading-segment snapshot (sequence 128, slot `64 & 63`, about 212,000 packets older) with the
same 22 entities, so every later delta lacked the `weapons` value the server had sent:
weapon bits disagreed with the client in 1,608 of 1,638 records while being reported as complete
and uninferred. With the current rule the file reports 0 strict and 634 offset bases, its weapon
bits and origin agree 1,638 of 1,638, and its verdict is *inconclusive*.

Removing an entity that is not in the base is ignored, as the engine does, and counted.

- `svc_packetentities` (full): each entity is built from an instanced baseline, an earlier entity
  in the same packet (`offset`), or the spawn baseline of its number.
- `svc_deltapacketentities`: entities of the base not mentioned are carried over unchanged;
  updates delta from the base entity; removals drop it; new entities start from an instanced or
  spawn baseline. Entities without a spawn baseline start from zero (counted, not an error).
- **Check:** the reconstructed entity count must equal the count the server declared in the
  message. A mismatch is an error. This is the main independent evidence that reconstruction
  matches the server.
- `svc_clientdata`: clientdata deltas from the base snapshot's clientdata; weapon data copies the
  base's 64 slots and applies per-slot deltas. HLTV recordings have an empty body.

## Events and actors

`svc_event` `packet_index` is resolved against the **current packet's reconstructed entity
array**. An index equal to the array length is the server's fallback for an invoker that is not
in the packet; the explicit `entindex` in the arguments is used then. Reliable events and
standalone demo `Event` records use their explicit `entindex`. Anything else is reported as
unresolved, never guessed.

### Player identity at event time

Each event records the player behind its actor entity *when the event was decoded*: segment,
slot (entity − 1 for entities 1 to `max_clients`), user id, incarnation and name. Later renames or
slot reuse never rewrite it. Entities outside the player range get no identity, and a player
entity whose slot is empty is reported as an unknown player.

- `svc_updateuserinfo` with a cleared userinfo string means the client dropped. **Observed** in
  `20250927_182948_dm_dust2_match_tdm.dem`: the server's `McCree dropped` print arrives in the same
  packet as the cleared userinfo for that slot.
- A slot's incarnation increments when its user id changes or after it was vacated, so a reused
  slot is a different player. The same match shows `JrFury` leaving and rejoining as user 80 (slot 3,
  incarnation 1) and `tarrhium` later taking slot 3 (incarnation 2).
- Reports carry a `name_history` of joins, renames and departures with their locations. Resent
  identical userinfo is not a change.

**Observed** in `naz_crossfire_pb_01.dem` (two players): the recorder's own weapon events appear
as standalone demo events (entity 1), the opponent's as `svc_event` resolved through the packet
array (entity 2), and OpenAG's delayed gauss stop as `svc_event_reliable` with `bparam2 = 1`.
Every event in that file resolved to an actor.

## Corpus decode (2026-10-14)

`decode_scan` example, read-only, over both local roots: **55,052 files, 103,171,395 network
frames, 404,821,263 messages, 1,170 s.**

| Result | Files |
|---|---:|
| Network decode complete, every entity packet matches its declared count | 55,048 |
| — of which no delta base had to be inferred | 55,004 |
| — of which some delta bases were inferred (`inferred_delta_bases`) | 44 |
| Directory offset zero (incomplete recording, recovery not implemented) | 4 |

No opcode without a known layout, no unregistered user message, and no grammar error occurred.
This shows the grammar and reconstruction agree with the server's own counts across the corpus;
it does not prove every field value matches what the game client computed. That needs the
engine trace comparison still open under M2. `max_delta_distance` is measured in recorded
numbering and is meaningless for files with drifting sequences.

# Validation against client-recorded values (M2)

The recording client writes its own view of the game into every demo: the 436-byte demo info
before each network record, and standalone `ClientData` records. These are produced by the game,
not by our decoder, so they are an independent check of decoded state
(`lambda-slicer crosscheck`, `validate::crosscheck`).

## Demo info layout

Offsets within the 436 bytes: timestamp 0; `ref_params_t` 4–235 (vieworg 4, viewangles 16,
intermission 72, paused 76, spectator 80, onground 84, waterlevel 88, simvel 92, simorg 104,
viewheight 116, health 144, maxclients 176, viewentity 180, playernum 184); `usercmd_t` 236–287;
`movevars_t` 288–419 (16 floats, int footsteps, 32-byte sky name, 8 floats); view 420;
viewmodel 432. `ClientData` record: origin 0, viewangles 12, weapon bits 24, fov 28.

**Observed:** both record the client state from **before** the network packet of the same frame
(every identity and server-copy check matches the "before" alignment).

## Checks and what they prove

| Kind | Check | Expectation |
|---|---|---|
| Identity | demo info max_clients, playernum vs `svc_serverinfo` | Exact in every frame |
| Identity | demo info viewentity vs `svc_setview` (state before or after the packet) | Exact in every frame |
| Identity | demo info `movevars_t` physics fields vs `svc_newmovevars` | Bit-exact in every frame |
| Server copy | `ClientData` origin vs decoded clientdata origin, while viewing through the player | Bit-exact |
| Server copy | `ClientData` weapon bits vs clientdata `weapons` | Exact |
| Client view | demo info health, waterlevel, onground, view height, simorg, simvel | Informational: predicted or HUD values |
| Client view | `ClientData` fov vs clientdata `fov` (0 = client default, up to two packets of lag) | Informational: client builds differ |
| Client view | demo info sky name vs `svc_newmovevars` sky name | Informational: the client copy sometimes reads `desert` |

Skipped and counted, never compared:

- records before the first playback packet (only loading-segment state exists then);
- frames where the client is spectating or in intermission;
- **level changes inside a recording**: from a new `svc_serverinfo` to the new session's first
  full entity update, plus packetless or empty-packet frames just before it. In singleplayer the
  client resets its state before the new serverinfo packet arrives, and the new session's full
  update can arrive in the same packet as its serverinfo;
- `ClientData` origins while the view is set to another entity (a camera): the record then holds
  that entity's position (seen in `funny_nj_tricky.dem` at a `svc_setview 174`).

Findings that shaped the checks:

- **fov is client behavior, not server copy.** In `naz_crossfire_pb_01.dem` every zoom shows the
  client applying the server's fov one or two packets late (21 zoomed crossbow shots, 21
  transitions). In other recordings (e.g. the 2023 `crossfire` matches and
  `20260929_001857_bootbox_match_tournament.dem`) the client keeps its own fov (105 or 108) while
  the server sends 20 or 105, in up to 97% of frames, while origin and weapon bits in the same
  files agree 100%.
- **The movevars sky name** in demo info reads the default `desert` in up to 46% of frames of
  some older recordings while the server sent another sky. Every physics field is bit-identical
  in those same frames.

**Observed in `naz_crossfire_pb_01.dem`:** 300,176 of 300,176 server-copy samples agree, and
identity checks are 100%. In that recording even the predicted `simorg`/`simvel` equal decoded
clientdata bit for bit.

**The local player's entity origin is never sent** to that player (Half-Life SDK
`dlls/client.cpp` `Player_Encode`: "Never send origin to local player, it's sent with more
resolution in clientdata_t structure"); it stays at its baseline. Remote players' event origins
are zeroed by the server when the invoker is in the packet (`SV_EmitEvents`), so POV demos give no
client-recorded counterpart for remote entity positions; remote entity state is checked only
through the declared entity counts and event actor resolution.

**Inferred delta bases are visibly imperfect.** In `20190630_232530_crossfire_[Ehll]_Indica.dem`,
the only `ClientData` origin outliers (4 records, 3.6–5.2 units) sit exactly at t = 279.76, the
point where a delta base had to be chosen by entity count. This is why `inferred_delta_bases`
files must not be trusted for export without further evidence.

## Corpus cross-check (2026-10-15)

`crosscheck_scan` example, read-only, over both local roots: 55,052 files (4 without a directory),
1,916 s.

| Kind | Check | Agreement |
|---|---|---|
| Identity | max_clients | 97,986,796 / 97,986,797 |
| Identity | playernum | 97,986,797 / 97,986,797 |
| Identity | viewentity | 97,986,796 / 97,986,797 |
| Identity | movevars physics | 97,928,799 / 97,986,797 (99.94%) |
| Server copy | `ClientData` origin, bit-exact | 189,831,373 / 189,836,785 (99.997%) |
| Server copy | `ClientData` weapon bits | 189,844,297 / 189,846,514 (99.999%) |
| Client view | fov / health / waterlevel / onground / view height / sky name | 99.79% / 99.99% / 99.95% / 97.33% / 96.97% / 99.43% |
| Client view | predicted simorg / simvel bit-exact | 35.9% / 36.7% (prediction) |

**54,991 of 55,042 files have every server-copy sample in agreement.** Of the 51 that do not,
18 have `inferred_delta_bases`. The others are **not explained**; at least one is a confirmed
decoder fault (below), and the rest must be treated the same way until investigated.

**Confirmed decoder fault: `20230921_185958_bkz_goldbhop.dem`** (weapon bits disagree in 1,608 of
1,638 records while origin is bit-exact; not flagged as inferred). The server sends
`weapons = 0x80000000` in the first three full clientdata updates and never again. At sequence
212,548 its delta references byte 64, meaning the previous packet under a recorded-sequence offset
of 3. The ring slot `64 & 63 = 0` still held a stale **loading-segment** snapshot (sequence 128)
with the same entity count (22), so the count check passed, nothing was flagged, and every later
delta built on a base without `weapons`. This is the failure mode an external review predicted:
an entity-count match does not prove the base is right, and a stale slot can be accepted
silently. **Fixed** by the strict/inferred base selection above: the file now agrees 1,638 of
1,638 and is reported as inferred (*inconclusive*), not as proven.

Movevars identity misses come from:

- single frames right after a level change, where the client still holds the previous map's
  movevars (e.g. `20221217_062714_kz_cg_venice.dem`, sky `green` vs `dusk`), even compared with
  the movevars from before the packet;
- **client-side differences:** in `20220102_191414_crossfire_match_tdm.dem` the recording
  client used `stepsize` 224 for the whole demo while the server sent 18 and never resent
  movevars. The identity check correctly flags this.

## Sample cross-check with verdicts (2026-10-15)

Full-corpus passes take 20–35 minutes, so routine checks use a 2,000-demo sample
(`make_sample`): the 81 demos earlier scans flagged plus 1,919 drawn round-robin across all 224
maps with seed 20261015. `crosscheck_scan --list work/sample-2000.txt`, 530 s:

| Outcome | Demos |
|---|---:|
| verified | 1,894 |
| inconclusive | 19 |
| failed | 83 |
| no directory (not decoded) | 4 |

Most failures are not yet triaged. They fall into: single movevars frames right after level
changes; single `ClientData` origin records with jumps of hundreds to thousands of units
(plausibly teleports or level edges, unconfirmed); and files with many disagreements
(`20230921_185958_bkz_goldbhop.dem`, the `kzmo_downhopunvis` and `bhop_mann` recordings, files
with inferred bases). The goldbhop case is the confirmed delta-base fault above; the others are
expected to share it until shown otherwise.

### After strict/inferred base selection (2026-10-15)

Same sample and command, 664 s:

| Outcome | Before | After |
|---|---:|---:|
| verified | 1,894 | 1,893 |
| inconclusive | 19 | 37 |
| failed | 83 | 66 |
| no directory | 4 | 4 |

Most remaining failures are single records, an origin jump of hundreds to thousands of units or
one movevars frame at a level change. The `kzmo_downhopunvis` recordings still disagree on
hundreds of samples whose bases were all strict. That fits a drifting sequence offset that never
caused a count failure, which this rule cannot detect (a drift of `k` makes a same-slot match
name the frame `k` packets too old, with the same entity count). They need their own
investigation before anything built on them is trusted.

## What M2 validation does and does not establish

- Compared and agreeing: decoded clientdata **origin** and **weapon possession bits** against the
  client's own copies, plus server/session identities, in the files with outcome `verified`.
- **Not compared:** the 64 reconstructed `weapon_data_t` slots (clip, attack and reload timers) and
  any other clientdata field. Nothing in a POV demo records them independently, so their decoding
  is unvalidated.
- **Not compared:** remote entities. They have no client-recorded counterpart in POV demos; their
  state rests on the declared-count check and event actor resolution, which cannot detect a wrong
  base with the same entity count.
- Per-file verdicts come from `lambda-slicer crosscheck` (`verified` exit 0, `failed` exit 1,
  `inconclusive` exit 2). Client-view checks never affect the verdict.

# Protocol profile 5/48 — container layer

Status: **container layer implemented and checked against real recordings.** Network message
payloads (svc_* grammar, delta schemas, entity state) are not decoded yet; that is milestone M2.

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

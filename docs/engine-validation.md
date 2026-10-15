# Engine validation record

Plan §6.1 requires S0 controls to pass in the installed Windows game **before** any arbitrary-start
(S1) work. This file records the generated controls and, once played, the observations.

## Environment

| Item | Value |
|---|---|
| Game install | `F:\SteamLibrary\steamapps\common\Half-Life` |
| Mod | `ag` (liblist labels AG 6.7/openag.pro; server DLL is Metamod — customized install) |
| Exact engine/client build | **not yet recorded** — fill in from the game console `version` output |

## S0 controls — generated and played 2026-10-14

Source: `F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem`, 97,633,668 bytes,
BLAKE3 `09c31cabbfab0fde200185461ab78a2558a1261304fb7e3243b687c65208740e`.
The source hash was verified unchanged before and after every export.

| Control | Command | Output | Output BLAKE3 | Structural |
|---|---|---|---|---|
| S0a exact copy | `control exact-copy SRC --output work\ls_s0a_crossfire.dem` | 97,633,668 bytes | `09c31cab…740e` (identical) | pass |
| S0b roundtrip | `control roundtrip SRC --output work\ls_s0b_crossfire.dem` | 97,633,668 bytes | `09c31cab…740e` (**byte-identical**) | pass |
| S0c truncate `[0, 4:48)` | `control truncate-end SRC --end 4:48 --output work\ls_s0c_crossfire.dem` | 23,266,328 bytes | `b864040fb2a009cc63be243a9545809f66613ea734f374b0107f2d8d546fbcf9` | pass |

S0c detail: requested end 00:04:48.000, effective end 00:04:48.002 (next frame-group boundary);
36,100 network records kept; directory frame count 36,100, track time 288.0027.

Also: S0b on `20220413_171130_prochallenge_bhop_skip.dem` (89 repeated LOADING terminals) is
byte-identical (BLAKE3 `a462e26f…a81a`).

Because S0a and S0b are byte-identical to the source, they prove the writer reproduces the
container exactly; their in-game run mainly confirms the test procedure. **S0c is the first
output whose bytes differ from any original and is the real gate.**

### Procedure (to run in the game)

1. Copy `work\ls_s0a_crossfire.dem` and `work\ls_s0c_crossfire.dem` into
   `F:\SteamLibrary\steamapps\common\Half-Life\ag\`.
2. Start AG normally, open the console, run `playdemo ls_s0a_crossfire`. Confirm it plays like the
   original.
3. Run `playdemo ls_s0c_crossfire`. Check:
   - it starts like the original;
   - it plays for about 4:48 (`host_framerate`/`timedemo` not used);
   - the last scene matches the original at ~4:48;
   - it stops cleanly: no error, hang, or disconnect message in the console.
4. Record results below, with the console `version` output and any console errors.

### Results

| Control | Date | Result | Observations |
|---|---|---|---|
| S0a | 2026-10-14 | pass | Played back to back with S0c in one session; plays like the original. |
| S0b | — | pass by identity | Byte-identical to the source, so not played separately. |
| S0c | 2026-10-14 | pass | Plays correctly and looks like the original. On first viewing, lighting looked different (some walls darker, others brighter). A/B in one session showed this affects **whichever demo is played first**, regardless of file. Not a slicer issue: the S0c output differs from the source in only 108 bytes (header directory offset and the regenerated terminal record + directory); everything played is byte-identical. |

**Testing note:** in this install, the first `playdemo` of a game session renders lighting
differently. For visual comparisons, play a throwaway demo first, or compare runs only after the
first playback.

Still to record: the console `version` output for the exact engine/client build.

## S1 (nonzero start)

First restart output, generated 2026-10-15 (layout in `docs/protocol-profile-5-48.md`, "Restart
export"):

```powershell
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem --start 1:36 --end 4:48 --output work\ls_s1_crossfire.dem
```

Requested `[1:36, 4:48)`, effective `[00:01:36.002, 00:04:48.002)`. One bootstrap full update
and 54 persistent messages precede 24,067 kept network records. Before publishing, every kept
snapshot decoded identical to the source except time-window fields carried from 4 `animtime`
values older than the 8-bit window allows. The output's own cross-check is *verified*.

### Procedure (to run in the game)

1. Copy `work\ls_s1_crossfire.dem` into `F:\SteamLibrary\steamapps\common\Half-Life\ag\`.
2. Start AG, play any demo once first (the first playback of a session renders lighting
   differently), then run `playdemo ls_s1_crossfire`.
3. Check, comparing with the original from 1:36 if useful:
   - it opens directly on the scene at 1:36 of the original, with no visible prefix;
   - players, models, weapons and the camera are right from the first frame, and nothing is
     frozen or missing;
   - the HUD is right from the start: health, armour, ammo, current weapon and clip, timer and
     scoreboard (Tab);
   - gauss charge and discharge, sounds and effects behave like the original;
   - it plays for about 3:12 and ends on the original's scene at 4:48 with a clean stop;
   - no console errors, overflow or disconnect messages.
4. Note anything that looks different in the first second, such as animation phases on
   decorative entities (the clamped `animtime` values), and record the results below.

### Results

| Test | Date | Result | Observations |
|---|---|---|---|
| S1 crossfire `[1:36, 4:48)` | 2026-10-15 | pass | Plays on its own from the 1:36 scene to the 4:48 scene and stops cleanly. No problems seen. |

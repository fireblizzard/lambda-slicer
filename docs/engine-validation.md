# Engine validation record

Plan §6.1 requires S0 controls to pass in the installed Windows game **before** any arbitrary-start
(S1) work. This file records the generated controls and, once played, the observations.

## Environment

| Item | Value |
|---|---|
| Game install | `F:\SteamLibrary\steamapps\common\Half-Life` |
| Mod | `ag` (liblist labels AG 6.7/openag.pro; server DLL is Metamod — customized install) |
| Exact engine/client build | Protocol 48, exe version 1.1.2.2 (valve), build 8684 of Aug 3 2020 (console `version`, 2026-10-17) |

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

That output was generated before the bootstrap group stopped copying the events of the last
group before the start. Regenerated outputs differ from it only in that group.

### S1 gauss charge across the start

```powershell
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\09aug2018\naz_crossfire_pb_01.dem --start 1:59.8 --end 2:10 --output work\ls_s1_gauss.dem
```

The recorder starts charging the gauss at 1:59.119 (spin-up animation and spin sound) and fires
at 2:00.338. The slice starts 0.7 s into the charge, so its bootstrap group carries the spin-up
animation and restarts the spin sound at the current pitch (see "Ongoing gauss charge" in
`docs/protocol-profile-5-48.md`).

Check, comparing with the original from 1:59.8:

- the spin sound is audible from the first frame and keeps rising in pitch;
- the view model shows the gauss spinning up, not idle;
- the charged shot at about 0:00.5 of the slice has its sound, beam and effects;
- the rest plays like the original until 2:10.

| Test | Date | Result | Observations |
|---|---|---|---|
| S1 gauss `[1:59.8, 2:10)` | 2026-10-15 | pass | Spin sound present from the first frame, charged shot has its sound, nothing odd. The view model spin-up was not checked: the custom gauss model in this install has no visible spin-up (a single colour). |

### S1 AG match HUD state

Source: `F:\HLScreeniesDemos\01jan2022\20210418_004941_vengeance.dem`, a 41-minute AG team
match with five players. Two slices test the replayed AG messages:

```powershell
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\01jan2022\20210418_004941_vengeance.dem --start 13:21 --end 13:51 --output work\ls_s1_vote.dem
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\01jan2022\20210418_004941_vengeance.dem --start 20:05 --end 20:15 --output work\ls_s1_sudden.dem
```

- `ls_s1_vote` starts during a vote (Vote messages at 13:18, 13:20 and 13:22). Its bootstrap
  group replays the user info of all five players (slot 4 joined at 4:35), `Settings`, the
  latest `Vote`, `Location` for teammates, `Timer` and the scores. 10 bootstrap frames.
- `ls_s1_sudden` starts one second after `SuddenDeath` at 20:04. 13 bootstrap frames.

Both decode identical to the source apart from time windows, and their own cross-checks are
*verified*.

Check, comparing with the original at the same times:

- the vote display is on screen from the first frame and updates at about 0:01;
- the AG timer shows the right remaining time from the start (it may lag by up to a second
  until the next `Timer` message);
- the scoreboard (Tab) lists all five players with the right names, teams and scores;
- teammate locations show as in the original;
- in `ls_s1_sudden`, the sudden-death notice or state shows as in the original;
- health, armour, ammo and the current weapon are right from the first frame.

| Test | Date | Result | Observations |
|---|---|---|---|
| S1 vengeance vote `[13:21, 13:51)` | 2026-10-15 | partial | Scoreboard and teammate locations right from the start. The vote display appeared only about 1 s in and the timer about 0.5 s in, with the next real messages. Cause and fix below, retest pending. |
| S1 vengeance sudden death `[20:05, 20:15)` | 2026-10-15 | not testable | 20:05 is already the end-of-match screen, which hides the HUD, so the sudden-death state cannot show. |

Cause: OpenAG shows Timer and Vote for a window after each message, timed from the HUD clock,
which only moves when a frame is drawn. The replayed messages were read before any frame at the
restart time, so their window had already passed. HUD messages now arrive in a group of their
own after the bootstrap group (see "Restart export" in `docs/protocol-profile-5-48.md`).
Regenerate `work\ls_s1_vote.dem` with the command above to retest.

| Test | Date | Result | Observations |
|---|---|---|---|
| S1 vengeance vote, HUD group | 2026-10-15 | pass | Vote display and timer show from the first frame, with everything else as before. |

### S1 effects under way at the start

Effects that started before the cut are carried over it (see "Effects under way at the start"
in `docs/protocol-profile-5-48.md`). Two slices from the sample exercise the common cases:

```powershell
.\target\release\lambda-slicer.exe slice 'F:\HLScreeniesDemos\09aug2018\dblcross_salchi_naz.dem' --start 2:00 --end 2:20 --output work\ls_s1_loop.dem
.\target\release\lambda-slicer.exe slice 'F:\HLScreeniesDemos\09aug2018\ag_bhop_master_beta_naz1.dem' --start 1:00 --end 1:20 --output work\ls_s1_text.dem
```

- `ls_s1_loop`: the ambient loop `ambience/alien_blipper.wav` on entity 23 is playing at 2:00
  and stopped later, so it is started again at the start.
- `ls_s1_text`: the HUD text "Time: 00:56 | CPs: 0 | TPs: 0" (channel 1, held 256 s) is sent
  again with its remaining time.

Both decode identical to the source and cross-check *verified*. Check, against the original at
the same times:

- the blipper sound is audible from the first frame near its source, like in the original;
- the timer text shows from the first frame and keeps counting as in the original.

Not yet tested in the game: the recorder's egon fire across the cut, which is carried as a
startup event and should show the beam (with a wind-up sound) from the first frame.

| Test | Date | Result | Observations |
|---|---|---|---|
| S1 sound loop `[2:00, 2:20)` | 2026-10-16 | matches original, not audible | The loop (started 1:57.8, stopped 2:20.2) is not audible at this point in the original either, so the carry cannot be judged by ear here. |
| S1 HUD text `[1:00, 1:20)` | 2026-10-16 | pass | The timer text shows from the first frame and updates like the original. |

## M7 release matrix

The release gate (implementation plan §10, M7) needs the Windows game to play multiplayer,
singleplayer and spectator cuts, including cuts that start after a level change. M7 made a
restart start in any session of a recording: the output's LOADING entry is then the setup the
session recorded in playback, from its `svc_serverinfo` to its `svc_signonnum` ("Later sessions"
in `docs/protocol-profile-5-48.md`). Generated 2026-10-17 with the release CLI:

```powershell
$sp = 'F:\SteamLibrary\steamapps\common\Half-Life\ag\run_92.dem'
.\target\release\lambda-slicer.exe slice $sp --start 0:03 --end 0:10 --output work\ls_m7_sp_session2.dem
.\target\release\lambda-slicer.exe slice $sp --start 0:30 --end 0:50 --output work\ls_m7_sp_session3.dem
.\target\release\lambda-slicer.exe slice $sp --start 1:30 --end 1:40 --output work\ls_m7_sp_session5.dem
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\05oct2019\ray_vs_alex_cross.dem --start 0:30 --end 0:50 --output work\ls_m7_spectator.dem
.\target\release\lambda-slicer.exe slice F:\HLScreeniesDemos\19jun2019\stalkx_5o5_asasasdgwgw.dem --start 1:36 --end 4:48 --output work\ls_m7_mp_4v4.dem
```

| Output | Source and kind | Effective range | Kept snapshots | Output BLAKE3 |
|---|---|---|---|---|
| `ls_m7_sp_session2` | `run_92.dem`, singleplayer speedrun, session 2 (`c3a1b`) | 00:00:03.000 to 00:00:10.001 | 1,683 | `a449bd51…b1ae2ccc` |
| `ls_m7_sp_session3` | session 3 (`c3a2e`) | 00:00:30.003 to 00:00:50.002 | 4,956 | `15bcff96…0f5f99aab` |
| `ls_m7_sp_session5` | session 5 (`c3a2a`) | 00:01:30.001 to 00:01:40.003 | 1,968 | `ed7e2aba…3a40d92880` |
| `ls_m7_spectator` | `ray_vs_alex_cross.dem`, spectating throughout | 00:00:30.011 to 00:00:50.015 | 1,513 | `b51961a5…c39cb` |
| `ls_m7_mp_4v4` | `stalkx_5o5_asasasdgwgw.dem`, 5 against 5 AG match on `stalkx`, also what the app's Export writes | 00:01:36.009 to 00:04:48.000 | 17,797 | `43534322…aedbc1e` |

Every kept snapshot decoded identical to the source apart from approximated time windows. The
sources' BLAKE3 hashes were the same before and after (`run_92.dem` `9fbf2e4f…c5ffb0`,
`ray_vs_alex_cross.dem` `997e496b…315338`, `stalkx_5o5_asasasdgwgw.dem` `f64eaf60…835f50`), and
`exports_leave_every_fixture_unchanged` in `tests/local_corpus.rs` repeats that for every fixture.

### Procedure (to run in the game)

1. Run `version` in the game console and record the output in the environment table above.
2. Copy the five `work\ls_m7_*.dem` files into `F:\SteamLibrary\steamapps\common\Half-Life\ag\`.
3. Play any demo once first, then `playdemo` each, for example `playdemo ls_m7_sp_session3`.
4. Check, against the original at the same times:
   - the singleplayer cuts open on their own map (`c3a1b`, `c3a2e`, `c3a2a`), not on `c3a1a`
     where the recording begins, with the world, monsters, doors and lifts in place;
   - the HUD (health, suit, ammo, current weapon) is right from the first frame and nothing of the
     previous level shows;
   - the spectator cut shows the spectated player's view as the original does, with the
     spectator HUD;
   - the 5 against 5 cut is like the earlier crossfire test: right from 1:36 to 4:48, with team
     scores and the AG HUD;
   - each plays to its end and stops cleanly, with no console errors, overflow or disconnect.

### Results

| Test | Date | Result | Observations |
|---|---|---|---|
| Engine `version` | 2026-10-17 | recorded | In the environment table above. |
| `ls_m7_sp_session2` | 2026-10-17 | pass | Plays well, in both generations. |
| `ls_m7_sp_session3` | 2026-10-17 | fail, then pass | First generation: `playdemo` printed "Playing demo from …" and nothing else happened, the console stayed up. Regenerated with startup records, it plays cleanly. |
| `ls_m7_sp_session5` | 2026-10-17 | fail, then pass | As session 3. |
| `ls_m7_diag_s3_early`, `ls_m7_diag_s5_early` | 2026-10-17 | fail, then pass | As session 3, ruling out the clock jump. Regenerated, both play cleanly. |
| `ls_m7_spectator` | 2026-10-17 | pass | Plays well. |
| `ls_m7_mp_4v4` | 2026-10-17 | pass | Opens on `stalkx` and plays well. |
| `run_92.dem` itself | 2026-10-17 | pass | Plays through every level change. |

**Cause of the session 3 and 5 failures.** The first generation wrote the later session's setup
into LOADING as ordinary network records (tag 1), keeping their source times, 22.8 s and 86.6 s.
Real LOADING entries hold startup records (tag 0), which the game reads at once whatever their
time (176.7 s in `run_92.dem`, 1,210.6 s in `naz_crossfire_pb_01.dem`). The game holds ordinary
records until its playback clock reaches their time, so the cuts would only have started after
about 23 s and 87 s. Session 2's setup is timed 1.95 s, which went unnoticed. Two diagnostic cuts
starting right after their level loaded (`ls_m7_diag_s3_early` `[0:23, 0:30)` and
`ls_m7_diag_s5_early` `[1:27.5, 1:35)`) failed the same way, which ruled out the server clock jump
at the start. The setup records are now written as startup records, the synthetic later-session
test checks their tags, and the singleplayer cuts above were generated again (hashes above).
All of them then played cleanly in the game, with no issue noticed, which completes the M7 engine
matrix.

After that test a review found that these cuts kept the source header's map, `c3a1a`, so the
library listed them under the wrong map. Their header now names the session's map with checksum
0 (protocol profile, "Later sessions"), and the singleplayer cuts were generated once more with
the hashes above. Only the header's map name changed, as `run_92.dem` already has checksum 0, and
the game showed it does not check that name by playing the cuts with the wrong one. A multiplayer
cut after a map change, whose source header has a real checksum, has not been played yet.

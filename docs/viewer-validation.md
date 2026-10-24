# The demo player compared with the game

How the demo player (the viewer) is compared with the game, and what the comparisons found. The viewer's rules are listed in the [viewer profile](viewer-profile.md), each marked
**Implemented** until a comparison here confirms it.

## Status

The game's frames were recorded on 2026-10-19 and compared with the viewer's (below). What matches
is confirmed for those three clips. What differs is listed under "Found" and not yet fixed.

## Comparison clips

Three short clips cut with `lambda-slicer slice`, so both the game and the viewer play the very same
files. Their names keep the label `9e` of the step they were made in. A clip's clock starts at 0 at its cut, so 2.2 s into `ls_9e_stalkx.dem` is 22.2 s into its
source.

| Clip | Source | Cut | What it shows |
|---|---|---|---|
| `ls_9e_stalkx.dem` | `stalkx_naz_248.dem` | 20 to 32 s | first person on stalkx, the bot, a gauss and a crossbow kill, item sprites |
| `ls_9e_cross.dem` | `ray_vs_alex_cross.dem` | 15 to 30 s | a spectator through a player's eyes on crossfire, the sky, other players |
| `ls_9e_run.dem` | `run_92.dem` | 6 to 22 s | singleplayer on c3a1b, aliens, the crowbar, dark corridors |

## Recording the game's frames

In the game (Half-Life 1.1.2.2, build 8684, AG), windowed at 1280 by 720, with the HUD, crosshair and
counters off so the frames show the scene alone:

```
hud_draw 0; crosshair 0; net_graph 0; cl_showfps 0; r_drawviewmodel 1
playdemo ls_9e_stalkx; startmovie ls9e_stalkx 10
```

`startmovie` writes a frame for every tenth of a second of the game's clock into the mod's folder,
loading screens included. After the demo ends, `endmovie` stops it. The same for `ls_9e_cross` and
`ls_9e_run`. The game's `gamma`, `brightness`, `texgamma`, `lightgamma` and `gl_overbright` at
the time are noted with the frames, for the viewer to draw with the same (`--viewer-gamma`).

## Capturing the viewer's frames

```
lambda-slicer-gui F:\SteamLibrary\steamapps\common\Half-Life\ag\ls_9e_stalkx.dem --game F:\SteamLibrary\steamapps\common\Half-Life --viewer-gamma 2,2,2,2 --viewer-capture 0,0.1,0.2 FOLDER --viewer-size 1280x720
```

saves the view at each time as `FOLDER/<seconds>.png`, waiting at each until the frame, its map
and every model it draws are in, drawn with the game's `gamma`, `brightness`, `texgamma` and
`lightgamma`, and `gl_overbright` when a fifth value is 1, for this run only. The game's frames are matched to the viewer's by finding, for
each, the viewer frame that differs least, which also gives the offset between the game's clock
and the clip's.

## The comparison of 2026-10-19

**Recorded:** each clip with `playdemo` and `startmovie <name> 10`, which wrote 1920 by 1080 frames
with the scene drawn 4:3, 1440 by 1080, pillarboxed in the middle. The movie went on after each
demo ended, through black frames and the menu, so of the 3887, 843 and 4224 frames the game
wrote, only 125, 152 and 165 are the demo, one each tenth of a second: frames 2 to 126 of
`ls9e_stalkx`, 4 to 155 of `ls9e_cross` and 2 to 166 of `ls9e_run`, the ones that change from one
to the next. The green number in the middle is AG's speedometer, which `hud_draw 0` does not hide.
The game's `gamma`, `brightness` and `lightgamma` were all 2.0.

**Matched:** the viewer captured each clip at 640 by 480, every 0.05 s. One offset between the
game's frames and the clip's clock fits each clip, found as the one that makes the pairs differ
least: -0.40 s for `ls_9e_stalkx`, -0.20 s for `ls_9e_cross`, -0.40 s for `ls_9e_run`. The pairs then
differ by 24.5, 27.8 and 15.0 on average (mean absolute difference of grey levels out of 255 on
160 by 120 thumbnails, the speedometer left out), against 38.9, 40.2 and 22.8 for frames a second
apart.

**Confirmed** in all three clips, by eye on pairs through each:

- The camera: the recorder's view and a spectator's through a player's eyes, with the recorded
  field of view across a 4:3 picture's width.
- The map: its faces, textures, lightmaps, light styles in the dark corridors of `c3a1b`, and its
  brush entities.
- Models in their places: players, aliens and items.
- The hornets' red and orange trails along the floor in `ls_9e_run`.
- The beams in `ls_9e_cross`, among them a gauss shot's white beam and its reflection.

**Found**, differing from the game:

| What | Game | Viewer | Cause |
|---|---|---|---|
| Sky | `dusk` on `ls_9e_cross`, `morning` on `ls_9e_stalkx` | the worldspawn's sky name | the game draws the sky the server's movement variables name (`sv_skyname`), which the demos carry. Fixed: the viewer now draws that sky, the map's own when the server names none |
| Viewmodel's look | added over the scene, the weapon see-through and its black arms gone, for every weapon and player | drawn solid | not in the demos' data, where every player's render mode, amount and effects are normal. The install's AG viewmodels (`ag/models/v_*.mdl`) mark every texture additive (`STUDIO_NF_ADDITIVE`), some chrome too, and the game adds such textures over the scene, where black adds nothing. Fixed: the viewer now adds additive model textures over the scene and gives chrome ones their coordinates from the view, and the gauss and other weapons show see-through in the game's colours |
| Viewmodel's place | lower and smaller | higher, larger and further right | not known. The game may draw it at another field of view, or with the bob the viewer leaves out |
| Viewmodel through a player's eyes | the watched player's weapon, see-through | none | the client takes the watched player's weapon for the viewmodel, which the spectator's own client data does not name. Fixed: the viewer now draws the viewmodel of the watched player's weapon model, as `V_FindViewModelByWeaponModel` maps them, from its first animation when the weapon changes |
| Players' colours on `ls_9e_cross` | dark models | flat blue and green | not looked into. The game's console says it could not load the models some players name |
| A glow sprite on a building in `ls_9e_cross` | bright, a large white glow | barely there | likely its size. The game grows a glow by its distance over 200 so it keeps its size on screen, which the viewer did not. The viewer now does, not yet compared again |
| Gauss balls and glows | gone by the next moment compared | some still in the air seconds after a shot | not known, the balls' life or their fading may be too long |

After the two fixes the pairs of `ls_9e_cross` differ by 23.6 on average, against 27.8 before.
`ls_9e_stalkx` stays at 24.5, as its map already names the sky the server does.

With additive and chrome model textures and everything else since (gibs, glows hidden and grown by
distance, render modes, bodies and skins), captured again the same way on 2026-10-19, the pairs
differ by 21.4 for `ls_9e_stalkx`, 19.4 for `ls_9e_cross` and 14.4 for `ls_9e_run`, against 24.5,
23.6 and 15.0, at the same offsets. Most of it is the viewmodel, which covers much of each picture.

With the game's gamma tables, captured again on 2026-10-19 under four settings, the pairs
differ as follows, the same offsets fitting every time:

| Gamma settings | `ls_9e_stalkx` | `ls_9e_cross` | `ls_9e_run` |
|---|---|---|---|
| None, a fixed brightness of 1.6 as before | 21.4 | 19.4 | 14.4 |
| The game's defaults (2.5, 0, 2.0, 2.5) | 20.9 | 19.4 | 13.6 |
| As the game was set (2.0, 2.0, 2.0, 2.0) | 17.0 | 19.5 | 8.4 |
| As set, with `gl_overbright` | 43.3 | 48.9 | 28.0 |

The game's settings bring `ls_9e_run`, indoors under artificial light, from 14.4 to 8.4, and
`ls_9e_stalkx` from 21.4 to 17.0. `ls_9e_cross` is mostly sky, which the tables leave alone, and
does not change. Doubling after the light table is far off, so the game drew without
`gl_overbright`, which is off unless set.

## Recording the game's sound

`startmovie` saves pictures only, so the game's sound is recorded from the sound card while it
plays the clip, with a recorder that takes what the speakers play (Audacity with Windows WASAPI
loopback, or OBS), as WAV at the device's rate:

```
playdemo ls_9e_stalkx
```

started after the recorder, and the recording stopped once the demo ends. Then

```
cargo run --release -p lambda-slicer-audio --example sound_compare -- F:\SteamLibrary\steamapps\common\Half-Life F:\SteamLibrary\steamapps\common\Half-Life\ag\ls_9e_stalkx.dem RECORDING.wav 1 10
```

mixes the clip from 1 s for 10 s as the viewer does, finds where that falls in the recording by
the loudness of both in 10 ms windows, pairs the sounds that start in each within 60 ms, and
reports how far apart the paired starts are and how much louder the recording is.

Checked on the viewer's own sound tap of `stalkx_naz_248.dem` (2026-10-19): 21 s found 0.93 s
into the tap, the loudness agreeing at 0.99, all 4 starts paired 0 to 10 ms apart, and the tap at
0.55 of the mix, played at a volume of 0.5. The game's sound is not yet recorded, so the sound
has not been compared with the game's.

## Fixture matrix

The cases playback must get right however a moment is reached, each pinned to a moment of a demo in the local fixture
manifest (`moments` on a fixture, `local-fixtures.example.json`). The moments were found with
`examples/fixture_moments.rs` in `lambda-slicer-playback`, which plays a demo through in steps of a
quarter or half second and notes when each case is under way: a gauss's spin sound on a player, the
Egon's beams, a client event waiting for its delay, a reload or clip sound starting, a brush entity
moving, a sprite in a render mode other than normal, a sound on a static channel, and a death
sound starting on an entity that is not a player.

| Case | Demo | Moment | What is there |
|---|---|---|---|
| Kill, killer and victim in view | `naz_crossfire_pb_01.dem` | 14.21 s | the recorder's gauss kills Paranoid 228 units away (`analyze --kills`) |
| Gauss charging | `naz_crossfire_pb_01.dem` | 11.0 s | the recorder's spin sound, before that kill |
| Delayed event across a seek | `naz_crossfire_pb_01.dem` | 72.0 s | a client event waiting for its delay |
| Reload | `naz_crossfire_pb_01.dem` | 5.0 s | the recorder's reload, its viewmodel and sound |
| Sustained Egon | `20230909_224143_stalkyard.dem` | 106.5 s | an Egon's beams drawn |
| Ambient loop | `run_92.dem` | 4.5 s | sounds on static channels, singleplayer |
| Moving brush entity | `run_92.dem` | 7.0 s | a brush entity moving between snapshots |
| Singleplayer monster death | `run_92.dem` | 57.0 s | an alien grunt shot dead in front of the recorder |
| Translucent sprite | `ls_9e_run.dem` | 3.2 s | glows and the beam entities' lightning, recorded by the game too |

Not covered:

- An expiring HUD notice. The viewer draws the HUD now, but no moment for it has been picked.
- A recording made for this, a short match with everyone in
  view, recorded by the game for comparing. The crossfire kill stands in for it until it is
  recorded.
- The game's own sound, for comparing the mix (see "Recording the game's sound").

"Equivalence" below reaches each moment every way playback can and compares what it holds.

## Equivalence

`examples/fixture_equivalence.rs` in `lambda-slicer-audio` reaches each moment of the fixture
matrix every way playback can, with the game's files, and compares what playback holds there with
sequential playback that uses no checkpoint:

```
fixture_equivalence INSTALL local-fixtures.json [CASE...]
```

The ways:

- The three seconds before the moment played in display frames.
- A seek from checkpoints covering the whole demo, built as the worker builds them.
- A seek back from 30 s later.
- A seek from a checkpoint kept a second before the moment, and one from a checkpoint kept while a
  client event waits for its delay, when one waits in the two seconds before.
- Three drags of the playhead through the controller and the worker, from 20 s after the moment to
  20 s before and back to it, a display frame apart, so that seeks are cut short and answered late
  as in the viewer.
- A clip from 2 s before the moment to 1 s after, looped through the controller in display frames,
  at normal speed, then double, then half with a pause.

Each way is compared on playback's whole state, the frame presented, and 0.5 s of sound mixed from
there at 48 kHz, sample for sample. The state is the decoder's public state and latest snapshot, the
scene with every entity slot, gait and interpolation sample, the sound channels, the events waiting
with their seeds, the animations, the temporary entities, the decals and the beams. A drag gives
only the worker's frame, and a clip's starts give no sound. Then a slice from 8 s before the moment
to 4 s after is exported, from later in the moment's session when the export refuses that start,
and the moment in it is checked the same way against sequential playback of the slice.

A demo whose playback stops on a decode error before its end is refused, with where and why, as
every way of reaching a moment would agree on the same broken state. So is a moment past where the
demo ends, and a case no fixture has, so a misspelt case cannot pass by checking nothing. Each fails
the run.

Effects draw their random numbers from seeds each event takes from where it was recorded, and the
seeds are part of the state compared. The viewer draws no HUD, so there is no notice's expiry to
compare.

**Measured (2026-10-19)**, release build: every way agreed with sequential playback at every
moment, in the demos and in every slice exported.

| Case | Moment | Event waiting before it | Slice |
|---|---|---|---|
| Kill, killer and victim in view | crossfire 14.21 s | the gauss's stop | from 6.2 s, agrees |
| Gauss charging | crossfire 11.0 s | none | from 3.0 s, agrees |
| Delayed event across a seek | crossfire 72.0 s | the gauss's stop | from 64.0 s, agrees |
| Reload | crossfire 5.0 s | none | from 0.1 s, agrees |
| Sustained Egon | stalkyard 106.5 s | none | from 98.5 s, agrees |
| Ambient loop | run_92 4.5 s | none | from 2.5 s, agrees |
| Moving brush entity | run_92 7.0 s | none | from 3.0 s, agrees |
| Singleplayer monster death | run_92 57.0 s | none | from 53.0 s, agrees |
| Translucent sprite | ls_9e_run 3.2 s | none | from 0.1 s, agrees |

Each moment was also captured by the viewer three ways, as `--viewer-capture` seeks to its times
in the order given: alone after loading, after a time 30 s later, and after 1.5, 1 and 0.5 s
before. All 9 moments' three captures are the same pixel for pixel, at 1280x720.

Found:

- The scene, the sound state and the animations kept some tables in hash maps, whose order differs
  between two playbacks, so their states printed differently although they held the same. Ordered
  maps made seeks a quarter slower (indexed seeks in crossfire 27 ms at p50 against 35 ms), so the
  hash maps stay and only print in key order.
- In the five fixture demos every client event that waits for its delay is the gauss's stop of its
  spin, sent 10 ms after the shot (387 in stalkyard, 740 in stalkx, none in the singleplayer ones).
  The shot's sound has already replaced the spin on the weapon's channel, so the stop finds nothing
  to stop. A deliberately broken restore that dropped the waiting events still agreed every way at
  72 s, so these demos cannot show that a waiting event survives a restore. The synthetic seek
  tests now carry a gauss shot that waits half a second across checkpoints, and the same broken
  restore fails three of them (see the [viewer profile](viewer-profile.md), "Checkpoints").
- The export refused slices round run_92's first moments for these two cases, the ambient loop at
  2.5 s, as the session begins at 1.95 s and has no full update yet, and the monster death at
  19.5 s, as a temporary entity is still moving at every start tried. Both moments agreed every way
  in the demo itself. The cases moved to the moments above, whose slices export. The monster
  death moved once more, from 33 s, where the death sound starts with the monster out of view, to
  57 s, where an alien grunt dies in front of the recorder. That moment too agrees every way and
  is captured the same three ways.

## Reference captures

A clip of each moment of the fixture matrix, cut with `lambda-slicer slice` into the game's `ag`
folder, so the game and the viewer play the very same files, as in the comparison of 2026-10-19.
Their names keep the label `12d` of the step they were made in. Each runs 6 s, the moment
2 or 3 s in. The translucent sprite's moment is in `ls_9e_run.dem`, which the game has recorded
already.

| Clip | Source | Cut | The moment in the clip |
|---|---|---|---|
| `ls_12d_kill.dem` | `naz_crossfire_pb_01.dem` | 11.21 to 17.21 s | 3.0 s, the gauss kill, with the shot, the death and after |
| `ls_12d_gauss.dem` | `naz_crossfire_pb_01.dem` | 8 to 14 s | 3.0 s, the gauss charging |
| `ls_12d_delayed.dem` | `naz_crossfire_pb_01.dem` | 69 to 75 s | 3.0 s, the gauss's stop waiting for its delay |
| `ls_12d_reload.dem` | `naz_crossfire_pb_01.dem` | 2 to 8 s | 3.0 s, the reload |
| `ls_12d_egon.dem` | `20230909_224143_stalkyard.dem` | 103.5 to 109.5 s | 3.0 s, an Egon's beams in a 32-player fight |
| `ls_12d_ambient.dem` | `run_92.dem` | 2.5 to 8.5 s | 2.0 s, ambient loops |
| `ls_12d_brush.dem` | `run_92.dem` | 4 to 10 s | 3.0 s, a brush entity moving |
| `ls_12d_monster.dem` | `run_92.dem` | 54 to 60 s | 3.0 s, an alien grunt shot dead in view |

**Recording the game's frames**, as in "Recording the game's frames", one clip at a time, windowed
at 1280 by 720 or any size, with the game's gamma settings noted:

```
hud_draw 0; crosshair 0; net_graph 0; cl_showfps 0; r_drawviewmodel 1
playdemo ls_12d_kill; startmovie ls12d_kill 10
```

and `endmovie` once the demo has ended, the movie named as the clip without its first underscore.
Then the same for the other seven. **Recording the game's sound**, as in "Recording the game's
sound", in a second pass without `startmovie`, which slows the game: the loopback recorder started,
`playdemo ls_12d_kill`, the recording saved as a WAV file, for

```
cargo run --release -p lambda-slicer-audio --example sound_compare -- F:\SteamLibrary\steamapps\common\Half-Life F:\SteamLibrary\steamapps\common\Half-Life\ag\ls_12d_kill.dem ls12d_kill.wav 0.5 5
```

**Comparing.** `scripts/viewer-references.ps1 -Game DIR -Out FOLDER` captures the viewer's frames
of every clip, at 640 by 480 every 0.05 s through the clip's length and a second more (`-Length`,
6 s unless given), with the game's gamma settings (`-Gamma`, 2,2,2,2 unless given). For each clip
whose game frames are in `ag`, it then runs `examples/frame_compare.rs` in `lambda-slicer-gui` and
writes a contact sheet of every tenth pair to `FOLDER\<clip>.png`, the game's frame on the left.

`frame_compare` aligns the game's frames with the clip's clock first, counting from the first
frame that differs from the one before, the demo's first after the loading screen, and then takes
those in the clip's first seconds, its end left out (`--seconds`, which the script gives as the
clip's length). Every frame those seconds should have counts, so a recording that ends early or
lacks a frame shows as frames without a pair, and the run fails. Given frame numbers instead
(`--frames FIRST-LAST`) are all compared, and refused when the recording ends before the last.
Without either it guesses the demo's frames as the longest run of changing frames, still for 3 s
at most, which a longer pause in the demo would cut short. It cuts both sides to the 4:3 scene and
makes grey 160 by 120 thumbnails, the speedometer left out, finds the offset between the game's
clock and the clip's within 2 s, and says how many frames found a pair, how much the pairs differ
against frames a second apart, and which differ most.

**Checked (2026-10-20)** on `ls_9e_run.dem` and the game's frames of it from 2026-10-19
(`-Clips ls_9e_run -Length 16`): an offset of -0.40 s, as found by hand then, so the clip's 16 s are the game's
frames 6 to 165, all 160 paired, differing by 9.3 against 23.4 for frames a second apart (8.4 on
2026-10-19, before the beam entities and steady glows changed this clip). The frame at the clip's end,
16.0 s, is left out, as a clip's end is not in it, and the game shows its menu there. Given frames
2 to 166, the frames taken on 2026-10-19, and with the frames guessed from the movement, it finds the same
offset and pairs differing by 9.4. With the recording cut after its 120th frame it counts the 45
frames missing and fails. The pairs differing most are at 4.4 to 4.9 s, where a crate breaks. The
viewer's wooden gibs are larger and nearer the camera than the game's, not yet looked into.

**Status:** the clips and the viewer's frames of them are ready. The game's frames and sound of
the eight new clips are not recorded yet, so nothing of them is compared.

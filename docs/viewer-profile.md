# Viewer profile

What the integrated viewer (implementation plan §13) reads, decodes and presents, and how far each
part is proven. This file grows with M8 to M12. Statements marked **Observed** were checked
against the installed game or real recordings, and **Implemented** marks a rule the code follows
that the game has not yet confirmed.

## Assets

The viewer loads the user's installed game content and bundles none (`lambda-slicer-assets`).

### Installation surveyed (2026-10-17)

`F:\SteamLibrary\steamapps\common\Half-Life`, the Steam release with AG installed:

| Folder | Holds |
|---|---|
| `ag` | AG's own content as loose files, 194 entries in `models` and many custom map WADs at the top level |
| `ag_downloads` | content downloaded from servers, mostly map WADs |
| `valve` | Half-Life's content as loose `maps`, `models`, `sound` and `sprites` folders and WADs (`halflife.wad`, `decals.wad`, `gfx.wad` and others), no PAK archive |
| `valve_addon`, `valve_hd`, `valve_downloads`, `valve_spanish` | the Steam client's addon, HD model, download and language folders |

AG's `liblist.gam` names no `fallback_dir`, so `valve` is its only fallback.

### Search order

**Implemented**, from the Steam client's folder conventions, not yet compared with the game.
For the mod, then its `fallback_dir` if any, then `valve`, each present folder in this order:

1. `<game>_addon`, when addon content is enabled
2. `<game>_<language>`, when a language other than English is chosen
3. `<game>_hd`, when HD models are enabled
4. `<game>`
5. `<game>_downloads`

Within a folder its PAK archives come before its loose files, the highest-numbered `pakN.pak`
first, as in Quake, whose file system GoldSrc's descends from. This install has no PAK archive,
so the order inside a folder cannot be checked here.

### Names

Asset names in demos use either slash and the game's own capitalisation. A name is split into
components, and a name that is absolute, carries a drive, or has an empty, `.` or `..` component
is refused, so no lookup leaves the game folders. Each component is matched exactly first, then
ignoring ASCII case, the way Windows matches, so the same demo resolves on Linux. Two entries that
differ only in case are reported as ambiguous instead of picking one.

## Resources

Each session lists its resources in `svc_resourcelist`. `lambda-slicer-assets` turns every entry
into a lookup (`resource::target`):

| Entry | Looked up as |
|---|---|
| sound `weapons/gauss2.wav` | the file `sound/weapons/gauss2.wav` |
| sound `!HG_ALERT0` | a sentence from `sound/sentences.txt`, resolved with the sound system (M10) |
| model `*12` | brush model 12 inside the map's own BSP |
| model, sprite, world or generic file | that file |
| decal `{blood1` | an entry of the first `decals.wad` in the search |
| decal named in the consistency list | not content |

**Observed:** servers that check clients' files list each one as a decal and name it in the
list's consistency list, as `../opengl32.dll` or `../bin/TrackerUI.dll`. Real decals are never in
that list (`stalkx_5o5_asasasdgwgw.dem`: 198 such checks, every one in the consistency list, and
none of its 222 decals). `protocol::messages::consistency_positions` turns the consistency list
into resource positions, and those decals are reported as checks, not missing images.

**Measured (2026-10-17)** with `examples/resource_probe.rs` against the surveyed install, AG with
default options (no addon, language or HD folders):

| Demo | Sessions | Resources per session | Missing |
|---|---|---|---|
| `naz_crossfire_pb_01.dem` | 1 | 594 | none |
| `stalkx_5o5_asasasdgwgw.dem` | 1 | 806, of them 198 file checks | none |
| `run_92.dem`, singleplayer | 5 | 621 to 744 | none |
| `20230909_224143_stalkyard.dem` | 2 | 548 | none |
| `ray_vs_alex_cross.dem`, spectator | 1 | 622 | none |
| `20210418_004941_vengeance.dem` | 2 | 719 and 583 | the decals `{sitea`, `{siteb` and `{hrpoint` |

Most content comes from `valve` (197 to 303 files a session) and `ag` (78 to 119), with a few map
WADs and models from `ag_downloads`. The vengeance server listed three decals that the installed
`decals.wad` does not have, so it used a `decals.wad` of its own. The viewer reports such decals
as missing instead of drawing something else.

## Playback

`lambda-slicer-playback` plays a demo through the core decoder, one record group at a time
(implementation plan §13.2 and §13.4).

### Groups and clocks

A group is the run of records sharing an entry and a frame number, as restarts count them. Every
entry is read in directory order, the LOADING entry first. Groups are numbered from 0 across the
whole file, so a position (group, session, source time) stays unique when server time starts again
after a map change.

| Clock | What it is |
|---|---|
| Source time | a record's time on the playback clock. LOADING records count as 0, the state playback begins with |
| Server time | the last `svc_time`, which starts again every session |
| Presentation time | the source time a frame is shown for, which can fall between groups |
| Wall time | the window's monotonic clock. Only the controller's clock turns it into source time |

Reaching a time applies every group at or before it, so an event recorded at exactly the chosen
time has happened. Stepping moves by whole groups, not display frames.

### Controller

Every control that moves the playhead becomes a command for one controller: load, play, pause, seek,
step, rate (0.25× to 4×), loop and camera. A load starts a new generation and every seek, step or loop
restart a new revision. The worker answers the newest request and skips any a newer one replaced,
and the controller shows a reply only when it answers the current generation and a revision no older
than the current one. Tested on a synthetic demo with two seeks in a row and a load over a pending
seek.

A seek, step or load sent while the worker is still answering another stops that one between
groups, unanswered, and dropping the worker stops it the same way, so changing demos never waits
for a seek no one wants (one cold seek took 7 s before checkpoints). The requests playing sends
each frame share a stamp and do not stop each other, so playback slower than the clock still shows
frames. A seek stopped part way leaves playback at a group it reached, which the next request goes
on from, and is tested to end where sequential playback does.

### Scene

Entities come from the decoder's newest stored snapshot, so a packet the client rejected never
reaches the scene. An entity is its session, number and life. A life ends when the entity leaves the
packet or its model changes, so a reused slot is never taken for the entity before it. A new session
starts an empty scene. The recorder's camera is the `ref_params_t` view of each network record, with
the field of view of the last client data record. Interpolation, attachments, effects and the HUD
come with M9 and M10.

### Sound state

**Implemented**, from the engine's rules as GoldSrc keeps them from Quake's sound code, not yet
compared with the game:

- A sound on an entity's channel replaces what that channel played. The automatic channel (0) never
  replaces, and the static channel (6) and `svc_spawnstaticsound` keep ambient sounds apart from
  the dynamic ones
- A stop, volume change or pitch change applies to the sound on that entity and channel with the
  same sample (any sentence matches any sentence). A change that finds nothing starts the sound
- `svc_stopsound` stops the dynamic channel whatever it plays
- A sound record carries no entity, so it plays on the entity the recorder views from
- A sound's position in its sample is integrated over its pitch history
- A new session silences everything, and its sounds' names are read again from its own resource
  list

- A sound that is not static takes one of the engine's eight dynamic channels. When all are busy
  it takes the one with the least time left, except that another entity's sound never takes one of
  the listener's own, and when nothing can be taken it is not heard (`SND_PickChannel`)

How long a sound plays comes from its WAV file (10a). Without the files every sound counts as
finished after ten seconds. Sentences are not resolved to their words yet, so they have no samples.

### Measured (2026-10-18)

`examples/playback_probe.rs`, release build, each demo played through, then three moments reached
directly and back from the end, comparing position, entities, sounds, camera and viewmodel:

| Demo | Sessions | Playback | Played through in | Moments agreeing |
|---|---|---|---|---|
| `stalkx_naz_248.dem` | 1 | 1,215 s | 3.8 s | 3 of 3 |
| `20210418_004941_vengeance.dem` | 2 | 2,506 s | 9.9 s | 3 of 3 |
| `naz_crossfire_pb_01.dem` | 1 | 1,206 s | 3.7 s | 3 of 3 |
| `dblcross_salchi_naz.dem` | 2 | 1,314 s | 3.8 s | 3 of 3 |
| `run_92.dem`, singleplayer | 5 | 104 s | 0.37 s | 3 of 3 |
| `ray_vs_alex_cross.dem`, spectator | 1 | 103 s | 0.10 s | 3 of 3 |

None stopped on a decode error. The decoder alone takes 1.1 s of stalkx's 3.8 s.

### Checkpoints

A checkpoint is everything sequential playback holds after one group: the decoder with its
schemas, baselines and delta history, the scene, the sound state and where the source goes on. A
seek restores the latest checkpoint that can reach the destination and plays on from there, or
plays on from where it is when that is nearer. Source times can step back between groups, so each
checkpoint keeps the latest time of any group up to it, and only one whose latest time is not after
the destination is used. Tables a session sets up (resources, baselines, user messages) are shared
between checkpoints.

Playback takes a checkpoint when a session begins and then every 2 s of source time. While no
request waits, the worker plays a second copy of the demo ahead in slices of 1,000 groups (a few
tens of milliseconds) to fill the same store. A checkpoint keeps the decoder's last 64 snapshots,
any of which a later packet may take as its delta base, so in a busy game it weighs 1.5 to 2 MiB.
When a demo's checkpoints pass 256 MiB by the decoder's estimate, every other one is dropped and the
spacing doubles. Checkpoints live in memory for the selected demo only. Keeping them on disk
between runs is left for later.

**Tested** on a synthetic demo with a second session and a group whose time steps back, against
sequential playback that never uses a checkpoint: forward and backward seeks to 61 moments and on
past each, 300 random seeks while checkpoints are still being added, group steps both ways, a clip
looped three times in display-sized steps, and the worker's answers once its index is complete.
The comparison covers the decoder's counters and latest snapshot, every entity, the camera, the
viewmodel, light styles, sounds and the scene's and sound state's change counts. Two deliberately
broken restores (a decoder counter changed, the source's read-ahead record lost) both fail it.

**Measured (2026-10-18)** with `examples/seek_probe.rs`, release build: every checkpoint built as
the worker builds them, then 60 seeks to random moments in random order, each compared with the
state one sequential pass has there. Memory is the growth in private bytes while building, which
keeps what thinning freed, next to the store's own estimate.

| Demo | Playback | Checkpoints | Built in | Memory, measured and estimated | Seek with none, to 3/4 | Indexed seeks p50, p95, max | Differing |
|---|---|---|---|---|---|---|---|
| `stalkx_naz_248.dem` | 1,216 s | 76, 16 s apart | 4.1 s | 247 and 163 MiB | 2.5 s | 23, 45, 49 ms | 0 |
| `20210418_004941_vengeance.dem` | 2,506 s | 79, 32 s apart | 9.9 s | 301 and 248 MiB | 7.0 s | 51, 118, 138 ms | 0 |
| `naz_crossfire_pb_01.dem` | 1,206 s | 76, 16 s apart | 4.3 s | 233 and 141 MiB | 3.0 s | 23, 48, 50 ms | 0 |
| `dblcross_salchi_naz.dem` | 1,314 s | 83, 16 s apart | 4.1 s | 277 and 231 MiB | 3.0 s | 18, 40, 49 ms | 0 |
| `run_92.dem`, singleplayer | 104 s | 54, 2 s apart | 0.4 s | 123 and 112 MiB | 0.37 s | 4, 10, 20 ms | 0 |
| `ray_vs_alex_cross.dem`, spectator | 103 s | 51, 2 s apart | 0.1 s | 51 and 122 MiB | 0.13 s | 0.3, 3.6, 4.7 ms | 0 |

Without the budget, stalkx keeps 608 checkpoints 2 s apart in 928 MiB and seeks in 2.9 ms at p50
and 5.1 ms at p95. Every indexed p95 is within the plan's target of 200 ms. The longest demo is
closest to it, as its spacing doubled twice, and a longer one would double again. A first seek
before the index reaches the destination replays from the start, at about 3 ms per second of a busy
demo.

## Viewer

The demo view plays the demo when a game folder is given with `--game DIR` (implementation plan
§13.8). A map that cannot be read is named in place of the view with the reason, nothing of the map
before it drawn, and is not asked for again. Playback that stops before the demo's end, because a
part of it could not be decoded, says why under the view. `--viewer-capture` saves its frames at chosen times for comparing with the game
([viewer validation](viewer-validation.md)). It keeps a playback controller and worker per demo, and a thread that reads the session's
map and the studio models its entities use. Play, pause and a time bar drive the controller, and
`--viewer-at SECONDS` seeks there once the demo is loaded. The asset folder setting and the rest
of the viewer's controls come with M11.

### What is drawn (9a)

| Part | How |
|---|---|
| Camera | the recorder's view of each network record, with the recorded field of view across the screen's width, as the game's is. 90 degrees when the recording names none |
| World | the session's map, read again when a later session changes it |
| Entities | each with a studio model, posed from its recorded sequence, frame, frame rate, animation time, controllers and blends, the frame estimated as the renderer's `StudioEstimateFrame` does. Client time is the last `svc_time` plus the source time since that packet |
| Players | the model their userinfo names (`models/player/<name>/<name>.mdl`), the stock player model when it is not installed, drawn upright |
| Left out | the entity the recorder views from, the player a spectator sees through the eyes of (observer mode 4 and its target, from the recorder's client data as the game's client takes them), and entities with `EF_NODRAW` |

**Observed:** AG ships some item models as sprites under a `.mdl` name (`ag/models/w_357.mdl`
starts with `IDSP`). The game tells a model's kind by its header, and so does the viewer, which
leaves those items for the sprites (9d).

### World (9b)

**Implemented**, from the game's renderer as GLQuake's source shows it, not yet compared with the
game (9e):

| Part | How |
|---|---|
| Brush entities | every BSP model an entity names as `*N`, turned and moved as the renderer turns brush models (yaw about z, pitch about y the opposite way, roll about x) |
| Render modes | normal and alpha-tested drawn with the world. Texture mode lit and blended by the render amount, colour mode a flat blended colour, additive and glow modes added unlit. Translucent entities are drawn last, the furthest first, and one at amount 0 is not drawn |
| Light styles | up to four lightmaps a face, each weighed by its style's brightness at the client's time. A pattern steps ten times a second, `a` to `z` meaning 0 to 550 where 256 is the lightmap as stored. A style no message set is 256 |
| Unlit faces | full brightness, as the game draws a face without light data |
| Animated textures | `+0` to `+9` chains at ten frames a second, and the `+a` to `+j` chain while a brush entity's frame is not 0 |
| Water | `!` and turbulent faces unlit, their texture coordinates warped by eight texels in a sine of the other coordinate and the time, at 64 texels a repeat |
| Sky | the sky the server's movement variables name (`sv_skyname`), and until it names one the `worldspawn` `skyname`, `desert` by default, from `gfx/env/<name>{rt,lf,bk,ft,up,dn}.tga` in the sky box's layout, drawn wherever nothing else was. Confirmed against the game, where the two differ, in `ls_9e_cross` ([viewer validation](viewer-validation.md)) |

Not drawn yet: visibility and frustum culling (the whole world is drawn every frame), random tiling
textures (`-0` names) and scrolling conveyors.

### Models (9c)

**Implemented**, from the game's client as the Half-Life SDK's `StudioModelRenderer` shows it, not yet
compared with the game (9e):

| Part | How |
|---|---|
| Interpolation | entities drawn 0.1 s behind the client's clock (`ex_interp`), each moved along its own samples from the last 32 snapshots, angles the short way round. Not through a new life, a move over 128 units between snapshots, or past the latest snapshot |
| Client clock | the latest snapshot's `svc_time` plus the source time since it arrived |
| Legs | `StudioEstimateGait` and `StudioProcessGait`, stepped once a record group, each group being a frame the recording client drew: the legs face the way the player moved, turn towards the body while standing, turn round with the movement counted back when the body faces more than 120 degrees away, and the torso twists the rest of the way over the four spine controllers |
| Gait frame | the distance moved through the gait sequence's linear movement, or its frame rate over the time passed when it does not move |
| Aim | a player with a gait sequence aims through its first blend from its pitch (`StudioPlayerBlend`), the rest of the pitch turning the model. One without keeps its controllers centred |
| Held weapons | the player's `weaponmodel`, its bones named as the player's taking the player's transforms (`StudioMergeBones`), the rest posed with the player's sequence |
| Lighting | the colour under the model from a line 2048 units down through the BSP tree to the first lit face, its lightmaps weighed by their styles (`R_LightPoint`). Its brightest channel is the ambient level up to 128 and the light from above up to 192, softened by a lambert of 1.5. Where nothing is under a model it is lit at 128 and 64 |
| Body and skin | each entity's body and skin choose the submodel of each body part (`(body / base) % models`) and the skin family, as `StudioSetupModel` does. A body past the model's own is the first's, and each combination is read once, after the model's first body is in. The viewmodel takes the body of its last animation |
| Render modes | normal models solid. Colour, texture and alpha modes blended by the render amount, glow and additive modes added by it, each lit as a solid model and without the render colour, which the game's studio drawing takes only for its glow shell. Translucent models are drawn after the sky and before translucent brushes, the furthest first, and one at amount 0 is not drawn. A player's weapon is drawn as the player is |
| Team colours | a player model's `DM_Base` textures with palette entries 160 to 191 turned to the top colour's hue and 80 to 111 to the bottom colour's, each keeping its brightness and saturation, as the game's `PaletteHueReplace` does. The surveyed AG install's own player models have no such textures |

The leg state lives in playback's scene, so checkpoints keep it and a seek reaches the same legs as
playing does. The gait frame takes the whole life's distance through the current gait sequence, so
when the gait sequence changes the legs keep their place in the cycle by distance, where the game
carries on from the frame it had. That shows as a jump in the legs' phase at a change from walking
to running, for example.

**Observed:** AG's `w_weaponbox.mdl`, the box a player drops on death, has a black and white
texture, so it looks like the checker drawn for a missing texture.

Not drawn yet: `remap` textures, sequence transitions, mouth movement, chrome and additive model
textures, render effects such as the glow shell, and the sky's light on models.

### Viewmodel and sprites (9d)

**Implemented**, from the game's client and GLQuake's sprite drawing, not yet compared with the game
(9e):

| Part | How |
|---|---|
| Viewmodel | the model the recorder's client data names (`viewmodel`), or through a player's eyes the viewmodel of that player's weapon model (`V_FindViewModelByWeaponModel`), starting at its first animation when the weapon changes, playing the last weapon animation, from a record or a message, from when it started. At the view's origin and angles, the pitch turned the other way as the game gives it to a viewmodel, and lit by the light under the eye. Drawn last against a cleared depth, so it never sinks into a wall |
| Viewmodel hidden | while the recorder's health is 0 or less, at intermission, and while spectating other than through a player's eyes |
| Sprites | `IDSP` version 2, single frames and frame groups, in normal, additive, index-alpha and alpha-tested formats. Every sprite in the surveyed install reads (268, AG's sprite items included) |
| Sprite entities | an entity whose model is a `.spr`, or one of AG's sprites under a `.mdl` name, at its frame and scale (0 meaning 1) |
| Orientation | facing the view, facing the view upright, facing the camera's position upright, turned by the entity's angles, or facing the view rolled by the entity, as the five `SPR_` types say |
| Render modes | normal and alpha modes cut out at half alpha, texture and colour modes blended by the render amount, additive mode added, and glow added without depth, fading as 19000 over the squared distance between 0.05 and 1, and grown by its distance over 200 to keep its size on screen unless it is steady. A glow the world hides from the eye, the line to it stopping more than 8 units short, is not drawn, brush entities and models hiding none (`R_SpriteGlowBlend`). Every mode takes the render colour, white when it is black, as Xash3D says the engine's sprite drawing does (until 10d only additive and glowing sprites did). Drawn after translucent brushes, the furthest first |

Not drawn yet: the bob and lag the game adds to the viewmodel.

**Measured (2026-10-18)** with `--viewer-bench 10`, playing from 20 s in a 1920 by 1080 window with
the view 1098 by 950 pixels, while the library scan ran:

| Demo | Models in view | 9a mean, p95 | With the world of 9b, mean, p95 | With the models of 9c, mean, p95 |
|---|---|---|---|---|
| `stalkx_naz_248.dem` | 4 | 3.7, 4.8 ms | 3.4, 4.3 ms | 3.2, 4.8 ms |
| `ray_vs_alex_cross.dem`, spectator | 8 to 9 | 6.7, 19.6 ms | 3.9, 5.1 ms | 4.6, 6.3 ms |
| `run_92.dem`, singleplayer | 10 | | 5.2, 6.3 ms | 2.9, 4.0 ms |

The library scan's load changes between runs, so the columns are not a measure of each step.

## Sound (10a)

`lambda-slicer-audio` reads the game's sounds and mixes playback's channels into stereo samples.

**Observed:** the game's sounds are PCM WAV files of 8 or 16 bits, mono except for a few, at rates
from 8,000 to 192,000 Hz. All 3,094 in the surveyed install's `valve` and `ag` sound folders read,
and 338 of them loop.

**Implemented**, from Quake's sound code as the game keeps it, not yet compared with the game:

| Part | How |
|---|---|
| Loops | a `cue ` chunk makes a sound loop from the sample offset 32 bytes past the chunk's header, and a following `LIST` chunk labelled `mark` gives the loop's length (`GetWavinfo`) |
| Names | `sound/<name>`, a leading `*` dropped |
| Position | where a channel is in its sample is playback's cursor, its pitch integrated over time, read between stored samples linearly at any rate |
| Placement | a sound from the entity the listener views from plays at full volume on both sides. Any other fades with distance times its attenuation over 1000 units, and moves between the ears by how far it lies to the listener's right, twice as loud on one side at most (`S_Spatialize`) |
| Mix | every channel added and scaled by a master volume, then clipped to full scale |

**Measured (2026-10-18)** with `examples/mix_probe.rs`, at 48 kHz and a master volume of 0.7, from the
recorder's view:

| Demo | Stretch | Mixed in | Sounds heard | Clipped samples |
|---|---|---|---|---|
| `stalkx_naz_248.dem` | 20 to 32 s | 0.19 s | 13 | 1 |
| `ray_vs_alex_cross.dem`, spectator | 15 to 30 s | 0.23 s | 22 | 1,437 of 1.4 million |
| `run_92.dem`, singleplayer | 6 to 22 s | 0.40 s | 90 | 12,581 of 1.5 million |

Not yet: sentences and room effects.

## Sound output (10b)

The demo view plays sound on the system's default output device while playing forward, with a
volume slider beside the controls. `lambda-slicer-audio`'s `device` feature holds the output, through
`cpal` (WASAPI on Windows, ALSA on Linux).

| Part | How |
|---|---|
| Timing | a mixing thread keeps a playback of its own of the demo the worker has open, sharing the worker's checkpoints, and moves it to the end of each 10 ms block before mixing the channels it then has. Each block hears the sounds playing during it, whenever the window's frames come |
| Clock | each block is placed where the playback clock will be once the blocks queued before it are heard, four of them. The mixer goes on from its own place unless a load or seek comes or it falls more than 40 ms from the clock |
| Silence | while paused, seeking, stepping or before the demo is open, and at the start of each new load |
| Device | the callback copies finished blocks out and returns them to the mixer to be filled again, with no file read, no lock held for long and nothing freed |
| Listener | the recorder's view at the end of the block, the entity it views from heard as its own |
| Old blocks | each block is marked with what was playing when it was mixed. A seek, load, pause or resume changes the mark at once and the device drops every block queued under the old one, so nothing from before plays after it. A seek the mixer is making stops when the mark changes |
| Closing | the stream is released before the mixer is waited for, and the mixer gives up a block the device does not take once the device closes, so a lost device never holds up closing or changing demos |
| Moving sounds | a sound from an entity plays from where the entity is at the end of each block, so it moves with it. The world's sounds, ambient sounds and a brush entity's stay where they started |

`LAMBDA_SLICER_SOUND_TAP=FILE` with `--viewer-play SECONDS` writes every block from the first heard
on to a WAV file, the device playing silence at the same pace, for comparing with
`examples/mix_probe.rs`.

**Measured (2026-10-18)** on Windows, `stalkx_naz_248.dem` played 8 s from 20 s at 44.1 kHz, the tap
compared with `mix_probe` at the same rate and a master volume of 0.7 against the device's 0.5:

| Check | Result |
|---|---|
| Alignment | one offset for the whole tap, every half second at the same sample |
| Level in 50 ms windows | 110 windows with sound, ratio 0.702 to 0.719 for the expected 0.714 |
| Difference left | 0.002 RMS against a signal of 0.057, from block edges falling at other times |

Before the mixer had a playback of its own it mixed the channels of the frame last shown, and
sounds shorter than a window frame came late or not at all (windows down to 0.17 of the expected
level).

## Client events (10c)

The game's client turns weapon events into sounds, weapon animations and effects in its event
handlers, and acts on the client events in the animations of the models it draws. Playback does
both as each group moves time on, so they come out the same however often the window draws and
whatever a seek passed through.

**Observed:**

- A demo's sound records hold only the sounds the recording client's movement makes. In
  `stalkx_naz_248.dem` they are 2,736 footsteps and 7 falls, and no weapon sound. Weapon sounds
  come only from the event handlers.
- Event records come with the recorder's own predicted shots, and network events with other
  players' shots and with the singleplayer player's.
- AG records a weapon animation record for the animations its predicted weapons start, but not for
  the one an event's handler starts. `stalkx_naz_248.dem` has the gauss shot at 5.92 s as events
  without a weapon animation record. `run_92.dem`, singleplayer, has neither records nor
  `svc_weaponanim` around its MP5 shots, so the handlers animate the viewmodel there.
- AG's own viewmodels (`ag/models/v_*.mdl`) carry reload sounds (event 5004) and no muzzle flash,
  where Half-Life's carry a muzzle flash on the first frame of their shoot sequences. The player
  models carry one on the first frame of each shoot sequence, on the hand's attachment.
- A shot fired mid-jump keeps the jump sequence and only sets `EF_MUZZLEFLASH`, which the engine
  turns into a light and no sprite. Of the 17 glock shots the opponent fired in the first 5
  minutes of `naz_crossfire_pb_01.dem`, 6 came that way.

**Implemented**, from the SDK's `ev_hldm.cpp` and `hl_events.cpp` and the engine's behaviour as
Xash3D restates it, not yet compared with the game:

| Part | How |
|---|---|
| Events | event records, `svc_event` and `svc_event_reliable`, each run through its handler when it arrives, or queued until its delay has passed. The gauss's stop takes back the spin events queued for its player (`EV_KillEvents`), and a new session drops them all (`CL_ResetEvents`) |
| Sounds | the glock, shotgun, MP5 and its grenade, python, crowbar, crossbow, RPG, hornet gun, gauss shot and charge, Egon start, run and stop, and the train, with the handlers' samples, channels, volumes and pitches. The gauss's charge changes its pitch as each spin event comes, and its stop silences it |
| Random choices | each event draws its own from a seed made of where it was recorded, the record, the message and the event's index, as the client's generator is not recorded. A seek makes the same choices |
| Place | the event's origin and angles. An event that gives none, as the server's events from an entity in the same packet do, takes its entity's origin and angles, a player's pitch turned back from the model's third of it to the view's. The recorder's own take its view, as a singleplayer recording can leave the player's entity where it spawned (`run_92.dem` holds it 2788 units away) |
| Viewmodel | for the local player's events, the recorder or the player watched through the eyes of (`EV_IsLocal`), the handler's weapon animation, the crowbar's swings counted through its three misses |
| Animation events | a model's client events (5000 on) fire when its frame passes them between one group and the next, from before the first frame when an animation that does not loop has begun. An entity's begins when its sequence changes or its frame goes back with a new animation time, the viewmodel's with each weapon animation. Each frame fires once in one animation. Models the game does not draw fire none |
| Muzzle flashes | events 5001, 5011, 5021 and 5031 at attachments 0 to 3. The option's last digit picks `sprites/muzzleflash1.spr` to `muzzleflash3.spr`, its tens the scale in tenths (0.5 for none). Added to the view, at a random frame, rolled 0 to 20 degrees for the first sprite and anywhere for the others, for 0.01 s and drawn at least once (`R_MuzzleFlash`) |
| Model sounds | event 5004's sample, heard as the listener's own at full volume (`pfnPlaySoundByNameAtLocation`) |
| Shells | the glock and MP5 throw out `models/shell.mdl`, the shotgun one `models/shotgunshell.mdl` a barrel, from 28 units over the event's origin (12 for a ducking player), forward, down and right by each weapon's amounts, flying 50 to 70 units a second right, 100 to 150 up and 25 forward on top of the event's velocity (`EV_GetDefaultShellInfo`, `EV_EjectBrass`). Turned to the shooter's yaw, spinning up to 255 degrees a second about each axis, for 2.5 s (`R_TempModel`) |
| Shell movement | each group, on along the velocity and turned by the spin, then traced from where it was through the world's point hull. Where the line meets something the shell stops there, its velocity reflected about the plane and halved, its yaw turned the other way and its angles cut by a tenth. Landing on a floor (normal over 0.9) falling slower than three groups of gravity stops it flat for good. Then the server's gravity pulls on it (`HUD_TempEntUpdate`) |
| World trace | Quake's `SV_RecursiveHullCheck` through the BSP's node tree from the world's first head node, a thirty-second of a unit short of the plane met, brush entities left out as the client's world-only trace leaves them |
| Bounce sounds | `player/pl_shell1.wav` to `3` or `weapons/sshell1.wav` to `3`, always for a bounce falling at 200 units a second or more, one in four otherwise, as loud as the fall over 350 and none for the landing that stops it (`CL_TempEntPlaySound`) |

`examples/sound_starts.rs` counts the sounds playback starts by where they come from, and
`examples/studio_events.rs` lists the client events in a folder's models. `mix_probe` also counts
muzzle flashes.

**Observed:** a shell landing on a slope steeper than about 25 degrees, such as the ramps of
`crossfire` (normal 0.894 up), never meets the floor test and hops on quietly until its life is
over, as the client's own code would have it. AG's `shell.mdl` is red.

**Measured (2026-10-18):**

| Demo | Stretch | Events | Result |
|---|---|---|---|
| `stalkx_naz_248.dem` | 0 to 60 s | 28 gauss shots, 29 stops, 27 spin starts, 1 crossbow shot | 28 `gauss2.wav` and 27 `pulsemachine.wav` starts, the crossbow's two sounds |
| `naz_crossfire_pb_01.dem` | 0 to 300 s | 11 own glock shots as event records, 17 of the opponent's as network events | 11 muzzle flashes on the opponent, AG's reload sounds from the viewmodel |
| `run_92.dem`, singleplayer | 0 to 100 s | 27 MP5, 11 hornet gun, 9 gauss, 3 shotgun and 2 crowbar events | 34 muzzle flashes, from soldiers' animations, none from AG's MP5 |

Played on the device from 18 s of `stalkx_naz_248.dem` through the gauss's charge, the mix still
matches `mix_probe`: 150 windows of 50 ms with sound, ratio 0.714 at the median, 2 windows at about
0.82 where a sound ends or is replaced within a block and the two mixes' blocks fall at other times.

With shells, the first 300 s of `naz_crossfire_pb_01.dem` throw out 28, one for each glock shot,
the recorder's and the opponent's. None falls more than 200 units, and the 300 s still mix in 3.2 s.

Not yet: the tripmine's and snark's handlers, which trace first, the view's punch, sparks (event 5002), and
the lights of muzzle flashes and `EF_MUZZLEFLASH`.

## Temporary entities (10d)

The server asks the client for effects with `svc_temp_entity`, and the client makes them as
temporary entities alongside the shells of 10c (`lambda-slicer-playback`'s `tempents`).

**Observed:** five demos (`stalkx_naz_248.dem`, `naz_crossfire_pb_01.dem`, `run_92.dem`,
`ray_vs_alex_cross.dem` and `20230909_224143_stalkyard.dem`, `examples/effects_survey.rs`) hold 21
kinds. The most common are world decals (16,635 messages), sparks (10,734), blood sprites (5,809),
explosions (1,160), smoke (661), gunshot decals (472), tracers (439) and beam trails (273).

**Implemented**, the builders from the engine as Xash3D restates them and the movement from the
SDK's `HUD_TempEntUpdate`, not yet compared with the game:

| Kind | How |
|---|---|
| Movement | each group, on along the velocity, the sprite's frame on by its frame rate, turned by its spin, traced through the world when it collides, then pulled down by gravity or half of it. A sprite that plays once is gone after its last frame. One that fades out loses its brightness at its fade speed once its time is up. At most 500 at once |
| Explosion | the sprite at its frame rate (10 for none) and scale, added at amount 180 or drawn solid, 10 units up and rising 8 a second, with one of `weapons/explode3.wav` to `5` unless the message says no sound |
| Smoke | the sprite alpha-tested in a dark grey of 20 to 35, 20 units up and rising 30 a second |
| Sprite | the sprite played once, added at the message's brightness |
| Sparks | an unseen shower flying out at up to 300 units a second under half gravity, bouncing off the world, glinting with `sprites/richo1.spr` every tenth of a second for 0.5 to 1.5 seconds. The glint fades out over an eighth of a second, a steady glow. The streaks the engine adds are not drawn |
| Blood sprite | a splash in the palette colour a little past the message's, playing through in a quarter of a second, and the message's size plus 2 to 16 drops flying out under half gravity in the colour before it, for 1 to 3 seconds |
| Sprite spray | sprites flying along a direction with noise under half gravity, fading out in a third of a second |
| Armour ricochet | the glint and one of `weapons/ric1.wav` to `5` |
| Gunshot decal | one of the ricochets for half of them, and the decal (below). Its dust is not drawn yet |
| Model | a studio model thrown out and bouncing as a shell does, for its given life, with a shell's or shotgun shell's sounds when the message names one |
| Breakable | the message's count of pieces of its model, or when it gives none one for each 144 square units of the box's mean face, at most 100 (`TE_BREAKMODEL`). Each starts at a random spot in the box outside the world's solid, four tries, and flies out at the message's velocity plus up to its random speed sideways and upward, under half gravity, bouncing off the world, most of them spinning, fading after the message's life and up to a second more. Glass and see-through pieces are blended at amount 128. One bounce in six makes the material's sound (`debris/glass1.wav` and the rest), louder the faster it falls |

A new session clears them all. Sprites take their colours from the game's palette
(`gfx/palette.lmp`) where the message gives an index. `mix_probe` counts the temporary entities
it makes, by model.

**Measured (2026-10-18):**

| Demo | Stretch | Made |
|---|---|---|
| `naz_crossfire_pb_01.dem` | 0 to 300 s | 7 explosions (`zerogxplode.spr`) with their sounds, 6 smoke puffs, 78 blood splashes with 1,651 drops, 101 spark showers with 1,104 glints, 28 shells. 300 s mix in 3.7 s, against 3.2 s before |
| `run_92.dem`, singleplayer | 0 to 100 s | 7 explosions, 61 blood splashes with 766 drops, 163 spark showers with 1,739 glints, 63 `muz1.spr` sprites, 35 `tinyspit.spr` sprays, 108 shells, and all five ricochets heard |
| `run_92.dem`, singleplayer | 0 to 400 s | 223 pieces of breakables, 88 of wood, 78 of metal plate, 30 of machinery, 16 of concrete and 11 of glass, with the wood, metal, concrete and glass sounds heard |

Each piece of a breakable shows one of its model's bodies at random, as the engine chooses among
them.

### Decals

**Observed:** no recording sends `svc_decalname`. The server names a decal by its place in
`decals.wad`'s directory, and so does the client, both reading the same file. `valve/decals.wad`
holds 222 and AG has none of its own. Read that way, the indices the demos send are the expected
ones: `{BLOOD1` to `{BLOOD6` (27 to 32) where players bleed, `{SCORCH1` and `{SCORCH2` (176, 177)
under explosions and `{YBLOOD` ones under aliens. Each decal texture keeps its colour in its
palette's last entry, from white at index 0, and every texel's index is how much of it shows.

**Implemented**, from the engine as Xash3D restates it, not yet compared with the game:

| Part | How |
|---|---|
| Messages | world decals (`TE_WORLDDECAL`, and `HIGH` for the second 256), decals on an entity (`TE_DECAL`, `TE_DECALHIGH`), gunshot decals and map decals (`TE_BSPDECAL`, which names the brush model) |
| Keeping | the newest 4096, oldest dropped first, cleared by a new session, kept in playback's state so a seek finds the same ones |
| Brush entities | a decal on a door or other brush entity is kept in the brush's own space, so it moves with it. One aimed at any other entity is dropped, as the engine puts decals only on brushes |
| Placement | the texture at one unit a texel, centred where it was aimed, right along each face's texture s axis and down along its t axis within the face's plane, on every face whose plane is within 4 units of the point (`DECAL_DISTANCE`). Each face is cut to the decal's square. Turbulent faces take none |
| Drawing | over the face a tenth of a unit off it, blended by the texels' coverage, lit by the face's lightmaps and light styles, after the opaque world and brushes, in the order they were made so a newer one covers an older |

Each decal is laid on the map once, when it first appears, and kept until playback drops it. After a
seek far into the 32-player `20230909_224143_stalkyard.dem` the full 4096 are laid in 107 ms, once,
and one new decal takes about 60 microseconds. While decals are only added, the window sends the
GPU the new ones' triangles alone, into a buffer with room to grow. A seek, a dropped decal or a
full buffer sends them all again (55,000 vertices for 4096 decals).

### Bullet impacts

The server leaves the impacts of the glock's, MP5's, shotgun's and python's bullets to the clients,
whose event handlers trace them (`EV_HLDM_FireBullets`). Playback traces them as it runs the
handlers of 10c.

**Implemented**, from the SDK's `ev_hldm.cpp`, not yet compared with the game:

| Part | How |
|---|---|
| Aim | from the eye, 28 units over the event's origin or 12 for a ducking player, along the event's angles. The glock, MP5 and python take the event's spread as it is (the server's shared random spread), and buckshot spreads over a disc of the weapon's cone: 4 pellets of the shotgun's single shot and 8 of its double in multiplayer, 6 and 12 in singleplayer |
| Trace | through the world and each solid brush entity's model (`SOLID_BSP`), unturned, and through each other living player's standing box where the latest snapshot has it |
| Material | of the face met in the world, the first along the line whose texture extent holds the crossing point (`R_TraceTexture`), named in `sound/materials.txt` by its first 12 letters, a `-0`, `+0`, `{`, `!` or `~` before the name left out. Concrete when the file names none or the bullet met a brush entity |
| Sound | the material's own (`player/pl_metal1.wav` and the rest, `debris/wood`, `debris/glass`) at its volume, `weapons/bullet_hit1.wav` or `2` for a player, at pitch 96 to 111. Buckshot makes none |
| Decal | on the world or a brush entity, one of `{shot1` to `{shot5`, or `{bproof1` on a brush drawn other than normally, with one of the five ricochets half the time |
| Tracers | the MP5's first bullet of every two is a tracer and leaves no impact, counted for each shooter |

**Measured (2026-10-18):** in the first 300 s of `naz_crossfire_pb_01.dem`, the 28 glock bullets
leave 23 shot decals (`{SHOT1` to `{SHOT5`, 179 to 183 in `decals.wad`) and play `bullet_hit` for the
5 that meet a player. The 300 s mix in 4.1 s, against 3.7 s before.

Not yet: hitboxes, as the game traces players through their models' hitboxes where its prediction
has them, so a shot past a model but inside its box meets the box here; brush entities turned by
their angles; the crossbow's bolts; player sprays (`TE_PLAYERDECAL`);
particles (the sparks' streaks, the explosions' flicker and an impact's dust); and dynamic lights.

### Beams and tracers

**Implemented**, from the engine as Xash3D restates it (`R_BeamFollow`, `R_BeamPoints`,
`R_TracerEffect`, `CL_DrawTracers`), not yet compared with the game:

| Kind | How |
|---|---|
| Trail (`TE_BEAMFOLLOW`) | follows its entity, leaving a point behind each time the entity has moved 32 units. A strip from the entity back through its points, its sprite repeating once a point, in the message's colour by its brightness, fading with each point's age over the trail's life and gone at the oldest. When the entity is gone the trail stops growing and ends with its last point. Its width is the message's, in units either side |
| Beam (`TE_BEAMPOINTS`) | a straight strip between two points for its life, its sprite at the message's frame and frame rate, its texture once each 100 units sliding at its scroll speed, a tenth of the message's width either side. The engine's noise, which bends a beam into a bolt, is not drawn |
| Tracer (`TE_TRACER`, and the MP5's every other bullet) | starts 20 to 39 units past where it was fired and flies towards where it ends at 6000 units a second (`tracerspeed`, `traceroffset`). Drawn as long as a tenth of its flight at most, by 0.8 (`tracerlength`), 1.5 units either side, a soft line in the default tracer colour (`tracerred`, `tracergreen`, `tracerblue` 0.8, 0.8, 0.4, by `traceralpha` 0.5). A player's MP5 tracer starts 4 units under, 2 right of and 16 ahead of the eye |
| Removal | `TE_KILLBEAM` ends an entity's trails, and a new session ends everything |
| Drawing | strips turned to face the eye, added after the other translucent things |

**Observed:** in `run_92.dem` the alien grunts' hornets fly with red and orange trails
(`sprites/laserbeam.spr` in 179, 39, 14 and 255, 128, 0 at half brightness, the SDK's hornet
colours), and at 13.7 s two beams of `sprites/lgtning.spr` cross the room.

Not yet: beams between entities (`TE_BEAMENTS`, `TE_BEAMENTPOINT`), rings, disks and cylinders, beam
noise, and beams drawn by entities, such as a tripmine's laser.

### Gauss and Egon (10e)

**Implemented**, from the SDK's `EV_FireGauss`, `EV_EgonFire`, `EV_EgonStop` and `EV_UpdateBeams`,
not yet compared with the game:

| Part | How |
|---|---|
| Gauss shot | traced from the eye along the aim, as a beam of `sprites/smoke.spr` for a tenth of a second, 1 unit either side for a primary shot and 2.5 for a charged one, the first from the gun's first attachment (the viewmodel's when the view is through the shooter) |
| Gauss reflection | a brush met less than 60 degrees off its plane reflects the shot, which loses that share of its energy, leaving a glow of `sprites/hotglow.spr` and three balls |
| Gauss impact | a brush met more squarely takes a shot decal, sometimes a ricochet, and a glow. A charged shot then punches through once, when the wall is thinner than its energy, losing that much, with balls, a decal and a glow on the far side. A primary shot stops with a smaller glow and eight balls. The shot goes on through players, the next trace passing the one it went through as the server's does, and ends when its energy is 10 or less or after ten hits |
| Glows and balls | steady glows, fading after their life (`R_TempSprite`). Balls fly out at 100 or 200 units a second with spread, cycling their frames at ten a second, bouncing under half gravity, fading after their life and up to four seconds more (`R_Sprite_Trail`) |
| Egon | for the local player only, as only the local player's are the client's own, two beams of `sprites/xbeam1.spr` from the gun, 3.5 and 5 units either side, their textures sliding at 55 and 25 a second, aimed each group from the eye along the view to 2048 units or what they meet, and ended by the Egon's stop. Other players' Egon beams are the server's beam entities, not drawn yet |
| Colours | a handler's colour times its brightness, at most full. This draws the gauss's primary beam yellow, its charged beam white and the Egon's pale blue, the only reading under which both handlers' numbers make sense, as one gives colours out of 255 and the other under 1 |

A trace that starts inside a wall now keeps where it goes on to once out, as the client's does,
which the punch through needs to find the wall's far side.

**Observed:** in `stalkx_naz_248.dem` the recorder's gauss shots draw from the viewmodel's barrel,
reflect off walls at a glancing angle, and leave glows and balls where they land.

Not yet: the beams' noise, the Egon's for other players, the lights of the gauss and Egon, and the
view's punch.

## Seeking with effects (10e)

Everything this milestone adds is part of playback's state, so a checkpoint holds it and a seek
restores it, then plays on to the moment asked for as sequential playback would: the sound channels
with their pitches and loops, the events still waiting for their time, the animations' frames and
their muzzle flashes, the temporary entities with their seeds, the decals, the trails, beams and
tracers, the Egon's beams. The checkpoints' size estimate counts the effects alive.

`examples/effects_seek.rs` builds every checkpoint of a demo with the game's files, as the worker
does, then seeks to 100 moments at random, in random order, and compares all of that with one
sequential pass.

**Measured (2026-10-19):**

| Demo | Length | Seeks differing |
|---|---|---|
| `stalkx_naz_248.dem` | 1216 s | 0 of 100 |
| `run_92.dem`, singleplayer | 104 s | 0 of 100 |
| `naz_crossfire_pb_01.dem` | 1206 s | 0 of 100 |
| `20230909_224143_stalkyard.dem`, 32 players | 937 s | 0 of 100 |
| `ray_vs_alex_cross.dem`, spectator | 103 s | 0 of 100 |
| `dblcross_salchi_naz.dem` | 1314 s | 0 of 100 |

The first run found every seek in `naz_crossfire_pb_01.dem` after 150 s different. The events'
count of each shooter's bullets was a hash map, whose order differs between two playbacks, so the
comparison failed although the counts were the same. It is an ordered map now.


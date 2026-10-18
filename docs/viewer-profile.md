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
- A new session silences everything

Not yet: sample lengths and loops (every sound counts as finished after ten seconds until M10 reads
the sample files), the limit of eight dynamic channels, and sentences resolved to their words.

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
§13.8). It keeps a playback controller and worker per demo, and a thread that reads the session's
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
| Sky | the `worldspawn` `skyname`, `desert` by default, from `gfx/env/<name>{rt,lf,bk,ft,up,dn}.tga` in the sky box's layout, drawn wherever nothing else was |

Not drawn yet: visibility and frustum culling (the whole world is drawn every frame), random tiling
textures (`-0` names), scrolling conveyors, and decals. Sprites and the viewmodel come with 9d.

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

The leg state lives in playback's scene, so checkpoints keep it and a seek reaches the same legs as
playing does. The gait frame takes the whole life's distance through the current gait sequence, so
when the gait sequence changes the legs keep their place in the cycle by distance, where the game
carries on from the frame it had. That shows as a jump in the legs' phase at a change from walking
to running, for example.

**Observed:** AG's `w_weaponbox.mdl`, the box a player drops on death, has a black and white
texture, so it looks like the checker drawn for a missing texture.

Not drawn yet: team colours (`DM_Base` and `remap` textures), sequence transitions, mouth
movement, chrome and additive model textures, and the sky's light on models.

### Viewmodel and sprites (9d)

**Implemented**, from the game's client and GLQuake's sprite drawing, not yet compared with the game
(9e):

| Part | How |
|---|---|
| Viewmodel | the model the recorder's client data names (`viewmodel`), playing the last weapon animation, from a record or a message, from when it started. At the view's origin and angles, the pitch turned the other way as the game gives it to a viewmodel, and lit by the light under the eye. Drawn last against a cleared depth, so it never sinks into a wall |
| Viewmodel hidden | while the recorder's health is 0 or less, at intermission, and while spectating other than through a player's eyes |
| Sprites | `IDSP` version 2, single frames and frame groups, in normal, additive, index-alpha and alpha-tested formats. Every sprite in the surveyed install reads (268, AG's sprite items included) |
| Sprite entities | an entity whose model is a `.spr`, or one of AG's sprites under a `.mdl` name, at its frame and scale (0 meaning 1) |
| Orientation | facing the view, facing the view upright, facing the camera's position upright, turned by the entity's angles, or facing the view rolled by the entity, as the five `SPR_` types say |
| Render modes | normal and alpha modes cut out at half alpha, texture and colour modes blended by the render amount, additive mode added, and glow added without depth, fading as 19000 over the squared distance between 0.05 and 1. Additive and glowing sprites take the render colour, white when it is black. Drawn after translucent brushes, the furthest first |

Not drawn yet: the viewmodel's body groups and the bob and lag the game adds to it, glow sprites'
check for what hides them, and the sprites of temporary effects, which come with M10.

**Measured (2026-10-18)** with `--viewer-bench 10`, playing from 20 s in a 1920 by 1080 window with
the view 1098 by 950 pixels, while the library scan ran:

| Demo | Models in view | 9a mean, p95 | With the world of 9b, mean, p95 | With the models of 9c, mean, p95 |
|---|---|---|---|---|
| `stalkx_naz_248.dem` | 4 | 3.7, 4.8 ms | 3.4, 4.3 ms | 3.2, 4.8 ms |
| `ray_vs_alex_cross.dem`, spectator | 8 to 9 | 6.7, 19.6 ms | 3.9, 5.1 ms | 4.6, 6.3 ms |
| `run_92.dem`, singleplayer | 10 | | 5.2, 6.3 ms | 2.9, 4.0 ms |

The library scan's load changes between runs, so the columns are not a measure of each step.

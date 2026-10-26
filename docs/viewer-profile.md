# How the demo player works

What the demo player (the viewer) reads, decodes and presents, and how far each part is proven.
Statements marked **Observed** were checked
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

The three options are the game's own. "Allow custom addon content" and "Use high quality models"
are `addons_folder` and `hdmodels` under the user's `Software\Valve\Half-Life\Settings`, read from
the registry on Windows and from Steam's `~/.steam/registry.vdf` elsewhere. The language is the
one Steam runs the game in, `UserConfig` > `language` in `appmanifest_70.acf` in the `steamapps`
folder holding `common/Half-Life`, and English has no folder of its own. On this machine both
switches were on and the language English, so `valve_hd`'s rocket and `valve_addon`'s BugfixedHL
sprites are now used, as the game uses them (2026-10-24).

Settings, under Game content, show the three as the game has them and can turn each on or off for
the player alone. The registry and Steam's files are only read, never written, so the game keeps
its own. A setting changed back to the game's follows the game again. With HD models turned off
there, the rocket at 14:30 of `afafsd.dem` is the old model again.

Within a folder its PAK archives come before its loose files, the highest-numbered `pakN.pak`
first, as in Quake, whose file system GoldSrc's descends from. This install has no PAK archive,
so the order inside a folder cannot be checked here.

Settings can name more Half-Life folders after the first, for maps, models and sounds it lacks.
Each is searched in the same order once everything in the one before it has been, so the first
folder's `valve` still comes before a later folder's mod. A later folder that can't be read is
left out, and its files are named with their folder in the list of what's missing. The floor
under players for midair kills is read through the same folders.

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
| sound `!HG_ALERT0` | a sentence from `sound/sentences.txt`, resolved with the sound system |
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

`lambda-slicer-playback` plays a demo through the core decoder, one record group at a time.

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
on from, and is tested to end where sequential playback does. Each request carries the count of
changes sent up to it, so a change sent after the request being answered always stops it. A seek
gets 100 ms before a newer one stops it, so dragging the playhead, which seeks every frame, still
shows frames, and dropping the worker stops it at once.

### Scene

Entities come from the decoder's newest stored snapshot, so a packet the client rejected never
reaches the scene. An entity is its session, number and life. A life ends when the entity leaves the
packet or its model changes, so a reused slot is never taken for the entity before it. A new session
starts an empty scene. The recorder's camera is the `ref_params_t` view of each network record, with
the field of view of the last client data record. Interpolation, attachments, effects and the HUD
are described below.

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

How long a sound plays comes from its WAV file ("Sound"). Without the files every sound counts as
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
any of which a later packet may take as its delta base, so in a busy game it weighs about 1 MiB, and
3 MiB with 32 players (see "Performance").
When a demo's checkpoints pass 256 MiB by the decoder's estimate, every other one is dropped and the
spacing doubles. Checkpoints live in memory for the selected demo only. Keeping them on disk
between runs is left for later.

**Tested** on a synthetic demo with a second session and a group whose time steps back, against
sequential playback that never uses a checkpoint: forward and backward seeks to 61 moments and on
past each, 300 random seeks while checkpoints are still being added, group steps both ways, a clip
looped three times in display-sized steps, and the worker's answers once its index is complete.
The comparison covers playback's whole state (`Playback::fingerprint`), with the decoder's public
state and latest snapshot, and the frame presented. A gauss shot every 1.5 s waits half a second for
its delay, so some checkpoints are taken while it waits. Three deliberately broken restores (a
decoder counter changed, the source's read-ahead record lost, the waiting events dropped) all fail
it.

`Playback::keep_checkpoint_at` keeps a checkpoint after a chosen group whatever the spacing, for a
place to come back to. The equivalence harness uses it to restore while an event waits (see
[viewer validation](viewer-validation.md), "Equivalence").

**Measured (2026-10-18)**, before the scene's entities were converted faster (see "Performance"),
with `examples/seek_probe.rs`, release build: every checkpoint built as
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
and 5.1 ms at p95. Every indexed p95 is within the target of 200 ms. The longest demo is
closest to it, as its spacing doubled twice, and a longer one would double again. A first seek
before the index reaches the destination replays from the start, at about 3 ms per second of a busy
demo.

## Viewer

The demo view plays the demo once the Half-Life folder is known, from the settings or from
`--game DIR`. A map that cannot be read is named in place of the view with the reason, nothing of the map
before it drawn, and is not asked for again. Playback that stops before the demo's end, because a
part of it could not be decoded, says why under the view. `--viewer-capture` saves its frames at chosen times for comparing with the game
([viewer validation](viewer-validation.md)), each once the map, the server's sky and every model
and sprite the frame draws are in, effects included. It keeps a playback controller and worker per demo, and a thread that reads the session's
map and the studio models its entities use. Play, pause and a time bar drive the controller, and
`--viewer-at SECONDS` seeks there once the demo is loaded.

### Timeline and viewer

The demo view's event table and timeline move the viewer's playhead through its controller:

| Action | What the viewer does |
|---|---|
| Choosing an event, in the table, on the timeline or with previous and next event | seeks to the event's time, playing on if it was playing. A seek applies every group at or before the time, so the event has happened |
| Clicking the timeline away from a mark | seeks to the time under the pointer, and chooses the clip there as before |
| Previous and next event | the rows before and after the chosen one in the table as filtered, or around the playhead when none is chosen |
| Play event | seeks to the chosen event's time less the preroll and plays, pausing at its time plus the postroll (the clip padding in Settings, 5 s each by default), once, as a looped clip stops looping. Any other seek, pause or play forgets the pause |
| Playhead | a line across the timeline at the viewer's time, moving while it plays |

The analysis and playback take an event's time from the same clock (a record's playback time), so
a time from the table needs no conversion. A demo whose times start again in a later session has
the same time twice, and a seek by time finds its first. Zooming and panning the timeline only
change what it shows.

### Transport

| Control | Key | What it does |
|---|---|---|
| Play, Pause | Space, L, or a click on the view | plays forward at the chosen rate, with sound, or pauses, playing backward too. A click plays on the way playback went last, so it rewinds again after pausing a rewind, and plays forward from the start once a rewind reached it. Paused at the end of the demo, a click plays it again from 00:00 |
| Rewind | J | plays toward the start at the chosen rate, without sound, pausing at the start. Past a loop's start it goes on from the loop's end |
| Pause | K | pauses either way |
| Previous frame, Next frame | comma, period | moves one recorded group, pausing |
| Rate | | 0.25×, 0.5×, 1×, 2× or 4×, for either direction, the playhead staying where it is |
| Time bar | left and right arrows | seeks, the arrows five seconds back or forward |

Keys work while no text box has the keyboard's focus. While a button or the time bar has it,
space and the arrows are left to it. Without a sound device "No sound" shows beside the controls,
the reason on hovering it. Sound plays only forward, the device's blocks from before
dropped when the direction changes.

Backward playback seeks a little earlier for every frame shown, each a restore from the nearest
checkpoint and playing forward from it. **Measured (2026-10-19)** on `naz_crossfire_pb_01.dem`
(checkpoints 16 s apart), release build, 120 steps of 1/60 s back: from 600 s, 26 ms per step at
the median and 35 ms at the 95th percentile, and from 300 s, 48 ms and 97 ms (209 ms at most).
Backward playback shows 10 to 40 pictures a second, still at the right time for each.

### Viewer settings

The settings window has a tab for the viewer, saved with the other settings and in all 14
languages. The Half-Life folder sits with the other folders on the general tab ([GUI](gui.md),
"Settings"):

| Setting | What it does |
|---|---|
| Half-Life folder | the folder holding `valve` that the viewer reads the game from. `--game` comes first. Changing it opens the viewer again for the demo shown |
| Gamma, brightness, texture gamma, light gamma, overbright lighting | the game's `gamma`, `brightness`, `texgamma`, `lightgamma` and `gl_overbright`, defaulting to the game's own (2.5, 0, 2.0, 2.5 and off) and kept in the game's ranges. Applied as soon as they change |
| Field of view as recorded, or a field of view | the recorded field of view, or one from 10 to 150 degrees across the view's width |
| Show the weapon in view | the viewmodel and its muzzle flashes, as `r_drawviewmodel` does |
| Volume | the sound's master volume, 0.1 by default, the same as the slider beside the time bar, saved once the pointer is let go |

Gamma works as the game's tables do (`BuildGammaTable`, as Xash3D restates it), worked out in the
shaders for each texel (`lambda-slicer-render`'s `gamma`):

| Part | How |
|---|---|
| Textures | world and model textures raised to `texgamma / gamma`, as the game does to their palettes. Sprites, skies and decals keep their colours |
| Light table | light taken to `lightgamma`, multiplied by a brightness over 1, its darkest part stretched by the brightness (up to 1/8 at 0, 1/20 from 1) and taken to `1 / gamma` |
| Lightmaps | the light under the face's styles at four times its stored value, raised by the light gamma's root of two, through the light table. With overbright, not raised but doubled after the table |
| Models | their light, 0 to 255, at four times its value through the light table |

The tables match Xash3D's published entries for two settings to within a step. This replaces the
fixed brightness of 1.6 the viewer drew with before.

A list under the view, folded away under its count, names every file the viewer could not read
and why: maps, models, sprites, skies and sounds. A capture now waits for a frame drawn before
saving it, where it once skipped the first frame of a run.

### See-through walls

A mode, off by default, shows players behind walls. "See through walls" beside the controls turns it
on and off, and Settings keeps it, with a second choice for every other model too, projectiles and
items among them. `--viewer-see-through` turns it on for one run.

| Part | How |
|---|---|
| Silhouettes | each chosen model, a player's weapon with it, drawn again in one colour at 0.6 after the world, its brushes and decals and before any model, only where they are nearer (depth compared greater, nothing written). The models then cover them where seen, so only the world and its brushes hide a player into a silhouette, not another player |
| Colours | a player's top colour as a hue, kept a little pale, which team games set by team. White for a player without one, and a pale yellow for other models |
| A model in view | not tinted, as it is drawn over its own silhouette, its arms and weapon across its body too. The faces turned away from the eye are left out, so a model partly behind a wall is tinted only where the wall hides it |

It can only show what the demo holds. The server sends the players in the recorder's potentially
visible set, which often holds players still behind a corner, and a server that hides the players
the recorder cannot see, as some anti-wallhack plugins do, leaves nothing to show. The setting's
description says so. It changes only how the viewer draws a recording.

### Clips in the viewer

| Action | What the viewer does |
|---|---|
| Play, on a clip | plays the clip from its start, pausing at its end |
| Loop, on a clip | plays the clip from its start, going back to the start at its end until Loop is turned off, the clip is removed, or another clip or an event is played. Changing the looped clip's start or end moves the loop |
| Editing a start or end | shows that moment, paused, while an edge is dragged on the timeline and when a typed time is committed. The clip keeps its own range, whatever the playhead does |

Going back to a loop's start is a seek, so the state at the start is restored as for any seek,
and the sound device drops every block it holds from before ("Sound output", "Old blocks"). The mixer also
mixes nothing from the loop's end, or a played range's end, on: it runs about 40 ms ahead of what
is heard and would otherwise mix past the end before the window's next frame goes back. A block
that starts before the end is mixed whole, so at most 10 ms past it can be heard.

### Scrubbing

Dragging the time bar, the timeline's ruler along its top, or the playhead line on the timeline
scrubs: playback holds still and silent while the playhead follows the pointer, and when the drag
is let go it plays on from there if it was playing, its sound starting from the state there with
none of the sounds passed over. Escape during a drag goes back to where it began, and the rest of
that drag moves nothing. A click on the time bar or the timeline seeks as before.

Every scrub is a new revision, and the controller shows no answer to an older revision, so a
drag whose seeks take longer than the pointer's moves showed few of the frames the worker
finished. While a scrub lasts, the controller now shows the answer to any of its requests, and
only the answers to requests from before the drag are refused. Once let go, the rule is the usual
one again.

**Measured (2026-10-21)** on `naz_crossfire_pb_01.dem`, release build, a drag sending a scrub
every 16 ms for 2 s through 60 s of the demo, frames counted as the controller accepted them for
showing, before and after that change:

| Drag | Frames shown, before and after | Longest wait between frames | Last frame after letting go |
|---|---|---|---|
| Indexed, 200 to 260 s | 125, 126 | 34 ms, 34 ms | at once |
| Indexed, 700 to 640 s, backward | 27, 82 | 504 ms, 68 ms | 81 ms, 82 ms |
| Before the index, 200 to 260 s | 25, 41 | 1,071 ms, 848 ms, the first seek | 0 ms, 9 ms |
| Before the index, 700 to 640 s | 13, 61 | 1,210 ms, 712 ms | 19 ms, 27 ms |

An earlier measurement counted the worker's answers rather than the frames shown, and missed
this.

Before the index covers the demo, the first seek of a drag plays from the nearest checkpoint, the
start at worst. A seek a newer one stops leaves playback where it got to, and the next goes on from
there, so the drag still catches up.

### What is drawn

| Part | How |
|---|---|
| Camera | the recorder's view of each network record, with the recorded field of view across the screen's width, as the game's is. 90 degrees when the recording names none |
| World | the session's map, read again when a later session changes it |
| Entities | each with a studio model, posed from its recorded sequence, frame, frame rate, animation time, controllers and blends, the frame estimated as the renderer's `StudioEstimateFrame` does. Client time is the last `svc_time` plus the source time since that packet |
| Players | the model their userinfo names (`models/player/<name>/<name>.mdl`), the stock player model when it is not installed, drawn upright |
| Left out | the entity the recorder views from, the player a spectator sees through the eyes of (observer mode 4 and its target, from the recorder's client data as the game's client takes them), and entities with `EF_NODRAW` |

**Observed:** AG ships some item models as sprites under a `.mdl` name (`ag/models/w_357.mdl`
starts with `IDSP`). The game tells a model's kind by its header, and so does the viewer, which
leaves those items for the sprites ("Viewmodel and sprites"). They are index-alpha icons in one flat colour each, the
palette's last (magenta for the .357 and its ammunition, yellow for the 9 mm weapons, green for the
crossbow), and an ammunition box's icon is a box outline round a bullet.

They are drawn in normal mode, and the game blends them by their index alpha rather than cutting
them out, so a battery shows its shading with the floor through it. The viewer cut them out at
first, which left flat shapes in one colour, and now blends a normal-mode index-alpha sprite by its
texels' alpha as the game does (2026-10-24, compared with a screenshot of a battery on `boot_camp`).

### World

**Implemented**, from the game's renderer as GLQuake's source shows it, not yet compared with the
game ([comparison](viewer-validation.md)):

| Part | How |
|---|---|
| Brush entities | every BSP model an entity names as `*N`, turned and moved as the renderer turns brush models (yaw about z, pitch about y the opposite way, roll about x) |
| Render modes | normal and alpha-tested drawn with the world. Texture mode lit and blended by the render amount, colour mode a flat blended colour, additive and glow modes added unlit. Translucent entities are drawn last, the furthest first, and one at amount 0 is not drawn |
| Light styles | up to four lightmaps a face, each weighed by its style's brightness at the client's time. A pattern steps ten times a second, `a` to `z` meaning 0 to 550 where 256 is the lightmap as stored. A style no message set is 256 |
| Unlit faces | full brightness, as the game draws a face without light data |
| Animated textures | `+0` to `+9` chains at ten frames a second, and the `+a` to `+j` chain while a brush entity's frame is not 0 |
| Conveyors | a brush entity's faces whose texture's name starts with `scroll` slide along the texture's s axis by the time, at the speed the entity's render colour holds, green and blue a 16th of a unit a second over the texture's width, against the axis unless red is set (`DrawGLPolyScroll`). Checked on screen on `vengeance`'s `func_conveyor` panels |
| Water | `!` and turbulent faces unlit, their texture coordinates warped by eight texels in a sine of the other coordinate and the time, at 64 texels a repeat |
| Sky | the sky the server's movement variables name (`sv_skyname`), and until it names one the `worldspawn` `skyname`, `desert` by default, from `gfx/env/<name>{rt,lf,bk,ft,up,dn}.tga` in the sky box's layout, drawn wherever nothing else was. Confirmed against the game, where the two differ, in `ls_9e_cross` ([viewer validation](viewer-validation.md)) |

Not drawn yet: visibility and frustum culling (the whole world is drawn every frame) and random
tiling textures (`-0` names).

### Models

**Implemented**, from the game's client as the Half-Life SDK's `StudioModelRenderer` shows it, not yet
compared with the game ([comparison](viewer-validation.md)):

| Part | How |
|---|---|
| Interpolation | entities drawn 0.1 s behind the client's clock (`ex_interp`), each moved along its own samples from the last 32 snapshots, angles the short way round. Not through a new life, a move over 128 units between snapshots, or past the latest snapshot |
| Client clock | the latest snapshot's `svc_time` plus the source time since it arrived |
| Legs | `StudioEstimateGait` and `StudioProcessGait`, stepped once a record group, each group being a frame the recording client drew: the legs face the way the player moved, turn towards the body while standing, turn round with the movement counted back when the body faces more than 120 degrees away, and the torso twists the rest of the way over the four spine controllers |
| Gait frame | the distance moved through the gait sequence's linear movement, or its frame rate over the time passed when it does not move |
| Aim | a player with a gait sequence aims through its first blend from its pitch (`StudioPlayerBlend`), the rest of the pitch turning the model. One without keeps its controllers centred |
| Held weapons | the player's `weaponmodel`, its bones named as the player's, ignoring case, taking the player's transforms (`StudioMergeBones`), the rest posed with the player's sequence and carried on from their parent as merged. AG's `p_` models name their arm bones otherwise than the player models (`Bip01 R Forearm` for `Bip01 R Arm2`), so only the spine and the hand match, and the weapon follows the hand. Until 2026-10-20 the rest kept their own pose, which left AG's weapons floating above their holders |
| Lighting | the colour under the model from a line 2048 units down through the BSP tree to the first lit face, its lightmaps weighed by their styles (`R_LightPoint`). Its brightest channel is the ambient level up to 128 and the light from above up to 192, softened by a lambert of 1.5. Where nothing is under a model it is lit at 128 and 64 |
| Texture flags | as the engine's studio renderer draws them: an additive texture of a solid model added over the scene with the translucent things, writing no depth, a chrome one's coordinates from its normal against two vectors square to the line from the eye to its bone (`R_StudioSetupChrome`), and a flat-shaded one lit by the ambient level and 0.8 of the shade whatever its normal. AG's viewmodels in the surveyed install mark every texture additive, so they show see-through as in the game |
| Body and skin | each entity's body and skin choose the submodel of each body part (`(body / base) % models`) and the skin family, as `StudioSetupModel` does. A body past the model's own is the first's, and each combination is read once, after the model's first body is in. Players' models take theirs in their team colours, from the stock model when theirs is not installed. The viewmodel takes the body of its last animation |
| Render modes | normal models solid. Colour, texture and alpha modes blended by the render amount, glow and additive modes added by it, each lit as a solid model and without the render colour, which the game's studio drawing takes only for its glow shell. Translucent models are drawn after the sky together with translucent brushes and sprites, all from the furthest in, and one at amount 0 is not drawn. A player's weapon is drawn as the player is |
| Glow shell | a model whose entity has `kRenderFxGlowShell`, as AG's CTF flags and their carriers do, drawn again with every point pushed out along the average of its normals by a 128th of a unit per render amount, at least one. Every texture of the copy is `sprites/shellchrome.spr` added over the scene in the render colour, its chrome coordinates reflecting a point 4000 units out that turns around the world at the game's default `r_glowshellfreq` of 2.2, rising and falling a third as fast, in place of the eye. Only the faces turned to the view are drawn. A carrier's weapon has it too, and so does the weapon in view while the player whose eyes it is from has one, as AG's client does |
| Team colours | a player model's `DM_Base` textures with palette entries 160 to 191 turned to the top colour's hue and 80 to 111 to the bottom colour's, each keeping its brightness and saturation, as the game's `PaletteHueReplace` does. The surveyed AG install's own player models have no such textures |

The leg state lives in playback's scene, so checkpoints keep it and a seek reaches the same legs as
playing does. The gait frame takes the whole life's distance through the current gait sequence, so
when the gait sequence changes the legs keep their place in the cycle by distance, where the game
carries on from the frame it had. That shows as a jump in the legs' phase at a change from walking
to running, for example.

**Observed:** AG's `w_weaponbox.mdl`, the box a player drops on death, has a black and white
texture, so it looks like the checker drawn for a missing texture.

Not drawn yet: `remap` textures, sequence transitions, mouth movement, render effects other than
the glow shell, and the sky's light on models.

### Viewmodel and sprites

**Implemented**, from the game's client and GLQuake's sprite drawing, not yet compared with the game
([comparison](viewer-validation.md)):

| Part | How |
|---|---|
| Viewmodel | the model the recorder's client data names (`viewmodel`), or through a player's eyes the viewmodel of that player's weapon model (`V_FindViewModelByWeaponModel`), starting at its first animation when the weapon changes, playing the last weapon animation, from a record or a message, from when it started. At the view's origin and angles, the pitch turned the other way as the game gives it to a viewmodel, and lit by the light under the eye. Drawn last against a cleared depth, so it never sinks into a wall |
| Viewmodel hidden | while the recorder's health is 0 or less, at intermission, and while spectating other than through a player's eyes |
| Kick and bob | the recorded view angles already carry the kick of shots and hits, and the recorded view height the recorder's bob, as their client added them. The recorder's viewmodel takes their aim without the kick (`cl_viewangles`), so it stays put while the view kicks, a unit under the view. It bobs by the recorder's own bob, the view's height over their eyes' within the game's -7 to 4: pushed 0.4 of it along the aim and tilted 0.3 of it in pitch, half in yaw and all of it in roll (`V_CalcNormalRefdef`). Settings can instead set a `cl_bob` of their own, 0 for none and 0.01 by default, and work the bob out from it, the game's `cl_bobcycle` 0.8 and `cl_bobup` 0.5 and the recorder's speed along the ground (`V_CalcBob`). A water level or a step up also lifts the view, which is then taken for bob. Through another player's eyes the weapon neither kicks nor bobs |
| Sprites | `IDSP` version 2, single frames and frame groups, in normal, additive, index-alpha and alpha-tested formats. Every sprite in the surveyed install reads (268, AG's sprite items included) |
| Sprite entities | an entity whose model is a `.spr`, or one of AG's sprites under a `.mdl` name, at its frame and scale (0 meaning 1) |
| Orientation | facing the view, facing the view upright, facing the camera's position upright, turned by the entity's angles, or facing the view rolled by the entity, as the five `SPR_` types say |
| Render modes | normal and alpha modes cut out at half alpha, except an index-alpha sprite in normal mode, blended by its texels' alpha, texture and colour modes blended by the render amount, additive mode added, and glow added without depth, fading as 19000 over the squared distance between 0.05 and 1, and grown by its distance over 200 to keep its size on screen unless it is steady. A glow the world hides from the eye, the line to it stopping more than 8 units short, is not drawn, brush entities and models hiding none (`R_SpriteGlowBlend`). Every mode takes the render colour, white when it is black, as Xash3D says the engine's sprite drawing does (at first only additive and glowing sprites did). Drawn together with translucent models and brushes, all from the furthest in |

**Measured (2026-10-18)** with `--viewer-bench 10`, playing from 20 s in a 1920 by 1080 window with
the view 1098 by 950 pixels, while the library scan ran:

| Demo | Models in view | First drawing, mean, p95 | With the world below, mean, p95 | With the models below, mean, p95 |
|---|---|---|---|---|
| `stalkx_naz_248.dem` | 4 | 3.7, 4.8 ms | 3.4, 4.3 ms | 3.2, 4.8 ms |
| `ray_vs_alex_cross.dem`, spectator | 8 to 9 | 6.7, 19.6 ms | 3.9, 5.1 ms | 4.6, 6.3 ms |
| `run_92.dem`, singleplayer | 10 | | 5.2, 6.3 ms | 2.9, 4.0 ms |

The library scan's load changes between runs, so the columns are not a measure of each step.

## Sound

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

## Sound output

The demo view plays sound on the system's default output device while playing forward, with a
volume slider beside the controls. `lambda-slicer-audio`'s `device` feature holds the output, through
`cpal` (WASAPI on Windows, ALSA on Linux).

| Part | How |
|---|---|
| Timing | a mixing thread keeps a playback of its own of the demo the worker has open, sharing the worker's checkpoints, and moves it to the end of each 10 ms block before mixing the channels it then has. Each block hears the sounds playing during it, whenever the window's frames come |
| Clock | each block is placed where the playback clock will be when it is heard: after the four blocks queued before it, half the block playing and the device's own latency, which the callback measures from the stream's timestamps (11g). The mixer goes on from its own place unless a load or seek comes or it falls more than 40 ms from the clock |
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

#### Sound and picture in step (11g)

`LAMBDA_SLICER_SOUND_SKEW=FILE` with `--viewer-play SECONDS` writes, for every block, how far the
sound heard runs ahead of the playhead when the block begins to be heard, taken from the
stream's timestamps, and beside it (`FILE` with `.video.txt`) how far each picture painted trails
the playhead. The sound tap can be set too, so the device plays silence.

**Measured (2026-10-19)** on Windows, `stalkx_naz_248.dem` played 12 s from 20 s, the first
second left out (about 1,000 blocks and 2,600 pictures):

| | Sound against the playhead | Device latency |
|---|---|---|
| Blocks placed four blocks ahead, as before | 56 ms behind, steady within a millisecond | 47 ms |
| Placed with the measured latency, two runs | 3.9 and 4.6 ms behind, 9 ms at most | 42 and 47 ms |

Pictures trail the playhead by 4.5 ms at the median and 33 ms at the 95th percentile, from the
worker's answer to the paint, not counting the display's own delay. So sound and picture now
agree within a few milliseconds where the sound came 50 ms late, inside the target of 40 ms.

## Client events

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

Not yet: the tripmine's and snark's handlers, which trace first, sparks (event 5002), and
the lights of muzzle flashes and `EF_MUZZLEFLASH`.

## Temporary entities

The server asks the client for effects with `svc_temp_entity`, and the client makes them as
temporary entities alongside the shells of "Client events" (`lambda-slicer-playback`'s
`tempents`).

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
handlers of "Client events".

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
their angles; the crossbow's bolts; player sprays (`TE_PLAYERDECAL`); and particles (the sparks'
streaks, the explosions' flicker and an impact's dust).

### Dynamic lights

**Implemented**, from the engine as Quake and Xash3D restate it (`CL_LinkPacketEntities`,
`CL_ParseTEnt`, `R_AddDynamicLights`, `R_StudioDynamicLight`), checked on screen but not yet
compared with the game. A frame lights the world with up to 32, the nearest to the camera first,
as the game keeps 32.

- **Sources.** An explosion (`TE_EXPLOSION` without `TE_EXPLFLAG_NODLIGHTS`) lights radius 200 in
  250, 250, 150 for 0.01 s, shrinking 800 units a second. `TE_DLIGHT` and `TE_ELIGHT` light as
  their messages say, a `TE_ELIGHT` models alone and from where it was sent rather than
  following its entity. An entity's `EF_BRIGHTLIGHT` lights radius 400, `EF_DIMLIGHT` 200, both
  16 units up, and `EF_LIGHT` 200 at its origin, such as a rocket's. A muzzle flash lights radius
  200 for 0.1 s, 18 units ahead of the gun and 16 up, or of the eye for the weapon in view, from
  each muzzle flash the events show and from `EF_MUZZLEFLASH` on an entity. The game adds up to
  31 to the radius at random each frame, which is left out, and colours lights the engine leaves
  unset white.
- **The world.** Each light adds what is left of its radius after the way off a face's plane and
  the way across it, in its colour, to the face's light, as the game adds it to the lightmap.
  The game does that per lightmap texel, every 16 units, and here per pixel, so the light's edge
  is smooth rather than blocky.
- **Models.** Each light near a model adds what is left of its radius after the way to the model
  to the light under it, in its colour, before the model's ambient and shade are worked out.

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

Beams the server draws as entities, as `env_beam` and `env_laser` do, are custom entities, which
playback marks. Their sprites were once drawn as plain sprites in their render mode, which for
such an entity is its beam type, so `ls_9e_run` showed dark green squares where lightning should
be. They are now drawn as the client does (`R_BeamDrawCustomEntity`):

| Part | How |
|---|---|
| Ends | from the entity's origin, or the entity its sequence's low 12 bits name, to the point its angles hold, or the entity its skin names, as the beam type in its render mode's low four bits says. A beam's angles are not turned between snapshots, being a point |
| Look | its sprite at its frame, its scale its width either side, its render colour by its render amount, added, scrolling at its animation time's rate |
| Noise | with a body, its amplitude a hundredth of it, a point every 4 units, or 16 under half an amplitude, no closer than its width allows and 64 at most, pushed aside by fractal noise made again each frame, across the view, or by a sine wave round the beam with the sine flag (`R_DrawSegs`). Its far end moves aside too, as the engine's does |

An entity's glow with `kRenderFxNoDissipation` (14) keeps its brightness and size at any distance,
as the temporary entities' steady glows already did.

Not yet: beams between entities as messages (`TE_BEAMENTS`, `TE_BEAMENTPOINT`), rings, disks and
cylinders, the noise of the beams messages make, and the start and end attachments of entity
beams.

### Gauss and Egon

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

Not yet: the beams' noise, the Egon's for other players, and the lights of the gauss and Egon.

## Seeking with effects

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

## AG's countdown

**Implemented**, from OpenAG's `CHudCountdown`, on 2026-10-20, not yet compared with the game. The
rest of the HUD is not drawn.

AG's server counts down before a match with the user message `Countdown`: a signed byte of seconds
left, a byte saying whether to speak it, and the two players of a duel, or while waiting the last
round's winner.

| Part | How |
|---|---|
| Sound | from ten down, a number spoken as it comes, `fvox/ten.wav` to `fvox/one.wav` and `barney/ba_bring.wav` at nought, where the listener is at full volume (`pfnPlaySoundByName`) |
| Shown | for five seconds from the last spoken number. At fifty, "Waiting for players to get ready..." or "The last round was won by" the winner, until the next message. Below nought, nothing |
| Drawn | over the view in HL's default HUD colour, the line "The match is about to start." or "<first> vs. <second>" seven text lines from the top and the number three lines below, sized as on a 640 by 480 screen. AG's colour codes, `^` and a digit, are left out of the names. The text is the game's own, in English |

It is part of playback's state, so a seek or a checkpoint restores it as everything else. The
viewer's captures read the scene alone, without it. In `20210424_012456_crossfire.dem` the server
counts from five at 0.83 s, a second apart, to nought at 5.83 s, and hides it at 6.83 s. Each
number is heard, centred, and the line and number show.

## Performance

The targets set before the demo player was built, measured on 2026-10-19 on the development machine,
an AMD Ryzen 5 3600X with 32 GB and an NVIDIA GeForce RTX 3070 drawn through Vulkan, Windows 10,
release build:

| Target | Result |
|---|---|
| 60 FPS sustained at 1080p | met, 139 to 324 fps on average, the 99th percentile 11.8 ms at most, the slowest frame 18 ms |
| Warm indexed seeks within 200 ms at p95 | met in all 8 demos, with the game's files 115 ms at most |
| Input feedback within 100 ms | the playhead moves on the frame of the input, and an indexed drag shows a picture every 68 ms at most ("Scrubbing") |
| Sound and picture within 40 ms | 4 to 5 ms ("Sound and picture in step") |

`scripts/viewer-perf.ps1` runs the viewer's bench, with the view drawn 1920 by 1080 pixels whatever
room the window gives it (`--viewer-size`) and vsync off, playing 10 s after a second to settle. It
samples the process every 100 ms for its private bytes and the GPU's count of its dedicated memory.
The bench says how long after opening the demo the first picture came, every model it wanted was
in, and the checkpoints covered the demo, the worker indexing while it plays.

| Demo, from | Models drawn | Mean, p50, p95, p99, max | First picture, every model in | Checkpoints for the demo | Peak private bytes, working set |
|---|---|---|---|---|---|
| `stalkx_naz_248.dem`, 20 s | 60 | 3.4, 3.3, 4.8, 5.7, 7.7 ms (292 fps) | 227, 233 ms | 4.8 s | 784, 580 MiB |
| `20230909_224143_stalkyard.dem`, 32 players, 100 s | 159 | 7.2, 6.8, 10.2, 11.8, 18.0 ms (139 fps) | 895, 916 ms | 7.9 s | 889, 619 MiB |
| `naz_crossfire_pb_01.dem`, 8 s | 19 | 4.4, 4.3, 5.9, 7.1, 14.4 ms (229 fps) | 189, 195 ms | 6.4 s | 818, 602 MiB |
| `run_92.dem`, singleplayer, 15 s | 11 | 3.4, 3.3, 4.5, 5.1, 8.5 ms (296 fps) | 327, 344 ms | 0.6 s | 493, 320 MiB |
| `20220319_194739_fastrun.dem`, 415 MB, 3000 s | 0 | 3.1, 3.0, 4.0, 4.6, 6.3 ms (324 fps) | 5.4, 5.4 s | 9.2 s | 785, 542 MiB |

The first picture comes after a seek from the start to where the bench begins, so fastrun's is a
cold seek through 3,000 s. The models drawn are those of the last frame timed. Timing begins after
a second of playback from wherever the bench starts, which an earlier version counted from the
demo's start, so it timed from the first frame when starting later. Measured again with the
second to settle, the figures moved by a few percent. Frame times vary by a millisecond or so between runs. The GPU's count of
dedicated memory is 248 MiB in every run, the driver's blocks rather than what is in them. The
scene's own buffers and textures are 19 to 34 MiB.

### Seeks

Indexed seeks, from checkpoints covering the demo as the worker builds them, to 60 moments at
random with `examples/seek_probe.rs`, and to 100 with `examples/effects_seek.rs` in
`lambda-slicer-audio`, which plays with the game's files as the worker does: bullets and temporary
entities traced through the map, models' events, sounds' lengths. Every seek agreed with sequential
playback. Memory is the growth in private bytes while building the checkpoints, next to the
store's own estimate, which the budget of 256 MiB holds.

| Demo | Length | Checkpoints | Memory, measured and estimated | Seek with none, to 3/4 | Indexed p50, p95 | With the game's files, p50, p95 |
|---|---|---|---|---|---|---|
| `stalkx_naz_248.dem` | 1,216 s | 152, 8 s apart | 357, 163 MiB | 1.6 s | 6.4, 13.7 ms | 11.7, 23.3 ms |
| `20210418_004941_vengeance.dem` | 2,506 s | 157, 16 s apart | 379, 229 MiB | 5.4 s | 24.8, 51.7 ms | 18.0, 51.9 ms |
| `naz_crossfire_pb_01.dem` | 1,206 s | 151, 8 s apart | 373, 169 MiB | 2.3 s | 10.6, 19.9 ms | 14.3, 35.0 ms |
| `dblcross_salchi_naz.dem` | 1,314 s | 166, 8 s apart | 335, 206 MiB | 3.0 s | 7.9, 34.1 ms | 9.8, 35.1 ms |
| `run_92.dem`, singleplayer | 104 s | 54, 2 s apart | 64, 46 MiB | 0.3 s | 3.8, 12.4 ms | 2.4, 11.1 ms |
| `ray_vs_alex_cross.dem`, spectator | 103 s | 51, 2 s apart | 31, 59 MiB | 0.1 s | 0.5, 4.1 ms | 0.4, 4.2 ms |
| `20230909_224143_stalkyard.dem`, 32 players | 937 s | 59, 16 s apart | 282, 193 MiB | 2.7 s | 28.9, 54.4 ms | 53.0, 114.8 ms |
| `20220319_194739_fastrun.dem`, 415 MB | 6,079 s | 191, 32 s apart | 333, 184 MiB | 5.8 s | 17.0, 41.6 ms | 18.6, 40.3 ms |

Found:

- Converting the scene's entities took longer than decoding them. Each packet rebuilt two hash maps
  holding every entity and copied each entity twice, and read each changed entity's 33 fields one
  by one by binary search, for about 8 million entities in a probe of stalkyard, 6 million of them
  changed. Now the entities of the packet before, in number order beside the states they came from,
  say which are unchanged, and a changed state's fields are read in one pass. The scene's share of
  that probe fell from 6.7 s to 1.9 s.
- Stalkyard first missed the seek target, at 231 ms at the median and 471 ms at the 95th percentile
  with the game's files. Its checkpoints weighed about 9 MiB each, so the budget kept them 64 s
  apart, and replaying a second of it takes about 7 ms, half of it moving some 400 temporary
  entities. Each entity state the decoder keeps had room for every field of its schema, where a
  player's state holds about 25 of them, a third. Each now has room for what it holds, so a
  checkpoint of stalkyard keeps 3.2 MiB of the decoder's snapshots where it kept 8.2 MiB, the
  budget keeps its checkpoints 16 s apart, and its seeks take 53 ms at the median and 115 ms at the
  95th percentile. The other demos keep checkpoints twice as close as before for the same budget.
- Together, indexed seeks without the game's files went from 23 to 6 ms at the median in stalkx,
  from 23 to 11 ms in crossfire and from 51 to 25 ms in vengeance, against the table in
  "Checkpoints" (measured before).

### Sound underruns

`--viewer-play` now says how many times the sound device found no block ready while the sound it
was playing went on, each heard as a gap. A new feed's first block after a seek is not counted.
Played for 20 s with the sound tap set, so the device plays silence at the same pace: none in
stalkyard from 100 s, stalkx from 20 s or fastrun from 3,000 s, two runs each. A mixer slowed on
purpose to 15 ms for each 10 ms block gave 192 in 5 s.

## Linux

Checked on 2026-10-19 under WSL 1 on the development machine, Ubuntu 26.04.1 with Rust 1.98.1. WSL
1 has no display, no sound device and no GPU, so Mesa's llvmpipe draws through software Vulkan.
The game's install and the demos were read from the Windows drive through `/mnt/f`, where names
match case-sensitively, as they would on a Linux install.

| Check | Result |
|---|---|
| `scripts\check.ps1 -Linux`, clippy with warnings as errors, every test and a release build | passed, the same tests as on Windows and one more, that two game files whose names differ only in case are ambiguous, which only Linux can have |
| `examples/seek_probe.rs` on stalkx, run_92 and crossfire | indexed seeks 5.2, 3.1 and 10.3 ms at the median, none differing from sequential playback |
| `examples/effects_seek.rs` with the game's files, the same demos | 11.1, 1.9 and 14.4 ms at the median, none differing |
| `examples/fixture_equivalence.rs` with the manifest's paths moved to `/mnt/f` | every moment reached every way agreed, 112 comparisons of state, frame and mixed sound, and every slice exported |
| `examples/render_bench.rs` on stalkx with a player model | drawn at 1920 by 1080 in 103 ms a frame on the CPU, every texture found. Against the same frame from the RTX 3070 on Windows, 7 pixels differ by more than 2 grey levels of 255, and the mean difference is 0.2 |
| The viewer's window under Xvfb (`xvfb-run`), its bench on stalkx from 20 s at 1920 by 1080 | the window opens on X11 and plays, 40 models drawn as on Windows, the first picture after 589 ms, at 13 fps, which is llvmpipe's speed on the CPU and says nothing of a GPU's |
| The viewer's captures of `ls_9e_run.dem` every 0.5 s at 640 by 480 with the game's gamma, against the same on Windows, both with default settings | the 33 pairs differ by 0.22 grey levels on average. A few pixels a picture differ by more than 16, along the edges of alpha-tested textures far off, where the two rasterizers choose texels differently |
| `--viewer-play` with no sound device | ALSA finds no device, the viewer says so and plays on without sound |

Xvfb and `libxkbcommon-x11-0`, which winit needs on X11, were installed in WSL for this.

Not tried, so not claimed for Linux:

- A Linux desktop, Wayland or X11 with a GPU: the viewer's speed there, and its window's
  behaviour beyond opening, playing and closing.
- Sound through ALSA on a device. The mix itself is checked above.
- Whether a moment's state is the same on both systems, as each is compared only with itself.

These wait for a Linux desktop, as the rest of the app's window does ([release](release.md),
"Linux").

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

None stopped on a decode error. The decoder alone takes 1.1 s of stalkx's 3.8 s. Going back starts
again from the first group, so a backward seek to near the end of stalkx costs nearly its whole 3.8 s. Checkpoints
(8e) are what bring that down.

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

# Viewer validation

How the viewer is compared with the game (implementation plan §13.9), and what the comparisons
found. The viewer's rules are listed in the [viewer profile](viewer-profile.md), each marked
**Implemented** until a comparison here confirms it.

## Status

The game's frames were recorded on 2026-10-19 and compared with the viewer's (below). What matches
is confirmed for those three clips. What differs is listed under "Found" and not yet fixed.

## Comparison clips (9e)

Three short clips cut with `lambda-slicer slice`, so both the game and the viewer play the very same
files. A clip's clock starts at 0 at its cut, so 2.2 s into `ls_9e_stalkx.dem` is 22.2 s into its
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
`ls_9e_run`. The game's `gamma`, `brightness` and `lightgamma` at the time are noted with the
frames, as the viewer does not yet apply the game's gamma tables.

## Capturing the viewer's frames

```
lambda-slicer-gui F:\SteamLibrary\steamapps\common\Half-Life\ag\ls_9e_stalkx.dem --game F:\SteamLibrary\steamapps\common\Half-Life --viewer-capture 0,0.1,0.2 FOLDER --viewer-size 1280x720
```

saves the view at each time as `FOLDER/<seconds>.png`, waiting at each until the frame, its map
and every model it draws are in. The game's frames are matched to the viewer's by finding, for
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
| Viewmodel's place | lower and smaller | higher, larger and further right | not known. The game may draw it at another field of view, or with the bob and lag the viewer leaves out |
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

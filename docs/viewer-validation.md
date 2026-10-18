# Viewer validation

How the viewer is compared with the game (implementation plan §13.9), and what the comparisons
found. The viewer's rules are listed in the [viewer profile](viewer-profile.md), each marked
**Implemented** until a comparison here confirms it.

## Status

The in-game recording was put off on 2026-10-18, so no rule of the viewer is confirmed against the
game yet. The clips below are in the surveyed install's `ag` folder for when it is done.

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

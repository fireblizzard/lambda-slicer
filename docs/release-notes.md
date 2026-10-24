# Release notes

Downloads are on the [Releases page](https://github.com/fireblizzard/lambda-slicer/releases). What
doesn't work yet is in [known limitations](limitations.md).

## Unreleased

- Windows starts with DirectX 12 by default. This avoids a startup crash observed in Intel's
  Vulkan driver on a machine with an NVIDIA GPU. Setting `WGPU_BACKEND`
  still overrides the backend; Linux keeps its existing defaults.

## 0.2.0, 2026-10-23

### The demo player's HUD

The demo player now shows most of the game's HUD, and Settings can hide most parts of it:

- Health, armour and ammo, item and weapon pickups, and where damage comes from and what kind it is.
- The weapon menu when the player changes weapons.
- The crosshair.
- Chat, which also shows in the event list.
- AG's mini scoreboard at the top left, and the server's settings at the top right.
- AG's match messages, such as votes, timeouts, pauses and sudden death, which also show in the
  event list.
- Text the server puts on the screen, such as KZ and bhop timers.

### Watching demos

- Full screen, with the button at the right of the controls, F or F11, or a double-click on the
  picture. The controls show over the picture while paused or when the mouse moves.
- Explosions look much closer to the game's.
- Ladders no longer cast a wrong shadow, and water no longer shows faces that should be hidden.
- Maps whose files have a small fault the game ignores, such as `homeworld`, now load.
- An open demo is analysed again when the game's files change, so wallgauss kills show once the
  Half-Life folder is set.

### Library

- Players and Teams columns and filters, counted when the match starts.
- Filters at start: a minimum length and number of kills applied every time the app opens.
- Demos with several maps leave out maps with no kills that lasted under 5 minutes. A setting
  turns this off.
- Open demo and Add folder pulse until you've added your first demos.
- A demo opened from the file explorer shows its length.
- Singleplayer demos from the old WON version (protocol 47) now open.

### Kills

- Kill speeds are more accurate. Repeated positions and the push of the killing hit no longer give
  speeds that are far too high. The speed badge now shows from 800 ups.
- Wallgauss checks now account for doors that have turned, doors in the way and crouching victims.
- Scrolling the event list by hand stops it from following the demo, even during fast playback.

### Clips

- Exported clips are named after the demo and the clip by default, such as
  `mymatch__Clip_1.dem`. Settings can name them after their times instead.

### Downloads

- Each new version is published on the Releases page with its Windows zip.
- The zip from the automatic builds is no longer packed inside a second zip.

## 0.1.0, 2026-10-22

The first release.

### Library

- Add folders of demos and they are analysed in the background. Results are kept, so later starts
  are quick. Rescan picks up new demos.
- Columns for the date, map, length, score, kills, gauss shots, the recorder's headshots and
  accuracy, intensity and time spent spectating, chosen with a right-click on a heading.
- Filters for file name, map, player, weapon, headshot kills, midair kills, accuracy, length,
  kills, date and point of view, and a default sort order.
- Demos with several maps show every map, and the score of the map played longest.
- Right-click a demo to show it in its folder.

### The demo view

- The demo plays with the game's maps, models, sprites and sounds, read from your Half-Life
  folder. Effects, decals, beams and the gauss and Egon are drawn as the game draws them.
- A seek bar, 10 seconds back and forward, frame by frame, rewind and speeds from x0.25 to x4.
- Zoom, players shown through walls, the game's gamma settings and field of view, and a limit on
  how many decals stay.
- The kill feed, the watched player's name, AG's match countdown, and a notice when the server
  changes the map.
- The event list follows the demo as it plays, can be filtered by player, and has a button to stop
  it from scrolling.
- Golden minute and the five most intense moments, the scoreboard at any moment, and a timeline
  with a lane per player.
- Kill filters for midair kills, speed and wallgauss.

### Clips

- Clip an event or one of the most intense moments, 4 seconds before to 1 second after by
  default, or type the times.
- Export writes a demo that plays on its own in the game. Every export is checked against the
  original, the original is never changed, and no file is ever replaced.
- With the export folder inside a mod folder, each clip shows the `playdemo` command that plays it.

### Other

- 14 languages, including right-to-left Arabic. Apart from English they are drafts that native
  speakers haven't checked yet.
- Restore defaults and Clear library buttons in Settings.
- `lambda-slicer.exe`, a command line tool with the same analysis and clipping
  ([command line](cli.md)).

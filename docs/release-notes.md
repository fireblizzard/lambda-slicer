# Release notes

Downloads are on the [Releases page](https://github.com/fireblizzard/lambda-slicer/releases). What
doesn't work yet is in [known limitations](limitations.md).

## v0.4.0, 2026-10-25

### Downloads

- The first start after updating analyses your library again, which takes a while for a big
  collection. Your favourites are looked at first.

### Other players' eyes

- Watch a demo through any player's eyes with the list left of the playback buttons. The
  recorder is first, then the map's players by score. 1 to 9 and 0 pick the top 10, ` goes back
  to the recorder, and Page Up, Page Down or the mouse wheel over the picture go to the previous
  or next player.
- Players who aren't playing at the moment, because they haven't joined yet, left or are
  spectating, are greyed out until they play again.
- While the demo doesn't have the player you follow, you see the recorder's view with a note
  saying so, and the view goes back to the player once they show again.
- Choosing a kill shows it through the killer's eyes, or the victim's. Playing it, with a double
  click or Play event, switches to them as soon as the demo has them, and goes back to the view
  you had once it has played.
- Click a player on the scoreboard to watch through their eyes.

### Watching demos

- Free camera: press C and fly around the map with W, A, S and D, Space to go up, Ctrl to go
  down and Shift to go faster, holding the right mouse button to look around. Zooming with the
  crossbow no longer narrows the free camera's view.
- Player names show over their heads, from the free camera too. N turns them off and on.
- Light up everything shows the map fully lit, to see into dark corners. F now turns it on and
  off, and F11 or a double click on the picture fills the screen.
- X turns seeing players through walls on and off.
- Show position and view angles, in Settings, puts where the picture is seen from on it, to tell
  someone exactly where to look.
- Some see-through sprites now look like they do in the game.
- The recorder's own model sometimes went missing from the free camera, and some players' names
  didn't show. Both are fixed.

### Game files

- Settings can list more Half-Life folders to look for maps, models and sounds in.
- The game's own HD models, add-on content and language content are used as the game's options
  set them, and Settings can change that for the app alone.
- Settings no longer asks for the game program, only the Half-Life folder.

### Beside the player

- The scoreboard is always shown above the events. It shows the final scores when the demo
  opens, and once you play or seek, the scores where you are, as they change. Players spectating
  are listed under it. Names are never cut short, and a scoreboard too wide for the panel scrolls
  sideways.
- The triangle on either part gives it the whole side panel, and dragging the line between them
  shares it another way, which is remembered. A double click on the line goes back to giving the
  scoreboard what it needs.
- The event filters fold into one row, the kinds of event are chosen in one menu, and Jump to
  highlight moved above the timeline, leaving more room for the events.
- A player who reconnected showed twice on the scoreboard. Fixed.

### Events

- Map changes and players starting or stopping spectating show under Match.
- Kills with the MP5's grenade and with a hand grenade are told apart.

### Library

- The Matches tab lists each match of every demo, and long stretches of free play, with whether
  the match was complete, joined late, interrupted or cut off by the recording's end. A filter
  picks which of those show.
- Favourites: right-click a demo, or use the star beside an open demo's Details, and see them
  alone with Favourites above the table.
- Hide demos you don't want to see from the right-click menu. Hidden lists them again.
- Demos with several maps show one score for each map in the Score column.
- Settings can choose whether the library opens on Demos or Matches.
- Watching a demo while the library is scanning no longer stutters.

## v0.3.0, 2026-10-24

### Downloads

- Linux x86-64 now has its own download on the Releases page: extract it and run
  `./lambda-slicer-gui`, or run `python3 install.py` to add it to your application menu. It needs
  Ubuntu 24.04 or a similarly recent distribution.
- The first start after updating analyses your library again, which takes a while for a big
  collection.

### Watching demos

- Chapter titles and other text from the game's `titles.txt` now show, such as "ANOMALOUS
  MATERIALS" at the start of a singleplayer chapter.
- Windows starts with DirectX 12. This avoids a crash at start on some computers with both Intel
  and NVIDIA graphics.

### Events

- Right-click an event and choose Copy to copy it as one line of text, such as
  `01:46.817 | andriu killed -<[ATK]>-naz with rpg_rocket`.
- Choose several events with Ctrl+click, Shift+click or Ctrl+A. Copy or Ctrl+C copies them all,
  one per line.
- With several events chosen, Clip it makes one clip from the first chosen event to the last,
  with the clip padding before and after.
- Event times leave out the hours, and the minutes too, when the demo is shorter than that.

### Clips

- Gibs and other debris still flying at a clip's start are left out of the clip instead of
  stopping the export.
- Status icons some servers show on the HUD are carried over a clip's start.
- When a clip can't be made because of something the app doesn't support yet, the message says
  so plainly, such as "The StatusIcon message isn't supported in clips yet."

### Library

- Dates are also read from file names like `dm_dust2-20261007-212429.dem`.

### Documents

- The README is rewritten for players, and [known limitations](limitations.md) lists what
  doesn't work yet.

## v0.2.0, 2026-10-23

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

## v0.1.0, 2026-10-22

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

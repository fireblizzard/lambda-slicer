# The app

How Lambda Slicer's window works, in more detail than the [README](../README.md). The demo player's
inner workings are in [how the demo player works](viewer-profile.md).

![The demo view](images/demo-view.png)

## Starting it

`lambda-slicer-gui` takes demos and folders on the command line and opens them for that session.
Other options are for testing, and the full list is at the top of
`crates/lambda-slicer-gui/src/main.rs`:

- `--locale TAG` picks a language for the run, and `--pseudo` lengthens every message to find text
  that doesn't fit ([languages](i18n.md)).
- `--cache FILE` and `--clips FILE` use another library and clips file, and `--game DIR` another
  Half-Life folder.
- `--demo FILE --screenshot OUT.png` opens a demo from the library, waits for its analysis and
  saves the window. `--viewer-at SECONDS`, `--viewer-fullscreen`, `--tab` and `--settings` choose
  what it shows.
- `--demo FILE --export DIR` exports every clip of that demo into `DIR`.

## Library

![The library](images/library.png)

The toolbar has Open demo, Add folder, Rescan, Cancel and Settings. Until the first demo or folder
is added, Open demo and Add folder pulse. Folders are scanned in the background, subfolders
included, and the status bar shows the progress. Its problem list says why a file couldn't be
read. Dropped files are checked by their header, and anything else is named as not a demo.

The table lists the [library](library.md): status, name, recording date, file date, map, length,
final score, players, teams, kills, gauss shots, the recorder's headshots and accuracy, intensity
and the share spent spectating. Columns chooses which show, as does a right-click on a heading, and
the choice is kept. Clicking a heading sorts by it. Rows load a page at a time.

- **Date** is read from the file name when it has one like `20230910_214553`, as automatic
  recordings do, and otherwise from when the file was last changed.
- **Map** lists every map of a demo with several. Maps with no kills that lasted under 5 minutes
  are left out unless Settings says otherwise. The score is the score of the map played longest.
- **Players** and **Teams** are counted when the match starts after its countdown, or at the first
  kill when the match was already on when the recording began. In team games only players on a
  team count. A demo with several matches keeps the most of each, and one with no match the most
  seen at any moment.
- **Intensity** is the demo's highest action score, with a bar against the highest in the library
  ([analysis](analysis-profile.md)).

Filters opens the filter pane: file name and folder, player, map, weapon, point of view (playing
or spectating), minimum length and kills, players and teams, minimum accuracy, headshot kills,
midair kills and recording dates. The `(?)` beside a filter explains how it's worked out.

Right-clicking a demo offers Show in folder and Copy file location.

## Demo view

Clicking a demo analyses it again on its own thread, since shots aren't kept in the library, and
opens it with the library as a narrow list on its left, which can be hidden. Choosing another demo
cancels that analysis. A demo opened with Open demo, dropped on the window or named on the command
line opens at once in the same way and joins the library when the scan stores it, so a missing or
broken library never stops a demo from opening. Library rows open their file from the path bytes
the library keeps, so names that aren't valid Unicode open too.

The header names the demo, its map, date, length and kind, with buttons for the details, Copy file
location, Show in folder and Close. Escape closes the demo.

### The player

Without the Half-Life folder, the player asks for `hl.exe` or the folder, and everything else keeps
working. With it, the demo plays with the game's files, and the HUD shows as the settings choose.
Under the picture are the seek bar and the controls: back to the start, rewind, 10 seconds back,
one frame back, play, one frame forward, 10 seconds forward, the speed, free camera, zoom, see through walls, the
volume, the picture settings and full screen. The keys are in the [README](../README.md#watching-a-demo).

In full screen the app hides its toolbar, status bar and demo list and asks the window to fill the
screen. The seek bar and controls show over the bottom of the picture while paused, under the
pointer, or for 2.5 seconds after it moves. A double-click on the picture enters or leaves full
screen, and its second click undoes the first one's play or pause.

### Timeline

Under the player, the timeline shows the whole demo with a bar for how busy each moment is. Player
lanes opens one lane per player connection, a reused slot being a new lane, with every name the
player used and AG's colour codes left out. The recorder's lane is marked. Actors that couldn't be
told apart share one lane, last.

- **Marks.** Shots are ticks in the weapon's colour, other actions grey ticks, kills triangles in
  the killer's lane and deaths circles in the victim's, and the recorder's weapon switches small
  squares. Gauss charges and Egon beams are bars from start to end. Marks closer than three pixels
  are drawn as one per kind, taller for more.
- **Moving.** The mouse wheel zooms around the pointer, dragging pans, and Fit shows the whole
  demo. The time runs left to right in every language.
- **Selecting.** Hovering a mark shows its time, player and action. Clicking a mark selects its
  event, and clicking anywhere else goes there.

### Events

The Events tab lists every event in order with its time, the players and the weapon. Kills show
badges for headshots, midair kills, speed, wallgauss, suicides and team kills, and hovering a kill
explains how it was worked out.

- **Shown.** Kills, shots, weapon switches, other actions, chat and match events (votes,
  timeouts, pauses, sudden death) can each be shown or hidden. A weapon and a player name narrow
  the list.
- **Kills with.** Midair, speed and wallgauss narrow the kills further. Wallgauss follows each
  gauss shot through the map, so it needs the game's files and checks the kills in the background
  ([analysis](analysis-profile.md)).
- **Jump to highlight** lists the five most intense minutes, the best one marked Golden minute,
  with their score, time and frags per side. Choosing one goes there, and Clip this intense moment
  clips it.
- **Following.** While the demo plays, the list keeps the current moment in view. Scrolling it by
  hand stops that, and the button at its top right brings it back.
- At the bottom, the previous and next event buttons (Ctrl+Left and Ctrl+Right), Play event, which
  plays the event with the clip padding around it, and Clip it.

### Scoreboard

The Scoreboard tab has one table per team with its total and each player's frags and deaths, two
tables for a duel, and one ranked table for three or more players without teams. It shows the
final scores, as they stood at the last kill, until an event is selected, then the scores at that
moment. Players who left drop out, as on the game's own scoreboard, and a team's `TeamScore`
overrides its players' sum. The last kill decides the result because players often leave between
the end of a game and the end of the recording.

HS counts headshot kills and Acc. the share of shots that hit, for the recording player only, as
other players' often happen out of the recording's view. In a duel the opponent's headshots show
too ([analysis](analysis-profile.md)).

## Clips

A clip is a named range of a demo to export. Clips are kept per demo in `clips.json` beside the
settings. A clips file that isn't valid is renamed to `clips.json.unreadable-<seconds>` and the
clips start empty, and one that can't be read stops saving for the session, so a stored clip is
never written over. Both, and a save that fails, show as a notice in the status bar until
dismissed.

- **Adding.** Clip it on an event adds a range around it, by default 4 seconds before and 1 after.
  Clip this intense moment does the same around a highlight. New clip adds ten seconds from the
  current moment. New clips are named Clip 1, Clip 2 and so on.
- **Editing.** Each clip's card in the Clips tab has its name, start and end. Times are typed as
  `96`, `1:36` or `00:01:36.250`, with a point or a comma before the fraction. Text that isn't a
  time is outlined in red and the clip keeps its old time. I and O, or the buttons beside the
  times, set the start or end to the current moment. On the timeline a clip is a band with its
  name, and dragging either edge moves it.
- **Playing.** A card plays its clip once or loops it, and shows it on the timeline.
- **What the export cuts.** Each card shows the clip's length and the range an export would really
  cut. An export starts at the first frame group at or after the start and ends before the first
  at or after the end, so 01:36 to 04:48 of `stalkx_5o5_asasasdgwgw.dem` cuts 00:01:36.009 to
  00:04:48.000. Ranges an export would refuse say why in orange: an end past the recording, a
  range with nothing in it, a start on the very first frame or while a map loads, or a range with
  a map change inside it. Refusals that need the whole demo read come from the export itself.
- **Parts.** A level or map change, or a reconnect, starts a new part of the demo. A clip can start
  in any part once its map has loaded. Where a part's times repeat an earlier one's, a clip's start
  would land in the earlier part, so the app offers no clip there.

## Export

Export on a card, or Export all clips, writes clips as demos that play on their own, one after
another on a thread of their own ([tested in the game](engine-validation.md)). Clips an export
would refuse are left out and their buttons are off.

- **Where.** Beside the demo, or in the export folder chosen in Settings, as the line under the
  clips says.
- **Names.** By default the demo's name and the clip's, such as `mymatch__Clip_1.dem`. Settings
  can leave out the demo's name, or name the file after its times instead, such as
  `mymatch__01m36s000-04m48s000.dem`. Names keep only ASCII letters, digits, `-`, `_` and `.`,
  since the game's console splits a command at spaces. A name already taken gets `_2`, `_3` and so
  on.
- **Safety.** Nothing is ever replaced. The export writes a temporary file beside the destination,
  checks it, and gives it its final name only if no file took that name meanwhile. The clip says
  Saved only after that. A failure leaves no file and says why in orange.
- **Progress.** The clip under way shows a spinner and the rest wait. Stop exporting skips the
  clips not yet started, and the one under way runs to its end. Closing the demo stops an export
  the same way.
- **Result.** The saved path and how many moments of the clip were checked against the original.
  Inside a mod folder, the nearest folder above with a `liblist.gam`, it also shows the console
  command that plays the clip, such as `playdemo clips/mymatch__Clip_1`, with a Copy button.
  Elsewhere it says to copy the file into the mod folder first.

## Settings

The settings window can be moved anywhere in the app's window, and keeps its place for the next
run. Every change applies at once. A setting that needs explaining has a `(?)` beside it.

- **General.** The Half-Life folder or `hl.exe`, the language, and how many demos are analysed at
  once.
- **Library.** The library folders, leaving out short maps with no kills, where the library is
  kept, Clear library, the order at start and the filters at start.
- **Demo viewer.** The picture (the game's gamma settings), the view (field of view, zoomed field
  of view, the weapon in view), each part of the HUD, seeing players through walls, how many
  decals stay, and the sound.
- **Clips.** The export folder, how files are named, and the clip padding.

Restore defaults lists every setting that differs from its default before putting them back. The
Half-Life folder and the library folders stay as they are.

## Right-to-left languages

In Arabic the whole layout mirrors. The demo list moves to the right, the tabs and lists run right
to left, and lane labels sit right of the lanes. Times and the timeline still run left to right
([right-to-left text](gui-text.md)).

![Arabic](images/demo-ar.png)

## Measured (2026-10-16)

`stlakfa.dem`, a 20 minute 5v5 match with 7997 shots and 503 kills, opens with its
analysis in under 3s on the development machine and gives nine lanes and 9345 events.

# Desktop app

`cargo run --release -p lambda-slicer-gui` starts the app. Demos and folders given on the command
line are opened for that session, `--locale TAG` picks a language for the run, and `--cache FILE`
uses another library cache. `--demo FILE --screenshot OUT.png` opens a cached demo, waits for its
analysis and saves the window, for checking a screen in any language.

![Library and demo view](images/demo.png)

## Library

The left side lists the library cache ([library](library.md)): status, name, recording date, map,
length, final score, gauss shots, kills, best action score and spectating share. Right-clicking any
heading chooses the columns, which are kept in the settings. Gauss shots, the number of primary and
charged gauss shots the recording saw from every player, start hidden. Beside an open demo only
name, date, map, length, score and gauss shots show. Rows load a page
at a time. Open demo, Add folder and dropping files or folders add to the library and scan in the
background. Dropped files are checked by their header, and anything else is named as not a demo.
The status bar shows the scan's progress, and its problem list says in the chosen language why a
file failed.

## Demo view

Selecting a demo analyses it again on its own thread, since shots are not kept in the cache, and
opens it beside a narrower library. Choosing another demo cancels that analysis.

- **Lanes.** One per player connection, a reused slot being a new lane, with every name the player
  used and AG colour codes left out. The recorder's lane is marked. Actors that could not be
  resolved share one lane, last.
- **Marks.** Shots are ticks in the weapon's colour, other recorded actions grey ticks, kills red
  triangles in the killer's lane and deaths red circles in the victim's, and the recorder's weapon
  switches small squares. Gauss charges and egon beams are bars from start to end.
- **Wide zoom.** Marks closer than three pixels are drawn as one per kind, taller for more.
- **Moving.** The mouse wheel zooms around the pointer, dragging pans, double-click or Fit shows
  the whole demo. The time axis runs left to right in every language.
- **Selecting.** Hovering a mark shows its time, player and action at once. Clicking selects it, draws a
  line through the lanes and scrolls the event table to it. Clicking a table row selects its event
  and brings it into view.
- **Filters.** Shots, other actions, kills and weapon switches can be hidden, and one weapon
  chosen, for both the lanes and the table.
- **Action windows.** Best action jumps to the busiest minute by action score
  ([analysis profile](analysis-profile.md)), and the list beside it offers the best five with their
  score, time and frags per side. The chosen window is shaded.
- **Final score.** Shown above the timeline with team or player names.
- **Event table.** Every event in time order with its time, the player's name at that moment, the
  weapon (the death notice's weapon string for kills) and what happened.

In Arabic the library moves to the right, lane labels sit right of the lanes, and the table's
columns run right to left.

![Arabic](images/demo-ar.png)

## Measured (2026-10-16)

`stlakfa.dem`, a 20 minute 5 against 5 match with 7,997 shots and 503 kills, opens with its
analysis in under 3 s on the development machine and gives nine lanes and 9,345 events.

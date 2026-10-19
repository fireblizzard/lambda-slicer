Lambda Slicer
=============

Finds weapon use in Half-Life: Adrenaline Gamer demos, cuts clips from them
that play on their own in the game, and plays demos with their sound. Nothing
is installed: keep this folder anywhere and run lambda-slicer-gui.exe.

Getting started
---------------

1. Run lambda-slicer-gui.exe. Add folder adds a folder of demos to the library,
   which analyses them in the background. Open demo adds single files, and
   files or folders can be dropped on the window.
2. Select a demo to see its scores, events and, with Show timeline, a lane
   per player. Golden minute jumps to the busiest minute.
3. Select an event or one of the most intense moments and add a clip around
   it, or type a start and end such as 1:36 and 4:48. Each clip shows the
   range an export would really cut.
4. Export writes the clip as a new demo. Originals are never changed and no
   file is ever replaced.
5. In Settings, set the export folder to a folder inside your mod folder (the
   one with liblist.gam, such as Half-Life\ag). Each exported clip then shows
   the console command that plays it, for example
   playdemo clips/mymatch__01m36s000-04m48s000

Settings and clips are kept in %APPDATA%\lambda-slicer, and the library cache
in %LOCALAPPDATA%\lambda-slicer. The interface is in 14 languages, chosen in
Settings.

Watching demos
--------------

In Settings, choose the Half-Life folder, the one that holds valve. The
selected demo then plays above its events, with the game's maps, models and
sounds. Click the view or press Space to play or pause, J to rewind, comma and
period to step one frame, and the arrows to jump five seconds. Choosing an
event or a clip moves the view there. Settings also hold the game's gamma
settings, the field of view and the weapon in view.

The view shows only what the demo holds: other players while the server sent
them to the recorder. It does not draw the HUD or play recorded voice, and some
of the game's effects, such as dynamic lights and particles, are not drawn yet.
Files the installation lacks are listed under the view. The release notes list
the rest (docs/release-notes.md in the source).

Command line
------------

lambda-slicer.exe is the same core for scripts and bug reports, for example

    lambda-slicer.exe analyze match.dem
    lambda-slicer.exe slice match.dem --start 1:36 --end 4:48 --output clip.dem
    lambda-slicer.exe scan D:\demos

lambda-slicer.exe help lists every command.

Limits
------

Some ranges are refused with the reason shown, for example a start while a
map is still loading, a range with a map change inside it, or gibs and
projectiles still flying at the start. Other players' actions appear only
while the server sends them to the recording player. The translations are
machine-assisted drafts awaiting review by native speakers.

Licenses
--------

Lambda Slicer is licensed under MIT OR Apache-2.0 (LICENSE-MIT,
LICENSE-APACHE). THIRD-PARTY-NOTICES.txt lists the libraries it includes and
their licenses. VERSION.txt names the build, for bug reports.

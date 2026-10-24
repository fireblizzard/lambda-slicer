Lambda Slicer
=============

A library and player for your Half-Life and Adrenaline Gamer demos. It
finds the best moments in them and cuts clips that play on their own in the
game.

Nothing has to be installed. Just keep this folder anywhere and run lambda-slicer-gui.exe.


Getting started
---------------

1. Press Add folder and pick a folder with demos. Lambda Slicer looks through
   it in the background, subfolders included. Open demo opens single demos,
   and you can also drop demos or folders on the window.

2. To watch demos, Lambda Slicer needs to know where Half-Life is
   installed. It asks the first time you open a demo: point it to hl.exe or
   to the Half-Life folder. You can change it later in Settings. It reads
   the game's maps, models and sounds from there and never starts the game.

3. Click a demo to open it. It plays beside its list of events. Click an
   event to go to it, or open Jump to highlight and pick Golden minute, the
   most intense minute of the match.

4. Press Clip it on an event to make a clip from 4 seconds before it to 1
   second after. Then press Export to save it as a new demo.


Playing clips in the game
-------------------------

In Settings, set the export folder to a folder inside your mod folder, such
as Half-Life\ag\clips. Each exported clip then shows the console command
that plays it, for example

    playdemo clips/mymatch__Clip_1

with a button to copy it. Lambda Slicer never changes your original demos and
never replaces a file that's already there.


Keys in the demo player
-----------------------

    Space, L or click   Play or pause
    J                   Rewind
    Left / Right        5 seconds back / forward
    Shift+Left / Right  10 seconds back / forward
    Comma / Period      One frame back / forward
    Z                   Zoom in or out
    F, F11              Full screen (or double-click the picture)
    Ctrl+Left / Right   Previous / next event
    I / O               Set the clip's start / end to now
    Escape              Leave full screen, or close the demo


Which demos work
----------------

- Half-Life and AG demos from the Steam version: playing, spectating and
  singleplayer.
- Singleplayer demos from the old WON version. Its multiplayer demos aren't
  tested yet.
- HLTV demos are listed and analysed, but clips can't be cut from them.
- Other mods, such as Counter-Strike or Opposing Force, aren't supported.


Good to know
------------

- A demo only holds what the server sent to the player who recorded it.
  Other players usually show up only while the recorder could see them.
- Some clips can't be cut, for example across a map change or while a map
  is loading. The app tells you why.
- The demo player doesn't show everything yet. The flashlight, dynamic
  lights, particles and voice chat are among what's missing.
- The app is in 14 languages, chosen in Settings. Apart from English, they
  are drafts that native speakers haven't checked yet.
- Settings and clips are kept in %APPDATA%\lambda-slicer, and the library in
  %LOCALAPPDATA%\lambda-slicer.


Command line
------------

lambda-slicer.exe does the same analysis and clipping from a command prompt,
for scripts and bug reports. For example

    lambda-slicer.exe analyze match.dem
    lambda-slicer.exe slice match.dem --start 1:36 --end 4:48 --output clip.dem
    lambda-slicer.exe scan D:\demos

lambda-slicer.exe help lists every command.


More
----

Source code, release notes and bug reports:
https://github.com/fireblizzard/lambda-slicer

Lambda Slicer is licensed under MIT or Apache-2.0, at your option
(LICENSE-MIT, LICENSE-APACHE). THIRD-PARTY-NOTICES.txt lists the libraries it
includes and their licenses. VERSION.txt names this build, which helps with
bug reports.

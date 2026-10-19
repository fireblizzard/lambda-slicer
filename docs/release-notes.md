# Release notes

## 0.1.0, not yet released

The first release. Lambda Slicer finds weapon use in Half-Life: Adrenaline Gamer demos, cuts
clips from them that play on their own in the game, and plays demos in a view of its own, with
their sound, from the game's files. What has to happen before the release is in
[release](release.md), "Before calling it release-ready".

### Analysis and clips

GoldSrc demos of protocol 5/48 are read: AG multiplayer recordings, singleplayer runs across level
changes and spectator recordings are analysed and cut. HLTV recordings are recognised and labelled
but not cut, and other protocols and mods are refused with the reason. Every export is checked
against its source before it is written, and no file is ever replaced. The details are in the
[support matrix](release.md#support-matrix).

### The demo view

With the game's folder chosen in Settings, the demo view plays the selected demo with the game's
maps, models, sprites and sounds. It seeks anywhere, rewinds, steps one recorded frame at a time,
plays at a quarter to four times the speed, plays events and clips and loops them, shows players
through walls when asked, and draws with the game's gamma settings.

**Recordings it is made for.** In each, a seek reaches the same state as playing up to the moment,
and at the fixture cases' moments also the same picture and sound, however the moment is reached
([viewer validation](viewer-validation.md), "Equivalence"):

| Recording | Status | Evidence |
|---|---|---|
| AG multiplayer, first person | supported | its frames compared with the game's (`ls_9e_stalkx`), demos of 2 to 32 players |
| A spectator through a player's eyes | supported | its frames compared with the game's (`ls_9e_cross`) |
| Singleplayer, across level changes | supported | its frames compared with the game's (`ls_9e_run`), a run of five maps |
| HLTV | not tried in the view | |
| Other protocols and mods | refused, as for analysis | |

**Speed** on the development machine (Windows 10, RTX 3070): 139 to 324 pictures a second at 1920
by 1080, seeks within 115 ms in 95 of 100 even in a 32-player game, sound within 5 ms of the
picture ([viewer profile](viewer-profile.md), "Performance"). On Linux the view was checked in WSL,
its window under Xvfb, and a Linux desktop with a GPU and a sound device has not been tried
([viewer profile](viewer-profile.md), "Linux").

**Missing files.** The view reads the game from the folder chosen in Settings, and plays nothing
without one. A file the demo names that the installation lacks is not replaced by another:

- A map that cannot be read is named in place of the view, with the reason.
- Models, sprites, skies and sounds that cannot be read are listed under the view, folded away
  under their count, each with the reason. A missing texture shows as a checker and a missing
  sound is silent.
- Decals a server brought with it, as some servers do, are counted missing when the installed
  `decals.wad` lacks them.

**What a demo does not hold**, so the view cannot show it:

- Other players are in the demo only while the server sends them to the recorder, usually while
  they could be seen from where it is, and their shots and effects only as the server passed them
  on. Showing players through walls shows only those the demo holds, and nothing from servers that
  hide players the recorder cannot see.

**Not shown or played yet**, though the demo holds it:

- The HUD, so no scores, health, messages, vote notices or crosshair.
- Recorded voice, and sentences (a monster's or the announcer's speech made of words). Rooms'
  echoes.
- Dynamic lights, such as muzzle flashes and the gauss's and Egon's light.
- Particles: sparks' streaks, explosions' flicker, impacts' dust.
- Beams messages draw between entities, rings, disks and cylinders, and the noise of beams from
  messages.
- Other players' Egon beams, the tripmine's and snark's events, crossbow bolts, player sprays and
  the view's punch.
- Shots are traced against players' boxes, not their models' hitboxes, so a decal or impact can
  land where the game's would not.
- On models, the glow shell and other render effects, remapped textures, blending between
  sequences, mouth movement and the sky's light. On the map, randomly tiled textures and
  scrolling conveyors.
- The weapon in view does not bob or lag, and sits higher and larger than in the game.

**Known differences from the game, not yet fixed** ([viewer validation](viewer-validation.md),
"Found" and "Reference captures"): gauss balls and glows lingering seconds after a shot, a breaking crate's wooden gibs
larger and nearer than the game's, and in one spectator recording some players drawn flat blue
and green where the game drew them dark, the game having failed to load the models they named.

**Not yet compared with the game:** the sound, which needs a recording of the game's, and the
clips of each fixture case, which need the game's frames ([viewer validation](viewer-validation.md),
"Reference captures").

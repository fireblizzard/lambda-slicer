# Known limitations

What Lambda Slicer can't do yet. Most of the demo player's stuff are things still to add.
The [TODO list](../TODO.md) has what's planned.

## Demos

- They only have what the recorder's game received from the server, so it **lacks events that
  happened too far away**. I wish I could make it so that you can switch to any other player's POV,
  but most of the time it's not possible because of that. So other players' shots, kills and
  movement elsewhere on the map aren't in the demo, and the counts of their shots, headshots and
  accuracy would be too low. That's why headshots and accuracy are only shown for the recording
  player, and for the opponent in a duel.
- Some servers use plugins to combat wallhackers, meaning they send even less info about others,
  only when they're about to show up in your view from behind a wall. In that case the feature to
  **see players through walls** has barely anything to show. Not a bad thing if you ask me.
- **HLTV** demos have no recording player, so their events follow whoever HLTV was watching. They
  are listed and analysed, but clips can't be cut from them, and playing them isn't tested.
- **WON multiplayer** demos (the old protocol 47) should work like Steam ones, but haven't been
  tested.
- **Other mods** such as Counter-Strike, Opposing Force or Blue Shift, aren't supported yet.

## Clips

Some ranges can't be clipped:

- A start on the very first moment of a demo, or while a map is still loading.
- A range with a map change or a reconnect inside it.
- A start while something the clip can't carry over is still going on, such as gibs or
  projectiles in the air.
- A range where the game threw away data that arrived late, or where the demo relies on data that
  had to be guessed.

Other things:

- Bullet holes, blood and sprays made before a clip's start aren't in the clip.
- An event the game scheduled just before the start, to play just after it,
  is lost. It's usually a sound or effect a few milliseconds long.
- Each frame of a demo carries a little data from the game's client, which
  a clip copies as it is. AG's plays correctly, but another client's might not.
- Stopping an export skips the clips not started yet, but the one under way runs to its end.
- After a map change or reconnect, a demo's clock can start over, so two parts
  share the same times. Clips can't tell them apart, so the app offers no clip in a later part
  whose times repeat an earlier one's.

## Demo player

Not shown or played yet:

- The flashlight and its HUD.
- Chapter titles and other text from `titles.txt`, such as "Saved..." in singleplayer.
- Dynamic lights, such as muzzle flashes and the gauss's and Egon's light.
- Particles: the streaks of sparks, the flicker of explosions and the dust of bullet impacts.
- Recorded voice chat, sentences (speech made of words, like the HEV suit's or soldiers') and
  room echoes.
- The tripmine's and snark's own effects and the view kicking up when shooting.
- The weapon's bob and lag as you move. The weapon also sits a little higher and larger than in
  the game.
- Models: the glow shell and other render effects, textures recoloured by name (`remap`),
  smooth blending between animations, and mouth movement.
- Randomly tiled textures and scrolling conveyors.

Known differences from the game:

- Gauss balls and glows can stay in the air a few seconds longer than in the game.
- The sound hasn't been properly compared with a recording of the game's yet, only by ear.

A map that can't be read shows its name and the reason, and missing models, sprites and
sounds are listed under the picture. A missing texture shows as a black and white checker,
and a missing sound is silent.

## The app

- Apart from English and Spanish, the other 12 language translations haven't been checked by native speakers,
  and the translations have been AI generated.
- Arabic text boxes show their text in the order it was typed, not right to left, while you
  edit it.
- It builds on Linux, its tests pass and the demo player was tried in a virtual window, but it hasn't been
  tested in a real Linux desktop, so the picture, sound, file dialogs, dropping files and fonts
  may have problems there.

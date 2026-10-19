# Deferred work

Lower-priority edge cases found in review (2026-10-15, through `ad45a7f`). They are deliberately
left until everything else, M4 included, is finished. Each entry says what goes wrong, where,
and the intended fix.

## Restart export (M3)

### Pending delayed events are discarded

Events before the start are treated as already played (`slice/restart.rs`, the plan pass that
keeps only DemoBuffer and ClientData records of the last group before the start, and
`slice/persistent.rs`, which drops `svc_event`). An event scheduled before the cut with a delay
that ends after it is lost. `naz_crossfire_pb_01.dem` has 550 events delayed by 10 ms, some of
them crossing frame-group boundaries.

**Fix:** keep events still pending at the boundary and re-emit them with their remaining delay,
or refuse that boundary.

### Generated packets have no engine size limit

Bootstrap full updates, the persistent-message record and the HUD record are written without
checking an engine packet limit. Structural validation uses the parser's 4 MiB cap, which is a
safety bound, not an engine limit. The largest measured so far is about 3.3 KB of bootstrap
updates, and real LOADING records exceed 11 KB, but nothing guarantees an upper bound.

**Fix:** establish the profile's real limit for demo playback, then split large bootstrap or
persistent records across several records, or refuse with `PacketOverflow`.

### Unknown DemoBuffer contents are copied

The bootstrap group copies the DemoBuffer record of the last group before the start
(`slice/restart.rs`, plan pass) without knowing its format. The observed AG buffers are 8 bytes
per frame and play correctly, but another client DLL's buffers may carry state that does not
survive being moved.

**Fix:** recognise the supported buffer formats (size and client) and refuse a restart over
unknown ones (`UnknownDemoBufferSemantics`).

### Decals are not carried over the start

Bullet holes, blood and sprays (temp entity kinds 13, 104, 109, 112, 116 to 118) are lasting world
state, but `slice/ongoing.rs` treats them as already applied, so a slice starts with clean walls.
`naz_crossfire_pb_01.dem` has about 600 of them and the vengeance match about 1,600.

**Fix:** replay the most recent decals up to the engine's decal limit, after measuring the
limit in this install, within the packet size limit above.

### Restarts refused for now

Not review findings, but known gaps of the same kind:

- **A range crossing packets the client dropped as too old.** The client's state is frozen during
  such a run, and the export would have to reproduce that.

## Desktop app (M6)

Parts of the plan's GUI section (§8) that the M6 gate does not require and that are left for
later.

- **Settings.** No cache size policy and no diagnostic verbosity. Instead of a game or mod
  location, the export folder setting and the nearest `liblist.gam` above it decide the
  `playdemo` command.
- **Export.** A clip under way cannot be stopped, only the clips after it. The export pane shows
  refusals, progress, the validation count and the final path, but no separate protocol profile
  line, since a demo whose profile is unsupported does not analyse.
- **Clips** in a session whose times repeat an earlier session's cannot be told apart from it,
  since clips and the CLI name a time and not a session, so the app does not offer them there.
- **Timeline lanes** show the same player once per session, the recorder marked only in the
  last.
- **Library.** Files from a folder removed in Settings stay listed until the cache is cleared.
  The cache keeps paths unique by their display text, so two Linux names differing only where
  they are not valid Unicode store only the first. The GUI opens rows and keys clips by the
  lossless path bytes.
- **Arabic text fields** show their text in logical order while it is edited
  ([gui-text](gui-text.md)).
- **Linux fonts.** CJK and Arabic glyphs come from the system fonts eframe finds on Windows.
  Linux font discovery is checked in M7.

## Viewer settings and see-through walls (M11)

Asked for on 2026-10-19 after the comparison with the game ([viewer
validation](viewer-validation.md)), to be done when M11 starts. Both are done (11e and 11f,
[viewer profile](viewer-profile.md), "Viewer settings" and "See-through walls").

### Viewer settings

The viewer takes the Half-Life folder only from `--game` and draws with no gamma at all, while the
game applies its gamma tables to every texture and lightmap. The comparison's game ran with
`gamma`, `brightness` and `lightgamma` at 2.0, so the viewer cannot match its brightness.

**Fix:** a viewer section in the settings window (`gui/src/settings.rs`, saved with the other
settings), every label in all 14 languages:

- Brightness, gamma, light gamma and texture gamma, the game's `brightness`, `gamma`,
  `lightgamma` and `texgamma`, defaulting to the game's own (0, 2.5, 2.5 and 2.0). The renderer
  builds the game's texture and light gamma tables from them and applies them to textures and
  lightmaps. Then the comparison runs again with the recorded values.
- Field of view, as recorded by default, or a fixed value.
- The Half-Life folder, in place of `--game`.
- Drawing the viewmodel, on by default.
- The sound volume, which is now a slider beside the time bar only.

### See-through walls

The server sends every entity in the recorder's potentially visible set, which often holds
players still behind a corner or a wall. Someone studying a demo may want to see them.

**Fix:** a mode, off by default, that draws players, and optionally projectiles and items, again
over the world as silhouettes tinted by team, so they show through walls. It can only show what
the demo holds. Servers that hide players the recorder cannot see, as some anti-wallhack plugins
do, leave nothing to show, which the mode's description and the viewer profile say. It changes
only how the viewer draws a recording.

## The game's sound (M10, M11)

The M10 gate asks for the viewer's sound to be compared with the game's, and M11 left it for a
recording of the game's sound, as `startmovie` saves none. `sound_compare` is ready for it
([viewer validation](viewer-validation.md), "Recording the game's sound").

**To do:** record the game's sound while it plays `ls_9e_stalkx`, run `sound_compare` on it and
write down where the paired sounds start apart and how loud they are, then the README's M10 and
M11 rows.

## The fixture clips in the game (M12)

The reference captures compare the viewer with the game's frames of a clip of each fixture case
([viewer validation](viewer-validation.md), "Reference captures"). The clips are in the game's
`ag` folder and the viewer's frames of them are taken, but the game's are not.

**To do:** record the game's frames of the eight `ls_12d_*` clips with `startmovie`, and their
sound in a second pass, run `scripts/viewer-references.ps1` and `sound_compare`, and write down
what matches and what differs.

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

- **A start in a later session** than the LOADING entry describes, after a map change. Needs
  that session's own signon state in the output.
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
- **Clips** can only be made in a recording's first session, as restarts are (above).
- **Library.** Files from a folder removed in Settings stay listed until the cache is cleared.
- **Arabic text fields** show their text in logical order while it is edited
  ([gui-text](gui-text.md)).
- **Linux fonts.** CJK and Arabic glyphs come from the system fonts eframe finds on Windows.
  Linux font discovery is checked in M7.

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

### Restarts refused for now

Not review findings, but known gaps of the same kind:

- **A start in a later session** than the LOADING entry describes, after a map change. Needs
  that session's own signon state in the output.
- **A range crossing packets the client dropped as too old.** The client's state is frozen during
  such a run, and the export would have to reproduce that.

## Documentation

### README status table is stale

`README.md` still lists M3 as "not started", while S1 restarts pass in the game
(`docs/engine-validation.md`).

**Fix:** update the status table when M3's remaining review findings are fixed.

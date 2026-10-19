# Fixtures

## Local manifest

Original demos stay outside Git. Copy `local-fixtures.example.json` to `local-fixtures.json`
(ignored by Git), adjust paths, and check it with:

```powershell
.\target\release\lambda-slicer.exe fixtures --manifest local-fixtures.json
```

Each fixture may pin `bytes`, `blake3`, and container expectations (`expect`). Only fields present
are checked. Hashes in the example were computed on 2026-10-14 from the developer's files; none of
these fixtures is a known-positive gauss fixture.

## Controlled recordings still needed

Plan §11.1 lists the required controlled recordings (gauss primary/charged/stop-only, two
simultaneous actors, every standard weapon action, singleplayer, slot reuse, name change, and the
slice-state cases). For each recording, note: date, map, mod/client build, participants, input
settings, the exact sequence of actions with approximate times, and what was visible in playback.

## Viewer moments

A fixture may list `moments`, each a case of the viewer's fixture matrix, a time in seconds of
playback and a note, as in `local-fixtures.example.json`. `lambda-slicer fixtures` reads them with
the rest. `examples/fixture_moments.rs` in `lambda-slicer-playback` finds candidates in a demo, and
the matrix is in [viewer validation](viewer-validation.md), "Fixture matrix".

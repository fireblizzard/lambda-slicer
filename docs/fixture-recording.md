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

## Recordings still wanted

Demos recorded on purpose, where it's known exactly what happened, test the analysis best. Still
wanted: gauss primary shots, charged shots and charges stopped without a shot, two players
shooting at once, every standard weapon action, singleplayer, a player slot reused by someone
else, a name change, and the cases clips must carry over their start. For each recording, note
the date, map, mod and client build, who played, their settings, what they did and roughly when,
and what could be seen when playing it back.

## Viewer moments

A fixture may list `moments`, each a case of the viewer's fixture matrix, a time in seconds of
playback and a note, as in `local-fixtures.example.json`. `lambda-slicer fixtures` reads them with
the rest. `examples/fixture_moments.rs` in `lambda-slicer-playback` finds candidates in a demo, and
the matrix is in [viewer validation](viewer-validation.md), "Fixture matrix".

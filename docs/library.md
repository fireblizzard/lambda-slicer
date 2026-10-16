# Library scan and cache

The library is a SQLite cache of per-demo analysis summaries, filled by `scan` and searched by
`query` (implementation plan §7).

## Scan

- **Discovery** walks each root depth first in name order and streams paths into a queue of at
  most 1,024, so results start before the walk ends. Symlinks and junctions are not followed,
  roots inside other roots are dropped, and unreadable directories are reported without stopping
  the walk.
- **Workers** (1 by default, 1 to 4 with `--workers`) take queued paths, with paths passed to
  `ScanControl::prioritize` first. A path whose size, modification time and analysis version
  match its cache entry is a cache hit, read from metadata alone. Otherwise the worker reads the
  file's identity, quick hash and full BLAKE3 hash (`--no-full-hash` skips it), the header, and
  analyses it.
- **Duplicates.** A path with the identity of a file already seen in the scan (volume serial and
  file index on Windows, device and inode elsewhere) is reported as a duplicate and not analysed.
- **One writer.** The calling thread stores each result in its own transaction.
- **Failures stay per file.** A corrupt file is stored with its decode status and error, an
  analysis that panics is stored as `failed`, and a file that cannot be read is reported and not
  stored. A file whose size or modification time changed during analysis is not stored.
- **Cancelling** (Ctrl-C in the CLI, `ScanControl::cancel` in the library) stops discovery and
  every analysis at the next frame. Finished results stay, and a cancelled analysis is never
  stored, so it cannot replace an earlier result.
- **Progress** is reported at most every 100 ms: files discovered, completed, cache hits,
  analysed, bytes read, and the files being analysed.

## Cache

The cache lives in `%LOCALAPPDATA%\lambda-slicer\library.sqlite` on Windows and
`$XDG_CACHE_HOME/lambda-slicer/library.sqlite` (or `~/.cache`) elsewhere. `--cache` picks another
file. It uses WAL so workers read while the writer stores.

| Table | Holds |
|---|---|
| `files` | path as text and in its native bytes, size, modification time, identity, quick and full hashes, analysis version, status, error, header facts, recording kind, recorder, playback and spectating fraction, shot, unknown-shot and kill counts, best action score, coverage JSON |
| `players` | every player identity seen, once per name |
| `weapons` | shots, kills, records and intervals per weapon |
| `kills` | each kill with its weapon string, profile weapon, linked shot weapon, cause, names, airborne flags, horizontal speeds, distance and flight time |
| `windows` | the best action windows with their sides and weapons |

**Invalidation.** Every entry records the cache schema, analysis schema, profile, decode report
schema and parser version it was made with. An entry with any other version is stale. It is
analysed again by the next scan and left out of queries. A cache with a different table layout is
rebuilt. `--force` analyses every file again.

**Not stored.** Weapon occurrences and intervals are not stored, because the library would grow
by one row per shot. A selected file is analysed again on demand. Raw player name bytes are not
stored either, only the decoded names.

A metadata match is a hint that the file is unchanged, not proof. Exports hash the bytes they read
and check them against the source, independently of the cache.

## Query

`query` lists current files matching every condition given, ordered by path:

- `--map TEXT`, `--player TEXT` (any name the player used)
- `--min-spectating F`, `--max-spectating F` (fraction of playback, 0 to 1)
- `--weapon-used NAME` (at least one shot, profile name)
- `--kill-weapon NAME`, `--victim-airborne`, `--killer-airborne`, `--min-speed U`. A file
  matches when one kill meets all kill conditions, and those kills are listed under it. The
  weapon matches the death notice string, the profile weapon or the linked shot's weapon,
  ignoring case
- `--min-action SCORE` (best action window)

`query --windows` lists the best action windows across the library instead.

## Measured (2026-10-16)

On the development machine over `F:\SteamLibrary\steamapps\common\Half-Life\ag` and
`F:\HLScreeniesDemos`:

- **Worker scaling.** 50 demos (2.95 GB) took 47.5 s with 1 worker on a cold read, 43.4 s with 1
  warm, 24.1 s with 2 and 14.3 s with 4. Analysis is CPU bound on this disk, so `--workers 4` is
  the practical choice for a full pass. The default stays at 1 to leave the machine responsive.
- **Full first pass** with 4 workers: 55,059 files, 155.6 GB read (the full hash plus the
  analysis) in 454 s. 55,055 decoded completely. The 4 others are unfinished recordings with a
  zero directory offset, stored as `failed` with that error. No duplicates, unreadable files or
  directories, or links were found. The cache file is 80 MB.
- **Cached rescan** of the same roots: 55,059 cache hits in 2.6 s.
- **Queries** over the full cache take 0.06 s (`--windows`) to 0.5 s (`--player naz`, 54,996
  files). `--min-spectating 0.5` finds 143 files in 0.24 s.
- **Peak memory** of one analysis: 8 MiB for the 396 MiB `20220319_194739_fastrun.dem`, 56 MiB
  for `naz_crossfire_pb_01.dem` and 362 MiB for the 32-slot `20230909_224143_stalkyard.dem`.
  Memory follows one file's collected events, not the library or the file size.

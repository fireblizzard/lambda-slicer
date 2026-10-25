# Handoff

For the next session working on Lambda Slicer. Read this, `CODE_STYLE.md` and `TODO.md` first.
Written 2026-10-25.

## How to work here

- **The user commits everything.** Never run `git commit` or `git push`, and never tag. Leave the
  work in the tree and suggest a short commit message at the end.
- **Code style.** Follow the whole of `CODE_STYLE.md`, not only its comment rules: paragraphs
  with a blank line before a comment that opens a new step, no dense `peek`/`next_if`/`filter`
  chains, `if`/`else` rather than `match` on a bool, comments with no final period and no
  colons or semicolons. Before wrapping up, read every hunk with `git diff` against the guide.
- **Words.** UI text and commit messages use plain words for ordinary players, but the gaming
  community's own terms are fine and preferred: POV (never "through their eyes"), fullbright,
  headshot/HS, multikill, double kill, spree, ups. Spanish uses gamer terms too (headshot, midair,
  la gauss, Shift, doble kill).
- **Release notes** are short, one change per bullet, each starting with a verb (Added, Fixed,
  Tweaked, Moved), keys linked to the README rather than listed.
- **Every new string goes into all 14 catalogs** in `crates/lambda-slicer-gui/src/i18n/ftl/`.
  Hand the translations to a general-purpose subagent allowed only Read, Grep and Edit, telling it
  where to insert each key, which existing keys to copy wording and tone from, and to keep LF
  line endings. The catalog tests check every catalog has every English key with the same
  arguments, and that plural selects cover every category, so a select on number literals such
  as `[2]` fails. Split such strings into separate keys instead.
- **Line endings are LF.** Never use PowerShell `Set-Content` on repo files. Edits through Python
  should write with `newline=""`.
- **Ask before installing packages.**
- **Never touch the user's real settings, library or clips.** Run the app with a scratch
  `APPDATA` and `--cache` and `--clips` pointing into the scratchpad. Reading the user's library
  read-only (`file:...?mode=ro`) has been fine.
- **`TODO.md` belongs to the user.** Don't tick items or edit it.
- Build with `--target-dir` in the scratchpad so the user's `target` stays untouched.

## Commands

From the repo root, with `S` the session's scratchpad:

```sh
cargo fmt --all
cargo clippy -q --workspace --all-targets --target-dir "$S/target" -- -D warnings
cargo test -q --workspace --target-dir "$S/target"
cargo build -q --release -p lambda-slicer-gui --target-dir "$S/target"
```

`scripts/check.ps1` runs what CI runs. Releases are in `docs/release.md`.

## Checking things in the app

The GUI takes options for screenshots and captures (all in `crates/lambda-slicer-gui/src/main.rs`
and `docs/gui.md`). Useful ones:

- `--cache FILE --clips FILE` with `APPDATA` set to a scratch folder holding
  `lambda-slicer/settings.json`. Write that JSON from Python, as shell `echo` breaks backslashes.
  Unset `APPDATA` again before running Python with PIL, or PIL isn't found.
- `--demo FILE` opens a demo already in the library. A demo given as a plain path argument is
  opened directly, which also adds it to the scratch library.
- `--game DIR` (the Half-Life folder), `--viewer-at SECONDS`, `--viewer-follow N`,
  `--viewer-free-at X,Y,Z,PITCH,YAW`, `--viewer-capture TIMES DIR` (pictures only, jumping to each
  time, so muzzle flashes and other effects built up while playing are missing),
  `--event N`, `--refine LIST` (`midair`, `speed`, `wallgauss`, `multikill`...), `--tab`,
  `--settings --section NAME`, `--locale es-419`, `--size WxH`, `--screenshot OUT --frames N`.
- Small playback probes as throwaway examples (`crates/lambda-slicer-playback/examples/zz_*.rs`,
  run with `cargo run --release -p lambda-slicer-playback --example ...`) have been the quickest
  way to check what a demo holds. Delete them afterwards.
- `lambda-slicer analyze DEMO --json OUT` (the CLI) dumps the whole analysis report.

Test demos and the game:

- `F:\HLScreeniesDemos\01jan2022\20210424_003525_endcamp.dem`, the recorder's speeds, dark
  rooms, an explosion lighting the walls at 50.887 s
- `F:\SteamLibrary\steamapps\common\Half-Life\ag\atascao.dem`, several maps, spectators, a rocket
  double kill at 5:03.9, `vengeance` with conveyors from 125.9 s (one in view at 164 s)
- `F:\SteamLibrary\steamapps\common\Half-Life\ag\20260314_215543_agctf_duff.dem`, CTF, several
  maps, a double kill
- `F:\SteamLibrary\steamapps\common\Half-Life\ag\20240211_175030_lost_village2.dem`, a player
  who disconnects at 3:03 and reconnects at 3:44, a 12-kill spree
- The game: `F:\SteamLibrary\steamapps\common\Half-Life`. Its `ag` folder holds 54,000 demos, so
  never point a scratch library at it; copy a few demos into a scratch folder instead.

## Where things stand

Released: v0.4.0 (tag pushed by the user). Committed since: killing spree badges, multikills,
dynamic lights, scrolling textures, the spree default of 10, the speedometer and last jump
speed (estimated with `~` for other players' POVs).

**Not committed yet**, ready to commit, checks passing:

- Library filters and columns for multikills and killing sprees, counting anyone's kills.
  - Counting moved into `crates/lambda-slicer-core/src/analysis/streaks.rs` (new), shared by the
    library and the events list (`KillFeatures::at_once` and `spree` in `timeline.rs`).
  - Cache: `CACHE_SCHEMA` 12, `kills.at_once` and `kills.spree`, `files.best_multikill` and
    `files.best_spree`, `KillQuery::min_at_once`, `FileQuery::min_spree`, `Sort::Multikill` and
    `Sort::Spree`. The schema change rescans the library once.
  - Library: Multikill buttons and a "Spree of at least" field in the filter pane, Multikill and
    Spree columns, and opening a demo with Multikill on turns on the same refinement in its
    events (`app.rs`).
  - Suggested commit message: "Add multikill and killing spree filters to the library".

## Known gaps and loose ends

- The library's Midair and Headshot filters don't carry into an opened demo's events; only
  Multikill does. Offered to the user, not asked for yet.
- Dynamic lights: muzzle-flash lights during play were never checked on screen (captures jump to
  a time, so no flashes). Light sizes don't flicker, and a `TE_ELIGHT` doesn't follow its entity.
- Scrolling textures: the direction hasn't been compared with the game. No unit test, as it
  would need a whole map.
- Other players' speed and jump estimates have no unit test, only probe numbers. Fast stairs or
  being pushed up can count as a jump.
- The scoreboard pane is capped at half the side panel, so a long spectator list scrolls.
- The scratch library used for checks (`mk-cache.sqlite` and the `mkdemos` folder in the old
  session's scratchpad) can be thrown away.

## Ideas offered but not decided

From the last TODO review, quick wins not started: x1.25 and x1.5 speeds with a new speed
control, holding left click to double the speed, mouse 4 and 5 for back and forward, the chat
and kill feed font size setting, and the export stop button reacting within a moment. Also
viewer HUD items: the recorder's fps, the flashlight HUD, and the scoreboard on `+showscores`.

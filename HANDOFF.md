# Handoff: Lambda Slicer TODO work

Written 2026-10-21 for the next agent. The user works through `TODO.md` one item at a time. This file is untracked, so delete it once it has been read.

## The project

- Lambda Slicer is a Rust 2024 workspace for browsing and clipping Half-Life / Adrenaline Gamer (AG) demos. The GUI uses egui/eframe 0.37.
- Crates under `crates/`: `lambda-slicer-gui`, `-core`, `-playback`, `-audio`, `-render`, `-assets` and `-cli`.
- The GUI's main files:
  - `viewer.rs`: the demo player, its transport (the buttons under it), seek bar, zoom and the watched player's name.
  - `kill_feed.rs`: the kill feed.
  - `demo.rs` and `demo/events.rs`: the demo view and the event list.
  - `library.rs`: the demo table.
  - `settings.rs` and `app/settings_window.rs`: settings and the Settings window.
  - `export.rs`: exporting clips.
  - `theme.rs` and `widgets.rs`: colours, icons and shared widgets.
  - `layout.rs`: rows that follow right-to-left languages.
- Interface text lives in Fluent catalogs, `crates/lambda-slicer-gui/src/i18n/ftl/*.ftl`, one per language.

## How the user wants the work done

1. **One TODO item at a time.** For each item, report what was done, its limitations and screenshots, plus a short plain commit message.
2. **The user commits. Never run `git commit`.** Everything up to item 26 is committed (HEAD `143cb69`).
3. **Follow `CODE_STYLE.md` closely, the whole guide and not just the comments:**
   - Code is grouped into short paragraphs, with a blank line before a comment that opens one.
   - Comments have no final period, no colons and no semicolons.
   - Use named constants and plain control flow. Avoid dense `peek`/`next_if`/`filter` chains.
   - Match the doc-comment style of the code around you.
   - Never put new code between a doc comment and its item. This mistake happened several times.
4. **No jargon** in interface text or commit messages. The audience is ordinary players, though gamer terms are fine. Reuse the wording the app already has, for example "current moment" rather than "playhead" in interface text.
5. **Every new interface string goes into all 14 catalogs:** en, ar, de, es-419, fi, fr, it, lt, pl, pt-BR, ru, tr, uk and zh-Hans.
   - Hand the 13 translations to a subagent that uses the Edit tool.
   - Then check its diff with `git diff -U0 -- crates/lambda-slicer-gui/src/i18n/ftl`. The last translator damaged a neighbouring line (`clip-it =Clippen`).
   - Spanish uses community gamer jargon, such as "headshot/HS" and "la gauss".
   - The test `every_catalog_has_the_english_messages_and_arguments` catches missing keys.
6. **Keep LF line endings.**
   - Never rewrite repository files with PowerShell `Get-Content`/`Set-Content`, because it broke every non-ASCII character in en.ftl once.
   - Use the Edit tool, or Python with `newline=''`.
7. **Ask before installing packages.**
8. **Never touch the user's real data.**
   - Don't write the user's real settings file (`%APPDATA%/lambda-slicer/settings.json`) or real clip store.
   - For runs and screenshots, point `APPDATA` at a scratch folder and pass `--cache` and `--clips`.
9. **`TODO.md` belongs to the user.** Leave it alone; they tick items themselves.

## Building, testing and screenshots

The user's own app may be running from `target/release` and locking it, so build into a separate target folder. The previous session used the one below. Its scratch folder sits in the temp directory and may be cleaned up at any time; if it is gone, make your own with the same layout.

```
S = C:\Users\naz\AppData\Local\Temp\claude\C--Users-naz-source-repos-lambda-slicer\b3eaf545-30a9-4599-bcb9-386fda15d98c\scratchpad
cargo build --release -p lambda-slicer-gui --target-dir "$S\target"
cargo clippy -p lambda-slicer-gui --all-targets --target-dir "$S\target" -- -D warnings
cargo test -p lambda-slicer-gui --target-dir "$S\target"
cargo fmt --all
```

A screenshot run, in PowerShell:

```powershell
$env:APPDATA = "$S\appdata-midair"   # scratch settings, game folder already set
$a = @('--cache', "$S\auto-cache.sqlite", '--clips', "$S\clips.json", '--size', '1440x900',
       '--screenshot', "$S\shots\name.png",
       '--demo', "$S\midair-demos\aghldmtv_law_vs_cas_cross.dem", '--viewer-at', '62')
Start-Process -FilePath "$S\target\release\lambda-slicer-gui.exe" -ArgumentList $a -Wait -NoNewWindow
```

- **Command-line options:** the full list is at the top of `crates/lambda-slicer-gui/src/main.rs`. It includes `--window`, `--event`, `--refine`, `--tab`, `--export` and `--frames`.
- **Test demos:** `$S\midair-demos\` holds a few crossfire demos.
  - The demos named in TODO.md (`somecrossgame`, `20240120_192554_ag_hidden_lab` and `20240120_190919_boot_camp`) are in the user's library folders. If `--demo` cannot find one, ask the user where it is.
  - When the cache schema changes, scratch caches empty themselves and the demo has to be scanned again before `--demo` opens it.
- **Cropping:** Pillow is not installed. Crop or enlarge screenshots with System.Drawing in PowerShell.
- **egui tests:** use `ctx.run_ui(input, ...)`, then `output.textures_delta.clear()`. Call `theme::install(&ctx)` first when the code draws text. See the test at the end of `demo/events.rs`, which turns the mouse wheel over the event list.

## What the last session finished

All of it is committed:

- **Item 17:** the play button is centred under the picture, in line with the player's name.
  - The speed menu follows the forward 10 s button.
  - The time sits centred on its own row under the play button.
  - The view controls stay beside the buttons only while the play button can still be centred. Otherwise they drop to the time's row, and the time then sits just before them.
  - Times show milliseconds, with hours only for demos over an hour (`clock()` in `viewer.rs`).
- **Item 26:** the kill feed now has an 8 px right margin (`RIGHT_MARGIN` in `kill_feed.rs`).
- **Item 25:** a crosshair button at the end of the "N events" row keeps the list on the current moment.
  - Scrolling by hand turns it off; this is detected in `events_list`.
  - Clicking it pans straight back. It starts on for every demo opened.

## Order agreed with the user for what is left

1. **23: wrong score on demos with more than one map** (for example `somecrossgame`: crossfire 93-79, then a little datacore 1-0).
   - The library and the scoreboard's starting view should use the map that was played longest, not the last one.
   - The score comes from the core analysis and the cache. See `demo/scores.rs` and the library's score column.
   - Changing what is cached probably needs a cache schema bump. The schema is currently 7, in core `cache.rs`.
2. **24: a player name filter in the event list.**
3. **28: decal limit.**
   - Playback already caps decals at 4096 (`MAX_DECALS` in `lambda-slicer-playback/src/decals.rs`).
   - The user wants 100 most recent by default, with a setting.
   - Decals that come with the map itself probably should not count.
4. **27: a "restore all settings to default" button in Settings.**
   - It needs a confirmation that lists every setting that will change, by its translated name.
5. **19 + 20, done together.**
   - 19: the library should show every map of a multi-map demo (`ag_hidden_lab, elixir`), and the map filter should match any of them.
   - 20: at a map change (intermission), show text saying the server is changing to that map.
6. **16: crosshair in the demo player, with a setting to turn it off.**
7. **21: ladder shadow, and 22: water drawing, both on `20240120_190919_boot_camp`.**
   - For 22, the water should probably only draw the faces of a water volume that are not hidden by others.
8. **18: less lag in the player while folders are being scanned.**
9. **7: the rest of AG's HUD, in pieces.** First the chat with AG colour codes, then votes, then the mini scoreboard.
   - AG colour codes are `^1` to `^8`; `^0` and `^9` reset.
   - Reuse `name_job()` in `kill_feed.rs`, which already applies colour codes.
10. **8: free camera.** WASD, Ctrl down and Space up, detached from the recorder's view.

The ideas for later in TODO.md (favourites, a crosshair editor, a custom sky) are not scheduled.

## Pitfalls already hit

- The PowerShell encoding damage and the doc-comment splitting described above.
- **Fluent catalog test:** it treats hyphenated words in some strings as message ids. A key once had to be renamed because of this (`"file-date"` → `"modified"`).
- **Fonts:** the app's font has no arrow glyphs, so key names like Shift+Left come from translated catalog entries (`key-shift-left`).
- **Long scripts:** a multi-line Python heredoc in the Bash tool failed with a quoting error once. Use the Edit tool for multi-part edits instead.
- **Disk space:** `target/debug` grows large, 19 GB last time. The user may run `cargo clean`, so the first build can take a few minutes.

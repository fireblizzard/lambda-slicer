# Releasing

How a release is built, checked and published, and what each kind of demo is known to support.
What a release brings is in the [release notes](release-notes.md), and what it lacks in [known
limitations](limitations.md).

## Building and checking

`.\scripts\check.ps1` runs what CI runs: formatting, clippy with warnings as errors, every test
and a release build, all `--locked`. `-Linux` repeats clippy, the tests and the release build in
WSL, building into the Linux home.

CI (`.github/workflows/ci.yml`) runs the same on `windows-latest` and `ubuntu-latest`, then
packages on Windows. It keeps the zip and its `.sha256` as build artifacts, each uploaded as it
is, so downloading the artifact gives the package zip itself rather than a zip inside a zip. The
private demo collection and the game stay out of CI. The first run on GitHub, for `00b9dee` on
2026-10-17, passed on both systems in about 13 minutes
([run 39056627064](https://github.com/fireblizzard/lambda-slicer/actions/runs/39056627064)).

The checks on local demos run with

```powershell
$env:LAMBDA_SLICER_FIXTURE_MANIFEST = "$PWD\local-fixtures.json"
cargo test --release -p lambda-slicer-core --test local_corpus -- --ignored
```

and include `exports_leave_every_fixture_unchanged`, which exports from every listed demo and
checks that its bytes didn't change.

## Windows package

`.\scripts\package.ps1` builds the release binaries and writes
`dist\lambda-slicer-<version>-windows-x86_64.zip` and its `.sha256`. The zip holds:

- `lambda-slicer-gui.exe`, which opens no console window in a release build
- `lambda-slicer.exe`, the [command line](cli.md) tool
- `README.txt`, a short guide for players, from `packaging\README.txt`
- `LICENSE-MIT` and `LICENSE-APACHE`
- `VERSION.txt`, with the version, commit and build time
- `THIRD-PARTY-NOTICES.txt`

`scripts\notices.py` writes the notices from `cargo metadata` for the Windows target: every crate
the binaries link (243 for 0.1.0), its license and the license files it publishes, including the
licenses of egui's bundled fonts. Build and development dependencies are left out, since none of
their code ships.

Nothing is installed. The app keeps its settings and clips in `%APPDATA%\lambda-slicer` and its
library in `%LOCALAPPDATA%\lambda-slicer`.

Lambda Slicer is licensed under MIT OR Apache-2.0. Every shipped dependency is under a license
compatible with that, and the corrected plural rules in `vendor` keep their own MIT OR Apache-2.0
license ([dependency audit](dependency-audit.md)).

## Publishing a release

1. Raise the version in `Cargo.toml` and add it to the [release notes](release-notes.md).
2. Run the [smoke test](#smoke-test) on a zip built from that commit.
3. Push a tag named after the version:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

The tag runs `.github/workflows/release.yml`. It checks that the tag matches the version in
`Cargo.toml`, runs the tests, builds the package with `scripts\package.ps1` and publishes the zip
and its `.sha256` on the repository's Releases page, titled `Lambda Slicer v0.2.0`, with notes
listing the changes since the last release.

- A version with a suffix, such as `v0.2.0-beta.1`, is marked as a pre-release.
- A tag that doesn't match the version stops the run before anything is built.
- Running it again for the same tag replaces the release's files.

## Smoke test

To run on the zip before a release, noting the date and anything odd in [engine
validation](engine-validation.md):

1. Unzip anywhere and start `lambda-slicer-gui.exe`. No console window opens, and Open demo and
   Add folder pulse.
2. Add a demo folder. The scan runs in the background, the window keeps responding, and Cancel
   stops it.
3. Drop a demo and a folder on the window. A file that isn't a demo is named as such.
4. Filter the library by map, player, weapon, players, headshots, accuracy and dates. Sort by a
   column, and choose columns with a right-click on a heading.
5. Open a demo and set the Half-Life folder when asked. The demo plays with sound and its HUD.
   Seek, rewind, step frames, change the speed, zoom and go full screen.
6. Golden minute jumps, the scoreboard follows a selected event, and the timeline zooms with the
   wheel and pans by dragging.
7. Clip an event, type a start as `1:36` and an end as `4:48`, drag a clip edge on the timeline
   and check the range the export would cut.
8. Set the export folder inside `Half-Life\ag`, export the clip, then export it again. The second
   export gets a `_2` name and nothing is replaced. Copy the `playdemo` command, paste it into the
   game's console and watch the clip.
9. Switch the language to Arabic and to Chinese without restarting. The text shows, and the layout
   mirrors in Arabic.

The translations also wait for native speakers to review them ([languages](i18n.md)).

## What's supported

| Demos or platform | Status | How it was checked |
|---|---|---|
| Steam Half-Life and AG demos (protocol 5/48), reading | supported | container tests, a scan of the whole local collection ([demo format](protocol-profile-5-48.md)) |
| AG multiplayer from the player's view, analysis | supported | demos recorded on purpose and the local collection ([analysis](analysis-profile.md)) |
| AG multiplayer, clips | supported | every export checked against its source, clips played in the game, among them `[96, 288)` and a 5 against 5 match ([engine validation](engine-validation.md)) |
| Singleplayer, analysis | supported | `run_92.dem` and `run_108.dem` |
| Singleplayer, clips, across level changes too | supported | clips starting after a level change, played in the game |
| Spectating | supported | a spectator clip played in the game |
| WON demos (protocol 5/47), singleplayer | supported | singleplayer demos from two WON installations |
| WON demos (protocol 5/47), multiplayer | not tested | |
| HLTV | listed and analysed, no clips | coverage note `coverage-hltv` |
| Other protocols and mods | refused with `protocol-unsupported` | |
| Damaged or cut-off demos | read as far as they go, no clip from unknown state | partial reports and refusals |
| Windows app | built and checked in screenshots, the smoke test above pending | |
| Interface languages | complete, checked in screenshots, native review pending | [languages](i18n.md) |
| Demo player on Windows | supported and measured, the picture compared with the game's in three clips, the sound not yet | [demo player](viewer-profile.md), [comparison](viewer-validation.md) |
| Linux | builds and tests pass, a real desktop and the game pending | below |

## Linux

The workspace builds and every test passes on Linux x86_64 (Ubuntu in WSL, 2026-10-17), and CI
repeats that. Building the app needs the ALSA development package for sound output
(`libasound2-dev` on Debian and Ubuntu) and `pkg-config`. At run time it needs ALSA, xkbcommon (with
`libxkbcommon-x11-0` on X11), a Vulkan or OpenGL driver, an XDG desktop portal for the file
dialogs, and fontconfig fonts for Chinese and Arabic text.

The demo player was checked on Linux in WSL on 2026-10-19, against real demos and the game's
files. Every seek and every way of reaching the test moments agreed with playing straight through,
its window opened and played under Xvfb (which needs `libxkbcommon-x11-0` on X11), and its
pictures drawn in software matched the Windows ones ([demo player](viewer-profile.md), "Linux").

**Not tried yet** on a real Linux desktop: the app's drawing, file dialogs, drag and drop, system
fonts, sound output, the demo player's speed with a GPU, and playing exported clips in the Linux
game.

### Desktop launcher

The app's window icon is embedded on Windows and Linux. Windows builds also embed the
multi-resolution program icon from `assets/branding/app.ico`. The MSVC build finds the Windows
SDK's resource compiler itself, and `RC` can point to another. GNU Windows builds use `windres`,
which `WINDRES` can override (use the target's own compiler when cross-compiling). Linux builds
use neither.

For a launcher on a Linux desktop, after building the release binary, install it with the desktop
entry and icons from the repository's root:

```sh
install -Dm755 target/release/lambda-slicer-gui "$HOME/.local/bin/lambda-slicer-gui"
install -Dm644 packaging/lambda-slicer.desktop "$HOME/.local/share/applications/lambda-slicer.desktop"
for size in 16 24 32 48 64 128 256 512 1024; do
  install -Dm644 "assets/branding/icons/icon-$size.png" "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/lambda-slicer.png"
done
```

`$HOME/.local/bin` has to be on the desktop session's `PATH`. The desktop file's name matches the
app's Wayland ID, so the compositor finds the launcher's icon. Some desktops need a new login to
show the icon. How the app looks on a Linux desktop hasn't been checked.

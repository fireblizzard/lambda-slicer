# Release

The M7 gate (implementation plan §10) and the support matrix (§12), with what is proven and what
still waits for a person to try it.

## Building and checking

`.\scripts\check.ps1` runs what CI runs: formatting, clippy with warnings as errors, every test
and a release build, all `--locked`. `-Linux` repeats clippy, the tests and the release build in
WSL, building into the Linux home. CI (`.github/workflows/ci.yml`) runs the same on
`windows-latest` and `ubuntu-latest`, then packages on Windows and keeps the zip as a build
artifact. The private corpus and the game stay out of CI. The first run on GitHub, for `00b9dee`
on 2026-10-17, passed on both systems in about 13 minutes and produced the zip
([run 39056627064](https://github.com/fireblizzard/lambda-slicer/actions/runs/39056627064)). The local fixture checks run with

```powershell
$env:LAMBDA_SLICER_FIXTURE_MANIFEST = "$PWD\local-fixtures.json"
cargo test --release -p lambda-slicer-core --test local_corpus -- --ignored
```

and include `exports_leave_every_fixture_unchanged`, which exports from every fixture and checks
that its bytes did not change.

## Windows package

`.\scripts\package.ps1` builds the release binaries and writes
`dist\lambda-slicer-<version>-windows-x86_64.zip` and its `.sha256`. The zip holds
`lambda-slicer-gui.exe` (no console window in a release build), `lambda-slicer.exe`, a short
user README (`packaging\README.txt`), `LICENSE-MIT`, `LICENSE-APACHE`, `VERSION.txt` (version,
commit, build time) and `THIRD-PARTY-NOTICES.txt`. `scripts\notices.py` writes the notices from
`cargo metadata` for the Windows target: every crate the binaries link (243 for 0.1.0), its
license and the license files it publishes, including the licenses of egui's bundled fonts.
Build and development dependencies are left out, since none of their code ships. Nothing is
installed: the app keeps its settings and clips in `%APPDATA%\lambda-slicer` and its cache in
`%LOCALAPPDATA%\lambda-slicer`.

Lambda Slicer is licensed under MIT OR Apache-2.0. All shipped dependencies are under licenses
compatible with that, and the corrected plural rules in `vendor` keep their own MIT OR
Apache-2.0 license ([dependency audit](dependency-audit.md)).

## Application icons and desktop launcher

The app's window icon is embedded on Windows and Linux. Windows builds also embed the
multi-resolution executable icon from `assets/branding/app.ico`. The MSVC build uses the
Windows SDK resource compiler, discovered automatically; `RC` can override its executable
path. GNU Windows builds use `windres`, overridable with `WINDRES` (use the target-specific
compiler when cross-compiling). Native Linux builds do not invoke either compiler.

For a Linux desktop launcher, after building the release binary, install it and the
matching desktop entry and icons from the repository root:

```sh
install -Dm755 target/release/lambda-slicer-gui "$HOME/.local/bin/lambda-slicer-gui"
install -Dm644 packaging/lambda-slicer.desktop "$HOME/.local/share/applications/lambda-slicer.desktop"
for size in 16 24 32 48 64 128 256 512 1024; do
  install -Dm644 "assets/branding/icons/icon-$size.png" "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/lambda-slicer.png"
done
```

Ensure `$HOME/.local/bin` is on the desktop session's `PATH`. The desktop filename matches
the Wayland app ID, allowing the compositor to find the launcher icon. Some desktops may
need a new login to refresh their icon cache. Linux desktop appearance remains untested.

## Linux

The workspace builds and every test passes on Linux x86_64 (Ubuntu in WSL, 2026-10-17), and CI
repeats that. Building the GUI needs the ALSA development package for sound output
(`libasound2-dev` on Debian and Ubuntu) and `pkg-config`. At run time it needs ALSA, xkbcommon, a
Vulkan or OpenGL driver, an XDG desktop portal for the file dialogs, and fontconfig fonts for CJK
and Arabic text. **Pending** until tried on a Linux desktop: the GUI's rendering,
file dialogs, drag and drop, system font discovery, and playback of exported demos in the Linux
game.

## Support matrix

| Capability | Status | Evidence |
|---|---|---|
| GoldSrc 5/48 metadata | supported | bounded container tests, corpus header scan ([protocol profile](protocol-profile-5-48.md)) |
| AG 5/48 multiplayer POV analysis | supported | controlled and corpus fixtures ([analysis profile](analysis-profile.md)) |
| AG 5/48 independent slicing | supported | structural and semantic validation of every export, S1 and M7 game tests including `[96, 288)` and a 5 against 5 match ([engine validation](engine-validation.md)) |
| Singleplayer analysis | supported | `run_92.dem` and `run_108.dem` fixtures |
| Singleplayer slicing, level changes included | supported | later-session restarts (M7), `ls_m7_sp_*` played in the game |
| Spectator recordings | supported | `ls_m7_spectator` played in the game |
| HLTV | recognised and labelled, not sliced | coverage note `coverage-hltv` |
| Network 46/47, other mods | refused with `protocol-unsupported` | profile check |
| Truncated or corrupt demos | diagnosed, no export from unknown state | partial decode reports, refusals |
| Windows GUI | built and screen-checked, interactive smoke test pending | checklist below |
| Interface languages | complete catalogs, screens checked, native review pending | [languages](i18n.md) |
| Linux | builds and tests pass, desktop and game pending | above |

## Interactive smoke test (Windows)

To run once on the release zip before calling it release-ready, recording the date and anything
odd in [engine validation](engine-validation.md):

1. Unzip anywhere and start `lambda-slicer-gui.exe`. No console window opens.
2. Add a demo folder. The scan runs in the background, the window stays responsive, and Cancel
   stops it.
3. Drop a demo and a folder on the window. A file that is not a demo is named as such.
4. Filter the library by map, player, weapon, headshots, accuracy and dates. Sort by a column and
   choose columns with a right-click on a heading.
5. Open a demo. Golden minute jumps, the scores follow a selected event, the timeline zooms with
   the wheel and pans by dragging.
6. Add a clip around an event, type a start as `1:36` and an end as `4:48`, drag a clip edge on
   the timeline and check the range the export would cut.
7. In Settings set the export folder inside `Half-Life\ag`, export the clip, then export it again.
   The second export gets a `_2` name and nothing is replaced. Copy the `playdemo` command, paste
   it into the game console and watch the clip.
8. Switch the language to Arabic and to Chinese without restarting. Text renders and the layout
   mirrors in Arabic.

## Before calling it release-ready

From plan §12: the M7 engine matrix played with observations recorded and the engine `version`
recorded (both done 2026-10-17, game build 8684), CI green on GitHub (done 2026-10-17), the smoke
test above done on the zip CI built, and the translations reviewed by native speakers. Known gaps
that do not block the release are in [deferred](deferred.md).

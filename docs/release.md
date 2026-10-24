# Releasing

How a release is built, checked and published, and what each kind of demo is known to support.
What a release brings is in the [release notes](release-notes.md), and what it lacks in [known
limitations](limitations.md).

## Building and checking

`.\scripts\check.ps1` runs what CI runs: formatting, clippy with warnings as errors, every test
and a release build, all `--locked`. `-Linux` repeats clippy, the tests and the release build in
WSL, building into the Linux home.

CI (`.github/workflows/ci.yml`) runs the same on `windows-latest` and `ubuntu-latest`, then
packages on Windows and Ubuntu 24.04. It keeps the Windows zip, Linux tar.gz and their `.sha256`
files as build artifacts, each uploaded as it is, without another zip wrapper. The
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
2. Run the [smoke test](#smoke-test) on packages built from that commit, on both platforms.
3. Push a tag named after the version:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

The tag runs `.github/workflows/release.yml`. Each platform checks that the tag matches the
version in `Cargo.toml` and runs the workspace tests. Windows builds with `scripts\package.ps1`;
Linux builds with `python3 scripts/package-linux.py` on `ubuntu-24.04` and checks the package
with `python3 scripts/check-linux-package.py`. The Linux runner is pinned so a change to
`ubuntu-latest` cannot silently raise the release's glibc requirement.

Only after both platforms succeed does one publishing job download all four assets, verify
their checksums and publish them on the repository's Releases page, titled
`Lambda Slicer v0.2.0`, with notes listing the changes since the last release. Only that job has
permission to write releases. Both build jobs retain their packages as workflow artifacts.

- A version with a suffix, such as `v0.2.0-beta.1`, is marked as a pre-release.
- A tag that doesn't match the version stops the run before anything is built.
- Running it again for the same tag replaces the release's files.
- Changes to this workflow take effect for tags pointing to commits containing them. Re-running
  an older tag uses that tag's workflow, so it does not acquire Linux packaging automatically.
- ARM64, AppImage, Flatpak, deb and rpm packages are not produced by this workflow.

## Linux package

On Linux x86-64, with Python 3.11+ and the build dependencies below:

```sh
python3 scripts/package-linux.py
python3 scripts/check-linux-package.py
```

The packager uses the pinned Rust toolchain, `--locked` and the explicit
`x86_64-unknown-linux-gnu` target. It respects Cargo's target directory, including
`CARGO_TARGET_DIR`. Output is `dist/lambda-slicer-<version>-linux-x86_64.tar.gz` and its
`.tar.gz.sha256`. The archive contains the GUI and CLI with executable permissions, a Linux
README, `install.py`, the desktop entry, all icon sizes, both app licenses, build provenance in
`VERSION.txt`, and third-party notices generated for the Linux target.

The package check verifies the checksum, extracts the actual archive, checks linked libraries
and executable permissions, runs the CLI, and installs twice into temporary homes, including
paths with spaces and a percent sign and a custom `XDG_DATA_HOME`. It validates the desktop
entry with `desktop-file-validate` (installed by CI), checks the icon and notices, and confirms
the installed CLI still runs after deleting the extracted package. It does not replace the
manual GUI, audio and game smoke tests.

The published build targets glibc 2.39+ and uses system graphics/audio libraries. It is intended
for Ubuntu 24.04+, CachyOS and similarly recent distributions, not every Linux installation.
An archive built locally inherits the build host's library requirements; building on a newer
distribution does not establish compatibility with Ubuntu 24.04. No game assets or FFmpeg are
bundled. The package README lists runtime dependencies and installation/removal instructions.

## Smoke test

To run on each package before a release, noting the date and anything odd in [engine
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

On Linux, extract the tar.gz and start `./lambda-slicer-gui` for step 1. Also run
`python3 install.py` without sudo, then launch from the desktop menu and check the sliced-lambda
icon in the menu and taskbar on KDE/Wayland. Check that launch still works after moving or
deleting the extracted folder. Use Linux game paths for step 8.

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
| Linux | builds and tests pass; CachyOS/VirtualBox desktop testing in progress | below |

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

The app has also been opened on CachyOS in VirtualBox. Desktop icon integration is being checked;
a full desktop smoke test, including file dialogs, drag and drop, fonts, sound, viewer performance
and playing exported clips in the Linux game, remains to be recorded.

### Desktop launcher

The app's window icon is embedded on Windows and Linux. Windows builds also embed the
multi-resolution program icon from `assets/branding/app.ico`. The MSVC build finds the Windows
SDK's resource compiler itself, and `RC` can point to another. GNU Windows builds use `windres`,
which `WINDRES` can override (use the target's own compiler when cross-compiling). Linux builds
use neither.

The Linux archive includes a per-user installer. From the extracted package:

```sh
python3 install.py
```

Python 3.10+ is needed only for installation, not to run the binaries directly. Run without
sudo. Both binaries go in `~/.local/bin`, icons and the launcher under `XDG_DATA_HOME` (default
`~/.local/share`), and licenses/notices under its `lambda-slicer/release` folder. The installer
copies files, so the extracted package can then be removed. Re-running it upgrades the app
without changing settings or the demo library; close the app before upgrading.

The launcher has an absolute, quoted executable path and an absolute icon path. It does not
depend on the desktop session's `PATH` or icon-theme lookup. Its filename matches the app's
Wayland ID (`lambda-slicer`). The installer refreshes the desktop database and KDE cache when
those tools are present. A stale pinned taskbar entry may need to be unpinned and pinned again;
some desktops need a new login. The embedded window icon alone is insufficient on Wayland.

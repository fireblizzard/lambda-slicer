# Development

Lambda Slicer is a Rust workspace. The Rust version is pinned in `rust-toolchain.toml` and
installs itself with [rustup](https://rustup.rs). Handwritten code follows
[CODE_STYLE.md](../CODE_STYLE.md).

## Crates

| Crate | What it does |
|---|---|
| `lambda-slicer-core` | Reads demos, works out shots, kills and highlights, cuts clips, and keeps the library |
| `lambda-slicer-assets` | Finds and reads the game's files: maps, models, sprites, textures and sounds |
| `lambda-slicer-render` | Draws maps and models with `wgpu` |
| `lambda-slicer-playback` | Plays a demo: time, seeking, effects, the HUD's state and sounds to play |
| `lambda-slicer-audio` | Mixes and plays the sound |
| `lambda-slicer-gui` | The app, built with egui |
| `lambda-slicer-cli` | `lambda-slicer.exe`, the [command line](cli.md) tool |

## Running and checking

```powershell
cargo run --release -p lambda-slicer-gui      # the app
.\scripts\check.ps1                           # formatting, clippy, tests and a release build, as CI runs them
.\scripts\check.ps1 -Linux                    # the same again in WSL
.\scripts\package.ps1                         # the Windows zip, in dist
```

The app takes options for testing, listed at the top of `crates/lambda-slicer-gui/src/main.rs`.
For example, `--locale ar` picks a language, `--cache` and `--clips` use other files than the
real ones, and `--demo FILE --screenshot OUT.png` saves a picture of the window once the demo is
open. [The app](gui.md) has more.

Building on Linux needs `pkg-config` and the ALSA development package (`libasound2-dev` on Debian
and Ubuntu). [Releasing](release.md) covers what it needs at run time.

## Tests with your own demos

Demos are never committed (`*.dem` is ignored). Some tests run on local demos listed in a
manifest ([fixtures](fixture-recording.md)):

```powershell
$env:LAMBDA_SLICER_FIXTURE_MANIFEST = "$PWD\local-fixtures.json"
cargo test --release -p lambda-slicer-core --test local_corpus -- --ignored
```

## Surveys of a demo collection

These read many demos and change none. A pass over a whole collection of tens of thousands of
demos takes 20 to 35 minutes, so they're usually run on a sample:

```powershell
# a 2000 demo sample: known problem demos plus a spread across maps, the same each time for one seed
cargo run --release -p lambda-slicer-core --example make_sample -- work\problem-demos.txt work\sample-2000.txt 2000 20261015 'F:\HLScreeniesDemos' 'F:\SteamLibrary\steamapps\common\Half-Life\ag'
cargo run --release -p lambda-slicer-core --example corpus_scan -- --list work\sample-2000.txt
cargo run --release -p lambda-slicer-core --example decode_scan -- --list work\sample-2000.txt
cargo run --release -p lambda-slicer-core --example crosscheck_scan -- --list work\sample-2000.txt
```

`work\problem-demos.txt` lists demos earlier surveys flagged, one path per line. It and the
samples stay out of Git, as they hold local paths.

## Documents

- [The app](gui.md), [the library](library.md) and [user messages](messages.md).
- [How kills and shots are worked out](analysis-profile.md).
- [The demo format](protocol-profile-5-48.md) and how clips were [tested in the
  game](engine-validation.md).
- The demo player: [how it works](viewer-profile.md), [why `wgpu`](viewer-backend.md) and [how it
  was compared with the game](viewer-validation.md).
- [Languages](i18n.md) and [right-to-left text](gui-text.md).
- [Dependencies](dependency-audit.md) and [releasing](release.md).
- [History](history/): the original plans, kept as a dated record.

![Lambda Slicer — Half-Life demo analysis and slicing](assets/branding/banner.png)

# Lambda Slicer

Lambda Slicer is a free app for your Half-Life and Adrenaline Gamer demos. It keeps your
whole demo collection in one table you can search and filter. And then you can watch
those demos, make smaller demos out of them, see the chat, the kill feed specifying
headshots, mid-air kills, kills when going very fast, and tries to highlight intense
moments so that you can find good footage quicker.

![A demo playing in Lambda Slicer, with its events and the demo list beside it](docs/images/demo-view.png)

## Download

**Windows 10 or 11, 64-bit:** get the latest zip from the
[Releases page](https://github.com/fireblizzard/lambda-slicer/releases). Unzip it anywhere and run
`lambda-slicer-gui.exe`. There is nothing to install.

You'll probably get a **SmartScreen** security alert when trying to open it, because it's not a popular app
and I haven't paid for a signed certificate or anything like that. The code for the app is here in this
repository, and if the app tried to do something harmful someone would point it out. So, unless you downloaded
the app from an unofficial page, you can just hit **More info** and **Run anyway**.

**Linux, x86-64:** new releases include a `linux-x86_64.tar.gz` on the
[Releases page](https://github.com/fireblizzard/lambda-slicer/releases). Extract it and run
`./lambda-slicer-gui`, or run `python3 install.py` inside the extracted folder to install it
in your application menu with its icon. Do not use `sudo` for the installer. The binaries
target Ubuntu 24.04 or newer and recent distributions such as CachyOS (glibc 2.39+).
See [Linux requirements](docs/release.md#linux). Older releases may only have Windows downloads,
but you can also [build it yourself](#building-it-yourself).

![The library with its filters](docs/images/library.png)

### Watching a demo

Lambda Slicer plays demos itself without starting the game. It only needs to know where Half-Life
is installed so that you can watch the demo, as it needs the textures, models, etc. to render it.
The first time you open a demo it will ask you to point it to `hl.exe` or the Half-Life folder.

You can also open a `.dem` from the Windows explorer and watch it right away.
Double click on a `.dem` or right click and then _Open with_, and link it to `lambda-slicer-gui.exe`.

The demo should show as if you were playing it in the game, with your own models, textures, the
same HUD including custom text and stuff that many servers like to add. But you'll benefit from
some quality of life features as it's just like watching a video, so you have a seek bar, buttons
to go 10 seconds back and forward, go frame by frame, rewind, slow down, speed up... and then other
stuff like a zooming in or seeing players through walls, as long as the demo has info about other players.

| Key | Does |
|---|---|
| Space or L, or click the picture | Play or pause |
| J | Rewind |
| K | Pause |
| Left / Right | 5 seconds back / forward |
| Shift+Left / Shift+Right | 10 seconds back / forward |
| Comma / Period | One frame back / forward |
| Home | Back to the start |
| Z | Zoom in or out |
| X | Toggle seeing players through walls |
| F | Toggle fullbright |
| N | Toggle player names |
| C | Free camera: fly with W, A, S and D, Space up, Ctrl down, Shift faster, and hold the right mouse button to look around |
| F11 or double-click the picture | Full screen |
| Ctrl+Left / Ctrl+Right | Previous / next event |
| I / O | Set the chosen clip's start / end to now |
| Escape | Leave full screen, close the settings, or close the demo |
| Ctrl+O | Open a demo |
| Ctrl+Comma | Settings |

### Finding the good moments

- First **filter** the demo library a bit, as you might have lots of trash demos not worth checking, so set a
  minimum duration, a minimum accuracy stat, tick headshots if that's important for you, if you're looking for
  kills with a specific weapon then select that in the filters, if you're looking for a specific tournament
  it might have a specific naming convention, so you can filter by name "LLHL" for example. Then **sort** the table
  by intensity, or by accuracy, try different things.
- The **Matches** tab lists every match on its own row, even when a demo has several, with its score,
  players and whether the whole match was recorded. Untick statuses in the _Status_ filter to skip the
  ones cut short, and click a match to start watching from its countdown.
- Then click on demos and look at the **event list**. It holds every kill, chat message, vote and match event in order, with
  the weapon, headshots, mid-air kills and the players' speeds. The list follows the demo as it plays, and clicking an
  event takes you there. Double clicking an event puts you a few seconds before the actual event, which is better most of
  the time. Filter by _Midair_ or look for the `AIR` badge on the event list if you want cool kills like that.
- **Jump to highlight** lists the five most intense minutes of the match, the best one first (the golden minute),
  and takes you straight there. These moments are ranked by a formula that penalizes onesided scores (10-0)
  and rewards more balanced ones (5-4). It doesn't account for headshots, mid-air shots and any other skill-based
  stuff, so there's a lot of room to improve there.

### Clips

- Press **Clip it** on an event, or **Clip this intense moment** on a highlight. The clip starts
  4 seconds before and ends 1 second after, which you can change in Settings. You can also type the times or set them
  while you watch.
- **Export** saves the clip as a new demo that plays on its own in the game. Lambda Slicer never
  changes your original demos and never replaces a file that's already there.
- Set the export folder to a folder in your mod folder, such as `Half-Life\ag\clips`. Each clip
  then shows the console command that plays it, like `playdemo clips/mymatch__Clip_1`,
  with a button to copy it.

## Which demos work

| Demos | Works |
|---|---|
| Half-Life and AG from the Steam version (protocol 48): playing, spectating, singleplayer | Yes |
| The older WON version (protocol 47), singleplayer | Yes |
| The older WON version (protocol 47), multiplayer | Not tested yet |
| HLTV | Listed and analysed, but no clips, and playback isn't tested |
| Other mods, such as Counter-Strike or Opposing Force | No |

## Good to know

- A demo only holds what the server sent to the player who recorded it. Other players usually show
  up only while the recorder could see them, so their shots and kills elsewhere aren't in the demo.
- Some clips can't be cut, for example across a map change or right while a map is loading. The
  app tells you why.
- The demo player doesn't show everything the game does yet. The flashlight, dynamic lights,
  particles and voice chat are among what's missing. The full list is in
  [known limitations](docs/limitations.md).
- The app is in 14 languages: English, Arabic, Chinese, Finnish, French, German, Italian,
  Lithuanian, Polish, Brazilian Portuguese, Russian, Latin American Spanish, Turkish and Ukrainian.
  Apart from English and Spanish, they are drafts that native speakers haven't checked yet.
  Corrections are welcome.
- Settings and clips are kept in `%APPDATA%\lambda-slicer`, and the library in
  `%LOCALAPPDATA%\lambda-slicer`.

## Building it yourself

You need [Rust](https://rustup.rs). The right version installs itself from `rust-toolchain.toml`.
On Linux you also need `pkg-config` and the ALSA development package (`libasound2-dev` on Debian
and Ubuntu).

```
cargo build --release -p lambda-slicer-gui
```

The app is then `target/release/lambda-slicer-gui`.

On Windows `.\scripts\package.ps1` builds the
same zip as the Releases page. On Linux, `python3 scripts/package-linux.py` builds the
Linux archive, including the desktop installer. [Development](docs/development.md) covers the checks and tests.

## More

- [Command line](docs/cli.md): `lambda-slicer.exe`, the same analysis and clipping for scripts.
- [Release notes](docs/release-notes.md): what changed in each version.
- [Known limitations](docs/limitations.md): what doesn't work yet.
- [How the app works](docs/gui.md), [the library](docs/library.md) and
  [how kills and shots are worked out](docs/analysis-profile.md).
- Technical notes on [the demo format](docs/protocol-profile-5-48.md),
  [the demo player](docs/viewer-profile.md) and how clips were [tested in the
  game](docs/engine-validation.md).
- [Releasing a new version](docs/release.md) and [translating](docs/i18n.md).

## License

Lambda Slicer is licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. The libraries it includes and their licenses are listed in `THIRD-PARTY-NOTICES.txt` in
the release archive.

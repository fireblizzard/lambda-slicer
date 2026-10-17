# Lambda Slicer artwork

The approved identity pairs a rounded orange lambda, split into three pieces, with
white “Slicer” lettering and a cyan timeline on charcoal. The word Lambda is supplied
by the symbol. The app icon omits the lettering and ruler to remain readable at small sizes.

| File | Use |
| --- | --- |
| `banner.png` | Approved 2172 × 724 README banner, unchanged |
| `icon-master.png` | Generated RGBA master with transparent corners |
| `icons/icon-{size}.png` | 16, 24, 32, 48, 64, 128, 256, 512 and 1024 px desktop exports |
| `app.ico` | Windows icon containing 16–256 px images; also usable as a favicon |
| `logo-transparent.png` | Standalone orange mark for other backgrounds |
| `wordmark-transparent.png` | Orange mark, white Slicer and cyan ruler for dark backgrounds |
| `social-preview.png` | 1280 × 640 card for repository link previews and announcements |
| `social-preview-master.png` | Original generated social card before size export |

The GUI embeds `icons/icon-256.png` for the native window. Its build script compiles
`app.ico` into the Windows executable, so the portable app needs no loose image files.
The resource is only linked into the GUI binary. Linux builds skip resource compilation.
The Linux desktop entry in `packaging` uses the matching `lambda-slicer` app/icon ID.

The generated masters and exports are committed. A normal Cargo build needs no image
generation service, Python or Pillow. To regenerate size/format exports after changing
the icon master, install Pillow and run `python scripts/export_branding.py`.

The images were generated with the built-in image tool; prompts are in [PROMPTS.md](PROMPTS.md).
The transparent mark and wordmark preserve the generator's alpha channel. Use the charcoal
banner or social card for a complete wordmark on light pages.

The social card is ready for upload in GitHub repository Settings → Social preview.
Committing the file alone does not set the repository's social preview.

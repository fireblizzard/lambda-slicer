"""Export desktop icons and the social preview from approved masters. Requires Pillow.

Run from any directory with: python scripts/export_branding.py
These are format/size exports only; the generated master artwork is preserved.
"""

from pathlib import Path

from PIL import Image


def main():
    branding = Path(__file__).resolve().parents[1] / "assets" / "branding"
    sizes = (16, 24, 32, 48, 64, 128, 256, 512, 1024)
    icons = branding / "icons"
    icons.mkdir(exist_ok=True)

    with Image.open(branding / "icon-master.png") as source:
        master = source.convert("RGBA")
    if master.width != master.height:
        raise ValueError("The icon master must be square")

    for size in sizes:
        icon = master.resize((size, size), Image.Resampling.LANCZOS)
        icon.save(icons / f"icon-{size}.png", optimize=True)

    master.save(
        branding / "app.ico",
        format="ICO",
        sizes=[(size, size) for size in sizes if size <= 256],
    )

    with Image.open(branding / "social-preview-master.png") as source:
        social = source.convert("RGB")
    if social.width != 2 * social.height:
        raise ValueError("The social preview master must have a 2:1 aspect ratio")
    social.resize((1280, 640), Image.Resampling.LANCZOS).save(
        branding / "social-preview.png", optimize=True
    )


if __name__ == "__main__":
    main()

Lambda Slicer for Linux x86-64
=============================

Extract the entire archive to a folder on your Linux disk. Run
./lambda-slicer-gui from that folder, or install it for your user:

    python3 install.py

Run the installer without sudo. It copies both executables to ~/.local/bin,
installs the menu entry and sliced-lambda icons, and refreshes KDE's launcher
cache when available. The launcher uses absolute paths, so it does not need
~/.local/bin on PATH. Open Lambda Slicer from the application menu afterward.
You can delete the extracted folder after installing. To upgrade, close the
app, extract the new release and run its installer again.

The installer needs Python 3.10 or newer. The app itself does not need Python,
Rust, Steam or FFmpeg. To watch demos, select your own Half-Life installation
folder containing valve/ and, for AG, ag/. Game assets are not included.

Requirements
------------

The published binaries are built on Ubuntu 24.04 for x86-64 Linux with glibc
2.39 or newer. CachyOS and recent Ubuntu/Fedora releases are intended targets;
this is not a static binary or an Alpine/musl package. Older distributions
can build from source. Other CPU architectures are not included yet.

You need ALSA, xkbcommon (including its X11 library on X11), an OpenGL or Vulkan
driver, fontconfig and fonts, and an XDG desktop portal with the backend for
your desktop. These normally come with a full desktop installation.

On CachyOS with KDE, missing libraries can be installed with:

    sudo pacman -Syu --needed alsa-lib libxkbcommon libxkbcommon-x11 mesa vulkan-icd-loader fontconfig xdg-desktop-portal-kde

On Ubuntu 24.04 with GNOME:

    sudo apt install libasound2t64 libxkbcommon0 libxkbcommon-x11-0 libgl1 libegl1 libvulkan1 mesa-vulkan-drivers fontconfig xdg-desktop-portal-gnome

Virtual machines may need 3D acceleration enabled. If the default graphics
backend fails, try:

    WGPU_BACKEND=opengl ~/.local/bin/lambda-slicer-gui

Verify the download before extracting, with the .sha256 file beside it:

    sha256sum -c lambda-slicer-<version>-linux-x86_64.tar.gz.sha256

Using the app
-------------

Add a demo folder or drop a .dem onto the window. Select your Half-Life folder
when asked. Click an event to seek to it, or Clip it to prepare a shorter demo.
Exports never modify the source demo or replace an existing exported demo.
To play an exported demo in the game, copy the playdemo command shown by the app.

lambda-slicer is the command-line tool. Run ./lambda-slicer --help for commands.
After installing, use ~/.local/bin/lambda-slicer, or add ~/.local/bin to PATH.

Uninstall
---------

Remove ~/.local/bin/lambda-slicer and ~/.local/bin/lambda-slicer-gui, the
lambda-slicer.desktop file in ~/.local/share/applications, lambda-slicer.png
in each ~/.local/share/icons/hicolor/*/apps directory, and
~/.local/share/lambda-slicer/release. If you set XDG_DATA_HOME, substitute that
for ~/.local/share. These files are separate from your settings and library.

Source, instructions and issue reports: https://github.com/fireblizzard/lambda-slicer
LICENSE-MIT and LICENSE-APACHE contain the app's licenses.
THIRD-PARTY-NOTICES.txt lists bundled dependencies. VERSION.txt identifies this build.

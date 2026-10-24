"""Install an extracted release for the current user, including its desktop icon."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def desktop_value(value):
    return str(value).replace('\\', '\\\\').replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t')


def desktop_command(path):
    value = str(path).replace('%', '%%')
    for character in ('\\', '"', '`', '$'):
        value = value.replace(character, '\\' + character)
    return desktop_value('"' + value + '"') + ' %F'


def install(source, destination, mode=0o644):
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as stream:
        temporary = Path(stream.name)
    try:
        shutil.copyfile(source, temporary)
        temporary.chmod(mode)
        temporary.replace(destination)
    finally:
        temporary.unlink(missing_ok=True)


def main():
    if sys.platform != 'linux' or os.geteuid() == 0:
        raise SystemExit('Run this installer on Linux as your normal user, without sudo.')
    source = Path(__file__).resolve().parent
    home = Path.home()
    data = Path(os.environ.get('XDG_DATA_HOME') or home / '.local/share')
    if not data.is_absolute():
        raise SystemExit('XDG_DATA_HOME must be an absolute path.')
    binaries = home / '.local/bin'
    icon = data / 'icons/hicolor/256x256/apps/lambda-slicer.png'
    required = ['lambda-slicer-gui', 'lambda-slicer', 'lambda-slicer.desktop',
                'README.txt', 'LICENSE-MIT', 'LICENSE-APACHE', 'THIRD-PARTY-NOTICES.txt', 'VERSION.txt']
    sizes = (16, 24, 32, 48, 64, 128, 256, 512, 1024)
    for relative in required + [f'icons/icon-{size}.png' for size in sizes]:
        if not (source / relative).is_file():
            raise SystemExit(f'Incomplete package: missing {relative}')
    for binary in ('lambda-slicer-gui', 'lambda-slicer'):
        install(source / binary, binaries / binary, 0o755)
    for size in sizes:
        install(source / f'icons/icon-{size}.png',
                data / f'icons/hicolor/{size}x{size}/apps/lambda-slicer.png')
    for name in required[3:]:
        install(source / name, data / 'lambda-slicer/release' / name)
    lines = (source / 'lambda-slicer.desktop').read_text(encoding='utf-8').splitlines()
    lines = [
        'Exec=' + desktop_command(binaries / 'lambda-slicer-gui') if line.startswith('Exec=')
        else 'Icon=' + desktop_value(icon) if line.startswith('Icon=') else line
        for line in lines
    ]
    launcher = data / 'applications/lambda-slicer.desktop'
    launcher.parent.mkdir(parents=True, exist_ok=True)
    launcher.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    launcher.chmod(0o644)
    for command in (['update-desktop-database', str(launcher.parent)], ['kbuildsycoca6', '--noincremental']):
        if shutil.which(command[0]):
            result = subprocess.run(command, check=False)
            if result.returncode:
                print(f'{command[0]} could not refresh the desktop cache; try logging out and back in.')
    print(f'Installed Lambda Slicer in {binaries}. Open it from your application menu.')
    print('You can now remove the extracted package. Close and reopen the app after upgrading.')


if __name__ == '__main__':
    main()

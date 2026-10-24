"""Check the actual Linux archive and install it into a temporary user directory."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib


def main():
    if sys.platform != 'linux' or os.geteuid() == 0:
        raise SystemExit('Run the package check on Linux without sudo.')
    root = Path(__file__).resolve().parent.parent
    version = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    name = f'lambda-slicer-{version}-linux-x86_64'
    archive = root / 'dist' / f'{name}.tar.gz'
    with archive.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    assert archive.with_suffix('.gz.sha256').read_text().strip() == f'{digest}  {archive.name}'
    if not shutil.which('desktop-file-validate'):
        raise SystemExit('Install desktop-file-utils to check the desktop launcher.')

    with tempfile.TemporaryDirectory(prefix='lambda-package-check-') as directory:
        work = Path(directory)
        with tarfile.open(archive) as tar:
            assert all(member.name == name or member.name.startswith(name + '/') for member in tar)
            tar.extractall(work, filter='data')
        package = work / name
        for binary in ('lambda-slicer-gui', 'lambda-slicer'):
            path = package / binary
            assert os.access(path, os.X_OK), f'{binary} lost its executable permission'
            with path.open('rb') as stream:
                assert stream.read(4) == b'\x7fELF', f'{binary} is not a Linux executable'
            linked = subprocess.check_output(['ldd', str(path)], text=True)
            assert 'not found' not in linked, linked
        result = subprocess.check_output([str(package / 'lambda-slicer'), '--version'], text=True)
        assert result.strip() == f'lambda-slicer {version}', result
        assert 'x86_64-unknown-linux-gnu' in (package / 'THIRD-PARTY-NOTICES.txt').read_text()

        for custom_data in (False, True):
            home = work / ('home with spaces %' + str(custom_data))
            data = home / ('custom data' if custom_data else '.local/share')
            env = dict(os.environ, HOME=str(home), XDG_CACHE_HOME=str(home / 'cache'),
                       XDG_CONFIG_HOME=str(home / 'config'))
            env.pop('XDG_DATA_HOME', None)
            if custom_data:
                env['XDG_DATA_HOME'] = str(data)
            sentinel = home / 'config/lambda-slicer/keep.txt'
            sentinel.parent.mkdir(parents=True)
            sentinel.write_text('user settings')
            for _ in range(2):
                subprocess.run([sys.executable, str(package / 'install.py')], env=env, check=True)
            assert sentinel.read_text() == 'user settings'
            launcher = data / 'applications/lambda-slicer.desktop'
            subprocess.run(['desktop-file-validate', str(launcher)], check=True)
            fields = dict(line.split('=', 1) for line in launcher.read_text().splitlines() if '=' in line)
            escaped_home = str(home).replace('%', '%%')
            assert fields['Exec'] == f'"{escaped_home}/.local/bin/lambda-slicer-gui" %F'
            icon = Path(fields['Icon'])
            assert icon == data / 'icons/hicolor/256x256/apps/lambda-slicer.png'
            assert icon.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
            assert fields['StartupWMClass'] == 'lambda-slicer'
            for binary in ('lambda-slicer-gui', 'lambda-slicer'):
                installed = home / '.local/bin' / binary
                assert installed.read_bytes() == (package / binary).read_bytes()
                assert os.access(installed, os.X_OK)
            assert (data / 'lambda-slicer/release/THIRD-PARTY-NOTICES.txt').stat().st_size > 1000
        shutil.rmtree(package)
        subprocess.run([str(home / '.local/bin/lambda-slicer'), '--help'], check=True,
                       stdout=subprocess.DEVNULL)
    print('Linux package: checksum, executable permissions, dependencies, CLI and desktop installation passed.')


if __name__ == '__main__':
    main()

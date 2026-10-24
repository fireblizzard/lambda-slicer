"""Build the Linux x86-64 release archive and its SHA-256 checksum."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib


def run(*args):
    return subprocess.check_output(args, text=True).strip()


def main():
    if sys.platform != 'linux' or platform.machine() != 'x86_64':
        raise SystemExit('Build this package on Linux x86-64.')
    root = Path(__file__).resolve().parent.parent
    os.chdir(root)
    target = 'x86_64-unknown-linux-gnu'
    version = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    name = f'lambda-slicer-{version}-linux-x86_64'
    subprocess.run([
        'cargo', 'build', '--release', '--locked', '--target', target,
        '-p', 'lambda-slicer-gui', '-p', 'lambda-slicer-cli',
    ], check=True)
    metadata = json.loads(run('cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked'))
    binaries = Path(metadata['target_directory']) / target / 'release'
    dist = root / 'dist'
    dist.mkdir(exist_ok=True)
    archive = dist / f'{name}.tar.gz'
    with tempfile.TemporaryDirectory(prefix='linux-package-', dir=dist) as work:
        stage = Path(work) / name
        stage.mkdir()
        for binary in ('lambda-slicer-gui', 'lambda-slicer'):
            shutil.copyfile(binaries / binary, stage / binary)
            (stage / binary).chmod(0o755)
        for license_name in ('LICENSE-MIT', 'LICENSE-APACHE'):
            shutil.copyfile(root / license_name, stage / license_name)
        for source, destination in (
            ('README-linux.txt', 'README.txt'),
            ('install-linux.py', 'install.py'),
            ('lambda-slicer.desktop', 'lambda-slicer.desktop'),
        ):
            shutil.copyfile(root / 'packaging' / source, stage / destination)
        shutil.copytree(root / 'assets/branding/icons', stage / 'icons')
        subprocess.run([
            sys.executable, 'scripts/notices.py', target, str(stage / 'THIRD-PARTY-NOTICES.txt'),
        ], check=True)
        commit = run('git', 'rev-parse', '--short', 'HEAD')
        dirty = ' with uncommitted changes' if run('git', 'status', '--porcelain') else ''
        built = datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%d %H:%M UTC')
        (stage / 'VERSION.txt').write_text(
            f'Lambda Slicer {version}\nCommit {commit}{dirty}\nBuilt {built} for {target}\n',
            encoding='utf-8',
        )
        temporary_archive = Path(work) / archive.name
        with tarfile.open(temporary_archive, 'w:gz') as tar:
            tar.add(stage, arcname=name)
        temporary_archive.replace(archive)
    with archive.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    archive.with_suffix('.gz.sha256').write_text(f'{digest}  {archive.name}\n', encoding='ascii')
    print(archive)
    print(f'sha256 {digest}')


if __name__ == '__main__':
    main()

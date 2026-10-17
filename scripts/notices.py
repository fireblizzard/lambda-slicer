"""Write THIRD-PARTY-NOTICES.txt for a release: every crate the shipped
binaries link, for one target, with its license and the license files its
published crate carries. Build-only and development dependencies are left
out, since none of their code is shipped. Identical license texts are written
once, listing the crates that share them.

    python scripts/notices.py x86_64-pc-windows-msvc dist/THIRD-PARTY-NOTICES.txt
"""
import json
import pathlib
import re
import subprocess
import sys

target, out = sys.argv[1], pathlib.Path(sys.argv[2])
meta = json.loads(subprocess.run(
    ['cargo', 'metadata', '--format-version', '1', '--locked', '--filter-platform', target],
    check=True, capture_output=True, text=True,
).stdout)

packages = {p['id']: p for p in meta['packages']}
nodes = {n['id']: n for n in meta['resolve']['nodes']}
members = set(meta['workspace_members'])
shipped = [i for i in members if packages[i]['name'] in ('lambda-slicer-gui', 'lambda-slicer-cli')]

# Normal dependencies reachable from the shipped binaries
seen, todo = set(), list(shipped)
while todo:
    node = nodes[todo.pop()]
    for dep in node['deps']:
        if any(k['kind'] is None for k in dep['dep_kinds']) and dep['pkg'] not in seen:
            seen.add(dep['pkg'])
            todo.append(dep['pkg'])
crates = sorted((packages[i] for i in seen - members), key=lambda p: (p['name'], p['version']))

LICENSE_FILE = re.compile(r'^(licen[cs]e|copying|notice|unlicense|copyright)', re.I)


def license_files(p):
    root = pathlib.Path(p['manifest_path']).parent
    files = sorted(f for f in root.iterdir() if f.is_file() and LICENSE_FILE.match(f.name))
    # Bundled fonts carry their licenses beside them, as egui's default fonts do
    for folder in ('fonts', 'licenses', 'LICENSES'):
        if (root / folder).is_dir():
            files += sorted(f for f in (root / folder).iterdir() if f.suffix.lower() == '.txt')
    if p.get('license_file'):
        extra = root / p['license_file']
        if extra.is_file():
            files.append(extra)
    # Folder names match either way on a case-insensitive file system
    unique = {f.resolve(): f for f in files}
    return sorted(unique.values())


texts = {}
missing = []
for p in crates:
    label = f"{p['name']} {p['version']}"
    files = license_files(p)
    if not files:
        missing.append(p)
    for f in files:
        text = f.read_text(encoding='utf8', errors='replace').strip()
        where = f.relative_to(pathlib.Path(p['manifest_path']).parent).as_posix()
        texts.setdefault(text, []).append(f'{label} ({where})')

lines = [
    'Third-party notices for Lambda Slicer',
    '',
    f'The Lambda Slicer binaries for {target} include code from the {len(crates)} crates below,',
    'each under the license shown. Their license texts follow, each written once with the crates',
    'that carry it.',
    '',
]
width = max(len(p['name']) + len(p['version']) + 1 for p in crates)
for p in crates:
    name = f"{p['name']} {p['version']}"
    lines.append(f"{name.ljust(width)}  {p.get('license') or 'see its license file'}")
if missing:
    lines += [
        '',
        'These crates publish no license file. Their license is the expression above, whose',
        'standard text is among the texts below (MIT and Apache-2.0 at the end), and their source is',
        'at the repository given:',
        '',
    ]
    for p in missing:
        lines.append(f"{p['name']} {p['version']}: {p.get('repository') or 'crates.io'}")
for text, owners in texts.items():
    lines += ['', '=' * 78, *owners, '-' * 78, text]
root = pathlib.Path(__file__).resolve().parent.parent
for name in ('LICENSE-MIT', 'LICENSE-APACHE'):
    lines += ['', '=' * 78, f'Standard text, {name}', '-' * 78, (root / name).read_text(encoding='utf8').strip()]

out.parent.mkdir(parents=True, exist_ok=True)
out.write_text('\n'.join(lines) + '\n', encoding='utf8')
print(f'{len(crates)} crates, {len(texts)} license texts, {len(missing)} without a license file -> {out}')

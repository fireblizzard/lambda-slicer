# Lambda Slicer

Desktop tool for slicing Half-Life: Adrenaline Gamer (GoldSrc) demos and finding weapon use in
them. The full design is in [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

## Status

| Milestone | State |
|---|---|
| M0 scaffold | done: workspace, CLI, fixture manifest, docs, CI |
| M1 container + S0 controls | done; S0 passed in game ([engine-validation](docs/engine-validation.md)) |
| M2 network state decoder | decodes the whole corpus; delta bases are proven or flagged as inferred ([protocol profile](docs/protocol-profile-5-48.md)) |
| M3 arbitrary-start slicing | not started |
| M4–M7 analysis, library, GUI, release | not started |

No GUI yet: the plan requires proving playable nonzero-start slices (M3) before GUI work.

## CLI

```powershell
cargo build --release
.\target\release\lambda-slicer.exe inspect  'path\to\demo.dem'            # add --json - for JSON
.\target\release\lambda-slicer.exe validate 'path\to\demo.dem'            # non-zero exit on errors
.\target\release\lambda-slicer.exe control exact-copy   SRC --output OUT.dem
.\target\release\lambda-slicer.exe control roundtrip    SRC --output OUT.dem
.\target\release\lambda-slicer.exe control truncate-end SRC --end 4:48 --output OUT.dem
.\target\release\lambda-slicer.exe decode   'path\to\demo.dem'            # --events, --trace FROM TO, --json -
.\target\release\lambda-slicer.exe crosscheck 'path\to\demo.dem'          # exit 0 verified, 1 failed, 2 inconclusive
.\target\release\lambda-slicer.exe hash FILES...
.\target\release\lambda-slicer.exe fixtures --manifest local-fixtures.json
```

With `--json -` stdout carries only the JSON document, and errors go to stderr. The text listings
from `--events` and `--trace` therefore need `--json FILE`.

Times accept `96`, `1:36`, or `00:01:36.250`. Exports never overwrite an existing file, never
modify the source, and work from a private snapshot of it. The reported source hash is computed
while that snapshot is copied, so it covers exactly the bytes the export read, and a source that
changes during the copy aborts the export.

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
# optional, needs local demos (see docs/fixture-recording.md):
$env:LAMBDA_SLICER_FIXTURE_MANIFEST = "$PWD\local-fixtures.json"
cargo test -p lambda-slicer-core --test local_corpus -- --ignored
# read-only surveys; a full corpus pass takes 20-35 minutes, so normally run them on a sample:
# build a 2000-demo sample (known problem files plus a seeded spread across maps)
cargo run --release -p lambda-slicer-core --example make_sample -- work\problem-demos.txt work\sample-2000.txt 2000 20261015 'F:\HLScreeniesDemos' 'F:\SteamLibrary\steamapps\common\Half-Life\ag'
cargo run --release -p lambda-slicer-core --example corpus_scan -- --list work\sample-2000.txt
cargo run --release -p lambda-slicer-core --example decode_scan -- --list work\sample-2000.txt
cargo run --release -p lambda-slicer-core --example crosscheck_scan -- --list work\sample-2000.txt
```

`work\problem-demos.txt` lists demos earlier scans flagged, one path per line; it and the sample
stay out of Git because they contain local paths.

Original demos are never committed (`*.dem` is ignored).

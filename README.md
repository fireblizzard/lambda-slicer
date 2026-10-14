# Lambda Slicer

Desktop tool for slicing Half-Life: Adrenaline Gamer (GoldSrc) demos and finding weapon use in
them. The full design is in [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

## Status

| Milestone | State |
|---|---|
| M0 scaffold | done: workspace, CLI, fixture manifest, docs, CI |
| M1 container + S0 controls | code done; **in-game S0 test pending** ([engine-validation](docs/engine-validation.md)) |
| M2 network state decoder | not started |
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
.\target\release\lambda-slicer.exe hash FILES...
.\target\release\lambda-slicer.exe fixtures --manifest local-fixtures.json
```

Times accept `96`, `1:36`, or `00:01:36.250`. Exports never overwrite an existing file, never
modify the source, and verify the source hash before and after.

## Development

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
# optional, needs local demos (see docs/fixture-recording.md):
$env:LAMBDA_SLICER_FIXTURE_MANIFEST = "$PWD\local-fixtures.json"
cargo test -p lambda-slicer-core --test local_corpus -- --ignored
# read-only corpus survey:
cargo run --release -p lambda-slicer-core --example corpus_scan -- 'F:\HLScreeniesDemos'
```

Original demos are never committed (`*.dem` is ignored).

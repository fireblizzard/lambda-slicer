# Command line

`lambda-slicer.exe` comes in the same zip as the app and uses the same analysis and clipping. It's
handy for scripts and for bug reports. `lambda-slicer.exe help` lists every command, and
`lambda-slicer.exe help COMMAND` explains one.

## Everyday commands

```powershell
lambda-slicer.exe analyze match.dem                       # weapon use, shots and who fired them
lambda-slicer.exe analyze match.dem --kills --weapon crossbow --victim-airborne
lambda-slicer.exe slice match.dem --start 1:36 --end 4:48 --output clip.dem
lambda-slicer.exe scan D:\demos --workers 4               # add demos to the library, Ctrl-C keeps what's done
lambda-slicer.exe query --kill-weapon crossbow --victim-airborne
lambda-slicer.exe query --min-spectating 0.5              # demos recorded mostly while spectating
lambda-slicer.exe query --windows --limit 20              # the most intense moments in the library
```

`scan` and `query` use the same library as the app.

Times can be written as `96`, `1:36` or `00:01:36.250`. A clip, like one exported from the app,
never replaces a file and never changes the original demo.

## Checking demos

```powershell
lambda-slicer.exe inspect match.dem       # the file's header and every frame
lambda-slicer.exe validate match.dem      # exits with an error when the file is damaged
lambda-slicer.exe decode match.dem        # read the whole match: --events, --trace FROM TO
lambda-slicer.exe crosscheck match.dem    # compare what was worked out with what the game recorded
lambda-slicer.exe hash FILES...           # size, date and BLAKE3 hash of files
```

`crosscheck` exits with 0 when everything matched, 1 when something didn't, and 2 when it couldn't
tell.

## Test copies

`control` writes simple copies of a demo for testing the game's playback:

```powershell
lambda-slicer.exe control exact-copy   match.dem --output copy.dem      # byte for byte
lambda-slicer.exe control roundtrip    match.dem --output rebuilt.dem   # rebuilt from its frames
lambda-slicer.exe control truncate-end match.dem --end 4:48 --output start.dem
```

## JSON

Most commands take `--json FILE` to write a JSON report, or `--json -` to print it. With
`--json -` only the JSON goes to the output and errors go to the error output, so the text
listings of `decode --events` and `--trace` need `--json FILE` instead.

# User messages

The core stays language-neutral (implementation plan §8.1). Everything it reports to a user is a
`message::Message` with a stable code, named arguments and the English text. The CLI, logs and
JSON show the English text, and the GUI looks the code up in its catalog for the chosen language.

```json
{"code": "container-too-short", "args": {"len": 3}, "text": "file is 3 bytes, shorter than the 544-byte header"}
```

## Rules

- **Codes** are kebab-case and double as Fluent message ids. A code never changes meaning once
  released. A new meaning gets a new code.
- **Arguments** are integers, numbers or text. Paths, player names and times are text, with times
  formatted as `HH:MM:SS.mmm` like the CLI. A `detail` argument carries developer-level English
  text, such as the system's I/O error or a decoder diagnostic, which catalogs show as is.
- **The full list** is `message::CODES`. Building a message with a code missing from it fails in
  debug builds, so tests catch a code nobody added, and the GUI's catalog test checks every code
  in the list.
- **Errors** implement `message::Coded`, which gives their code and arguments while their
  `Display` stays the English text.

## Coded today

| Source | Codes |
|---|---|
| Opening and reading a demo | `io-*`, `container-*`, `container-frame-*` |
| Protocol | `protocol-unsupported`, `protocol-unsupported-network` |
| Network decoding | `decode-failed`, with the English diagnostic as `detail` |
| Coverage notes | `coverage-visible-only`, `coverage-no-local-events`, `coverage-singleplayer`, `coverage-hltv`, `coverage-unknown`, `coverage-mostly-spectating` |
| Export blockers | `export-*` |
| Library | `cache-*`, `analysis-panicked`, and scan outcomes as enum tags |

The decode report's `error`, the analysis report's `decode_error` and coverage `notes`, the
inspect report's `error`, and scan outcomes carry messages. The library cache stores each file's
error code and arguments next to the text (cache schema 2).

## Still English only

These are developer diagnostics. The GUI shows them under a translated heading, untranslated.

- Structural findings, which have stable codes but English details full of byte offsets
- Cross-check reasons
- Restart export details (`carried`, `approximations`, ongoing effect notes) and fixture
  mismatches

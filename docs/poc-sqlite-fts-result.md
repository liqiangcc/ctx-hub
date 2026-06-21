# SQLite FTS POC Result

## Result

The SQLite FTS POC is considered validated after CI passed on Linux, macOS, and Windows.

## Verified scope

- Rust project can build.
- The `ctx` CLI binary can be produced.
- SQLite is embedded through the Rust dependency chain and does not require a separate user installation.
- The application can create a local database file.
- Records can be inserted into the SQLite records table.
- SQLite FTS5 search is available.
- Trigram search is available for substring-style lookup.
- CJK n-gram generation is available for short Chinese search terms.
- CLI smoke tests exercise init, add, search, show, tag, and list-tags.
- CI uses strict rustfmt and clippy gates.

## CI gates

The POC branch requires all of the following gates to pass:

- Format check.
- Compile check.
- Clippy with warnings as errors.
- Test suite.
- Release build.

## Not covered by this POC

- MCP server integration.
- Clipboard copy support.
- JSONL import and export.
- Schema migration management.
- Performance benchmark data.
- Production-grade search ranking.

## Decision

The POC result supports moving from technical research into formal MVP development.

The recommended next step is to convert the POC into a cleaner MVP implementation plan rather than merging the POC branch as production-ready code.

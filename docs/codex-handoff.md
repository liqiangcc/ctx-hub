# Codex Handoff

This document is the working handoff for continuing `liqiangcc/ctx-hub` with Codex.

## Current repository state

Repository: `liqiangcc/ctx-hub`

Current branch: `feat/mvp-storage`

Current pull request: PR #3, `MVP Stage 2: Formalize SQLite storage`

Stage 1 has already been completed and merged.

Stage 2 is implemented in PR #3 and should be finished first before starting the next stage.

## What PR #3 contains

PR #3 formalizes the storage layer.

Main changes:

- Added `src/lib.rs` so integration tests can import project modules.
- Added `Storage` trait in `src/storage/mod.rs`.
- Split SQLite schema into `src/storage/schema.rs`.
- Added schema version tracking in `src/storage/migration.rs`.
- Updated `SqliteStorage` to use schema and migration modules.
- Implemented `Storage` for `SqliteStorage`.
- Moved FTS query escaping into `src/core/query.rs`.
- Added query escaping tests.
- Added schema initialization tests.
- Added migration tests.
- Added SQLite integration tests in `tests/storage_sqlite.rs`.
- Added CJK n-gram integration tests in `tests/ngram.rs`.

## CI requirement

Do not weaken CI.

The following commands must keep passing:

- `cargo fmt --all -- --check`
- `cargo check --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build --release --all-features`

CI must pass on:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`

Before this handoff document was added, the current PR had a green effective CI run across all three platforms.

After this file is committed, verify CI again before merging.

## Rules for Codex

- Keep changes small and focused.
- Prefer one stage per PR.
- Do not remove tests just to make CI pass.
- Use real temporary SQLite databases for storage and search tests.
- Keep CLI, storage, core logic, and MCP boundaries separate.
- MCP is read-only for the MVP.
- Do not add unrelated infrastructure features to this project.
- Update documentation when user-facing behavior changes.

## Immediate next step

Finish PR #3 first.

Checklist:

- Verify PR #3 CI is green after this handoff file is committed.
- If CI fails, fix the real failure and keep all CI gates strict.
- Remove draft status only after CI is green and the user approves.
- Merge PR #3 into `main` only after approval.

Do not start Stage 3 until PR #3 is merged or the user explicitly asks to continue on the same branch.

## Stage 3: search formalization

After PR #3 is merged, create a new branch from `main` named `feat/mvp-search`.

Goal: make search behavior explicit, tested, and stable.

Tasks:

- Make exact key match highest priority.
- Keep FTS5 search for normal full-text lookup.
- Keep trigram search for substring lookup.
- Keep CJK n-gram search for short Chinese terms.
- Add service-name search tests.
- Add command-fragment search tests.
- Add tag search tests.
- Add FTS escaping edge-case tests.
- Add snippet behavior tests.
- Keep storage/search tests backed by real SQLite temporary databases.

Acceptance criteria:

- Exact key search returns the intended record first.
- Chinese short-term search works.
- English keyword search works.
- Service-name search works.
- Command-fragment search works.
- Special query characters do not break search.
- Search results include useful snippets.
- CI passes on all three platforms.

Suggested test files:

- `tests/search_fts.rs`
- `tests/search_cjk.rs`
- `tests/search_trigram.rs`
- `tests/search_ranking.rs`

## Stage 4: CLI MVP

After Stage 3 is merged, create a new branch from `main` named `feat/mvp-cli`.

Expected CLI commands for MVP:

- `ctx db init`
- `ctx db info`
- `ctx db rebuild-index`
- `ctx add`
- `ctx search <keyword>`
- `ctx show <key-or-id>`
- `ctx tag <tag>`
- `ctx list-tags`
- `ctx copy <key-or-id>`

Tasks:

- Improve CLI help text.
- Improve CLI error messages.
- Implement `ctx copy <key-or-id>`.
- Expand CLI smoke tests.
- Keep output readable for humans.

Acceptance criteria:

- CLI works with a temporary database path.
- Add, search, show, tag, list-tags, db info, and rebuild-index work.
- Copy behavior is documented and tested.
- CI passes on all three platforms.

## Stage 5: JSONL import and export

Create a branch named `feat/jsonl-import-export`.

Expected commands:

- `ctx db export --format jsonl`
- `ctx db import <file>`

Tasks:

- Define a stable JSONL record schema.
- Export active records.
- Import records into a new database.
- Define duplicate key behavior.
- Add import/export round-trip tests.

Acceptance criteria:

- Export creates valid JSONL.
- Import restores records into a new database.
- Duplicate key behavior is explicit.
- Search works after import.
- CI passes.

## Stage 6: read-only MCP

Create a branch named `feat/mcp-readonly`.

Expected tools:

- `search_context`
- `get_context_by_key`
- `list_tags`
- `get_service_context`

Rules:

- MCP must call core or storage APIs.
- MCP must not duplicate database logic.
- MCP must stay read-only for MVP.
- Add tests that prove read-only behavior.

Acceptance criteria:

- MCP can search context.
- MCP can get one record by key.
- MCP can list tags.
- MCP can return service-related context.
- Documentation includes a basic MCP configuration example.
- CI passes.

## Stage 7: release readiness

Create a branch named `feat/release-readiness`.

Tasks:

- Update README installation steps.
- Document default database path.
- Document `CTX_HUB_DB`.
- Document backup and restore.
- Document MCP setup.
- Add release workflow if needed.

Acceptance criteria:

- User can build and run the CLI from README.
- User can initialize a database.
- User can add and search records.
- User can configure MCP.
- User can back up and restore data.
- CI passes.

## Definition of done for MVP

The MVP is done when:

- Storage is stable.
- Search covers key personal use cases.
- CLI has the required MVP commands.
- JSONL import/export works.
- Read-only MCP works.
- README explains setup and usage.
- CI passes on Linux, macOS, and Windows.

## Codex starting instruction

When Codex starts, read this file first.

Start by checking the current branch and PR status.

If PR #3 is still open, finish PR #3 first. After it is green and approved, merge it into `main`. Then start Stage 3 on a new branch.

Always keep CI strict and do not remove tests to make CI pass.

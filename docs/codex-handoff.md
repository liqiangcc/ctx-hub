# Codex Handoff

This document is the working handoff for continuing `liqiangcc/ctx-hub` with Codex.

## Current Repository State

Repository: `liqiangcc/ctx-hub`

Canonical branch: `main`

Active MVP pull request: none

MVP status: Stage 1 through Stage 7 are complete and merged.

Latest completed main commit:

```text
d45c346 MVP Stage 7: Document release readiness
```

The post-merge CI run for `d45c346` completed successfully on:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`

CI run:

```text
https://github.com/liqiangcc/ctx-hub/actions/runs/28031166388
```

## Completed MVP Stages

- Stage 1: project/module skeleton, merged through PR #2.
- Stage 2: SQLite storage formalization, merged through PR #3.
- Stage 3: search formalization, merged through PR #5.
- Stage 4: CLI MVP, merged through PR #6.
- Stage 5: JSONL import/export, merged through PR #7.
- Stage 6: read-only MCP server, merged through PR #8.
- Stage 7: release readiness, merged through PR #9.

## Current MVP Capabilities

The MVP now supports:

- local SQLite-backed storage
- schema initialization and migration tracking
- exact key lookup
- FTS5 full-text search
- trigram substring search
- CJK n-gram search
- tag search
- service-name search
- human-readable CLI commands
- `ctx copy`
- JSONL import/export for backup and restore
- read-only stdio MCP tools
- README installation, database path, backup/restore, and MCP setup docs
- CI on Linux, macOS, and Windows
- release artifact workflow for tag or manual builds

## CI Requirement

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

## Rules For Codex

- Keep changes small and focused.
- Prefer one stage or one clear follow-up per PR.
- Do not remove tests just to make CI pass.
- Use real temporary SQLite databases for storage and search tests.
- Keep CLI, storage, core logic, and MCP boundaries separate.
- MCP is read-only for the MVP.
- Do not add unrelated infrastructure features to this project.
- Update documentation when user-facing behavior changes.

## Definition Of Done For MVP

The MVP is done when:

- Storage is stable.
- Search covers key personal use cases.
- CLI has the required MVP commands.
- JSONL import/export works.
- Read-only MCP works.
- README explains setup and usage.
- CI passes on Linux, macOS, and Windows.

All items above are complete as of `d45c346`.

## Immediate Next Step

There is no remaining MVP implementation stage in this handoff.

Recommended next action after this handoff update is merged:

1. Create a `v0.1.0` tag from the latest `main`.
2. Push the tag to trigger `.github/workflows/release.yml`.
3. Verify the release artifact workflow uploads Linux, macOS, and Windows binaries.

Use the existing package version in `Cargo.toml`:

```text
0.1.0
```

Do not start a broad post-MVP feature stage until the user explicitly asks for a new roadmap or selects a follow-up issue.

## Suggested Post-MVP Backlog

Possible follow-ups after `v0.1.0`:

- Create a GitHub Release from the generated artifacts.
- Add checksums for release binaries.
- Add shell completion generation.
- Improve install instructions for Homebrew, Scoop, or direct binary downloads.
- Add update and delete commands.
- Add record editing commands.
- Add more structured filters for service, environment, source, and status.
- Dogfood the CLI with real records and turn friction into small issues.

## Codex Starting Instruction

When Codex starts, read this file first.

Start by checking the current branch, working tree status, and latest `main` CI status.

If the user says to execute the next step and no newer roadmap exists, continue with the `v0.1.0` tag and release-artifact verification flow described above.

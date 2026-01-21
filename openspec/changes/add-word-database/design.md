## Context
The app stores word data locally and already plans SQLite usage. A stable schema is needed for word lists, words, and list membership.

## Goals / Non-Goals
- Goals: define core SQLite tables and constraints for word storage.
- Non-Goals: add UI, API, or import workflow behavior.

## Decisions
- Use SQLite3 tables `word_list`, `word`, and `word_list_map`, with `created_at` timestamps on `word_list` and `word`.
- Enforce unique words via a unique constraint on `word`.
- Enforce unique list membership via a composite uniqueness constraint on (`word_list_id`, `word_id`).
- Enable SQLite foreign key enforcement when creating connections (for `word_list_map` integrity).

## Risks / Trade-offs
- Future schema changes will require migrations and backfills.

## Migration Plan
- Add a migration to create the tables and constraints.

## Open Questions
- None.

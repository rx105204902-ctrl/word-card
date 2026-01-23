## Context
The app already exposes a Settings view with a Fuzzy Words navigation entry but lacks the subpage and management behaviors. The fuzzy list needs consistent sorting and a system-managed review list.

## Goals / Non-Goals
- Goals: Provide a fuzzy words subpage, sorting, bulk operations, and a system-managed fuzzy word list for review.
- Non-Goals: Custom naming of the system list, remote sync, or export flows.

## Decisions
- Store fuzzy mark state with a timestamp per word to support sorting by marked time.
- Use a reserved list name ("Ä£ºý´Ê´Ê¿â") for the system-managed review list and block manual deletion.
- Default sorting to marked time descending, with an option to sort alphabetically.

## Risks / Trade-offs
- Adding a new timestamp column requires a migration and backfill for existing data.

## Migration Plan
- Add a schema migration to store fuzzy mark timestamps (nullable).
- On startup or after mark updates, create/remove the system-managed list based on fuzzy count.

## Open Questions
- Should fuzzy marks be scoped per word list or global across all lists?

## Context
We need a home screen learning flow that presents words randomly, supports backtracking, and tracks per-word proficiency.

## Goals / Non-Goals
- Goals: session-based word allocation, consistent history navigation, per-word proficiency with feedback, and a visible indicator.
- Non-Goals: spaced repetition beyond the defined tiers, audio playback, cloud sync.

## Decisions
- Decision: Store per-word progress in user_word_learning with columns word_id, proficiency_score, last_learned_at, learn_count.
- Decision: Allocate 50 words per session from the active word list using the requested tiered sampling and fill shortages from other tiers, then roll over automatically after completion.
- Decision: Use a history stack so Previous always returns the most recently learned word.

## Risks / Trade-offs
- Random selection can repeat words across sessions; we accept this for simplicity and will rely on tiered sampling for balance.

## Migration Plan
- Add the user_word_learning table and any needed indexes via a schema migration.
- Backfill is not required; rows are created on first learn.

## Open Questions
- None.

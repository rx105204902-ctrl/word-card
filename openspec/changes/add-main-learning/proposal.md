# Change: Add main learning mode on the home screen

## Why
The current home screen lacks a structured learning flow that tracks progress and proficiency per word. This change defines the learning session, navigation, and feedback loop needed for effective study.

## What Changes
- Add a learning session that allocates 50 words per entry and presents them in random order.
- Add a user_word_learning table to track proficiency_score, last_learned_at, and learn_count per word.
- Add Previous, Next, and Fuzzy actions with a history stack for backtracking.
- Add a proficiency indicator on the home screen.

## Impact
- Affected specs: learn-words
- Affected code: database schema/migrations, repository queries, learning session service, Tauri commands, Vue home screen UI

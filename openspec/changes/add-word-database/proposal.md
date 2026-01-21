# Change: Add word database schema

## Why
The app needs a local, normalized schema to persist word lists and words in SQLite3 so import and review features can rely on stable storage.

## What Changes
- Add SQLite3 tables `word_list`, `word`, and `word_list_map` with required columns.
- Define uniqueness and foreign key constraints for words and list membership.
- Introduce a new capability spec for word bank storage.

## Impact
- Affected specs: word-bank-storage (new)
- Affected code: src-tauri database migrations and repository layer

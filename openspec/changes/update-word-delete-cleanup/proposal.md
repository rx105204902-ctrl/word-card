# Change: Cleanup words when deleting a word list

## Why
Deleting a word list should also remove orphaned words to free storage while preserving words used by other lists.

## What Changes
- When a word list is deleted, remove its word mappings.
- Delete words that are no longer referenced by any word list.

## Impact
- Affected specs: dictionary-management
- Affected code: word list deletion logic in Rust

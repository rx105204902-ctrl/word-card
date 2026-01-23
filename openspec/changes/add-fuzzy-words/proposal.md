# Change: Add fuzzy words management

## Why
Users need a dedicated place to review words they marked as fuzzy, manage them in bulk, and revisit them for focused review.

## What Changes
- Add a Fuzzy Words subpage under Settings that lists fuzzy-marked words with sorting, selection, and detail navigation.
- Support removing fuzzy marks from single or multiple words.
- Persist fuzzy mark timestamps for sorting by marked time.
- Automatically create and maintain a system-managed "Ä£ºý´Ê´Ê¿â" word list when fuzzy words exist; remove it when none remain.

## Impact
- Affected specs: view-fuzzy-words (new), word-bank-fuzzy-words (new)
- Affected code: src/App.vue, settings view components, word repository/services, database migrations

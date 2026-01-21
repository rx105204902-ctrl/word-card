# Change: Add word list selection for imports

## Why
Users need to choose a target word list when importing uploaded CSV files so the imported words can be organized and linked without losing existing associations.

## What Changes
- Show a modal on Import to select an existing word list or create a new one by name.
- Import words from uploaded CSV files using the existing header format validation.
- Upsert words by `word`, updating fields when duplicates are imported.
- Associate imported words with the selected word list without removing existing list links.

## Impact
- Affected specs: word-bank-import (modified)
- Affected code: src/App.vue, src-tauri/src/lib.rs, src-tauri/src/word_bank.rs

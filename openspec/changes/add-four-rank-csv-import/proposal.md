# Change: Add four-rank CSV import in Settings > More

## Why
Users need a fast way to seed the word bank with a curated list. The project already includes `src-tauri/file/four_rank.csv`; importing the same CSV format enables the app to start with real data.

## What Changes
- Add an import control in Settings > More that allows selecting a CSV file (e.g. `src-tauri/file/four_rank.csv`).
- Parse CSV content that matches the `four_rank.csv` header and persist entries to a local SQLite3 database.
- Show a simple import summary (total/upserted/skipped) and clear errors for invalid formats.

## Impact
- Affected specs: view-settings (modified), word-bank-import (new capability)
- Affected code: src/App.vue, src-tauri/src/lib.rs, src-tauri/Cargo.toml

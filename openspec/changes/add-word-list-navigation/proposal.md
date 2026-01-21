# Change: Add word list navigation with active selection

## Why
Users need to see available word lists, their sizes, and switch the active list from the UI while keeping only one list active at a time.

## What Changes
- Add a word list navigation section showing word list cards with name and word count.
- Add an active list state with single-selection behavior and automatic sort to the top.
- Provide a use button on each card that becomes disabled and shows "Active" for the current list.

## Impact
- Affected specs: word-bank-navigation (new)
- Affected code: src/App.vue, src-tauri/src/word_bank.rs, src-tauri/src/lib.rs

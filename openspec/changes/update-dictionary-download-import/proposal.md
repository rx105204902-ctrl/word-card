# Change: Add dictionary download buttons with auto-import

## Why
Users need one-click dictionary downloads that import into local word lists, with clear downloaded state and re-download support.

## What Changes
- Add download actions for CET4/CET6 that fetch CSV and import into local word lists.
- Hide raw URLs and present named dictionary entries only.
- Track downloaded state based on existing word list names, and allow re-download overwrite.

## Impact
- Affected specs: dictionary-management
- Affected code: frontend dictionary download UI, word list import/merge pipeline, Tauri commands

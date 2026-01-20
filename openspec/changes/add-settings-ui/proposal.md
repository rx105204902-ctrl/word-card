# Change: Add settings view navigation

## Why
Users need a dedicated settings screen reachable from the main UI without cluttering the card view.

## What Changes
- Replace the settings entry in the top-right of the main UI with an icon-only button that navigates to a settings view.
- Add a back button in the top-left of the settings view to return to the main UI.
- Add a compact left-side navigation in the settings view with three items: Word Bank, Fuzzy Words, Study Calendar.
- Settings entry has no functional options yet (navigation only).

## Impact
- Affected specs: view-settings (new capability)
- Affected code: src/App.vue (UI layout + view state), styles in the same file

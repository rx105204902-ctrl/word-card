# Change: Add more settings subpages

## Why
Users need a clear place to access window size and hide behavior options without cluttering the primary settings sections.

## What Changes
- Add a "More" settings subpage with two buttons: Window Size Adjustment and Hide Mode.
- Each button navigates to its own dedicated subpage within the settings view.
- Provide a path to return to the More subpage from those destinations.

## Impact
- Affected specs: view-settings
- Affected code: src/App.vue (settings content, navigation state, styles)

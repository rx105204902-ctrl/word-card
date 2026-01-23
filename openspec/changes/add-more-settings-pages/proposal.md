# Change: Add more settings subpages

## Why
Users need explicit controls for window size adjustment and hide behavior, plus a quick minimize-to-tray action, without cluttering the primary settings sections.

## What Changes
- Add a "More" settings subpage with two buttons: Window Size Adjustment and Hide Mode.
- Each button navigates to its own dedicated subpage within the settings view and can return to "More".
- Window size adjustment only affects the full window, preserving the current aspect ratio with a maximum width of 450px.
- Hide mode adds an edge-line option where the window collapses to a thin white line on screen edge and restores on hover.
- Add a minimize button next to settings in the main UI that hides the window to the system tray; clicking the tray icon restores the window.

## Impact
- Affected specs: view-settings, word-card-ui
- Affected code: src/App.vue (settings UI, sizing logic, hide mode), src-tauri/src/lib.rs (tray icon), src-tauri/Cargo.toml (tray feature)

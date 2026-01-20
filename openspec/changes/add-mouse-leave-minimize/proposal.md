# Change: Add mouse-leave minimize mode

## Why
The current window stays large even when the user is not interacting with it. A compact mode on pointer leave keeps the word visible while minimizing distraction.

## What Changes
- Add a compact mode that triggers when the pointer leaves the app window in any view.
- Start in a fixed 150x50 compact size and hide secondary UI.
- Restore the full UI to a fixed 350x150 window size when the pointer re-enters.
- Design the full UI for proportional layout within 350x150 with a top proficiency color box, word + phonetic line, example sentence, and only Previous/Next buttons.
- Display Chinese translations for the word and example sentence.
- Add a settings button in the top-right corner of the window UI.
- Remove the logo mark from the window and disable window resizing.
- Disable window maximizing to prevent double-click enlargement.
- Pin the window to the bottom-right corner of the desktop.
- Use the desktop work area to avoid overlapping the taskbar.
- Preserve the bottom-right anchor when switching between compact and full sizes after the user moves the window.
- Remove the native title bar (decorations: false) so the UI blends with the window.
- Allow the window to be dragged by the user.

## Impact
- Affected specs: word-card-ui
- Affected code: src/App.vue

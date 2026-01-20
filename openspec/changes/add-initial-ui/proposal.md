# Change: Add initial UI shell

## Why
The current UI is the default Tauri/Vite template and does not represent the word-card app. We need a minimal, focused UI shell to enable further feature work and consistent styling.

## What Changes
- Replace the template UI with a word-card focused layout.
- Introduce base styles, typography, and layout primitives for the card view.
- Prepare placeholders for word, definition, and review actions (no data wiring yet).

## Impact
- Affected specs: word-card-ui (new)
- Affected code: src/App.vue, index.html

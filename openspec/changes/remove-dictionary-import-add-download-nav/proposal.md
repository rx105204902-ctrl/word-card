# Change: Remove dictionary import, drop default data, add download navigation

## Why
The app should not bundle default dictionaries or support importing them. Users should download dictionaries via explicit links instead.

## What Changes
- Remove default dictionary data from the app bundle and initial data seeding.
- Remove dictionary import functionality (CSV upload/import).
- Add a navigation entry for dictionary downloads with explicit CET4/CET6 links.

## Impact
- Affected specs: dictionary-management
- Affected code: frontend navigation, dictionary data initialization, import flow

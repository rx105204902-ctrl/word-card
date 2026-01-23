# Change: Add study calendar views in settings

## Why
Users need to review daily learning activity in the Study Calendar page and switch between a calendar snapshot and a trend view.

## What Changes
- Add a Study Calendar subpage in Settings with a view toggle button in the top-right.
- Provide a calendar view that highlights days with learning records in light green and shows the studied word count in the bottom-left of each date cell.
- Provide a line chart view that shows daily studied word counts over time with auto-scaled y-axis ticks.

## Impact
- Affected specs: view-study-calendar (new capability)
- Affected code: settings view UI in `src/App.vue`, study record aggregation, and view-specific styles

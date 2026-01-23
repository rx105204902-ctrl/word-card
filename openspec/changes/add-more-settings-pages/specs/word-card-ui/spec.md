## MODIFIED Requirements
### Requirement: Mouse-leave compact mode
The app SHALL start in compact mode at a fixed 150x50 size. When the pointer leaves the full window, the hide behavior SHALL follow the selected hide mode: Compact mode switches to the 150x50 compact view, while Edge Line mode collapses the window to a thin white line docked to the screen edge. The full UI SHALL restore to the configured full size when the pointer re-enters the compact window or approaches the edge line.

#### Scenario: Pointer leaves window in compact mode
- **WHEN** the pointer exits the app window and hide mode is Compact
- **THEN** the window resizes to 150x50 and only the compact view is visible

#### Scenario: Pointer leaves window in edge line mode
- **WHEN** the pointer exits the app window and hide mode is Edge Line
- **THEN** the window collapses to a thin white line on the screen edge

#### Scenario: Pointer approaches edge line
- **WHEN** the pointer moves onto the edge line
- **THEN** the full UI is displayed

#### Scenario: Initial launch
- **WHEN** the app launches
- **THEN** the window appears in 150x50 compact mode with only the active word visible

### Requirement: Fixed-size proportional layout
The app SHALL render the full UI proportionally within a fixed aspect ratio based on the 350x155 reference size. The full window width SHALL be adjustable between 350px and 450px, and the height SHALL follow the same ratio.

#### Scenario: Full UI restored at adjusted size
- **WHEN** the full UI is shown after a width adjustment
- **THEN** the layout fits within the configured width and proportional height without clipping core elements

## ADDED Requirements
### Requirement: Minimize to tray button
The app SHALL provide a minimize button to the left of the settings button in the main UI. Activating the button SHALL hide the window to the system tray, and clicking the tray icon SHALL restore the window.

#### Scenario: Minimize to tray
- **WHEN** the user clicks the minimize button
- **THEN** the app window is hidden and remains accessible via the tray icon

#### Scenario: Restore from tray
- **WHEN** the user clicks the tray icon
- **THEN** the app window becomes visible and focused

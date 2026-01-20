## ADDED Requirements
### Requirement: Mouse-leave compact mode
The app SHALL start in compact mode at a fixed 150x50 size. The app SHALL enter compact mode when the pointer leaves the window from any view and restore the full UI to a fixed 350x150 window size when the pointer re-enters the window.

#### Scenario: Pointer leaves window
- **WHEN** the pointer exits the app window from any view
- **THEN** the window resizes to 150x50 and only the active word remains visible.

#### Scenario: Pointer re-enters window
- **WHEN** the pointer re-enters the app window
- **THEN** the window restores to 350x150 and the full UI is visible.

#### Scenario: Initial launch
- **WHEN** the app launches
- **THEN** the window appears in 150x50 compact mode with only the active word visible.

### Requirement: Fixed-size proportional layout
The app SHALL render the full UI proportionally within a fixed 350x150 window without clipping core elements.

#### Scenario: Full UI restored
- **WHEN** the pointer re-enters the app window
- **THEN** the full UI renders within 350x150 with proportional spacing.

### Requirement: Previous and next controls
The app SHALL provide Previous and Next buttons as the only controls in the full UI for word navigation.

#### Scenario: Navigation controls visible
- **WHEN** the full UI is visible
- **THEN** the Previous and Next buttons are visible and no other action buttons are displayed.

### Requirement: Proficiency indicator box
The app SHALL display proficiency as a single small color box above the word, using five levels of color from green to blue.

#### Scenario: Proficiency level display
- **WHEN** a proficiency level is shown
- **THEN** the proficiency box appears above the word and uses the corresponding color level with no text label.

### Requirement: Word and phonetic line
The app SHALL display the word with phonetic text immediately after it, with the phonetic rendered two size steps smaller.

#### Scenario: Word line visible
- **WHEN** the full UI is visible
- **THEN** the phonetic text appears after the word at a smaller size.

### Requirement: Example sentence
The app SHALL display an example sentence beneath the word line.

#### Scenario: Example sentence visible
- **WHEN** the full UI is visible
- **THEN** an example sentence is shown beneath the word line.

### Requirement: Chinese translations
The app SHALL display Chinese translations for the word and the example sentence.

#### Scenario: Translation lines visible
- **WHEN** the full UI is visible
- **THEN** the word translation and example translation are shown.

### Requirement: Settings button
The app SHALL display a settings button in the top-right corner of the window UI.

#### Scenario: Settings control visible
- **WHEN** the full UI is visible
- **THEN** a settings button appears in the top-right.

### Requirement: Logo-free window
The app SHALL NOT render a logo mark in the window UI.

#### Scenario: Full UI visible
- **WHEN** the full UI is visible
- **THEN** no logo mark is displayed.

### Requirement: Non-resizable window
The app SHALL prevent the window from being resized by the user.

#### Scenario: Window resize attempt
- **WHEN** the user attempts to resize the window via system controls
- **THEN** the window size remains fixed.

### Requirement: Non-maximizable window
The app SHALL prevent the window from being maximized, including via double-click on the draggable region.

#### Scenario: Double-click maximize attempt
- **WHEN** the user double-clicks the draggable region
- **THEN** the window size remains unchanged.

### Requirement: Bottom-right default placement
The app SHALL position the window at the bottom-right of the active desktop work area on launch, avoiding the taskbar area. The app SHALL allow the user to drag the window without snapping it back automatically.

#### Scenario: Launch position
- **WHEN** the app launches
- **THEN** the window appears at the bottom-right corner of the active desktop work area.

#### Scenario: Window moved
- **WHEN** the user moves the window
- **THEN** the window remains at the user-selected position.

### Requirement: Bottom-right resize anchor
The app SHALL keep the bottom-right corner fixed when switching between full and compact sizes after the user has moved the window.

#### Scenario: Compact after move
- **WHEN** the user has moved the full window and it switches to compact mode
- **THEN** the compact window aligns its bottom-right corner with the previous full window bottom-right.

#### Scenario: Expand after move
- **WHEN** the compact window expands back to full size
- **THEN** the full window aligns its bottom-right corner with the previous compact window bottom-right.

### Requirement: Borderless window
The app SHALL hide the native title bar so the UI blends with the window.

#### Scenario: Window chrome
- **WHEN** the window is visible
- **THEN** no native title bar is shown.

## ADDED Requirements
### Requirement: More settings entry points
The settings view SHALL include a "More" subpage that lists two buttons: "Window Size Adjustment" and "Hide Mode".

#### Scenario: Display more settings actions
- **WHEN** the settings view shows the More subpage
- **THEN** both buttons are visible

### Requirement: Window size adjustment settings
The settings view SHALL provide a Window Size Adjustment subpage that controls the full window width between 350px and 450px while keeping the current aspect ratio.

#### Scenario: Display size controls
- **WHEN** the Window Size Adjustment subpage is displayed
- **THEN** the width control and current size preview are visible

#### Scenario: Adjust full window width
- **WHEN** the user changes the width control
- **THEN** the full window size updates and the height follows the existing ratio

### Requirement: Hide mode settings
The settings view SHALL provide a Hide Mode subpage with two options: Compact and Edge Line.

#### Scenario: Display hide mode options
- **WHEN** the Hide Mode subpage is displayed
- **THEN** both hide mode options are visible

#### Scenario: Select hide mode
- **WHEN** the user selects a hide mode option
- **THEN** the selection is applied to window hide behavior

### Requirement: More settings subpages
The system SHALL navigate to a dedicated subpage for window size adjustment when the "Window Size Adjustment" button is activated, and to a dedicated subpage for hide mode when the "Hide Mode" button is activated. The system SHALL allow returning to the More subpage from either destination.

#### Scenario: Open window size adjustment page
- **WHEN** the user activates "Window Size Adjustment"
- **THEN** the window size adjustment page is displayed

#### Scenario: Open hide mode page
- **WHEN** the user activates "Hide Mode"
- **THEN** the hide mode page is displayed

#### Scenario: Return to More settings
- **WHEN** the user navigates back from either subpage
- **THEN** the More settings subpage is displayed

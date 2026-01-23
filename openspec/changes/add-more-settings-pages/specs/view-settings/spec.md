## ADDED Requirements
### Requirement: More settings entry points
The settings view SHALL include a "More" subpage that lists two buttons: "Window Size Adjustment" and "Hide Mode".

#### Scenario: Display more settings actions
- **WHEN** the settings view shows the More subpage
- **THEN** both buttons are visible

### Requirement: More settings subpages
The system SHALL navigate to a dedicated subpage for window size adjustment when the "Window Size Adjustment" button is activated, and to a dedicated subpage for hide mode when the "Hide Mode" button is activated.

#### Scenario: Open window size adjustment page
- **WHEN** the user activates "Window Size Adjustment"
- **THEN** the window size adjustment page is displayed

#### Scenario: Open hide mode page
- **WHEN** the user activates "Hide Mode"
- **THEN** the hide mode page is displayed

#### Scenario: Return to More settings
- **WHEN** the user navigates back from either subpage
- **THEN** the More settings subpage is displayed

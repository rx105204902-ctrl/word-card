## ADDED Requirements
### Requirement: Settings view navigation
The system SHALL provide an icon-only settings entry in the top-right of the main UI that navigates to a settings view. The settings view SHALL include a back button in the top-left that returns to the main UI.

#### Scenario: Open settings view
- **WHEN** the user clicks the settings icon on the main UI
- **THEN** the settings view is displayed

#### Scenario: Return to main view
- **WHEN** the user clicks the back button in the settings view
- **THEN** the main UI is displayed

### Requirement: Settings view left navigation
The settings view SHALL include a compact left-side navigation with three items: Word Bank, Fuzzy Words, Study Calendar.

#### Scenario: Display settings navigation
- **WHEN** the settings view is displayed
- **THEN** the left navigation shows Word Bank, Fuzzy Words, Study Calendar

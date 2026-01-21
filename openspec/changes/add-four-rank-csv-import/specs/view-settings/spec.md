## MODIFIED Requirements
### Requirement: Settings view left navigation
The settings view SHALL include a compact left-side navigation with four items: Word Bank, Fuzzy Words, Study Calendar, More.

#### Scenario: Display settings navigation
- **WHEN** the settings view is displayed
- **THEN** the left navigation shows Word Bank, Fuzzy Words, Study Calendar, and More.

## ADDED Requirements
### Requirement: CSV import entry in More
The settings view SHALL provide an import control within the More section that lets the user choose a CSV file in the four-rank format and triggers an import into the local word bank.

#### Scenario: Trigger import flow
- **WHEN** the user is in the More section and clicks Import
- **THEN** a file chooser is presented and the selected CSV is submitted for import.

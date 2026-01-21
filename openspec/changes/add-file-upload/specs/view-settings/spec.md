## MODIFIED Requirements
### Requirement: Settings view left navigation
The settings view SHALL include a compact left-side navigation with five items: Word Bank, Fuzzy Words, Study Calendar, Import, More.

#### Scenario: Display settings navigation
- **WHEN** the settings view is displayed
- **THEN** the left navigation shows Word Bank, Fuzzy Words, Study Calendar, Import, and More.

## ADDED Requirements
### Requirement: Import entry navigation only
The settings view SHALL provide an Import entry that switches to the import page without opening the system file picker automatically.

#### Scenario: Enter import page
- **WHEN** the user clicks Import in the settings navigation
- **THEN** the import page is displayed and no file picker is shown.

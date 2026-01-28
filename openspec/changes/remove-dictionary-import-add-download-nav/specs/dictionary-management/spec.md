## ADDED Requirements
### Requirement: Dictionary download navigation
The system SHALL provide a navigation entry for dictionary downloads.

#### Scenario: User opens download navigation
- **WHEN** the user opens the navigation menu
- **THEN** a dictionary download entry is visible

### Requirement: Dictionary download links
The system SHALL display explicit download links for CET4 and CET6 dictionaries.

#### Scenario: User views download links
- **WHEN** the user opens the dictionary download entry
- **THEN** CET4 and CET6 links are shown and include the configured URLs

## REMOVED Requirements
### Requirement: Default dictionary data
The system SHALL preload a default dictionary dataset on first run.

#### Scenario: Default data seeding
- **WHEN** the application initializes storage for a new user
- **THEN** a built-in dictionary is automatically loaded

### Requirement: Dictionary import
The system SHALL allow importing dictionaries via file upload.

#### Scenario: Import dictionary file
- **WHEN** the user uploads a dictionary CSV file
- **THEN** the dictionary is imported into the word list

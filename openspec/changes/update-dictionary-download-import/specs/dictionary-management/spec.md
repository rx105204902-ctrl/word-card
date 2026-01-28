## ADDED Requirements
### Requirement: Dictionary download action
The system SHALL provide download actions for CET4 and CET6 dictionaries that import into local word lists.

#### Scenario: User downloads CET4
- **WHEN** the user clicks the CET4 download button
- **THEN** the CET4 CSV is downloaded and imported into a word list named "CET4"

### Requirement: Downloaded state indicator
The system SHALL indicate a dictionary as downloaded when a word list with the same name exists.

#### Scenario: Word list exists
- **WHEN** a word list named "CET6" exists
- **THEN** the CET6 entry is shown as downloaded

### Requirement: Re-download overwrite
The system SHALL allow re-downloading a dictionary to overwrite the existing word list content.

#### Scenario: Re-download CET6
- **WHEN** the user clicks the CET6 download button and a CET6 word list already exists
- **THEN** the CET6 word list content is replaced by the downloaded dictionary

## MODIFIED Requirements
### Requirement: Dictionary download links
The system SHALL display named dictionary entries without exposing raw URLs.

#### Scenario: User views download list
- **WHEN** the user opens the dictionary download section
- **THEN** only dictionary names and action buttons are displayed

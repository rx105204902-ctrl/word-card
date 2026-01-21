## ADDED Requirements
### Requirement: Import target selection
The import flow SHALL require the user to select an existing word list or create a new word list by name before import begins.

#### Scenario: Choose existing list
- **WHEN** the user clicks Import with at least one uploaded file
- **THEN** a word list selection dialog is shown and the user can select an existing list to proceed.

#### Scenario: Create new list
- **WHEN** the user chooses to create a new list and provides a name
- **THEN** the system creates the list and uses it as the import target.

### Requirement: Import associates words to lists
The system SHALL associate each imported word with the selected word list without removing existing list associations.

#### Scenario: Preserve existing list links
- **WHEN** a word is already linked to other lists and is imported again to a new list
- **THEN** the new association is added and existing associations remain.

## MODIFIED Requirements
### Requirement: Four-rank CSV format
The system SHALL accept CSV data whose header matches: `word`, `phonetic`, `part_of_speech_and_meanings`, `example_sentence`, `example_translation`, `audio_uk`, `audio_us`. The system SHALL reject imports with invalid headers regardless of import entry point.

#### Scenario: Valid CSV provided
- **WHEN** the user imports CSV data with the expected header and at least one row
- **THEN** each row is parsed into a word entry and persisted to the local word bank.

#### Scenario: Invalid header provided
- **WHEN** the user imports CSV data with a missing or mismatched header
- **THEN** the import is rejected with a clear error message and no rows are persisted.

### Requirement: Idempotent upsert behavior
The system SHALL treat `word` as the unique key and update the existing record with imported field values. Duplicate imports SHALL NOT create extra rows.

#### Scenario: Re-import same dataset
- **WHEN** the same CSV is imported multiple times
- **THEN** the number of distinct words does not grow and existing fields reflect the latest import data.

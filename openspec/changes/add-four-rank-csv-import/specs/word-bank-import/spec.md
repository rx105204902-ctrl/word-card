## ADDED Requirements
### Requirement: Four-rank CSV format
The system SHALL accept CSV data whose header matches: `word`, `phonetic`, `part_of_speech_and_meanings`, `example_sentence`, `example_translation`, `audio_uk`, `audio_us`.

#### Scenario: Valid CSV provided
- **WHEN** the user imports CSV data with the expected header and at least one row
- **THEN** each row is parsed into a word entry and persisted to the local word bank.

#### Scenario: Invalid header provided
- **WHEN** the user imports CSV data with a missing or mismatched header
- **THEN** the import is rejected with a clear error message and no rows are persisted.

### Requirement: Local SQLite3 persistence
The system SHALL store imported word entries in a local SQLite3 database under the app's data directory.

#### Scenario: App restart
- **WHEN** the app restarts after a successful import
- **THEN** previously imported entries remain stored in the local database.

### Requirement: Idempotent upsert behavior
The system SHALL treat `word` as the unique key and upsert entries on import.

#### Scenario: Re-import same dataset
- **WHEN** the same CSV is imported multiple times
- **THEN** the number of distinct words does not grow due to duplicates.


## ADDED Requirements
### Requirement: Fuzzy mark tracking
The system SHALL persist a fuzzy mark state and the time it was marked for each word.

#### Scenario: Mark word as fuzzy
- **WHEN** a word is marked as fuzzy
- **THEN** the system stores the fuzzy mark state and timestamp for that word

### Requirement: System-managed fuzzy word list
The system SHALL maintain a system-managed word list named "Ä£ºý´Ê´Ê¿â" that includes all fuzzy-marked words, and users SHALL NOT be able to delete it manually. The system SHALL remove the list when no fuzzy words remain.

#### Scenario: Create fuzzy word list
- **WHEN** the first word is marked as fuzzy
- **THEN** the system ensures the "Ä£ºý´Ê´Ê¿â" list exists and includes the word

#### Scenario: Remove fuzzy word list
- **WHEN** the last fuzzy mark is removed
- **THEN** the system removes the "Ä£ºý´Ê´Ê¿â" list

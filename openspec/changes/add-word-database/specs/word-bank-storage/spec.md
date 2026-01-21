## ADDED Requirements
### Requirement: Word list table
The system SHALL persist word list metadata in a local SQLite3 table named `word_list` with columns: `id`, `name`, `created_at`.
The system SHALL enforce unique `name` values.

#### Scenario: Schema migration executes
- **WHEN** the database schema migration runs
- **THEN** the `word_list` table exists with the required columns and unique `name` constraint.

### Requirement: Word table
The system SHALL persist words in a local SQLite3 table named `word` with columns: `id`, `word`, `phonetic`, `part_of_speech_and_meanings`, `example_sentence`, `example_translation`, `audio_uk`, `audio_us`, `created_at`.
The system SHALL enforce uniqueness on `word`.

#### Scenario: Duplicate word rejected
- **WHEN** the same `word` value is inserted more than once
- **THEN** the database rejects the duplicate and only one record exists for that word.

### Requirement: Word list mapping table
The system SHALL persist list membership in a local SQLite3 table named `word_list_map` with columns: `word_list_id`, `word_id`.
The system SHALL enforce referential integrity to `word_list.id` and `word.id`, and unique pairs of (`word_list_id`, `word_id`).

#### Scenario: Duplicate mapping rejected
- **WHEN** the same (`word_list_id`, `word_id`) pair is inserted more than once
- **THEN** the database rejects the duplicate mapping.

#### Scenario: Invalid references rejected
- **WHEN** a mapping references a non-existent `word_list` or `word`
- **THEN** the database rejects the mapping.

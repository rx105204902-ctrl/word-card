## MODIFIED Requirements
### Requirement: Delete word list
The system SHALL remove a word list and its word mappings, and delete words that are not referenced by any remaining word list.

#### Scenario: Delete list with shared words
- **WHEN** a word list is deleted and some words are linked to other lists
- **THEN** only the word list mappings are removed and shared words are preserved

#### Scenario: Delete list with orphaned words
- **WHEN** a word list is deleted and some words are not linked to any other list
- **THEN** those orphaned words and their learning records are removed

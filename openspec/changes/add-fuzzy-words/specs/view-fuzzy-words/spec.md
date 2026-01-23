## ADDED Requirements
### Requirement: Fuzzy words subpage list
The system SHALL provide a Fuzzy Words subpage under Settings that lists words marked as fuzzy, showing each word and its meaning.

#### Scenario: Open fuzzy words list
- **WHEN** the user navigates to the Fuzzy Words subpage
- **THEN** the system displays the list of fuzzy-marked words with their meanings

### Requirement: Fuzzy words sorting
The system SHALL allow sorting the fuzzy words list by marked time or alphabetical order.

#### Scenario: Change sort order
- **WHEN** the user switches the sort option
- **THEN** the list order updates based on the selected criterion

### Requirement: Remove fuzzy marks in bulk
The system SHALL allow users to select one or more fuzzy words and remove the fuzzy mark from the selection.

#### Scenario: Remove fuzzy marks
- **WHEN** the user removes the fuzzy mark for selected words
- **THEN** those words are removed from the fuzzy words list

### Requirement: Fuzzy word detail view
The system SHALL open a detail view when the user selects a word, showing its meaning, example sentence, and example translation.

#### Scenario: Open word detail
- **WHEN** the user clicks a word in the fuzzy words list
- **THEN** the system displays the word detail view with meaning and example content

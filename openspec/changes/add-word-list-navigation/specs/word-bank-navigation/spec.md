## ADDED Requirements
### Requirement: Word list navigation cards
The settings view SHALL display a word list navigation area that renders each word list as a card with the list name and word count. Each card SHALL provide a use button.

#### Scenario: Show word list cards
- **WHEN** the user opens the word bank section
- **THEN** each word list is shown as a card with its name, word count, and a use button.

### Requirement: Single active word list
The system SHALL allow only one word list to be active at a time. Activating a list SHALL deactivate the previously active list.

#### Scenario: Switch active list
- **WHEN** the user clicks use on a different word list
- **THEN** the newly selected list becomes active and the previous list is no longer active.

### Requirement: Active list ordering and state
The active word list SHALL be sorted to the first position in the navigation. The active list's use button SHALL be disabled and display "Active".

#### Scenario: Highlight active list
- **WHEN** a word list is active
- **THEN** the list appears first and its use button is disabled and labeled "Active".

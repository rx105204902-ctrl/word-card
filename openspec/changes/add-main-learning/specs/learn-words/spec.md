## ADDED Requirements
### Requirement: Learning session allocation
The system SHALL allocate a learning session of exactly 50 words whenever the user enters learning mode.
The allocation SHALL include 20 words never learned before, 20 words with proficiency_score < 4, 6 words with proficiency_score between 4 and 8 inclusive, and 4 words with proficiency_score between 8 and 10 inclusive.
If any bucket lacks enough words, the system SHALL fill the remaining slots from other buckets while still totaling 50 words.

#### Scenario: Allocate session with shortages
- **WHEN** the user enters learning mode and a bucket has fewer words than required
- **THEN** the system allocates 50 total words by filling the remaining slots from other buckets.

### Requirement: Random word navigation with history stack
The home screen SHALL display words in random order within the current learning session.
The Next action SHALL show a new random word from the remaining session set and push the current word onto a history stack.
The Previous action SHALL return the most recently learned word from the history stack.

#### Scenario: Navigate to previous word
- **WHEN** the user presses Previous after learning at least one word
- **THEN** the system shows the immediately prior word from the history stack.

### Requirement: Learning progress tracking
The system SHALL persist per-word learning state in a user_word_learning table with columns: word_id, proficiency_score, last_learned_at, learn_count.
The system SHALL initialize missing rows with proficiency_score = 0 and learn_count = 0.

#### Scenario: Record first learning event
- **WHEN** a word is learned for the first time
- **THEN** a user_word_learning row exists with the required columns and default values before updates are applied.

### Requirement: Proficiency updates
When the user presses Next to advance after a normal learn, the current word's proficiency_score SHALL increase by 1 up to a maximum of 10 and learn_count SHALL increment by 1.
When the user presses Fuzzy, the current word's proficiency_score SHALL decrease by 1 down to a minimum of 0.

#### Scenario: Increment proficiency on next
- **WHEN** the user presses Next on a word with proficiency_score 9
- **THEN** the proficiency_score becomes 10 and does not exceed 10.

#### Scenario: Decrement proficiency on fuzzy
- **WHEN** the user presses Fuzzy on a word with proficiency_score 0
- **THEN** the proficiency_score remains 0.

### Requirement: Proficiency indicator
The home screen SHALL display a small square indicator at the top-left showing the current word's proficiency tier.
The indicator SHALL map score tiers as 0-1, 2-3, 4-5, 6-7, and 8-10 and render a color gradient from green (low) to blue (high).

#### Scenario: Show tiered indicator
- **WHEN** the current word has proficiency_score 5
- **THEN** the indicator displays the 4-5 tier with a mid-range green-to-blue color.

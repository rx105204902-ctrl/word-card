## ADDED Requirements
### Requirement: Initial UI shell
The app SHALL render a focused word-card UI shell that includes a header, the active word area, and review controls.

#### Scenario: Default launch
- **WHEN** the app launches
- **THEN** the user sees a word-card UI shell with a header, word display, and review action placeholders.

### Requirement: Responsive layout baseline
The UI SHALL adapt to small widths without clipping core content.

#### Scenario: Narrow width
- **WHEN** the window width is less than or equal to 360px
- **THEN** the word-card layout stacks vertically and remains readable.

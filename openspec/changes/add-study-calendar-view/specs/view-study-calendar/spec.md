## ADDED Requirements
### Requirement: Study calendar subpage
The settings view SHALL display a Study Calendar subpage when the user selects Study Calendar in the left navigation.

#### Scenario: Open Study Calendar
- **WHEN** the user selects Study Calendar in the settings navigation
- **THEN** the Study Calendar subpage is displayed

### Requirement: Study calendar view toggle
The Study Calendar subpage SHALL provide a view toggle control in the top-right that switches between Calendar and Line Chart views.

#### Scenario: Switch study calendar views
- **WHEN** the user uses the view toggle control
- **THEN** the Study Calendar view switches between Calendar and Line Chart

### Requirement: Calendar view daily highlights
In Calendar view, the system SHALL highlight date cells with learning records using a light green background and SHALL show the day's studied word count in the bottom-left of each date cell.

#### Scenario: Display a day with study activity
- **WHEN** the calendar shows a date that has learning records
- **THEN** the date cell is light green and shows the day's word count in the bottom-left

### Requirement: Line chart view daily trend
In Line Chart view, the system SHALL render a line chart of daily studied word counts with date on the x-axis and word count on the y-axis; the y-axis scale SHALL adjust automatically to the data range.

#### Scenario: Render daily study trend
- **WHEN** the Study Calendar is set to Line Chart view
- **THEN** the line chart shows daily word counts with an auto-scaled y-axis

### Requirement: Study calendar history navigation
The Study Calendar SHALL allow users to navigate to historical months and dates, with the earliest accessible month being January 2025.

#### Scenario: Navigate to historical months
- **WHEN** the user navigates backward in the Study Calendar
- **THEN** the calendar can reach January 2025 and SHALL NOT navigate earlier

### Requirement: Line chart hover tag
In Line Chart view, the system SHALL display a hover tag showing the corresponding daily word count when the user hovers the line.

#### Scenario: Hover line chart data point
- **WHEN** the user hovers the line chart
- **THEN** the tag displays the corresponding data point value

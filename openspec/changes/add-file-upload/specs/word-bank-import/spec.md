## ADDED Requirements
### Requirement: Import page upload entry
The import page SHALL display a centered "Upload File" button. The system SHALL open the file picker only when the user clicks this button.

#### Scenario: Enter import page
- **WHEN** the user navigates to the import page
- **THEN** the Upload File button is visible and no file picker is shown automatically.

#### Scenario: Choose files
- **WHEN** the user clicks Upload File
- **THEN** the system file picker is displayed.

### Requirement: File size bounds
The system SHALL accept files between 10 MB and 100 MB inclusive. Files outside this range SHALL be rejected with a clear message.

#### Scenario: File too small
- **WHEN** the user selects a file smaller than 10 MB
- **THEN** the file is rejected and a size error is shown.

#### Scenario: File too large
- **WHEN** the user selects a file larger than 100 MB
- **THEN** the file is rejected and a size error is shown.

### Requirement: Hash-based chunked upload
The system SHALL split accepted files into chunks and compute a hash per chunk to identify uploads. The upload progress SHALL reflect completed chunks for each file.

#### Scenario: Upload large file
- **WHEN** the user selects a valid file for upload
- **THEN** the file is uploaded in hashed chunks and progress advances as chunks complete.

### Requirement: Per-file progress and cancel
The import page SHALL show real-time progress for each uploading file and allow canceling an individual upload.

#### Scenario: Cancel upload
- **WHEN** the user cancels an in-progress file upload
- **THEN** the upload stops and the file is marked as canceled.

### Requirement: Uploaded file list and delete
After a file upload completes, the system SHALL add it to the import list. The list SHALL allow deleting individual files.

#### Scenario: Delete file from list
- **WHEN** the user deletes a file from the import list
- **THEN** the file is removed from the list and will not be imported.

### Requirement: Fixed footer actions
The import page SHALL keep a fixed footer with Continue Upload and Import actions. Continue Upload SHALL open the file picker and append selected files to the list. Import SHALL confirm the current list and trigger the import action.

#### Scenario: Continue upload
- **WHEN** the user clicks Continue Upload
- **THEN** the file picker opens and newly selected files are appended to the list.

#### Scenario: Confirm import
- **WHEN** the user clicks Import with at least one uploaded file
- **THEN** the system starts the import operation for the current list.

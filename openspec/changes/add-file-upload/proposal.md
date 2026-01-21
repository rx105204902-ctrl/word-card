# Change: Add chunked file upload import flow

## Why
Users need to import large files without being forced into an immediate file picker. The flow should provide explicit control, show real-time progress, and allow canceling or managing individual files before confirming import.

## What Changes
- Add an import page entry that only switches the view (no automatic file picker).
- Show a centered "Upload File" button on the import page; file selection is user-initiated.
- Support 10-100 MB files via hash-based chunked upload.
- Show per-file real-time upload progress and allow canceling an individual upload.
- Show an uploaded file list with per-file delete.
- Add fixed footer actions: Continue Upload (append files) and Import (confirm list).

## Impact
- Affected specs: view-settings (modified), word-bank-import (modified)
- Affected code: src/App.vue, src-tauri/src/lib.rs, src-tauri/src/word_bank.rs

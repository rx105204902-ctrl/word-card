## Context
The app is a local Tauri + Vue desktop UI. Import currently triggers a file picker immediately. The new flow needs a dedicated import page with large-file support, chunked uploads, progress, and cancellation.

## Goals / Non-Goals
- Goals: Explicit user-initiated file selection, chunked upload for 10-100 MB files, per-file progress and cancel, confirm-before-import.
- Non-Goals: Network upload, remote storage, background resume across restarts.

## Decisions
- Decision: The import entry only switches the view; file selection is triggered by explicit user actions (Upload File / Continue Upload).
- Decision: Use hash-based chunking to identify chunks and track progress, with chunk assembly into a temporary file before import.
- Decision: Enforce the 10-100 MB size bound in both UI selection and backend acceptance.

## Risks / Trade-offs
- Large file IO can impact UI responsiveness if chunk size or concurrency is too aggressive.
- Hash computation strategy needs to balance correctness and performance on 100 MB files.

## Migration Plan
No data migration required. The feature is additive and gated by the import page.

## Open Questions
- Chunk size default and concurrency level.
- Hash algorithm and implementation location (frontend vs Rust) to be confirmed after documentation review.

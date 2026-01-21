## 1. Implementation
- [x] 1.1 Define SQLite schema for imported word entries
- [x] 1.2 Implement `import_four_rank_csv` Tauri command (parse CSV + upsert into SQLite)
- [x] 1.3 Add Settings > More import button + file picker UI
- [x] 1.4 Display import progress and result summary in UI

## 2. Validation
- [ ] 2.1 Manually verify importing `src-tauri/file/four_rank.csv` reports success
- [ ] 2.2 Manually verify invalid CSV header fails without partial writes

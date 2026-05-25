## 1. Verify Existing Post-Processing Path

- [x] 1.1 Inspect schedule run completion reconciliation for output asset links, album membership, tag application and run output upsert.
- [x] 1.2 Confirm implementation uses existing core album/tag/run-output APIs and remains idempotent.

## 2. Strengthen Tests

- [x] 2.1 Add daemon scheduler test coverage that verifies scheduled run-now output asset is added to the target album.
- [x] 2.2 Extend the test to verify job tags are visible as canonical asset tags.
- [x] 2.3 Re-run schedule reconciliation for the same completed task and verify counters remain stable.

## 3. Verification

- [x] 3.1 Run `openspec validate fix-scheduled-output-post-processing --strict`.
- [x] 3.2 Run `cargo test -p imglab-daemon`.
- [x] 3.3 Run `cargo test -p imglab-core`.

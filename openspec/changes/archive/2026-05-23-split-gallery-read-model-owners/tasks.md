# Tasks: Split Gallery Read Model Owners

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm the change preserves persistence and runtime contracts.

## 2. Search Read Model Extraction

- [x] 2.1 Move search-specific filtering and result mapping out of `library/gallery.rs`.
- [x] 2.2 Keep shared gallery card loading behavior reusable without changing query semantics.
- [x] 2.3 Verify existing search tests still cover the extracted owner.

## 3. Documentation

- [x] 3.1 Update architecture inventory with the new read-model owner.
- [x] 3.2 Record remaining gallery read-model split targets.

## 4. Verification

- [x] 4.1 Run Rust core and CLI tests.
- [x] 4.2 Run architecture guardrails.
- [x] 4.3 Run OpenSpec validation.

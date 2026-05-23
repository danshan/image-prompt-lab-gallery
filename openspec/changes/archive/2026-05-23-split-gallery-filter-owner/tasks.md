# Tasks: Split Gallery Filter Owner

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm the change preserves persistence and runtime contracts.

## 2. Filter Owner Extraction

- [x] 2.1 Move album filter context loading and validation out of `library/gallery.rs`.
- [x] 2.2 Move shared gallery predicate and smart album preview filtering to the new owner.
- [x] 2.3 Move gallery sort behavior that depends on album context to the new owner.
- [x] 2.4 Keep Gallery query and smart album behavior unchanged.

## 3. Documentation

- [x] 3.1 Update architecture inventory with the new filtering owner.
- [x] 3.2 Record remaining gallery read-model split targets.

## 4. Verification

- [x] 4.1 Run Rust core tests.
- [x] 4.2 Run architecture guardrails.
- [x] 4.3 Run OpenSpec validation.

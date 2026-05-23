# Tasks: Split Gallery Version Tree Owner

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm the change preserves persistence and runtime contracts.

## 2. Version Tree Extraction

- [x] 2.1 Move version tree row loading and model construction out of `library/gallery.rs`.
- [x] 2.2 Move promoted-source and asset-scoped lineage read logic to the version tree owner.
- [x] 2.3 Keep `GalleryReadService` detail and list outputs behaviorally unchanged.

## 3. Documentation

- [x] 3.1 Update architecture inventory with the new read-model owner.
- [x] 3.2 Record remaining gallery read-model split targets.

## 4. Verification

- [x] 4.1 Run Rust core tests.
- [x] 4.2 Run architecture guardrails.
- [x] 4.3 Run OpenSpec validation.

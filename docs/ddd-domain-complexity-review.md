# DDD Domain Complexity Review

本文档记录 `refactor-core-ddd-boundaries` 当前 domain migration 阶段的 complexity review。目标是确保 DDD 分层不是简单移动文件, 而是在每个 bounded context 内保留可复用逻辑, 低冗余和低圈复杂度。

## Shared

- Owner: `crates/imglab-core/src/domain/shared`.
- Current scope: identity value objects.
- Complexity notes: 仅包含 ID newtype, 无业务分支.
- Deferred work: 后续可加入更强的 validated IDs, 但本 change 当前保持 public constructor 兼容.

## Library

- Owner: `crates/imglab-core/src/domain/library`.
- Extracted logic: `LibraryManifest`, `ResourceLibrary`, `RegistryAlias`, schema compatibility, manifest-to-summary mapping.
- Reuse point: `RegistryAlias::parse`, `ensure_schema_supported`, `summary_from_manifest`.
- Complexity notes: schema compatibility 和 alias validation 已从 IO/SQLite path 中抽出, 分支简单.
- Deferred work: library layout validation 仍在 infrastructure-like `library/service.rs`, 后续 infrastructure migration 时迁入 filesystem adapter.

## Asset

- Owner: `crates/imglab-core/src/domain/asset`.
- Extracted logic: `Asset`, `AssetVersion`, `ReferenceSourceKind`, version naming, next version number, same-asset parent validation, reference source classification.
- Reuse point: `version_name`, `next_version_number`, `classify_reference_source`, `ensure_same_asset_parent`.
- Complexity notes: lineage/reference source rule 已集中, 不再需要在多个 generation/import use case 中复制判断.
- Deferred work: current write path 仍由 SQLite helper 分配 version number; application use case migration 会改为先调用 domain policy.

## Generation

- Owner: `crates/imglab-core/src/domain/generation`.
- Extracted logic: provider normalization, default model label, operation inference, operation storage mapping, default provider capability.
- Reuse point: `normalize_provider_name`, `infer_generation_operation`, `operation_to_str`, `operation_from_str`.
- Complexity notes: request planning 中纯决策逻辑已与 file input loading 分离.
- Deferred work: `GenerateImageUseCase` 仍未迁移, provider execution orchestration 仍在 legacy service.

## Metadata Review

- Owner: `crates/imglab-core/src/domain/metadata_review`.
- Extracted logic: suggestion status constants, pending check, confidence normalization.
- Reuse point: `PENDING_REVIEW_STATUS`, `ACCEPTED_STATUS`, `REJECTED_STATUS`, `is_pending_review`, `normalize_confidence_json`.
- Complexity notes: confidence score parsing已集中, 避免 review detail 和 review service 各自实现 score normalization.
- Deferred work: accept/reject transaction orchestration 仍在 legacy SQLite implementation.

## Album

- Owner: `crates/imglab-core/src/domain/album`.
- Extracted logic: album kind storage mapping, manual/smart guard, smart query field allowlist.
- Reuse point: `album_kind_to_str`, `album_kind_from_str`, `ensure_manual_album_kind`, `ensure_supported_smart_query_field`.
- Complexity notes: smart query parser 仍较集中, 但 allowlist 和 kind policy 已先抽出.
- Deferred work: smart query parser 可在 later application/query migration 中拆成 typed parser helpers.

## Task

- Owner: `crates/imglab-core/src/domain/task`.
- Extracted logic: `TaskType`, `TaskStatus`, `TaskErrorClassification`, `TaskOutputType`, terminal/retryable status policy, auto-retry classification.
- Reuse point: enum parse/as_str, `is_terminal_status`, `is_retryable_status`, `should_auto_retry`.
- Complexity notes: status and retry policy 已从 generic DTO namespace 迁出, 后续 scheduler 可以直接复用 domain policy.
- Deferred work: scheduler selection policy 仍主要在 `task_scheduler.rs`; application task migration 时继续拆分 runnable selection 和 wait reason policy.

## Current Verification

Current focused domain tests are part of `cargo test -p imglab-core`:

```text
domain::library::model::tests::*
domain::asset::policies::tests::*
domain::generation::policies::tests::*
domain::metadata_review::policies::tests::*
domain::album::policies::tests::*
domain::task::policies::tests::*
```

Latest checked result during this stage:

```text
cargo fmt --all --check
passed

scripts/check-architecture.sh
passed

cargo test -p imglab-core
passed: 78 passed, 0 failed, 1 ignored
```

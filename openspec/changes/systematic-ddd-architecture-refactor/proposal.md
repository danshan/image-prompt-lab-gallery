# Proposal: Systematic DDD Architecture Refactor

## Problem

The project has already moved toward a DDD architecture, but current evidence shows several transitional boundaries remain. Legacy `library/*` services, application use cases, runtime adapters, daemon scheduling, desktop workflow controllers, and query/read-model implementation still overlap in ways that make long-term evolution risky.

The project also lacks a systematic, evidence-driven decision gate for persistence and search architecture. SQLite is the current local-first baseline, but gallery, smart album, version tree, search, and task queue workloads need measured validation before the project decides whether to stay with tuned SQLite, add FTS5/projection tables, introduce an embedded search index such as Tantivy, add a DuckDB analytical sidecar, or plan a future PostgreSQL path.

## Goals

- Turn the systematic DDD code review into a staged, verifiable refactor backlog.
- Preserve current public behavior unless a spec explicitly defines a behavior change.
- Consolidate business rule ownership in domain/application boundaries.
- Evaluate persistence/search options with workload evidence before implementation.
- Reduce large-owner and long-method risk by splitting by ownership and change reason.
- Strengthen tests and architecture guardrails.

## Non-Goals

- Do not redesign desktop visuals.
- Do not implement multi-user collaboration, cloud sync, encryption, or remote service behavior.
- Do not mandate PostgreSQL or any DB replacement without workload evidence.
- Do not change resource library schema or file layout without migration and rollback requirements.

## Impact

- Core DDD specs will define stronger primary-owner and runtime-bypass rules.
- Performance/code-health specs will require systematic review evidence and hotspot tracking.
- Resource library specs will define persistence/search decision gates and compatibility requirements.
- Task manager specs will clarify transition and scheduler ownership.
- Desktop specs will clarify workflow ownership and refresh/polling policy.

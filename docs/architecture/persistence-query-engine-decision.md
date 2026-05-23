# Persistence and Query Engine Decision

## Purpose

This document records the first workload evidence for the `systematic-ddd-architecture-refactor` persistence/search decision gate. It does not replace product-level performance testing, but it gives the project a repeatable baseline before changing database architecture.

## Decision Summary

Keep SQLite as the authoritative resource library store for the next implementation wave. Do not introduce PostgreSQL, DuckDB, Tantivy, or another mandatory database in the core write path yet.

The immediate next step should be SQLite read-model hardening:

1. Keep `library.sqlite` as the transactional source of truth.
2. Add or tune SQLite indexes and query shape where the workload smoke shows full scans or temporary sort structures.
3. Treat FTS5 and projection tables as the first supplemental candidates if real gallery/search workloads exceed tuned SQLite.
4. Treat Tantivy as a rebuildable search sidecar candidate only if FTS5/projection tables do not satisfy faceted/text search.
5. Treat DuckDB as an analytical sidecar only for reporting workloads.
6. Treat PostgreSQL as a future remote/multi-user architecture option, not a local-first desktop dependency.

## Workload Smoke Fixture

The baseline script is:

```bash
scripts/sqlite-workload-smoke.sh
```

It creates a temporary SQLite database with these tables and indexes aligned to the current resource library query concerns:

- `assets`
- `asset_versions`
- `metadata_suggestions`
- `tags`
- `asset_tags`
- `albums`
- `album_items`
- `tasks`
- `task_outputs`

It inserts synthetic assets, versions, tags, album memberships, metadata suggestions, and tasks. It then runs query plans and timed queries for:

- gallery latest page,
- tag filtering,
- smart-album-like text/rating filtering,
- version tree summary,
- task queue listing.

The script defaults to:

```text
assets=1000
versions_per_asset=3
tags=25
tasks=500
```

The scale can be changed with:

```bash
IMGLAB_WORKLOAD_ASSETS=10000 \
IMGLAB_WORKLOAD_VERSIONS_PER_ASSET=3 \
IMGLAB_WORKLOAD_TASKS=3000 \
scripts/sqlite-workload-smoke.sh /tmp/imglab-workload-smoke-10k.sqlite
```

## Baseline Results

### Rust Read-Model Checkpoint

Command:

```bash
cargo test -p imglab-core synthetic_gallery_search_checkpoint_10k_assets -- --ignored --nocapture
```

Observed on this machine:

```text
synthetic seed 10k assets: 654.425542ms
synthetic gallery 10k assets: 3.853228291s
synthetic search 10k assets: 4.045591792s
synthetic album order 5k assets: 3.606798167s
```

Interpretation: SQLite itself is still viable, but Rust read-model assembly is the real next hardening target. The selected path remains SQLite authoritative storage plus query-shape/read-model refactors before introducing a supplemental engine.

### 1k Asset Smoke

Command:

```bash
bash scripts/sqlite-workload-smoke.sh /tmp/imglab-workload-smoke.sqlite
```

Fixture counts:

```text
assets=1000
versions=3000
asset_tags=1000
album_items=500
metadata_suggestions=200
tasks=500
```

Observed query notes:

- Gallery latest page returned 100 rows in approximately `0.003s`.
- Tag filter returned 40 rows in approximately `0.000s`.
- Smart-album-like text/rating filter returned 100 rows in approximately `0.000s`.
- Version tree summary returned 1000 grouped rows in approximately `0.002s`.
- Task queue returned 100 rows in approximately `0.000s`.

Important query plan signals:

- Gallery latest page uses `idx_assets_library_id`, `idx_asset_versions_asset_created`, a correlated subquery, and a temporary B-tree for ordering.
- Tag filter uses tag name lookup and existing asset/tag indexes, but still sorts through a temporary B-tree.
- Smart-album-like text/rating filtering uses `idx_assets_library_id` and a temporary B-tree for ordering.
- Version tree summary scans `asset_versions` through `idx_asset_versions_asset_created`.
- Task queue uses `idx_tasks_library_status_order`, but the `IN` status query still needs a temporary B-tree for ordering.

### 10k Asset Smoke

Command:

```bash
IMGLAB_WORKLOAD_ASSETS=10000 \
IMGLAB_WORKLOAD_VERSIONS_PER_ASSET=3 \
IMGLAB_WORKLOAD_TASKS=3000 \
bash scripts/sqlite-workload-smoke.sh /tmp/imglab-workload-smoke-10k.sqlite
```

Fixture counts:

```text
assets=10000
versions=30000
asset_tags=10000
album_items=5000
metadata_suggestions=2000
tasks=3000
```

Observed query notes:

- Gallery latest page returned 100 rows in approximately `0.030s`.
- Tag filter returned 100 rows in approximately `0.003s`.
- Smart-album-like text/rating filter returned 100 rows in approximately `0.001s`.
- Version tree summary returned 10000 grouped rows in approximately `0.014s`.
- Task queue returned 100 rows in approximately `0.000s`.

Important query plan signals:

- Gallery latest page still uses a correlated version lookup and temporary B-tree sort.
- Version tree summary is an all-version index scan.
- Task queue uses the existing task status/order index, but status `IN` still results in a temporary sort.

## Interpretation

The smoke results do not justify replacing SQLite as the authoritative local-first store. At 10k assets and 30k versions, the synthetic baseline is still fast on this machine. However, the query plans identify where future real workloads can degrade:

- latest-version gallery queries depend on correlated subqueries and sorting,
- gallery and task paths use temporary B-trees,
- version tree summaries scan all version rows,
- text search is still simple `LIKE` behavior, not a true search index,
- the smoke does not yet exercise image dimensions, file IO, concurrent writes, or full Rust read-model mapping cost.

This means the next implementation should focus on read-model ownership and query-shape hardening before adding another database.

## Option Comparison

### SQLite Tuning

Recommended as the next step.

Use when:

- query plan issues are limited to missing composite indexes, query shape, pagination, or avoidable full scans,
- the resource library must remain portable,
- all data should remain inside `library.sqlite`.

Implementation candidates:

- latest-version projection or query rewrite,
- composite indexes for gallery sort/filter combinations,
- task queue query rewrite for status-specific ordering,
- pagination before full map assembly,
- query-count tests and smoke fixture in CI or release checks.

### SQLite FTS5 / Projection Tables

Recommended as the first supplemental option if tuned SQLite is not enough.

Use when:

- title/description/schema prompt text search becomes a real bottleneck,
- smart album filters need durable query acceleration,
- gallery cards need stable precomputed read models.

Required constraints:

- projection tables must be rebuildable from authoritative tables,
- repair must know how to rebuild projections,
- backup/restore must not depend on stale projection state,
- migrations must include backfill and rollback behavior.

### Tantivy

Defer until FTS5/projection tables are proven insufficient.

Use when:

- faceted search, fuzzy search, tokenization, or ranking needs exceed SQLite ergonomics,
- index rebuild cost is acceptable,
- eventual consistency can be made explicit.

Required constraints:

- SQLite remains authoritative,
- Tantivy index is rebuildable,
- backup/restore can omit or rebuild the index,
- corruption handling is explicit.

### DuckDB

Defer to analytical/reporting workflows.

Use when:

- the product needs aggregate-heavy reports,
- query patterns are analytical rather than high-frequency transactional UI reads.

Not recommended for:

- the primary asset/version/task metadata write path.

### PostgreSQL

Defer to a future multi-user or remote-service architecture.

Use when:

- the project intentionally leaves single-user local-first portability,
- remote service, collaboration, or cloud sync becomes a primary product goal.

Not recommended for:

- current local-first desktop MVP,
- portable resource library backup/restore semantics.

## Selected Path

For this OpenSpec change, choose:

```text
SQLite authoritative store + workload smoke + query-shape/read-model hardening.
```

Add FTS5/projection design only if later measurement proves tuned SQLite is insufficient. Do not add Tantivy, DuckDB, or PostgreSQL to the runtime dependency graph in this change.

## Migration and Compatibility Constraints

Any future schema/index/projection work must preserve:

- existing library open behavior,
- `manifest.json` identity,
- managed file layout,
- backup/restore clone-on-conflict semantics,
- repair behavior,
- rollback plan for failed migrations.

Supplemental indexes must be treated as rebuildable unless a future spec explicitly upgrades them to authoritative storage.

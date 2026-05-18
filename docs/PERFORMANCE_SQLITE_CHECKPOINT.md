# SQLite Sufficiency Checkpoint

> Change: `refactor-performance-code-health`
> Date: 2026-05-18

## Purpose

This checkpoint keeps storage replacement as an evidence-based follow-up instead of a speculative rewrite. The current change first removes known SQLite misuse:

- Gallery/Search N+1 reads.
- Per-asset album membership and album order queries.
- Missing hot-path indexes.
- Full-file reads for checksum and dimensions.

## Repeatable Synthetic Workload

Run the ignored core test when a local machine can spend time seeding 10k synthetic assets:

```text
cargo test --offline -p imglab-core synthetic_gallery_search_checkpoint_10k_assets -- --ignored --nocapture
```

The test seeds:

- 10,000 assets.
- 10,000 current versions.
- 10,000 generation events.
- Two tags per asset across eight tags.
- Pending metadata suggestions on 20% of assets.
- One manual album containing 5,000 assets.

It reports elapsed time for:

- Default Gallery query over 10k assets.
- Search query with text, provider and tag filters.
- Manual album query with `album_order` sort over 5k assets.

## Current Conclusion

On the current local machine, the synthetic checkpoint produced:

| Operation | Workload | Observed elapsed |
|---|---:|---:|
| Seed synthetic data | 10k assets | 685.433667ms |
| Default Gallery query | 10k assets | 173.880625ms |
| Search query | text + provider + tag over 10k assets | 186.73525ms |
| Manual album order query | 5k album assets | 188.63175ms |

SQLite remains sufficient for the current local-first MVP with the optimized read model. This change should not replace SQLite directly.

Storage replacement should be proposed as a separate OpenSpec change only if repeated checkpoint runs show unacceptable latency after the N+1, index and bounded IO fixes are in place.

## Deferred Follow-ups

- Parallel or incremental integrity hashing.
- Direct album-to-library lookup schema changes.
- Batch tag attach optimization for review acceptance.
- Thumbnail generation for the Gallery grid.
- Broader frontend file-level component split.

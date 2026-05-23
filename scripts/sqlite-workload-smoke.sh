#!/usr/bin/env bash
set -euo pipefail

DB_PATH="${1:-}"
ASSET_COUNT="${IMGLAB_WORKLOAD_ASSETS:-1000}"
VERSIONS_PER_ASSET="${IMGLAB_WORKLOAD_VERSIONS_PER_ASSET:-3}"
TAG_COUNT="${IMGLAB_WORKLOAD_TAGS:-25}"
TASK_COUNT="${IMGLAB_WORKLOAD_TASKS:-500}"

if [[ -z "$DB_PATH" ]]; then
  DB_PATH="$(mktemp -t imglab-workload-XXXXXX.sqlite)"
fi

rm -f "$DB_PATH" "$DB_PATH-wal" "$DB_PATH-shm"

sqlite3 "$DB_PATH" <<SQL
.timer on
.bail on
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE assets (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  media_type TEXT NOT NULL,
  title TEXT,
  description TEXT,
  schema_prompt TEXT,
  category TEXT,
  rating INTEGER,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  captured_at TEXT
);

CREATE TABLE asset_versions (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL,
  parent_version_id TEXT,
  generation_event_id TEXT,
  file_path TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  checksum_algorithm TEXT NOT NULL DEFAULT 'SHA-256',
  checksum TEXT,
  width INTEGER,
  height INTEGER,
  mime_type TEXT NOT NULL,
  version_number INTEGER NOT NULL,
  version_label TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE metadata_suggestions (
  id TEXT PRIMARY KEY,
  asset_id TEXT NOT NULL,
  source TEXT NOT NULL,
  suggested_title TEXT,
  suggested_description TEXT,
  suggested_schema_prompt TEXT,
  suggested_tags_json TEXT NOT NULL,
  suggested_category TEXT,
  confidence_json TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  reviewed_at TEXT
);

CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE asset_tags (
  asset_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  source TEXT NOT NULL,
  confirmed_at TEXT,
  PRIMARY KEY(asset_id, tag_id)
);

CREATE TABLE albums (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  kind TEXT NOT NULL,
  smart_query_json TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE album_items (
  album_id TEXT NOT NULL,
  asset_id TEXT NOT NULL,
  sort_order INTEGER NOT NULL,
  added_at TEXT NOT NULL,
  PRIMARY KEY(album_id, asset_id)
);

CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  task_type TEXT NOT NULL,
  status TEXT NOT NULL,
  queue_position INTEGER NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,
  provider TEXT,
  operation_type TEXT,
  concurrency_group TEXT,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  next_retry_at TEXT,
  input_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_error_code TEXT,
  last_error_message TEXT,
  error_classification TEXT,
  wait_reason TEXT
);

CREATE TABLE task_outputs (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  output_type TEXT NOT NULL,
  target_id TEXT NOT NULL,
  payload_json TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(task_id, output_type, target_id)
);

CREATE INDEX idx_assets_library_id ON assets(library_id);
CREATE INDEX idx_asset_versions_asset_created ON asset_versions(asset_id, created_at DESC, id DESC);
CREATE INDEX idx_metadata_suggestions_asset_status ON metadata_suggestions(asset_id, status);
CREATE INDEX idx_album_items_asset ON album_items(asset_id);
CREATE INDEX idx_album_items_album_sort ON album_items(album_id, sort_order ASC, asset_id);
CREATE INDEX idx_asset_tags_asset ON asset_tags(asset_id);
CREATE INDEX idx_asset_tags_tag ON asset_tags(tag_id);
CREATE INDEX idx_tasks_library_status_order ON tasks(library_id, status, priority DESC, queue_position ASC, created_at ASC);
CREATE INDEX idx_task_outputs_task ON task_outputs(task_id);

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < $ASSET_COUNT
)
INSERT INTO assets (
  id, library_id, media_type, title, description, schema_prompt, category,
  rating, status, created_at, updated_at, captured_at
)
SELECT
  printf('asset-%06d', n),
  'library',
  'image/png',
  printf('Asset %06d', n),
  printf('Description %06d', n),
  printf('Schema prompt %06d', n),
  CASE n % 5
    WHEN 0 THEN 'portrait'
    WHEN 1 THEN 'product'
    WHEN 2 THEN 'landscape'
    WHEN 3 THEN 'reference'
    ELSE 'concept'
  END,
  n % 6,
  CASE n % 7 WHEN 0 THEN 'reference' ELSE 'generated' END,
  printf('2026-05-23T00:%02d:%02dZ', n % 60, n % 60),
  printf('2026-05-23T01:%02d:%02dZ', n % 60, n % 60),
  NULL
FROM seq;

WITH RECURSIVE asset_seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM asset_seq WHERE n < $ASSET_COUNT
),
version_seq(v) AS (
  SELECT 1
  UNION ALL
  SELECT v + 1 FROM version_seq WHERE v < $VERSIONS_PER_ASSET
)
INSERT INTO asset_versions (
  id, asset_id, parent_version_id, generation_event_id, file_path, sha256,
  checksum_algorithm, checksum, width, height, mime_type, version_number,
  version_label, created_at
)
SELECT
  printf('version-%06d-%02d', n, v),
  printf('asset-%06d', n),
  CASE WHEN v = 1 THEN NULL ELSE printf('version-%06d-%02d', n, v - 1) END,
  printf('event-%06d-%02d', n, v),
  printf('originals/generated/asset-%06d-v%02d.png', n, v),
  printf('checksum-%06d-%02d', n, v),
  'SHA-256',
  printf('checksum-%06d-%02d', n, v),
  1024 + (n % 128),
  768 + (v % 64),
  'image/png',
  v,
  printf('v%d', v),
  printf('2026-05-23T02:%02d:%02dZ', n % 60, v % 60)
FROM asset_seq
CROSS JOIN version_seq;

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < $TAG_COUNT
)
INSERT INTO tags (id, name, color, created_at)
SELECT printf('tag-%03d', n), printf('tag-%03d', n), NULL, '2026-05-23T00:00:00Z'
FROM seq;

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < $ASSET_COUNT
)
INSERT INTO asset_tags (asset_id, tag_id, source, confirmed_at)
SELECT
  printf('asset-%06d', n),
  printf('tag-%03d', (n % $TAG_COUNT) + 1),
  'manual',
  '2026-05-23T00:00:00Z'
FROM seq;

INSERT INTO albums (id, name, description, kind, smart_query_json, sort_order, created_at, updated_at)
VALUES
  ('album-manual', 'Manual Album', NULL, 'manual', NULL, 1, '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z'),
  ('album-smart', 'Smart Album', NULL, 'smart', '{"text":"Asset","rating":3}', 2, '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z');

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < ($ASSET_COUNT / 2)
)
INSERT INTO album_items (album_id, asset_id, sort_order, added_at)
SELECT 'album-manual', printf('asset-%06d', n), n, '2026-05-23T00:00:00Z'
FROM seq;

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < ($ASSET_COUNT / 5)
)
INSERT INTO metadata_suggestions (
  id, asset_id, source, suggested_title, suggested_description,
  suggested_schema_prompt, suggested_tags_json, suggested_category,
  confidence_json, status, created_at, reviewed_at
)
SELECT
  printf('suggestion-%06d', n),
  printf('asset-%06d', n),
  'fixture',
  printf('Suggested %06d', n),
  NULL,
  NULL,
  '[]',
  NULL,
  '{}',
  'pending_review',
  '2026-05-23T00:00:00Z',
  NULL
FROM seq;

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < $TASK_COUNT
)
INSERT INTO tasks (
  id, library_id, task_type, status, queue_position, priority,
  provider, operation_type, concurrency_group, attempt_count, max_attempts,
  next_retry_at, input_json, created_at, updated_at,
  last_error_code, last_error_message, error_classification, wait_reason
)
SELECT
  printf('task-%06d', n),
  'library',
  CASE n % 3 WHEN 0 THEN 'metadata_suggestion_generation' ELSE 'image_generation' END,
  CASE n % 5 WHEN 0 THEN 'running' WHEN 1 THEN 'retry_waiting' ELSE 'queued' END,
  n,
  n % 10,
  CASE n % 3 WHEN 0 THEN NULL ELSE 'fake' END,
  CASE n % 2 WHEN 0 THEN 'text_to_image' ELSE 'image_to_image' END,
  CASE n % 3 WHEN 0 THEN 'metadata' ELSE 'fake' END,
  n % 2,
  3,
  NULL,
  '{}',
  '2026-05-23T00:00:00Z',
  '2026-05-23T00:00:00Z',
  NULL,
  NULL,
  NULL,
  NULL
FROM seq;

WITH RECURSIVE seq(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < ($TASK_COUNT / 2)
)
INSERT INTO task_outputs (id, task_id, output_type, target_id, payload_json, created_at)
SELECT
  printf('output-%06d', n),
  printf('task-%06d', n),
  'asset',
  printf('asset-%06d', ((n - 1) % $ASSET_COUNT) + 1),
  '{}',
  '2026-05-23T00:00:00Z'
FROM seq;

SELECT 'fixture_counts' AS metric,
  (SELECT COUNT(*) FROM assets) AS assets,
  (SELECT COUNT(*) FROM asset_versions) AS versions,
  (SELECT COUNT(*) FROM asset_tags) AS asset_tags,
  (SELECT COUNT(*) FROM album_items) AS album_items,
  (SELECT COUNT(*) FROM metadata_suggestions) AS metadata_suggestions,
  (SELECT COUNT(*) FROM tasks) AS tasks;

SELECT 'gallery_latest_page_query_plan' AS query;
EXPLAIN QUERY PLAN
SELECT a.id, a.title, av.id, av.version_number, av.file_path
FROM assets a
INNER JOIN asset_versions av ON av.asset_id = a.id
WHERE a.library_id = 'library'
  AND NOT EXISTS (
    SELECT 1
    FROM asset_versions newer
    WHERE newer.asset_id = av.asset_id
      AND (newer.version_number > av.version_number
        OR (newer.version_number = av.version_number AND newer.created_at > av.created_at))
  )
ORDER BY a.updated_at DESC, a.created_at DESC
LIMIT 100;
SELECT COUNT(*)
FROM (
  SELECT a.id
  FROM assets a
  INNER JOIN asset_versions av ON av.asset_id = a.id
  WHERE a.library_id = 'library'
    AND NOT EXISTS (
      SELECT 1
      FROM asset_versions newer
      WHERE newer.asset_id = av.asset_id
        AND (newer.version_number > av.version_number
          OR (newer.version_number = av.version_number AND newer.created_at > av.created_at))
    )
  ORDER BY a.updated_at DESC, a.created_at DESC
  LIMIT 100
);

SELECT 'tag_filter_query_plan' AS query;
EXPLAIN QUERY PLAN
SELECT a.id
FROM assets a
INNER JOIN asset_tags at ON at.asset_id = a.id
INNER JOIN tags t ON t.id = at.tag_id
WHERE a.library_id = 'library'
  AND t.name = 'tag-001'
ORDER BY a.updated_at DESC
LIMIT 100;
SELECT COUNT(*)
FROM (
  SELECT a.id
  FROM assets a
  INNER JOIN asset_tags at ON at.asset_id = a.id
  INNER JOIN tags t ON t.id = at.tag_id
  WHERE a.library_id = 'library'
    AND t.name = 'tag-001'
  ORDER BY a.updated_at DESC
  LIMIT 100
);

SELECT 'smart_album_text_rating_query_plan' AS query;
EXPLAIN QUERY PLAN
SELECT a.id
FROM assets a
WHERE a.library_id = 'library'
  AND a.rating >= 3
  AND (a.title LIKE '%Asset%' OR a.description LIKE '%Asset%' OR a.schema_prompt LIKE '%Asset%')
ORDER BY a.updated_at DESC
LIMIT 100;
SELECT COUNT(*)
FROM (
  SELECT a.id
  FROM assets a
  WHERE a.library_id = 'library'
    AND a.rating >= 3
    AND (a.title LIKE '%Asset%' OR a.description LIKE '%Asset%' OR a.schema_prompt LIKE '%Asset%')
  ORDER BY a.updated_at DESC
  LIMIT 100
);

SELECT 'version_tree_summary_query_plan' AS query;
EXPLAIN QUERY PLAN
SELECT asset_id, COUNT(*), MAX(version_number)
FROM asset_versions
GROUP BY asset_id;
SELECT COUNT(*)
FROM (
  SELECT asset_id, COUNT(*), MAX(version_number)
  FROM asset_versions
  GROUP BY asset_id
);

SELECT 'task_queue_query_plan' AS query;
EXPLAIN QUERY PLAN
SELECT id, task_type, status, provider, queue_position, priority
FROM tasks
WHERE library_id = 'library'
  AND status IN ('queued', 'retry_waiting', 'running')
ORDER BY priority DESC, queue_position ASC, created_at ASC
LIMIT 100;
SELECT COUNT(*)
FROM (
  SELECT id
  FROM tasks
  WHERE library_id = 'library'
    AND status IN ('queued', 'retry_waiting', 'running')
  ORDER BY priority DESC, queue_position ASC, created_at ASC
  LIMIT 100
);
SQL

echo "workload_db=$DB_PATH"
echo "assets=$ASSET_COUNT versions_per_asset=$VERSIONS_PER_ASSET tags=$TAG_COUNT tasks=$TASK_COUNT"

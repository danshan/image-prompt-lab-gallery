## MODIFIED Requirements

### Requirement: Asset metadata mutation preserves existing library storage

Asset metadata mutation SHALL preserve current library schema and compatibility behavior unless an explicit migration is specified.

#### Scenario: Tag add preserves storage semantics

- **WHEN** a tag is added through the application owner
- **THEN** existing tag rows are reused by tag name
- **AND** the asset-tag relation is upserted without changing the SQLite schema

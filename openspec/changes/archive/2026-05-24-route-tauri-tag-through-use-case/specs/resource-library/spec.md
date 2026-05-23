## MODIFIED Requirements

### Requirement: Asset metadata mutation preserves existing library storage

Asset metadata mutation SHALL preserve current library schema and compatibility behavior unless an explicit migration is specified.

#### Scenario: Tauri tag add preserves storage semantics

- **WHEN** a tag is added through the Tauri command adapter
- **THEN** the command delegates to the asset application owner
- **AND** existing tag storage and asset-tag upsert semantics are preserved

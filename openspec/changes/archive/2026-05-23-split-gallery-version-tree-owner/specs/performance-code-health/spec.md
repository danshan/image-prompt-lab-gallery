## MODIFIED Requirements

### Requirement: Large read-model hotspots are decomposed incrementally

Large read-model modules MUST be reduced through behavior-preserving extractions before adding new persistence technology.

#### Scenario: Version tree read model has a focused owner

- **GIVEN** gallery list and asset detail views need version tree names, branch counts, promoted-source labels, and asset-scoped lineage
- **WHEN** version tree read behavior changes
- **THEN** the tree construction, degradation reporting, promoted-source lookup, and lineage traversal SHOULD live in a focused owner module
- **AND** gallery list and asset detail orchestration SHOULD consume that owner without changing SQLite schema or runtime DTO payloads

## MODIFIED Requirements

### Requirement: Bounded Context 拥有自己的语言模型

系统 SHALL 为 resource library, asset/version, generation, metadata review, albums/search 和 task manager 建立清晰 bounded context。每个 bounded context MUST 拥有自己的 aggregate, command, query/read model 和 repository port 边界, 不得继续把所有业务结构集中在单个 shared DTO namespace 中作为主要模型。Search read-model behavior SHOULD be separated from gallery list/detail behavior once both are migrated behind application/query owners.

#### Scenario: Gallery card list read model has a focused owner

- **WHEN** Gallery card list projection behavior changes
- **THEN** latest-version lookup, generation-event summaries, aggregate counts, tags, albums, version tree labels, and card DTO assembly SHOULD live in a focused gallery card read-model owner
- **AND** `GalleryReadService` SHOULD remain the runtime-facing query boundary that delegates card projection before filtering and sorting

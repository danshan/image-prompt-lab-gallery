## MODIFIED Requirements

### Requirement: Bounded Context 拥有自己的语言模型

系统 SHALL 为 resource library, asset/version, generation, metadata review, albums/search 和 task manager 建立清晰 bounded context。每个 bounded context MUST 拥有自己的 aggregate, command, query/read model 和 repository port 边界, 不得继续把所有业务结构集中在单个 shared DTO namespace 中作为主要模型。Search read-model behavior SHOULD be separated from gallery list/detail behavior once both are migrated behind application/query owners.

#### Scenario: Asset Version 语言模型独立

- **WHEN** asset/version 代码被重构
- **THEN** asset aggregate 拥有 version number, version name, parent chain 和 reference source rule 的 domain model

#### Scenario: Task 语言模型独立

- **WHEN** task manager 代码被重构
- **THEN** task status, attempt, event, output link 和 scheduler policy 的模型位于 task bounded context 或 application task modules 中

#### Scenario: Search read model has a focused owner

- **WHEN** search behavior is migrated from a shared gallery compatibility module
- **THEN** search-specific filtering and result mapping live in a focused search read-model owner
- **AND** shared gallery card loading may remain shared until projection or query-shape hardening requires a deeper split

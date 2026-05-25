## MODIFIED Requirements

### Requirement: Review Workspace 状态覆盖

Review workspace SHALL 覆盖 pending list, selected detail, generated results, batch actions 和 task mirror 的 loading, empty, error 和 recovery states. Accept failure MUST 保留当前 local draft. Pending suggestions list SHALL remain vertically scrollable inside the Review Inbox panel when its content exceeds the available desktop workflow height, while header and batch controls remain reachable above the list.

#### Scenario: Accept Failure Preserves Draft
- **WHEN** 用户接受 Review draft 时写入失败
- **THEN** Review UI 保留用户当前 draft, 展示可恢复错误, 并允许用户重试或继续编辑

#### Scenario: No Pending Suggestions
- **WHEN** 当前 library 没有 pending metadata suggestion
- **THEN** Review workspace 展示 empty state, 清理旧 selected suggestion detail, 并保留其他 workflow context

#### Scenario: Related Task Failed
- **WHEN** Review field generation 相关 task failed
- **THEN** Review workspace 展示 task failure summary 和 Open task detail 入口, 不覆盖当前 draft

#### Scenario: Pending Review List Scrolls
- **WHEN** Review Inbox pending suggestions exceed the available height of the left panel
- **THEN** Review UI allows vertical scrolling within the pending suggestions list
- **AND** batch actions and add-to-album controls remain reachable above the list

## MODIFIED Requirements

### Requirement: 模块复杂度必须可治理

系统 SHALL 持续降低高复杂度模块的职责密度。任何超过局部维护阈值的模块 MUST 通过 ownership-based decomposition, focused tests, or documented staged refactor plan 进行治理, 而不是只移动代码位置。

#### Scenario: Gallery detail projection is split from gallery list composition

- **WHEN** gallery detail or inspector projection logic grows independently from gallery list composition
- **THEN** detail-specific helpers SHOULD live in a separate owner module
- **AND** the split SHOULD preserve behavior while reducing the responsibility density of `library/gallery.rs`

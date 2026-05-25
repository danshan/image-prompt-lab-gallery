## ADDED Requirements

### Requirement: Gallery Query 默认排除 Archived Assets

系统 SHALL 在默认 Gallery query 和 album-scoped Gallery query 中排除 archived assets. Archived asset MUST only appear in explicit archived content read models, not in normal Gallery, Albums detail, smart album result, manual album result, or add-to-album source query.

#### Scenario: Gallery 默认排除 Archived Asset

- **WHEN** 一个 asset 已被 archive
- **THEN** 默认 Gallery query 不返回该 asset
- **AND** text, tag, provider, rating 和 review filters 也不返回该 asset

#### Scenario: Manual Album Detail 排除 Archived Asset

- **WHEN** 一个 manual album 包含 archived asset membership
- **THEN** album detail query 不返回该 archived asset
- **AND** restore 该 asset 后 album detail query 可以再次返回该 asset

#### Scenario: Smart Album 排除 Archived Asset

- **WHEN** 一个 archived asset 满足 smart album query
- **THEN** smart album result 不返回该 asset
- **AND** restore 该 asset 后 smart album result 可以按原 query 返回该 asset

#### Scenario: Add To Album Source 排除 Archived Asset

- **WHEN** 用户打开 Albums add drawer
- **THEN** source query 不返回 archived assets

### Requirement: Albums 图墙支持 Full Image Preview

Albums workspace SHALL allow users to open full image preview from album detail thumbnails. Thumbnail preview MUST use the same lightbox behavior as Gallery. Manual album remove action MUST remain membership-only and MUST NOT archive or delete the asset.

#### Scenario: Album Thumbnail Opens Lightbox

- **WHEN** 用户在 Albums detail 图墙点击 asset thumbnail
- **THEN** 桌面应用打开 full image lightbox
- **AND** lightbox 展示该 asset 当前 image path

#### Scenario: Album Card Selection Still Works

- **WHEN** 用户点击 Albums detail 中 thumbnail 以外的 card area
- **THEN** 桌面应用执行 album asset selection 或 context action
- **AND** 不打开 lightbox

#### Scenario: Manual Album Remove Is Membership Only

- **WHEN** 用户从 manual album 中 remove asset
- **THEN** 系统只删除 album membership
- **AND** 不 archive asset
- **AND** 不删除 asset versions 或 managed files

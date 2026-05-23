## MODIFIED Requirements

### Requirement: Album workflows keep one runtime owner

Album workflows SHALL route runtime mutations and read commands through the album application owner where an application boundary exists.

#### Scenario: Desktop album list and create use application owner

- **WHEN** desktop requests album list or manual album creation
- **THEN** the Tauri command adapter SHOULD call the album application owner
- **AND** command view mapping remains in the Tauri adapter

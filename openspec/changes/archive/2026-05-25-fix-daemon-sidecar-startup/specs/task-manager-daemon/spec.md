## ADDED Requirements

### Requirement: Daemon startup failures are bounded and actionable
Desktop daemon startup and background daemon service control SHALL fail with recoverable diagnostics when required local daemon infrastructure is missing or OS service commands do not return promptly.

#### Scenario: Daemon binary missing before sidecar startup
- **WHEN** desktop needs to start an app-owned daemon sidecar and the daemon binary cannot be found
- **THEN** desktop returns a recoverable error that identifies the missing daemon binary condition
- **AND** desktop MUST NOT attempt to spawn a non-existent sidecar process

#### Scenario: Daemon binary missing before background daemon installation
- **WHEN** Settings Automation attempts to start, repair, or install background daemon support and the daemon binary cannot be found
- **THEN** desktop returns a recoverable error before writing a LaunchAgent that points to a non-existent binary

#### Scenario: Packaged app locates bundled daemon binary
- **WHEN** desktop runs from a packaged app bundle
- **THEN** daemon binary discovery includes the app bundle resources location and Tauri external sidecar target-triple filename
- **AND** release packaging includes the daemon binary in that location

#### Scenario: Background daemon service command times out
- **WHEN** an OS-level background daemon service command does not complete within the bounded service-control timeout
- **THEN** desktop returns a recoverable timeout diagnostic
- **AND** Settings Automation controls become usable again after the command returns

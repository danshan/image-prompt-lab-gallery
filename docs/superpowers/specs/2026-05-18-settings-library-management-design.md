# Settings Library Management Design

## Context

Image Prompt Lab already has a managed resource library model, an app-level registry, a sidebar library selector, and a Settings logs browser. The current Settings page only supports basic path/name entry plus create/open actions. It does not provide a complete maintenance surface for multiple registered libraries.

This design adds a dedicated Settings information architecture:

- `Settings / Libraries`: library lifecycle and backup maintenance.
- `Settings / Logs`: app log browsing and diagnostics.

The design keeps Rust core as the business fact boundary. Desktop commands handle transport, native dialogs, and operating system integration.

## Goals

- Support multiple library maintenance in Settings.
- Support creating a new library.
- Support opening an existing library folder.
- Support renaming a local registry display name.
- Support closing a library by unregistering it without deleting files.
- Support exporting a full library backup zip.
- Support importing a full library backup zip into a new local library directory.
- Support revealing a library folder in Finder.
- Keep Logs as a separate Settings subpage.

## Non-Goals

- Delete library files from disk.
- Merge a backup zip into the current library.
- Add cloud sync, multi-user collaboration, encryption, or daemon-backed library management.
- Replace Gallery or Album asset-level export. Settings backup export is a separate full-library workflow.
- Rewrite the existing resource library layout.

## Product Semantics

### Settings / Libraries

`Settings / Libraries` is the maintenance surface for registered libraries.

`Create Library` creates a managed library in a selected or typed empty directory, initializes the required layout, writes `manifest.json`, initializes `library.sqlite`, registers the library, and makes it current.

`Open Existing Library` opens a system folder picker, validates an existing library root, registers it, and makes it current.

`Rename` updates only the local app registry display name. It does not modify `manifest.json`. This makes rename a local alias operation, not a change to the portable library identity.

`Close` unregisters the library from this app. It does not delete the library directory, database, images, sidecars, exports, or manifest. If the closed library is current, the workbench enters a no-library state and clears stale gallery, inspector, album, review, and queue context.

`Export Zip` creates a full library backup package. The package is intended to restore an openable library and includes the manifest, database, managed directories, and required files.

`Import Zip` imports a full library backup package into a new local library directory. It does not merge assets into the current library. If the imported `manifest.id` already exists in the registry, the import creates a clone by generating a new library id and rewriting the imported manifest before registration.

`Reveal in Finder` opens the library root folder in the operating system file manager. It does not change registry or current library state.

### Settings / Logs

`Settings / Logs` keeps the existing log browser behavior: recent log list, preview, refresh, loading states, and truncated content indication. It does not contain library lifecycle actions.

## Information Architecture

The sidebar keeps one top-level `Settings` entry. Inside the Settings workspace, a local segmented control or tab control switches between:

- `Libraries`
- `Logs`

The default section is `Libraries`.

Switching Settings sections does not change the current library.

## UI Design

### Libraries View

The view is table-centered and optimized for maintenance.

Header:

- Title: `Libraries`.
- Secondary text: registered count and current library display name/path.

Toolbar:

- `Create Library`.
- `Open Existing Library`.
- `Import Zip`.

Table columns:

- `Name`.
- `Path`.
- `Schema`.
- `Status`.
- `Actions`.

Row actions:

- `Switch`.
- `Rename`.
- `Export Zip`.
- `Reveal in Finder`.
- `Close`.

The current library row shows a restrained `Current` badge. Long paths are single-line truncated with the full path available through hover/title text.

Missing-path rows show `Missing on disk`. For missing paths, `Close` remains enabled, while `Export Zip` and `Reveal in Finder` are disabled or return a recoverable error.

`Close` uses confirmation copy that explicitly states files on disk are not deleted.

### Logs View

The Logs view reuses the current logs browser:

- Header with `Logs` and `Refresh`.
- Left-side log list.
- Right-side preview.
- Existing loading, empty, and truncated states.

## Architecture

### Core

The core service remains the business boundary for registry and backup operations.

Add or adjust library service operations:

- `rename_library_alias(library_id, alias)`: updates only the registry display name.
- `unregister_library(library_id)`: removes the registry entry without touching library files.
- `export_library_backup_zip(library_path, output_zip_path)`: validates the library root and writes a full backup zip.
- `import_library_backup_zip(zip_path, destination_dir)`: extracts, validates, handles id conflict by cloning, registers, and returns the imported library summary.

Existing operations remain:

- `create_library`.
- `open_library`.
- `list_libraries`.

The existing directory-based `export_library` remains separate from backup zip export. It should continue to serve asset/sidecar export workflows and should not be overloaded with full-library backup semantics.

The existing hidden registry behavior can remain for backward compatibility, but the new UI does not expose hide. New close behavior should unregister rather than hide.

### Tauri

Tauri commands map desktop inputs to core service calls and handle OS-specific operations.

Add commands:

- `rename_library_alias`.
- `unregister_library`.
- `export_library_backup_zip`.
- `import_library_backup_zip`.
- `reveal_library_folder`.

Native dialog handling can be implemented with Tauri dialog APIs or thin commands, depending on the project dependency choice:

- pick folder for create/open.
- pick zip input for import.
- pick destination folder for import.
- pick zip save path for export.

`reveal_library_folder` belongs in Tauri rather than core because it is operating system integration.

### Frontend

Split the Settings UI:

- `SettingsWorkspace`.
- `SettingsLibrariesView`.
- `SettingsLogsView`.

State:

- `settingsSection: "libraries" | "logs"`, defaulting to `"libraries"`.
- Operation state scoped to library actions, so export/import/rename/close buttons cannot be repeatedly triggered.
- Logs state remains scoped to the Logs section.

After successful library mutations, the frontend refreshes the library registry. If the current library changed or was closed, it also clears stale workbench context.

## Data Flows

### Create Library

1. User chooses or enters an empty directory and name.
2. Frontend calls `create_library`.
3. Core creates layout, manifest, and database, then registers the library.
4. Frontend refreshes libraries, sets the new library current, and clears stale context.

### Open Existing Library

1. User chooses a folder.
2. Frontend calls `open_library`.
3. Core validates layout, manifest, schema, and required directories.
4. Core registers the library.
5. Frontend refreshes libraries and sets it current.

### Rename

1. User edits the row display name.
2. Frontend calls `rename_library_alias`.
3. Core updates the registry entry only.
4. Frontend refreshes libraries. If the renamed library is current, the sidebar selector updates.

Empty aliases are rejected.

### Close

1. User chooses `Close`.
2. UI confirms that files on disk are not deleted.
3. Frontend calls `unregister_library`.
4. Core removes the registry entry.
5. Frontend refreshes libraries.
6. If the closed library was current, frontend clears current library state and all stale workbench context.

### Export Zip

1. User chooses `Export Zip` for a row.
2. UI asks for a `.zip` save path.
3. Frontend calls `export_library_backup_zip`.
4. Core validates the library root and writes a full backup zip through a temporary file.
5. Core finalizes the zip path only after success.
6. UI shows completion status and summary.

### Import Zip

1. User chooses a backup zip and destination directory.
2. Frontend calls `import_library_backup_zip`.
3. Core extracts into a staging directory.
4. Core validates manifest, database, schema, and required directories.
5. If the manifest id already exists in the registry, core generates a new id and rewrites the imported manifest.
6. Core moves the staged library into the destination, registers it, and returns a summary.
7. Frontend refreshes libraries and makes the imported library current.

### Reveal in Finder

1. User chooses `Reveal in Finder`.
2. Frontend calls `reveal_library_folder`.
3. Tauri opens the path in Finder.
4. Missing paths return a recoverable error.

## Error Handling

Use recoverable Settings-local status for library maintenance errors.

Important error cases:

- `InvalidLibraryBackup`: the zip is not a complete backup package.
- `ImportDestinationNotEmpty`: the destination is not safe for import.
- `ZipIoError`: compression or extraction failed.
- `SchemaMismatch`: the imported or opened library schema is unsupported.
- `LibraryNotFound`: the registered path no longer exists.
- `LibraryIdConflictCloned`: warning/status, not a failure. The imported library was cloned with a new id.

Export should avoid leaving a successful-looking partial zip. Use a temporary path and final rename.

Import should avoid leaving a partially registered library. Register only after extraction and validation pass.

## Testing

### Core Tests

- `rename_library_alias` updates registry display name and does not modify `manifest.json`.
- `unregister_library` removes the registry entry and does not delete the library root.
- Backup zip export includes `manifest.json`, `library.sqlite`, required directories, and managed files.
- Importing a valid backup zip registers the library and allows it to be opened.
- Importing a backup whose id already exists rewrites the manifest id and registers a clone.
- Importing an invalid zip returns `InvalidLibraryBackup` and does not leave a registered partial library.
- Export uses a temporary file and finalizes only on success.

### Tauri Tests

If the project has a practical command test harness, cover:

- camelCase input/output mapping.
- recoverable error mapping for missing reveal path.
- command mapping for rename, unregister, export zip, and import zip.

If no practical Tauri harness exists, keep most coverage in core tests and use desktop smoke testing for commands.

### Frontend Tests

- Settings default section is `libraries`.
- Switching between `libraries` and `logs` does not change current library.
- Closing the current library clears gallery, inspector detail, albums, suggestions, tasks, and selected ids.
- Renaming the current library updates the displayed alias.
- Missing-path rows keep `Close` enabled and disable export/reveal actions.

## Rollout Plan

1. Add OpenSpec artifacts for Settings library management.
2. Add core registry operations: alias rename and unregister.
3. Add backup zip import/export in core.
4. Add Tauri commands and native dialog or reveal integration.
5. Split Settings frontend into `Libraries` and `Logs` sections.
6. Add focused core and frontend tests.
7. Manually smoke test create, open existing, rename, close, export zip, import zip, reveal, and logs navigation.

## Open Decisions

No blocking open decisions remain for the initial implementation.

## 1. Window Chrome

- [x] 1.1 Restore Tauri system window decorations so native titlebar controls are provided by the host OS.
- [x] 1.2 Remove the React app-provided top bar and custom window control buttons.
- [x] 1.3 Remove custom titlebar, drag region and window control CSS.
- [x] 1.4 Adjust the workbench grid so content starts at the top of the application content area without an empty titlebar row.

## 2. Inspector And Metadata Layout

- [x] 2.1 Remove asset resolution display from Sidebar, Library Status or Gallery asset summary surfaces.
- [x] 2.2 Keep resolution display in Inspector File section when width and height are both available.
- [x] 2.3 Add aspect ratio formatting derived from file width and height, using unavailable behavior when either dimension is missing or invalid.
- [x] 2.4 Move the Inspector Rating card so it appears immediately below the Prompt card while preserving existing rating update behavior.

## 3. Tests And Verification

- [x] 3.1 Add or update frontend tests for aspect ratio formatting and missing dimension behavior.
- [x] 3.2 Run desktop frontend tests with `npm test --prefix apps/desktop`.
- [x] 3.3 Run desktop TypeScript build or typecheck with the existing project command.
- [x] 3.4 Manually verify in real Tauri mode that the system titlebar can drag the window and close, minimize, maximize or restore controls work.
- [x] 3.5 Manually verify Gallery or Sidebar no longer shows asset resolution, Inspector File shows resolution plus aspect ratio, and Rating appears below Prompt.

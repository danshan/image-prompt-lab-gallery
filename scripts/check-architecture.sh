#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOMAIN_DIR="$ROOT_DIR/crates/imglab-core/src/domain"
APPLICATION_DIR="$ROOT_DIR/crates/imglab-core/src/application"
RUNTIME_DIRS=(
  "$ROOT_DIR/crates/imglab-cli/src"
  "$ROOT_DIR/crates/imglab-daemon/src"
  "$ROOT_DIR/apps/desktop/src-tauri/src"
)
DESKTOP_SRC_DIR="$ROOT_DIR/apps/desktop/src"

if [[ ! -d "$DOMAIN_DIR" ]]; then
  echo "missing domain directory: $DOMAIN_DIR" >&2
  exit 1
fi

violations="$(rg -n 'rusqlite|std::fs|tauri|imglab_daemon|imglab_cli|crate::infrastructure|crate::library::LocalLibraryService' "$DOMAIN_DIR" || true)"

if [[ -n "$violations" ]]; then
  echo "Architecture dependency violations found in domain modules:" >&2
  echo "$violations" >&2
  exit 1
fi

application_violations="$(rg -n 'rusqlite|std::fs|tauri|imglab_daemon|imglab_cli|crate::infrastructure|crate::library::' "$APPLICATION_DIR" || true)"

if [[ -n "$application_violations" ]]; then
  echo "Architecture dependency violations found in application modules:" >&2
  echo "$application_violations" >&2
  exit 1
fi

runtime_rule_violations="$(rg -n 'LocalGenerationService|MAX\(version_number\)|max\(version_number\)|version_number \+ 1' "${RUNTIME_DIRS[@]}" || true)"

if [[ -n "$runtime_rule_violations" ]]; then
  echo "Runtime modules appear to bypass application/domain business rules:" >&2
  echo "$runtime_rule_violations" >&2
  exit 1
fi

desktop_state_violations="$(rg -n 'from "(\./|\.\./|\.\./\.\./|\.\./\.\./\.\./)?workbench-state(\.js)?("|$)' "$DESKTOP_SRC_DIR" || true)"

if [[ -n "$desktop_state_violations" ]]; then
  echo "Desktop workflow modules still import workbench-state as a primary state owner:" >&2
  echo "$desktop_state_violations" >&2
  exit 1
fi

echo "Architecture dependency check passed"

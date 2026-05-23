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

runtime_legacy_service_usage="$(rg -n 'LocalLibraryService' "${RUNTIME_DIRS[@]}" || true)"
runtime_legacy_service_violations=""
allowed_runtime_legacy_service_files=(
  "$ROOT_DIR/crates/imglab-cli/src/main.rs"
  "$ROOT_DIR/crates/imglab-daemon/src/lib.rs"
  "$ROOT_DIR/crates/imglab-daemon/src/runtime.rs"
)

if [[ -n "$runtime_legacy_service_usage" ]]; then
  while IFS= read -r line; do
    file="${line%%:*}"
    allowed="false"
    for allowed_file in "${allowed_runtime_legacy_service_files[@]}"; do
      if [[ "$file" == "$allowed_file" ]]; then
        allowed="true"
        break
      fi
    done
    if [[ "$allowed" != "true" ]]; then
      runtime_legacy_service_violations+="$line"$'\n'
    fi
  done <<< "$runtime_legacy_service_usage"
fi

if [[ -n "$runtime_legacy_service_violations" ]]; then
  echo "Runtime modules introduced direct LocalLibraryService usage outside the documented compatibility allowlist:" >&2
  echo "$runtime_legacy_service_violations" >&2
  echo "Update application/use-case wiring instead, or document a bounded compatibility exception in docs/architecture/ddd-boundary-inventory.md and scripts/check-architecture.sh." >&2
  exit 1
fi

desktop_state_violations="$(rg -n 'from "(\./|\.\./|\.\./\.\./|\.\./\.\./\.\./)?workbench-state(\.js)?("|$)' "$DESKTOP_SRC_DIR" || true)"

if [[ -n "$desktop_state_violations" ]]; then
  echo "Desktop workflow modules still import workbench-state as a primary state owner:" >&2
  echo "$desktop_state_violations" >&2
  exit 1
fi

echo "Architecture dependency check passed"

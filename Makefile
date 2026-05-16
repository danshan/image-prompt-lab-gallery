.PHONY: fmt lint test check desktop-build desktop-dev

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

check:
	cargo check --workspace

desktop-build:
	cd apps/desktop && npm run build

desktop-dev:
	cd apps/desktop && npm run tauri dev

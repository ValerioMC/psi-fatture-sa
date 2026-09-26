# PSI Fatture is a desktop app (Tauri 2 + Vue 3 + SQLite); there is no server process or
# database to start beyond the app itself, so this file is short on purpose.

NPM        ?= npm
CARGO      ?= cargo
MANIFEST   := src-tauri/Cargo.toml
BUNDLE_DIR := src-tauri/target/release/bundle

.DEFAULT_GOAL := help
.PHONY: help dev build app install test test-frontend test-backend lint fmt

help: ## Show this help
	@grep -hE '^[a-zA-Z0-9_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

dev: ## Start the app in dev mode (hot-reload)
	$(NPM) run tauri dev

build: ## Build the frontend and bundle the desktop app (unsigned) into src-tauri/target/release/bundle
	$(NPM) install
	$(NPM) run tauri build
	@echo "$(BUNDLE_DIR)"

# Mounts the disk image `make build` just produced and runs PSI Fatture straight from it. Nothing
# is copied into /Applications, so an installed copy (if any) is left alone.
app: build ## macOS: mount the disk image and run PSI Fatture from it, without installing it
	./scripts/macos-app.sh

# Replaces whatever is at /Applications/PSI Fatture.app, unlike `app`, which leaves it alone.
install: build ## macOS: mount the disk image and install PSI Fatture.app into /Applications
	./scripts/macos-install.sh

test: test-frontend test-backend ## Everything

test-frontend: ## vue-tsc typecheck and vitest
	npx vue-tsc --noEmit
	$(NPM) test

test-backend: ## cargo test (invoice totals, validation, config)
	$(CARGO) test --manifest-path $(MANIFEST)

lint: ## clippy over the backend, as CI runs it
	$(CARGO) clippy --manifest-path $(MANIFEST) --all-targets -- -D warnings

fmt: ## cargo fmt --check over the backend, as CI runs it
	$(CARGO) fmt --manifest-path $(MANIFEST) --check

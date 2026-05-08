.PHONY: install uninstall build dev

APP_NAME  = Lumos
APP_PATH  = /Applications/$(APP_NAME).app
BUILD_OUT = src-tauri/target/release/bundle/macos/$(APP_NAME).app

# Install Lumos to /Applications (builds first if needed)
install: build
	@echo "→ Installing $(APP_NAME) to /Applications..."
	@rm -rf "$(APP_PATH)"
	@cp -r "$(BUILD_OUT)" "$(APP_PATH)"
	@echo "✓ Installed. Launch from Spotlight or open /Applications/$(APP_NAME).app"

# Remove from /Applications
uninstall:
	@echo "→ Removing $(APP_NAME) from /Applications..."
	@rm -rf "$(APP_PATH)"
	@echo "✓ Uninstalled."

# Build release binary (no code signing required for local use)
build:
	@echo "→ Building $(APP_NAME)..."
	@export PATH="$$HOME/.cargo/bin:$$PATH" && \
		TAURI_SIGNING_PRIVATE_KEY="" \
		pnpm tauri build --bundles app 2>&1 | grep -v "^warning"

# Start dev server (hot reload, for development)
dev:
	@export PATH="$$HOME/.cargo/bin:$$PATH" && pnpm tauri dev

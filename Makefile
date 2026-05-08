.PHONY: build-electron build-rust install-electron install-rust install clean help

ELECTRON_OUT  = out/Lumos - Electron-darwin-arm64/Lumos - Electron.app
RUST_OUT      = src-tauri/target/release/bundle/macos/Lumos - Rust.app
INSTALL_DIR   = /Applications

# ── Electron (macOS .app, no signing) ───────────────────────────────────────
build-electron: _electron-deps
	@echo "→ Building Lumos - Electron..."
	cp package.electron.json package.json
	npx electron-forge package -- --no-sign
	cp package.tauri.json package.json 2>/dev/null || true
	@echo "✓ Built: $(ELECTRON_OUT)"

_electron-deps:
	@# Save Tauri package.json and install Electron deps
	@cp package.json package.tauri.json
	@cp package.electron.json package.json
	@echo "→ Installing Electron dependencies..."
	@npm install --loglevel=error 2>&1 | tail -3
	@cp package.tauri.json package.json

# ── Rust / Tauri (.app, no signing) ─────────────────────────────────────────
build-rust:
	@echo "→ Building Lumos - Rust..."
	@export PATH="$$HOME/.cargo/bin:$$PATH" && \
		TAURI_SIGNING_PRIVATE_KEY="" \
		pnpm tauri build --bundles app 2>&1 | grep -v "^warning"
	@echo "✓ Built: $(RUST_OUT)"

# ── Install both to /Applications ───────────────────────────────────────────
install-electron:
	@echo "→ Installing Lumos - Electron to /Applications..."
	@sudo cp -r "$(ELECTRON_OUT)" "$(INSTALL_DIR)/"
	@echo "✓ Installed."

install-rust:
	@echo "→ Installing Lumos - Rust to /Applications..."
	@sudo cp -r "$(RUST_OUT)" "$(INSTALL_DIR)/"
	@echo "✓ Installed."

install: install-electron install-rust

# ── Build + install both ─────────────────────────────────────────────────────
all: build-electron build-rust install
	@echo ""
	@echo "✓ Both builds installed:"
	@echo "  /Applications/Lumos - Electron.app"
	@echo "  /Applications/Lumos - Rust.app"

clean:
	rm -rf out/ dist/ .webpack/ package.tauri.json
	export PATH="$$HOME/.cargo/bin:$$PATH" && cargo clean --manifest-path src-tauri/Cargo.toml

help:
	@echo "Lumos build targets:"
	@echo "  make build-electron   Build Electron app (no signing)"
	@echo "  make build-rust       Build Tauri/Rust app (no signing)"
	@echo "  make install-electron Install Electron app to /Applications"
	@echo "  make install-rust     Install Tauri/Rust app to /Applications"
	@echo "  make all              Build + install both"
	@echo "  make clean            Remove all build artifacts"

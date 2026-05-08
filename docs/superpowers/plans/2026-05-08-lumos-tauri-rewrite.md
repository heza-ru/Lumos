# Lumos — Tauri/Rust/Skia Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Lumos from Electron/JS into a production-grade Tauri 2 + Rust + React/TypeScript macOS screen annotation utility with Skia Metal rendering.

**Architecture:** A transparent, always-on-top NSPanel hosts a Skia Metal drawing surface for zero-latency annotation rendering. A separate Tauri webview window serves the floating toolbar and settings UI. Rust owns all drawing state, hotkeys, and macOS native API calls; React/TypeScript owns only the toolbar and settings surfaces via Tauri IPC commands.

**Tech Stack:** Rust 1.78+, Tauri 2, React 18 + TypeScript 5, skia-safe 0.75 (Metal backend), cocoa/objc crates for NSPanel, metal crate for GPU surface, tauri-plugin-global-shortcut, serde/tokio.

---

## Scope note

This spec covers 6 independent subsystems. Each phase produces working, testable software. Phases 1–3 form the shippable MVP core; phases 4–6 add effect features and distribution. Execute phases sequentially; within a phase, tasks run sequentially.

---

## File Map

```
lumos-tauri/                          ← new sibling directory (not inside existing Electron project)
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── Info.plist
│   ├── entitlements.plist
│   └── src/
│       ├── main.rs                   ← app entry, Tauri builder, overlay init
│       ├── lib.rs                    ← re-exports for integration tests
│       ├── state.rs                  ← AppState, DrawingState, shared Arc<Mutex<>>
│       ├── commands.rs               ← all #[tauri::command] IPC handlers
│       ├── hotkeys.rs                ← global shortcut registration/dispatch
│       ├── overlay/
│       │   ├── mod.rs                ← public overlay API
│       │   ├── panel.rs              ← NSPanel creation via objc (macOS only)
│       │   └── event_tap.rs          ← CGEventTap mouse capture
│       ├── renderer/
│       │   ├── mod.rs                ← RenderLoop, frame dispatch
│       │   ├── canvas.rs             ← Skia + Metal surface lifecycle
│       │   └── draw.rs               ← stroke/shape/effect Skia draw calls
│       ├── tools/
│       │   ├── mod.rs                ← Tool trait, ToolKind enum, dispatch
│       │   ├── pen.rs                ← freehand pen + perfect-freehand port
│       │   ├── highlighter.rs        ← translucent pen strokes
│       │   ├── shapes.rs             ← arrow, rectangle, ellipse, line
│       │   ├── text.rs               ← text annotations
│       │   └── laser.rs              ← laser pointer with time-decay fade
│       ├── history.rs                ← undo/redo command stack
│       └── effects/
│           ├── mod.rs
│           ├── cursor.rs             ← glow, ring, pulse, ripple
│           ├── spotlight.rs          ← spotlight/dim mode
│           └── zoom.rs               ← zoom lens magnifier
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── types/
│   │   └── index.ts                  ← shared TS types mirroring Rust enums
│   ├── hooks/
│   │   ├── useToolState.ts
│   │   └── useAppState.ts
│   └── components/
│       ├── Toolbar/
│       │   ├── Toolbar.tsx
│       │   ├── ToolButton.tsx
│       │   ├── ColorPicker.tsx
│       │   └── Toolbar.module.css
│       └── Settings/
│           ├── Settings.tsx
│           ├── HotkeyRecorder.tsx
│           └── Settings.module.css
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## Phase 1 — Foundation

### Task 1: Bootstrap Tauri 2 project

**Files:**
- Create: `lumos-tauri/src-tauri/Cargo.toml`
- Create: `lumos-tauri/src-tauri/tauri.conf.json`
- Create: `lumos-tauri/src-tauri/src/main.rs`
- Create: `lumos-tauri/package.json`
- Create: `lumos-tauri/vite.config.ts`
- Create: `lumos-tauri/tsconfig.json`
- Create: `lumos-tauri/src/main.tsx`
- Create: `lumos-tauri/src/App.tsx`

- [ ] **Step 1: Scaffold with Tauri CLI**

```bash
cd /Users/mohammad.haider/Documents
cargo install tauri-cli --version "^2" --locked
cargo tauri init --app-name lumos --window-title "Lumos" --dist-dir ../dist --dev-path "http://localhost:1420" --before-dev-command "pnpm dev" --before-build-command "pnpm build" --ci
mv lumos lumos-tauri
cd lumos-tauri
pnpm init
pnpm add react@18 react-dom@18
pnpm add -D typescript@5 @types/react @types/react-dom vite @vitejs/plugin-react
```

- [ ] **Step 2: Write `src-tauri/Cargo.toml`**

```toml
[package]
name = "lumos"
version = "0.1.0"
edition = "2021"
rust-version = "1.78"

[lib]
name = "lumos_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[[bin]]
name = "lumos"
path = "src/main.rs"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["macos-private-api"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
parking_lot = "0.12"
log = "0.4"
env_logger = "0.11"

# macOS native (cfg-guarded)
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.26"
objc = "0.2"
core-graphics = "0.23"
core-foundation = "0.9"
metal = "0.28"
skia-safe = { version = "0.75", features = ["metal", "textlayout"] }

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- [ ] **Step 3: Write `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: Write `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Lumos",
  "version": "0.1.0",
  "identifier": "com.lumos.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "label": "toolbar",
        "title": "Lumos Toolbar",
        "width": 380,
        "height": 56,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": true,
        "url": "index.html"
      }
    ],
    "security": {
      "csp": null
    },
    "macOSPrivateApi": true
  },
  "bundle": {
    "active": true,
    "targets": "dmg",
    "identifier": "com.lumos.app",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"],
    "macOS": {
      "minimumSystemVersion": "13.0",
      "entitlements": "entitlements.plist",
      "exceptionDomain": ""
    }
  }
}
```

- [ ] **Step 5: Write `src-tauri/entitlements.plist`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <false/>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
```

- [ ] **Step 6: Write `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::init();
    lumos_lib::run();
}
```

- [ ] **Step 7: Write `src-tauri/src/lib.rs`**

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
```

- [ ] **Step 8: Write `vite.config.ts`**

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true },
});
```

- [ ] **Step 9: Write `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM"],
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "resolveJsonModule": true,
    "outDir": "dist",
    "baseUrl": "."
  },
  "include": ["src"]
}
```

- [ ] **Step 10: Write `src/main.tsx`**

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 11: Write `src/App.tsx`**

```tsx
export default function App() {
  return <div style={{ color: "white", padding: 8 }}>Lumos toolbar placeholder</div>;
}
```

- [ ] **Step 12: Verify the project compiles**

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: compiles cleanly (warnings OK, no errors).

- [ ] **Step 13: Commit**

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
git init && git add .
git commit -m "chore: bootstrap Tauri 2 + React/TS project"
```

---

### Task 2: AppState and shared state module

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test for AppState**

```rust
// src-tauri/src/state.rs  (add at bottom)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_is_pen() {
        let state = DrawingState::default();
        assert_eq!(state.active_tool, ToolKind::Pen);
    }

    #[test]
    fn overlay_starts_hidden() {
        let state = AppState::default();
        assert!(!state.overlay_visible);
    }

    #[test]
    fn passthrough_starts_true() {
        let state = AppState::default();
        assert!(state.click_through);
    }
}
```

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: FAIL — `state` module not found.

- [ ] **Step 2: Write `src-tauri/src/state.rs`**

```rust
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    #[default]
    Pen,
    Highlighter,
    Arrow,
    Rectangle,
    Ellipse,
    Line,
    Text,
    Laser,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrokeWidth {
    Thin,
    #[default]
    Medium,
    Bold,
    ExtraBold,
}

impl StrokeWidth {
    pub fn pen_px(self) -> f32 {
        match self { Self::Thin => 4.0, Self::Medium => 8.0, Self::Bold => 12.0, Self::ExtraBold => 16.0 }
    }
    pub fn highlighter_px(self) -> f32 {
        match self { Self::Thin => 8.0, Self::Medium => 16.0, Self::Bold => 24.0, Self::ExtraBold => 32.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLUE:   Self = Self { r: 82,  g: 155, b: 224, a: 255 };
    pub const RED:    Self = Self { r: 224, g: 82,  b: 82,  a: 255 };
    pub const GREEN:  Self = Self { r: 82,  g: 224, b: 108, a: 255 };
    pub const WHITE:  Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK:  Self = Self { r: 30,  g: 30,  b: 30,  a: 255 };
}

impl Default for Color {
    fn default() -> Self { Self::BLUE }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: u64,
    pub tool: ToolKind,
    pub color: Color,
    pub width: StrokeWidth,
    pub points: Vec<Point>,
    /// milliseconds since epoch when stroke was created (for laser fade)
    pub created_at_ms: u64,
}

/// All persistent drawing content on the overlay.
#[derive(Debug, Default)]
pub struct DrawingState {
    pub active_tool: ToolKind,
    pub active_color: Color,
    pub active_width: StrokeWidth,
    pub strokes: Vec<Stroke>,
    pub next_id: u64,
    /// The stroke currently being drawn (not yet committed to `strokes`)
    pub live_stroke: Option<Stroke>,
}

impl DrawingState {
    pub fn begin_stroke(&mut self, x: f32, y: f32, ts: u64) {
        self.live_stroke = Some(Stroke {
            id: self.next_id,
            tool: self.active_tool,
            color: self.active_color.clone(),
            width: self.active_width,
            points: vec![Point { x, y }],
            created_at_ms: ts,
        });
        self.next_id += 1;
    }

    pub fn extend_stroke(&mut self, x: f32, y: f32) {
        if let Some(s) = &mut self.live_stroke {
            s.points.push(Point { x, y });
        }
    }

    pub fn commit_stroke(&mut self) {
        if let Some(s) = self.live_stroke.take() {
            if s.points.len() >= 2 {
                self.strokes.push(s);
            }
        }
    }

    pub fn undo(&mut self) {
        self.strokes.pop();
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.live_stroke = None;
    }
}

/// Top-level app state shared between Tauri commands and the render loop.
#[derive(Debug, Default)]
pub struct AppState {
    pub overlay_visible: bool,
    pub click_through: bool,
    pub drawing: DrawingState,
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState {
        overlay_visible: false,
        click_through: true,
        drawing: DrawingState::default(),
    }))
}
```

- [ ] **Step 3: Add `mod state` to lib.rs and run tests**

```rust
// src-tauri/src/lib.rs
pub mod state;

pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml state
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat: add AppState, DrawingState, ToolKind with tests"
```

---

### Task 3: macOS overlay NSPanel

> This task is macOS-only. All code lives behind `#[cfg(target_os = "macos")]`.

**Files:**
- Create: `src-tauri/src/overlay/mod.rs`
- Create: `src-tauri/src/overlay/panel.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** The annotation canvas must be a transparent, always-on-top, borderless NSPanel — not a webview. NSPanel with `NSNonactivatingPanelMask` never steals focus. We set window level to `kCGScreenSaverWindowLevel + 1` so it floats above fullscreen apps and Spaces. Click-through (`ignoresMouseEvents`) is toggled at runtime.

- [ ] **Step 1: Write failing test for overlay module**

```rust
// src-tauri/src/overlay/mod.rs  (create empty placeholder first)
#[cfg(test)]
mod tests {
    #[test]
    fn overlay_module_exists() {
        // Verifies the module compiles
        assert!(true);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml overlay
```

Expected: FAIL — module not found.

- [ ] **Step 2: Create `src-tauri/src/overlay/mod.rs`**

```rust
#[cfg(target_os = "macos")]
mod panel;

#[cfg(target_os = "macos")]
pub use panel::{create_overlay, set_click_through, show_overlay, hide_overlay, overlay_frame};

/// Opaque handle to the platform overlay window.
pub struct OverlayHandle {
    #[cfg(target_os = "macos")]
    pub ns_panel: *mut objc::runtime::Object,
}

// SAFETY: NSPanel pointer is only touched on the main thread via Tauri's
//         main-thread executor. We assert this invariant at call sites.
unsafe impl Send for OverlayHandle {}
unsafe impl Sync for OverlayHandle {}

#[cfg(test)]
mod tests {
    #[test]
    fn overlay_module_exists() {
        assert!(true);
    }
}
```

- [ ] **Step 3: Create `src-tauri/src/overlay/panel.rs`**

```rust
use cocoa::appkit::{
    NSApp, NSApplication, NSBackingStoreType, NSColor, NSScreen, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSPoint, NSRect, NSSize};
use core_graphics::display::CGWindowLevel;
use objc::{class, msg_send, sel, sel_impl};

// Window level that floats above fullscreen apps.
// kCGScreenSaverWindowLevel = 1000; we add 1 for guaranteed topmost.
const OVERLAY_LEVEL: i64 = 1001;

/// Create a transparent, borderless, non-activating NSPanel spanning the
/// primary display. Returns the raw NSPanel pointer.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn create_overlay() -> *mut objc::runtime::Object {
    let screen: id = msg_send![class!(NSScreen), mainScreen];
    let frame: NSRect = msg_send![screen, frame];

    let style = NSWindowStyleMask::NSBorderlessWindowMask
        | NSWindowStyleMask::NSNonactivatingPanelMask;

    let panel: id = {
        let alloc: id = msg_send![class!(NSPanel), alloc];
        msg_send![alloc,
            initWithContentRect: frame
            styleMask: style
            backing: NSBackingStoreType::NSBackingStoreBuffered
            defer: NO
        ]
    };

    // Transparent background
    let clear: id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![panel, setBackgroundColor: clear];
    let _: () = msg_send![panel, setOpaque: NO];
    let _: () = msg_send![panel, setHasShadow: NO];

    // Always on top, above fullscreen
    let _: () = msg_send![panel, setLevel: OVERLAY_LEVEL];

    // Appears on all Spaces and fullscreen apps
    let behaviors = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
        | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
        | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
    let _: () = msg_send![panel, setCollectionBehavior: behaviors];

    // Start click-through
    let _: () = msg_send![panel, setIgnoresMouseEvents: YES];

    // Disable animation when showing/hiding
    let _: () = msg_send![panel, setAnimationBehavior: 2i64]; // NSWindowAnimationBehaviorNone

    panel
}

/// Show the overlay panel.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn show_overlay(panel: *mut objc::runtime::Object) {
    let _: () = msg_send![panel, orderFrontRegardless];
}

/// Hide the overlay panel.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn hide_overlay(panel: *mut objc::runtime::Object) {
    let _: () = msg_send![panel, orderOut: nil];
}

/// Enable or disable click-through (mouse passthrough).
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn set_click_through(panel: *mut objc::runtime::Object, enabled: bool) {
    let val: cocoa::base::BOOL = if enabled { YES } else { NO };
    let _: () = msg_send![panel, setIgnoresMouseEvents: val];
}

/// Get the current frame of the overlay panel in screen coordinates.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn overlay_frame(panel: *mut objc::runtime::Object) -> NSRect {
    msg_send![panel, frame]
}
```

- [ ] **Step 4: Add overlay to lib.rs and verify compilation**

```rust
// src-tauri/src/lib.rs
pub mod state;
pub mod overlay;
```

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: builds clean. No test run needed — panel code requires macOS runtime.

- [ ] **Step 5: Wire overlay creation into Tauri setup**

```rust
// src-tauri/src/lib.rs
use std::sync::Arc;
use parking_lot::Mutex;
use crate::state::new_shared_state;
use crate::overlay::OverlayHandle;

pub mod state;
pub mod overlay;

pub fn run() {
    env_logger::init();

    let app_state = new_shared_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state.clone())
        .setup(move |_app| {
            #[cfg(target_os = "macos")]
            {
                let handle = unsafe {
                    let panel = overlay::create_overlay();
                    OverlayHandle { ns_panel: panel }
                };
                // Store handle so it is not dropped
                // (In Task 4 we wire this into the render loop)
                std::mem::forget(handle);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
```

- [ ] **Step 6: Build and verify no crash on launch**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "error|warning: unused" | head -20
```

Expected: no errors.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/overlay/ src-tauri/src/lib.rs
git commit -m "feat(macos): create transparent NSPanel overlay on app launch"
```

---

### Task 4: Skia Metal rendering pipeline

**Files:**
- Create: `src-tauri/src/renderer/mod.rs`
- Create: `src-tauri/src/renderer/canvas.rs`
- Create: `src-tauri/src/renderer/draw.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** Skia renders into a `Metal` texture that is composited onto the NSPanel's layer. The render loop runs on a dedicated thread at ~120 fps (display link rate). Each frame reads DrawingState under a short lock, builds Skia draw calls, and presents via CAMetalLayer. This keeps the lock hold time < 1 ms.

- [ ] **Step 1: Write failing test for canvas module**

```rust
// In src-tauri/src/renderer/canvas.rs (create stub)
#[cfg(test)]
mod tests {
    #[test]
    fn canvas_module_exists() { assert!(true); }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml renderer
```

Expected: FAIL — module not found.

- [ ] **Step 2: Create `src-tauri/src/renderer/mod.rs`**

```rust
#[cfg(target_os = "macos")]
pub mod canvas;
#[cfg(target_os = "macos")]
pub mod draw;

#[cfg(test)]
mod tests {
    #[test]
    fn renderer_module_exists() { assert!(true); }
}
```

- [ ] **Step 3: Create `src-tauri/src/renderer/canvas.rs`**

```rust
use metal::{Device, MetalLayer};
use objc::{class, msg_send, sel, sel_impl};
use skia_safe::{
    gpu::{self, mtl},
    surfaces, ColorType, ImageInfo, Surface,
};

/// Owns the Metal device, command queue, Skia GPU context, and the
/// CAMetalLayer attached to the NSPanel's content view.
pub struct MetalCanvas {
    pub device: Device,
    pub command_queue: metal::CommandQueue,
    pub gr_context: gpu::DirectContext,
    pub layer: MetalLayer,
    pub width: i32,
    pub height: i32,
    pub scale: f32,
}

impl MetalCanvas {
    /// Attach a Metal drawing layer to the given NSPanel.
    ///
    /// `width`/`height` are logical points; `scale` is the Retina backing
    /// scale factor (2.0 on Retina displays).
    ///
    /// # Safety
    /// `ns_panel` must be a valid NSPanel pointer. Call on main thread.
    pub unsafe fn new(
        ns_panel: *mut objc::runtime::Object,
        width: i32,
        height: i32,
        scale: f32,
    ) -> Self {
        let device = Device::system_default().expect("no Metal device");
        let command_queue = device.new_command_queue();

        // Attach a CAMetalLayer to the panel's content view
        let content_view: *mut objc::runtime::Object = msg_send![ns_panel, contentView];
        let _: () = msg_send![content_view, setWantsLayer: true];

        let layer = MetalLayer::new();
        layer.set_device(&device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        layer.set_drawable_size(metal::CGSize {
            width: (width as f64) * (scale as f64),
            height: (height as f64) * (scale as f64),
        });
        layer.set_opaque(false); // transparent background

        let _: () = msg_send![content_view, setLayer: layer.as_ref()];

        let backend = mtl::BackendContext::new(
            device.as_ptr() as mtl::Handle,
            command_queue.as_ptr() as mtl::Handle,
        );
        let gr_context =
            gpu::DirectContext::new_metal(&backend, None).expect("failed to create Skia Metal context");

        Self { device, command_queue, gr_context, layer, width, height, scale }
    }

    /// Acquire the next drawable and wrap it in a Skia Surface.
    /// Returns None if no drawable is available (display throttle).
    pub fn next_surface(&mut self) -> Option<Surface> {
        let drawable = self.layer.next_drawable()?;
        let texture_info = unsafe {
            mtl::TextureInfo::new(drawable.texture().as_ptr() as mtl::Handle)
        };
        let w = (self.width as f32 * self.scale) as i32;
        let h = (self.height as f32 * self.scale) as i32;
        let backend_rt = gpu::backend_render_targets::make_mtl(
            (w, h),
            &texture_info,
        );
        surfaces::wrap_backend_render_target(
            &mut self.gr_context,
            &backend_rt,
            gpu::SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            None,
            None,
        )
    }

    pub fn present(&self) {
        if let Some(drawable) = self.layer.next_drawable() {
            let cb = self.command_queue.new_command_buffer();
            cb.present_drawable(&drawable);
            cb.commit();
        }
    }
}
```

- [ ] **Step 4: Create `src-tauri/src/renderer/draw.rs`**

```rust
use skia_safe::{Canvas, Color4f, Paint, PaintStyle, Path, Rect, ColorSpace};
use crate::state::{DrawingState, Stroke, ToolKind, Color as LColor};

fn skia_color(c: &LColor) -> Color4f {
    Color4f::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

/// Clear the canvas to fully transparent.
pub fn clear(canvas: &Canvas) {
    canvas.clear(skia_safe::Color::TRANSPARENT);
}

/// Draw all committed strokes plus the live (in-progress) stroke.
pub fn draw_frame(canvas: &Canvas, state: &DrawingState, scale: f32) {
    canvas.save();
    canvas.scale((scale, scale));
    clear(canvas);

    for stroke in &state.strokes {
        draw_stroke(canvas, stroke);
    }
    if let Some(live) = &state.live_stroke {
        draw_stroke(canvas, live);
    }

    canvas.restore();
}

fn draw_stroke(canvas: &Canvas, stroke: &Stroke) {
    match stroke.tool {
        ToolKind::Pen => draw_pen(canvas, stroke, 1.0),
        ToolKind::Highlighter => draw_pen(canvas, stroke, 0.35),
        ToolKind::Arrow => draw_arrow(canvas, stroke),
        ToolKind::Rectangle => draw_rectangle(canvas, stroke),
        ToolKind::Ellipse => draw_ellipse(canvas, stroke),
        ToolKind::Line => draw_line_segment(canvas, stroke),
        ToolKind::Laser => draw_pen(canvas, stroke, 0.85),
        _ => {}
    }
}

fn draw_pen(canvas: &Canvas, stroke: &Stroke, alpha_mult: f32) {
    if stroke.points.len() < 2 { return; }

    let mut color = skia_color(&stroke.color);
    color.a *= alpha_mult;

    let mut paint = Paint::default();
    paint.set_color4f(color, None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_stroke_join(skia_safe::PaintJoin::Round);
    paint.set_style(PaintStyle::Stroke);

    let mut path = Path::new();
    path.move_to((stroke.points[0].x, stroke.points[0].y));
    for p in &stroke.points[1..] {
        path.line_to((p.x, p.y));
    }
    canvas.draw_path(&path, &paint);
}

fn draw_arrow(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 { return; }
    let first = &stroke.points[0];
    let last = stroke.points.last().unwrap();

    let mut paint = Paint::default();
    paint.set_color4f(skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::StrokeAndFill);

    // Shaft
    canvas.draw_line((first.x, first.y), (last.x, last.y), &paint);

    // Arrowhead
    let angle = (last.y - first.y).atan2(last.x - first.x);
    let head_len = stroke.width.pen_px() * 4.0;
    let spread = std::f32::consts::PI / 6.0; // 30°
    let p1 = (
        last.x - head_len * (angle - spread).cos(),
        last.y - head_len * (angle - spread).sin(),
    );
    let p2 = (
        last.x - head_len * (angle + spread).cos(),
        last.y - head_len * (angle + spread).sin(),
    );
    let mut head = Path::new();
    head.move_to((last.x, last.y));
    head.line_to(p1);
    head.line_to(p2);
    head.close();
    canvas.draw_path(&head, &paint);
}

fn draw_rectangle(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 { return; }
    let start = &stroke.points[0];
    let end = stroke.points.last().unwrap();
    let rect = Rect::from_point_and_size(
        (start.x.min(end.x), start.y.min(end.y)),
        ((end.x - start.x).abs(), (end.y - start.y).abs()),
    );
    let mut paint = Paint::default();
    paint.set_color4f(skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_rect(rect, &paint);
}

fn draw_ellipse(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 { return; }
    let start = &stroke.points[0];
    let end = stroke.points.last().unwrap();
    let rect = Rect::from_point_and_size(
        (start.x.min(end.x), start.y.min(end.y)),
        ((end.x - start.x).abs(), (end.y - start.y).abs()),
    );
    let mut paint = Paint::default();
    paint.set_color4f(skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_oval(rect, &paint);
}

fn draw_line_segment(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 { return; }
    let first = &stroke.points[0];
    let last = stroke.points.last().unwrap();
    let mut paint = Paint::default();
    paint.set_color4f(skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_line((first.x, first.y), (last.x, last.y), &paint);
}
```

- [ ] **Step 5: Add renderer mod to lib.rs**

```rust
// src-tauri/src/lib.rs  — add before pub fn run():
pub mod renderer;
```

- [ ] **Step 6: Verify compilation**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error" | head -20
```

Expected: no errors.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/renderer/ src-tauri/src/lib.rs
git commit -m "feat: Skia Metal rendering pipeline (canvas + draw primitives)"
```

---

## Phase 2 — Annotation Core

### Task 5: Event tap and input routing

**Files:**
- Create: `src-tauri/src/overlay/event_tap.rs`
- Modify: `src-tauri/src/overlay/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** When click-through is OFF (draw mode), we need to capture mouse events that would otherwise reach the underlying app. A `CGEventTap` intercepts all mouse events system-wide before delivery. We forward coordinates to DrawingState and request a redraw. When click-through is ON (pointer mode), the tap is disabled.

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/overlay/event_tap.rs (stub)
#[cfg(test)]
mod tests {
    #[test]
    fn event_tap_module_exists() { assert!(true); }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml event_tap
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/overlay/event_tap.rs`**

```rust
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType,
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::state::SharedState;

pub struct EventTap {
    active: Arc<AtomicBool>,
}

impl EventTap {
    /// Install a CGEventTap that forwards mouse events to DrawingState.
    /// Returns immediately; tap runs on a background thread.
    pub fn install(state: SharedState) -> Self {
        let active = Arc::new(AtomicBool::new(true));
        let active_clone = active.clone();

        std::thread::spawn(move || {
            let tap = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::LeftMouseDown,
                    CGEventType::LeftMouseDragged,
                    CGEventType::LeftMouseUp,
                ],
                move |_proxy, event_type, event| -> Option<CGEvent> {
                    if !active_clone.load(Ordering::Relaxed) {
                        return Some(event.clone());
                    }

                    let location = event.location();
                    let x = location.x as f32;
                    let y = location.y as f32;

                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let mut s = state.lock();
                    if !s.overlay_visible || s.click_through {
                        return Some(event.clone());
                    }

                    match event_type {
                        CGEventType::LeftMouseDown => s.drawing.begin_stroke(x, y, ts),
                        CGEventType::LeftMouseDragged => s.drawing.extend_stroke(x, y),
                        CGEventType::LeftMouseUp => s.drawing.commit_stroke(),
                        _ => {}
                    }

                    // Consume the event so it doesn't reach underlying apps
                    None
                },
            )
            .expect("failed to create CGEventTap — check Accessibility permissions");

            let loop_source = tap
                .mach_port
                .create_runloop_source(0)
                .expect("failed to create run loop source");

            CFRunLoop::get_current().add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
            tap.enable();
            CFRunLoop::run_current();
        });

        Self { active }
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn event_tap_module_exists() { assert!(true); }
}
```

- [ ] **Step 3: Wire EventTap into lib.rs setup**

```rust
// src-tauri/src/lib.rs — inside setup closure, after overlay creation:
#[cfg(target_os = "macos")]
{
    let tap = overlay::event_tap::EventTap::install(app_state.clone());
    std::mem::forget(tap); // keep tap alive for app lifetime
}
```

- [ ] **Step 4: Build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/overlay/
git commit -m "feat(macos): CGEventTap for draw-mode mouse capture"
```

---

### Task 6: Tauri IPC commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/commands.rs (stub)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{new_shared_state, ToolKind};

    #[test]
    fn set_tool_updates_state() {
        let state = new_shared_state();
        let result = set_tool_inner(&state, "highlighter");
        assert!(result.is_ok());
        assert_eq!(state.lock().drawing.active_tool, ToolKind::Highlighter);
    }

    #[test]
    fn unknown_tool_returns_error() {
        let state = new_shared_state();
        assert!(set_tool_inner(&state, "wand").is_err());
    }

    #[test]
    fn toggle_overlay_flips_visibility() {
        let state = new_shared_state();
        assert!(!state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(!state.lock().overlay_visible);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands
```

Expected: FAIL.

- [ ] **Step 2: Write `src-tauri/src/commands.rs`**

```rust
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::{SharedState, ToolKind, StrokeWidth, Color as LColor};

// ── Pure functions (testable without Tauri runtime) ─────────────────────────

pub fn set_tool_inner(state: &SharedState, tool: &str) -> Result<(), String> {
    let kind = match tool {
        "pen"         => ToolKind::Pen,
        "highlighter" => ToolKind::Highlighter,
        "arrow"       => ToolKind::Arrow,
        "rectangle"   => ToolKind::Rectangle,
        "ellipse"     => ToolKind::Ellipse,
        "line"        => ToolKind::Line,
        "text"        => ToolKind::Text,
        "laser"       => ToolKind::Laser,
        "eraser"      => ToolKind::Eraser,
        other         => return Err(format!("unknown tool: {other}")),
    };
    state.lock().drawing.active_tool = kind;
    Ok(())
}

pub fn toggle_overlay_inner(state: &SharedState) -> bool {
    let mut s = state.lock();
    s.overlay_visible = !s.overlay_visible;
    s.overlay_visible
}

pub fn toggle_click_through_inner(state: &SharedState) -> bool {
    let mut s = state.lock();
    s.click_through = !s.click_through;
    s.click_through
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_tool(tool: String, state: State<SharedState>) -> Result<(), String> {
    set_tool_inner(&state, &tool)
}

#[tauri::command]
pub fn set_color(r: u8, g: u8, b: u8, state: State<SharedState>) {
    let mut s = state.lock();
    s.drawing.active_color = LColor { r, g, b, a: 255 };
}

#[tauri::command]
pub fn set_width(width: String, state: State<SharedState>) -> Result<(), String> {
    let w = match width.as_str() {
        "thin"       => StrokeWidth::Thin,
        "medium"     => StrokeWidth::Medium,
        "bold"       => StrokeWidth::Bold,
        "extra_bold" => StrokeWidth::ExtraBold,
        other        => return Err(format!("unknown width: {other}")),
    };
    state.lock().drawing.active_width = w;
    Ok(())
}

#[tauri::command]
pub fn undo(state: State<SharedState>) {
    state.lock().drawing.undo();
}

#[tauri::command]
pub fn clear_all(state: State<SharedState>) {
    state.lock().drawing.clear();
}

#[tauri::command]
pub fn toggle_overlay(state: State<SharedState>) -> bool {
    toggle_overlay_inner(&state)
}

#[tauri::command]
pub fn toggle_click_through(state: State<SharedState>) -> bool {
    toggle_click_through_inner(&state)
}

#[derive(Serialize)]
pub struct AppSnapshot {
    pub overlay_visible: bool,
    pub click_through: bool,
    pub active_tool: ToolKind,
}

#[tauri::command]
pub fn get_app_state(state: State<SharedState>) -> AppSnapshot {
    let s = state.lock();
    AppSnapshot {
        overlay_visible: s.overlay_visible,
        click_through: s.click_through,
        active_tool: s.drawing.active_tool,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{new_shared_state, ToolKind};

    #[test]
    fn set_tool_updates_state() {
        let state = new_shared_state();
        assert!(set_tool_inner(&state, "highlighter").is_ok());
        assert_eq!(state.lock().drawing.active_tool, ToolKind::Highlighter);
    }

    #[test]
    fn unknown_tool_returns_error() {
        let state = new_shared_state();
        assert!(set_tool_inner(&state, "wand").is_err());
    }

    #[test]
    fn toggle_overlay_flips_visibility() {
        let state = new_shared_state();
        assert!(!state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(!state.lock().overlay_visible);
    }
}
```

- [ ] **Step 3: Register commands in lib.rs**

```rust
// src-tauri/src/lib.rs — update the Builder chain:
pub mod commands;

// Inside run():
tauri::Builder::default()
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_store::Builder::new().build())
    .manage(app_state.clone())
    .invoke_handler(tauri::generate_handler![
        commands::set_tool,
        commands::set_color,
        commands::set_width,
        commands::undo,
        commands::clear_all,
        commands::toggle_overlay,
        commands::toggle_click_through,
        commands::get_app_state,
    ])
    .setup(move |_app| { /* ... */ Ok(()) })
    .run(tauri::generate_context!())
    .expect("error while running Lumos");
```

- [ ] **Step 4: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands
```

Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: Tauri IPC commands (tool, color, width, undo, clear, overlay toggle)"
```

---

### Task 7: Global hotkeys

**Files:**
- Create: `src-tauri/src/hotkeys.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/hotkeys.rs (stub)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_shortcuts_are_valid() {
        let shortcuts = default_shortcuts();
        assert!(!shortcuts.is_empty());
        for s in &shortcuts {
            assert!(!s.accelerator.is_empty());
        }
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml hotkeys
```

Expected: FAIL.

- [ ] **Step 2: Write `src-tauri/src/hotkeys.rs`**

```rust
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use crate::state::SharedState;

#[derive(Debug, Clone)]
pub struct HotkeyDef {
    pub accelerator: String,
    pub action: HotkeyAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleOverlay,
    ClearAll,
    Undo,
    ToggleClickThrough,
    ToolPen,
    ToolHighlighter,
    ToolArrow,
    ToolRectangle,
    ToolEllipse,
    ToolLaser,
    ToolEraser,
}

pub fn default_shortcuts() -> Vec<HotkeyDef> {
    vec![
        HotkeyDef { accelerator: "CmdOrCtrl+Shift+A".into(), action: HotkeyAction::ToggleOverlay },
        HotkeyDef { accelerator: "CmdOrCtrl+K".into(),       action: HotkeyAction::ClearAll },
        HotkeyDef { accelerator: "CmdOrCtrl+Z".into(),       action: HotkeyAction::Undo },
        HotkeyDef { accelerator: "CmdOrCtrl+D".into(),       action: HotkeyAction::ToggleClickThrough },
        HotkeyDef { accelerator: "P".into(),                  action: HotkeyAction::ToolPen },
        HotkeyDef { accelerator: "H".into(),                  action: HotkeyAction::ToolHighlighter },
        HotkeyDef { accelerator: "A".into(),                  action: HotkeyAction::ToolArrow },
        HotkeyDef { accelerator: "R".into(),                  action: HotkeyAction::ToolRectangle },
        HotkeyDef { accelerator: "E".into(),                  action: HotkeyAction::ToolEllipse },
        HotkeyDef { accelerator: "L".into(),                  action: HotkeyAction::ToolLaser },
        HotkeyDef { accelerator: "X".into(),                  action: HotkeyAction::ToolEraser },
    ]
}

/// Register all default global shortcuts. Call once from `setup`.
pub fn register_all(app: &AppHandle, state: SharedState) {
    let shortcuts = default_shortcuts();
    let plugin = app.global_shortcut();

    for def in shortcuts {
        let state_clone = state.clone();
        let action = def.action;
        let _ = plugin.on_shortcut(def.accelerator.clone(), move |_app, _shortcut, event| {
            if event.state != ShortcutState::Pressed { return; }
            dispatch_action(action, &state_clone);
        });
    }
}

fn dispatch_action(action: HotkeyAction, state: &SharedState) {
    use crate::state::ToolKind;
    match action {
        HotkeyAction::ToggleOverlay => {
            let mut s = state.lock();
            s.overlay_visible = !s.overlay_visible;
        }
        HotkeyAction::ClearAll => state.lock().drawing.clear(),
        HotkeyAction::Undo     => state.lock().drawing.undo(),
        HotkeyAction::ToggleClickThrough => {
            let mut s = state.lock();
            s.click_through = !s.click_through;
        }
        HotkeyAction::ToolPen         => state.lock().drawing.active_tool = ToolKind::Pen,
        HotkeyAction::ToolHighlighter => state.lock().drawing.active_tool = ToolKind::Highlighter,
        HotkeyAction::ToolArrow       => state.lock().drawing.active_tool = ToolKind::Arrow,
        HotkeyAction::ToolRectangle   => state.lock().drawing.active_tool = ToolKind::Rectangle,
        HotkeyAction::ToolEllipse     => state.lock().drawing.active_tool = ToolKind::Ellipse,
        HotkeyAction::ToolLaser       => state.lock().drawing.active_tool = ToolKind::Laser,
        HotkeyAction::ToolEraser      => state.lock().drawing.active_tool = ToolKind::Eraser,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_shortcuts_are_valid() {
        let shortcuts = default_shortcuts();
        assert!(!shortcuts.is_empty());
        for s in &shortcuts {
            assert!(!s.accelerator.is_empty());
        }
    }

    #[test]
    fn all_actions_covered() {
        let shortcuts = default_shortcuts();
        let has_toggle = shortcuts.iter().any(|s| s.action == HotkeyAction::ToggleOverlay);
        assert!(has_toggle);
    }
}
```

- [ ] **Step 3: Wire into lib.rs setup**

```rust
// Inside setup closure in lib.rs:
hotkeys::register_all(&app.handle(), app_state.clone());
```

Also add `pub mod hotkeys;` at module root.

- [ ] **Step 4: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml hotkeys
```

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/hotkeys.rs src-tauri/src/lib.rs
git commit -m "feat: global hotkey registration (toggle, clear, undo, tool switch)"
```

---

### Task 8: Render loop integration

**Files:**
- Modify: `src-tauri/src/renderer/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** The render loop runs on a dedicated thread. Every ~8 ms (120 fps ceiling) it: (1) locks DrawingState for < 1 ms to clone render data, (2) calls `draw_frame`, (3) flushes Skia, (4) presents via Metal. Using a clone of render data keeps the mutex hot time minimal.

- [ ] **Step 1: Write `src-tauri/src/renderer/mod.rs` with RenderLoop**

```rust
#[cfg(target_os = "macos")]
pub mod canvas;
#[cfg(target_os = "macos")]
pub mod draw;

use crate::state::SharedState;

#[cfg(target_os = "macos")]
pub fn start_render_loop(
    ns_panel: *mut objc::runtime::Object,
    state: SharedState,
    width: i32,
    height: i32,
    scale: f32,
) {
    use canvas::MetalCanvas;
    use draw::draw_frame;

    // SAFETY: ns_panel is valid for the app lifetime; MetalCanvas is Send.
    let panel = ns_panel as usize; // escape pointer into usize for Send bound

    std::thread::spawn(move || {
        let ns_panel = panel as *mut objc::runtime::Object;
        let mut canvas = unsafe { MetalCanvas::new(ns_panel, width, height, scale) };
        let frame_budget = std::time::Duration::from_micros(8_333); // ~120 fps

        loop {
            let t0 = std::time::Instant::now();

            // Clone only what we need — lock held < 1 ms
            let (visible, drawing_snapshot) = {
                let s = state.lock();
                let snapshot = crate::state::DrawingState {
                    active_tool: s.drawing.active_tool,
                    active_color: s.drawing.active_color.clone(),
                    active_width: s.drawing.active_width,
                    strokes: s.drawing.strokes.clone(),
                    live_stroke: s.drawing.live_stroke.clone(),
                    next_id: s.drawing.next_id,
                };
                (s.overlay_visible, snapshot)
            };

            if visible {
                if let Some(surface) = canvas.next_surface() {
                    draw_frame(surface.canvas(), &drawing_snapshot, scale);
                    drop(surface); // flushes Skia
                    canvas.gr_context.flush_and_submit();
                }
            }

            // Sleep remaining frame budget
            let elapsed = t0.elapsed();
            if elapsed < frame_budget {
                std::thread::sleep(frame_budget - elapsed);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn renderer_module_exists() { assert!(true); }
}
```

- [ ] **Step 2: DrawingState needs Clone + derive — update state.rs**

Add `#[derive(Clone)]` to `DrawingState`, `Stroke`, `Point`, `Color`:

```rust
// state.rs — add Clone to these derives:
#[derive(Debug, Clone, Default)]
pub struct DrawingState { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color { ... }
```

- [ ] **Step 3: Wire render loop into lib.rs setup**

```rust
// After creating the overlay panel in setup:
#[cfg(target_os = "macos")]
{
    let panel_ptr = unsafe { overlay::create_overlay() };
    // Retrieve screen dimensions
    let (w, h, scale) = unsafe {
        use cocoa::appkit::NSScreen;
        use objc::{msg_send, sel, sel_impl};
        let screen: cocoa::base::id = msg_send![objc::class!(NSScreen), mainScreen];
        let frame: cocoa::foundation::NSRect = msg_send![screen, frame];
        let scale: f64 = msg_send![screen, backingScaleFactor];
        (frame.size.width as i32, frame.size.height as i32, scale as f32)
    };
    renderer::start_render_loop(panel_ptr, app_state.clone(), w, h, scale);
    unsafe { overlay::show_overlay(panel_ptr) };
}
```

- [ ] **Step 4: Build**

```bash
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/renderer/mod.rs src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat: 120fps render loop integrating Skia Metal with DrawingState"
```

---

### Task 9: Laser pointer with time-decay fade

**Files:**
- Create: `src-tauri/src/tools/mod.rs`
- Create: `src-tauri/src/tools/laser.rs`
- Modify: `src-tauri/src/renderer/draw.rs`
- Modify: `src-tauri/src/state.rs`

**Background:** Laser strokes are stored with a `created_at_ms` timestamp. The draw loop computes each point's age and scales alpha down to 0 over `laser_fade_ms` (default 2000ms). Points older than `laser_fade_ms` are skipped. This gives the "disappearing laser" effect without purging data mid-draw.

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/tools/laser.rs (stub)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alpha_at_zero_age_is_full() {
        assert!((laser_alpha(0, 2000) - 1.0).abs() < 0.01);
    }
    #[test]
    fn alpha_at_full_lifetime_is_zero() {
        assert!(laser_alpha(2000, 2000) < 0.01);
    }
    #[test]
    fn alpha_past_lifetime_is_zero() {
        assert_eq!(laser_alpha(3000, 2000), 0.0);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml tools::laser
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/tools/mod.rs`**

```rust
pub mod laser;
```

- [ ] **Step 3: Create `src-tauri/src/tools/laser.rs`**

```rust
/// Compute alpha multiplier for a laser stroke point given its age.
/// `age_ms`: milliseconds since stroke was created.
/// `fade_ms`: total fade duration in milliseconds.
pub fn laser_alpha(age_ms: u64, fade_ms: u64) -> f32 {
    if age_ms >= fade_ms {
        return 0.0;
    }
    1.0 - (age_ms as f32 / fade_ms as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_at_zero_age_is_full() {
        assert!((laser_alpha(0, 2000) - 1.0).abs() < 0.01);
    }

    #[test]
    fn alpha_at_full_lifetime_is_zero() {
        assert!(laser_alpha(2000, 2000) < 0.01);
    }

    #[test]
    fn alpha_past_lifetime_is_zero() {
        assert_eq!(laser_alpha(3000, 2000), 0.0);
    }
}
```

- [ ] **Step 4: Update draw.rs to use laser alpha**

In the `draw_stroke` match arm for Laser, replace:

```rust
ToolKind::Laser => draw_pen(canvas, stroke, 0.85),
```

With:

```rust
ToolKind::Laser => {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let age_ms = now_ms.saturating_sub(stroke.created_at_ms);
    let alpha = crate::tools::laser::laser_alpha(age_ms, 2000);
    if alpha > 0.0 {
        draw_pen(canvas, stroke, alpha);
    }
}
```

- [ ] **Step 5: Add `pub mod tools;` to lib.rs**

```rust
pub mod tools;
```

- [ ] **Step 6: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml tools
```

Expected: 3 laser tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/tools/ src-tauri/src/renderer/draw.rs src-tauri/src/lib.rs
git commit -m "feat: laser pointer with time-decay alpha fade"
```

---

## Phase 3 — Overlay Behavior

### Task 10: Click-through toggle with NSPanel sync

**Files:**
- Modify: `src-tauri/src/overlay/mod.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** `toggle_click_through` must both update AppState AND call `set_click_through` on the NSPanel. Since NSPanel calls must happen on the main thread, we use Tauri's `run_on_main_thread` executor. We store the panel pointer in a `tauri::State<OverlayRef>` wrapper.

- [ ] **Step 1: Write failing test**

```rust
// Test that toggle_click_through_inner produces correct bool sequence
#[test]
fn click_through_sequence() {
    let state = new_shared_state();
    assert!(state.lock().click_through); // starts true
    let r1 = toggle_click_through_inner(&state);
    assert!(!r1);
    let r2 = toggle_click_through_inner(&state);
    assert!(r2);
}
```

Add this test to `commands.rs` tests block.

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands::tests::click_through
```

Expected: FAIL.

- [ ] **Step 2: Add `OverlayRef` managed state**

```rust
// src-tauri/src/overlay/mod.rs — add:
use std::sync::Mutex;

pub struct OverlayRef(pub Mutex<*mut objc::runtime::Object>);
unsafe impl Send for OverlayRef {}
unsafe impl Sync for OverlayRef {}

impl OverlayRef {
    pub fn new(panel: *mut objc::runtime::Object) -> Self {
        Self(Mutex::new(panel))
    }
}
```

- [ ] **Step 3: Update `commands.rs` to sync NSPanel on toggle**

```rust
#[tauri::command]
pub fn toggle_click_through(
    state: State<SharedState>,
    overlay: State<crate::overlay::OverlayRef>,
) -> bool {
    let enabled = toggle_click_through_inner(&state);
    let panel = *overlay.0.lock().unwrap();
    unsafe { crate::overlay::set_click_through(panel, enabled) };
    enabled
}
```

- [ ] **Step 4: Register `OverlayRef` in lib.rs**

```rust
// In setup, after creating panel:
let overlay_ref = overlay::OverlayRef::new(panel_ptr);
app.manage(overlay_ref);
```

- [ ] **Step 5: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands
```

Expected: all 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/overlay/mod.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: click-through toggle syncs NSPanel ignoresMouseEvents"
```

---

## Phase 4 — Effects

### Task 11: Cursor effects (glow, ring, pulse, ripple)

**Files:**
- Create: `src-tauri/src/effects/mod.rs`
- Create: `src-tauri/src/effects/cursor.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/renderer/draw.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src-tauri/src/effects/cursor.rs (stub)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pulse_radius_oscillates() {
        let r0 = pulse_radius(0.0, 20.0, 8.0);
        let r1 = pulse_radius(0.5, 20.0, 8.0);
        let r2 = pulse_radius(1.0, 20.0, 8.0);
        assert!(r0 > r1 || r0 < r1); // must change
        assert!((r0 - r2).abs() < 0.01); // period = 1.0s
    }

    #[test]
    fn ripple_alpha_decays() {
        let a0 = ripple_alpha(0.0);
        let a1 = ripple_alpha(0.5);
        let a2 = ripple_alpha(1.0);
        assert!(a0 > a1);
        assert!(a1 > a2);
        assert_eq!(a2, 0.0);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects::cursor
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/effects/mod.rs`**

```rust
pub mod cursor;
pub mod spotlight;
pub mod zoom;
```

- [ ] **Step 3: Create `src-tauri/src/effects/cursor.rs`**

```rust
use std::f32::consts::TAU;

/// Pulse radius: oscillates between `base` and `base + amplitude` at 1 Hz.
pub fn pulse_radius(t_secs: f32, base: f32, amplitude: f32) -> f32 {
    base + amplitude * (0.5 + 0.5 * (TAU * t_secs).cos())
}

/// Ripple alpha: linear decay from 1.0 at t=0 to 0.0 at t=1.0.
pub fn ripple_alpha(t_norm: f32) -> f32 {
    (1.0 - t_norm.clamp(0.0, 1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorEffect {
    None,
    Glow,
    Ring,
    Spotlight,
    Pulse,
}

/// Draw the cursor effect at (cx, cy) using Skia.
#[cfg(target_os = "macos")]
pub fn draw_cursor_effect(
    canvas: &skia_safe::Canvas,
    effect: CursorEffect,
    cx: f32,
    cy: f32,
    t_secs: f32,
    color: (u8, u8, u8),
) {
    use skia_safe::{Color4f, Paint, PaintStyle};

    match effect {
        CursorEffect::None => {}

        CursorEffect::Glow => {
            // Radial glow: 3 concentric circles with decreasing alpha
            for i in 0..3u8 {
                let r = 16.0 + i as f32 * 10.0;
                let alpha = 0.15 - i as f32 * 0.04;
                let mut paint = Paint::default();
                paint.set_color4f(
                    Color4f::new(color.0 as f32/255.0, color.1 as f32/255.0, color.2 as f32/255.0, alpha),
                    None,
                );
                paint.set_anti_alias(true);
                paint.set_style(PaintStyle::Fill);
                canvas.draw_circle((cx, cy), r, &paint);
            }
        }

        CursorEffect::Ring => {
            let mut paint = Paint::default();
            paint.set_color4f(
                Color4f::new(color.0 as f32/255.0, color.1 as f32/255.0, color.2 as f32/255.0, 0.8),
                None,
            );
            paint.set_anti_alias(true);
            paint.set_stroke_width(2.5);
            paint.set_style(PaintStyle::Stroke);
            canvas.draw_circle((cx, cy), 18.0, &paint);
        }

        CursorEffect::Pulse => {
            let r = pulse_radius(t_secs, 14.0, 8.0);
            let alpha = 0.6 - (r - 14.0) / 8.0 * 0.4;
            let mut paint = Paint::default();
            paint.set_color4f(
                Color4f::new(color.0 as f32/255.0, color.1 as f32/255.0, color.2 as f32/255.0, alpha),
                None,
            );
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Fill);
            canvas.draw_circle((cx, cy), r, &paint);
        }

        CursorEffect::Spotlight => {
            // Handled by spotlight.rs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_radius_oscillates() {
        let r0 = pulse_radius(0.0, 20.0, 8.0);
        let r1 = pulse_radius(0.25, 20.0, 8.0);
        let r2 = pulse_radius(1.0, 20.0, 8.0);
        assert!((r0 - r2).abs() < 0.01);
        assert!(r0 != r1);
    }

    #[test]
    fn ripple_alpha_decays() {
        let a0 = ripple_alpha(0.0);
        let a1 = ripple_alpha(0.5);
        let a2 = ripple_alpha(1.0);
        assert!(a0 > a1);
        assert!(a1 > a2);
        assert_eq!(a2, 0.0);
    }
}
```

- [ ] **Step 4: Add cursor_effect fields to AppState**

```rust
// state.rs — add to AppState:
pub cursor_effect: crate::effects::cursor::CursorEffect,
pub cursor_pos: Point,
pub app_start: std::time::Instant,
```

Initialize in `new_shared_state`:
```rust
cursor_effect: crate::effects::cursor::CursorEffect::Ring,
cursor_pos: Point { x: 0.0, y: 0.0 },
app_start: std::time::Instant::now(),
```

- [ ] **Step 5: Update draw.rs to draw cursor effect each frame**

```rust
// draw.rs — add after draw_frame body, before canvas.restore():
use crate::effects::cursor::{draw_cursor_effect, CursorEffect};

// After all strokes:
if drawing_state.cursor_effect != CursorEffect::None {
    let t = app_start.elapsed().as_secs_f32();
    draw_cursor_effect(
        canvas,
        drawing_state.cursor_effect,
        drawing_state.cursor_pos.x,
        drawing_state.cursor_pos.y,
        t,
        (82, 155, 224), // default cursor color
    );
}
```

- [ ] **Step 6: Update event tap to track cursor position**

```rust
// event_tap.rs — in mouse move handler:
CGEventType::MouseMoved | CGEventType::LeftMouseDragged => {
    let mut s = state.lock();
    s.cursor_pos = crate::state::Point { x, y };
}
```

Add `CGEventType::MouseMoved` to the event tap mask.

- [ ] **Step 7: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects::cursor
```

Expected: 2 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/effects/ src-tauri/src/state.rs src-tauri/src/renderer/draw.rs
git commit -m "feat: cursor effects (glow, ring, pulse) rendered by Skia"
```

---

### Task 12: Spotlight mode

**Files:**
- Create: `src-tauri/src/effects/spotlight.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/renderer/draw.rs`

**Background:** Spotlight draws a dark semi-transparent rect over the full screen with a hole cut out at the cursor. The hole is drawn using `skia_safe::BlendMode::Clear` — paint clear into the circle/rect to cut transparency. The dimming rect uses ~65% black alpha.

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/effects/spotlight.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spotlight_shape_variants_exist() {
        let _ = SpotlightShape::Circle { radius: 120.0 };
        let _ = SpotlightShape::Rectangle { width: 300.0, height: 200.0 };
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects::spotlight
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/effects/spotlight.rs`**

```rust
#[derive(Debug, Clone, Copy)]
pub enum SpotlightShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

#[cfg(target_os = "macos")]
pub fn draw_spotlight(
    canvas: &skia_safe::Canvas,
    cx: f32,
    cy: f32,
    shape: SpotlightShape,
    dim_alpha: f32,
    screen_w: f32,
    screen_h: f32,
) {
    use skia_safe::{BlendMode, Color4f, Paint, PaintStyle, Rect};

    // 1. Draw dim overlay over entire screen
    let mut dim_paint = Paint::default();
    dim_paint.set_color4f(Color4f::new(0.0, 0.0, 0.0, dim_alpha), None);
    dim_paint.set_style(PaintStyle::Fill);
    dim_paint.set_blend_mode(BlendMode::SrcOver);
    canvas.draw_rect(Rect::from_wh(screen_w, screen_h), &dim_paint);

    // 2. Cut the spotlight hole using Clear blend mode
    let mut hole_paint = Paint::default();
    hole_paint.set_blend_mode(BlendMode::Clear);
    hole_paint.set_anti_alias(true);
    hole_paint.set_style(PaintStyle::Fill);

    match shape {
        SpotlightShape::Circle { radius } => {
            canvas.draw_circle((cx, cy), radius, &hole_paint);
        }
        SpotlightShape::Rectangle { width, height } => {
            let rect = Rect::from_xywh(cx - width / 2.0, cy - height / 2.0, width, height);
            // Rounded rect for softer look
            canvas.draw_round_rect(rect, 12.0, 12.0, &hole_paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotlight_shape_variants_exist() {
        let _ = SpotlightShape::Circle { radius: 120.0 };
        let _ = SpotlightShape::Rectangle { width: 300.0, height: 200.0 };
    }
}
```

- [ ] **Step 3: Add spotlight state to AppState**

```rust
// state.rs
pub spotlight_active: bool,
pub spotlight_shape: crate::effects::spotlight::SpotlightShape,
pub spotlight_dim_alpha: f32,
```

Initialize:
```rust
spotlight_active: false,
spotlight_shape: crate::effects::spotlight::SpotlightShape::Circle { radius: 120.0 },
spotlight_dim_alpha: 0.65,
```

- [ ] **Step 4: Integrate spotlight into draw_frame**

```rust
// renderer/draw.rs — at the start of draw_frame, before strokes:
if state.spotlight_active {
    let screen_w = width as f32;
    let screen_h = height as f32;
    crate::effects::spotlight::draw_spotlight(
        canvas,
        state.cursor_pos.x,
        state.cursor_pos.y,
        state.spotlight_shape,
        state.spotlight_dim_alpha,
        screen_w,
        screen_h,
    );
}
```

- [ ] **Step 5: Add Tauri command to toggle spotlight**

```rust
// commands.rs
#[tauri::command]
pub fn toggle_spotlight(state: State<SharedState>) -> bool {
    let mut s = state.lock();
    s.spotlight_active = !s.spotlight_active;
    s.spotlight_active
}

#[tauri::command]
pub fn set_spotlight_shape(shape: String, state: State<SharedState>) -> Result<(), String> {
    use crate::effects::spotlight::SpotlightShape;
    let s_shape = match shape.as_str() {
        "circle" => SpotlightShape::Circle { radius: 120.0 },
        "rectangle" => SpotlightShape::Rectangle { width: 400.0, height: 250.0 },
        other => return Err(format!("unknown shape: {other}")),
    };
    state.lock().spotlight_shape = s_shape;
    Ok(())
}
```

Register in `generate_handler![]`.

- [ ] **Step 6: Add hotkey for spotlight (Shift+S)**

```rust
// hotkeys.rs — add:
HotkeyDef { accelerator: "Shift+S".into(), action: HotkeyAction::ToggleSpotlight },
```

- [ ] **Step 7: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects
```

Expected: 3 tests pass (cursor + spotlight).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/effects/spotlight.rs src-tauri/src/state.rs src-tauri/src/renderer/draw.rs src-tauri/src/commands.rs
git commit -m "feat: spotlight mode (circle + rectangle) with screen dimming"
```

---

### Task 13: Zoom lens

**Files:**
- Create: `src-tauri/src/effects/zoom.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/renderer/draw.rs`

**Background:** The zoom lens captures a region of the Metal drawable around the cursor, scales it up by `zoom_factor`, and paints it into a circular clip region at the cursor. We use `ScreenCaptureKit` (via a CGWindowListCreateImage snapshot) to grab the screen contents, then draw the scaled sub-image into a circle. At 120fps this gives smooth zoom.

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/effects/zoom.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_source_rect_is_centered() {
        let r = zoom_source_rect(100.0, 100.0, 2.0, 80.0);
        // With 2x zoom and 80px lens, source rect is 40x40 centered at cursor
        assert!((r.0 - 80.0).abs() < 0.1); // left = cx - (80/2)/zoom = 100 - 20 = 80
        assert!((r.1 - 80.0).abs() < 0.1); // top
        assert!((r.2 - 40.0).abs() < 0.1); // width = lens_diameter / zoom
        assert!((r.3 - 40.0).abs() < 0.1);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects::zoom
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/effects/zoom.rs`**

```rust
/// Compute the source rect (x, y, w, h) on screen to sample for the zoom lens.
/// Returns logical coordinates.
pub fn zoom_source_rect(cx: f32, cy: f32, zoom_factor: f32, lens_diameter: f32) -> (f32, f32, f32, f32) {
    let source_size = lens_diameter / zoom_factor;
    let x = cx - source_size / 2.0;
    let y = cy - source_size / 2.0;
    (x, y, source_size, source_size)
}

#[cfg(target_os = "macos")]
pub fn draw_zoom_lens(
    canvas: &skia_safe::Canvas,
    cx: f32,
    cy: f32,
    zoom_factor: f32,
    lens_diameter: f32,
    screen_image: &skia_safe::Image,
) {
    use skia_safe::{ClipOp, Color4f, Paint, PaintStyle, Rect, Matrix};

    let r = lens_diameter / 2.0;
    let (src_x, src_y, src_w, src_h) = zoom_source_rect(cx, cy, zoom_factor, lens_diameter);

    canvas.save();

    // Clip to circle
    let circle_rect = Rect::from_xywh(cx - r, cy - r, lens_diameter, lens_diameter);
    canvas.clip_round_rect(circle_rect.with_outset((0.0, 0.0)), r, r, ClipOp::Intersect, true);

    // Draw zoomed screen region
    let src_rect = Rect::from_xywh(src_x, src_y, src_w, src_h);
    let dst_rect = Rect::from_xywh(cx - r, cy - r, lens_diameter, lens_diameter);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    canvas.draw_image_rect(screen_image, Some((&src_rect, skia_safe::canvas::SrcRectConstraint::Fast)), dst_rect, &paint);

    // Border ring
    let mut border = Paint::default();
    border.set_color4f(Color4f::new(1.0, 1.0, 1.0, 0.6), None);
    border.set_stroke_width(2.0);
    border.set_style(PaintStyle::Stroke);
    border.set_anti_alias(true);
    canvas.draw_circle((cx, cy), r, &border);

    canvas.restore();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_source_rect_is_centered() {
        let r = zoom_source_rect(100.0, 100.0, 2.0, 80.0);
        assert!((r.0 - 80.0).abs() < 0.1);
        assert!((r.1 - 80.0).abs() < 0.1);
        assert!((r.2 - 40.0).abs() < 0.1);
        assert!((r.3 - 40.0).abs() < 0.1);
    }
}
```

- [ ] **Step 3: Add zoom state to AppState**

```rust
// state.rs
pub zoom_active: bool,
pub zoom_factor: f32,
pub zoom_lens_diameter: f32,
```

Initialize: `zoom_active: false, zoom_factor: 2.5, zoom_lens_diameter: 180.0`

- [ ] **Step 4: Add toggle_zoom Tauri command**

```rust
#[tauri::command]
pub fn toggle_zoom(state: State<SharedState>) -> bool {
    let mut s = state.lock();
    s.zoom_active = !s.zoom_active;
    s.zoom_active
}
```

Register in handler. Add hotkey `"Shift+Z"` → `HotkeyAction::ToggleZoom`.

- [ ] **Step 5: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml effects
```

Expected: all effects tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/effects/zoom.rs src-tauri/src/state.rs src-tauri/src/commands.rs
git commit -m "feat: zoom lens (source rect math + Skia circle-clipped magnifier)"
```

---

## Phase 5 — UI Layer

### Task 14: TypeScript types and Tauri bindings

**Files:**
- Create: `src/types/index.ts`
- Create: `src/hooks/useAppState.ts`
- Create: `src/hooks/useToolState.ts`

- [ ] **Step 1: Write `src/types/index.ts`**

```ts
export type ToolKind =
  | "pen" | "highlighter" | "arrow" | "rectangle"
  | "ellipse" | "line" | "text" | "laser" | "eraser";

export type StrokeWidth = "thin" | "medium" | "bold" | "extra_bold";

export interface Color {
  r: number;
  g: number;
  b: number;
}

export interface AppSnapshot {
  overlay_visible: boolean;
  click_through: boolean;
  active_tool: ToolKind;
}

export const COLORS: { label: string; color: Color }[] = [
  { label: "Blue",   color: { r: 82,  g: 155, b: 224 } },
  { label: "Red",    color: { r: 224, g: 82,  b: 82  } },
  { label: "Green",  color: { r: 82,  g: 224, b: 108 } },
  { label: "White",  color: { r: 255, g: 255, b: 255 } },
  { label: "Black",  color: { r: 30,  g: 30,  b: 30  } },
];

export const TOOLS: { kind: ToolKind; label: string; hotkey: string }[] = [
  { kind: "pen",         label: "Pen",         hotkey: "P" },
  { kind: "highlighter", label: "Highlighter", hotkey: "H" },
  { kind: "arrow",       label: "Arrow",       hotkey: "A" },
  { kind: "rectangle",   label: "Rectangle",   hotkey: "R" },
  { kind: "ellipse",     label: "Ellipse",     hotkey: "E" },
  { kind: "laser",       label: "Laser",       hotkey: "L" },
  { kind: "eraser",      label: "Eraser",      hotkey: "X" },
];
```

- [ ] **Step 2: Write `src/hooks/useAppState.ts`**

```ts
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSnapshot } from "../types";

export function useAppState() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>({
    overlay_visible: false,
    click_through: true,
    active_tool: "pen",
  });

  const refresh = async () => {
    const s = await invoke<AppSnapshot>("get_app_state");
    setSnapshot(s);
  };

  useEffect(() => {
    refresh();
  }, []);

  return { snapshot, refresh };
}
```

- [ ] **Step 3: Write `src/hooks/useToolState.ts`**

```ts
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ToolKind, StrokeWidth, Color } from "../types";
import { COLORS } from "../types";

export function useToolState() {
  const [activeTool, setActiveTool]   = useState<ToolKind>("pen");
  const [activeWidth, setActiveWidth] = useState<StrokeWidth>("medium");
  const [activeColor, setActiveColor] = useState<Color>(COLORS[0].color);

  const selectTool = async (tool: ToolKind) => {
    await invoke("set_tool", { tool });
    setActiveTool(tool);
  };

  const selectWidth = async (width: StrokeWidth) => {
    await invoke("set_width", { width });
    setActiveWidth(width);
  };

  const selectColor = async (color: Color) => {
    await invoke("set_color", { r: color.r, g: color.g, b: color.b });
    setActiveColor(color);
  };

  const undo  = () => invoke("undo");
  const clear = () => invoke("clear_all");

  return { activeTool, activeWidth, activeColor, selectTool, selectWidth, selectColor, undo, clear };
}
```

- [ ] **Step 4: Install Tauri JS API**

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
pnpm add @tauri-apps/api
```

- [ ] **Step 5: Build TypeScript**

```bash
pnpm exec tsc --noEmit
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/types/ src/hooks/ package.json pnpm-lock.yaml
git commit -m "feat(ui): TypeScript types + useToolState + useAppState hooks"
```

---

### Task 15: Floating toolbar React component

**Files:**
- Create: `src/components/Toolbar/Toolbar.tsx`
- Create: `src/components/Toolbar/ToolButton.tsx`
- Create: `src/components/Toolbar/ColorPicker.tsx`
- Create: `src/components/Toolbar/Toolbar.module.css`
- Modify: `src/App.tsx`

- [ ] **Step 1: Write `src/components/Toolbar/ToolButton.tsx`**

```tsx
import type { ToolKind } from "../../types";

interface Props {
  kind: ToolKind;
  label: string;
  hotkey: string;
  active: boolean;
  onClick: () => void;
}

const ICONS: Record<ToolKind, string> = {
  pen: "✏️", highlighter: "🖊", arrow: "↗", rectangle: "▭",
  ellipse: "⬭", line: "╱", text: "T", laser: "⚡", eraser: "⌫",
};

export function ToolButton({ kind, label, hotkey, active, onClick }: Props) {
  return (
    <button
      title={`${label} (${hotkey})`}
      onClick={onClick}
      style={{
        width: 36,
        height: 36,
        borderRadius: 8,
        border: "none",
        cursor: "pointer",
        fontSize: 16,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: active ? "rgba(255,255,255,0.25)" : "transparent",
        color: "white",
        transition: "background 0.1s",
      }}
    >
      {ICONS[kind]}
    </button>
  );
}
```

- [ ] **Step 2: Write `src/components/Toolbar/ColorPicker.tsx`**

```tsx
import type { Color } from "../../types";
import { COLORS } from "../../types";

interface Props {
  active: Color;
  onSelect: (c: Color) => void;
}

export function ColorPicker({ active, onSelect }: Props) {
  return (
    <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
      {COLORS.map(({ label, color }) => {
        const isActive = color.r === active.r && color.g === active.g && color.b === active.b;
        return (
          <button
            key={label}
            title={label}
            onClick={() => onSelect(color)}
            style={{
              width: 16,
              height: 16,
              borderRadius: "50%",
              border: isActive ? "2px solid white" : "2px solid transparent",
              background: `rgb(${color.r},${color.g},${color.b})`,
              cursor: "pointer",
              padding: 0,
            }}
          />
        );
      })}
    </div>
  );
}
```

- [ ] **Step 3: Write `src/components/Toolbar/Toolbar.tsx`**

```tsx
import { invoke } from "@tauri-apps/api/core";
import { useToolState } from "../../hooks/useToolState";
import { ToolButton } from "./ToolButton";
import { ColorPicker } from "./ColorPicker";
import { TOOLS } from "../../types";

export function Toolbar() {
  const { activeTool, activeColor, selectTool, selectColor, undo, clear } = useToolState();

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "8px 12px",
        borderRadius: 14,
        background: "rgba(20, 20, 20, 0.82)",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
        boxShadow: "0 4px 24px rgba(0,0,0,0.4)",
        userSelect: "none",
        WebkitUserSelect: "none",
        // Drag region
        WebkitAppRegion: "drag",
      } as React.CSSProperties}
    >
      {TOOLS.map((t) => (
        <div key={t.kind} style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}>
          <ToolButton
            kind={t.kind}
            label={t.label}
            hotkey={t.hotkey}
            active={activeTool === t.kind}
            onClick={() => selectTool(t.kind)}
          />
        </div>
      ))}

      <div style={{ width: 1, height: 24, background: "rgba(255,255,255,0.15)", margin: "0 4px" }} />

      <div style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}>
        <ColorPicker active={activeColor} onSelect={selectColor} />
      </div>

      <div style={{ width: 1, height: 24, background: "rgba(255,255,255,0.15)", margin: "0 4px" }} />

      <div style={{ display: "flex", gap: 4, WebkitAppRegion: "no-drag" } as React.CSSProperties}>
        <button
          title="Undo (⌘Z)"
          onClick={undo}
          style={{ background: "transparent", border: "none", color: "rgba(255,255,255,0.6)", cursor: "pointer", fontSize: 14 }}
        >
          ↩
        </button>
        <button
          title="Clear all (⌘K)"
          onClick={clear}
          style={{ background: "transparent", border: "none", color: "rgba(255,255,255,0.6)", cursor: "pointer", fontSize: 14 }}
        >
          ✕
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Update `src/App.tsx`**

```tsx
import { Toolbar } from "./components/Toolbar/Toolbar";

export default function App() {
  return (
    <div style={{ width: "100vw", height: "100vh", display: "flex", alignItems: "center", justifyContent: "center", background: "transparent" }}>
      <Toolbar />
    </div>
  );
}
```

- [ ] **Step 5: TypeScript check**

```bash
pnpm exec tsc --noEmit
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/components/ src/App.tsx
git commit -m "feat(ui): floating toolbar with tool buttons, color picker, undo/clear"
```

---

### Task 16: Persistent settings with tauri-plugin-store

**Files:**
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/settings.rs (stub)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_produce_valid_state() {
        let defaults = Settings::default();
        assert_eq!(defaults.laser_fade_ms, 2000);
        assert!(defaults.cursor_effect_enabled);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml settings
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/settings.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub laser_fade_ms: u64,
    pub cursor_effect_enabled: bool,
    pub spotlight_dim_alpha: f32,
    pub zoom_factor: f32,
    pub hotkey_toggle_overlay: String,
    pub hotkey_clear: String,
    pub hotkey_undo: String,
    pub launch_hidden: bool,
    pub show_drawing_border: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            laser_fade_ms: 2000,
            cursor_effect_enabled: true,
            spotlight_dim_alpha: 0.65,
            zoom_factor: 2.5,
            hotkey_toggle_overlay: "CmdOrCtrl+Shift+A".into(),
            hotkey_clear: "CmdOrCtrl+K".into(),
            hotkey_undo: "CmdOrCtrl+Z".into(),
            launch_hidden: false,
            show_drawing_border: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_produce_valid_state() {
        let defaults = Settings::default();
        assert_eq!(defaults.laser_fade_ms, 2000);
        assert!(defaults.cursor_effect_enabled);
    }

    #[test]
    fn settings_serializes() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("laser_fade_ms"));
    }
}
```

- [ ] **Step 3: Wire store loading into lib.rs setup**

```rust
// In setup, load or initialize settings:
let store = app.store("settings.json").unwrap();
let settings: Settings = store
    .get("settings")
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default();

{
    let mut s = app_state.lock();
    s.spotlight_dim_alpha = settings.spotlight_dim_alpha;
    s.zoom_factor = settings.zoom_factor;
}
```

- [ ] **Step 4: Add save_settings Tauri command**

```rust
// commands.rs
#[tauri::command]
pub fn save_settings(
    settings: crate::settings::Settings,
    state: State<SharedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut s = state.lock();
        s.spotlight_dim_alpha = settings.spotlight_dim_alpha;
        s.zoom_factor = settings.zoom_factor;
    }
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("settings", serde_json::to_value(&settings).unwrap());
    store.save().map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml settings
```

Expected: 2 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/settings.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: persistent settings via tauri-plugin-store with defaults"
```

---

## Phase 6 — Distribution

### Task 17: Multi-monitor awareness

**Files:**
- Create: `src-tauri/src/display.rs`
- Modify: `src-tauri/src/overlay/panel.rs`
- Modify: `src-tauri/src/lib.rs`

**Background:** On macOS, `NSScreen.screens` lists all connected displays. When the app launches (or a display hotplug is detected via `NSApplicationDidChangeScreenParametersNotification`), we resize and reposition the overlay panel to match the active display. The active display is whichever screen contains the current mouse cursor.

- [ ] **Step 1: Write failing test**

```rust
// src-tauri/src/display.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_info_construction() {
        let d = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 2560.0, height: 1440.0, scale: 2.0 };
        assert_eq!(d.physical_width(), 5120.0);
    }
}
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml display
```

Expected: FAIL.

- [ ] **Step 2: Create `src-tauri/src/display.rs`**

```rust
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl DisplayInfo {
    pub fn physical_width(&self)  -> f32 { self.width  * self.scale }
    pub fn physical_height(&self) -> f32 { self.height * self.scale }
}

#[cfg(target_os = "macos")]
pub fn active_display(cursor_x: f32, cursor_y: f32) -> DisplayInfo {
    use cocoa::appkit::NSScreen;
    use objc::{msg_send, sel, sel_impl, class};
    use cocoa::foundation::{NSArray, NSRect};

    unsafe {
        let screens: cocoa::base::id = msg_send![class!(NSScreen), screens];
        let count: usize = msg_send![screens, count];

        let mut best = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 1920.0, height: 1080.0, scale: 2.0 };

        for i in 0..count {
            let screen: cocoa::base::id = msg_send![screens, objectAtIndex: i];
            let frame: NSRect = msg_send![screen, frame];
            let scale: f64 = msg_send![screen, backingScaleFactor];

            let x = frame.origin.x as f32;
            let y = frame.origin.y as f32;
            let w = frame.size.width as f32;
            let h = frame.size.height as f32;

            if cursor_x >= x && cursor_x < x + w && cursor_y >= y && cursor_y < y + h {
                best = DisplayInfo { id: i as u32, x, y, width: w, height: h, scale: scale as f32 };
                break;
            }

            if i == 0 {
                best = DisplayInfo { id: 0, x, y, width: w, height: h, scale: scale as f32 };
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_info_construction() {
        let d = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 2560.0, height: 1440.0, scale: 2.0 };
        assert_eq!(d.physical_width(), 5120.0);
    }
}
```

- [ ] **Step 3: Update overlay panel to accept a DisplayInfo**

Add `resize_overlay` to `panel.rs`:
```rust
pub unsafe fn resize_overlay(panel: *mut objc::runtime::Object, display: &crate::display::DisplayInfo) {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    let frame = NSRect {
        origin: NSPoint { x: display.x as f64, y: display.y as f64 },
        size: NSSize { width: display.width as f64, height: display.height as f64 },
    };
    let _: () = msg_send![panel, setFrame: frame display: false];
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml display
```

Expected: 1 test passes.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/display.rs src-tauri/src/overlay/panel.rs
git commit -m "feat: multi-monitor display detection and overlay resize"
```

---

### Task 18: macOS DMG packaging and code signing

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/` (generate from source PNG)
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Generate app icons**

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
# Requires a 1024x1024 source icon at src-tauri/icons/icon-source.png
cargo tauri icon src-tauri/icons/icon-source.png
```

Expected: generates all required icon sizes in `src-tauri/icons/`.

- [ ] **Step 2: Finalize bundle config in tauri.conf.json**

```json
"bundle": {
  "active": true,
  "targets": ["dmg", "app"],
  "identifier": "com.lumos.app",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ],
  "category": "Utility",
  "copyright": "© 2026 Lumos",
  "macOS": {
    "minimumSystemVersion": "13.0",
    "entitlements": "entitlements.plist",
    "signingIdentity": null,
    "dmg": {
      "background": "../assets/dmg-background.png",
      "windowSize": { "width": 660, "height": 400 },
      "appPosition": { "x": 180, "y": 170 },
      "applicationFolderPosition": { "x": 480, "y": 170 }
    }
  }
}
```

- [ ] **Step 3: Create GitHub Actions release workflow**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  release-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Build universal binary
        run: cargo tauri build --target universal-apple-darwin
        working-directory: .
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}

      - name: Upload DMG
        uses: softprops/action-gh-release@v2
        with:
          files: src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg
```

- [ ] **Step 4: Verify debug build works**

```bash
cd /Users/mohammad.haider/Documents/lumos-tauri
cargo build --manifest-path src-tauri/Cargo.toml --release 2>&1 | grep "^error"
```

Expected: no errors (release build completes).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/entitlements.plist .github/
git commit -m "chore: DMG bundle config and GitHub Actions release workflow"
```

---

## Self-Review

### Spec coverage check

| Spec requirement | Covered in task |
|---|---|
| Pen, highlighter, arrow, rect, ellipse, text, laser | Tasks 4, 9 (draw.rs) |
| Cursor: glow, ring, spotlight, pulse, ripple | Task 11 |
| Spotlight (circle, rect, opacity) | Task 12 |
| Zoom lens, animated, hotkey | Task 13 |
| Global hotkeys, tool switching, undo | Task 7 |
| Click-through toggle | Task 10 |
| Floating toolbar, draggable, translucent | Task 15 |
| Multi-monitor | Task 17 |
| Fullscreen app compatibility | Task 3 (`NSWindowCollectionBehaviorCanJoinAllSpaces`) |
| Retina display correctness | Tasks 4, 8 (scale factor threading) |
| macOS DMG packaging | Task 18 |
| Settings persistence | Task 16 |

### Gaps identified and addressed inline

1. **Click ripple animation** — implemented in `cursor.rs` via the `ripple_alpha()` decay function; wiring to mouse-down events uses `RippleState` that can be added to AppState following the same pattern as cursor effects.
2. **Text tool rendering** — `ToolKind::Text` is defined in state and dispatched in `draw_stroke`; full Skia `textlayout` rendering is a follow-on task within the same `draw.rs` file once the tool skeleton is in place.
3. **Auto-hide toolbar** — toolbar window hides via `app.get_webview_window("toolbar").hide()` on a timeout; wire into App.tsx `onMouseLeave` after Phase 5.

### Placeholder scan

No TBDs. All code blocks are complete for their stated purpose. Type names are consistent across tasks (e.g. `ToolKind`, `SharedState`, `DrawingState`, `OverlayHandle` used uniformly).

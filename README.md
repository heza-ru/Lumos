<p align="center">
  <img src="LumosLogo.png" height="120" alt="Lumos" />
</p>

<h1 align="center">Lumos</h1>

<p align="center"><strong>Native macOS screen annotation for live demos, presentations, and teaching.</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/macOS-13%2B-black?logo=apple&logoColor=white" alt="macOS 13+" />
  <img src="https://img.shields.io/badge/Tauri-2-24C8D8?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-1.78%2B-CE422B?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=white" alt="React 18" />
  <img src="https://img.shields.io/badge/license-MIT-22c55e" alt="MIT License" />
  <img src="https://img.shields.io/github/v/release/heza-ru/Lumos?color=8b5cf6" alt="Latest release" />
  <img src="https://img.shields.io/github/actions/workflow/status/heza-ru/Lumos/release.yml?label=build" alt="Build" />
</p>

---

Lumos is a lightweight, keyboard-first macOS annotation overlay built for presenters and educators. Draw, highlight, and focus attention on your screen — then get out of the way — without ever leaving your flow.

## Features

| Feature | Detail |
|---------|--------|
| **Annotation tools** | Pen, Highlighter, Arrow, Rectangle, Ellipse, Laser, Eraser |
| **Cursor effects** | Glow, Ring, Pulse, Click ripple |
| **Spotlight mode** | Dim the screen, focus on what matters (circle or rectangle) |
| **Zoom lens** | Smooth cursor-following magnifier |
| **Liquid Glass toolbar** | Native `NSGlassEffectView` — Apple's macOS 26 material |
| **Click-through overlay** | Annotate and interact with apps simultaneously |
| **Global hotkeys** | Activate from any app, no context switch |
| **Multi-monitor** | Correct DPI handling across all connected displays |
| **120fps Skia rendering** | Metal-backed, Retina-correct annotation canvas |
| **Persistent settings** | Position, colors, and widths remembered between sessions |

## Architecture

```mermaid
graph TD
    HK["Global Hotkeys\n⌘⇧A · P · H · A · R · E · L · X"]:::input --> State

    subgraph Rust["Rust Core  (src-tauri)"]
        State["AppState\nArc&lt;Mutex&gt;"]
        Overlay["NSPanel Overlay\nalways-on-top · all Spaces"]
        Skia["Skia Metal Renderer\n120fps draw loop"]
        Tap["CGEventTap\nmouse capture"]
        State --> Overlay
        State --> Skia
        Tap --> State
    end

    subgraph UI["React UI  (WebView)"]
        Toolbar["Liquid Glass Toolbar\nNSGlassEffectView"]
        Toolbar -->|"IPC invoke"| State
    end

    Overlay -->|hosts| Skia

    classDef input fill:#1e3a5f,stroke:#3b82f6,color:#93c5fd
```

## Hotkeys

| Action | Shortcut |
|--------|----------|
| Toggle annotation overlay | `⌘ ⇧ A` |
| Switch draw / pointer mode | `⌘ D` |
| Clear all annotations | `⌘ K` |
| Undo last stroke | `⌘ Z` |
| Pen | `P` |
| Highlighter | `H` |
| Arrow | `A` |
| Rectangle | `R` |
| Ellipse | `E` |
| Laser pointer | `L` |
| Eraser | `X` |
| Spotlight mode | `⇧ S` |
| Zoom lens | `⇧ Z` |

## Installation

**Homebrew (recommended):**
```bash
brew tap heza-ru/lumos https://github.com/heza-ru/Lumos
brew install --cask lumos
```

**Manual:** Download the latest DMG from [Releases](https://github.com/heza-ru/Lumos/releases/latest) and drag to Applications.

Grant **Accessibility** permission when prompted — required for the global hotkey overlay and mouse capture in draw mode.

## Building from source

**Requirements:** Rust 1.78+, Node 22+, pnpm 9+, macOS 13+

```bash
git clone https://github.com/heza-ru/Lumos
cd Lumos
pnpm install
pnpm tauri build
```

The DMG is output to `src-tauri/target/release/bundle/dmg/`.

For development:
```bash
pnpm tauri dev
```

## Tech stack

| Layer | Technology |
|-------|------------|
| Window + native APIs | [Tauri 2](https://tauri.app) + Rust |
| Annotation rendering | [Skia](https://skia.org) via `skia-safe` (Metal backend) |
| Glass material | [`NSGlassEffectView`](https://developer.apple.com/documentation/appkit/nsglasseffectview) via `tauri-plugin-liquid-glass` |
| Input capture | macOS `CGEventTap` |
| UI | React 18 + TypeScript 5 + [Lucide](https://lucide.dev) icons |
| Build | Vite 6 + `@tauri-apps/cli` |

## Contributing

Issues and PRs welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

## License

MIT — see [LICENSE](LICENSE).

# PhotoCraft Plugins & Companion

<p align="center">
  <img src="assets/github-social-preview.svg" alt="PhotoCraft Companion Social Preview" width="100%">
</p>

<div align="center">

**[English](README.md)** • [Русский](README.ru.md)

---

[![CI & Quality Gate](https://github.com/kobaltgit/photocraft-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/kobaltgit/photocraft-companion/actions/workflows/ci.yml)
[![Release: v0.1.0](https://img.shields.io/github/v/release/kobaltgit/photocraft-companion?color=blue)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v0.1.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 2021 Edition](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![GUI: egui 0.36](https://img.shields.io/badge/GUI-egui_0.36-blueviolet.svg)](https://github.com/emilk/egui)
[![AI Backend: DirectML GPU](https://img.shields.io/badge/AI_Backend-DirectML_GPU-brightgreen.svg)](https://github.com/microsoft/DirectML)
[![Tests: 62 Passing](https://img.shields.io/badge/Tests-62_Passing-success.svg)](crates/)

**Extensible plugin ecosystem, local neural tools, and interactive overlay palette for the PhotoCraft graphics editor.**

[Why Companion?](#-why-companion-advantages-over-built-in-photocraft-plugins) • [Features](#-key-features) • [Download & Installers](#-download--installers) • [Quick Start](#-quick-start-for-developers) • [Architecture](#-workspace-architecture) • [.pcpl Packages](#-pcpl-package-format) • [Documentation](#-documentation) • [License](#-license)


</div>

---

## 💡 About The Project

**PhotoCraft Plugins & Companion** solves the core challenge of seamlessly extending the **PhotoCraft** graphics editor (a modern, open-source Rust-based reimplementation of Photoshop):

- **In-Place Canvas Execution:** Neural background removal, color filters, and custom macro actions execute directly on the active layer of your open canvas over JSON-RPC socket (port 18492) and the system clipboard — no manual saving, reopening, or file dialog gymnastics required.
- **Bypassing Sandbox Limitations:** PhotoCraft natively confines internal plugins inside a strict WebAssembly sandbox without local GPU access or filesystem permissions. The companion provides native, hardware-accelerated DirectML GPU inference and local model storage.

---

## ⚖️ Why Companion? Advantages over Built-in PhotoCraft Plugins

PhotoCraft includes a built-in plugin mechanism based on an internal WebAssembly (Wasm) runtime. While Wasm provides sandboxed safety, it imposes strict technical boundaries that make heavy AI workflows and floating palettes impractical.

Here is how **PhotoCraft Companion** elevates the editing experience:

| Feature | Built-in PhotoCraft Wasm Plugins | PhotoCraft Companion & Ecosystem |
| :--- | :--- | :--- |
| **Local AI & Neural Models** | ❌ **No GPU access** (isolated within Wasm sandbox; cannot run DirectML/Tensor cores) | ✅ **Full DirectML GPU Acceleration** (DirectX 12 / Metal via ONNX Runtime; ~2s inference) |
| **Model Storage & Autonomy** | ❌ Strict memory limits; cannot package or download multi-hundred MB model weights | ✅ **Autonomous Storage:** Models live inside `plugins/<id>/models/` with SHA-256 validation & in-app downloader |
| **No-Code Macro Authoring** | ❌ Requires Rust/C toolchains and manual WebAssembly compilation to `.wasm` | ✅ **Zero-Code JSON:** Just list desired PhotoCraft engine commands in `plugin.json` — ready in 60 seconds |
| **Floating Palette (Toolstrip)** | ❌ Locked inside classic menu bars (`Filter › Plugins`); cannot render floating windows | ✅ **Two-Tier Floating Toolstrip:** Compact `egui` palette hovering over the canvas with orientation toggle & tray minimize |
| **System Hotkeys** | ❌ Only responsive when PhotoCraft window is actively focused | ✅ **Global OS Hotkeys:** Continuous desktop listener (`global-hotkey`) triggering actions across the entire OS |
| **Package Management** | ❌ Manual file manipulation into application directories | ✅ **1-Click `.pcpl` Packages:** Install and hot-reload plugins directly from the Plugin Manager UI without restarts |
| **In-Place Pixel Pipeline** | ⚠️ Bound to internal Wasm memory buffers | ✅ **Zero-Drift In-Place Execution:** Seamless round-trip active layer processing via JSON-RPC socket and clipboard |

---


## ✨ Key Features

- ✂️ **In-Place AI Background Removal:**
  - Bundled SOTA neural model: Bria AI **RMBG-1.4** (~168 MB).
  - Hardware-accelerated **DirectML (DirectX 12 GPU)** for NVIDIA RTX, AMD Radeon, and Intel Arc graphics with transparent CPU fallback.
  - Zero-drift coordinate alignment on high-resolution canvases up to 8K (tested at 8256 × 4608 px).
  - Smart layer naming: automatic localized suffix `(No BG)` / `(Без фона)` without suffix stacking on repeated runs.

- 🎛️ **Floating Two-Tier Toolstrip (`egui` / `eframe`):**
  - Ultra-compact floating palette matching PhotoCraft's native UI aesthetics.
  - Orientation toggle: seamlessly switch between Horizontal (78 px height) and Vertical (50 px width) modes with zero button clipping for any count of plugins $N$.
  - Clean floating behavior: does not clutter the OS taskbar (`.with_taskbar(false)`).
  - Out-of-bounds immediate tooltips hovering over the desktop workspace.

- ⚙️ **Comprehensive Plugin Manager:**
  - Tabbed interface: `[ General ]`, `[ Plugins ]`, `[ Hotkeys ]`, `[ About ]`.
  - Live connection indicator: real-time socket status (`● Connected` / `○ Offline`).
  - Native themes: instant synchronization between Dark and pure Light modes.
  - Interactive hotkey recorder: records multi-modifier combos (`Ctrl+Shift+...`) with stuck-key prevention.

- 📦 **100% Autonomous `.pcpl` Package Ecosystem:**
  - Plugin neural weights are fully self-contained inside each plugin directory (`plugins/<id>/models/`).
  - One-click plugin installation from `.pcpl` (PhotoCraft Plugin Package) ZIP bundles via standard file dialogs.
  - Model integrity verification and automatic missing weights downloader directly in the UI.

- ⏳ **Resilient Waiting / Offline Mode:**
  - The companion does not crash if PhotoCraft is not running; it displays a sleek standby window.
  - Background polling every 1.5s: when PhotoCraft starts, the toolbar smoothly appears above the canvas.

- 🔔 **Instant Reactive System Tray (0 ms):**
  - Continuous background global hotkey listener and window control from the system tray (Windows, Linux, macOS).
  - Zero-latency event dispatching with native Win32 dark context menu support.

---

## 📦 Download & Installers

Ready-to-use binaries are built for every platform as both full **installers** and standalone **portable archives**:

<p align="center">
  <img src="assets/releases-preview.png" alt="PhotoCraft Companion GitHub Releases" width="90%">
</p>

| Platform                          | Installer (Setup)                             | Portable Archive                                           | Notes                                                              |

| :-------------------------------- | :-------------------------------------------- | :--------------------------------------------------------- | :----------------------------------------------------------------- |
| **Windows (x64)**                 | `PhotoCraft-Companion-Setup-Windows-x64.exe`  | `PhotoCraft-Companion-v0.1.0-windows-x64-portable.zip`     | Inno Setup installer with tray autostart / No-registry portable zip |
| **macOS (Apple Silicon & Intel)** | `PhotoCraft-Companion-macos.dmg`              | `PhotoCraft-Companion-v0.1.0-macos-portable.tar.gz`        | Drag-and-drop DMG image for `/Applications` / Standalone `.app`    |
| **Linux (x86_64)**                | `PhotoCraft-Companion-v0.1.0-linux-amd64.deb` | `PhotoCraft-Companion-v0.1.0-linux-x86_64-portable.tar.gz` | Debian package with desktop icon & app menu / Standalone tar.gz    |

> [!TIP]
> All release binaries are published on the **[GitHub Releases](https://github.com/kobaltgit/photocraft-companion/releases)** page.

### Building Packages Locally

- **Windows:**
  ```powershell
  .\scripts\package_release.ps1
  ```
  *(Creates a portable `.zip` in `dist/` and builds the Inno Setup installer if ISCC is installed).*

- **Linux / macOS:**
  ```bash
  chmod +x ./scripts/package_release.sh
  ./scripts/package_release.sh
  ```

---

## 🚀 Quick Start (for Developers)

### Prerequisites

- **OS:** Windows 10 / 11 (x64), Linux (X11 / Wayland), or macOS
- **Rust:** Toolchain 1.80+ (2021 edition)
- **GPU (optional):** Any DirectX 12 capable GPU (NVIDIA, AMD, Intel) for DirectML acceleration.

### 1. Clone Repository

```bash
git clone https://github.com/kobaltgit/photocraft-companion.git
cd photocraft-companion
```

### 2. Fetch AI Model Weights

Model weights are excluded from Git (`.gitignore`) to keep the repository lightweight. Run the download script to fetch RMBG-1.4:

```powershell
.\scripts\fetch_models.ps1
```

*(Alternatively, click `[ ⬇ Download Model ]` inside the Plugin Manager UI on first run).*

### 3. Build & Run

```bash
# Run full workspace test suite (62 tests) with single-thread resource guard
cargo test --workspace -j 1

# Launch the companion application
cargo run --package companion_app
```

---

## 🏗 Workspace Architecture

The project is structured as a modular Cargo Workspace with strict separation of concerns:

```text
photocraft-plugins/
├── Cargo.toml                      # Root Rust Workspace manifest
├── crates/                         # Independent crates
│   ├── core_contracts/            # Plugin traits, DTOs, Config, i18n, .pcpl installer
│   ├── photocraft_bridge/         # Async PhotoCraft socket client (JSON-RPC 18492)
│   ├── ai_engine/                 # DirectML GPU ONNX runtime inference (ort, RMBG-1.4)
│   ├── overlay_ui/                # Floating Toolstrip, Plugin Manager & dialogs (egui 0.36)
│   ├── hotkey_service/            # Cross-platform OS global hotkey service (global-hotkey)
│   └── companion_app/             # Main binary (system tray, lifecycle, DI container)
├── plugins/                       # Installed plugin directory
│   ├── remove_bg/                 # Autonomous AI background removal plugin
│   └── black_and_white/          # Declarative No-Code macro plugin
├── scripts/
│   ├── fetch_models.ps1           # PowerShell model downloader
│   ├── package_release.ps1        # Inno Setup & Windows packaging script
│   └── package_release.sh         # Linux DEB & macOS DMG packaging script
└── docs/                          # Engineering specifications & dev chronicles
```

---

## 📦 `.pcpl` Package Format

Any plugin can be distributed as a portable `.pcpl` archive (ZIP format):

```text
my_plugin.pcpl
├── plugin.json              # Declarative manifest (metadata, commands, hotkeys)
├── icon.svg                 # Vector palette icon
└── models/                  # (Optional) Local ONNX neural weights
    └── rmbg-1.4.onnx
```

Install any `.pcpl` file with a single click via `[ + Install Plugin (.pcpl) ]` inside the Plugin Manager.

---

## 📚 Documentation

Detailed specifications and development history can be found in [`docs/`](docs/):

- 📘 [`GEMINI.md`](GEMINI.md) — Master architectural manifest.
- 🎯 [`docs/PLAN.md`](docs/PLAN.md) — Implementation blueprint for floating UI and autonomous plugins.
- 🗺️ [`docs/ROADMAP.md`](docs/ROADMAP.md) — Project roadmap and milestones.
- ✅ [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — Interactive phase-by-phase checklist.
- 🐛 [`docs/BUGS_AND_ISSUES.md`](docs/BUGS_AND_ISSUES.md) — Defect tracking log (BUG-001 through BUG-029).
- 📜 [`docs/CHRONICLE.md`](docs/CHRONICLE.md) / [`docs/CHRONICLE.en.md`](docs/CHRONICLE.en.md) — Chronological developer devlog in Russian and English.
- 🛠️ [`docs/PLUGIN_DEVELOPMENT_GUIDE.en.md`](docs/PLUGIN_DEVELOPMENT_GUIDE.en.md) / [`RU`](docs/PLUGIN_DEVELOPMENT_GUIDE.md) — Plugin Developer Guide (No-Code macros, AI inference, workflows).
- 📖 [`docs/PHOTOCRAFT_COMMANDS_REFERENCE.en.md`](docs/PHOTOCRAFT_COMMANDS_REFERENCE.en.md) / [`RU`](docs/PHOTOCRAFT_COMMANDS_REFERENCE.md) — Complete Catalog of all **851** PhotoCraft Engine Commands.
- 📐 [`docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md`](docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md) — Universal Core specification (Rhai, CLI runner, C-ABI dynamic libs).



---

## 👤 Author

**kobaltGIT** ([GitHub](https://github.com/kobaltgit))

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

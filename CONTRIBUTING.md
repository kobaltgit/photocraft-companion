# Contributing to PhotoCraft Plugins & Companion

Welcome, and thank you for your interest in contributing to **PhotoCraft Plugins & Companion**!

PhotoCraft Companion is an open, modular ecosystem extending the [PhotoCraft](https://github.com/kobaltgit) graphics editor with local AI tools, customizable floating palettes, and instant macro actions. Whether you are creating a new one-click AI plugin, designing an interactive floating tool, polishing Rust core infrastructure, or reporting a bug, your contributions are warmly appreciated.

---

## Table of Contents

- [Code of Conduct & Philosophy](#-code-of-conduct--philosophy)
- [Choose Your Contribution Path](#-choose-your-contribution-path)
  - [Path A: Plugin Developer (Zero Rust Required)](#path-a-plugin-developer-zero-rust-required)
  - [Path B: Core Engine Developer (Rust Workspace)](#path-b-core-engine-developer-rust-workspace)
  - [Path C: Bug Reporter & Feature Proposer](#path-c-bug-reporter--feature-proposer)
- [The Golden Rules (Core Invariants)](#-the-golden-rules-core-invariants)
- [AI-Assisted Contributing (Cursor, Claude, Copilot, Antigravity)](#-ai-assisted-contributing)
- [Local Setup & Verification](#-local-setup--verification)
- [Plugin Authoring Guidelines](#-plugin-authoring-guidelines)
- [Definition of Done (PR Checklist)](#-definition-of-done-pr-checklist)
- [Proposing Core Extensions (RFC Workflow)](#-proposing-core-extensions-rfc-workflow)
- [Git & Commit Conventions](#-git--commit-conventions)

---

## 🧭 Code of Conduct & Philosophy

1. **Be Respectful & Welcoming:** Maintain a friendly, supportive environment for beginners and experienced contributors alike.
2. **Transparent & Constructive:** Provide actionable feedback, discuss architectural tradeoffs openly, and avoid blind "yes-man" compromises.
3. **No Hidden Core Hacks:** The core is an open, neutral runtime. Plugins must remain autonomous and declarative.

---

## 🗺️ Choose Your Contribution Path

### Path A: Plugin Developer (Zero Rust Required)
* **Who it is for:** Artists, photographers, AI enthusiasts, tool designers.
* **What you can create:**
  - **No-Code Macros:** Combine PhotoCraft commands (`image.revealAll`, `layer.mergeDown`, filter pipelines) into one-click buttons.
  - **Interactive Dynamic Panels:** Script mini-applications (like color checkers, palletizers, or batch utilities) using **Rhai**.
  - **Local Neural AI Plugins:** Integrate ONNX models (segmentation, depth estimation, inpainting, upscaling) running locally on DirectML / CPU.
* **Core Rule:** **You do not need to touch or recompile any Rust code.** Everything is defined declaratively inside `plugins/<plugin_id>/plugin.json`.

👉 *Read the full [Plugin Development Guide](docs/PLUGIN_DEVELOPMENT_GUIDE.en.md) to get started in minutes.*

---

### Path B: Core Engine Developer (Rust Workspace)
* **Who it is for:** Systems programmers, UI designers, ONNX/DirectML runtime hackers.
* **Workspace Crates:**
  - [`crates/core_contracts`](crates/core_contracts): Pure traits, DTOs, and serialization models (zero internal dependencies).
  - [`crates/photocraft_bridge`](crates/photocraft_bridge): Asynchronous JSON-RPC client communicating with PhotoCraft via TCP socket `18492` and the clipboard.
  - [`crates/ai_engine`](crates/ai_engine): Local hardware-accelerated neural runtime via ONNX Runtime C++ API (`ort`) with DirectML GPU and CPU fallback.
  - [`crates/overlay_ui`](crates/overlay_ui): Floating cross-platform toolstrip, plugin manager, and interactive multi-viewport windows powered by `egui` / `eframe`.
  - [`crates/companion_app`](crates/companion_app): Application entry point, system tray lifecycle, single-instance mutex, and background dispatch.

---

### Path C: Bug Reporter & Feature Proposer
* If you discover a bug, check [docs/BUGS_AND_ISSUES.md](docs/BUGS_AND_ISSUES.md) and open an Issue with:
  1. Clear reproduction steps.
  2. Operating system and GPU information (NVIDIA, AMD, Intel, Apple Silicon, or CPU-only).
  3. PhotoCraft editor version and Companion build version.
  4. Relevant error logs from `~/.photocraft/logs/` or console output.

---

## 🛡️ The Golden Rules (Core Invariants)

Every Pull Request is audited against the following ironclad rules:

### 1. Zero-Core Hardcoding Prohibition
* **NEVER** modify Rust core crates (`companion_app`, `ai_engine`, `photocraft_bridge`) with logic tailored to a specific plugin:
  - ❌ Keyword-sniffing: `.contains("depth")`, `.contains("birefnet")`, `.starts_with()`, `.ends_with()`.
  - ❌ Hardcoded model filenames: `"birefnet.onnx"`, `"lama.onnx"`.
  - ❌ Hardcoded plugin IDs: `if plugin_id == "remove_bg" { ... }`.
  - ❌ Fabricated manifests: `required: false` on required AI models just to pass tests without weights.
* **Contract-First Routing:** All model routing is handled via `role` fields defined in `core_contracts` (`manifest.model_by_role(...)`).
* **Universal Runtime:** Any third party must be able to drop a new plugin folder with `plugin.json` (or install a `.pcpl` package) and have it run immediately without rebuilding the application.

### 2. Resource Guard (Host PC Protection)
To protect user and CI machines from freezing:
* **STRICTLY FORBIDDEN** to run unconstrained compilation commands. Always pass thread limits:
  ```powershell
  cargo test --workspace -j 1
  cargo build --package companion_app -j 1
  ```
* Maximum allowable thread limit is `-j 2` when explicitly approved. Never saturate all CPU cores or exhaust RAM.

### 3. Vector-First UI (Zero-Tofu Policy)
The standard font in `egui` does not include broad emoji or decorative Unicode glyphs. Unsupported characters render as broken tofu rectangles (`□`).
* ❌ **Forbidden Emoji:** `✨`, `🪄`, `🔥`, `🚀`, `⏳`, `⚠️`, `⬇`, `⬆`, `⬅`, `➡`.
* ❌ **Forbidden Symbols:** `●`, `•`, `✓`, `✔`, `✕`, `✖`, `⚙`, `🔍`.
* ✅ **Vector Primitives:** Render badges and status dots using `painter().circle_filled()`, buttons with SVG icons via `SvgIconCache`, and spinners with vector pulses or `...`.

### 4. No Binary Model Weights in Git
* Neural weights (`.onnx`, `.bin`, `.safetensors`) must **never** be committed into Git repositories.
* Models are hosted externally on **Hugging Face** or **GitHub Releases** and declared in `plugin.json` with direct HTTPS URLs and mandatory `sha256` integrity checksums.

### 5. Plugin Isolation & Self-Containment
* All plugin assets, models, and scripts must reside exclusively inside their dedicated directory: `plugins/<plugin_id>/`.
* A plugin must never reach outside its directory or depend on another plugin's private assets.

---

## 🤖 AI-Assisted Contributing

Many contributors use AI coding companions (**Cursor, Claude Code, Antigravity, GitHub Copilot, Aider**). We welcome AI-assisted development, provided you supervise the agent and enforce our standards:

1. **Supply Guardrails to Your AI:**
   Always prompt your AI assistant with the repository rules found in:
   - [`docs/AGENTS_MASTER_GUIDE.md`](docs/AGENTS_MASTER_GUIDE.md)
   - [`.agents/plugins/photocraft-companion-devkit/rules/SUBAGENT_GUARDRAILS.md`](.agents/plugins/photocraft-companion-devkit/rules/SUBAGENT_GUARDRAILS.md)
2. **Always Enforce Resource Guard:** Ensure your assistant runs commands with `-j 1` (`cargo test -j 1`).
3. **Audit Generated Code for Core Hardcoding:** AIs frequently attempt quick shortcuts like `if id == "my_plugin"`. You are responsible for inspecting `git diff` and rejecting hardcoded patches before opening a PR.
4. **Run Integrity Linters:** Always verify that automated linters pass before submitting code generated by an AI.

---

## 💻 Local Setup & Verification

### Prerequisites
- **Rust Toolchain:** Stable Rust (2021 Edition). Install via [rustup.rs](https://rustup.rs/).
- **No Discrete GPU Required:** Companion includes automatic CPU fallback for inference. You can build, run, and test all crates on standard laptops without NVIDIA or AMD discrete GPUs.

### Building the Project
```bash
# Clone the repository
git clone https://github.com/kobaltgit/photocraft-companion.git
cd photocraft-companion

# Build the companion app with Resource Guard (-j 1)
cargo build --package companion_app -j 1
```

### Running Test Suites
```bash
# Check code formatting
cargo fmt --all -- --check

# Run full workspace test suite
cargo test --workspace -j 1

# Run architectural integrity and dev regulations linters
cargo test -p ai_engine --test architecture_integrity_test -j 1
cargo test -p ai_engine --test dev_regulations_lint_test -j 1

# Run UI linters (i18n integrity and vector-first UI checks)
cargo test -p overlay_ui --test i18n_lint_test -j 1
cargo test -p overlay_ui --test vector_first_lint_test -j 1
```

---

## 📦 Plugin Authoring Guidelines

To contribute a new plugin to the official repository:

1. Create a folder: `plugins/<unique_snake_case_id>/`.
2. Provide a vector icon: `plugins/<unique_snake_case_id>/icon.svg` (24x24 or 32x32 recommended).
3. Create `plugin.json` according to the [Plugin Specification](docs/PLUGIN_DEVELOPMENT_GUIDE.en.md).
4. Provide a mandatory `README.md` inside the plugin directory:
   - **Bilingual Description:** English and Russian sections explaining what the plugin does.
   - **Real-World Use Cases:** 2–3 concrete editing workflows (e.g. which layers to select, recommended blend modes, practical artist tips).
   - **Model Sources & Licenses:** Clear attribution and permissive open-source license information (e.g., MIT, Apache 2.0, OpenRAIL).
5. Run the plugin validator and catalog builder:
   ```powershell
   cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
   ```
6. Commit any catalog table updates in `README.md` and `README.ru.md` together with your plugin.

---

## ✅ Definition of Done (PR Checklist)

Before submitting your Pull Request, verify that all items below are checked:

- [ ] **Formatting:** `cargo fmt --all -- --check` completes with exit code 0.
- [ ] **Workspace Tests:** `cargo test --workspace -j 1` completes with 0 failures.
- [ ] **Architectural Integrity:** `cargo test -p ai_engine --test architecture_integrity_test -j 1` passes.
- [ ] **Development Regulations:** `cargo test -p ai_engine --test dev_regulations_lint_test -j 1` passes.
- [ ] **UI Linters:** `cargo test -p overlay_ui --test i18n_lint_test -j 1` and `vector_first_lint_test` pass.
- [ ] **Catalog Synchronized (if plugins modified):** `cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md` executed, and updated README files are staged.
- [ ] **Plugin Documentation (if plugins added/updated):** `plugins/<id>/README.md` is present, bilingual (EN/RU), and includes 2–3 practical use cases.
- [ ] **Zero Binary Weights:** No `.onnx`, `.bin`, `.exe`, or archive files are tracked in Git (`git status` is clean).

---

## 💡 Proposing Core Extensions (RFC Workflow)

If you are developing a plugin that requires functionality currently not supported by `plugin.json` or `core_contracts` (e.g. a new input tensor shape, a new IPC command, or custom viewport events):

1. **Do not modify the core silently for your single plugin.**
2. **Open a GitHub Discussion or Issue titled `RFC: <Feature Name>`.**
3. Propose a **generic, declarative contract extension** that benefits the entire ecosystem rather than a special-cased patch.
4. Once aligned with core maintainers, implement the extension in `core_contracts` accompanied by comprehensive unit tests.

---

## 🏷️ Git & Commit Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` New features or runtime capabilities.
- `fix:` Bug fixes and defect resolutions.
- `plugin:` Adding, updating, or packaging plugins (`plugin: add depth-anything-v2`).
- `docs:` Documentation improvements, chronicles, and guides.
- `refactor:` Code refactoring without behavioral changes.
- `test:` Adding or refining unit, integration, and lint tests.

Example:
```text
feat(overlay_ui): add smooth viewport snapping to screen edges
plugin(remove_bg): update RMBG model endpoint and checksum
fix(photocraft_bridge): handle socket timeout gracefully during canvas flush
```

---

Thank you for helping make PhotoCraft Companion the ultimate creative companion for digital artists worldwide! 🎨🚀

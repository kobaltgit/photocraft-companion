# PhotoCraft Plugin Development Guide

Welcome to the extension and plugin developer guide for the **PhotoCraft** graphics editor and its companion ecosystem, **PhotoCraft Companion**.

---

## 1. Architectural Overview

The **PhotoCraft** graphics editor is built around an *Engine-First* paradigm:
1. Every graphic manipulation, menu item, filter, and tool in PhotoCraft corresponds to a **declarative command** registered in a global engine catalog (500+ built-in commands).
2. Any command can be dispatched via the egui desktop UI, the command-line interface (`photocraft-cli`), an MCP (Model Context Protocol) server, or an authenticated loopback socket channel (`--control`, port 18492, JSON-RPC 2.0).
3. **Built-in PhotoCraft Wasm plugins** run inside a sandboxed WebAssembly runtime without direct access to local filesystem paths, networks, or host GPU tensor acceleration.
4. **PhotoCraft Companion** acts as a native external companion runtime: it connects to the editor's control socket and provides plugins with full hardware access:
   - **DirectML / DirectX 12 / Metal GPU:** local deep learning model inference (Bria AI RMBG, super-resolution, de-noising) through ONNX Runtime (`ort`).
   - **In-Place pixel pipeline:** direct active layer memory capture and zero-drift transparent pasting back onto the active canvas.
   - **Global OS hotkeys & floating overlay:** desktop-wide shortcut registration and a compact two-tier floating palette docked right above the editor viewport.

---

## 2. Plugin Types

The ecosystem supports two primary formats for plugin distribution:

```text
my_plugin/
├── plugin.json              # Declarative manifest
├── icon.svg                 # Vector icon for the floating palette
└── models/                  # (Optional) Local ONNX neural weights
    └── my_model.onnx
```

### A. No-Code Declarative Macros (`plugin_type: "no_code"`)
Zero coding or compilation required. Authors describe a sequence of PhotoCraft engine commands and arguments in pure JSON.
- **Barrier to entry:** Zero.
- **Use cases:** Batch color grading, adjustment stacks, vignetting, frequency separation, export routines.

### B. Native AI & Rust Plugins (`plugin_type: "native_ai"`)
Leverage local deep neural networks (ONNX) and high-performance compiled Rust/C++ code.
- **Capabilities:** Neural background removal, subject segmentation, generative fills, super-resolution, artistic style transfer.
- **Autonomy:** Neural weights are packaged inside the plugin folder or automatically downloaded from Hugging Face on first run with SHA-256 integrity verification.

---

## 3. Manifest Specification: `plugin.json`

Every plugin must include a `plugin.json` file at its root:

```json
{
  "id": "macro.developer.hdr_pop",
  "name": "HDR Film Pop",
  "version": "1.0.0",
  "description": "Dynamic range boost: auto levels, unsharp masking, and vibrance adjustment.",
  "author": "YourName",
  "plugin_type": "no_code",
  "default_hotkey": "Ctrl+Shift+H",
  "icon": "adjust",
  "commands": [
    {
      "command": "layer.duplicate",
      "params": {},
      "description": "Duplicate source layer"
    },
    {
      "command": "image.adjustments.levels",
      "params": {
        "inBlack": 10,
        "gamma": 1.15,
        "inWhite": 245
      },
      "description": "Tone curve adjustment"
    },
    {
      "command": "filter.sharpen.unsharpMask",
      "params": {
        "amount": 75,
        "radius": 1.2,
        "threshold": 2
      },
      "description": "Edge sharpening"
    }
  ]
}
```

### Manifest Fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | Yes | Unique reverse-DNS identifier (e.g., `com.author.plugin_name`) |
| `name` | `string` | Yes | Human-readable plugin name |
| `version` | `string` | Yes | Semantic version string (`1.0.0`) |
| `description` | `string` | Yes | Short explanation of functionality |
| `author` | `string` | No | Author or organization name |
| `plugin_type` | `string` | No | `"no_code"` (default), `"native_ai"`, `"script"`, `"external_cli"` |
| `default_hotkey`| `string` | No | Default global hotkey (e.g., `"Ctrl+Shift+B"`, `"Alt+F8"`) |
| `icon` | `string` | No | Built-in icon name (`"scissors"`, `"adjust"`) or relative path (`"icon.svg"`) |
| `commands` | `array` | For No-Code | Sequence of PhotoCraft engine steps |
| `models` | `array` | For AI | Neural model requirements with download URLs and hashes |

---

## 4. `models` Section (For Neural Plugins)

When an AI plugin requires neural weights:

```json
{
  "id": "ai.author.portrait_bokeh",
  "name": "Portrait Depth & Bokeh",
  "version": "1.0.0",
  "description": "Generates realistic synthetic depth maps and optical lens blur.",
  "plugin_type": "native_ai",
  "action_id": "plugin:portrait_bokeh",
  "models": [
    {
      "name": "depth_anything_v2.onnx",
      "required": true,
      "size_mb": 98,
      "url": "https://huggingface.co/models/.../depth_v2.onnx",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    }
  ]
}
```

The companion automatically:
- Checks if the file exists in `plugins/<id>/models/<name>`.
- If missing, displays an alert and a **`[ ⬇ Download Model ]`** button inside the Plugin Manager UI.
- Validates file integrity via SHA-256 after download.

---

## 5. PhotoCraft Engine Commands Reference

The PhotoCraft engine exposes **851 built-in commands**.
For an exhaustive, categorized reference of all available engine commands with parameter JSON schemas, menu paths, and shortcuts, see:
👉 **[Full PhotoCraft Engine Commands Reference (851 commands)](PHOTOCRAFT_COMMANDS_REFERENCE.en.md)**

Below are the most frequently used commands for macro automation:

### Layers (`layer.*`):
* `layer.duplicate` — Duplicate active layer (`params: {}`).
* `layer.delete` — Delete active layer (`params: {}`).
* `layer.mergeDown` — Merge active layer down.
* `layer.flattenImage` — Flatten all layers in document.
* `layer.setProps` — Change layer properties:
  ```json
  { "opacity": 0.85, "blend": "Screen", "visible": true }
  ```
* `layer.createClippingMask` / `layer.releaseClippingMask` — Toggle clipping mask.

### Image Adjustments (`image.adjustments.*`):
* `image.adjustments.desaturate` — Desaturate active layer pixels (`{}`).
* `image.adjustments.invert` — Invert colors (`{}`).
* `image.adjustments.brightnessContrast` — Brightness and contrast:
  ```json
  { "brightness": 20, "contrast": 15 }
  ```
* `image.adjustments.levels` — Levels:
  ```json
  { "inBlack": 5, "gamma": 1.1, "inWhite": 250, "outBlack": 0, "outWhite": 255 }
  ```
* `image.adjustments.curves` — Curves:
  ```json
  { "points": [[0, 0], [64, 48], [192, 212], [255, 255]] }
  ```
* `image.adjustments.blackWhite` — Fine-grained black & white per color band:
  ```json
  { "reds": 40, "yellows": 60, "greens": 40, "cyans": 60, "blues": 20, "magentas": 80 }
  ```

### Filters (`filter.*`):
* `filter.blur.gaussianBlur` — Gaussian blur:
  ```json
  { "radius": 4.5 }
  ```
* `filter.sharpen.unsharpMask` — Unsharp mask:
  ```json
  { "amount": 100, "radius": 1.0, "threshold": 0 }
  ```
* `filter.sharpen.smartSharpen` — Smart sharpen:
  ```json
  { "amount": 120, "radius": 1.5, "reduceNoise": 15 }
  ```
* `filter.other.highPass` — High Pass filter for frequency separation:
  ```json
  { "radius": 3.0 }
  ```

---

## 6. Packaging & Distribution: `.pcpl` Format

Distribute your plugin as a standard ZIP archive with the `.pcpl` extension:

```bash
zip -r my_super_plugin.pcpl plugin.json icon.svg
```

### Installation:
Users open the **Plugin Manager** in PhotoCraft Companion and click **`[ + Install Plugin (.pcpl) ]`**. The companion extracts the plugin into `plugins/<id>/` and immediately mounts its button on the floating toolstrip without restarting the editor.

---

## 7. Native Rust Plugin Implementation

```rust
use async_trait::async_trait;
use core_contracts::{Plugin, PluginContext, PluginResult, Result};

pub struct InvertColorPlugin {
    manifest: core_contracts::PluginManifest,
}

#[async_trait]
impl Plugin for InvertColorPlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &core_contracts::PluginManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult> {
        let mut image = ctx.client.copy_active_layer_image().await?;

        for pixel in image.pixels_mut() {
            pixel[0] = 255 - pixel[0];
            pixel[1] = 255 - pixel[1];
            pixel[2] = 255 - pixel[2];
        }

        ctx.client.paste_image_in_place(&image).await?;
        Ok(PluginResult::ok("Layer inverted successfully"))
    }
}
```

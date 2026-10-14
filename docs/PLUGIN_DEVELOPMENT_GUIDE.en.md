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
├── plugin.json              # Declarative manifest (required)
├── icon.svg                 # Vector icon for the floating palette (required)
├── README.md                # Mandatory plugin documentation (EN & RU)
└── models/                  # (Optional) Local ONNX neural weights
    └── my_model.onnx
```

### A. No-Code Declarative Macros (`plugin_type: "no_code"`)
Zero coding or compilation required. Authors describe a sequence of PhotoCraft engine commands and arguments in pure JSON.
- **Barrier to entry:** Zero.
- **Use cases:** Batch color grading, adjustment stacks, vignetting, frequency separation, export routines.

### B. Native AI Plugins (`plugin_type: "native_ai"`)
Leverage local deep neural networks (ONNX) through ONNX Runtime (`ort`) with GPU acceleration (DirectML / CoreML / CPU fallback).
- **Capabilities:** Neural background removal, subject segmentation, monocular 3D depth estimation, inpainting, super-resolution.
- **Autonomy:** Neural weights are packaged inside the plugin folder or automatically downloaded from Hugging Face on first run with SHA-256 integrity verification.

### C. Native Algorithm Plugins (`plugin_type: "native_algorithm"`)
Leverage high-performance compiled Rust/C++ algorithms embedded in the companion core without neural networks.
- **Capabilities:** High-fidelity raster-to-vector tracing (VTracer / Bezier curve fitting), direct clipboard export to Figma/Illustrator, SVG generation.
- **Zero Weights:** Does not require heavy `.onnx` models, executes instantly on CPU.

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
| `plugin_type` | `string` | No | `"no_code"`, `"native_ai"`, `"native_algorithm"`, `"script"`, `"external_cli"` |
| `default_hotkey`| `string` | No | Default global hotkey (e.g., `"Ctrl+Shift+B"`, `"Alt+F8"`) |
| `icon` | `string` | No | Built-in icon name or relative path (`"icon.svg"`) |
| `input` | `string` | No | Image source: `"active_layer"`, `"selection"`, `"canvas"` |
| `output_mode` | `string` | No | Insertion mode: `"in_place"`, `"new_layer"`, `"mask"` |
| `commands` | `array` | For No-Code | Sequence of PhotoCraft engine steps |
| `post_commands` | `array` | No | PhotoCraft engine commands run after inference (e.g. `layer.mergeDown`) |
| `ai` | `object` | For Native AI | Declarative neural pipeline specification (`AiPipelineConfig`) |
| `algorithm` | `object` | For Native Algo | Declarative algorithm parameters (`AlgorithmConfig`) |
| `models` | `array` | For AI | Neural model requirements with download URLs and hashes |

---

## 4. Native AI Pipeline Specification (`ai` / `AiPipelineConfig`)

When creating a `native_ai` plugin, configure the neural pipeline in `plugin.json`:

```json
{
  "id": "ai.photocraft.plugins.depth_map",
  "name": "AI Depth Map (Depth Anything V2)",
  "version": "1.0.0",
  "plugin_type": "native_ai",
  "input": "active_layer",
  "output_mode": "new_layer",
  "ai": {
    "task": "segmentation_mask",
    "input_size": 518,
    "normalize": "image_net",
    "output_type": "grayscale_layer"
  },
  "models": [
    {
      "name": "depth-anything-v2-small.onnx",
      "required": true,
      "size_mb": 99,
      "url": "https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx",
      "sha256": "afb6a5c28f3b6bf1618c6e43f02073ef9dfdc70e937502d51603e57b0a1df10c"
    }
  ]
}
```

### Parameters:
* **`task`** (required): Pipeline inference paradigm:
  - `"segmentation_mask"`: Object segmentation or disparity/depth map estimation.
  - `"image_to_image"`: 1:1 pixel transform (denoising, colorization).
  - `"super_resolution"`: Upscaling resolution (e.g., Real-ESRGAN).
  - `"inpainting"`: Content-aware fill / brush replacement (e.g., LaMa).
* **`input_size`** (optional): Target square tensor resolution in pixels (e.g., `518` for Depth Anything V2, `1024` for RMBG-1.4). Defaults to `1024`.
* **`normalize`** (optional): Input tensor normalization method:
  - `"zero_to_one"`: `[0.0, 1.0]`.
  - `"centered"`: `(pixel / 255.0 - 0.5) / 1.0` (RMBG).
  - `"raw_255"`: Raw floating point `[0.0, 255.0]` (LaMa).
  - `"image_net"`: ImageNet standardization `(pixel/255.0 - mean) / std` (ViT / Depth Anything V2).
* **`output_type`** (optional): Result layer representation:
  - `"alpha_cutout"` (default): Extracted subject cutout with soft transparent alpha channel.
  - `"grayscale_layer"`: 8-bit monochromatic layer `Rgba([d, d, d, 255])` for depth maps, height maps, or masks.
* **`scale`** (optional): Integer multiplier for super-resolution (e.g., `2` or `4`).
* **`tile_size`** & **`tile_pad`** (optional): Tile dimensions and blend padding for processing ultra-high resolution images on limited GPU VRAM (e.g., `512` and `32`).

---

## 5. Native Algorithm Specification (`algorithm` / `AlgorithmConfig`)

For plugins leveraging embedded high-speed compiled Rust/C++ algorithms without neural weights:

```json
{
  "id": "algorithm.photocraft.plugins.vectorize",
  "name": "Smart Vectorizer / SVG Trace",
  "version": "1.0.0",
  "plugin_type": "native_algorithm",
  "input": "active_layer",
  "output_mode": "new_layer",
  "algorithm": {
    "task": "vector_trace",
    "mode": "spline",
    "color_mode": "color",
    "filter_speckle": 4,
    "corner_threshold": 60,
    "copy_clipboard": true,
    "export_file": true
  },
  "models": []
}
```

* **`task`**: Core algorithm identifier (`"vector_trace"`).
* **`mode`**: Curve fitting mode (`"spline"`, `"polygon"`).
* **`color_mode`**: Color segmentation strategy (`"color"`, `"binary"`).
* **`filter_speckle`**: Noise suppression threshold in square pixels.
* **`corner_threshold`**: Angle tolerance in degrees for smooth corners.
* **`copy_clipboard`**: Automatically copies generated SVG markup to the OS clipboard.
* **`export_file`**: Writes output `.svg` to local export folder.

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

## 6.1. Mandatory Plugin Documentation (`README.md`)

Every plugin directory in `plugins/<id>/` **must contain a `README.md` file**. Any plugin submitted without clear documentation violates the project's Definition of Done (DoD).

> [!TIP]
> **Interactive In-App Display:** The content of your `README.md` is rendered directly inside the **PhotoCraft Companion Manager**!
> Users can click the vector document icon on any plugin card to open a styled modal with bilingual tabs `[ Russian | English ]`, vector bullet dots, code badges, and clickable links to model weights.

### Structure Requirements:
1. **Bilingual Presentation:** Include clear sections in both **English** and **Russian**.
2. **Practical Use Cases:** Provide at least 2 concrete examples of how users apply the plugin's output in PhotoCraft (e.g. layer masks, bokeh blur, retouching).
3. **Model Links:** If using neural models, provide direct download links to verified Hugging Face or GitHub Release sources.

### Example Template:
```markdown
# <Plugin Name>

[![Plugin: <Name>](https://img.shields.io/badge/Plugin-<Name>-blue.svg)](plugin.json)
[![Model: <Model>](https://img.shields.io/badge/Model-<Model>-purple.svg)](<Model URL>)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
<Clear description of what the plugin does, input requirements, and underlying model/algorithm.>

### Use Cases:
- **<Use Case 1>:** <Step-by-step application in PhotoCraft, layers, effects, purpose.>
- **<Use Case 2>:** <Integration with design workflows.>

---

**Русский:**
<Русскоязычное описание возможностей плагина и формата входных данных.>

### Практическое применение:
- **<Сценарий 1>:** <Пошаговое описание использования в PhotoCraft, как комбинировать с инструментами редактора.>
- **<Сценарий 2>:** <Второй пример практической пользы.>
```

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

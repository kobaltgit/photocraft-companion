# PhotoCraft Plugin Development Guide

Welcome to the **PhotoCraft** extension ecosystem!

PhotoCraft Companion provides an extensible framework for creating and distributing plugins across four primary categories:
1. **No-Code Macro (`no_code`)**: Declarative chains of PhotoCraft commands with zero code compilation;
2. **Native AI (`native_ai`)**: Local ONNX neural inference (DirectML with automatic CPU fallback) featuring seamless tiling;
3. **Native Algorithm (`native_algorithm`)**: High-performance compiled algorithms (e.g. vector tracing, geometry);
4. **Interactive UI (`interactive_ui`)**: Floating tool palettes, assistants, and color harmony panels powered by declarative widgets and a sandboxed Rhai script engine with bidirectional canvas synchronization.

The companion core is built upon the **Zero-Core Hardcoding** principle: the runtime remains 100% agnostic to individual plugin semantics and declaratively interprets `plugin.json` manifests.

---

## 1. Plugin Package Structure (`.pcpl`)

A `.pcpl` package is a standard ZIP archive structured as follows:

```text
my_plugin/
├── plugin.json         # Main declarative manifest (mandatory)
├── icon.svg            # Custom vector SVG icon (mandatory)
├── README.md           # Plugin documentation and use cases (mandatory)
├── logic.rhai          # Reactive script logic (mandatory for interactive_ui)
└── LICENSE             # License file (MIT, Apache-2.0, etc.)
```

> [!IMPORTANT]
> **Heavy models (`*.onnx`, weight binaries) are STRICTLY PROHIBITED inside `.pcpl` archives and Git commits!**  
> Plugin packages must remain lightweight (< 50 KB). Model weights are downloaded automatically by the companion manager using direct `url` links declared in the `"models"` section.

---

## 2. Vector Icon Standards (`icon.svg`)

To represent your plugin on the floating **Toolstrip** and in the Plugin Manager, a clean vector icon is required.

1. **Format:** Pure Strict SVG (`.svg`). Embedded bitmaps (`.png`, `.jpg`) are rejected.
2. **ViewBox:** Standard `viewBox="0 0 24 24"` with a 1:1 aspect ratio.
3. **Theme Adaptation (Dark / Light):**
   * Use `stroke="currentColor"` or `fill="currentColor"` for primary shapes.
   * The companion UI engine automatically adapts `currentColor`:
     - Light contrast `#E6E6E6` in Dark Mode.
     - Dark contrast `#1E1E1E` in Light Mode.
   * Monochromatic shades (`#FFFFFF` or `#000000`) are also safely inverted.
   * Custom accent colors (`#3186FF`, `#FF5555`) remain preserved.
4. **Line Styles:** Recommended `stroke-width="1.75"` or `"2"`, with `stroke-linecap="round"` and `stroke-linejoin="round"`.
5. **No Raster Data:** Never embed base64 raster images (`<image xlink:href="data:image/png;base64,...">`).

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="9"/>
  <path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor"/>
</svg>
```

---

## 3. Plugin Manifest (`plugin.json`)

### 3.1. No-Code Macro (`plugin_type: "no_code"`)
Declarative sequence of PhotoCraft commands executed over the control socket:

```json
{
  "id": "macro.photocraft.plugins.black_and_white",
  "name": "B&W Film Pop (No-Code)",
  "version": "1.0.0",
  "description": "Duplicates active layer, converts to grayscale with auto-contrast adjustment.",
  "author": "kobaltGIT",
  "plugin_type": "no_code",
  "icon": "icon.svg",
  "commands": [
    {
      "command": "layer.duplicate",
      "params": {},
      "description": "Duplicate source layer"
    },
    {
      "command": "image.adjustments.desaturate",
      "params": {},
      "description": "Desaturate layer pixels"
    }
  ]
}
```

### 3.2. Native AI (`plugin_type: "native_ai"`)
Local neural execution using embedded ONNX Runtime (DirectML with CPU fallback and tile decomposition):

```json
{
  "id": "ai.photocraft.plugins.remove_bg",
  "name": "AI Remove Background (RMBG-1.4)",
  "version": "1.0.0",
  "description": "High-fidelity neural background removal for PhotoCraft using Bria AI RMBG-1.4 model via local ONNX Runtime.",
  "author": "kobaltGIT",
  "plugin_type": "native_ai",
  "action_id": "plugin:remove_bg",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "new_layer",
  "post_commands": [],
  "ai": {
    "task": "segmentation_mask",
    "input_size": 1024,
    "normalize": "centered",
    "output_type": "alpha_cutout"
  },
  "models": [
    {
      "name": "rmbg-1.4.onnx",
      "required": true,
      "size_mb": 168,
      "url": "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx",
      "sha256": "4733475c754d9c727ebc4c81b953d6ab1bf55eecdfca692289659341496a77d2"
    }
  ]
}
```

### 3.3. Native Algorithm (`plugin_type: "native_algorithm"`)
Native compiled routines (such as raster-to-SVG vectorization via VTracer):

```json
{
  "id": "algorithm.photocraft.plugins.vectorize",
  "name": "Smart Vectorizer / SVG Trace",
  "version": "1.0.0",
  "description": "High-fidelity vectorization and SVG tracing of raster layers with Bezier curve fitting.",
  "author": "kobaltGIT",
  "plugin_type": "native_algorithm",
  "action_id": "plugin:vectorize",
  "icon": "icon.svg",
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

### 3.4. Interactive UI (`plugin_type: "interactive_ui"`)
Interactive tool windows, floating assistants, and live palettes with Rhai scripting and multi-viewport rendering:

```json
{
  "id": "ui.photocraft.plugins.chromasense",
  "name": "ChromaSense",
  "version": "1.0.0",
  "description": "Accessibility Color Assistant for artists with color vision deficiencies",
  "author": "PhotoCraft Community",
  "plugin_type": "interactive_ui",
  "icon": "icon.svg",
  "commands": [],
  "interactive_ui": {
    "ipc_subscriptions": [
      "session.colors"
    ],
    "window_title": "ChromaSense A11y",
    "window_title_ru": "ChromaSense Доступность",
    "default_width": 350.0,
    "default_height": 570.0,
    "script": "logic.rhai",
    "widgets": [
      {
        "type": "color_slot",
        "id": "color_fg",
        "label": "Active Color",
        "label_ru": "Выбранный цвет",
        "default_hex": "#2563eb",
        "selectable": true,
        "target_binding": "edit_target"
      },
      {
        "type": "button",
        "id": "swap_colors",
        "label": "Swap Colors",
        "label_ru": "Поменять цвета",
        "icon": "swap"
      },
      {
        "type": "color_slot",
        "id": "color_sim",
        "label": "Vision Simulation",
        "label_ru": "Как видит зритель",
        "default_hex": "#2563eb",
        "read_only": true
      },
      {
        "type": "color_slot",
        "id": "color_bg",
        "label": "Background",
        "label_ru": "Цвет фона",
        "default_hex": "#ffffff",
        "selectable": true,
        "target_binding": "edit_target"
      },
      {
        "type": "dropdown",
        "id": "vision_profile",
        "label": "Vision Profile",
        "label_ru": "Профиль зрения",
        "options": [
          "Normal Vision",
          "Deuteranopia (Green-blind)",
          "Protanopia (Red-blind)",
          "Tritanopia (Blue-blind)"
        ],
        "options_ru": [
          "Обычное зрение",
          "Дейтеранопия (зелёная слепота)",
          "Протанопия (красная слепота)",
          "Тританопия (синяя слепота)"
        ],
        "default_index": 0
      },
      {
        "type": "hue_wheel",
        "id": "compass",
        "source_color_id": "edit_target",
        "show_pointer": true
      },
      {
        "type": "banner",
        "id": "wcag_status",
        "message_binding": "collision_warning",
        "style_binding": "collision_style"
      },
      {
        "type": "slider",
        "id": "red",
        "label": "Red",
        "label_ru": "Красный",
        "min": 0.0,
        "max": 255.0,
        "step": 1.0,
        "default": 37.0
      }
    ]
  }
}
```

---

## 4. Declarative Widget Specification (`widgets`)

All window elements are declared inside the `"widgets"` array. Each widget has a mandatory `"type"` tag:

| Type (`type`) | Parameters | Description |
| :--- | :--- | :--- |
| **`color_slot`** | `id`, `label`, `label_ru`, `default_hex`, `read_only`, `selectable`, `target_binding` | Interactive color swatch. When `selectable: true`, clicking sets focus by writing its `id` to `target_binding`. When `read_only: true`, it displays simulated perception results. |
| **`slider`** | `id`, `label`, `label_ru`, `min`, `max`, `step`, `default` | Smooth numeric slider. Value is bound to `state[id]`. |
| **`dropdown`** | `id`, `label`, `label_ru`, `options`, `options_ru`, `default_index` | ComboBox selector supporting localized option arrays (`options_ru`). Current index stored in `state[id]`. |
| **`toggle`** | `id`, `label`, `label_ru`, `default` | Boolean checkbox switch (`true`/`false`). |
| **`button`** | `id`, `label`, `label_ru`, `icon` | Trigger button. On click, sets transient action flag `action_<id> = true` in the state. Supports vector glyphs (e.g., `"swap"`). |
| **`hue_wheel`** | `id`, `source_color_id`, `show_pointer` | Vector circular HSL color wheel. Supports dynamic target indirection: pointing `source_color_id` to `edit_target` automatically tracks the active swatch. |
| **`banner`** | `id`, `message_binding`, `style_binding` | Multi-line diagnostic banner. `style_binding` supports `"info"`, `"warning"`, `"error"`, `"success"`. |
| **`label`** | `text_binding` | Dynamic text label bound to a state key. |
| **`separator`** | *(none)* | Subtle horizontal dividing line. |
| **`row`** | `items: [ WidgetNode, ... ]` | Horizontal container grouping widgets onto a single line. |

---

## 5. Rhai Scripting Logic (`logic.rhai`)

`interactive_ui` plugins are driven by sandboxed **Rhai** scripts (safe Rust runtime with zero uncontrolled filesystem or network access).

### 5.1. Lifecycle Hooks

#### 1. `on_state_changed(state) -> state`
Triggered whenever the user modifies a widget in the UI (moving sliders, clicking buttons, selecting dropdowns):
```rhai
fn on_state_changed(state) {
    if state["action_swap_colors"] == true {
        state["action_swap_colors"] = false;
        let temp = state["color_fg"];
        state["color_fg"] = state["color_bg"];
        state["color_bg"] = temp;
    }

    state = recalculate_diagnostics(state);

    // Push updated active color to PhotoCraft
    execute_command("tools.setColors", #{
        foreground: state["color_fg"],
        background: state["color_bg"]
    });

    return state;
}
```

#### 2. `on_session_colors(fg_hex, bg_hex, state) -> state`
Triggered automatically when the artist changes colors inside PhotoCraft using the **Eyedropper (`I`)**, reset defaults **`D`**, or inversion **`X`**:
```rhai
fn on_session_colors(fg_hex, bg_hex, state) {
    state["color_fg"] = fg_hex;
    state["color_bg"] = bg_hex;

    let active_hex = state[state["edit_target"]];
    let rgb = hex_to_rgb(active_hex);
    state["red"] = rgb["r"];
    state["green"] = rgb["g"];
    state["blue"] = rgb["b"];

    state = recalculate_diagnostics(state);

    // NOTE: NEVER call set_colors or execute_command("tools.setColors") inside this hook
    // to prevent infinite echo loops!
    return state;
}
```

### 5.2. Built-in Native Rhai Functions

* **Editor Control:**
  - `execute_command(command, params_map)`: Dispatch JSON-RPC command to PhotoCraft socket.
  - `execute_command(command)`: Dispatch command with null arguments.
  - `set_colors(fg_hex, bg_hex)`: Instant foreground & background brush color assignment.
* **Localization (i18n):**
  - `t("English text", "Russian text")`: Resolves translated string matching active companion language.
  - `current_language()`: Returns `"ru"` or `"en"`.
* **Color Math & Accessibility:**
  - `rgb_to_hex(r, g, b) -> String`: Converts 0..255 components to `#rrggbb`.
  - `hex_to_rgb(hex) -> Map`: Returns associative map `#{ r: 255, g: 0, b: 0 }`.
  - `get_named_color(r, g, b) -> String`: Perceptual search across 152 CSS3/fine art pigments.
  - `simulate_cvd(r, g, b, profile_idx) -> String`: Brettel LMS color blindness simulation (0: Normal, 1: Deuteranopia, 2: Protanopia, 3: Tritanopia).
  - `calculate_contrast(r1, g1, b1, r2, g2, b2) -> Float`: Symmetric W3C WCAG 2.1 relative luminance contrast calculation.

---

## 6. Bidirectional Sync & Echo Guard (`ipc_subscriptions`)

Declaring `"ipc_subscriptions": ["session.colors"]` engages companion synchronization:
1. **Zero-Waste Socket Poller:** PhotoCraft socket is queried (180ms) **only when at least one subscribed tool window is open**. Closed windows incur 0% CPU overhead.
2. **Echo Loop Prevention:** When a plugin calls `set_colors` or `tools.setColors`, values are tracked in `echo_guard`. Incoming socket echoes are discarded, preventing slider jitter and feedback storms.

---

## 7. Neural Pipeline Configuration (`ai` / `AiPipelineConfig`)

For `native_ai` plugins:
* **`task`** (required):
  - `"segmentation_mask"`: Cutouts, depth/normal maps (RMBG, Depth Anything).
  - `"image_to_image"`: 1:1 pixel transforms, face restoration (GFPGAN).
  - `"super_resolution"`: Resolution upscaling (Real-ESRGAN).
  - `"inpainting"`: Selection fill and object erasure (LaMa).
* **`input_size`** (optional): Square input tensor resolution (e.g. `518`, `1024`). Defaults to `1024`.
* **`normalize`** (optional):
  - `"zero_to_one"`: `[0.0, 1.0]`.
  - `"centered"`: `(pixel / 255.0 - 0.5) / 1.0`.
  - `"raw_255"`: `[0.0, 255.0]`.
  - `"image_net"`: ImageNet mean/std standardization.
* **`output_type`** (optional):
  - `"alpha_cutout"` (default): Transparent alpha cutout layer.
  - `"grayscale_layer"`: 8-bit monochromatic depth/height map layer.
* **`scale`** (optional): Integer multiplier for super-resolution (e.g. `2` or `4`).
* **`tile_size`** & **`tile_pad`** (optional): Tile dimensions and blending margins for processing huge canvases on limited GPU VRAM (e.g. `512` and `32`).

---

## 8. Native Algorithm Configuration (`algorithm` / `AlgorithmConfig`)

For `native_algorithm` plugins:
* **`task`**: `"vector_trace"`.
* **`mode`**: `"spline"`, `"polygon"`.
* **`color_mode`**: `"color"`, `"binary"`.
* **`filter_speckle`**: Noise speckle filter size in pixels (e.g. `4`).
* **`corner_threshold`**: Corner threshold angle (e.g. `60`).
* **`copy_clipboard`**: Auto-copy generated SVG to system clipboard.
* **`export_file`**: Export `.svg` file to companion temporary export folder.

---

## 9. Mandatory Plugin Documentation (`README.md`)

Every plugin directory `plugins/<id>/` **must contain a `README.md`**. Plugins lacking documentation are rejected by the CI validator:
1. **Bilingual:** Synchronized sections in both **English** and **Russian**.
2. **Practical Use Cases:** Minimum 2 concrete application workflows in PhotoCraft.
3. **Model Links:** Direct URLs to Hugging Face or GitHub Releases for neural weights.

---

## 10. Validation and Packaging (`.pcpl`)

To validate compliance, build registry indices, and produce `.pcpl` archives:

```powershell
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
```

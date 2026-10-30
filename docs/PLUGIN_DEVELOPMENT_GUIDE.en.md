# PhotoCraft Plugin Development Guide

Welcome to the **PhotoCraft** extension ecosystem!

PhotoCraft Companion provides an extensible framework for creating and distributing plugins across four primary categories:
1. **No-Code Macro (`no_code`)**: Declarative chains of PhotoCraft commands powered by the **Workflow Engine**, featuring context variables, conditionals, and error handling without code compilation;
2. **Native AI (`native_ai`)**: Local ONNX neural inference (DirectML with automatic CPU fallback, warm session caching, and seamless tiling), pre-commands, Photoshop-style tool variations (**Sub-Actions**), and post-commands;
3. **Native Algorithm (`native_algorithm`)**: High-performance compiled algorithms (e.g. vector tracing, geometry) with direct canvas output, system clipboard copying, and file export;
4. **Interactive UI (`interactive_ui`)**: Dedicated Multi-Viewport floating tool palettes, assistants, and color harmony panels powered by declarative widgets and a sandboxed Rhai script engine with bidirectional canvas synchronization (`session.colors`) and echo suppression.

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

The manifest specifies the plugin category, metadata, input source, execution pipelines, AI or algorithm configurations, sub-actions, and output integration mode.

### 3.1. No-Code Macro (`plugin_type: "no_code"`)
Declarative sequence of PhotoCraft commands powered by the **Workflow Engine**. For `no_code` plugins, the `commands` array represents the plugin body and executes exactly once:

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
      "save_as": "monochrome_layer",
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
Local neural execution using embedded ONNX Runtime (DirectML with CPU fallback, session caching, and tile decomposition). Supports preparatory commands (`commands`), Photoshop-style tool variations (`sub_actions`), and finishing commands (`post_commands`):

```json
{
  "id": "ai.photocraft.plugins.remove_bg",
  "name": "AI Remove Background (RMBG-1.4)",
  "version": "1.1.0",
  "description": "High-fidelity neural background removal for PhotoCraft using Bria AI RMBG-1.4 model via local ONNX Runtime.",
  "author": "kobaltGIT",
  "plugin_type": "native_ai",
  "action_id": "plugin:remove_bg",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "new_layer",
  "post_commands": [],
  "sub_actions": [
    {
      "id": "cutout",
      "name": "Remove Background (Cutout)",
      "name_ru": "Удаление фона (Вырезка)",
      "icon": "icon.svg",
      "post_commands": [
        {
          "command": "layer.setProps",
          "params": {
            "layer": "$source_layer",
            "visible": false
          },
          "description": "Hide source layer to avoid visual duplicate"
        }
      ]
    },
    {
      "id": "mask",
      "name": "Create Layer Mask",
      "name_ru": "Создать маску слоя",
      "icon": "icon_mask.svg",
      "post_commands": [
        {
          "command": "select.loadSelection",
          "params": {
            "channel": "transparency",
            "layer": "$result_layer"
          },
          "description": "Load alpha channel of AI result into active selection"
        },
        {
          "command": "layer.delete",
          "params": {
            "layer": "$result_layer"
          },
          "description": "Clean up temporary AI result layer"
        },
        {
          "command": "layer.select",
          "params": {
            "layer": "$source_layer"
          },
          "description": "Switch focus back to original layer"
        },
        {
          "command": "layer.layerMask.revealSelection",
          "params": {},
          "description": "Create non-destructive layer mask on source layer"
        }
      ]
    }
  ],
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
Interactive tool windows, assistants, and palettes in dedicated Viewports with declarative UI widgets and Rhai reactive scripting:

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

### 3.5. Metadata and Parameter Internationalization (i18n)

PhotoCraft Companion automatically detects the active PhotoCraft host language (across all 22 BCP 47 supported languages) and dynamically adapts plugins on the fly.

#### A. Multilingual Plugin Metadata (`locales`):
To localize plugin names and descriptions in Companion Manager and Toolstrip, declare a `locales` map in `plugin.json`. Backward-compatible fields `name_ru` and `description_ru` are also supported:

```json
{
  "id": "macro.photocraft.plugins.dodge_and_burn",
  "name": "Pro Dodge & Burn (50% Gray)",
  "name_ru": "Pro Dodge & Burn (50% Серый)",
  "description": "Creates a non-destructive 50% Gray Soft Light layer...",
  "description_ru": "Создает недеструктивный слой 50% серого...",
  "locales": {
    "de": {
      "name": "Pro Dodge & Burn (50% Grau)",
      "description": "Erstellt eine zerstörungsfreie 50% Grau Weiches Licht-Ebene..."
    },
    "fr": {
      "name": "Pro Dodge & Burn (Gris 50%)",
      "description": "Crée un calque non destructif Gris 50% en Lumière tamisée..."
    },
    "ja": {
      "name": "プロ Dodge & Burn (50% グレー)",
      "description": "プロの輪郭強調のための..."
    },
    "zh-hans": {
      "name": "专业双曲线加深减淡 (50% 中性灰)",
      "description": "创建非破坏性50%柔光中性灰图层..."
    }
  }
}
```

For sub-actions (`sub_actions`), the `locales` dictionary is also supported:
```json
"sub_actions": [
  {
    "id": "mask",
    "name": "Create Layer Mask",
    "name_ru": "Создать маску слоя",
    "locales": {
      "de": { "name": "Ebenenmaske erstellen" },
      "fr": { "name": "Créer un masque de fusion" },
      "ja": { "name": "レイヤーマスクを作成" }
    }
  }
]
```

#### B. Canonical Parameters and Smart Parameter Retry:
* **The Golden Rule:** Always declare command names and parameter string values using **canonical English** (e.g. `"blend": "Soft Light"`, `"channel": "transparency"`).
* **Smart Parameter Retry Engine:** When PhotoCraft runs with localized UI (e.g. German, French, Russian), certain internal operations expect localized parameters (such as `"Weiches Licht"` instead of `"Soft Light"`). Companion's workflow runner handles this seamlessly:
  1. It first submits the canonical English command (`"Soft Light"`).
  2. If PhotoCraft accepts it, execution proceeds with zero overhead.
  3. If PhotoCraft returns a parameter rejection error, Companion translates the parameter value into the active host locale (among all 22 supported languages) and retries the command automatically.
  Plugin creators do not need to maintain language forks or conditional logic for host locales!

---

## 4. Workflow Engine & Command Pipelines

PhotoCraft Companion includes a declarative **Workflow Engine** enabling multi-step image manipulation pipelines without compiling Rust code.

### 4.1. Runtime Context Variables

During `commands` and `post_commands` execution, the engine automatically resolves context variables:

| Variable | Type | Description |
| :--- | :--- | :--- |
| **`$source_layer`** | integer (`u64`) | ID of the source layer active before the plugin executed. |
| **`$result_layer`** | integer (`u64`) | ID of the newly generated result layer (from AI or algorithm). |
| **`$active_layer`** | integer (`u64`) | ID of the currently selected layer in PhotoCraft. |
| **`$doc_width`** | integer (`u32`) | Canvas width in pixels. |
| **`$doc_height`** | integer (`u32`) | Canvas height in pixels. |
| **`$has_selection`** | boolean (`bool`) | Whether an active selection exists on the canvas. |
| **`$fg_color`** | string | Current brush foreground hex color (e.g. `"#2563eb"`). |
| **`$bg_color`** | string | Current brush background hex color (e.g. `"#ffffff"`). |
| **`$custom_var`** | any | Custom variable saved by a preceding step with `save_as`. |

Variables can be assigned directly to JSON fields:
```json
"params": {
  "layer": "$source_layer",
  "visible": false
}
```
Or interpolated into strings via `{{...}}`:
```json
"params": {
  "name": "Layer copy of {{$source_layer}} ({{$doc_width}}x{{$doc_height}})"
}
```

### 4.2. Command Step Specification (`PluginCommandStep`)

Each step in `commands` or `post_commands` is declared as an object:

```json
{
  "command": "layer.setProps",
  "params": {
    "layer": "$source_layer",
    "visible": false
  },
  "save_as": "updated_layer",
  "when": "$has_selection == true",
  "ignore_errors": true,
  "description": "Hide source layer non-destructively"
}
```

* **`command`** (mandatory): PhotoCraft action identifier (e.g. `layer.duplicate`, `layer.select`, `layer.delete`, `layer.setProps`, `image.adjustments.desaturate`, `select.loadSelection`, `layer.layerMask.revealSelection`).
* **`params`** (optional): Arguments passed to the command. Defaults to `{}` when empty or null.
* **`save_as`** (optional): Name of the variable to store the resulting layer ID for subsequent steps (`$updated_layer`).
* **`when`** (optional): Boolean expression condition. If false, the step is safely skipped.
* **`ignore_errors`** (optional, default `false`): If `true`, step failure does not abort the workflow (useful for `image.revealAll` or `layer.mergeDown`).
* **`script`** (optional): Inline Rhai script snippet for dynamic evaluations.
* **`description`** (optional): Diagnostic note for logging and debugging.

### 4.3. Execution Lifecycle (Pre vs Post vs No-Code)

1. **`no_code` Plugins:**
   The `commands` array represents the **plugin body** and executes exactly once via `plugin.execute()`. The dispatcher guarantees zero layer duplicates (BUG-074).
2. **`native_ai` and `native_algorithm` Plugins:**
   * Steps in `commands` run **before** AI/algorithm execution (Pre-commands) to prepare the canvas.
   * The core runs inference/algorithm, pastes the result, and sets `$source_layer` and `$result_layer`.
   * Steps in `post_commands` run **after** the result is pasted for masking, compositing, or cleaning up temporary layers.

---

## 5. Photoshop-Style Tool Variations (`sub_actions`)

Plugins can offer multiple tool variations under a single Toolstrip button (e.g. Cutout vs Layer Mask).

### 5.1. Declaring Sub-Actions in `plugin.json`

```json
"sub_actions": [
  {
    "id": "cutout",
    "name": "Remove Background (Cutout)",
    "name_ru": "Удаление фона (Вырезка)",
    "icon": "icon.svg",
    "post_commands": [
      {
        "command": "layer.setProps",
        "params": { "layer": "$source_layer", "visible": false },
        "description": "Hide source layer"
      }
    ]
  },
  {
    "id": "mask",
    "name": "Create Layer Mask",
    "name_ru": "Создать маску слоя",
    "icon": "icon_mask.svg",
    "post_commands": [
      {
        "command": "select.loadSelection",
        "params": { "channel": "transparency", "layer": "$result_layer" }
      },
      { "command": "layer.delete", "params": { "layer": "$result_layer" } },
      { "command": "layer.select", "params": { "layer": "$source_layer" } },
      { "command": "layer.layerMask.revealSelection", "params": {} }
    ]
  }
]
```

### 5.2. User Interface Interaction

1. **Corner Chevron Indicator:** If `sub_actions` are declared, a subtle vector chevron is rendered at the bottom-right corner of the tool button.
2. **Dedicated Flyout Viewport:**
   * Clicking the chevron or **long-pressing the tool button (> 350 ms)** opens a dedicated floating flyout window displaying all sub-tool variations.
   * Clicking a variation instantly activates it and persists the choice in user settings (`active_sub_actions`).
3. **Execution Routing:**
   * Subsequent single-clicks on the tool button execute the currently selected sub-action.
   * The core dispatcher routes execution through the chosen sub-action's `post_commands`.

---

## 6. Declarative Widgets Specification (`widgets`)

In `interactive_ui`, all window controls are declared inside the `"widgets"` array:

| Widget Type (`type`) | Parameters | Description |
| :--- | :--- | :--- |
| **`color_slot`** | `id`, `label`, `label_ru`, `default_hex`, `read_only`, `selectable`, `target_binding` | Interactive color swatch. If `selectable: true`, clicking selects this swatch as active target (`target_binding`). When `read_only: true`, displays non-editable results. |
| **`slider`** | `id`, `label`, `label_ru`, `min`, `max`, `step`, `default` | Numeric slider with vector rendering. Value is accessible in `state[id]`. |
| **`dropdown`** | `id`, `label`, `label_ru`, `options`, `options_ru`, `default_index` | Dropdown combobox with localized option labels. Selected index stored in `state[id]`. |
| **`toggle`** | `id`, `label`, `label_ru`, `default` | Checkbox toggle storing boolean (`true`/`false`). |
| **`button`** | `id`, `label`, `label_ru`, `icon` | Action button. When clicked, sets `action_<id> = true` in state. Supports vector icons (e.g. `"swap"`). |
| **`hue_wheel`** | `id`, `source_color_id`, `show_pointer` | Vector circular HSL color wheel with pointer indicating current tone. |
| **`banner`** | `id`, `message_binding`, `style_binding` | Dynamic notification banner (`"info"`, `"warning"`, `"error"`, `"success"`). |
| **`label`** | `text_binding` | Text line bound to a state key. |
| **`separator`** | *(none)* | Thin visual dividing line. |
| **`row`** | `items: [ WidgetNode, ... ]` | Horizontal container layout. |
| **`modifier_stack`** | `id`, `label`, `label_ru`, `allow_reordering`, `items: [ ModifierItemConfig, ... ]` | Non-destructive modifier stack with vector up/down reorder chevrons, step toggle checkboxes, adaptive column auto-alignment without stair-stepping (Rule 6.1 Layout Guard), and discrete formatted sliders. Modifier sequence is stored in `state[id + "_order"]`, step toggles in `state[item_id + "_enabled"]`, and radii in `state[item_id + "_value"]`. Reordered state persists across sessions automatically. |
| **`vector_pad`** | `id`, `label`, `label_ru`, `default_angle_deg`, `default_distance`, `max_distance` | Circular vector angle (0..360°) and distance controller with interactive viewfinder crosshair and gradient directional beam. Stores angle in `state[id + "_angle"]` and distance in `state[id + "_distance"]`. |
| **`image_preview`** | `id`, `height` | Interactive live preview canvas rendered by the raster engine with proportional scaling, a vector refresh button to re-fetch the layer from PhotoCraft, and texture memory guard. |

---

## 7. Reactive Scripting with Rhai (`logic.rhai`)

`interactive_ui` plugins are driven by a lightweight, sandboxed script in **Rhai**.

### 7.1. Lifecycle Hooks

#### 1. `on_state_changed(state) -> state`
Fired whenever the user interacts with any widget:
```rhai
fn on_state_changed(state) {
    if state["action_swap_colors"] == true {
        state["action_swap_colors"] = false;
        let temp = state["color_fg"];
        state["color_fg"] = state["color_bg"];
        state["color_bg"] = temp;
    }

    state = recalculate_diagnostics(state);

    execute_command("tools.setColors", #{
        foreground: state["color_fg"],
        background: state["color_bg"]
    });

    return state;
}
```

#### 2. `on_session_colors(fg_hex, bg_hex, state) -> state`
Fired automatically when the artist changes colors in PhotoCraft using the **Eyedropper (`I`)**, default reset (**`D`**), or swap (**`X`**):
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
    return state;
}
```

#### 3. `on_selection_state(has_selection, bounds, state) -> state`
Fired automatically when selection presence or selection contour bounds change in PhotoCraft (requires subscription to `"document.selection"`):
- `has_selection: bool` — whether an active selection exists on the canvas (`true`/`false`);
- `bounds: Array` — selection bounding box coordinates `[x, y, w, h]` (or `()` if no selection);
- `state: Map` — reactive state of the tool window widgets.
```rhai
fn on_selection_state(has_selection, bounds, state) {
    if !has_selection {
        state["base_saved"] = false;
        state["last_bounds"] = ();
    } else {
        let prev_bounds = state["last_bounds"];
        // If the artist redrew the selection contour in PhotoCraft, invalidate cached base
        if prev_bounds != () && bounds != () && prev_bounds != bounds {
            state["base_saved"] = false;
        }
        state["last_bounds"] = bounds;
    }
    return state;
}
```

### 7.2. Built-in Native Functions

* **Editor Control & Raster Engine:**
  - `execute_command(command, params_map)` — send command to PhotoCraft socket;
  - `execute_command(command)` — send command without parameters;
  - `set_colors(fg_hex, bg_hex)` — immediately set brush foreground and background colors;
  - `execute_raster_process(params_map, mode)` — execute high-speed 2.5D extrusion, bevel and shadow generation via native `raster_engine`. Modes: `"merged"` (single flat layer) or `"split"` (layered Shadow + Extrusion + Facade).
* **Localization (i18n):**
  - `t("English text", "Русский текст")` — returns translation for the active app language;
  - `current_language()` — returns `"ru"` or `"en"`.
* **Color Processing & Math:**
  - `rgb_to_hex(r, g, b) -> String` — convert $0..255$ integers to `#rrggbb`;
  - `hex_to_rgb(hex) -> Map` — returns map `#{ r: 255, g: 0, b: 0 }`;
  - `get_named_color(r, g, b) -> String` — returns semantic shade name from a 152-color database in active language;
  - `simulate_cvd(r, g, b, profile_idx) -> String` — Brettel-Vienot LMS color-blindness simulation;
  - `calculate_contrast(r1, g1, b1, r2, g2, b2) -> Float` — W3C WCAG 2.1 symmetric luminance contrast ratio.

---

## 8. Bidirectional Sync & Event Subscriptions (`ipc_subscriptions`)

Plugins can subscribe to system editor events via `"ipc_subscriptions"`:
```json
"ipc_subscriptions": [
  "document.selection",
  "session.colors",
  "active_layer.image"
]
```
### Supported Subscription Topics:

1. **`"document.selection"` — Selection Bi-Directional Reactivity:**
   - Dispatches selection existence and exact bounding box coordinates `[x, y, w, h]` into `on_selection_state(has_selection, bounds, state)`.
   - Allows plugins to immediately adapt to contour changes made by the artist on the PhotoCraft canvas (Lasso, Marquee, moving selection), implementing a robust non-destructive lifecycle and preventing contour overwrite (**Zero Data-Loss UX**).

2. **`"session.colors"` — Live PhotoCraft Palette Sync:**
   - Dispatches active foreground and background colors into `on_session_colors(fg_hex, bg_hex, state)`.
   - Leverages a smart **Zero-Waste Poller** (180 ms only while at least one plugin window is open, 0% CPU when closed) and **Echo Guard** (suppresses echoed command events to eliminate slider flicker and feedback loops).

3. **`"active_layer.image"` — Live Active Layer Capture:**
   - Asynchronously captures the raster image of the active layer for display in the `image_preview` widget and processing by the local raster engine.

### Interactive Tool Lifecycle Standards (Lifecycle Best Practices):
- **Zero Data-Loss UX on Reset:** The *Reset* button must reset slider values and checkboxes only in the Companion UI, under no circumstances clearing or destroying the artist's live selection contour on the canvas (`select.loadSelection` is never called). The current canvas contour is committed to the working base as a clean reference point.
- **Commit-on-Uncheck (Bake):** Unchecking an active modifier step must zero its radius and commit (bake) the current altered canvas shape into the working base alpha channel `__pc_companion_base_sel`.



---

## 9. AI Pipeline Specification (`ai` / `AiPipelineConfig`)

For `native_ai` plugins:

* **`task`** (mandatory):
  - `"segmentation_mask"`: object cutouts, depth maps, surface normals;
  - `"image_to_image"`: pixel-to-pixel transformation (denoising, face restoration);
  - `"super_resolution"`: upscale resolution (Real-ESRGAN);
  - `"inpainting"`: object deletion and generative fill (LaMa).
* **`input_size`** (optional): model input resolution (e.g. `518` for Depth Anything V2, `1024` for RMBG-1.4). Default `1024`.
* **`normalize`** (optional):
  - `"zero_to_one"`: `[0.0, 1.0]`;
  - `"centered"`: `(pixel / 255.0 - 0.5) / 1.0`;
  - `"raw_255"`: `[0.0, 255.0]`;
  - `"image_net"`: ImageNet standardization.
* **`output_type`** (optional):
  - `"alpha_cutout"` (default): transparent cutout pasted in-place;
  - `"grayscale_layer"`: 8-bit grayscale layer (depth maps, saliency).
* **`scale`** (optional): magnification factor for `super_resolution` (`2` or `4`).
* **`tile_size`** and **`tile_pad`** (optional): tile parameters for large canvas processing. The runtime automatically caches DirectML/CPU sessions across all tiles, accelerating inference up to 25x.

---

## 10. Algorithm Specification (`algorithm` / `AlgorithmConfig`)

For `native_algorithm` plugins:

* **`task`**: core algorithm (`"vector_trace"`).
* **`mode`**: curve fitting (`"spline"`, `"polygon"`).
* **`color_mode`**: color model (`"color"`, `"binary"`).
* **`filter_speckle`**: speckle filter size in pixels (e.g. `4`).
* **`corner_threshold`**: corner smoothing angle in degrees (e.g. `60`).
* **`copy_clipboard`**: auto-copy clean SVG to clipboard.
* **`export_file`**: save standalone `.svg` to export directory.

---

## 11. Mandatory Plugin Documentation (`README.md`)

Every plugin directory in `plugins/<id>/` **must contain a `README.md`**.

> [!TIP]
> **In-App Display:** Plugin `README.md` files are displayed directly inside the **PhotoCraft Companion Manager**! Users can view documentation with a language toggle `[ Russian | English ]` and clickable model weight links.

### `README.md` Requirements:
1. **Bilingual:** Synchronous English and Russian sections.
2. **Practical Use Cases:** At least 2 practical workflows in PhotoCraft.
3. **Weight Links:** Direct download links for reproducible downloads.

---

## 12. Validation and Packaging (`.pcpl`)

To validate plugins and package `.pcpl` archives:

```powershell
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
```

The validator automatically:
- Validates `plugin.json` syntax and mandatory fields;
- Verifies pure vector `icon.svg` compliance;
- Verifies bilingual `README.md` structure;
- Compiles `.pcpl` packages into `dist/`;
- Synchronizes plugin catalog tables in `README.md` and `README.ru.md`.

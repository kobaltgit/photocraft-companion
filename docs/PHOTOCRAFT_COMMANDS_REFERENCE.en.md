# PhotoCraft Engine Commands Reference

Complete catalog of built-in commands in the **PhotoCraft** graphics engine, available for No-Code declarative macros (`plugin_type: "no_code"`), native plugins, and the JSON-RPC control socket (port 18492).

> 💡 **Total Commands in PhotoCraft Engine:** **851**

**[English Version](PHOTOCRAFT_COMMANDS_REFERENCE.en.md)** • [Russian Version](PHOTOCRAFT_COMMANDS_REFERENCE.md)

---

### Table of Contents:

- [Actions (7)](#-actions)
- [Adjustment Layers (16)](#-adjustment-layers)
- [Brush (14)](#-brush)
- [Channel (21)](#-channel)
- [CloneSource (5)](#-clonesource)
- [Color (1)](#-color)
- [Count (7)](#-count)
- [Document & Engine Inspection (11)](#-document-engine-inspection)
- [Edit & Clipboard (77)](#-edit-clipboard)
- [File Operations (49)](#-file-operations)
- [Filter (17)](#-filter)
- [Filter Gallery (47)](#-filter-gallery)
- [Filter: Blur (16)](#-filter-blur)
- [Filter: Distort (9)](#-filter-distort)
- [Filter: Noise (5)](#-filter-noise)
- [Filter: Pixelate (7)](#-filter-pixelate)
- [Filter: Render (9)](#-filter-render)
- [Filter: Sharpen (5)](#-filter-sharpen)
- [Filter: Stylize (9)](#-filter-stylize)
- [Gradient (12)](#-gradient)
- [History (2)](#-history)
- [Image (41)](#-image)
- [Image Adjustments (23)](#-image-adjustments)
- [Jobs (2)](#-jobs)
- [Layer Operations (164)](#-layer-operations)
- [Layer Styles & Effects (23)](#-layer-styles-effects)
- [LayerComp (13)](#-layercomp)
- [MeasurementLog (3)](#-measurementlog)
- [Notes (4)](#-notes)
- [Paint (25)](#-paint)
- [Path (19)](#-path)
- [Pattern (10)](#-pattern)
- [Plugin (5)](#-plugin)
- [Prefs (3)](#-prefs)
- [Selections (34)](#-selections)
- [Shape (9)](#-shape)
- [Slice (7)](#-slice)
- [Style (5)](#-style)
- [Swatches (12)](#-swatches)
- [Timeline (7)](#-timeline)
- [Tool (5)](#-tool)
- [Tools & Colors (8)](#-tools-colors)
- [Type (59)](#-type)
- [Variables (1)](#-variables)
- [View (23)](#-view)

---

## 🔹 Actions (7)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `actions.move` | Move Action or Step | — | <details><summary><code>JSON Schema</code></summary><code>{"action":name¦index, "step":index?, "to":index}. Move a step within its action, or the action itself. to is the final zero-based index. Pending recorded steps can be moved.</code></details> | — |
| `actions.list` | List Actions | — | <code>{} → {actions:[{name, steps:count}], recording:name¦null}</code> | — |
| `actions.get` | Get Action | — | <details><summary><code>JSON Schema</code></summary><code>{"action":name¦index} → {name, steps:[[id, params]…]} (the shape file.automate.batch and droplets take)</code></details> | — |
| `actions.record` | Record Action | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str? (new action, default "Action N")} or {"action":name¦index} (append) → {action, index}</code></details> | — |
| `actions.stop` | Stop Recording | — | <details><summary><code>JSON Schema</code></summary><code>{} → {action, steps:count} (steps recorded since actions.record, queries and action-editing commands omitted)</code></details> | — |
| `actions.play` | Play Action | — | <details><summary><code>JSON Schema</code></summary><code>{"action":name¦index, "from":step?} → {action, ran, failed?:{step, id, error}}. step and from are 0-based. Stops at the first error (the command still returns ok, with failed set) and leaves one history step per step that ran. Nested calls are supported up to 16 levels; cycles are rejected. Recording stores a named call rather than its expanded commands. Each step is checked with Session::authorize when one is installed.</code></details> | — |
| `actions.delete` | Delete Action | — | <details><summary><code>JSON Schema</code></summary><code>{"action":name¦index, "step":index?} → {deleted:name} or {action, deletedStep, steps:count}. step is 0-based, including pending recorded steps. Whole-action deletion is refused while recording.</code></details> | — |

## 🔹 Adjustment Layers (16)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `layer.newAdjustmentLayer.brightnessContrast` | Brightness/Contrast… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"brightness":-150..150=0,"contrast":-50..100=0,"legacy":bool=false} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.levels` | Levels… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"inBlack":0..253=0,"gamma":0.01..9.99=1,"inWhite":2..255=255,"outBlack":0..255=0,"outWhite":0..255=255,"red":json,"green":json,"blue":json} (top level = composite; per channel {"inBlack","gamma","inWhite","outBlack","outWhite"} under red/green/blue, gray, cyan/magenta/yellow/black or lightness/a/b) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.curves` | Curves… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"points":json,"red":json,"green":json,"blue":json,"eyedropper":json} (curves as [[in,out],…] in 0..255, 2..19 points: points = composite; red/green/blue, gray, cyan/magenta/yellow/black or lightness/a/b per channel; eyedropper = the dialog's Set Black/Neutral Gray/White Point picker {"point":"black¦gray¦white","at":[x,y]} or {"point",…,"color":[r,g,b] 0..1}, RGB and Grayscale (no gray) only, applied over the given curves) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.exposure` | Exposure… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"exposure":-20..20=0,"offset":-0.5..0.5=0,"gamma":0.01..9.99=1} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.vibrance` | Vibrance… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"vibrance":-100..100=0,"saturation":-100..100=0} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.hueSaturation` | Hue/Saturation… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"hue":-180..180=0,"saturation":-100..100=0,"lightness":-100..100=0,"colorize":bool=false,"reds":json,"yellows":json,"greens":json,"cyans":json,"blues":json,"magentas":json} (per range {"hue","saturation","lightness","range":[4 hue degrees]}; colorize: hue 0..360, saturation 0..100) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.colorBalance` | Color Balance… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"shadows":json,"midtones":json,"highlights":json,"preserveLuminosity":bool=true} (each tone [cyan-red, magenta-green, yellow-blue] in -100..100) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.blackWhite` | Black & White… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"reds":-200..300=40,"yellows":-200..300=60,"greens":-200..300=40,"cyans":-200..300=60,"blues":-200..300=20,"magentas":-200..300=80,"tint":bool=false,"tintColor":"#rrggbb"} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.photoFilter` | Photo Filter… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"filter":"warming85¦warmingLBA¦warming81¦cooling80¦coolingLBB¦cooling82¦red¦orange¦yellow¦green¦cyan¦blue¦violet¦magenta¦sepia¦deepRed¦deepBlue¦deepEmerald¦deepYellow¦underwater","color":"#rrggbb","density":0..100=25,"preserveLuminosity":bool=true} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.channelMixer` | Channel Mixer… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"red":json,"green":json,"blue":json,"gray":json,"monochrome":bool=false} (per output channel [red %, green %, blue %, constant %] in -200..200; gray = the monochrome mix) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.invert` | Invert | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.posterize` | Posterize… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"levels":2..255=4} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.threshold` | Threshold… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"level":1..255=128} + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.gradientMap` | Gradient Map… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"stops":json,"reverse":bool=false,"dither":bool=false} (stops [[location 0..1, "#rrggbb"], …], 2..64) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.selectiveColor` | Selective Color… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"method":"relative¦absolute"="relative","colors":"reds¦yellows¦greens¦cyans¦blues¦magentas¦whites¦neutrals¦blacks"="reds","cyan":-100..100=0,"magenta":-100..100=0,"yellow":-100..100=0,"black":-100..100=0,"reds":json} (per-range [c,m,y,k] arrays: reds yellows greens cyans blues magentas whites neutrals blacks) + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |
| `layer.newAdjustmentLayer.colorLookup` | Color Lookup… | Layer › New Adjustment Layer | <details><summary><code>JSON Schema</code></summary><code>{"lut":"none¦warm¦cool¦tealOrange¦bleachBypass¦fadedFilm¦dayForNight¦monoContrast¦crossProcess"="none","file":text,"interpolation":"trilinear¦tetrahedral"="trilinear","dither":bool=false,"data":json} (file: .cube/.3dl/.look path; data: file text + "fileName") + {"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask, or white with no selection)}</code></details> | — |

## 🔹 Brush (14)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `brush.presets.list` | List Brush Presets | — | <code>{"full":bool=false}</code> | — |
| `brush.presets.save` | Save Brush Preset | — | <code>{"name":string,"brush":{…BrushSettings}?=current brush}</code> | — |
| `brush.presets.update` | Update Brush Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"name":string?=the preset the current brush was picked from,"brush":{…BrushSettings}?=current brush}</code></details> | — |
| `brush.presets.delete` | Delete Brush Preset | — | <code>{"name":string}</code> | — |
| `brush.presets.setCurrent` | Set Current Brush Preset | — | <code>{"name":string}</code> | — |
| `brush.defineFromSelection` | Define Brush Preset… | — | <code>{"name":string}</code> | — |
| `brush.get` | Get Brush | — | `{}` | — |
| `brush.presets.rename` | Rename Brush | — | <code>{"name":string,"newName":string} → {name}</code> | — |
| `brush.presets.move` | Move Brush | — | <details><summary><code>JSON Schema</code></summary><code>{"name":string,"group":string?=its group ("" = ungrouped),"folder":[name]? (nested folder names in the group, outermost first; [] = the group itself)=before's folder, else its own if it stays in its group, else [],"before":name? (preset to land before, in that folder) ¦ "index":n? (position in the group)=end of the folder} → {name, group, folder, index}</code></details> | — |
| `brush.presets.moveGroup` | Move Brush Group | — | <details><summary><code>JSON Schema</code></summary><code>{"group":string,"folder":[name]? (move this nested folder among its sibling folders instead),"before":group? (or sibling folder name) ¦ "index":n? (group or sibling folder position)=end} → {group, folder?, index}</code></details> | — |
| `brush.presets.renameGroup` | Rename Brush Group | — | <details><summary><code>JSON Schema</code></summary><code>{"group":string,"folder":[name]? (rename this nested folder instead),"newName":string} → {group, folder?}</code></details> | — |
| `brush.presets.deleteGroup` | Delete Brush Group | — | <details><summary><code>JSON Schema</code></summary><code>{"group":string,"folder":[name]? (delete this nested folder instead)} → {deleted, count}</code></details> | — |
| `brush.presets.importAbr` | Import Brushes… | — | <details><summary><code>JSON Schema</code></summary><code>{"path":".abr file"?,"data":base64 bytes?,"group":string?=file name,"replace":bool=true (replace a group of the same name),"select":bool=false (make the first imported preset current)} → {group, imported, count, folders (nested folders kept from the file), version, warnings}</code></details> | — |
| `brush.texturePattern` | Brush Texture Pattern | — | <details><summary><code>JSON Schema</code></summary><code>{"pattern":id¦name,"scale":1..1000?,"enabled":bool=true} (sets the session brush's Texture to that pattern)</code></details> | — |

## 🔹 Channel (21)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `channel.new` | New Channel… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"fill":"black¦white¦selection"="black","color":"#rrggbb"="#ff0000","opacity":0..100=50,"indicates":"masked¦selected"="masked"}</code></details> | — |
| `channel.newSpot` | New Spot Channel… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"color":"#rrggbb"="#ff0000","solidity":0..100=0,"fromSelection":bool=true}</code></details> | — |
| `channel.duplicate` | Duplicate Channel… | — | <details><summary><code>JSON Schema</code></summary><code>{"channel":index¦name¦"red¦…"¦"composite"¦"quickMask","name":str?,"document":index¦"new"?=active,"invert":bool=false}</code></details> | — |
| `channel.delete` | Delete Channel | — | <code>{"channel":index¦name?=targeted}</code> | — |
| `channel.rename` | Rename Channel | — | <code>{"channel":index¦name,"name":str}</code> | — |
| `channel.move` | Reorder Channel | — | <code>{"channel":index¦name,"to":index}</code> | — |
| `channel.target` | Target Channel | — | <code>{"channel":"composite"¦"red¦green¦blue¦…"¦index¦name="composite"}</code> | — |
| `channel.setVisible` | Channel Visibility | — | <code>{"channel":"composite"¦"red¦…"¦index¦name¦"quickMask","visible":bool?=toggle}</code> | — |
| `channel.options` | Channel Options… | — | <details><summary><code>JSON Schema</code></summary><code>{"channel":index¦name¦"quickMask","name":str?,"indicates":"masked¦selected¦spot"?,"color":"#rrggbb"?,"opacity":0..100?,"solidity":0..100?}</code></details> | — |
| `channel.mergeSpot` | Merge Spot Channel | — | <code>{"channel":index¦name?=targeted or first spot}</code> | — |
| `channel.split` | Split Channels | — | <code>{"closeOriginal":bool=true}</code> | — |
| `channel.merge` | Merge Channels… | — | <details><summary><code>JSON Schema</code></summary><code>{"mode":"rgb¦cmyk¦lab"="rgb","documents":[index,…]?=open grayscale docs of the active size,"name":str?}</code></details> | — |
| `channel.list` | Channels | — | `{}` | — |
| `channel.target.composite` | Target Composite Channel | — | `{}` | `Cmd+2` |
| `channel.target.slot3` | Target Channel 3 | — | `{}` | `Cmd+3` |
| `channel.target.slot4` | Target Channel 4 | — | `{}` | `Cmd+4` |
| `channel.target.slot5` | Target Channel 5 | — | `{}` | `Cmd+5` |
| `channel.target.slot6` | Target Channel 6 | — | `{}` | `Cmd+6` |
| `channel.target.slot7` | Target Channel 7 | — | `{}` | `Cmd+7` |
| `channel.target.slot8` | Target Channel 8 | — | `{}` | `Cmd+8` |
| `channel.target.slot9` | Target Channel 9 | — | `{}` | `Cmd+9` |

## 🔹 CloneSource (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `cloneSource.list` | Clone Sources | — | <details><summary><code>JSON Schema</code></summary><code>{} → {active,sources:[{index,source,anchor,offset,layer,width,height,rotation,flipH,flipV}],overlay}</code></details> | — |
| `cloneSource.select` | Select Clone Source | — | <code>{"index":0..4}</code> | — |
| `cloneSource.set` | Set Clone Source | — | <details><summary><code>JSON Schema</code></summary><code>{"index":0..4?=active,"source":[x,y]? (⌥-click; re-pairs with the next stroke),"offset":[dx,dy]?,"width":%?,"height":%?¦"scale":[w%,h%]?,"rotation":deg?,"flipH":bool?,"flipV":bool?,"layer":id?,"clear":bool?,"select":bool=true}</code></details> | — |
| `cloneSource.resetTransform` | Reset Transform | — | <code>{"index":0..4?=active}</code> | — |
| `cloneSource.overlay` | Clone Source Overlay | — | <details><summary><code>JSON Schema</code></summary><code>{"show":bool?,"opacity":0..100?,"clipped":bool?,"autoHide":bool?,"invert":bool?,"blend":"normal¦darken¦lighten¦difference"?}</code></details> | — |

## 🔹 Color (1)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `color.profileMismatch` | Embedded Profile Mismatch | — | <code>{"action":"preserve¦convert¦discard¦assignWorking"}</code> | — |

## 🔹 Count (7)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `count.add` | Add Count | — | <code>{"x":px,"y":px,"group":index?}</code> | — |
| `count.remove` | Remove Count | — | <code>{"index":n,"group":index?} ¦ {"x":px,"y":px,"radius":px=6}</code> | — |
| `count.move` | Move Count | — | <code>{"index":n,"group":index?,"to":[x,y]} ¦ {"x":px,"y":px,"to":[x,y]}</code> | — |
| `count.clear` | Clear Count | — | <code>{"group":index¦"all"=active}</code> | — |
| `count.newGroup` | New Count Group | — | <code>{"name":str?,"color":"#rrggbb"¦[r,g,b]?}</code> | — |
| `count.deleteGroup` | Delete Count Group | — | <code>{"group":index=active}</code> | — |
| `count.setGroup` | Count Group Options | — | <details><summary><code>JSON Schema</code></summary><code>{"group":index=active,"name":str?,"color":"#rrggbb"¦[r,g,b]?,"markerSize":1..10?,"labelSize":8..72?,"visible":bool?,"active":bool?}</code></details> | — |

## 🔹 Document & Engine Inspection (11)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `session.inspect` | Inspect Session | — | `{}` | — |
| `document.inspect` | Inspect Document | — | <code>{"document":index?}</code> | — |
| `document.activate` | Activate Document | — | <code>{"document":index}</code> | — |
| `document.move` | Move Document | — | <code>{"document":index?,"to":index}</code> | — |
| `command.list` | List Commands | — | `{}` | — |
| `document.pixel` | Read Composite Pixel | — | <code>{"x":i32,"y":i32}</code> | — |
| `document.sampleColor` | Sample Color | — | <details><summary><code>JSON Schema</code></summary><code>{"x":f64,"y":f64,"size":"point"¦1¦3¦5¦11¦31¦51¦101=1,"sampleLayer":"current"¦"currentAndBelow"¦"all"¦"allNoAdjustments"¦"currentAndBelowNoAdjustments"="all"} → [r,g,b,a] (the Eyedropper's Sample Size and Sample: the composite pixel, or the average of the size×size square centred on it, clipped to the canvas; transparent off the canvas)</code></details> | — |
| `document.presets.list` | List Document Presets | — | `{}` | — |
| `document.presets.get` | Get Document Preset | — | <code>{"name":str} -> {preset,params} (params for file.new; no document is created)</code> | — |
| `document.presets.save` | Save Document Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str (trimmed, ASCII case-insensitive unique, 1..255 characters),"settings":{"width":px,"height":px,"resolution":ppi=72,"mode":"rgb¦gray¦cmyk¦lab"="rgb","depth":8¦16¦32=8,"background":"white¦black¦transparent¦backgroundColor¦#rrggbb"="white","backgroundColor":[r,g,b]?,"unit":"px¦in¦cm¦mm¦pt¦pica"="px","resolutionUnit":"in¦cm"="in"}}</code></details> | — |
| `document.presets.delete` | Delete Document Preset | — | <code>{"name":str}</code> | — |

## 🔹 Edit & Clipboard (77)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `edit.undo` | Undo | Edit | `{}` | `Cmd+Z` |
| `edit.redo` | Redo | Edit | `{}` | `Cmd+Shift+Z` |
| `edit.fill` | Fill… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"contents":"foreground¦background¦color¦contentAware¦pattern¦history¦black¦gray¦white"="color","color":"#rrggbb¦[r,g,b,a]"=foreground (contents=color),"pattern":id¦name (contents=pattern),"scale":%=100,"angle":deg,"state":index? (contents=history; default the oldest state),"colorAdaptation":bool=true (contents=contentAware),"mode":"normal¦multiply¦…"="normal","opacity":0..100=100,"preserveTransparency":bool=false,"target":"pixels"¦{"channel":i}¦"quickMask"?}</code></details> | `Shift+F5` |
| `edit.clear` | Clear | Edit | `{}` | `Delete` |
| `edit.transform` | Free Transform | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"rect":[x0,y0,x1,y1]? (source frame; default = layer content ∩ selection),"quad":[[x,y]×4]? (where the frame's corners go, clockwise from top-left),"matrix":[a,b,c,d,e,f]? (affine alternative),"interpolation":"bicubic¦bilinear¦nearest"="bicubic","target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target (an unlinked mask, an alpha channel or the Quick Mask transforms alone; a linked mask moves with its layer)}</code></details> | — |
| `edit.cut` | Cut | Edit | `{}` | `Cmd+X` |
| `edit.copy` | Copy | Edit | `{}` | `Cmd+C` |
| `edit.copyMerged` | Copy Merged | Edit | `{}` | `Cmd+Shift+C` |
| `edit.paste` | Paste | Edit | <details><summary><code>JSON Schema</code></summary><code>{"center":[x,y]? (view centre; default keeps the position when it overlaps the canvas),"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target (not pixels: the luminosity pasted into that mask or channel)} (with no document open: a new document from the clipboard)</code></details> | `Cmd+V` |
| `edit.pasteSpecial.pasteInPlace` | Paste in Place | Edit › Paste Special | <details><summary><code>JSON Schema</code></summary><code>{"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target (not pixels: the luminosity pasted into that mask or channel)}</code></details> | `Cmd+Shift+V` |
| `edit.toggleLastState` | Toggle Last State | Edit | `{}` | `Cmd+Alt+Z` |
| `edit.transform.again` | Again | Edit › Transform | `{}` | `Cmd+Shift+T` |
| `edit.transform.againCopy` | Transform Again on a Copy | — | <details><summary><code>JSON Schema</code></summary><code>{} (duplicates the active layer and repeats the last transform on the copy: step and repeat)</code></details> | `Cmd+Alt+Shift+T` |
| `edit.assignProfile` | Assign Profile… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"profile":"working¦srgb¦display-p3¦adobe-rgb-compat¦prophoto-compat¦linear-srgb¦rec2020¦gray-gamma-2.2¦sgray¦lab-d50¦coated-cmyk¦none" (or a path to an .icc file)}</code></details> | — |
| `edit.convertToProfile` | Convert to Profile… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"profile":"srgb¦display-p3¦adobe-rgb-compat¦prophoto-compat¦linear-srgb¦rec2020¦gray-gamma-2.2¦sgray¦lab-d50¦coated-cmyk¦working" (or a path to an .icc file),"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true}</code></details> | — |
| `edit.colorSettings` | Color Settings… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"workingRgb":"srgb¦display-p3¦adobe-rgb-compat¦prophoto-compat¦linear-srgb¦rec2020","workingCmyk":"coated-cmyk","workingGray":"sgray¦gray-gamma-2.2","policyRgb":"preserve¦convert¦off","policyCmyk":"preserve¦convert¦off","policyGray":"preserve¦convert¦off","askOnMismatch":bool=true,"askOnPaste":bool=true,"askOnMissing":bool=false,"intent":"relative¦perceptual¦saturation¦absolute","blendTextGamma":1.0..2.2¦bool=1.45,"bpc":bool=true,"dither":bool=true,"monitorProfile":"auto¦srgb¦display-p3¦adobe-rgb-compat¦prophoto-compat¦rec2020","reset":bool=false} (working spaces and the monitor profile also accept .icc paths; monitor 'auto' = the main display's profile when the platform provides it, else sRGB; the reply's 'monitorStatus' says which profile is in use and why: source auto¦manual¦fallback, reason)</code></details> | `Cmd+Shift+K` |
| `edit.profileInfo` | Profile Info | — | <code>{"profile":"<builtin id>¦document¦/path/to/profile.icc"=document}</code> | — |
| `edit.stroke` | Stroke… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"width":1..250=1,"color":"#rrggbb¦[r,g,b,a]"=foreground,"location":"inside¦center¦outside"="center","mode":"any layer blend mode"="normal","opacity":0..100=100,"preserveTransparency":bool=false}</code></details> | — |
| `edit.transform.rotate180` | Rotate 180° | Edit › Transform | `{}` | — |
| `edit.transform.rotate90Cw` | Rotate 90° Clockwise | Edit › Transform | `{}` | — |
| `edit.transform.rotate90Ccw` | Rotate 90° Counter Clockwise | Edit › Transform | `{}` | — |
| `edit.transform.flipHorizontal` | Flip Horizontal | Edit › Transform | `{}` | — |
| `edit.transform.flipVertical` | Flip Vertical | Edit › Transform | `{}` | — |
| `edit.pasteSpecial.pasteInto` | Paste Into | Edit › Paste Special | <details><summary><code>JSON Schema</code></summary><code>{"center":[x,y]?,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target (not pixels: the luminosity pasted into that mask or channel)}</code></details> | `Cmd+Alt+Shift+V` |
| `edit.pasteSpecial.pasteOutside` | Paste Outside | Edit › Paste Special | <details><summary><code>JSON Schema</code></summary><code>{"center":[x,y]?,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target (not pixels: the luminosity pasted into that mask or channel)}</code></details> | — |
| `edit.checkSpelling` | Check Spelling… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"action":"list¦change¦changeAll¦addToDictionary¦removeFromDictionary¦suggest"="list","allLayers":bool=true,"layer":id? (check one layer),"ignore":[word]? (Ignore All for this check),"suggestions":n=5; change: "layer","start","end" (chars),"word"? (verified),"replace"; changeAll: "word","replace"; addToDictionary/removeFromDictionary/suggest: "word"} → list: {"misspellings":[{"layer","start","end","word","suggestions"}],"count"}</code></details> | — |
| `edit.preferences.general` | General… | Edit › Preferences | `{}` | `Cmd+K` |
| `edit.preferences.interface` | Interface… | Edit › Preferences | `{}` | — |
| `edit.preferences.workspace` | Workspace… | Edit › Preferences | `{}` | — |
| `edit.preferences.tools` | Tools… | Edit › Preferences | `{}` | — |
| `edit.preferences.historyLog` | History Log… | Edit › Preferences | `{}` | — |
| `edit.preferences.fileHandling` | File Handling… | Edit › Preferences | `{}` | — |
| `edit.preferences.export` | Export… | Edit › Preferences | `{}` | — |
| `edit.preferences.performance` | Performance… | Edit › Preferences | `{}` | — |
| `edit.preferences.scratchDisks` | Scratch Disks… | Edit › Preferences | `{}` | — |
| `edit.preferences.cursors` | Cursors… | Edit › Preferences | `{}` | — |
| `edit.preferences.transparencyAndGamut` | Transparency & Gamut… | Edit › Preferences | `{}` | — |
| `edit.preferences.unitsAndRulers` | Units & Rulers… | Edit › Preferences | `{}` | — |
| `edit.preferences.guidesGridAndSlices` | Guides, Grid & Slices… | Edit › Preferences | `{}` | — |
| `edit.preferences.plugIns` | Plug-ins… | Edit › Preferences | `{}` | — |
| `edit.preferences.type` | Type… | Edit › Preferences | `{}` | — |
| `edit.preferences.enhancedControls` | Enhanced Controls… | Edit › Preferences | `{}` | — |
| `edit.preferences.rawDefaults` | Camera Raw… | Edit › Preferences | `{}` | — |
| `edit.preferences.integrations` | Integrations… | Edit › Preferences | `{}` | — |
| `edit.keyboardShortcuts` | Keyboard Shortcuts… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"set":{"<command id>¦tools.temporary.hand¦zoomIn¦zoomOut":"Cmd+Shift+X"¦""(remove)¦null(default)}?,"reset":true¦["<id>",…]?,"removeConflicts":bool=true,"filter":str?,"list":bool=false}</code></details> | `Cmd+Alt+Shift+K` |
| `edit.menus` | Menus… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"hide":["<id>",…]?,"show":["<id>",…]?,"color":{"<id>":"red¦orange¦yellow¦green¦blue¦violet¦gray¦none"}?,"reset":bool=false}</code></details> | `Cmd+Alt+Shift+M` |
| `edit.toolbar` | Toolbar… | Edit | <code>{"hidden":["<tool>",…]?,"order":["<tool>",…]?,"reset":bool=false}</code> | — |
| `edit.fade` | Fade… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"opacity":0..100=100,"mode":"normal¦multiply¦screen¦overlay¦softLight¦hardLight¦darken¦lighten¦difference¦color¦luminosity"}</code></details> | `Cmd+Shift+F` |
| `edit.purge.undo` | Undo | Edit › Purge | `{}` | — |
| `edit.purge.clipboard` | Clipboard | Edit › Purge | `{}` | — |
| `edit.purge.histories` | Histories | Edit › Purge | `{}` | — |
| `edit.purge.videoCache` | Video Cache | Edit › Purge | `{}` | — |
| `edit.purge.all` | All | Edit › Purge | `{}` | — |
| `edit.contentAwareFill` | Content-Aware Fill… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"sampling":"auto¦rectangular¦custom","margin":px?,"area":[x,y,w,h]?,"channel":index¦name?,"colorAdaptation":"default¦none¦high¦veryHigh","rotationAdaptation":"none¦low¦medium¦high¦full","scale":bool=false,"mirror":bool=false,"output":"current¦new¦duplicate","seed":u64=1}</code></details> | — |
| `edit.deleteAndFillSelection` | Delete and Fill Selection | — | `{}` | — |
| `edit.contentAwareScale` | Content-Aware Scale | Edit | <details><summary><code>JSON Schema</code></summary><code>{"width":px?,"height":px?,"scaleX":1..400=100,"scaleY":1..400=100,"amount":0..100=100,"protect":"none"¦channel index¦name,"protectSkinTones":bool=false}</code></details> | `Cmd+Alt+Shift+C` |
| `edit.defineBrushPreset` | Define Brush Preset… | Edit | <code>{"name":text}</code> | — |
| `edit.defineCustomShape` | Define Custom Shape… | Edit | <code>{"name":text,"path":"work¦<saved path name>"?}</code> | — |
| `edit.findAndReplaceText` | Find and Replace Text… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"find":text,"replace":text,"action":"changeAll¦find¦change¦changeFind","caseSensitive":bool=false,"wholeWord":bool=false,"forward":bool=true,"allLayers":bool=true}</code></details> | — |
| `edit.presets.presetManager` | Preset Manager… | Edit › Presets | <details><summary><code>JSON Schema</code></summary><code>{"action":"list¦rename¦delete¦move","kind":"brushes¦customShapes¦patterns","index":n?,"name":str?,"newName":str?,"to":n?}</code></details> | — |
| `edit.presets.exportImportPresets` | Export/Import Presets… | Edit › Presets | <details><summary><code>JSON Schema</code></summary><code>{"action":"export¦import","kinds":["brushes","customShapes"]?,"data":json (import),"includeBuiltins":bool=false}</code></details> | — |
| `edit.fillForeground` | Fill with Foreground Color | — | <code>{"layer":id?,"target":"pixels"¦{"channel":i}¦"quickMask"?}</code> | `Alt+Backspace` |
| `edit.fillBackground` | Fill with Background Color | — | <code>{"layer":id?,"target":"pixels"¦{"channel":i}¦"quickMask"?}</code> | `Cmd+Backspace` |
| `edit.fillForegroundPreserve` | Fill with Foreground Color, Preserve Transparency | — | <code>{"layer":id?,"target":"pixels"¦{"channel":i}¦"quickMask"?}</code> | `Alt+Shift+Backspace` |
| `edit.fillBackgroundPreserve` | Fill with Background Color, Preserve Transparency | — | <code>{"layer":id?,"target":"pixels"¦{"channel":i}¦"quickMask"?}</code> | `Cmd+Shift+Backspace` |
| `edit.autoAlignLayers` | Auto-Align Layers… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"projection":"auto¦perspective¦cylindrical¦spherical¦collage¦reposition","reference":layer id?=bottom selected layer,"geometricCorrection":bool=false,"interpolation":"bicubic¦bilinear¦nearest"}</code></details> | — |
| `edit.autoBlendLayers` | Auto-Blend Layers… | Edit | <code>{"method":"panorama¦stack","seamlessTones":bool=true}</code> | — |
| `edit.definePattern` | Define Pattern… | Edit | <details><summary><code>JSON Schema</code></summary><code>{"name":text,"rect":[x0,y0,x1,y1]?} → {"pattern":id} (name: default the document's name numbered past the library's names; rect: default the selection bounds or else the canvas; added to the library from the visible composite)</code></details> | — |
| `edit.transform.warp` | Warp | Edit › Transform | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"rect":[x0,y0,x1,y1]? (warp box; default = layer content ∩ selection),"style":"custom¦none¦arc¦arcLower¦arcUpper¦arch¦bulge¦shellLower¦shellUpper¦flag¦wave¦fish¦rise¦fisheye¦inflate¦squeeze¦twist","bend":%=50,"hDistort":%,"vDistort":%,"vertical":bool,"mesh":{"us":[0,…,1],"vs":[0,…,1],"points":[[x,y]…]} ((3c+1)×(3r+1) control points, row-major, document px),"grid":[cols,rows]?,"warp":{full warp object}?,"interpolation":"bicubic¦bilinear¦nearest"}</code></details> | — |
| `edit.transform.splitWarpCrosswise` | Split Warp Crosswise | Edit › Transform | <details><summary><code>JSON Schema</code></summary><code>{"warp":{…}? (the warp being edited; returned split),"rect":[x0,y0,x1,y1]?,"at":[x,y]? (document point; default = middle of the patch)} — without 'warp', edits the active smart object's warp. A 1×1 custom mesh is divided at the thirds first (the guides drawn on the default grid), so splitWarpCrosswise on an identity custom warp returns a 4×4 mesh (knots at 0, 1/3, 1/2, 2/3 and 1), not a 2×2. A preset is fitted as one patch and split without that step.</code></details> | — |
| `edit.transform.splitWarpHorizontally` | Split Warp Horizontally | Edit › Transform | <details><summary><code>JSON Schema</code></summary><code>{"warp":{…}? (the warp being edited; returned split),"rect":[x0,y0,x1,y1]?,"at":[x,y]? (document point; default = middle of the patch)} — without 'warp', edits the active smart object's warp. A 1×1 custom mesh is divided at the thirds first (the guides drawn on the default grid), so splitWarpCrosswise on an identity custom warp returns a 4×4 mesh (knots at 0, 1/3, 1/2, 2/3 and 1), not a 2×2. A preset is fitted as one patch and split without that step.</code></details> | — |
| `edit.transform.splitWarpVertically` | Split Warp Vertically | Edit › Transform | <details><summary><code>JSON Schema</code></summary><code>{"warp":{…}? (the warp being edited; returned split),"rect":[x0,y0,x1,y1]?,"at":[x,y]? (document point; default = middle of the patch)} — without 'warp', edits the active smart object's warp. A 1×1 custom mesh is divided at the thirds first (the guides drawn on the default grid), so splitWarpCrosswise on an identity custom warp returns a 4×4 mesh (knots at 0, 1/3, 1/2, 2/3 and 1), not a 2×2. A preset is fitted as one patch and split without that step.</code></details> | — |
| `edit.transform.removeWarpSplit` | Remove Warp Split | Edit › Transform | <details><summary><code>JSON Schema</code></summary><code>{"warp":{…}? (the warp being edited; returned split),"rect":[x0,y0,x1,y1]?,"at":[x,y]? (document point; default = middle of the patch)} — without 'warp', edits the active smart object's warp. A 1×1 custom mesh is divided at the thirds first (the guides drawn on the default grid), so splitWarpCrosswise on an identity custom warp returns a 4×4 mesh (knots at 0, 1/3, 1/2, 2/3 and 1), not a 2×2. A preset is fitted as one patch and split without that step.</code></details> | — |
| `edit.transform.warpGrid` | Warp Grid | — | <details><summary><code>JSON Schema</code></summary><code>{"warp":{…}? (the warp being edited; returned with the new mesh),"rect":[x0,y0,x1,y1]?,"size":n¦"default"¦"3"¦"4"¦"5" (square patches, 1..=64; 1 and "default" are the single-patch grid)} — without 'warp'/'style'/'mesh', edits the active smart object's warp. The surface is kept and the new lines are real splits at i/n.</code></details> | — |
| `edit.puppetWarp` | Puppet Warp | Edit | <details><summary><code>JSON Schema</code></summary><code>{"pins":[{"src":[x,y],"dst":[x,y],"rotate":deg?,"depth":n?}…],"mode":"rigid¦normal¦distort"="normal","density":"fewer¦normal¦more"="normal","expansion":px=2,"interpolation":"bicubic¦bilinear¦nearest","layer":id?} — mesh over the opaque region, as-rigid-as-possible; on a smart object it becomes a smart filter</code></details> | — |
| `edit.perspectiveWarp` | Perspective Warp | Edit | <details><summary><code>JSON Schema</code></summary><code>{"planes":[{"src":[[x,y]×4],"dst":[[x,y]×4]}…] (corners clockwise from top-left; corners that coincide in src are linked),"straighten":"horizontal¦vertical¦auto"?,"interpolation":"bicubic¦bilinear¦nearest","layer":id?}</code></details> | — |
| `edit.presets.migratePresets` | Migrate Presets | Edit › Presets | <details><summary><code>JSON Schema</code></summary><code>{path} → {migrated, added:{gradients,styles,shapes,patternGroups,toolPresets}}: merge a presets/preferences file into the library (appends groups by name)</code></details> | — |

## 🔹 File Operations (49)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `file.new` | New… | File | <details><summary><code>JSON Schema</code></summary><code>{"width":1..300000=1920,"height":1..300000=1080 (pixel counts are rounded),"mode":"rgb¦gray¦grayscale¦cmyk¦lab"="rgb","depth":8¦16¦32=8,"background":"white¦black¦backgroundColor¦transparent¦#rrggbb"="white","backgroundColor":[r,g,b]? (0..1; defaults to toolbox colour),"resolution":1..30000=72 (ppi),"name":str}</code></details> | `Cmd+N` |
| `file.close` | Close | File | <code>{"document":index?}</code> | `Cmd+W` |
| `file.newFromClipboard` | New from Clipboard | File | <code>{} (a new document the size of the clipboard image, holding it as one layer)</code> | — |
| `file.scripts.deleteAllEmptyLayers` | Delete All Empty Layers | File › Scripts | `{}` | — |
| `file.closeAll` | Close All | File | `{}` | `Cmd+Alt+W` |
| `file.closeOthers` | Close Others | File | <code>{"document":index? (the one to keep; default active)}</code> | `Cmd+Alt+P` |
| `file.revealInFinder` | Reveal in Finder | — | <code>{"document":index? (default active),"dryRun":bool=false}</code> | — |
| `file.revert` | Revert | File | <code>{} (reloads the saved file as one undoable step)</code> | `F12` |
| `file.saveACopy` | Save a Copy… | File | <details><summary><code>JSON Schema</code></summary><code>{"path":str (format from the extension),"quality":0..12? (JPEG),"layers":bool=true,"tiffLayers":bool=false (TIFF: keep the layers; flat by default)}</code></details> | `Cmd+Alt+S` |
| `file.openAs` | Open As… | File | <code>{"path":str,"as":"psd¦png¦jpg¦tiff¦…"? (decode as this format)}</code> | `Cmd+Alt+Shift+O` |
| `file.placeEmbedded` | Place Embedded… | File | <details><summary><code>JSON Schema</code></summary><code>{"path":str,"scale":%? (default: fit when larger than the canvas),"fit":bool=true,"center":[x,y]?}</code></details> | — |
| `file.placeLinked` | Place Linked… | File | <code>{"path":str,"scale":%?,"fit":bool=true,"center":[x,y]?}</code> | — |
| `file.fileInfo` | File Info… | File | <details><summary><code>JSON Schema</code></summary><code>{"title":str?,"author":str?,"authorTitle":str?,"description":str?,"keywords":[str]¦"a; b"?,"copyright":str?,"copyrightStatus":"unknown¦copyrighted¦publicDomain"?,"copyrightUrl":str?} (no keys: read)</code></details> | `Cmd+Alt+Shift+I` |
| `file.automate.fitImage` | Fit Image… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"width":px,"height":px,"dontEnlarge":bool=false,"resample":"bicubic¦bilinear¦nearest¦lanczos¦preserveDetails"="bicubic"}</code></details> | — |
| `file.automate.conditionalModeChange` | Conditional Mode Change… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"from":["rgb","grayscale","cmyk","lab","indexed","bitmap",…]¦"any"="any","to":"rgb¦grayscale¦cmyk¦lab"}</code></details> | — |
| `file.automate.batch` | Batch… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"steps":[[commandId,params]¦{"command":id,"params":{}}…] (a recorded action),"input":folder¦[paths],"output":folder,"format":"same¦png¦jpg¦psd¦tiff¦…"="same","quality":0..12?,"tiffLayers":bool=false} → {files, errors} (an input whose output name was already written in the run goes to errors)</code></details> | — |
| `file.scripts.imageProcessor` | Image Processor… | File › Scripts | <details><summary><code>JSON Schema</code></summary><code>{"input":folder¦[paths],"output":folder,"format":"jpg¦png¦psd¦tiff¦…"="jpg","quality":0..12=8,"tiffLayers":bool=false,"width":px?,"height":px? (fit, never enlarge),"convertToSrgb":bool=false} → {files, errors} (an input whose output name was already written in the run goes to errors)</code></details> | — |
| `file.scripts.loadFilesIntoStack` | Load Files into Stack… | File › Scripts | <details><summary><code>JSON Schema</code></summary><code>{"paths":[str]¦folder,"createSmartObject":bool=false} → new document with one layer per file (inside one smart object with createSmartObject)</code></details> | — |
| `file.scripts.flattenAllLayerEffects` | Flatten All Layer Effects | File › Scripts | `{}` | — |
| `file.scripts.flattenAllMasks` | Flatten All Masks | File › Scripts | <code>{} (pixel layers; masks on other layer kinds are left)</code> | — |
| `file.export.layersToFiles` | Layers to Files… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"dir":folder,"format":"png¦jpg¦psd¦tiff¦…"="png","prefix":str=document name,"visibleOnly":bool=true,"quality":0..12?,"tiffLayers":bool=false} → {files}</code></details> | — |
| `file.export.colorLookupTables` | Color Lookup Tables… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"path":str? (.cube; omit to return the text),"size":2..129=33,"title":str?,"scope":"all¦selected"="all","layers":[id,…]?} (selected layer IDs use original document stacking; masks are spatial and omitted)</code></details> | — |
| `file.automate.photomerge` | Photomerge… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"paths":[str]¦folder,"useOpenDocuments":bool=false,"layout":"auto¦perspective¦cylindrical¦spherical¦collage¦reposition","blend":bool=true,"vignetteRemoval":bool=false,"geometricCorrection":bool=false,"contentAwareFill":bool=false,"focalLength":mm=0 (35 mm equivalent; 0 = EXIF or estimated)} → new document, one masked layer per image</code></details> | — |
| `file.automate.mergeToHdrPro` | Merge to HDR Pro… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"paths":[str]¦folder,"useOpenDocuments":bool=false,"exposures":[ev]? (default EXIF, else estimated),"align":bool=true,"removeGhosts":bool=false,"ghostBase":int?,"mode":"32¦16¦8","method":"localAdaptation¦exposureGamma¦highlightCompression¦equalizeHistogram","radius":1..500=7,"strength":0.1..4=0.52,"gamma":0.1..2=1,"exposure":-5..5=0,"detail":-100..300=30,"shadow":-100..100=0,"highlight":-100..100=0,"vibrance":-100..100=0,"saturation":-100..100=20} → new document (32-bit linear, or tone-mapped 16/8-bit)</code></details> | — |
| `file.automate.cropAndStraightenPhotos` | Crop and Straighten Photos | File › Automate | <code>{} → one new document per photo found on the scan</code> | — |
| `file.automate.lensCorrection` | Lens Correction… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"input":folder¦[files],"output":folder,"format":"same¦jpg¦png¦tif¦psd","quality":0..12?, …Filter › Lens Correction params ("profile" defaults to "auto")}</code></details> | — |
| `file.export.layerCompsToFiles` | Layer Comps to Files… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"dir":folder,"format":"png¦jpg¦psd¦tiff¦…"="png","prefix":str=document name,"selectedOnly":bool=false (only the last applied comp),"comps":[id]?,"quality":0..12?} → {files}</code></details> | — |
| `file.export.artboardsToFiles` | Artboards to Files… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"dir":folder,"format":"png¦jpg¦psd¦tiff¦…"="png","prefix":str=document name ("" = none),"artboards":[id]? (default all),"quality":0..12?} → {files}</code></details> | — |
| `file.export.artboardsToPdf` | Artboards to PDF… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"path":str (.pdf),"artboards":[id]?,"quality":0..12=10} → {path, pages} (one raster page per board)</code></details> | — |
| `file.import.notes` | Notes… | File › Import | <code>{"path":".psd¦.psb¦.pcraft with notes"}</code> | — |
| `file.export.saveForWebLegacy` | Save for Web (Legacy)… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"preset":"GIF 128 Dithered¦JPEG High¦PNG-24¦…"?,"format":"gif¦png8¦png24¦jpeg¦wbmp"="png24","palette":"perceptual¦selective¦adaptive¦restrictive¦exact¦systemMac¦systemWindows¦uniform"="selective","colors":2..256=256,"dither":"none¦diffusion¦pattern¦noise"="diffusion","ditherAmount":0..100=88,"transparency":bool=true,"matte":"#rrggbb¦none"="#ffffff","interlaced":bool=false,"webSnap":0..100=0,"quality":0..100=60,"progressive":bool=false,"optimized":bool=true,"embedIcc":bool=false,"metadata":"none¦copyright¦copyrightAndContact¦all"="copyright","convertToSrgb":bool=true,"width"¦"height"¦"percent"? (image size),"resample":"bicubic¦bilinear¦nearest"?,"path":file? (whole image),"dir":folder? (one file per slice in images/),"html":bool=false,"slices":"all¦user"="all","numbers":[n]?} → no path/dir: {bytes,width,height,colors} estimate; else {files, html, bytes}</code></details> | `Cmd+Alt+Shift+S` |
| `file.export.exportPreferences` | Export Preferences… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"quickExportFormat":"png¦jpg¦gif¦webp"?,"quickExportLocation":"ask¦sameFolder"?,"jpegQuality":1..100?,"webpLossless":bool?,"webpQuality":1..100?,"metadata":"none¦copyright¦all"?,"convertToSrgb":bool?} → {values}</code></details> | — |
| `file.export.quickExport` | Quick Export | — | <details><summary><code>JSON Schema</code></summary><code>{"path":str? (required unless Export Preferences › Location is "sameFolder" and the document is saved)} → {path, format, bytes}</code></details> | — |
| `file.generate.imageAssets` | Image Assets | File › Generate | <details><summary><code>JSON Schema</code></summary><code>{"on":bool? (default: toggle),"dir":folder? (default <document>-assets next to the file)} → {enabled, files, errors}; layers named like "foo.png", "200% foo@2x.png", "48x48 icons/a.png8", "photo.jpg80%" are exported, now and after each save</code></details> | — |
| `file.automate.contactSheetII` | Contact Sheet II… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"input":folder¦[paths],"units":"inches¦cm¦mm¦pixels"="inches","width":8,"height":10,"resolution":ppi=300,"mode":"rgb¦gray¦cmyk¦lab"="rgb","depth":8¦16=8,"columns":5,"rows":6,"placeAcrossFirst":bool=true,"autoSpacing":bool=true,"horizontal":units?,"vertical":units?,"rotateForBestFit":bool=false,"caption":bool=true (file name as caption),"font":family?,"fontSize":pt=12,"flatten":bool=false} → {documents, pages, images}</code></details> | — |
| `file.automate.createDroplet` | Create Droplet… | File › Automate | <details><summary><code>JSON Schema</code></summary><code>{"path":str (.pcdroplet),"steps":[[id,params]…] (the action),"name":str?,"output":folder?,"format":"same¦png¦jpg¦…"?,"quality":0..12?,"shim":bool=true on Unix (writes <name>.command calling 'photocraft-cli droplet')} → {path, shim}</code></details> | — |
| `file.automate.runDroplet` | Run Droplet | — | <details><summary><code>JSON Schema</code></summary><code>{"droplet":path,"input":[files or folders],"output":folder? (default: droplet's, else <input folder>/droplet-output)} → {files, errors} (an input whose output name was already written in the run goes to errors)</code></details> | — |
| `file.scripts.statistics` | Statistics… | File › Scripts | <details><summary><code>JSON Schema</code></summary><code>{"mode":"mean¦median¦maximum¦minimum¦range¦summation¦variance¦standardDeviation¦skewness¦kurtosis¦entropy"="median","input":folder¦[paths],"align":bool=false (Auto-Align first)} → new document with one stack-mode smart object</code></details> | — |
| `file.scripts.browse` | Browse… | File › Scripts | <details><summary><code>JSON Schema</code></summary><code>{"path":script file ¦ "script":text ¦ "steps":[[id,params]…]} (JSON action, or one 'command.id {json}' per line) → {steps, ok, results}</code></details> | — |
| `file.scripts.scriptEventsManager` | Script Events Manager… | File › Scripts | <details><summary><code>JSON Schema</code></summary><code>{"enabled":bool?,"add":{"event":"startApplication¦newDocument¦openDocument¦saveDocument¦closeDocument¦print¦export¦everything","script":path?¦"steps":[…]?,"name":str?}?,"remove":index?,"removeAll":bool?} → {enabled, bindings, events}</code></details> | — |
| `file.print` | Print… | File | <details><summary><code>JSON Schema</code></summary><code>{"printer":name? (default printer),"copies":1..999=1,"paper":"letter¦legal¦tabloid¦a3¦a4¦a5¦4x6¦5x7"¦[w,h] pt="letter","orientation":"portrait¦landscape"="portrait","center":bool=true,"top":in?,"left":in?,"scale":%=100,"scaleToFit":bool=false,"colorHandling":"printerManages¦photocraftManages¦noColorManagement"="printerManages","printerProfile":profile? (photocraftManages),"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true,"cornerCropMarks":bool,"centerCropMarks":bool,"registrationMarks":bool,"description":bool,"labels":bool,"output":pdf path? (print to PDF; then "send" defaults to false),"send":bool?,"dryRun":bool=false (render the PDF, report the lp command, don't spool)} → {pdf, imageRect, command, sent}</code></details> | `Cmd+P` |
| `file.printOneCopy` | Print One Copy | File | <code>{} (the last Print settings, one copy; any Print key overrides)</code> | `Cmd+Alt+Shift+P` |
| `file.package` | Package… | File | <details><summary><code>JSON Schema</code></summary><code>{"dir":folder,"format":"pcraft¦psd¦psb"="pcraft"} → {folder, document, links, missing} (copies the document and its linked files into <dir>/<name>/, relinked to Links/)</code></details> | — |
| `file.export.pathsToIllustrator` | Paths to Illustrator… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{"path":str? (.ai; omit to return the text),"paths":"all¦work¦<path name>"="all"} → {path, paths}</code></details> | — |
| `file.import.videoFramesToLayers` | Video Frames to Layers… | File › Import | <code>{} → {}</code> | — |
| `file.export.renderVideo` | Render Video… | File › Export | <code>{} → {}</code> | — |
| `file.import.wiaSupport` | WIA Support… | File › Import | <code>{} → {available, devices, note}: acquire from a scanner/camera (Windows only)</code> | — |
| `file.import.variableDataSets` | Variable Data Sets… | File › Import | <details><summary><code>JSON Schema</code></summary><code>{path, delimiter?} → {imported}: CSV header = variable names, each row a data set (first column may be the data-set name)</code></details> | — |
| `file.export.dataSetsAsFiles` | Data Sets as Files… | File › Export | <details><summary><code>JSON Schema</code></summary><code>{dir, format?:png, dataSets?[names], naming?:"{name}¦{index}¦{document}"} → {files,count}: apply each data set and export the flattened document</code></details> | — |

## 🔹 Filter (17)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.other.highPass` | High Pass… | Filter › Other | <code>{"radius":0.1..1000=10}</code> | — |
| `filter.other.minimum` | Minimum… | Filter › Other | <code>{"radius":0.2..500=1,"preserve":"squareness¦roundness"}</code> | — |
| `filter.other.maximum` | Maximum… | Filter › Other | <code>{"radius":0.2..500=1,"preserve":"squareness¦roundness"}</code> | — |
| `filter.other.offset` | Offset… | Filter › Other | <code>{"horizontal":px=0,"vertical":px=0,"undefinedAreas":"wrap¦repeat¦transparent"}</code> | — |
| `filter.lastFilter` | Last Filter | Filter | `{}` | `Cmd+Alt+F` |
| `filter.other.custom` | Custom… | Filter › Other | <code>{"kernel":int[25],"scale":1..9999=1,"offset":-9999..9999=0}</code> | — |
| `filter.other.hsbHsl` | HSB/HSL | Filter › Other | <code>{"inputMode":"rgb¦hsb¦hsl","rowOrder":"hsb¦hsl¦rgb"}</code> | — |
| `filter.video.deInterlace` | De-Interlace… | Filter › Video | <code>{"eliminate":"oddFields¦evenFields","createBy":"interpolation¦duplication"}</code> | — |
| `filter.video.ntscColors` | NTSC Colors | Filter › Video | `{}` | — |
| `filter.other.colorToAlpha` | Color to Alpha… | Filter › Other | <code>{"color":color=#ffffff,"transparencyThreshold":0..1=0,"opacityThreshold":0..1=1}</code> | — |
| `filter.filterGallery` | Filter Gallery… | Filter | <code>{"effects":json,"foreground":json,"background":json,"list":bool}</code> | — |
| `filter.convertForSmartFilters` | Convert for Smart Filters | Filter | <code>{"layer":id?} (default: combine selected layers)</code> | — |
| `filter.lensCorrection` | Lens Correction… | Filter | <details><summary><code>JSON Schema</code></summary><code>{"profile":"none¦auto¦generic","focalLength":mm=0,"correctDistortion":bool=true,"correctVignette":bool=true,"correctCA":bool=true,"autoScale":bool=false,"distortion":-100..100=0,"redCyan":-100..100=0,"blueYellow":-100..100=0,"vignetteAmount":-100..100=0,"vignetteMidpoint":0..100=50,"vertical":-100..100=0,"horizontal":-100..100=0,"angle":-180..180=0,"scale":50..150=100,"edge":"transparency¦edgeExtension¦black¦white","straighten":[[x,y],[x,y]]?}</code></details> | `Cmd+Shift+R` |
| `filter.adaptiveWideAngle` | Adaptive Wide Angle… | Filter | <details><summary><code>JSON Schema</code></summary><code>{"model":"auto¦fisheye¦perspective¦fullSpherical","focalLength":0..200=0,"cropFactor":0.1..10=1,"scale":50..150=100,"constraints":[{"a":[x,y],"b":[x,y],"orientation":"free¦horizontal¦vertical"}]}</code></details> | `Cmd+Alt+Shift+A` |
| `filter.cameraRaw` | Camera Raw Filter… | Filter | <details><summary><code>JSON Schema</code></summary><code>{"temperature":-100..100=0,"tint":-100..100=0,"exposure":-5..5=0,"contrast":-100..100=0,"highlights":-100..100=0,"shadows":-100..100=0,"whites":-100..100=0,"blacks":-100..100=0,"texture":-100..100=0,"clarity":-100..100=0,"dehaze":-100..100=0,"vibrance":-100..100=0,"saturation":-100..100=0,"curveHighlights":-100..100=0,"curveLights":-100..100=0,"curveDarks":-100..100=0,"curveShadows":-100..100=0,"curveSplits":[25,50,75],"pointCurve":[[in,out]],"pointCurveRed":[[in,out]],"pointCurveGreen":[[in,out]],"pointCurveBlue":[[in,out]],"hslHue":[8],"hslSat":[8],"hslLum":[8],"gradeShadows":{"hue":deg,"sat":0..100,"lum":-100..100},"gradeMidtones":{},"gradeHighlights":{},"gradeGlobal":{},"gradeBlending":0..100=50,"gradeBalance":-100..100=0,"sharpenAmount":0..150=0,"sharpenRadius":0.5..3=1,"sharpenDetail":0..100=25,"sharpenMasking":0..100=0,"noiseLuminance":0..100=0,"noiseLuminanceDetail":0..100=50,"noiseColor":0..100=0,"noiseColorDetail":0..100=50,"grainAmount":0..100=0,"grainSize":0..100=25,"grainRoughness":0..100=50,"vignetteAmount":-100..100=0,"vignetteMidpoint":0..100=50,"vignetteRoundness":-100..100=0,"vignetteFeather":0..100=50,"vignetteHighlights":0..100=0,"vignetteStyle":"highlightPriority¦colorPriority¦paintOverlay","seed":u32=0}</code></details> | `Cmd+Shift+A` |
| `filter.vanishingPoint` | Vanishing Point… | Filter | <details><summary><code>JSON Schema</code></summary><code>{"planes":[{"corners":[[x,y]×4]} ¦ {"from":plane,"edge":"top¦right¦bottom¦left","angle":deg=90,"depth":1}],"focalLength":px=0,"paste":[{"plane":0,"layer":id? (else the clipboard),"at":[u,v]=[0.25,0.25],"width":0..1=0.5}],"clone":[{"source":[x,y],"points":[[x,y]],"size":px=40,"hardness":0..100=50,"opacity":0..100=100}],"newLayer":bool=false}</code></details> | `Cmd+Alt+V` |
| `filter.liquify` | Liquify… | Filter | <details><summary><code>JSON Schema</code></summary><code>{"strokes":[{"tool":"forwardWarp¦reconstruct¦smooth¦twirlCw¦twirlCcw¦pucker¦bloat¦pushLeft¦freeze¦thaw¦lassoMask¦reconstructAll","size":px=100,"density":0-100=50,"pressure":0-100=100,"rate":0-100=80,"points":[[x,y,pressure?]…],"amount":%? (reconstructAll; lassoMask: 1 freezes the polygon in points, 0 thaws it)}…],"meshSize":px? (field resolution, px per node; default 2, or 4 above 4 MP),"layer":id?} — strokes replay in order on a fresh field; a selection limits the effect; on a smart object it becomes a smart filter</code></details> | `Cmd+Shift+X` |

## 🔹 Filter Gallery (47)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.gallery.coloredPencil` | Colored Pencil | — | <code>{"pencilWidth":1..24=4,"strokePressure":0..15=8,"paperBrightness":0..50=25}</code> | — |
| `filter.gallery.cutout` | Cutout | — | <code>{"numberOfLevels":2..8=4,"edgeSimplicity":0..10=4,"edgeFidelity":1..3=2}</code> | — |
| `filter.gallery.dryBrush` | Dry Brush | — | <code>{"brushSize":0..10=2,"brushDetail":0..10=8,"texture":1..3=1}</code> | — |
| `filter.gallery.filmGrain` | Film Grain | — | <code>{"grain":0..20=4,"highlightArea":0..20=0,"intensity":0..10=10}</code> | — |
| `filter.gallery.fresco` | Fresco | — | <code>{"brushSize":0..10=2,"brushDetail":0..10=8,"texture":1..3=1}</code> | — |
| `filter.gallery.neonGlow` | Neon Glow | — | <code>{"glowSize":-24..24=5,"glowBrightness":0..50=15,"glowColor":json}</code> | — |
| `filter.gallery.paintDaubs` | Paint Daubs | — | <details><summary><code>JSON Schema</code></summary><code>{"brushSize":1..50=8,"sharpness":0..40=7,"brushType":"simple¦lightRough¦darkRough¦wideSharp¦wideBlurry¦sparkle"}</code></details> | — |
| `filter.gallery.paletteKnife` | Palette Knife | — | <code>{"strokeSize":1..50=25,"strokeDetail":1..3=3,"softness":0..10=0}</code> | — |
| `filter.gallery.plasticWrap` | Plastic Wrap | — | <code>{"highlightStrength":0..20=15,"detail":1..15=9,"smoothness":1..15=7}</code> | — |
| `filter.gallery.posterEdges` | Poster Edges | — | <code>{"edgeThickness":0..10=2,"edgeIntensity":0..10=1,"posterization":0..6=2}</code> | — |
| `filter.gallery.roughPastels` | Rough Pastels | — | <details><summary><code>JSON Schema</code></summary><code>{"strokeLength":0..40=6,"strokeDetail":1..20=4,"texture":"canvas¦brick¦burlap¦sandstone","scaling":50..200=100,"relief":0..50=20,"light":"bottom¦bottomLeft¦left¦topLeft¦top¦topRight¦right¦bottomRight","invert":bool}</code></details> | — |
| `filter.gallery.smudgeStick` | Smudge Stick | — | <code>{"strokeLength":0..10=2,"highlightArea":0..20=0,"intensity":0..10=10}</code> | — |
| `filter.gallery.sponge` | Sponge | — | <code>{"brushSize":0..10=2,"definition":0..25=12,"smoothness":1..15=5}</code> | — |
| `filter.gallery.underpainting` | Underpainting | — | <details><summary><code>JSON Schema</code></summary><code>{"brushSize":0..40=6,"textureCoverage":0..40=16,"texture":"canvas¦brick¦burlap¦sandstone","scaling":50..200=100,"relief":0..50=4,"light":"top¦topRight¦right¦bottomRight¦bottom¦bottomLeft¦left¦topLeft","invert":bool}</code></details> | — |
| `filter.gallery.watercolor` | Watercolor | — | <code>{"brushDetail":1..14=9,"shadowIntensity":0..10=1,"texture":1..3=1}</code> | — |
| `filter.gallery.accentedEdges` | Accented Edges | — | <code>{"edgeWidth":1..14=2,"edgeBrightness":0..50=38,"smoothness":1..15=5}</code> | — |
| `filter.gallery.angledStrokes` | Angled Strokes | — | <code>{"directionBalance":0..100=50,"strokeLength":3..50=15,"sharpness":0..10=3}</code> | — |
| `filter.gallery.crosshatch` | Crosshatch | — | <code>{"strokeLength":3..50=9,"sharpness":0..20=6,"strength":1..3=1}</code> | — |
| `filter.gallery.darkStrokes` | Dark Strokes | — | <code>{"balance":0..10=5,"blackIntensity":0..10=6,"whiteIntensity":0..10=2}</code> | — |
| `filter.gallery.inkOutlines` | Ink Outlines | — | <code>{"strokeLength":1..50=4,"darkIntensity":0..50=20,"lightIntensity":0..50=10}</code> | — |
| `filter.gallery.spatter` | Spatter | — | <code>{"sprayRadius":0..25=10,"smoothness":1..15=5}</code> | — |
| `filter.gallery.sprayedStrokes` | Sprayed Strokes | — | <details><summary><code>JSON Schema</code></summary><code>{"strokeLength":0..20=12,"sprayRadius":0..25=7,"strokeDirection":"rightDiagonal¦horizontal¦leftDiagonal¦vertical"}</code></details> | — |
| `filter.gallery.sumiE` | Sumi-e | — | <code>{"strokeWidth":3..15=10,"strokePressure":0..15=2,"contrast":0..40=16}</code> | — |
| `filter.gallery.diffuseGlow` | Diffuse Glow | — | <details><summary><code>JSON Schema</code></summary><code>{"graininess":0..10=6,"glowAmount":0..20=10,"clearAmount":0..20=15,"background":json}</code></details> | — |
| `filter.gallery.glass` | Glass | — | <details><summary><code>JSON Schema</code></summary><code>{"distortion":0..20=5,"smoothness":1..15=3,"texture":"frosted¦blocks¦canvas¦tinyLens","scaling":50..200=100,"invert":bool}</code></details> | — |
| `filter.gallery.oceanRipple` | Ocean Ripple | — | <code>{"rippleSize":1..15=9,"rippleMagnitude":0..20=9}</code> | — |
| `filter.gallery.basRelief` | Bas Relief | — | <details><summary><code>JSON Schema</code></summary><code>{"detail":1..15=13,"smoothness":1..15=3,"light":"bottom¦bottomLeft¦left¦topLeft¦top¦topRight¦right¦bottomRight","foreground":json,"background":json}</code></details> | — |
| `filter.gallery.chalkCharcoal` | Chalk & Charcoal | — | <details><summary><code>JSON Schema</code></summary><code>{"charcoalArea":0..20=6,"chalkArea":0..20=6,"strokePressure":0..5=1,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.charcoal` | Charcoal | — | <details><summary><code>JSON Schema</code></summary><code>{"charcoalThickness":1..7=1,"detail":0..5=5,"lightDarkBalance":0..100=50,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.chrome` | Chrome | — | <code>{"detail":0..10=4,"smoothness":0..10=7}</code> | — |
| `filter.gallery.conteCrayon` | Conté Crayon | — | <details><summary><code>JSON Schema</code></summary><code>{"foregroundLevel":1..15=11,"backgroundLevel":1..15=7,"texture":"canvas¦brick¦burlap¦sandstone","scaling":50..200=100,"relief":0..50=4,"light":"top¦topRight¦right¦bottomRight¦bottom¦bottomLeft¦left¦topLeft","invert":bool,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.graphicPen` | Graphic Pen | — | <details><summary><code>JSON Schema</code></summary><code>{"strokeLength":1..15=15,"lightDarkBalance":0..100=50,"strokeDirection":"rightDiagonal¦horizontal¦leftDiagonal¦vertical","foreground":json,"background":json}</code></details> | — |
| `filter.gallery.halftonePattern` | Halftone Pattern | — | <details><summary><code>JSON Schema</code></summary><code>{"size":1..12=1,"contrast":0..50=5,"patternType":"dot¦circle¦line","foreground":json,"background":json}</code></details> | — |
| `filter.gallery.notePaper` | Note Paper | — | <details><summary><code>JSON Schema</code></summary><code>{"imageBalance":0..50=25,"graininess":0..20=10,"relief":0..25=11,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.photocopy` | Photocopy | — | <code>{"detail":1..24=7,"darkness":1..50=8,"foreground":json,"background":json}</code> | — |
| `filter.gallery.plaster` | Plaster | — | <details><summary><code>JSON Schema</code></summary><code>{"imageBalance":0..50=20,"smoothness":1..15=2,"light":"top¦topRight¦right¦bottomRight¦bottom¦bottomLeft¦left¦topLeft","foreground":json,"background":json}</code></details> | — |
| `filter.gallery.reticulation` | Reticulation | — | <details><summary><code>JSON Schema</code></summary><code>{"density":0..50=12,"foregroundLevel":0..50=40,"backgroundLevel":0..50=5,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.stamp` | Stamp | — | <details><summary><code>JSON Schema</code></summary><code>{"lightDarkBalance":0..50=25,"smoothness":1..50=5,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.tornEdges` | Torn Edges | — | <details><summary><code>JSON Schema</code></summary><code>{"imageBalance":0..50=25,"smoothness":1..15=11,"contrast":1..25=17,"foreground":json,"background":json}</code></details> | — |
| `filter.gallery.waterPaper` | Water Paper | — | <code>{"fiberLength":3..50=15,"brightness":0..100=60,"contrast":0..100=80}</code> | — |
| `filter.gallery.glowingEdges` | Glowing Edges | — | <code>{"edgeWidth":1..14=2,"edgeBrightness":0..20=6,"smoothness":1..15=5}</code> | — |
| `filter.gallery.craquelure` | Craquelure | — | <code>{"crackSpacing":2..100=15,"crackDepth":0..10=6,"crackBrightness":0..10=9}</code> | — |
| `filter.gallery.grain` | Grain | — | <details><summary><code>JSON Schema</code></summary><code>{"intensity":0..100=40,"contrast":0..100=50,"grainType":"regular¦soft¦sprinkles¦clumped¦contrasty¦enlarged¦stippled¦horizontal¦vertical¦speckle","foreground":json,"background":json}</code></details> | — |
| `filter.gallery.mosaicTiles` | Mosaic Tiles | — | <code>{"tileSize":2..100=12,"groutWidth":1..15=3,"lightenGrout":0..10=9}</code> | — |
| `filter.gallery.patchwork` | Patchwork | — | <code>{"squareSize":0..10=4,"relief":0..25=8}</code> | — |
| `filter.gallery.stainedGlass` | Stained Glass | — | <details><summary><code>JSON Schema</code></summary><code>{"cellSize":2..50=10,"borderThickness":1..20=4,"lightIntensity":0..10=3,"foreground":json}</code></details> | — |
| `filter.gallery.texturizer` | Texturizer | — | <details><summary><code>JSON Schema</code></summary><code>{"texture":"canvas¦brick¦burlap¦sandstone","scaling":50..200=100,"relief":0..50=4,"light":"top¦topRight¦right¦bottomRight¦bottom¦bottomLeft¦left¦topLeft","invert":bool}</code></details> | — |

## 🔹 Filter: Blur (16)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.blur.gaussianBlur` | Gaussian Blur… | Filter › Blur | <code>{"radius":0.1..1000=1}</code> | — |
| `filter.blur.blur` | Blur | Filter › Blur | `{}` | — |
| `filter.blur.blurMore` | Blur More | Filter › Blur | `{}` | — |
| `filter.blur.boxBlur` | Box Blur… | Filter › Blur | <code>{"radius":1..2000=1}</code> | — |
| `filter.blur.motionBlur` | Motion Blur… | Filter › Blur | <code>{"angle":-360..360=0,"distance":1..2000=10}</code> | — |
| `filter.blur.radialBlur` | Radial Blur… | Filter › Blur | <details><summary><code>JSON Schema</code></summary><code>{"amount":1..100=10,"method":"spin¦zoom","quality":"draft¦good¦best"="good","centerX":0..1=0.5,"centerY":0..1=0.5}</code></details> | — |
| `filter.blur.surfaceBlur` | Surface Blur… | Filter › Blur | <code>{"radius":1..100=5,"threshold":2..255=15}</code> | — |
| `filter.blur.smartBlur` | Smart Blur… | Filter › Blur | <details><summary><code>JSON Schema</code></summary><code>{"radius":0.1..100=3,"threshold":0.1..100=25,"quality":"high¦medium¦low","mode":"normal¦edgeOnly¦overlayEdge"}</code></details> | — |
| `filter.blur.lensBlur` | Lens Blur… | Filter › Blur | <details><summary><code>JSON Schema</code></summary><code>{"radius":0..100=15,"shape":"hexagon¦triangle¦square¦pentagon¦heptagon¦octagon","bladeCurvature":0..100=0,"rotation":0..360=0,"depthMap":"none¦transparency¦layerMask","focalDistance":0..255=0,"invert":bool,"brightness":0..100=0,"threshold":0..255=255,"noise":0..100=0,"distribution":"uniform¦gaussian","monochromatic":bool,"seed":u32=0}</code></details> | — |
| `filter.blur.shapeBlur` | Shape Blur… | Filter › Blur | <details><summary><code>JSON Schema</code></summary><code>{"radius":5..1000=10,"shape":"circle¦ring¦square¦diamond¦triangle¦hexagon¦star¦heart¦cross"}</code></details> | — |
| `filter.blurGallery.tiltShift` | Tilt-Shift… | Filter › Blur Gallery | <details><summary><code>JSON Schema</code></summary><code>{"blur":0..500=15,"centerX":0..1=0.5,"centerY":0..1=0.5,"angle":-90..90=0,"focus":0..1=0.1,"transition":0.01..1=0.15}</code></details> | — |
| `filter.blurGallery.irisBlur` | Iris Blur… | Filter › Blur Gallery | <details><summary><code>JSON Schema</code></summary><code>{"blur":0..500=15,"centerX":0..1=0.5,"centerY":0..1=0.5,"radiusX":0.01..1=0.35,"radiusY":0.01..1=0.25,"angle":-180..180=0,"roundness":0..100=0,"feather":0..0.99=0.5,"pins":json}</code></details> | — |
| `filter.blurGallery.fieldBlur` | Field Blur… | Filter › Blur Gallery | <code>{"blur":0..500=15,"centerX":0..1=0.5,"centerY":0..1=0.5,"pins":json}</code> | — |
| `filter.blurGallery.spinBlur` | Spin Blur… | Filter › Blur Gallery | <details><summary><code>JSON Schema</code></summary><code>{"blurAngle":0..360=15,"centerX":0..1=0.5,"centerY":0..1=0.5,"radiusX":0.01..1=0.3,"radiusY":0.01..1=0.3,"angle":-180..180=0,"pins":json}</code></details> | — |
| `filter.blurGallery.pathBlur` | Path Blur… | Filter › Blur Gallery | <details><summary><code>JSON Schema</code></summary><code>{"speed":0..500=50,"taper":0..100=0,"startX":0..1=0.2,"startY":0..1=0.5,"endX":0..1=0.8,"endY":0..1=0.5,"paths":json}</code></details> | — |
| `filter.blur.average` | Average | Filter › Blur | `{}` | — |

## 🔹 Filter: Distort (9)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.distort.twirl` | Twirl… | Filter › Distort | <code>{"angle":-999..999=50}</code> | — |
| `filter.distort.pinch` | Pinch… | Filter › Distort | <code>{"amount":-100..100=50}</code> | — |
| `filter.distort.spherize` | Spherize… | Filter › Distort | <code>{"amount":-100..100=100,"mode":"normal¦horizontalOnly¦verticalOnly"}</code> | — |
| `filter.distort.wave` | Wave… | Filter › Distort | <details><summary><code>JSON Schema</code></summary><code>{"generators":1..999=5,"wavelengthMin":1..998=10,"wavelengthMax":2..999=120,"amplitudeMin":1..998=5,"amplitudeMax":1..999=35,"type":"sine¦triangle¦square","undefinedAreas":"wrap¦repeat","seed":u32=0}</code></details> | — |
| `filter.distort.ripple` | Ripple… | Filter › Distort | <code>{"amount":-999..999=100,"size":"small¦medium¦large"}</code> | — |
| `filter.distort.polarCoordinates` | Polar Coordinates… | Filter › Distort | <code>{"mode":"rectangularToPolar¦polarToRectangular"}</code> | — |
| `filter.distort.displace` | Displace… | Filter › Distort | <details><summary><code>JSON Schema</code></summary><code>{"horizontal":-999..999=10,"vertical":-999..999=10,"fit":"stretch¦tile","undefinedAreas":"repeat¦wrap","mapDocument":doc,"mapPath":text,"mapLayer":json}</code></details> | — |
| `filter.distort.shear` | Shear… | Filter › Distort | <code>{"amount":-100..100=0,"undefinedAreas":"wrap¦repeat","points":json}</code> | — |
| `filter.distort.zigZag` | ZigZag… | Filter › Distort | <details><summary><code>JSON Schema</code></summary><code>{"amount":-100..100=10,"ridges":0..20=5,"style":"pondRipples¦outFromCenter¦aroundCenter"}</code></details> | — |

## 🔹 Filter: Noise (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.noise.despeckle` | Despeckle | Filter › Noise | `{}` | — |
| `filter.noise.addNoise` | Add Noise… | Filter › Noise | <details><summary><code>JSON Schema</code></summary><code>{"amount":0.1..400=12.5,"distribution":"uniform¦gaussian","monochromatic":bool,"seed":u32=0}</code></details> | — |
| `filter.noise.median` | Median… | Filter › Noise | <code>{"radius":1..500=1}</code> | — |
| `filter.noise.dustAndScratches` | Dust & Scratches… | Filter › Noise | <code>{"radius":1..500=1,"threshold":0..255=0}</code> | — |
| `filter.noise.reduceNoise` | Reduce Noise… | Filter › Noise | <details><summary><code>JSON Schema</code></summary><code>{"strength":0..10=6,"preserveDetails":0..100=60,"reduceColorNoise":0..100=45,"sharpenDetails":0..100=25,"removeJpegArtifact":bool}</code></details> | — |

## 🔹 Filter: Pixelate (7)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.pixelate.mosaic` | Mosaic… | Filter › Pixelate | <code>{"cellSize":2..200=10}</code> | — |
| `filter.pixelate.colorHalftone` | Color Halftone… | Filter › Pixelate | <details><summary><code>JSON Schema</code></summary><code>{"maxRadius":4..127=8,"channel1":-360..360=108,"channel2":-360..360=162,"channel3":-360..360=90,"channel4":-360..360=45}</code></details> | — |
| `filter.pixelate.crystallize` | Crystallize… | Filter › Pixelate | <code>{"cellSize":3..300=10,"seed":u32=0}</code> | — |
| `filter.pixelate.facet` | Facet | Filter › Pixelate | `{}` | — |
| `filter.pixelate.fragment` | Fragment | Filter › Pixelate | `{}` | — |
| `filter.pixelate.mezzotint` | Mezzotint… | Filter › Pixelate | <details><summary><code>JSON Schema</code></summary><code>{"type":"fineDots¦mediumDots¦grainyDots¦coarseDots¦shortLines¦mediumLines¦longLines¦shortStrokes¦mediumStrokes¦longStrokes","seed":u32=0}</code></details> | — |
| `filter.pixelate.pointillize` | Pointillize… | Filter › Pixelate | <code>{"cellSize":3..300=5,"seed":u32=0,"background":json}</code> | — |

## 🔹 Filter: Render (9)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.render.fibers` | Fibers… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"variance":1..64=16,"strength":1..64=4,"seed":u32=0,"foreground":json,"background":json}</code></details> | — |
| `filter.render.lensFlare` | Lens Flare… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"brightness":10..300=100,"centerX":0..1=0.5,"centerY":0..1=0.5,"lens":"zoom¦prime35¦prime105¦moviePrime"}</code></details> | — |
| `filter.render.lightingEffects` | Lighting Effects… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"lightType":"spot¦point¦infinite","intensity":-100..100=75,"lightX":0..1=0.25,"lightY":0..1=0.2,"lightZ":0..2=0.6,"targetX":0..1=0.5,"targetY":0..1=0.55,"cone":1..89=45,"hotspot":0..100=50,"angle":-180..180=135,"elevation":0..90=45,"gloss":-100..100=0,"metallic":-100..100=0,"exposure":-100..100=0,"ambience":-100..100=8,"texture":"none¦red¦green¦blue¦alpha¦luminance","height":0..100=50,"whiteIsHigh":bool=true,"lights":json}</code></details> | — |
| `filter.render.relight` | Relight… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"angle":-180..180=45,"elevation":0..90=40,"intensity":0..100=40,"ambient":0..100=55,"warmth":-100..100=0,"softness":1..100=25}</code></details> | — |
| `filter.render.clouds` | Clouds | Filter › Render | <code>{"seed":u32=0}</code> | — |
| `filter.render.differenceClouds` | Difference Clouds | Filter › Render | <code>{"seed":u32=0}</code> | — |
| `filter.render.flame` | Flame… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"flameType":"oneFlameAlongPath¦multipleFlamesAlongPath¦multipleFlamesPathDirections¦multipleFlamesVariousLength¦candleLight¦multipleFlamesOneDirection","length":1..1000=150,"randomizeLength":bool,"width":1..1000=40,"angle":-180..180=0,"interval":1..1000=60,"adjustIntervalForLoops":bool=true,"useCustomColor":bool,"color":color,"turbulent":0..100=25,"jag":0..100=25,"opacity":0..100=75,"flameLines":1..100=20,"flameBottomAlignment":0..100=20,"flameStyle":"normal¦violent¦flat","flameShape":"parallel¦toCenter¦spread¦oval¦pointed","randomizeShapes":bool,"seed":u32=0,"quality":"draft¦low¦medium¦high¦fine","path":text,"newLayer":bool} → {layer,newLayer,bounds,primitives,usedPath}</code></details> | — |
| `filter.render.pictureFrame` | Picture Frame… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"frame":"vineWithFlowers¦vineWithLeaves¦ivy¦roses¦daisies¦berries¦bamboo¦waves¦zigzag¦dots¦rope¦scallops¦doubleLine¦snowflakes¦stars¦hearts","margin":0..30=4,"size":1..100=50,"arrangement":1..100=50,"lines":1..5=1,"thickness":1..100=30,"fade":0..100=0,"vineColor":color,"flowerColor":color,"leafColor":color,"seed":u32=0,"newLayer":bool} → {layer,newLayer,bounds,primitives,frame}</code></details> | — |
| `filter.render.tree` | Tree… | Filter › Render | <details><summary><code>JSON Schema</code></summary><code>{"baseTreeType":1..34=1,"lightDirection":1..5=3,"leavesAmount":0..100=50,"leavesSize":0..200=100,"branchesHeight":50..300=100,"branchesThickness":50..200=100,"defaultLeaves":bool=true,"leavesColor":color,"customBranchColor":bool,"branchesColor":color,"flatShading":bool,"seed":u32=0,"x":0..1=0.5,"y":0..1=0.95,"size":0.05..2=0.8,"newLayer":bool} → {layer,newLayer,bounds,primitives,treeType}</code></details> | — |

## 🔹 Filter: Sharpen (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.sharpen.sharpen` | Sharpen | Filter › Sharpen | `{}` | — |
| `filter.sharpen.sharpenMore` | Sharpen More | Filter › Sharpen | `{}` | — |
| `filter.sharpen.sharpenEdges` | Sharpen Edges | Filter › Sharpen | `{}` | — |
| `filter.sharpen.unsharpMask` | Unsharp Mask… | Filter › Sharpen | <code>{"amount":1..500=50,"radius":0.1..1000=1,"threshold":0..255=0}</code> | — |
| `filter.sharpen.smartSharpen` | Smart Sharpen… | Filter › Sharpen | <code>{"amount":1..500=100,"radius":0.1..64=1,"reduceNoise":0..100=10}</code> | — |

## 🔹 Filter: Stylize (9)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `filter.stylize.emboss` | Emboss… | Filter › Stylize | <code>{"angle":-180..180=135,"height":1..100=3,"amount":1..500=100}</code> | — |
| `filter.stylize.findEdges` | Find Edges | Filter › Stylize | `{}` | — |
| `filter.stylize.solarize` | Solarize | Filter › Stylize | `{}` | — |
| `filter.stylize.diffuse` | Diffuse… | Filter › Stylize | <code>{"mode":"normal¦darkenOnly¦lightenOnly¦anisotropic","seed":u32=0}</code> | — |
| `filter.stylize.extrude` | Extrude… | Filter › Stylize | <details><summary><code>JSON Schema</code></summary><code>{"type":"blocks¦pyramids","size":2..255=30,"depth":1..255=30,"depthMode":"random¦levelBased","solidFrontFaces":bool,"maskIncompleteBlocks":bool,"seed":u32=0}</code></details> | — |
| `filter.stylize.oilPaint` | Oil Paint… | Filter › Stylize | <details><summary><code>JSON Schema</code></summary><code>{"stylization":0.1..10=4,"cleanliness":0..10=5,"scale":0.1..10=1,"bristleDetail":0..10=5,"lighting":bool=true,"angle":-180..180=-60,"shine":0..10=1}</code></details> | — |
| `filter.stylize.tiles` | Tiles… | Filter › Stylize | <details><summary><code>JSON Schema</code></summary><code>{"count":1..99=10,"maxOffset":1..99=10,"fill":"background¦foreground¦inverse¦unaltered","seed":u32=0,"foreground":json,"background":json}</code></details> | — |
| `filter.stylize.traceContour` | Trace Contour… | Filter › Stylize | <code>{"level":0..255=128,"edge":"lower¦upper"}</code> | — |
| `filter.stylize.wind` | Wind… | Filter › Stylize | <code>{"method":"wind¦blast¦stagger","direction":"fromRight¦fromLeft","seed":u32=0}</code> | — |

## 🔹 Gradient (12)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `gradient.fill.create` | Gradient (Live) | — | <details><summary><code>JSON Schema</code></summary><code>{"from":[x,y],"to":[x,y],"style":"linear¦radial¦angle¦reflected¦diamond"="linear","gradient":preset name?,"stops":[[t,"#rrggbb"¦"foreground"¦"background"],…]?,"transparency":[[t,0..100],…]? (default: the current gradient),"reverse":bool=false,"dither":bool=true,"opacity":0..100=100,"mode":"normal¦multiply¦…"="normal"} → {"layer":id}; a Gradient Fill layer above the active layer, masked by the selection</code></details> | — |
| `gradient.fill.set` | Edit Gradient Fill | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"from":[x,y]?,"to":[x,y]?,"style":"linear¦radial¦angle¦reflected¦diamond"?,"angle":deg?,"scale":%?,"offset":[x%,y%]?,"reverse":bool?,"dither":bool?,"align":bool?,"stops":[[t,"#rrggbb"¦"#rrggbbaa"¦[r,g,b,a]¦"foreground"¦"background"],…]?,"transparency":[[t,0..100],…]?,"midpoints":[0.05..0.95 per segment]?} → the gradient (see gradient.fill.get)</code></details> | — |
| `gradient.fill.stop` | Edit Gradient Stop | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"action":"add¦move¦delete¦color¦opacity¦midpoint","kind":"color¦opacity"="color","index":stop (midpoint: segment) number,"location":0..1 (midpoint: of the segment),"color":"#rrggbb"¦[r,g,b,a]¦"foreground"¦"background" (add: default the colour there),"opacity":0..100} → the gradient</code></details> | — |
| `gradient.fill.get` | Gradient Fill Settings | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} → {"style","angle","scale":%,"offset":[%,%],"reverse","dither","align","from":[x,y],"to":[x,y],"stops":[[t,"#hex"]],"transparency":[[t,%]],"midpoints":[…]}</code></details> | — |
| `gradient.presets.importGrd` | Import Gradients… | — | <details><summary><code>JSON Schema</code></summary><code>{"path":".grd file"?,"data":base64 bytes?,"group":string?=file name,"replace":bool=true (replace a group of the same name)}</code></details> | — |
| `gradient.presets.list` | Gradient Presets | — | <code>{} → {groups:[{name,presets:[{name,stops,transparency}]}],current}</code> | — |
| `gradient.presets.select` | Select Gradient | — | <details><summary><code>JSON Schema</code></summary><code>{"preset":name¦"stops":[[t 0..1,"#rrggbb"¦"foreground"¦"background"],…],"transparency":[[t,opacity 0..100],…]?,"group":name?,"applyToLayer":bool=true (also recolours a selected Gradient Fill layer)} → {current,layer?}. The Gradient tool paints with it.</code></details> | — |
| `gradient.presets.stop` | Edit Gradient Stop | — | <details><summary><code>JSON Schema</code></summary><code>{"action":"add¦move¦delete¦color¦opacity","kind":"color¦opacity"="color","index":stop number,"location":0..1,"color":"#rrggbb"¦"foreground"¦"background" (add: default the colour there),"opacity":0..100} → {"current"}. Edits the Gradient tool's current gradient (no layer, midpoints stay centred).</code></details> | — |
| `gradient.presets.apply` | New Gradient Fill Layer from Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"preset":name?=current¦"stops":[[t 0..1,"#rrggbb"¦"foreground"¦"background"],…],"transparency":[[t,opacity 0..100],…]?,"angle":deg=90,"style":"linear¦radial¦angle¦reflected¦diamond"="linear","scale":10..150=100,"reverse":bool} → {layer}</code></details> | — |
| `gradient.presets.new` | New Gradient Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str="Custom","group":name?=first,"stops":[[t 0..1,"#rrggbb"¦"foreground"¦"background"],…],"transparency":[[t,opacity 0..100],…]? (default: the current gradient)}</code></details> | — |
| `gradient.presets.edit` | Edit Gradient Presets | — | <details><summary><code>JSON Schema</code></summary><code>{"action":"rename¦delete¦move¦newGroup¦renameGroup¦deleteGroup","preset":name¦[names] (rename/delete/move),"group":name? (narrows the lookup; the group for renameGroup/deleteGroup),"name":str (rename: new name; newGroup/renameGroup: group name),"to":group (move),"index":n? (move)}</code></details> | — |
| `gradient.presets.reset` | Restore Default Gradients | — | <code>{"append":bool=false}</code> | — |

## 🔹 History (2)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `history.deleteState` | Delete State | — | <details><summary><code>JSON Schema</code></summary><code>{"state":index?} (History entry, 0 = oldest; default the current state) → {states,current}: removes the state and every later one</code></details> | — |
| `history.newDocument` | New Document from State | — | <details><summary><code>JSON Schema</code></summary><code>{"state":index?,"name":str?} (History entry, 0 = oldest; default the current state) → {document,name}</code></details> | — |

## 🔹 Image (41)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `image.imageRotation.flipCanvasHorizontal` | Flip Canvas Horizontal | Image › Image Rotation | `{}` | — |
| `image.imageRotation.flipCanvasVertical` | Flip Canvas Vertical | Image › Image Rotation | `{}` | — |
| `image.imageRotation.180` | 180° | Image › Image Rotation | `{}` | — |
| `image.imageRotation.90cw` | 90° Clockwise | Image › Image Rotation | `{}` | — |
| `image.imageRotation.90ccw` | 90° Counter Clockwise | Image › Image Rotation | `{}` | — |
| `image.autoTone` | Auto Tone | Image | `{}` | `Cmd+Shift+L` |
| `image.autoContrast` | Auto Contrast | Image | `{}` | `Cmd+Alt+Shift+L` |
| `image.autoColor` | Auto Color | Image | `{}` | `Cmd+Shift+B` |
| `image.imageSize` | Image Size… | Image | <details><summary><code>JSON Schema</code></summary><code>{"width":px,"height":px,"resolution":ppi,"resample":"bicubic¦bilinear¦nearest¦lanczos¦preserveDetails¦none"="bicubic"} → {width,height} (the document's size after the command; "none" changes the resolution only)</code></details> | — |
| `image.canvasSize` | Canvas Size… | Image | <details><summary><code>JSON Schema</code></summary><code>{"width":px,"height":px,"relative":bool=false,"anchor":"topLeft¦top¦topRight¦left¦center¦right¦bottomLeft¦bottom¦bottomRight"="center","extensionColor":"background¦foreground¦white¦black¦transparent¦#rrggbb"="background"}</code></details> | — |
| `image.crop` | Crop | Image | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"width":px,"height":px,"angle":-3600..3600=0,"deleteCroppedPixels":bool=true}</code></details> | — |
| `image.trim` | Trim… | Image | <details><summary><code>JSON Schema</code></summary><code>{"basedOn":"transparent¦topLeft¦bottomRight"="transparent","top":bool=true,"bottom":bool=true,"left":bool=true,"right":bool=true}</code></details> | — |
| `image.mode.rgb` | RGB Color | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"profile":"<builtin id>¦working¦/path/to/profile.icc"=working,"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true}</code></details> | — |
| `image.mode.grayscale` | Grayscale | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"profile":"<builtin id>¦working¦/path/to/profile.icc"=working,"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true}</code></details> | — |
| `image.mode.cmyk` | CMYK Color | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"profile":"<builtin id>¦working¦/path/to/profile.icc"=working,"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true}</code></details> | — |
| `image.mode.lab` | Lab Color | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"profile":"<builtin id>¦working¦/path/to/profile.icc"=working,"intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true}</code></details> | — |
| `image.mode.bits8` | 8 Bits/Channel | Image › Mode | `{}` | — |
| `image.mode.bits16` | 16 Bits/Channel | Image › Mode | `{}` | — |
| `image.mode.bits32` | 32 Bits/Channel | Image › Mode | `{}` | — |
| `image.duplicate` | Duplicate… | Image | <code>{"name":str,"mergedOnly":bool=false}</code> | — |
| `image.revealAll` | Reveal All | Image | `{}` | — |
| `image.applyImage` | Apply Image… | Image | <details><summary><code>JSON Schema</code></summary><code>{"source":{"document":index?,"layer":id¦"merged"="merged","channel":"composite"¦"red¦…"¦index¦"selection"¦"transparency"¦"mask"="composite","invert":bool=false},"blending":"normal¦multiply¦screen¦overlay¦softLight¦hardLight¦colorDodge¦colorBurn¦darken¦lighten¦difference¦exclusion¦linearBurn¦linearDodge¦add¦subtract¦…"="multiply","opacity":0..100=100,"scale":1..2=1,"offset":-255..255=0,"preserveTransparency":bool=false,"mask":{"document","layer","channel","invert"}?,"sourceChannel¦sourceDocument¦sourceLayer¦sourceInvert¦maskChannel¦…":flat form of source/mask?}</code></details> | — |
| `image.calculations` | Calculations… | Image | <details><summary><code>JSON Schema</code></summary><code>{"source1":{"document","layer","channel","invert"},"source2":{…},"blending":"multiply¦…"="multiply","opacity":0..100=100,"scale":1..2=1,"offset":-255..255=0,"mask":{…}?,"result":"newChannel¦newDocument¦selection"="newChannel","name":str?}</code></details> | — |
| `image.rotation.arbitrary` | Arbitrary… | Image › Image Rotation | <details><summary><code>JSON Schema</code></summary><code>{"angle":-359.99..359.99=0,"direction":"cw¦ccw"="cw","interpolation":"bicubic¦bilinear¦nearest"="bicubic"}</code></details> | — |
| `image.mode.indexedColor` | Indexed Color… | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"palette":"selective¦perceptual¦adaptive¦exact¦systemMac¦systemWindows¦web¦uniform"="selective","colors":2..256=256,"forced":"blackWhite¦none¦primaries¦web"="blackWhite","transparency":bool=true,"dither":"diffusion¦none¦pattern¦noise"="diffusion","amount":0..100=75} (flattens)</code></details> | — |
| `image.mode.colorTable` | Color Table… | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"table":"custom¦blackBody¦grayscale¦spectrum¦systemMac¦systemWindows¦web"="custom","colors":json,"entries":json,"transparent":json} (colors: ["#rrggbb", …]; entries: {"index": "#rrggbb"}; transparent: index¦null)</code></details> | — |
| `image.mode.bitmap` | Bitmap… | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"method":"diffusion¦threshold¦pattern¦halftone"="diffusion","frequency":1..999=53,"angle":-180..180=45,"shape":"round¦ellipse¦line¦square¦diamond¦cross"="round"} (halftone frequency in lines/inch; flattens)</code></details> | — |
| `image.mode.duotone` | Duotone… | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{"type":"duotone¦monotone¦tritone¦quadtone"="duotone","inks":json} (inks: [{"name","color":"#rrggbb","curve":[[in,out],…] in 0..100}, …])</code></details> | — |
| `image.mode.multichannel` | Multichannel | Image › Mode | <details><summary><code>JSON Schema</code></summary><code>{} (flattens; RGB → Cyan/Magenta/Yellow, CMYK → Cyan/Magenta/Yellow/Black, Lab → Alpha 1–3, Grayscale → Black, Duotone → its inks)</code></details> | — |
| `image.analysis.setMeasurementScale` | Set Measurement Scale… | Image › Analysis | <details><summary><code>JSON Schema</code></summary><code>{"preset":"default¦custom"="custom","pixelLength":px,"logicalLength":number,"units":str (e.g. "mm")} (no params: read the scale)</code></details> | — |
| `image.analysis.selectDataPoints` | Select Data Points… | Image › Analysis | <details><summary><code>JSON Schema</code></summary><code>{"selection":[keys]¦{key:bool}?,"ruler":[keys]¦{key:bool}?,"count":[keys]¦{key:bool}?,"reset":bool=false} (keys: label,dateTime,document,source,scale,scaleUnits,scaleFactor,count,area,perimeter,circularity,height,width,grayMin,grayMax,grayMean,grayMedian,integratedDensity,histogram,length,angle)</code></details> | — |
| `image.analysis.recordMeasurements` | Record Measurements | Image › Analysis | <details><summary><code>JSON Schema</code></summary><code>{"source":"auto¦selection¦ruler¦count"="auto"} → appended Measurement Log rows (selection: summary + one row per feature)</code></details> | — |
| `image.analysis.rulerTool` | Ruler Tool | Image › Analysis | <details><summary><code>JSON Schema</code></summary><code>{"start":[x,y],"end":[x,y],"protractor":[x,y]¦null?,"clear":bool=false} (no params: read) → X/Y/W/H/angle/L1/L2</code></details> | — |
| `image.analysis.countTool` | Count Tool | Image › Analysis | <code>{} → count groups and markers (edit with count.*)</code> | — |
| `image.analysis.placeScaleMarker` | Place Scale Marker… | Image › Analysis | <details><summary><code>JSON Schema</code></summary><code>{"length":logical units=nice ≈ width/5,"font":str?,"fontSize":pt=12,"displayText":bool=true,"textPosition":"top¦bottom"="bottom","color":"black¦white"="black"}</code></details> | — |
| `image.analysis.straightenLayer` | Straighten Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"crop":bool=(active layer is the Background)} (rotates so the ruler line is level; crop = rotate the canvas and crop to the image)</code></details> | — |
| `image.analysis.info` | Analysis Info | — | <code>{} → scale, ruler readout, count groups, note and log counts</code> | — |
| `image.trap` | Trap… | Image | <code>{width:px>=1=1} → {trapped,width}: spread inks at colour edges (CMYK only)</code> | — |
| `image.variables.define` | Define… | Image › Variables | <details><summary><code>JSON Schema</code></summary><code>{defs:[{name, layer:id, type:visibility¦textReplacement¦pixelReplacement, method?:fit¦fill¦asIs¦conform, align?, clip?}]} → {defs,dataSets,active}</code></details> | — |
| `image.variables.dataSets` | Data Sets… | Image › Variables | <details><summary><code>JSON Schema</code></summary><code>{dataSets:[{name, values:[{variable, kind:visibility¦text¦pixels, value}]}], append?} → {defs,dataSets,active}</code></details> | — |
| `image.applyDataSet` | Apply Data Set… | Image | <details><summary><code>JSON Schema</code></summary><code>{name¦index} → {applied, index}: sets layer visibility/text/pixels from the data set (one history step)</code></details> | — |

## 🔹 Image Adjustments (23)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `image.adjustments.desaturate` | Desaturate | Image › Adjustments | `{}` | `Cmd+Shift+U` |
| `image.adjustments.brightnessContrast` | Brightness/Contrast… | Image › Adjustments | <code>{"brightness":-150..150=0,"contrast":-50..100=0,"legacy":bool=false}</code> | — |
| `image.adjustments.levels` | Levels… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"inBlack":0..253=0,"gamma":0.01..9.99=1,"inWhite":2..255=255,"outBlack":0..255=0,"outWhite":0..255=255,"red":json,"green":json,"blue":json} (top level = composite; per channel {"inBlack","gamma","inWhite","outBlack","outWhite"} under red/green/blue, gray, cyan/magenta/yellow/black or lightness/a/b)</code></details> | `Cmd+L` |
| `image.adjustments.curves` | Curves… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"points":json,"red":json,"green":json,"blue":json,"eyedropper":json} (curves as [[in,out],…] in 0..255, 2..19 points: points = composite; red/green/blue, gray, cyan/magenta/yellow/black or lightness/a/b per channel; eyedropper = the dialog's Set Black/Neutral Gray/White Point picker {"point":"black¦gray¦white","at":[x,y]} or {"point",…,"color":[r,g,b] 0..1}, RGB and Grayscale (no gray) only, applied over the given curves)</code></details> | `Cmd+M` |
| `image.adjustments.exposure` | Exposure… | Image › Adjustments | <code>{"exposure":-20..20=0,"offset":-0.5..0.5=0,"gamma":0.01..9.99=1}</code> | — |
| `image.adjustments.vibrance` | Vibrance… | Image › Adjustments | <code>{"vibrance":-100..100=0,"saturation":-100..100=0}</code> | — |
| `image.adjustments.hueSaturation` | Hue/Saturation… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"hue":-180..180=0,"saturation":-100..100=0,"lightness":-100..100=0,"colorize":bool=false,"reds":json,"yellows":json,"greens":json,"cyans":json,"blues":json,"magentas":json} (per range {"hue","saturation","lightness","range":[4 hue degrees]}; colorize: hue 0..360, saturation 0..100)</code></details> | `Cmd+U` |
| `image.adjustments.colorBalance` | Color Balance… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"shadows":json,"midtones":json,"highlights":json,"preserveLuminosity":bool=true} (each tone [cyan-red, magenta-green, yellow-blue] in -100..100)</code></details> | `Cmd+B` |
| `image.adjustments.blackWhite` | Black & White… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"reds":-200..300=40,"yellows":-200..300=60,"greens":-200..300=40,"cyans":-200..300=60,"blues":-200..300=20,"magentas":-200..300=80,"tint":bool=false,"tintColor":"#rrggbb"}</code></details> | `Cmd+Alt+Shift+B` |
| `image.adjustments.photoFilter` | Photo Filter… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"filter":"warming85¦warmingLBA¦warming81¦cooling80¦coolingLBB¦cooling82¦red¦orange¦yellow¦green¦cyan¦blue¦violet¦magenta¦sepia¦deepRed¦deepBlue¦deepEmerald¦deepYellow¦underwater","color":"#rrggbb","density":0..100=25,"preserveLuminosity":bool=true}</code></details> | — |
| `image.adjustments.channelMixer` | Channel Mixer… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"red":json,"green":json,"blue":json,"gray":json,"monochrome":bool=false} (per output channel [red %, green %, blue %, constant %] in -200..200; gray = the monochrome mix)</code></details> | — |
| `image.adjustments.invert` | Invert | Image › Adjustments | `{}` | `Cmd+I` |
| `image.adjustments.posterize` | Posterize… | Image › Adjustments | <code>{"levels":2..255=4}</code> | — |
| `image.adjustments.threshold` | Threshold… | Image › Adjustments | <code>{"level":1..255=128}</code> | — |
| `image.adjustments.gradientMap` | Gradient Map… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"stops":json,"reverse":bool=false,"dither":bool=false} (stops [[location 0..1, "#rrggbb"], …], 2..64)</code></details> | — |
| `image.adjustments.selectiveColor` | Selective Color… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"method":"relative¦absolute"="relative","colors":"reds¦yellows¦greens¦cyans¦blues¦magentas¦whites¦neutrals¦blacks"="reds","cyan":-100..100=0,"magenta":-100..100=0,"yellow":-100..100=0,"black":-100..100=0,"reds":json} (per-range [c,m,y,k] arrays: reds yellows greens cyans blues magentas whites neutrals blacks)</code></details> | — |
| `image.adjustments.colorLookup` | Color Lookup… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"lut":"none¦warm¦cool¦tealOrange¦bleachBypass¦fadedFilm¦dayForNight¦monoContrast¦crossProcess"="none","file":text,"interpolation":"trilinear¦tetrahedral"="trilinear","dither":bool=false,"data":json} (file: .cube/.3dl/.look path; data: file text + "fileName")</code></details> | — |
| `image.adjustments.equalize` | Equalize | Image › Adjustments | `{}` | — |
| `image.adjustments.shadowsHighlights` | Shadows/Highlights… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"shadowAmount":0..100=35,"shadowTone":0..100=50,"shadowRadius":0..2500=30,"highlightAmount":0..100=0,"highlightTone":0..100=50,"highlightRadius":0..2500=30,"color":-100..100=20,"midtone":-100..100=0,"blackClip":0..50=0.01,"whiteClip":0..50=0.01}</code></details> | — |
| `image.adjustments.replaceColor` | Replace Color… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"color":json,"fuzziness":0..200=40,"hue":-180..180=0,"saturation":-100..100=0,"lightness":-100..100=0} (color: "#rrggbb", default the foreground colour)</code></details> | — |
| `image.adjustments.matchColor` | Match Color… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"source":doc,"sourceLayer":json,"luminance":1..200=100,"intensity":1..200=100,"fade":0..100=0,"neutralize":bool=false,"useSelectionInSource":bool=false,"useSelectionInTarget":bool=false} (source: document index; sourceLayer: layer id, default the merged image)</code></details> | — |
| `image.adjustments.hdrToning` | HDR Toning… | Image › Adjustments | <details><summary><code>JSON Schema</code></summary><code>{"radius":1..500=30,"strength":0.1..4=0.5,"gamma":0.1..2=1,"exposure":-5..5=0,"detail":-100..300=30,"shadow":-100..100=0,"highlight":-100..100=0,"vibrance":-100..100=0,"saturation":-100..100=20,"curve":[[in,out],…] 0..255} (flattens the image)</code></details> | — |
| `image.adjustments.colorLookup.list` | List Color Lookup Looks | — | `{}` | — |

## 🔹 Jobs (2)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `jobs.list` | List Background Jobs | — | `{}` | — |
| `jobs.cancel` | Cancel Background Job | — | <code>{"job":u64? (default: every running job)}</code> | — |

## 🔹 Layer Operations (164)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `layer.new.layer` | Layer… | Layer › New | <code>{"name":str?}</code> | `Cmd+Shift+N` |
| `layer.new.group` | Group… | Layer › New | <code>{"name":str?}</code> | — |
| `layer.groupLayers` | Group Layers | Layer | <code>{"layer":id?,"name":str?} (no layer: every selected layer)</code> | `Cmd+G` |
| `layer.duplicate` | Duplicate Layer… | Layer | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"inPlace":bool=false (artboards: keep the copy on the original instead of beside it)} (no layer: every selected layer)</code></details> | — |
| `layer.delete` | Delete Layer | Layer › Delete | <code>{"layer":id?} (no layer: every selected layer)</code> | — |
| `layer.select` | Select Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id,"mode":"replace¦toggle¦range¦add"="replace"} (toggle = ⌘-click, range = ⇧-click)</code></details> | — |
| `layer.setProps` | Layer Properties | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"name":str?,"visible":bool?,"opacity":0..1?,"fill":0..1?,"blend":"Multiply¦…"?,"clipped":bool?,"locked":bool?,"locks":{"transparency","pixels","position","artboard","all":bool}?,"channels":[bool,…]? (Advanced Blending: which colour channels blend, R G B / C M Y K / L a b)}</code></details> | — |
| `layer.arrange.bringForward` | Bring Forward | Layer › Arrange | <code>{"layer":id?}</code> | `Cmd+]` |
| `layer.arrange.sendBackward` | Send Backward | Layer › Arrange | <code>{"layer":id?}</code> | `Cmd+[` |
| `layer.arrange.bringToFront` | Bring to Front | Layer › Arrange | <code>{"layer":id?}</code> | `Cmd+Shift+]` |
| `layer.arrange.sendToBack` | Send to Back | Layer › Arrange | <code>{"layer":id?}</code> | `Cmd+Shift+[` |
| `layer.createClippingMask` | Create Clipping Mask | Layer | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} (no layer: every selected layer but the lowest clips to the layer below it)</code></details> | `Cmd+Alt+G` |
| `layer.releaseClippingMask` | Release Clipping Mask | Layer | <code>{"layer":id?} (no layer: every selected layer)</code> | — |
| `layer.layerMask.revealAll` | Reveal All | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.layerMask.hideAll` | Hide All | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.layerMask.revealSelection` | Reveal Selection | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.layerMask.delete` | Delete | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.mergeDown` | Merge Down | Layer | <code>{"layer":id?}</code> | — |
| `layer.flattenImage` | Flatten Image | Layer | `{}` | — |
| `layer.newFillLayer.solidColor` | Solid Color… | Layer › New Fill Layer | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb"=foreground,"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask, making a shape; else the selection is the layer mask)}</code></details> | — |
| `layer.newFillLayer.gradient` | Gradient… | Layer › New Fill Layer | <details><summary><code>JSON Schema</code></summary><code>{"from":"#rrggbb","to":"#rrggbb","angle":deg=90,"style":"linear¦radial¦angle¦reflected¦diamond","reverse":bool,"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask)}</code></details> | — |
| `layer.setAdjustment` | Adjustment Properties | — | <code>{"layer":id?, …params of that adjustment kind}</code> | — |
| `layer.moveTo` | Reorder Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?¦ "layers":[id,…]?, "target":id,"position":"above¦below¦into"="above","copy":bool=false} (layers: one undoable move, document order preserved; copy: place duplicates there and leave the layers, as ⌥-dragging a Layers panel row)</code></details> | — |
| `layer.translate` | Move Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"dx":i32,"dy":i32} (no layer: every selected layer; linked layers follow)</code></details> | — |
| `layer.fill.solidColor.set` | Solid Color… | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? (default active),"document":id? (reject a stale dialog),"color":"#rrggbb"¦"#rrggbbaa"¦[r,g,b,a?]¦{mode,c:[f32;4],alpha:f32}}; RGB values without alpha preserve it; stored Color preserves its model and precision</code></details> | — |
| `layer.vectorMask.add` | Add Vector Mask | Layer › Vector Mask | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"path":{…}? ¦ "name":str¦"work"? (copy a document path),"hide":bool=false (Hide All = inverted)} → vector mask info. Without a path the mask reveals all.</code></details> | — |
| `layer.vectorMask.fromPath` | Vector Mask from Current Path | Layer › Vector Mask | <code>{"layer":id?,"name":str¦"work"="work"}</code> | — |
| `layer.vectorMask.edit` | Edit Vector Mask | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"path":{…}?,"enabled":bool?,"linked":bool?,"density":0..100?,"feather":px?,"invert":true?}. path: {"subpaths":[{"closed":bool=true,"op":"combine¦subtract¦intersect¦exclude","knots":[[x,y] ¦ {"anchor":[x,y],"in":[x,y],"out":[x,y],"smooth":bool}]}],"fillRule":"nonzero¦evenodd","inverted":bool}</code></details> | — |
| `layer.vectorMask.delete` | Delete Vector Mask | Layer › Vector Mask | <code>{"layer":id?}</code> | — |
| `layer.rasterize.vectorMask` | Rasterize Vector Mask | — | <code>{"layer":id?} (multiplied into the layer mask)</code> | — |
| `layer.vectorMask.revealAll` | Vector Mask: Reveal All | — | <code>{"layer":id?}</code> | — |
| `layer.vectorMask.hideAll` | Vector Mask: Hide All | — | <code>{"layer":id?}</code> | — |
| `layer.vectorMask.currentPath` | Vector Mask: Current Path | — | <code>{"layer":id?,"name":str¦"work"="work"}</code> | — |
| `layer.vectorMask.enabled` | Enable Vector Mask | — | <code>{"layer":id?,"enabled":bool? (default: toggle)}</code> | — |
| `layer.vectorMask.linked` | Link Vector Mask | — | <code>{"layer":id?,"linked":bool? (default: toggle)}</code> | — |
| `layer.rasterize.shape` | Rasterize Shape | — | <code>{"layer":id?}</code> | — |
| `layer.combineShapes.unite` | Unite Shapes | — | <code>{"layer":id?,"subpath":index? (default all but the first)}</code> | — |
| `layer.combineShapes.subtractFrontShape` | Subtract Front Shape | — | <code>{"layer":id?,"subpath":index?}</code> | — |
| `layer.combineShapes.intersectShapeAreas` | Intersect Shape Areas | — | <code>{"layer":id?,"subpath":index?}</code> | — |
| `layer.combineShapes.excludeOverlappingShapes` | Exclude Overlapping Shapes | — | <code>{"layer":id?,"subpath":index?}</code> | — |
| `layer.combineShapes.mergeShapeComponents` | Merge Shape Components | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"tolerance":px=0.1} (bakes the path operations into plain combined outlines, traced from the exact coverage)</code></details> | — |
| `layer.vectorMask.info` | Vector Mask Info | — | <code>{"layer":id?} → {layer,path,enabled,linked,density,feather}</code> | — |
| `layer.removeBackground` | Remove Background | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"sampleAllLayers":bool=false,"refine":bool=true} → {layer,bounds} (adds a layer mask from Select Subject; the Background becomes a normal layer)</code></details> | — |
| `layer.new.layerViaCopy` | Layer via Copy | Layer › New | `{}` | `Cmd+J` |
| `layer.new.layerViaCut` | Layer via Cut | Layer › New | `{}` | `Cmd+Shift+J` |
| `layer.mergeVisible` | Merge Visible | Layer | `{}` | `Cmd+Shift+E` |
| `layer.new.layerFromBackground` | Layer From Background… | Layer › New | `{}` | — |
| `layer.layerMask.enabled` | Disable Layer Mask | Layer › Layer Mask | <code>{"layer":id?,"enabled":bool? (default: toggle)}</code> | — |
| `layer.layerMask.linked` | Unlink Layer Mask | Layer › Layer Mask | <code>{"layer":id?,"linked":bool? (default: toggle)}</code> | — |
| `layer.rasterize.layer` | Layer | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `layer.rasterize.allLayers` | All Layers | Layer › Rasterize | `{}` | — |
| `layer.rasterize.type` | Type | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `layer.rasterize.fillContent` | Fill Content | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `layer.rasterize.smartObject` | Smart Object | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `layer.delete.hiddenLayers` | Hidden Layers | Layer › Delete | `{}` | — |
| `layer.ungroupLayers` | Ungroup Layers | Layer | <code>{"layer":id?}</code> | `Cmd+Shift+G` |
| `layer.hideLayers` | Hide Layers | Layer | <code>{"layer":id?}</code> | `Cmd+,` |
| `layer.showLayers` | Show Layers | — | <code>{"layer":id?}</code> | — |
| `layer.showOnly` | Show Only This Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} → {shownAlone} (⌥-click a layer's eye; again restores the other layers' visibility)</code></details> | — |
| `layer.smartObjects.convertToSmartObject` | Convert to Smart Object | Layer › Smart Objects | <code>{"layer":id?} (default: combine selected layers)</code> | — |
| `layer.smartObjects.newSmartObjectViaCopy` | New Smart Object via Copy | Layer › Smart Objects | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.rasterize` | Rasterize | Layer › Smart Objects | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.editContents` | Edit Contents | Layer › Smart Objects | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} → opens the contents as a new document; saving (layer.smartObjects.saveContents) or closing it updates the smart object</code></details> | — |
| `layer.smartObjects.saveContents` | Save Contents | — | <code>{} (in an Edit Contents document)</code> | — |
| `layer.smartObjects.replaceContents` | Replace Contents… | Layer › Smart Objects | <code>{"layer":id?,"path":str}</code> | — |
| `layer.smartObjects.exportContents` | Export Contents… | Layer › Smart Objects | <code>{"layer":id?,"path":str}</code> | — |
| `layer.smartObjects.relinkToFile` | Relink to File… | Layer › Smart Objects | <code>{"layer":id?,"path":str}</code> | — |
| `layer.smartObjects.updateModifiedContent` | Update Modified Content | Layer › Smart Objects | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.updateAllModifiedContent` | Update All Modified Content | Layer › Smart Objects | `{}` | — |
| `layer.smartObjects.convertToLayers` | Convert to Layers | Layer › Smart Objects | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} (contents unpacked at the placement: one layer, or a group named after the smart object; smart filters are discarded)</code></details> | — |
| `layer.smartObjects.convertToEmbedded` | Convert to Embedded | Layer › Smart Objects | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.convertToLinked` | Convert to Linked… | Layer › Smart Objects | <code>{"layer":id?,"path":str} (writes the contents there)</code> | — |
| `layer.smartFilter.disableSmartFilters` | Disable Smart Filters | Layer › Smart Filter | <code>{"layer":id?,"enabled":bool? (default: toggle)}</code> | — |
| `layer.smartFilter.deleteFilterMask` | Delete Filter Mask | Layer › Smart Filter | <code>{"layer":id?}</code> | — |
| `layer.smartFilter.disableFilterMask` | Disable Filter Mask | Layer › Smart Filter | <code>{"layer":id?,"enabled":bool? (default: toggle)}</code> | — |
| `layer.smartFilter.blendingOptions` | Blending Options… | Layer › Smart Filter | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"index":u32? (0 = bottom; default top),"blend":str?,"opacity":0..1?}</code></details> | — |
| `layer.smartFilter.clearSmartFilters` | Clear Smart Filters | Layer › Smart Filter | <code>{"layer":id?}</code> | — |
| `layer.smartFilter.setVisible` | Show/Hide Smart Filter | — | <code>{"layer":id?,"index":u32?,"visible":bool? (default: toggle)}</code> | — |
| `layer.smartFilter.setParams` | Edit Smart Filter | — | <code>{"layer":id?,"index":u32?,"params":{…} (merged)}</code> | — |
| `layer.smartFilter.delete` | Delete Smart Filter | — | <code>{"layer":id?,"index":u32?}</code> | — |
| `layer.smartFilter.move` | Move Smart Filter | — | <code>{"layer":id?,"index":u32?,"to":u32}</code> | — |
| `layer.selectLinkedLayers` | Select Linked Layers | Layer | `{}` | — |
| `layer.align.topEdges` | Top Edges | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.align.verticalCenters` | Vertical Centers | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.align.bottomEdges` | Bottom Edges | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.align.leftEdges` | Left Edges | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.align.horizontalCenters` | Horizontal Centers | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.align.rightEdges` | Right Edges | Layer › Align | <details><summary><code>JSON Schema</code></summary><code>{"to":"auto¦layers¦selection¦canvas"="auto"} (auto: the selection bounds with one layer and an active selection, else the selected layers' bounds)</code></details> | — |
| `layer.distribute.topEdges` | Top Edges | Layer › Distribute | `{}` | — |
| `layer.distribute.verticalCenters` | Vertical Centers | Layer › Distribute | `{}` | — |
| `layer.distribute.bottomEdges` | Bottom Edges | Layer › Distribute | `{}` | — |
| `layer.distribute.leftEdges` | Left Edges | Layer › Distribute | `{}` | — |
| `layer.distribute.horizontalCenters` | Horizontal Centers | Layer › Distribute | `{}` | — |
| `layer.distribute.rightEdges` | Right Edges | Layer › Distribute | `{}` | — |
| `layer.distribute.horizontally` | Horizontally | Layer › Distribute | <code>{} (equal horizontal gaps)</code> | — |
| `layer.distribute.vertically` | Vertically | Layer › Distribute | <code>{} (equal vertical gaps)</code> | — |
| `layer.linkLayers` | Link Layers | Layer | <code>{} (toggles: unlinks when the selection is already one link group)</code> | — |
| `layer.mergeLayers` | Merge Layers | Layer | <code>{} (one layer selected: Merge Down)</code> | `Cmd+E` |
| `layer.new.groupFromLayers` | Group from Layers… | Layer › New | <code>{"name":str?}</code> | — |
| `layer.arrange.reverse` | Reverse | Layer › Arrange | `{}` | — |
| `layer.lockLayers` | Lock Layers… | Layer | <details><summary><code>JSON Schema</code></summary><code>{"transparency":bool?,"pixels":bool?,"position":bool?,"artboard":bool?,"all":bool?} (none given: toggle lock all)</code></details> | — |
| `layer.renameLayer` | Rename Layer | Layer | <code>{"layer":id?,"name":str}</code> | — |
| `layer.copyToDocument` | Copy Layers to Document | — | <details><summary><code>JSON Schema</code></summary><code>{"document":index (destination),"layers":[id,…]?=the source's selected layers,"source":index?=active,"center":bool=false (centre on the destination's canvas),"at":[x,y]? (centre the copies on this point),"offset":[dx,dy]?=[0,0] (from their place in the source)}</code></details> | — |
| `layer.stampVisible` | Stamp Visible | — | <details><summary><code>JSON Schema</code></summary><code>{} → {layer} (every visible layer merged into a new layer above the active one; the originals stay)</code></details> | `Cmd+Alt+Shift+E` |
| `layer.stampDown` | Stamp Down | — | <details><summary><code>JSON Schema</code></summary><code>{} → {layer} (a copy of the active layer merged into the pixel layer below; several selected: into a new layer above them)</code></details> | `Cmd+Alt+E` |
| `layer.layerMask.apply` | Apply | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.layerMask.fromTransparency` | From Transparency | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.layerMask.hideSelection` | Hide Selection | Layer › Layer Mask | <code>{"layer":id?}</code> | — |
| `layer.maskAllObjects` | Mask All Objects | Layer | <code>{"layer":id?}</code> | — |
| `layer.matting.defringe` | Defringe… | Layer › Matting | <code>{"width":1..200=1}</code> | — |
| `layer.matting.removeBlackMatte` | Remove Black Matte | Layer › Matting | `{}` | — |
| `layer.matting.removeWhiteMatte` | Remove White Matte | Layer › Matting | `{}` | — |
| `layer.matting.colorDecontaminate` | Color Decontaminate… | Layer › Matting | <code>{"amount":0..100=100,"radius":1..100=4}</code> | — |
| `layer.smartObjects.stackMode.entropy` | Entropy | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.kurtosis` | Kurtosis | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.maximum` | Maximum | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.mean` | Mean | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.median` | Median | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.minimum` | Minimum | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.range` | Range | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.skewness` | Skewness | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.standardDeviation` | Standard Deviation | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.summation` | Summation | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.variance` | Variance | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.stackMode.none` | None | Layer › Smart Objects › Stack Mode | <code>{"layer":id?}</code> | — |
| `layer.smartObjects.revealInFinder` | Reveal in Finder | Layer › Smart Objects | <code>{"dryRun":bool=false}</code> | — |
| `layer.layerContentOptions` | Layer Content Options… | Layer | `{}` | — |
| `layer.quickExportAsPng` | Quick Export as PNG | Layer | <code>{"layer":id?,"path":text}</code> | — |
| `layer.exportAs` | Export As… | Layer | <code>{"layer":id?,"path":text,"scale":1..1000=100} (format from the path's extension)</code> | — |
| `layer.setLabelColor` | Layer Color | — | <details><summary><code>JSON Schema</code></summary><code>{"color":"none¦red¦orange¦yellow¦green¦seafoam¦blue¦indigo¦magenta¦fuchsia¦violet¦gray","layer":id?} (no layer: selected layers; groups include descendants)</code></details> | — |
| `layer.newFillLayer.pattern` | Pattern… | Layer › New Fill Layer | <details><summary><code>JSON Schema</code></summary><code>{"pattern":id¦name?=first library pattern,"scale":1..1000=100,"angle":deg=0,"link":bool=true,"phase":[x,y]?,"path":"work"¦saved path name¦{…path}? (the active path: becomes the vector mask; else the selection is the layer mask)}</code></details> | — |
| `layer.smartObjects.warp` | Warp | Layer › Smart Objects | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"rect":[x0,y0,x1,y1]? (warp box; default = layer content ∩ selection),"style":"custom¦none¦arc¦arcLower¦arcUpper¦arch¦bulge¦shellLower¦shellUpper¦flag¦wave¦fish¦rise¦fisheye¦inflate¦squeeze¦twist","bend":%=50,"hDistort":%,"vDistort":%,"vertical":bool,"mesh":{"us":[0,…,1],"vs":[0,…,1],"points":[[x,y]…]} ((3c+1)×(3r+1) control points, row-major, document px),"grid":[cols,rows]?,"warp":{full warp object}?,"interpolation":"bicubic¦bilinear¦nearest"}</code></details> | — |
| `layer.new.artboard` | Artboard… | Layer › New | <details><summary><code>JSON Schema</code></summary><code>{"rect":[x,y,w,h]? ¦ "x","y","width","height"? (default: canvas size, right of the last board),"preset":"iPhone 14¦Web 1920¦A4¦…"?,"name":str?,"background":"white¦black¦transparent¦custom"="white","color":[r,g,b]¦"#rrggbb"? (custom)} → {layer, rect}</code></details> | — |
| `layer.new.artboardFromGroup` | Artboard from Group… | Layer › New | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? (a top-level group; default active),"background":…?} → {layer, rect}</code></details> | — |
| `layer.new.artboardFromLayers` | Artboard from Layers… | Layer › New | <code>{"name":str?,"background":…?} (the selected layers) → {layer, rect}</code> | — |
| `layer.artboard.set` | Edit Artboard | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? (default: the active artboard),"x","y","width","height"?¦"rect":[x,y,w,h]?,"preset":str?,"background":"white¦black¦transparent¦custom"?,"color":…?,"name":str?,"moveContents":bool=true} → {layer, rect}</code></details> | — |
| `layer.smartObjects.puppetWarp` | Puppet Warp | Layer › Smart Objects | <details><summary><code>JSON Schema</code></summary><code>{"pins":[{"src":[x,y],"dst":[x,y],"rotate":deg?,"depth":n?}…],"mode":"rigid¦normal¦distort"="normal","density":"fewer¦normal¦more"="normal","expansion":px=2,"interpolation":"bicubic¦bilinear¦nearest","layer":id?} — mesh over the opaque region, as-rigid-as-possible; on a smart object it becomes a smart filter</code></details> | — |
| `layer.smartObjects.perspectiveWarp` | Perspective Warp | Layer › Smart Objects | <details><summary><code>JSON Schema</code></summary><code>{"planes":[{"src":[[x,y]×4],"dst":[[x,y]×4]}…] (corners clockwise from top-left; corners that coincide in src are linked),"straighten":"horizontal¦vertical¦auto"?,"interpolation":"bicubic¦bilinear¦nearest","layer":id?}</code></details> | — |
| `layer.newLayerBasedSlice` | New Layer Based Slice | Layer | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"name":str?,"url":str?,"alt":str?} (follows the layer's bounds with effects) → {slice, rect}</code></details> | — |
| `layer.pickAt` | Auto-Select Layer | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"target":"layer¦group"="layer","select":bool=true,"mode":"replace¦toggle¦add"="replace","list":bool=false (return every layer with pixels there, topmost first)} → {layer} ¦ {layers}</code></details> | — |
| `layer.selectAbove` | Select Layer Above | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+]` |
| `layer.selectBelow` | Select Layer Below | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+[` |
| `layer.selectTop` | Select Top Layer | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+.` |
| `layer.selectBottom` | Select Bottom Layer | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+,` |
| `layer.addAboveToSelection` | Add Layer Above to Selection | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+Shift+]` |
| `layer.addBelowToSelection` | Add Layer Below to Selection | — | <code>{} (follows the Layers panel's rows; collapsed groups are one row)</code> | `Alt+Shift+[` |
| `layer.new.frameFromLayers` | Frame from Layers | Layer › New | <details><summary><code>JSON Schema</code></summary><code>{"name":str?} → {layer, frame:[x,y,w,h]}: groups the selected layers and clips them to a frame rect</code></details> | — |
| `layer.videoLayers.newBlankVideoLayer` | New Blank Video Layer | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.insertBlankFrame` | Insert Blank Frame | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.duplicateFrame` | Duplicate Frame | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.deleteFrame` | Delete Frame | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.restoreFrame` | Restore Frame | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.restoreAllFrames` | Restore All Frames | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.showAlteredVideo` | Show Altered Video | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.rasterize` | Rasterize | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.rasterize.video` | Video | Layer › Rasterize | <code>{} → {}</code> | — |
| `layer.videoLayers.newVideoLayerFromFile` | New Video Layer from File… | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.replaceFootage` | Replace Footage… | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.interpretFootage` | Interpret Footage… | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.videoLayers.reloadFrame` | Reload Frame | Layer › Video Layers | <code>{} → {}</code> | — |
| `layer.setExpanded` | Expand/Collapse Group | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"expanded":bool?,"all":bool?} (no expanded: toggle; all: every group; not an undo step)</code></details> | — |
| `layer.setEffectsExpanded` | Expand/Collapse Effects | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"expanded":bool?,"all":bool?} (no expanded: toggle; all: every layer with effects; view state, not an undo step)</code></details> | — |
| `layer.setEffectsVisible` | Show/Hide Effects | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"index":n?,"visible":bool?} (no index: all of the layer's effects, the "Effects" row; index: the effect at that position in the layer's list; no visible: toggle)</code></details> | — |
| `layer.toggleMaskOverlay` | Toggle Layer Mask Overlay | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} (shows or hides the active layer's mask as a 50% red rubylith over the composite, the '\' key; view state, not an undo step; painting while shown paints the mask)</code></details> | `\` |

## 🔹 Layer Styles & Effects (23)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `layer.layerStyle.dropShadow` | Drop Shadow… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb","opacity":0..100=75,"blend":str="multiply","angle":deg=120,"useGlobalLight":bool,"distance":px=5,"spread":0..100,"size":px=5,"contour":"Linear¦Cone¦Cone (Inverted)¦Domed¦Domed (Inverted)¦Diagonal (Descending)"="Linear","noise":0..100,"knocksOut":bool,"add":bool,"layer":id}</code></details> | — |
| `layer.layerStyle.innerShadow` | Inner Shadow… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb","opacity":0..100=75,"blend":str,"angle":deg,"distance":px,"choke":0..100,"size":px,"contour":name,"noise":0..100,"add":bool}</code></details> | — |
| `layer.layerStyle.outerGlow` | Outer Glow… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb","opacity":0..100=75,"blend":str="screen","technique":"softer¦precise","spread":0..100,"size":px,"range":0..100,"contour":name,"noise":0..100,"add":bool}</code></details> | — |
| `layer.layerStyle.innerGlow` | Inner Glow… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb","opacity":0..100=75,"blend":str="screen","technique":"softer¦precise","source":"edge¦center","choke":0..100,"size":px,"contour":name,"noise":0..100,"add":bool}</code></details> | — |
| `layer.layerStyle.stroke` | Stroke… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"size":px=3,"position":"outside¦inside¦center","color":"#rrggbb","from":"#rrggbb","to":"#rrggbb","style":str,"angle":deg,"opacity":0..100,"blend":str,"add":bool}</code></details> | — |
| `layer.layerStyle.colorOverlay` | Color Overlay… | Layer › Layer Style | <code>{"color":"#rrggbb","opacity":0..100=100,"blend":str,"add":bool}</code> | — |
| `layer.layerStyle.gradientOverlay` | Gradient Overlay… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"from":"#rrggbb","to":"#rrggbb","style":"linear¦radial¦angle¦reflected¦diamond","angle":deg=90,"scale":10..150=100,"reverse":bool,"opacity":0..100,"blend":str,"add":bool}</code></details> | — |
| `layer.layerStyle.patternOverlay` | Pattern Overlay… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"pattern":id¦name?=first library pattern,"opacity":0..100=100,"blend":str,"scale":1..1000=100,"angle":deg=0,"link":bool=true,"phaseX":px,"phaseY":px,"add":bool}</code></details> | — |
| `layer.layerStyle.bevelEmboss` | Bevel & Emboss… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"style":"inner¦outer¦emboss¦pillow¦stroke","technique":"smooth¦chiselHard¦chiselSoft","contour":bool,"contourRange":1..100=50,"texture":pattern id¦name?,"textureScale":1..1000=100,"textureDepth":-1000..1000=100,"textureInvert":bool,"textureLink":bool=true,"depth":1..1000=100,"direction":"up¦down","size":px=5,"soften":px,"angle":deg,"altitude":deg,"add":bool}</code></details> | — |
| `layer.layerStyle.satin` | Satin… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb","opacity":0..100=50,"blend":str,"angle":deg,"distance":px,"size":px,"invert":bool,"add":bool}</code></details> | — |
| `layer.layerStyle.replace` | Edit Layer Style | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id,"effects":[{"kind":"dropShadow¦innerShadow¦outerGlow¦innerGlow¦stroke¦colorOverlay¦gradientOverlay¦patternOverlay¦satin¦bevelEmboss","params":{…layer.layerStyle.<kind> params…},"fx":<effect snapshot carried through the Layer Style dialog, optional>}]} (replaces the layer's whole effect list; an entry with 'fx' edits that effect with the params present, one without builds from params)</code></details> | — |
| `layer.layerStyle.makeDefault` | Make Layer Style Default | — | <details><summary><code>JSON Schema</code></summary><code>{"kind":"stroke¦dropShadow¦innerShadow¦outerGlow¦innerGlow¦colorOverlay¦gradientOverlay¦patternOverlay¦satin¦bevelEmboss","params":{…param set…}} (stores the user default the Layer Style dialog's Reset to Default restores)</code></details> | — |
| `layer.layerStyle.defaultFor` | Layer Style Default | — | <details><summary><code>JSON Schema</code></summary><code>{"kind":str} → {"params":{…},"user":bool} (the saved user default, else the factory defaults)</code></details> | — |
| `layer.layerStyle.clear` | Clear Layer Style | Layer › Layer Style | <code>{"layer":id}</code> | — |
| `layer.layerStyle.copyLayerStyle` | Copy Layer Style | Layer › Layer Style | <code>{"layer":id?}</code> | — |
| `layer.layerStyle.pasteLayerStyle` | Paste Layer Style | Layer › Layer Style | <code>{"layer":id?}</code> | — |
| `layer.layerStyle.hideAllEffects` | Hide All Effects | Layer › Layer Style | `{}` | — |
| `layer.layerStyle.showAllEffects` | Show All Effects | — | `{}` | — |
| `layer.layerStyle.blendingOptions` | Blending Options… | Layer › Layer Style | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"blend":"normal¦multiply¦…"?,"opacity":0..100?,"fillOpacity":0..100?,"blendIf":{"channel":"gray¦red¦green¦blue¦cyan¦…"¦index="gray","thisLayer":[black,white]¦[blackLo,blackHi,whiteLo,whiteHi]?,"underlying":[…]?}¦[{…},…]¦null?,"channels":[bool,…]?,"knockout":"none¦shallow¦deep"?,"blendInteriorEffectsAsGroup":bool?,"blendClippedLayersAsGroup":bool?,"transparencyShapesLayer":bool?,"layerMaskHidesEffects":bool?,"vectorMaskHidesEffects":bool?} (Blend If values 0..255; split points fade; null resets. Advanced Blending: channels = which colour channels blend (R G B / C M Y K / L a b); PhotoCraft's defaults are knockout none, interior effects off, clipped layers as group on, transparency shapes on, masks hide effects off)</code></details> | — |
| `layer.layerStyle.globalLight` | Global Light… | Layer › Layer Style | <code>{"angle":-180..180=120,"altitude":0..90=30}</code> | — |
| `layer.layerStyle.createLayer` | Create Layer | Layer › Layer Style | <code>{"layer":id?}</code> | — |
| `layer.layerStyle.scaleEffects` | Scale Effects… | Layer › Layer Style | <code>{"scale":1..1000=100}</code> | — |
| `layer.layerStyle.transferEffects` | Move Layer Effects | — | <code>{"from":id,"to":id,"effect":index? (default: all effects),"copy":bool=false}</code> | — |

## 🔹 LayerComp (13)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `layerComp.new` | New Layer Comp… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str="Layer Comp N","comment":str="","visibility":bool=true,"position":bool=true,"appearance":bool=true} → {comp}</code></details> | — |
| `layerComp.update` | Update Layer Comp | — | <details><summary><code>JSON Schema</code></summary><code>{"comp":id¦name¦"*"? (default: last applied; "*" = all),"what":"all¦visibility¦position¦appearance"="all"}</code></details> | — |
| `layerComp.apply` | Apply Layer Comp | — | <code>{"comp":id¦name? (default: last applied)} → {comp, missingLayers}</code> | — |
| `layerComp.previous` | Apply Previous Layer Comp | — | `{}` | — |
| `layerComp.next` | Apply Next Layer Comp | — | `{}` | — |
| `layerComp.delete` | Delete Layer Comp | — | <code>{"comp":id¦name?}</code> | — |
| `layerComp.duplicate` | Duplicate Layer Comp | — | <code>{"comp":id¦name?} → {comp}</code> | — |
| `layerComp.rename` | Rename Layer Comp | — | <code>{"comp":id¦name?,"name":str}</code> | — |
| `layerComp.setComment` | Layer Comp Comment | — | <code>{"comp":id¦name?,"comment":str}</code> | — |
| `layerComp.setOptions` | Layer Comp Options… | — | <details><summary><code>JSON Schema</code></summary><code>{"comp":id¦name?,"visibility":bool?,"position":bool?,"appearance":bool?,"name":str?,"comment":str?}</code></details> | — |
| `layerComp.restoreLastDocumentState` | Restore Last Document State | — | `{}` | — |
| `layerComp.updateWarnings` | Layer Comp Warnings | — | <details><summary><code>JSON Schema</code></summary><code>{"clear":bool=false (drop states of deleted layers)} → {warnings:[{comp,name,missingLayers}],count}</code></details> | — |
| `layerComp.list` | List Layer Comps | — | <details><summary><code>JSON Schema</code></summary><code>{} → {comps:[{id,name,comment,visibility,position,appearance,layers,missingLayers}],lastApplied,hasLastDocumentState}</code></details> | — |

## 🔹 MeasurementLog (3)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `measurementLog.list` | Measurement Log | — | <code>{} → rows and columns</code> | — |
| `measurementLog.delete` | Delete Measurements | — | <code>{"rows":[ids]}¦{"all":true}</code> | — |
| `measurementLog.export` | Export Measurements… | — | <code>{"path":str? (CSV file; omitted → returns the CSV text),"rows":[ids]?}</code> | — |

## 🔹 Notes (4)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `notes.add` | New Note | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"text":str="","author":str="","color":"#rrggbb"¦[r,g,b]=pale yellow,"open":bool=true}</code></details> | — |
| `notes.set` | Edit Note | — | <details><summary><code>JSON Schema</code></summary><code>{"index":n,"text":str?,"author":str?,"color":"#rrggbb"¦[r,g,b]?,"x":px?,"y":px?,"open":bool?}</code></details> | — |
| `notes.delete` | Delete Note | — | <code>{"index":n}¦{"all":true}</code> | — |
| `notes.list` | Notes | — | <code>{} → notes (index, author, text, colour, position, open, modified)</code> | — |

## 🔹 Paint (25)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `paint.stroke` | Brush Stroke | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?,tiltX?,tiltY?,rotation?,timeMs?,wheel?],…],"brush":{…BrushSettings}?,"preset":name?,"size":0.5..5000 px?,"hardness":0..1?,"opacity":0..1?,"flow":0..1?,"spacing":0..10?,"color":"#rrggbb"?=foreground,"mode":"normal¦multiply¦screen¦…"="normal","erase":bool?,"smoothing":0..1?,"zoom":number=1,"seed":u64?,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target}</code></details> | — |
| `paint.setSymmetry` | Set Painting Symmetry | — | <details><summary><code>JSON Schema</code></summary><code>{"mode":"vertical¦horizontal¦dual¦diagonal","center":[x,y]?,"rotation":degrees=0}</code></details> | — |
| `paint.symmetryFromPath` | Make Symmetry Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":"work"¦"layer"¦savedPathName="work"} — mirror Brush, Pencil and Eraser strokes across the path</code></details> | — |
| `paint.symmetryDisable` | Disable Painting Symmetry | — | `{}` | — |
| `paint.pencil` | Pencil | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?,tiltX?,tiltY?,rotation?,timeMs?,wheel?],…],"brush":{…}?,"preset":name?,"size":0.5..5000 px?,"opacity":0..1?,"color":"#rrggbb"?=foreground,"mode":"normal¦multiply¦screen¦…"="normal","erase":bool?,"autoErase":bool=false,"seed":u64?,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target}</code></details> | — |
| `paint.mixerBrush` | Mixer Brush | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[…],"brush":{…}?,"preset":name?,"size":0.5..5000 px?,"wet":0..100=brush.mixer.wet,"load":0..100=brush.mixer.load,"mix":0..100=brush.mixer.mix,"flow":0..100=brush.mixer.flow,"color":"#rrggbb"?=foreground,"sampleAllLayers":bool=brush.mixer.sampleAllLayers,"cleanAfterStroke":bool=true,"loadAfterStroke":bool=true,"seed":u64?}</code></details> | — |
| `paint.colorReplacement` | Color Replacement | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[…],"brush":{…}?,"size":px?,"mode":"hue¦saturation¦color¦luminosity"="color","sampling":"continuous¦once¦backgroundSwatch"="continuous","limits":"contiguous¦discontiguous¦findEdges"="contiguous","tolerance":0..100=30,"antiAlias":bool=true,"color":"#rrggbb"?=foreground,"seed":u64?}</code></details> | — |
| `paint.magicEraser` | Magic Eraser | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"tolerance":0..255=32,"antiAlias":bool=true,"contiguous":bool=true,"sampleAllLayers":bool=false,"opacity":1..100=100}</code></details> | — |
| `paint.backgroundEraser` | Background Eraser | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":px?,"hardness":0..1?,"brush":{…}?,"preset":name?,"sampling":"continuous¦once¦backgroundSwatch"="continuous","limits":"discontiguous¦contiguous¦findEdges"="contiguous","tolerance":0..100=50,"protectForegroundColor":bool=false,"seed":u64?}</code></details> | — |
| `paint.cloneStamp` | Clone Stamp | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"source":[sx,sy] (sampled under the first point) ¦ "offset":[dx,dy],"aligned":bool=true,"sampleLayer":"current¦currentAndBelow¦all"="current","mode":"normal¦multiply¦…"="normal" → {"damage","offset","aligned","nextSource"}}</code></details> | — |
| `paint.patternStamp` | Pattern Stamp | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"pattern":id¦name?=Patterns panel current,"scale":1..1000 %=100,"angle":deg=0,"aligned":bool=true,"impressionist":bool=false,"mode":"normal¦multiply¦…"="normal","phase":[px,py]? → {"damage","phase","aligned","pattern"}}</code></details> | — |
| `paint.healingBrush` | Healing Brush | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"source":[sx,sy] ¦ "offset":[dx,dy],"aligned":bool=true,"sampleLayer":"current¦currentAndBelow¦all"="current","mode":"normal¦…"="normal" → {"damage","offset","aligned","nextSource"}}</code></details> | — |
| `paint.spotHealing` | Spot Healing Brush | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"type":"contentAware¦createTexture¦proximityMatch"="contentAware","sampleAllLayers":bool=false}</code></details> | — |
| `paint.patch` | Patch | — | <details><summary><code>JSON Schema</code></summary><code>{"offset":[dx,dy] (how far the selection was dragged),"mode":"source¦destination"="source","layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target} → {"damage","offset"}</code></details> | — |
| `paint.contentAwareMove` | Content-Aware Move | — | <details><summary><code>JSON Schema</code></summary><code>{"offset":[dx,dy] (how far the selection was dragged),"mode":"move¦extend"="move","structure":1..7=4 (7 keeps the content up to its edge, lower blends a wider edge band),"color":0..10=0 (how far the content's colour adapts to its new place),"sampleAllLayers":bool=false,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target} → {"damage","offset","mode"} (a background job; the selection moves with the content)</code></details> | — |
| `paint.dodge` | Dodge | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"range":"shadows¦midtones¦highlights"="midtones","exposure":1..100=50,"protectTones":bool=true}</code></details> | — |
| `paint.burn` | Burn | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"range":"shadows¦midtones¦highlights"="midtones","exposure":1..100=50,"protectTones":bool=true}</code></details> | — |
| `paint.sponge` | Sponge | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"mode":"desaturate¦saturate"="desaturate","vibrance":bool=true (flow = sponge strength)}</code></details> | — |
| `paint.blur` | Blur | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"strength":1..100=50,"sampleAllLayers":bool=false}</code></details> | — |
| `paint.sharpen` | Sharpen | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"strength":1..100=50,"protectDetail":bool=true,"sampleAllLayers":bool=false}</code></details> | — |
| `paint.smudge` | Smudge | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"strength":1..100=50,"fingerPainting":bool=false (starts with the foreground colour),"sampleAllLayers":bool=false}</code></details> | — |
| `paint.historyBrush` | History Brush | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y,pressure?],…],"size":1..5000 px=tool size,"hardness":0..100=tool hardness,"opacity":1..100=100,"flow":1..100=100,"spacing":1..1000 (% of size)=25,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target,"state":index (into history.entries; default 0 = the oldest held state, normally the Open snapshot)}</code></details> | — |
| `paint.redEye` | Red Eye | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px, "y":px} ¦ {"points":[[x,y],…] (at most 64)},"pupilSize":1..100=50,"darken":0..100=50,"layer":id?=active,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i} → {"damage":[x,y,w,h], "pixels":n}</code></details> | — |
| `paint.bucket` | Paint Bucket | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"tolerance":0..255=32,"contiguous":bool=true,"antiAlias":bool=true,"sampleAllLayers":bool=false,"contents":"foreground¦pattern"="foreground","color":"#rrggbb"=foreground,"pattern":id¦name (contents=pattern),"scale":%=100,"angle":deg,"opacity":1..100=100,"target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target}</code></details> | — |
| `paint.gradient` | Gradient | — | <details><summary><code>JSON Schema</code></summary><code>{"from":[x,y],"to":[x,y],"style":"linear¦radial¦angle¦reflected¦diamond"="linear","colors":["#rrggbb",…]? (evenly spaced),"gradient":preset name?,"stops":[[t,"#rrggbb"¦"foreground"¦"background"],…]?,"transparency":[[t,0..100],…]? (default: the current gradient, see gradient.presets.select),"reverse":bool=false,"dither":bool=true,"opacity":1..100=100,"mode":"normal¦multiply¦…"="normal","target":"pixels"¦"mask"¦"quickMask"¦{"channel":i}=Channels panel target}</code></details> | — |

## 🔹 Path (19)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `path.list` | List Paths | — | <details><summary><code>JSON Schema</code></summary><code>{} → {paths:[{name,knots,subpaths}],workPath,clippingPath,layerPath (active layer's shape path / vector mask)}</code></details> | — |
| `path.info` | Path Info | — | <code>{"name":str¦"work"¦"layer"="work"} → {name,path}</code> | — |
| `path.set` | Set Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"="work","path":{…},"op":"combine¦subtract¦intersect¦exclude"? (append to the existing path with this op instead of replacing)}. path: {"subpaths":[{"closed":bool=true,"op":"combine¦subtract¦intersect¦exclude","knots":[[x,y] ¦ {"anchor":[x,y],"in":[x,y],"out":[x,y],"smooth":bool}]}],"fillRule":"nonzero¦evenodd","inverted":bool}</code></details> | — |
| `path.delete` | Delete Path | — | <code>{"name":str¦"work"="work"}</code> | — |
| `path.transform` | Free Transform Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (with layer target),"matrix":[a,b,c,d,e,f] ¦ "translateX":px?,"translateY":px?,"scaleX":factor?,"scaleY":factor?,"angle":degrees?} (numeric transforms pivot on path bounds center)</code></details> | — |
| `path.clippingPath.set` | Clipping Path | — | <code>{"name":savedPathName,"flatness":0..100=0} (PSD export clipping path)</code> | — |
| `path.clippingPath.clear` | Clear Clipping Path | — | `{}` | — |
| `path.style.copyFill` | Copy Fill | — | <code>{} (copies active shape layer fill)</code> | — |
| `path.style.copyStroke` | Copy Complete Stroke | — | <code>{} (copies active shape layer stroke)</code> | — |
| `path.style.pasteFill` | Paste Fill | — | <code>{"layer":id?} (pastes copied fill onto active shape layer)</code> | — |
| `path.style.pasteStroke` | Paste Complete Stroke | — | <code>{"layer":id?} (pastes copied stroke onto active shape layer)</code> | — |
| `path.rename` | Rename Path | — | <code>{"name":str¦"work"="work","to":str} (renaming the work path saves it)</code> | — |
| `path.toSelection` | Make Selection from Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","feather":px=0,"antiAlias":bool=true,"mode":"replace¦add¦subtract¦intersect"="replace"}</code></details> | `Cmd+Enter` |
| `path.fill` | Fill Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (pixel layer, default active),"color":"#rrggbb"=foreground,"opacity":0..100=100,"mode":"normal¦multiply¦…"="normal","feather":px=0,"antiAlias":bool=true}</code></details> | — |
| `path.stroke` | Stroke Path | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (pixel layer),"tool":"brush¦pencil¦eraser"="brush","size":0.5..5000 px?,"hardness":0..1?,"opacity":0..100?,"color":"#rrggbb"=foreground} (current brush settings otherwise)</code></details> | — |
| `path.moveAnchors` | Drag Anchor Points | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (with "layer": a shape layer's path or a vector mask),"anchors":[[subpath,knot],…],"move":[dx,dy] (anchors move with their handles; Direct Selection drag)} → {name,path}</code></details> | — |
| `path.moveHandle` | Drag Direction Point | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (with "layer": a shape layer's path or a vector mask),"subpath":i,"knot":j,"handle":"in¦out","to":[x,y],"independent":bool=false (a smooth point turns its other handle to stay collinear; independent = ⌥ / Convert Point, makes a corner)} → {name,path}</code></details> | — |
| `path.bendSegment` | Drag Segment | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (with "layer": a shape layer's path or a vector mask),"subpath":i,"knot":j (the segment leaving it),"t":0..1=0.5 (where it was grabbed),"move":[dx,dy] (a straight segment moves with its anchors; a curve reshapes through the dragged point)} → {name,path}</code></details> | — |
| `path.convertPoint` | Convert Point | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str¦"work"¦"layer"="work","layer":id? (with "layer": a shape layer's path or a vector mask),"subpath":i,"knot":j,"out":[x,y]? (none: corner with retracted handles; given: smooth with this out handle and its mirror)} → {name,path}</code></details> | — |

## 🔹 Pattern (10)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `pattern.list` | Patterns | — | <code>{} → [{id,name,width,height,mode,depth,inDocument,inLibrary}]</code> | — |
| `pattern.rename` | Rename Pattern | — | <code>{"pattern":id¦name,"name":str}</code> | — |
| `pattern.delete` | Delete Pattern | — | <code>{"pattern":id¦name} (library only; documents keep their copy)</code> | — |
| `pattern.import` | Import Patterns… | — | <code>{"path":".pat file"}</code> | — |
| `pattern.export` | Export Patterns… | — | <code>{"path":".pat file","patterns":[id¦name]? (default: the whole library)}</code> | — |
| `pattern.presets.list` | Pattern Presets | — | <code>{} → {groups:[{name,patterns:[{id,name,width,height}]}],current}</code> | — |
| `pattern.presets.select` | Select Pattern | — | <details><summary><code>JSON Schema</code></summary><code>{"pattern":id¦name,"applyToLayer":bool=true (also changes a selected Pattern Fill layer)} (Fill and new Pattern Fill layers default to it)</code></details> | — |
| `pattern.presets.apply` | New Pattern Fill Layer from Preset | — | <code>{"pattern":id¦name?=selected,"scale":1..1000=100,"angle":deg=0} → {layer}</code> | — |
| `pattern.presets.new` | New Pattern Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"group":name?,"rect":[x0,y0,x1,y1]?} (Define Pattern from the selection/canvas into a group)</code></details> | — |
| `pattern.presets.edit` | Edit Pattern Presets | — | <details><summary><code>JSON Schema</code></summary><code>{"action":"rename¦delete¦move¦newGroup¦renameGroup¦deleteGroup","preset":name¦[names] (rename/delete/move),"group":name? (narrows the lookup; the group for renameGroup/deleteGroup),"name":str (rename: new name; newGroup/renameGroup: group name),"to":group (move),"index":n? (move)}</code></details> | — |

## 🔹 Plugin (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `plugin.list` | List Plug-ins | — | `{}` | — |
| `plugin.install` | Install Plug-in… | Filter › Plug-ins | <details><summary><code>JSON Schema</code></summary><code>{"path":text,"data":json,"replace":bool=true} (path: a .wasm file, native only; data: the module as base64)</code></details> | — |
| `plugin.remove` | Remove Plug-in | — | <code>{"id":text}</code> | — |
| `plugin.reload` | Reload Plug-ins | — | <details><summary><code>JSON Schema</code></summary><code>{"path":text} (a folder of .wasm plug-ins; default: the Plug-ins preference folder)</code></details> | — |
| `plugin.run` | Run Plug-in | — | <details><summary><code>JSON Schema</code></summary><code>{"id":text,"params":json} (plug-in parameters under "params" or at the top level; see plugin.list)</code></details> | — |

## 🔹 Prefs (3)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `prefs.get` | Get Preferences | — | <details><summary><code>JSON Schema</code></summary><code>{"path":"section.key"?=everything (e.g. "performance.historyStates", "colorSettings.workingRgb")}</code></details> | — |
| `prefs.set` | Set Preferences | — | <details><summary><code>JSON Schema</code></summary><code>{"path":"section.key","value":json} or {"values":{"section.key":json,…}} (validated; all or nothing)</code></details> | — |
| `prefs.reset` | Reset Preferences | — | <code>{"path":"section¦section.key"?=everything}</code> | — |

## 🔹 Selections (34)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `select.all` | All | Select | `{}` | `Cmd+A` |
| `select.deselect` | Deselect | Select | `{}` | `Cmd+D` |
| `select.inverse` | Inverse | Select | `{}` | `Cmd+Shift+I` |
| `select.rect` | Rectangular Selection | — | <details><summary><code>JSON Schema</code></summary><code>{"x":i32,"y":i32,"width":u32,"height":u32,"mode":"replace¦add¦subtract¦intersect"="replace","ellipse":bool=false,"antiAlias":bool=true,"feather":px=0}</code></details> | — |
| `select.float` | Float Selection | — | <details><summary><code>JSON Schema</code></summary><code>{"dx":px=0,"dy":px=0,"copy":bool=false} → {layer, offset} (cuts the selected pixels of the active layer into a floating piece the first time, or with copy leaves them and floats a duplicate; on the Background the hole takes the background colour; then moves it by whole pixels; dropped by select.drop or any other command, put back by edit.undo)</code></details> | — |
| `select.drop` | Drop Floating Selection | — | <details><summary><code>JSON Schema</code></summary><code>{} → {dropped, offset} (drops the floating piece into its layer and moves the selection with it: one history step)</code></details> | — |
| `select.toWorkPath` | Make Work Path | — | <code>{"tolerance":0.5..10 px=2} → {subpaths,knots}</code> | — |
| `select.convertToShape` | Convert Selection to Shape | — | <code>{"tolerance":px=2,"fill":"#rrggbb"=foreground,"name":str?} → shape.info</code> | — |
| `select.quick` | Quick Selection | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y],…],"size":px=30,"mode":"add¦subtract¦replace"="add","sampleAllLayers":bool=false,"enhanceEdge":bool=false}</code></details> | — |
| `select.object` | Object Selection | — | <details><summary><code>JSON Schema</code></summary><code>{"rect":[x,y,w,h],"mode":"replace¦add¦subtract¦intersect"="replace","sampleAllLayers":bool=false}</code></details> | — |
| `select.subject` | Subject | Select | <code>{"sampleAllLayers":bool=true,"mode":"replace¦add¦subtract¦intersect"="replace"}</code> | — |
| `select.refineEdge` | Refine Edge | — | <details><summary><code>JSON Schema</code></summary><code>{"radius":px=0,"smartRadius":bool=false,"smooth":0..100=0,"feather":px=0,"contrast":0..100=0,"shiftEdge":-100..100=0,"decontaminate":bool=false,"amount":0..100=100,"output":"selection¦layerMask¦newLayer¦newLayerWithMask"="selection","sampleAllLayers":bool=false}</code></details> | — |
| `select.focusArea` | Focus Area… | Select | <details><summary><code>JSON Schema</code></summary><code>{"range":0..1=0.5,"noise":0..1=0,"sampleAllLayers":bool=true,"mode":"replace¦add¦subtract¦intersect"="replace"}</code></details> | — |
| `select.magicWand` | Magic Wand | — | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px,"tolerance":0..255=32,"contiguous":bool=true,"antiAlias":bool=true,"sampleAllLayers":bool=false,"mode":"replace¦add¦subtract¦intersect"="replace"}</code></details> | — |
| `select.colorRange` | Color Range… | Select | <details><summary><code>JSON Schema</code></summary><code>{"select":"sampledColors¦reds¦yellows¦greens¦cyans¦blues¦magentas¦highlights¦midtones¦shadows¦outOfGamut"="sampledColors","color":"#rrggbb"=foreground,"colors":["#rrggbb",…]?,"points":[[x,y],…]? (eyedropper samples),"subtractPoints":[[x,y],…]? (minus eyedropper),"order":["+"¦"-",…]? (click order of points and subtractPoints; default all points first),"fuzziness":0..200=40 (tones: 0..100 %=20),"localized":bool=false,"range":0..100=100 (% of the longer side),"tonalRange":level¦[lo,hi] (shadows 65, highlights 190, midtones [105,150]),"invert":bool=false,"sampleAllLayers":bool=true,"mode":"replace¦add¦subtract¦intersect"="replace"}</code></details> | — |
| `select.modify.border` | Border… | Select › Modify | <code>{"radius":1..200=1}</code> | — |
| `select.modify.smooth` | Smooth… | Select › Modify | <code>{"radius":1..500=1,"applyAtCanvasBounds":bool=false}</code> | — |
| `select.modify.expand` | Expand… | Select › Modify | <code>{"radius":1..500=1,"applyAtCanvasBounds":bool=false}</code> | — |
| `select.modify.contract` | Contract… | Select › Modify | <code>{"radius":1..500=1,"applyAtCanvasBounds":bool=false}</code> | — |
| `select.modify.feather` | Feather… | Select › Modify | <code>{"radius":0.1..1000=1,"applyAtCanvasBounds":bool=false}</code> | — |
| `select.grow` | Grow | Select | <code>{"tolerance":0..255=32,"sampleAllLayers":bool=false}</code> | — |
| `select.similar` | Similar | Select | <code>{"tolerance":0..255=32,"sampleAllLayers":bool=false}</code> | — |
| `select.lasso` | Lasso | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y],…],"mode":"replace¦add¦subtract¦intersect"="replace","antiAlias":bool=true,"feather":px=0}</code></details> | — |
| `select.magneticLasso` | Magnetic Lasso | — | <details><summary><code>JSON Schema</code></summary><code>{"points":[[x,y],…] (≥3 fastening points in order; with trace=false, the finished outline),"width":1..256=10 (px: follows edges this close to the points),"contrast":1..100=10 (%: weaker edges are ignored),"close":"magnetic¦straight"="magnetic" (how the last point joins the first),"trace":bool=true,"mode":"replace¦add¦subtract¦intersect"="replace","antiAlias":bool=true,"feather":0..1000=0 (px)}</code></details> | — |
| `select.sky` | Sky | Select | <details><summary><code>JSON Schema</code></summary><code>{"mode":"replace¦add¦subtract¦intersect"="replace","sampleAllLayers":bool=true,"threshold":0..100=50 (higher = stricter),"softness":0..100=50 (edge softness)} → {selected, changed, coverage (0–1 of the canvas), bounds [x,y,w,h]}</code></details> | — |
| `select.isolateLayers` | Isolate Layers | Select | <details><summary><code>JSON Schema</code></summary><code>{"on":bool? (default: toggle)} — Layers panel lists only the selected layers → {isolated, layers}</code></details> | — |
| `select.transformSelection` | Transform Selection | Select | <details><summary><code>JSON Schema</code></summary><code>{"rect":[x0,y0,x1,y1]? (frame; default = selection bounds),"quad"¦"corners":[[x,y]×4]? (where the frame's corners go: distort/perspective),"matrix":[a,b,c,d,e,f]?,"scaleX":%=100,"scaleY":%=100,"rotate":deg=0,"skewX":deg=0,"skewY":deg=0,"dx":px=0,"dy":px=0,"reference":"center¦topLeft¦top¦topRight¦left¦right¦bottomLeft¦bottom¦bottomRight"¦[x,y]="center","style"¦"mesh"¦"grid"¦"warp":… (warp, as edit.transform.warp),"interpolation":"bilinear¦bicubic¦nearest"="bilinear"}</code></details> | — |
| `select.reselect` | Reselect | Select | `{}` | `Cmd+Shift+D` |
| `select.allLayers` | All Layers | Select | `{}` | `Cmd+Alt+A` |
| `select.deselectLayers` | Deselect Layers | Select | `{}` | — |
| `select.findLayers` | Find Layers | Select | <code>{"name":str (case-insensitive substring)}</code> | `Cmd+Alt+Shift+F` |
| `select.saveSelection` | Save Selection… | Select | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"channel":"new"¦index¦name="new","document":index?,"operation":"new¦replace¦add¦subtract¦intersect"="new"}</code></details> | — |
| `select.loadSelection` | Load Selection… | Select | <details><summary><code>JSON Schema</code></summary><code>{"channel":index¦name¦"composite"¦"red¦green¦blue¦…"¦"transparency"¦"mask"¦"vectorMask"¦"quickMask"¦"selection","layer":id?,"document":index?,"invert":bool=false,"operation":"new¦add¦subtract¦intersect"="new"}</code></details> | — |
| `select.editInQuickMaskMode` | Edit in Quick Mask Mode | Select | <code>{"on":bool?=toggle}</code> | `Q` |

## 🔹 Shape (9)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `shape.create` | New Shape Layer | Layer › New | <details><summary><code>JSON Schema</code></summary><code>{"kind":"rect¦roundedRect¦ellipse¦polygon¦star¦line¦path"="rect","rect":[x,y,w,h] (rect/roundedRect/ellipse/polygon/star),"radii":[tl,tr,br,bl]¦r (roundedRect=10),"sides":3..100=5,"starRatio":0..1 (star=0.5),"from":[x,y],"to":[x,y],"weight":px=1 (line),"path":{…} (kind path),"fill":"#rrggbb"¦[r,g,b,a]¦{"gradient":{"stops":[[t,"#hex"]],"angle":deg,"scale":%,"style":"linear¦radial¦angle¦reflected¦diamond","reverse":bool}}¦{"pattern":name}¦null (no fill)=foreground,"stroke":{"width":px,"color":"#rrggbb"¦fill,"opacity":0..100,"align":"inside¦center¦outside","cap":"butt¦round¦square","join":"miter¦round¦bevel","miterLimit":n,"dashes":[multiples of width],"dashOffset":n}¦null=none,"name":str?,"addTo":layerId? + "op":"combine¦subtract¦intersect¦exclude" (append to an existing shape layer)} → shape.info. path: {"subpaths":[{"closed":bool=true,"op":"combine¦subtract¦intersect¦exclude","knots":[[x,y] ¦ {"anchor":[x,y],"in":[x,y],"out":[x,y],"smooth":bool}]}],"fillRule":"nonzero¦evenodd","inverted":bool}</code></details> | — |
| `shape.edit` | Edit Shape | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"path":{…}? (replaces; drops live shape),"kind":str?,"rect":[x,y,w,h]?,"radii":[tl,tr,br,bl]¦r?,"sides":n?,"starRatio":r?,"from":[x,y]?,"to":[x,y]?,"weight":px? (live-shape params regenerate the path),"move":[dx,dy]?,"transform":[a,b,c,d,e,f]?,"op":"combine¦…"? + "subpath":index? (default all but the first),"fillRule":"nonzero¦evenodd"?,"fill":"#rrggbb"¦[r,g,b,a]¦{"gradient":{"stops":[[t,"#hex"]],"angle":deg,"scale":%,"style":"linear¦radial¦angle¦reflected¦diamond","reverse":bool}}¦{"pattern":name}¦null (no fill)?,"stroke":{"width":px,"color":"#rrggbb"¦fill,"opacity":0..100,"align":"inside¦center¦outside","cap":"butt¦round¦square","join":"miter¦round¦bevel","miterLimit":n,"dashes":[multiples of width],"dashOffset":n}¦null? (merged into the current stroke),"name":str?} → shape.info</code></details> | — |
| `shape.info` | Shape Layer Info | — | <code>{"layer":id?} → {layer,name,kind,path,fill,stroke,live,bounds:[x,y,w,h]}</code> | — |
| `shape.rasterize` | Rasterize Shape | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `shape.presets.list` | Shape Presets | — | <code>{} → {groups:[{name,shapes:[{name,subpaths}]}]}</code> | — |
| `shape.presets.place` | Place Custom Shape | — | <details><summary><code>JSON Schema</code></summary><code>{"preset":name,"group":name?,"rect":[x,y,w,h]? (default: centred, half the canvas),"keepAspect":bool=true,"fill":…?=foreground,"stroke":…?,"name":str?,"addTo":layerId?,"op":"combine¦subtract¦intersect¦exclude"?} → shape.info (Custom Shape tool / Shapes panel; fill and stroke as in shape.create)</code></details> | — |
| `shape.presets.new` | New Shape Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str="Shape","group":name?,"path":{…path}¦"M x y L …" (preset path language)¦"work"¦saved path name? (default: work path, active shape layer or vector mask)}</code></details> | — |
| `shape.presets.edit` | Edit Shape Presets | — | <details><summary><code>JSON Schema</code></summary><code>{"action":"rename¦delete¦move¦newGroup¦renameGroup¦deleteGroup","preset":name¦[names] (rename/delete/move),"group":name? (narrows the lookup; the group for renameGroup/deleteGroup),"name":str (rename: new name; newGroup/renameGroup: group name),"to":group (move),"index":n? (move)}</code></details> | — |
| `shape.presets.reset` | Restore Default Shapes | — | <code>{"append":bool=false}</code> | — |

## 🔹 Slice (7)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `slice.new` | Slice Tool | — | <details><summary><code>JSON Schema</code></summary><code>{"rect":[x,y,w,h] ¦ "x","y","width","height", plus Slice Options ("name","kind":"image¦noImage¦table","url","target","message","alt","cellText","cellTextIsHtml","background":"none¦#rrggbb")?} → {slice, number}</code></details> | — |
| `slice.fromGuides` | Slices From Guides | — | <code>{} (replaces every slice by the grid of the canvas guides) → {slices}</code> | — |
| `slice.set` | Slice Options… | — | <details><summary><code>JSON Schema</code></summary><code>{"slice":id ¦ "number":n (an auto slice is promoted), "name"?,"kind":"image¦noImage¦table"?,"url"?,"target"?,"message"?,"alt"?,"cellText"?,"cellTextIsHtml"?,"horizontalAlign":0..4?,"verticalAlign":0..4?,"background":"none¦#rrggbb"?,"outsets":[t,l,b,r]? (layer slices),"rect":[x,y,w,h]? (move/resize; a layer slice becomes a user slice)} → the slice</code></details> | — |
| `slice.promote` | Promote | — | <code>{"slice":id ¦ "number":n} (auto or layer-based → user slice) → {slice}</code> | — |
| `slice.delete` | Delete Slice | — | <code>{"slice":id ¦ "number":n ¦ "slices":[id…]} → {deleted}</code> | — |
| `slice.divide` | Divide Slice… | — | <details><summary><code>JSON Schema</code></summary><code>{"slice":id ¦ "number":n,"horizontal":n=1 (slices down),"vertical":n=1 (slices across); at most 10000 parts} → {slices}</code></details> | — |
| `slice.list` | List Slices | — | <details><summary><code>JSON Schema</code></summary><code>{} → {slices:[{number,id,origin:auto¦layer¦user,name,rect:[x,y,w,h],kind,layer,url,alt,…}], locked}</code></details> | — |

## 🔹 Style (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `style.presets.list` | Style Presets | — | <code>{} → {groups:[{name,presets:[{name,effects,blend,fillOpacity}]}]}</code> | — |
| `style.presets.apply` | Apply Style | — | <details><summary><code>JSON Schema</code></summary><code>{"preset":name,"group":name?,"layers":[ids]?¦"layer":id? (default: the selected layers),"add":bool=false (⇧-click: add to the existing effects)} → {layers,style}</code></details> | — |
| `style.presets.new` | New Style… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str="Style","group":name?,"layer":id?,"includeEffects":bool=true,"includeBlending":bool=true,"effects":[[kind, params]]? (explicit list, e.g. the Layer Style dialog's pending state; overrides includeEffects),"blend":str?,"fillOpacity":0..100? (override the layer's blending options)}</code></details> | — |
| `style.presets.edit` | Edit Style Presets | — | <details><summary><code>JSON Schema</code></summary><code>{"action":"rename¦delete¦move¦newGroup¦renameGroup¦deleteGroup","preset":name¦[names] (rename/delete/move),"group":name? (narrows the lookup; the group for renameGroup/deleteGroup),"name":str (rename: new name; newGroup/renameGroup: group name),"to":group (move),"index":n? (move)}</code></details> | — |
| `style.presets.reset` | Restore Default Styles | — | <code>{"append":bool=false}</code> | — |

## 🔹 Swatches (12)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `swatches.list` | Swatches | — | <details><summary><code>JSON Schema</code></summary><code>{} → {groups:[{name,swatches:[{name,model,values,hex,kind?}]}],count}. values are in Color Picker units; hex is the colour in the RGB working space.</code></details> | — |
| `swatches.add` | New Swatch… | — | <details><summary><code>JSON Schema</code></summary><code>{"color":"#rrggbb"¦{"model":"rgb¦hsb¦cmyk¦lab¦gray","values":[rgb 0..255 ¦ hue°,s%,b% ¦ c%,m%,y%,k% ¦ L,a,b ¦ k%]}?=foreground,"name":str?="Color Swatch N","group":name?=first (created when missing),"index":n?=end,"kind":"process¦global¦spot"="process"} → {name,group,index}</code></details> | — |
| `swatches.delete` | Delete Swatch | — | <details><summary><code>JSON Schema</code></summary><code>{"swatch":name (or "index":n across all groups),"group":name? (narrows the name lookup); "swatch" may be a list of names} → {deleted:[names]}</code></details> | — |
| `swatches.rename` | Rename Swatch… | — | <details><summary><code>JSON Schema</code></summary><code>{"swatch":name (or "index":n across all groups),"group":name? (narrows the name lookup),"name":str} → {name}</code></details> | — |
| `swatches.move` | Move Swatch | — | <details><summary><code>JSON Schema</code></summary><code>{"swatch":name (or "index":n across all groups),"group":name? (narrows the name lookup),"to":group?=its own group,"position":n?=end} → {group,position}</code></details> | — |
| `swatches.newGroup` | New Swatch Group… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str="Group" (made unique),"swatches":[names]? (moved into the new group)} → {group,moved}</code></details> | — |
| `swatches.renameGroup` | Rename Swatch Group… | — | <code>{"group":name,"name":str} → {group}</code> | — |
| `swatches.deleteGroup` | Delete Swatch Group | — | <details><summary><code>JSON Schema</code></summary><code>{"group":name,"keepSwatches":bool=false (move its swatches into the group above)} → {group,deleted¦kept}</code></details> | — |
| `swatches.use` | Use Swatch | — | <details><summary><code>JSON Schema</code></summary><code>{"swatch":name (or "index":n across all groups),"group":name? (narrows the name lookup),"target":"foreground¦background"="foreground"} → {swatch,target,hex}. CMYK, Lab and gray swatches are converted to the RGB working space with colour management.</code></details> | — |
| `swatches.reset` | Reset Swatches | — | <details><summary><code>JSON Schema</code></summary><code>{"append":bool=false (add the defaults instead of replacing the library)} → {groups,count}</code></details> | — |
| `swatches.import` | Import Swatches… | — | <details><summary><code>JSON Schema</code></summary><code>{"path":".aco or .ase file"?,"data":base64 bytes?,"format":"aco¦ase"? (detected),"mode":"append¦replace"="append","group":str?=file name (the group for ungrouped colours)} → {format,mode,groups,count,warnings}</code></details> | — |
| `swatches.export` | Export Swatches… | — | <details><summary><code>JSON Schema</code></summary><code>{"format":"aco¦ase"?=from path, else aco,"path":file? (else the bytes come back as base64 'data'),"group":name? (only that group),"swatches":[names]? (only these)} → {format,count,bytes,warnings,path¦data}</code></details> | — |

## 🔹 Timeline (7)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `timeline.create` | Create Video Timeline | — | <code>{duration?:30, fps?:30} → {timeline}</code> | — |
| `timeline.delete` | Delete Timeline | — | <code>{} → {timeline:null}</code> | — |
| `timeline.setFrame` | Go to Frame | — | <code>{frame} → {timeline}</code> | — |
| `timeline.nextFrame` | Next Frame | — | <code>{} → {timeline}</code> | — |
| `timeline.previousFrame` | Previous Frame | — | <code>{} → {timeline}</code> | — |
| `timeline.setProps` | Timeline Settings | — | <code>{fps?, duration?, workStart?, workEnd?} → {timeline}</code> | — |
| `timeline.info` | Timeline Info | — | <code>{} → {timeline}</code> | — |

## 🔹 Tool (5)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `tool.presets.list` | Tool Presets | — | <code>{"tool":name? (Current Tool Only)} → {presets:[{name,tool}]}</code> | — |
| `tool.presets.new` | New Tool Preset… | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?=tool,"tool":name,"options":{…}? (opaque; "brush" defaults to the current brush for painting tools),"includeColor":bool=false}</code></details> | — |
| `tool.presets.select` | Select Tool Preset | — | <details><summary><code>JSON Schema</code></summary><code>{"preset":name} → {name,tool,options} (applies "brush" and "foreground"; the shell switches tool and options)</code></details> | — |
| `tool.presets.edit` | Edit Tool Presets | — | <code>{"action":"rename¦delete","preset":name¦[names],"name":str (rename)}</code> | — |
| `tool.presets.reset` | Reset Tool Presets | — | `{}` | — |

## 🔹 Tools & Colors (8)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `tools.setColors` | Set Colors | — | <code>{"foreground":"#rrggbb"?,"background":"#rrggbb"?}</code> | — |
| `tools.swapColors` | Switch Foreground and Background Colors | — | `{}` | `X` |
| `tools.defaultColors` | Default Foreground and Background Colors | — | `{}` | `D` |
| `tools.setBrush` | Set Brush | — | <code>{"preset":name?,"reset":bool?,…BrushSettings fields (camelCase, deep-merged)}</code> | — |
| `tools.decreaseBrushSize` | Decrease Brush Size | — | `{}` | `[` |
| `tools.increaseBrushSize` | Increase Brush Size | — | `{}` | `]` |
| `tools.decreaseBrushHardness` | Decrease Brush Hardness | — | `{}` | `Shift+[` |
| `tools.increaseBrushHardness` | Increase Brush Hardness | — | `{}` | `Shift+]` |

## 🔹 Type (59)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `type.create` | New Type Layer | Layer › New | <details><summary><code>JSON Schema</code></summary><code>{"x":px,"y":px (baseline anchor of point text),"text":str,"box":[x,y,w,h]? (paragraph text),"name":str?,"align":"left¦center¦right¦justify…"?,"orientation":"horizontal¦vertical"="horizontal", …character keys: "font","size":pt=12,"color",…}</code></details> | — |
| `type.edit` | Edit Type | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"text":str? (replace all, styles kept),"replace":{"start":char,"end":char,"text":str}?,"runs":[{"start":char,"end":char,…character keys}]?,"box":[x,y,w,h]? (to paragraph text),"point":[x,y]? (to point text),"move":[dx,dy]?,"transform":[a,b,c,d,e,f]?,"antialias":"none¦sharp¦crisp¦strong¦smooth"?,"name":str?,"kerning":1/1000em¦"metrics"¦"optical"¦"off"? (with "range":[startChar,endChar]?, default all),"kernPair":{"at":caretChar,"by":1/1000em}? (Alt+←/→: the pair before the caret becomes manual, its current kerning + by)}</code></details> | — |
| `type.setStyle` | Set Type Style | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"range":[startChar,endChar]? (default all), "font":str,"fontStyle":str,"weight":100..900,"italic":bool,"size":0.1..=1296 pt,"color":"#rrggbb"¦[r,g,b,a],"tracking":-1000..=10000 (1/1000 em),"leading":pt¦"auto","baselineShift":pt,"horizontalScale":%,"verticalScale":%,"underline":bool,"strikethrough":bool,"fauxBold":bool,"fauxItalic":bool,"kerning":1/1000em (manual, after each character)¦"metrics"¦"optical"¦"off","caps":"normal¦small¦all","ligatures":bool,"discretionaryLigatures":bool,"features":{"ss01":1},"variations":{"wght":650},"language":str, paragraph keys: "align":"left¦center¦right¦justify¦justifyCenter¦justifyRight¦justifyAll","firstLineIndent":pt,"startIndent":pt,"endIndent":pt,"spaceBefore":pt,"spaceAfter":pt,"autoLeading":%,"direction":"auto¦ltr¦rtl","hyphenate":bool}</code></details> | — |
| `type.rasterize` | Rasterize Type | Layer › Rasterize | <code>{"layer":id?}</code> | — |
| `type.info` | Type Layer Info | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?} → text, runs/paragraphs (char offsets + styles), shape, transform, laid-out lines (text space px), bounds</code></details> | — |
| `type.fonts` | List Fonts | — | <code>{"family":str? (faces of one family)}</code> | — |
| `type.antiAlias.none` | None | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.sharp` | Sharp | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.crisp` | Crisp | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.strong` | Strong | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.smooth` | Smooth | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.windowsLcd` | Windows LCD | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.antiAlias.windows` | Windows | Type › Anti-Alias | <code>{"layer":id?}</code> | — |
| `type.orientation.horizontal` | Horizontal | Type › Orientation | <code>{"layer":id?}</code> | — |
| `type.orientation.vertical` | Vertical | Type › Orientation | <code>{"layer":id?}</code> | — |
| `type.openType.standardLigatures` | Standard Ligatures | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.contextualAlternates` | Contextual Alternates | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.discretionaryLigatures` | Discretionary Ligatures | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.swash` | Swash | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.oldstyle` | Oldstyle | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.stylisticAlternates` | Stylistic Alternates | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.titlingAlternates` | Titling Alternates | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.ornaments` | Ornaments | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.ordinals` | Ordinals | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.openType.fractions` | Fractions | Type › OpenType | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"on":bool? (default: toggle),"range":[startChar,endChar]? (default all)}</code></details> | — |
| `type.createWorkPath` | Create Work Path | Type | <code>{"layer":id?}</code> | — |
| `type.convertToShape` | Convert to Shape | Type | <code>{"layer":id?}</code> | — |
| `type.rasterizeTypeLayer` | Rasterize Type Layer | Type | <code>{"layer":id?}</code> | — |
| `type.convertToParagraphText` | Convert to Paragraph Text | Type | <code>{"layer":id?}</code> | — |
| `type.convertToPointText` | Convert to Point Text | Type | <code>{"layer":id?}</code> | — |
| `type.warpText` | Warp Text… | Type | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"style":"none¦arc¦arcLower¦arcUpper¦arch¦bulge¦shellLower¦shellUpper¦flag¦wave¦fish¦rise¦fisheye¦inflate¦squeeze¦twist"="arc","bend":-100..100=50,"horizontalDistortion":-100..100=0,"verticalDistortion":-100..100=0,"orientation":"horizontal¦vertical"="horizontal"}</code></details> | — |
| `type.updateAllTextLayers` | Update All Text Layers | Type | `{}` | — |
| `type.replaceAllMissingFonts` | Replace All Missing Fonts | Type | <code>{} (with the default family)</code> | — |
| `type.resolveMissingFonts` | Resolve Missing Fonts… | Type | <details><summary><code>JSON Schema</code></summary><code>{"map":{"Missing Family":"Installed Family"}?} (no map: list the missing families)</code></details> | — |
| `type.pasteLoremIpsum` | Paste Lorem Ipsum | Type | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"at":char? (default end),"new":bool=false (new paragraph text layer)}</code></details> | — |
| `type.saveDefaultTypeStyles` | Save Default Type Styles | Type | <code>{} (from the active type layer; new type layers start from them)</code> | — |
| `type.loadDefaultTypeStyles` | Load Default Type Styles | Type | <code>{"layer":id?}</code> | — |
| `type.characterStyle.new` | New Character Style | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"attrs":{model fields as in type.info ("size_pt":36,"font_family":"Inter","align":"Center",…) or Character/Paragraph panel keys as in type.setStyle ("size":36,"font":"Inter","color":"#rrggbb","align":"center",…)}?,"fromSelection":bool=true (start from the targeted text's formatting),"apply":bool=false,"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} → {"id","name"}</code></details> | — |
| `type.characterStyle.duplicate` | Duplicate Style | — | <code>{"id":u32}</code> | — |
| `type.characterStyle.delete` | Delete Style | — | <code>{"id":u32} (text keeps its formatting as overrides)</code> | — |
| `type.characterStyle.rename` | Rename Style | — | <code>{"id":u32,"name":str}</code> | — |
| `type.characterStyle.set` | Character Style Options | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32 (paragraph: 0 = Basic Paragraph),"name":str?,"attrs":{model fields as in type.info ("size_pt":36,"font_family":"Inter","align":"Center",…) or Character/Paragraph panel keys as in type.setStyle ("size":36,"font":"Inter","color":"#rrggbb","align":"center",…)},"replace":bool=false (drop attributes not given),"clear":[field]?} (text using the style updates, overrides kept)</code></details> | — |
| `type.characterStyle.apply` | Apply Character Style | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32¦0¦null (0/null: None / Basic Paragraph),"clearOverrides":bool=false (Alt-click),"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)}</code></details> | — |
| `type.characterStyle.redefine` | Redefine Character Style | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32? (default: the targeted text's style),"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} (the style takes the formatting of the start of the targeted text)</code></details> | — |
| `type.characterStyle.clearOverride` | Clear Override | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)}</code></details> | — |
| `type.characterStyle.list` | List Character Styles | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} → {"styles":[{"id","name",attributes,"resolved"}],"current":{"character":id¦null (mixed),"characterOverride":bool,"paragraph":id¦null,"paragraphOverride":bool}}</code></details> | — |
| `type.paragraphStyle.new` | New Paragraph Style | — | <details><summary><code>JSON Schema</code></summary><code>{"name":str?,"attrs":{model fields as in type.info ("size_pt":36,"font_family":"Inter","align":"Center",…) or Character/Paragraph panel keys as in type.setStyle ("size":36,"font":"Inter","color":"#rrggbb","align":"center",…)}?,"fromSelection":bool=true (start from the targeted text's formatting),"apply":bool=false,"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} → {"id","name"}</code></details> | — |
| `type.paragraphStyle.duplicate` | Duplicate Style | — | <code>{"id":u32}</code> | — |
| `type.paragraphStyle.delete` | Delete Style | — | <code>{"id":u32} (text keeps its formatting as overrides)</code> | — |
| `type.paragraphStyle.rename` | Rename Style | — | <code>{"id":u32,"name":str}</code> | — |
| `type.paragraphStyle.set` | Paragraph Style Options | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32 (paragraph: 0 = Basic Paragraph),"name":str?,"attrs":{model fields as in type.info ("size_pt":36,"font_family":"Inter","align":"Center",…) or Character/Paragraph panel keys as in type.setStyle ("size":36,"font":"Inter","color":"#rrggbb","align":"center",…)},"replace":bool=false (drop attributes not given),"clear":[field]?} (text using the style updates, overrides kept)</code></details> | — |
| `type.paragraphStyle.apply` | Apply Paragraph Style | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32¦0¦null (0/null: None / Basic Paragraph),"clearOverrides":bool=false (Alt-click),"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)}</code></details> | — |
| `type.paragraphStyle.redefine` | Redefine Paragraph Style | — | <details><summary><code>JSON Schema</code></summary><code>{"id":u32? (default: the targeted text's style),"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} (the style takes the formatting of the start of the targeted text)</code></details> | — |
| `type.paragraphStyle.clearOverride` | Clear Override | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)}</code></details> | — |
| `type.paragraphStyle.list` | List Paragraph Styles | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id? ¦ "layers":[id]? (default: the selected type layers),"range":[startChar,endChar]? (one layer; default all text)} → {"styles":[{"id","name",attributes,"resolved"}],"current":{"character":id¦null (mixed),"characterOverride":bool,"paragraph":id¦null,"paragraphOverride":bool}}</code></details> | — |
| `type.insertText` | Insert Glyph | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"text":str,"at":char? (default: end of text),"range":[startChar,endChar]? (replaced),"label":str?} → {"layer","caret"}</code></details> | — |
| `type.hitTest` | Hit-Test Type | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?, "x":px, "y":px} → {"layer","index":char,"line","inside":bool}. No layer: the topmost visible type layer whose laid-out text or rendered pixels contain the point (same order as the Type tool); a miss is an error. With a layer, index is the nearest caret even when inside is false</code></details> | — |
| `type.caret` | Type Caret | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?, "index":char (0..=length; empty text is 0)} → {"index","line","segment":[[x,y],[x,y]]} caret segment in document pixels, for rotated and vertical type too</code></details> | — |
| `type.navigate` | Navigate Type | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?, "index":char, "move":"wordPrev¦wordNext¦linePrev¦lineNext¦lineStart¦lineEnd¦start¦end", "x":px?} → {"index"}. Words are alphanumeric runs. x is the document x to keep across linePrev/lineNext (a caret segment's x); omitted uses the caret's own column. Empty text returns index 0</code></details> | — |

## 🔹 Variables (1)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `variables.list` | List Variables | — | <code>{} → {defs,dataSets,active}</code> | — |

## 🔹 View (23)

| Command ID (`command`) | Menu Label | Menu Path | Parameters (`params`) | Shortcut |
| :--- | :--- | :--- | :--- | :--- |
| `view.newGuide` | New Guide… | View | <code>{"orientation":"horizontal¦vertical","position":px}</code> | — |
| `view.moveGuide` | Move Guide | — | <code>{"orientation":"horizontal¦vertical","index":n,"position":px}</code> | — |
| `view.deleteGuide` | Delete Guide | — | <code>{"orientation":"horizontal¦vertical","index":n}</code> | — |
| `view.clearGuides` | Clear Guides | View | `{}` | — |
| `view.proofSetup` | Proof Setup… | — | <details><summary><code>JSON Schema</code></summary><code>{"profile":"working-cmyk¦srgb¦display-p3¦adobe-rgb-compat¦prophoto-compat¦linear-srgb¦rec2020¦gray-gamma-2.2¦sgray¦lab-d50¦coated-cmyk" (or a path to an .icc file)="working-cmyk","intent":"perceptual¦relative¦saturation¦absolute"="relative","bpc":bool=true,"simulatePaper":bool=false}</code></details> | — |
| `view.proofColors` | Proof Colors | View | <code>{"on":bool=toggle}</code> | `Cmd+Y` |
| `view.gamutWarning` | Gamut Warning | View | <code>{"on":bool=toggle,"threshold":deltaE=4,"profile":"<proof profile override>"}</code> | `Cmd+Shift+Y` |
| `view.newGuideLayout` | New Guide Layout… | View | <details><summary><code>JSON Schema</code></summary><code>{"columns":n=0,"width":px?,"gutter":px=0,"rows":n=0,"height":px?,"rowGutter":px=gutter,"margin":px¦[top,left,bottom,right]=0,"centerColumns":bool=false,"clearExisting":bool=false}</code></details> | — |
| `view.newGuidesFromShape` | New Guides From Shape | View | <code>{"layer":id?}</code> | — |
| `view.clearCanvasGuides` | Clear Canvas Guides | View | `{}` | — |
| `view.clearSelectedArtboardGuides` | Clear Selected Artboard Guides | View | <code>{} (guides inside the active artboard)</code> | — |
| `view.proofSetup.workingCyanPlate` | Working Cyan Plate | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.workingMagentaPlate` | Working Magenta Plate | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.workingYellowPlate` | Working Yellow Plate | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.workingBlackPlate` | Working Black Plate | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.workingCmyPlate` | Working CMY Plate | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.legacyMacintoshRgb` | Legacy Macintosh RGB | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.colorBlindnessProtanopia` | Color Blindness — Protanopia-type | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.proofSetup.colorBlindnessDeuteranopia` | Color Blindness — Deuteranopia-type | View › Proof Setup | <code>{} (sets the proof and turns Proof Colors on)</code> | — |
| `view.thirtyTwoBitPreviewOptions` | 32-bit Preview Options… | View | <details><summary><code>JSON Schema</code></summary><code>{"method":"exposureGamma¦highlightCompression"="exposureGamma","exposure":-20..20=0,"gamma":0.1..9.99=1}</code></details> | — |
| `view.lockSlices` | Lock Slices | View | <code>{"on":bool? (default: toggle)} → {locked}</code> | — |
| `view.clearSlices` | Clear Slices | View | <code>{} (deletes every user and layer-based slice) → {cleared}</code> | — |
| `view.layerMask` | View Layer Mask | — | <details><summary><code>JSON Schema</code></summary><code>{"layer":id?,"mode":"off"¦"gray"¦"overlay"¦"toggleGray"¦"toggleOverlay"="toggleGray"} (gray: the mask alone, ⌥-click its thumbnail; overlay: red rubylith over the composite, ⇧⌥-click or '\'; view state, not an undo step; painting while shown paints the mask)</code></details> | — |


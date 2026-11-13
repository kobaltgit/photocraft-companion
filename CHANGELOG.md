# Changelog

All notable changes to the **PhotoCraft Companion** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.1.1] - 2026-11-14

### Added
- **Official Cloud AI Partner Integration: Flexy Studio (`plugins/flexy_studio/`)**:
  - Full REST API client for [Flexy AI](https://flexy-ai.com/main) by XpucT;
  - 4-in-1 UI: Txt2Img generation with active layer ControlNet guidance (*Line Art*, *Canny*, *Depth*, *Scribble*, *Pose*);
  - Smart Eraser, Generative Fill, and Watermark Remover;
  - Studio portrait retouching (*Natural*, *Studio Glamour*) and age transformations (*Oldify*);
  - Canvas outpainting, background changer, and archival B&W colorization;
  - Secure API key storage and 1-click balance check.
- **Industrial Context Crop & Seamless In-Place Patch Compositing (SPEC-004 / BUG-126)**:
  - Bounding box extraction with 35% context padding, preserving 100% native resolution without server downscaling;
  - Automatic Catmull-Rom down/upscaling fallback for oversized selections;
  - Full-canvas transparent patch composition (`orig_w x orig_h`) for pixel-perfect canvas alignment;
  - Declarative `inpaint_in_place` toggle for automatic layer fusion (`layer.mergeDown`) or standalone patch layers;
  - Selection clipping protection via automatic `client.deselect()` before pasting.
- **Declarative Selection Mask Protocol (SPEC-003 / BUG-125)**:
  - Non-destructive `copy_active_layer_and_selection_mask()` in `BridgeOperations` and `PhotoCraftClient`;
  - Elimination of HTTP 400 Bad Request in cloud inpainting endpoints;
  - Reactive selection state updates via `document.selection` subscription.
- **Multi-Monitor Window Geometry Persistence (SPEC-001 / BUG-123)**:
  - Persistent window positions and sizes per plugin with multi-screen boundary clamping.

### Fixed
- **Windows Console Window Elimination (BUG-127)**:
  - Restored `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in `companion_app/src/main.rs`.
  - Recompiled with `IMAGE_SUBSYSTEM_WINDOWS_GUI` (Subsystem = 2), eliminating terminal window flashes on launch.
- **Modular Codebase Decomposition (SPEC-002 / Chapter 132)**:
  - Decomposed all legacy monolith files exceeding 800 lines into structured submodules with clean facades;
  - Enforced strict 800-line Hard Cap via automated `file_size_lint_test.rs`.

---

## [2.1.0] - 2026-11-12

### Added
- **Timelapse Studio (`plugins/timelapse_studio/`)**:
  - Event-driven reactive speedpaint recorder using PhotoCraft's native `document.revision` protocol;
  - Zero-timer frame capture on brush release (0.0% idle CPU, silky-smooth 60+ FPS drawing);
  - Headless screenshot RPC (`capture_canvas_frame`) without window focus theft;
  - FFmpeg video compilation with blur-fill vertical padding and sanitize logs.

### Fixed
- Elimination of main UI thread timer loop and DWM ghost windows (`pending_close_viewports` queue).
- Guaranteed session directory presence and empty session export guards.

---

## [2.0.3] - 2026-11-04

### Added
- Harmonic Color Studio and PopCraft 3D plugins.
- Adaptive Frame Pacing and process fork bomb elimination.

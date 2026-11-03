"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { SectionShell, SectionEyebrow } from "@/components/section-shell";
import { Terminal, RefreshCw, Sliders, CheckCircle2 } from "lucide-react";
import { assetPath } from "@/lib/asset-path";

interface Scenario {
  id: "remove_bg" | "depth_studio";
  name: string;
  pillLabel: string;
  modelBadge: string;
  icon: string;
  actionName: string;
  modelName: string;
  task: string;
  input: string;
  output: string;
  docTitle: string;
  beforeImg: string;
  afterImg: string;
  beforeLabel: string;
  afterLabel: string;
  defaultLatency: number;
}

const SCENARIOS: Scenario[] = [
  {
    id: "remove_bg",
    name: "AI Remove Background",
    pillLabel: "01 // REMOVE BG",
    modelBadge: "Bria RMBG 2.0 · DirectML",
    icon: assetPath("/plugins/remove_bg.svg"),
    actionName: "RUN REMOVE BG",
    modelName: "bria-rmbg-2.0.onnx",
    task: "SEGMENTATION_MASK",
    input: "ACTIVE_LAYER",
    output: "LAYER_MASK",
    docTitle: "PhotoCraft 1.0 — Gemini_Generated_Image.jpg @ 54.3% (Layer 0, RGB/8)",
    beforeImg: assetPath("/demo/photocraft-before.png"),
    afterImg: assetPath("/demo/photocraft-after.png"),
    beforeLabel: "BEFORE: RAW CANVAS",
    afterLabel: "AFTER: REMOVE BG (MASK COMMITTED)",
    defaultLatency: 380,
  },
  {
    id: "depth_studio",
    name: "Depth Studio (Fog & Bokeh)",
    pillLabel: "02 // DEPTH STUDIO",
    modelBadge: "Depth Anything V2 · DirectML",
    icon: assetPath("/plugins/depth_studio.svg"),
    actionName: "RUN DEPTH STUDIO",
    modelName: "depth-anything-v2-small.onnx",
    task: "DEPTH_ESTIMATION",
    input: "ACTIVE_LAYER",
    output: "2 LAYERS (FOG + BOKEH)",
    docTitle: "PhotoCraft 1.0 — MAI_52d43e822818e44c.png @ 67.7% (Atmospheric Fog, RGB/8)*",
    beforeImg: assetPath("/demo/depth-before.png"),
    afterImg: assetPath("/demo/depth-after.png"),
    beforeLabel: "BEFORE: SHARP BACKGROUND",
    afterLabel: "AFTER: DEPTH STUDIO (FOG & BOKEH)",
    defaultLatency: 420,
  },
];

export function InteractivePlayground() {
  const [activeScenarioId, setActiveScenarioId] = useState<"remove_bg" | "depth_studio">("remove_bg");
  const [sliderPos, setSliderPos] = useState<number>(50);
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [isInferring, setIsInferring] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string>(
    "DirectML ready · Socket 18492 idle"
  );
  const [latency, setLatency] = useState<number>(380);

  const activeScenario = SCENARIOS.find((s) => s.id === activeScenarioId) || SCENARIOS[0];
  const containerRef = useRef<HTMLDivElement>(null);

  // Switch scenario smoothly
  const handleSelectScenario = (id: "remove_bg" | "depth_studio") => {
    if (id === activeScenarioId) return;
    setActiveScenarioId(id);
    const targetScenario = SCENARIOS.find((s) => s.id === id) || SCENARIOS[0];
    setSliderPos(50);
    setLatency(targetScenario.defaultLatency);
    setStatusMessage(`SWITCHED: ${targetScenario.name} (${targetScenario.modelName})`);
  };

  // Handle pointer dragging
  const handlePointerMove = useCallback((clientX: number) => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const x = clientX - rect.left;
    const percentage = Math.max(0, Math.min(100, (x / rect.width) * 100));
    setSliderPos(percentage);
  }, []);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (isDragging) handlePointerMove(e.clientX);
    };
    const onMouseUp = () => setIsDragging(false);

    const onTouchMove = (e: TouchEvent) => {
      if (isDragging && e.touches[0]) handlePointerMove(e.touches[0].clientX);
    };
    const onTouchEnd = () => setIsDragging(false);

    if (isDragging) {
      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
      window.addEventListener("touchmove", onTouchMove);
      window.addEventListener("touchend", onTouchEnd);
    }

    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
      window.removeEventListener("touchmove", onTouchMove);
      window.removeEventListener("touchend", onTouchEnd);
    };
  }, [isDragging, handlePointerMove]);

  // Smooth animation helper
  const animateSliderTo = (target: number, durationMs = 600) => {
    const start = sliderPos;
    const startTime = performance.now();

    const step = (now: number) => {
      const elapsed = now - startTime;
      const progress = Math.min(1, elapsed / durationMs);
      const ease = 1 - Math.pow(1 - progress, 3);
      setSliderPos(start + (target - start) * ease);

      if (progress < 1) {
        requestAnimationFrame(step);
      }
    };

    requestAnimationFrame(step);
  };

  // Trigger plugin simulation
  const handleRunPlugin = () => {
    if (isInferring) return;
    setIsInferring(true);
    setStatusMessage(`INFERRING: ${activeScenario.modelName} (DirectML GPU)...`);

    // If already showing mostly after, reset to before first
    if (sliderPos < 25) {
      setSliderPos(100);
    }

    setTimeout(() => {
      const execLatency = Math.floor(activeScenario.defaultLatency + (Math.random() * 40 - 20));
      setLatency(execLatency);
      setIsInferring(false);
      setStatusMessage(`COMMITTED: ${activeScenario.output} applied in ${execLatency}ms`);
      animateSliderTo(0, 800);
    }, 450);
  };

  const handleReset = () => {
    animateSliderTo(100, 500);
    setStatusMessage("RESET: Restored original raw canvas");
  };

  const handleSplit = () => {
    animateSliderTo(50, 400);
    setStatusMessage("SPLIT: 50% comparison view");
  };

  return (
    <SectionShell id="playground">
      <SectionEyebrow
        index="01"
        label="INTERACTIVE PLAYGROUND"
        annotation="LIVE CANVAS COMPARISON"
      />

      {/* Header with Headline & Scenario Switcher */}
      <div className="flex flex-col gap-6 px-6 py-12 md:px-10 md:py-16">
        <div className="flex flex-col md:flex-row md:items-end md:justify-between gap-6">
          <div>
            <h2 className="max-w-3xl font-display text-3xl sm:text-5xl md:text-6xl font-medium tracking-tight text-ink-strong leading-[1.05]">
              See Companion in{" "}
              <span className="font-serif italic font-normal text-white">action.</span>
            </h2>
            <p className="mt-4 max-w-xl text-base sm:text-lg text-muted leading-relaxed">
              Experience in-place neural execution. Select a showcase scenario below, drag the
              comparison curtain, or click the Companion buttons to simulate live socket execution.
            </p>
          </div>

          {/* Scenario Tabs Switcher */}
          <div className="flex items-center gap-2 border border-line bg-bg-card p-1 self-start md:self-auto">
            {SCENARIOS.map((sc) => {
              const isActive = sc.id === activeScenarioId;
              return (
                <button
                  key={sc.id}
                  onClick={() => handleSelectScenario(sc.id)}
                  className={`flex items-center gap-2.5 px-4 py-2.5 font-mono text-xs font-semibold transition-all ${
                    isActive
                      ? "bg-accent text-black shadow-sm"
                      : "text-muted hover:text-white hover:bg-invert-bg"
                  }`}
                >
                  {/* eslint-disable-next-line @next/next/no-img-element */}
                  <img
                    src={sc.icon}
                    alt=""
                    className={`h-3.5 w-3.5 ${isActive ? "invert" : ""}`}
                  />
                  <span>{sc.pillLabel}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Action Controls Bar */}
        <div className="flex flex-wrap items-center justify-between gap-4 pt-2 border-t border-line">
          <div className="flex items-center gap-3">
            <span className="hud-label text-accent">ACTIVE SCENARIO:</span>
            <span className="font-mono text-xs font-bold text-white">
              {activeScenario.name}
            </span>
            <span className="hidden sm:inline text-xs text-muted font-mono">
              ({activeScenario.modelBadge})
            </span>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <button
              onClick={handleRunPlugin}
              disabled={isInferring}
              className={`group relative inline-flex items-center gap-2.5 font-mono text-xs font-bold px-6 py-3 transition-all shadow-sm active:translate-y-px ${
                isInferring
                  ? "bg-amber-400 text-black cursor-wait"
                  : "bg-accent text-black hover:bg-accent-ink"
              }`}
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={activeScenario.icon}
                alt=""
                className={`h-4 w-4 invert ${isInferring ? "animate-spin" : ""}`}
              />
              <span>{isInferring ? "INFERRING..." : activeScenario.actionName}</span>
            </button>

            <button
              onClick={handleSplit}
              className="inline-flex items-center gap-2 border border-line-strong bg-bg-card px-4 py-3 hud-label text-ink hover:bg-invert-bg hover:text-invert-fg transition-all"
              title="Split 50/50"
            >
              <Sliders className="h-3.5 w-3.5 text-accent" />
              <span>50/50 SPLIT</span>
            </button>

            <button
              onClick={handleReset}
              className="inline-flex items-center gap-2 border border-line-strong bg-bg-card px-4 py-3 hud-label text-ink hover:bg-invert-bg hover:text-invert-fg transition-all"
              title="Reset to raw image"
            >
              <RefreshCw className="h-3.5 w-3.5 text-muted group-hover:text-ink" />
              <span>RESET</span>
            </button>
          </div>
        </div>
      </div>

      {/* Interactive Canvas Frame */}
      <div className="px-6 pb-16 md:px-10">
        <div className="relative mx-auto max-w-[1180px] border border-line bg-bg-sunken shadow-2xl overflow-hidden">
          
          {/* Simulated PhotoCraft Window Titlebar */}
          <div className="flex h-9 items-center justify-between border-b border-line bg-[#18191c] px-4 text-xs font-mono text-muted select-none">
            <div className="flex items-center gap-3">
              <span className="flex gap-1.5">
                <span className="h-2.5 w-2.5 rounded-full bg-[#3a3b40]" />
                <span className="h-2.5 w-2.5 rounded-full bg-[#3a3b40]" />
                <span className="h-2.5 w-2.5 rounded-full bg-[#3a3b40]" />
              </span>
              <span className="text-[11px] font-semibold text-ink truncate max-w-[450px]">
                {activeScenario.docTitle}
              </span>
            </div>

            <div className="hidden sm:flex items-center gap-4 text-[10px] text-faint">
              <span className="flex items-center gap-1.5">
                <span
                  className={`h-1.5 w-1.5 rounded-full ${
                    isInferring ? "bg-amber-400 animate-ping" : "bg-accent"
                  }`}
                />
                <span>SOCKET 18492 ACTIVE</span>
              </span>
              <span>GPU: DIRECTML</span>
            </div>
          </div>

          {/* Interactive Comparison Viewport */}
          <div
            ref={containerRef}
            onPointerDown={(e) => {
              setIsDragging(true);
              handlePointerMove(e.clientX);
            }}
            className="relative w-full aspect-[16/9] max-h-[640px] cursor-ew-resize select-none overflow-hidden bg-black"
          >
            {/* Base Layer: AFTER (Committed Mask / Depth Bokeh Fog) */}
            <div className="absolute inset-0 pointer-events-none">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={activeScenario.afterImg}
                alt={activeScenario.afterLabel}
                className="h-full w-full object-cover object-center pointer-events-none"
                draggable={false}
              />
              <div className="absolute top-4 right-4 z-10 hud-label bg-black/75 backdrop-blur-sm border border-line px-3 py-1.5 text-[11px] text-accent flex items-center gap-2">
                <CheckCircle2 className="h-3 w-3 text-accent" />
                <span>{activeScenario.afterLabel}</span>
              </div>
            </div>

            {/* Top Clipped Layer: BEFORE (Raw Untouched Canvas) */}
            <div
              className="absolute inset-0 pointer-events-none"
              style={{
                clipPath: `polygon(0 0, ${sliderPos}% 0, ${sliderPos}% 100%, 0 100%)`,
              }}
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={activeScenario.beforeImg}
                alt={activeScenario.beforeLabel}
                className="h-full w-full object-cover object-center pointer-events-none"
                draggable={false}
              />
              <div className="absolute top-4 left-4 z-10 hud-label bg-black/75 backdrop-blur-sm border border-line px-3 py-1.5 text-[11px] text-muted flex items-center gap-2">
                <span>{activeScenario.beforeLabel}</span>
              </div>
            </div>

            {/* Draggable Divider Line & Handle */}
            <div
              className="absolute top-0 bottom-0 z-20 w-0.5 bg-accent cursor-ew-resize pointer-events-none"
              style={{ left: `${sliderPos}%` }}
            >
              <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 h-10 w-10 rounded-full bg-accent text-black font-mono font-bold text-xs flex items-center justify-center shadow-lg border-2 border-black transition-transform hover:scale-110 active:scale-95 pointer-events-auto">
                <span className="text-[10px] tracking-tighter select-none">◄►</span>
              </div>
            </div>

            {/* Floating Interactive Toolstrip Hint (Top Right) */}
            <div className="absolute top-14 right-20 z-30 hidden md:flex flex-col items-end pointer-events-none">
              <div className="bg-[#1e1f24]/95 backdrop-blur-md border border-accent/60 p-3 rounded shadow-2xl text-right max-w-xs pointer-events-auto">
                <div className="flex items-center gap-2 mb-1 justify-end">
                  <span className="h-1.5 w-1.5 rounded-full bg-accent animate-pulse" />
                  <span className="font-mono text-[10px] font-bold text-white tracking-wider">
                    COMPANION TOOLBAR
                  </span>
                </div>
                <p className="text-[11px] text-muted mb-2.5">
                  Click below to execute neural inpainting or segmentation:
                </p>
                <button
                  onClick={handleRunPlugin}
                  disabled={isInferring}
                  className="w-full flex items-center justify-center gap-2 bg-accent text-black font-mono text-[11px] font-bold py-2 px-3 rounded hover:bg-accent-ink transition-colors shadow-sm"
                >
                  {/* eslint-disable-next-line @next/next/no-img-element */}
                  <img src={activeScenario.icon} alt="" className="h-3.5 w-3.5 invert" />
                  <span>{isInferring ? "Inferring..." : `Execute ${activeScenario.name}`}</span>
                </button>
              </div>
            </div>

          </div>

          {/* Real-time Telemetry & Protocol Console */}
          <div className="border-t border-line bg-[#101114] p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 font-mono text-xs text-muted">
            <div className="flex items-center gap-2">
              <Terminal className="h-3.5 w-3.5 text-accent shrink-0" />
              <span className="text-faint">&gt;</span>
              <span className={isInferring ? "text-amber-400 font-semibold" : "text-ink"}>
                {statusMessage}
              </span>
            </div>

            <div className="flex items-center gap-4 text-[11px] text-faint">
              <span>LATENCY: <strong className="text-white">{latency}ms</strong></span>
              <span>TASK: <strong className="text-white">{activeScenario.task}</strong></span>
              <span>OUTPUT: <strong className="text-white">{activeScenario.output}</strong></span>
            </div>
          </div>

        </div>
      </div>
    </SectionShell>
  );
}

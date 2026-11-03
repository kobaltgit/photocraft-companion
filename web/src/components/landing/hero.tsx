import { Terminal, Cpu, ShieldCheck } from "lucide-react";
import { DownloadCta } from "./download-cta";

export function Hero() {
  return (
    <section id="hero" className="relative pt-16 pb-20 md:pt-24 md:pb-28 border-b border-line">
      <div className="relative mx-auto max-w-[1280px] px-6 md:px-10">
        
        {/* Monospace Architectural Eyebrow */}
        <div className="flex items-center gap-3 hud-label text-muted mb-6">
          <span className="inline-block h-2 w-2 bg-accent rounded-full animate-pulse" />
          <span>COMPANION RUNTIME & PLUGINS ECOSYSTEM</span>
          <span className="text-faint">/</span>
          <span className="text-faint">DESKTOP FIRST</span>
        </div>

        {/* Big Editorial Headline with Italic Accent */}
        <h1 className="max-w-4xl font-display text-4xl sm:text-6xl md:text-7xl font-bold tracking-tight text-ink-strong leading-[1.05]">
          The desktop companion for{" "}
          <span className="font-serif italic font-normal text-white">artists.</span>
        </h1>

        {/* Technical Subtitle */}
        <p className="mt-7 max-w-2xl text-lg md:text-xl text-muted leading-relaxed">
          Native, open-source neural engine and floating companion toolbar for PhotoCraft.
          Direct in-place canvas execution over local JSON-RPC socket 18492 with DirectML GPU
          acceleration and zero Wasm sandbox limits.
        </p>

        {/* Action Buttons: Installer, Portable & Discord */}
        <div className="mt-10">
          <DownloadCta layout="hero" />
        </div>

        {/* Technical Feature Specs Bar */}
        <div className="mt-16 grid grid-cols-2 sm:grid-cols-4 gap-px border border-line bg-line">
          <div className="bg-bg p-5 flex flex-col gap-1.5">
            <span className="hud-label text-faint flex items-center gap-1.5">
              <Terminal className="h-3 w-3 text-accent" />
              <span>IPC PROTOCOL</span>
            </span>
            <span className="font-mono text-sm text-ink font-semibold">Local Socket 18492</span>
            <span className="text-xs text-muted">JSON-RPC two-way bridge</span>
          </div>

          <div className="bg-bg p-5 flex flex-col gap-1.5">
            <span className="hud-label text-faint flex items-center gap-1.5">
              <Cpu className="h-3 w-3 text-accent" />
              <span>HARDWARE ACCELERATION</span>
            </span>
            <span className="font-mono text-sm text-ink font-semibold">DirectML / ONNX</span>
            <span className="text-xs text-muted">Native GPU inference with CPU fallback</span>
          </div>

          <div className="bg-bg p-5 flex flex-col gap-1.5">
            <span className="hud-label text-faint flex items-center gap-1.5">
              <ShieldCheck className="h-3 w-3 text-accent" />
              <span>ZERO-CORE CONTRACT</span>
            </span>
            <span className="font-mono text-sm text-ink font-semibold">Declarative JSON</span>
            <span className="text-xs text-muted">Zero core hardcoding or recompilation</span>
          </div>

          <div className="bg-bg p-5 flex flex-col gap-1.5">
            <span className="hud-label text-faint flex items-center gap-1.5">
              <span className="h-2 w-2 rounded-full bg-accent" />
              <span>LICENSE & FREEDOM</span>
            </span>
            <span className="font-mono text-sm text-ink font-semibold">100% Free & Open Source</span>
            <span className="text-xs text-muted">Permissive MIT License</span>
          </div>
        </div>

      </div>
    </section>
  );
}

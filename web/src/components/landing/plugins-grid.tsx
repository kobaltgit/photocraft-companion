"use client";

import { useState } from "react";
import { ArrowUpRight, Cpu, Download, ExternalLink, Sparkles, X } from "lucide-react";
import { SectionShell, SectionEyebrow } from "@/components/section-shell";
import { usePluginsList, RegistryPlugin } from "@/lib/plugins-data";

export function PluginsGrid() {
  const { plugins, totalCount } = usePluginsList();
  const [selectedPlugin, setSelectedPlugin] = useState<RegistryPlugin | null>(null);

  const getCategoryLabel = (type: string) => {
    switch (type) {
      case "native_ai":
        return "AI Neural";
      case "interactive_ui":
        return "Interactive UI";
      case "no_code":
        return "Procedural Macro";
      case "native_algorithm":
        return "Rust Algorithm";
      default:
        return "Plugin";
    }
  };

  const getModelLabel = (plugin: RegistryPlugin) => {
    if (plugin.models && plugin.models.length > 0) {
      return plugin.models.map((m) => m.name).join(", ");
    }
    return "Procedural Rust (No Model Required)";
  };

  return (
    <SectionShell id="plugins">
      <SectionEyebrow
        index="02"
        label="COMPANION PLUGINS"
        annotation="OPEN SOURCE · MIT LICENSE"
      />

      {/* Section Header with Dynamic Count */}
      <div className="flex flex-col gap-4 px-6 py-12 md:flex-row md:items-end md:justify-between md:px-10 md:py-16">
        <div>
          <h2 className="max-w-3xl font-display text-3xl sm:text-5xl md:text-6xl font-medium tracking-tight text-ink-strong leading-[1.05]">
            {totalCount} plugins. One{" "}
            <span className="font-serif italic font-normal text-white">companion.</span>
          </h2>
          <p className="mt-4 max-w-xl text-base sm:text-lg text-muted leading-relaxed">
            Native, open-source desktop plugins from the PhotoCraft ecosystem, built in Rust and free to use.
            Synchronized directly with the official plugin registry.
          </p>
        </div>

        <a
          href="https://github.com/kobaltgit/photocraft-companion/tree/main/plugins"
          target="_blank"
          rel="noopener noreferrer"
          className="hud-label flex shrink-0 items-center gap-1.5 text-muted hover:text-ink transition-colors"
        >
          <span>ALL {totalCount} PLUGINS ON GITHUB</span>
          <ArrowUpRight className="h-3.5 w-3.5" />
        </a>
      </div>

      {/* Dynamic Grid of Plugins */}
      <nav
        aria-label="Companion Plugins"
        className="grid grid-cols-2 gap-px border-t border-line bg-line sm:grid-cols-4 lg:grid-cols-7"
      >
        {plugins.map((plugin) => (
          <button
            key={plugin.id}
            type="button"
            onClick={() => setSelectedPlugin(plugin)}
            className="group relative flex flex-col items-center justify-center gap-3 bg-bg px-4 py-8 text-center transition-colors hover:bg-bg-raised cursor-pointer"
          >
            {/* Top accent indicator line on hover */}
            <span
              aria-hidden
              className="absolute inset-x-0 top-0 h-0.5 origin-left scale-x-0 bg-accent transition-transform duration-300 group-hover:scale-x-100"
            />

            {/* Version Badge (Top Right) */}
            <span className="absolute top-2 right-2 font-mono text-[9px] text-faint group-hover:text-muted transition-colors">
              v{plugin.version}
            </span>

            {/* Pure Vector SVG Icon (32x32) */}
            <div className="h-8 w-8 flex items-center justify-center transition-transform duration-300 ease-out group-hover:-translate-y-1">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={plugin.icon_local}
                onError={(e) => {
                  // Fallback to GitHub raw icon if local SVG isn't found
                  (e.target as HTMLImageElement).src = plugin.icon_url;
                }}
                alt={plugin.name}
                className="h-8 w-8 object-contain"
              />
            </div>

            {/* Plugin Title */}
            <span className="font-mono text-xs font-medium text-ink transition-colors group-hover:text-white line-clamp-2">
              {plugin.name}
            </span>

            {/* Micro Category indicator */}
            <span className="text-[10px] font-mono uppercase tracking-wider text-faint group-hover:text-accent transition-colors">
              {getCategoryLabel(plugin.plugin_type)}
            </span>
          </button>
        ))}
      </nav>

      {/* Interactive Detail Modal for Selected Plugin */}
      {selectedPlugin && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200"
          onClick={() => setSelectedPlugin(null)}
        >
          <div
            className="relative w-full max-w-xl border border-line-strong bg-bg p-6 md:p-8 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Close Button */}
            <button
              type="button"
              onClick={() => setSelectedPlugin(null)}
              className="absolute top-5 right-5 p-1 text-muted hover:text-white transition-colors"
              title="Close modal"
            >
              <X className="h-5 w-5" />
            </button>

            {/* Modal Header */}
            <div className="flex items-center gap-4 border-b border-line pb-5">
              <div className="h-12 w-12 border border-line bg-bg-raised flex items-center justify-center p-2.5 shrink-0">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img
                  src={selectedPlugin.icon_local}
                  onError={(e) => {
                    (e.target as HTMLImageElement).src = selectedPlugin.icon_url;
                  }}
                  alt={selectedPlugin.name}
                  className="h-7 w-7 object-contain"
                />
              </div>
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2 mb-1">
                  <h3 className="font-mono text-base font-bold text-white uppercase truncate">
                    {selectedPlugin.name}
                  </h3>
                  <span className="font-mono text-[10px] px-2 py-0.5 bg-black/30 text-accent border border-accent/40 font-semibold">
                    v{selectedPlugin.version}
                  </span>
                </div>
                <span className="hud-label text-muted text-[11px]">
                  {getCategoryLabel(selectedPlugin.plugin_type)} · by {selectedPlugin.author}
                </span>
              </div>
            </div>

            {/* Modal Body */}
            <div className="mt-5 space-y-4">
              <p className="text-sm text-ink leading-relaxed">
                {selectedPlugin.description}
              </p>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
                <div className="border border-line bg-bg-raised p-3">
                  <span className="hud-label text-faint flex items-center gap-1.5 mb-1">
                    <Sparkles className="h-3 w-3 text-accent" />
                    <span>MODEL / INFERENCE</span>
                  </span>
                  <span className="font-mono text-xs font-semibold text-white break-words">
                    {getModelLabel(selectedPlugin)}
                  </span>
                </div>

                <div className="border border-line bg-bg-raised p-3">
                  <span className="hud-label text-faint flex items-center gap-1.5 mb-1">
                    <Cpu className="h-3 w-3 text-accent" />
                    <span>PACKAGE ID</span>
                  </span>
                  <span className="font-mono text-xs font-semibold text-white truncate block">
                    {selectedPlugin.dirName}.pcpl
                  </span>
                </div>
              </div>
            </div>

            {/* Modal Footer with Direct .PCPL Download */}
            <div className="mt-6 flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-line">
              <div className="flex items-center gap-2">
                {/* Download .PCPL Package */}
                <a
                  href={selectedPlugin.download_url}
                  className="inline-flex items-center gap-2 bg-accent text-black font-mono text-xs font-bold px-5 py-2.5 hover:bg-accent-ink transition-colors shadow-sm"
                  title={`Download ${selectedPlugin.dirName}.pcpl`}
                >
                  <Download className="h-3.5 w-3.5" />
                  <span>DOWNLOAD .PCPL</span>
                </a>

                {/* View on GitHub */}
                <a
                  href={selectedPlugin.github_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 border border-line-strong px-4 py-2.5 hud-label text-ink hover:bg-invert-bg hover:text-invert-fg transition-colors"
                >
                  <span>SOURCE</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </div>

              <button
                type="button"
                onClick={() => setSelectedPlugin(null)}
                className="hud-label px-4 py-2.5 text-muted hover:text-white transition-colors"
              >
                CLOSE
              </button>
            </div>
          </div>
        </div>
      )}
    </SectionShell>
  );
}

"use client";

import { useReleaseInfo, DetectedOS } from "@/lib/release-info";
import { Download, Archive, ExternalLink, Monitor, Coffee } from "lucide-react";

interface DownloadCtaProps {
  layout?: "hero" | "banner";
}

export function DownloadCta({ layout = "hero" }: DownloadCtaProps) {
  const {
    version,
    os,
    installerUrl,
    installerName,
    portableUrl,
    portableName,
    releaseUrl,
  } = useReleaseInfo();

  const osLabelMap: Record<DetectedOS, string> = {
    windows: "Windows (x64)",
    macos: "macOS (Apple / Intel)",
    linux: "Linux (amd64)",
    other: "Desktop",
  };

  const osLabel = osLabelMap[os] || "Windows (x64)";

  return (
    <div className="flex flex-col gap-4">
      {/* Primary Row: Installer CTA + Portable + Discord + Coffee */}
      <div className="flex flex-wrap items-center gap-3">
        {/* Main Installer Button */}
        <a
          href={installerUrl}
          className="group relative inline-flex items-center gap-3 bg-accent text-black font-mono text-xs font-bold px-7 py-4 hover:bg-accent-ink transition-all shadow-sm active:translate-y-px"
          title={`Download ${installerName}`}
        >
          <Download className="h-4 w-4 transition-transform group-hover:-translate-y-0.5" />
          <span>
            DOWNLOAD FOR {osLabel.toUpperCase()}
          </span>
          <span className="inline-block bg-black/15 text-black px-2 py-0.5 text-[10px] font-mono tracking-wider">
            {version}
          </span>
        </a>

        {/* Portable Version Button */}
        <a
          href={portableUrl}
          className="inline-flex items-center gap-2 border border-line-strong bg-bg-card px-5 py-4 hud-label text-ink hover:bg-invert-bg hover:text-invert-fg transition-all"
          title={`Download portable archive: ${portableName}`}
        >
          <Archive className="h-3.5 w-3.5 text-accent" />
          <span>PORTABLE (.ZIP)</span>
        </a>

        {/* Discord Community Server CTA */}
        <a
          href="https://discord.gg/2gzn4Un4Bw"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2.5 border border-line-strong bg-[#5865F2]/10 text-white px-5 py-4 hud-label hover:bg-[#5865F2] hover:text-white transition-all group"
          title="Join official PhotoCraft Discord Server"
        >
          <svg
            className="h-4 w-4 fill-current text-[#5865F2] group-hover:text-white transition-colors"
            viewBox="0 0 24 24"
          >
            <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.929 1.793 8.18 1.793 12.061 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.893.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z" />
          </svg>
          <span>DISCORD COMMUNITY</span>
        </a>

        {/* Support on Lava.top (Buy me coffee) */}
        <a
          href="https://app.lava.top/kobaltgit"
          target="_blank"
          rel="noopener noreferrer"
          className="group inline-flex items-center gap-2.5 border border-[#55D61E]/40 bg-[#55D61E]/10 text-[#55D61E] px-5 py-4 hud-label hover:bg-[#55D61E] hover:text-black transition-all"
          title="Buy me a coffee on Lava.top"
        >
          <Coffee className="h-4 w-4 transition-colors" />
          <span className="text-white group-hover:text-black transition-colors">BUY ME COFFEE</span>
        </a>
      </div>

      {/* Secondary Meta Row: Direct Platform Switching & Release Notes */}
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs text-muted font-mono">
        <span className="flex items-center gap-1.5 text-faint">
          <Monitor className="h-3 w-3" />
          <span>Detected: {osLabel}</span>
        </span>
        <span className="text-line-strong">•</span>
        <a
          href={releaseUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-white transition-colors underline decoration-line-strong underline-offset-4"
        >
          All Platforms (macOS, Linux, Win32)
        </a>
        <span className="text-line-strong">•</span>
        <a
          href="https://github.com/kobaltgit/photocraft-companion"
          target="_blank"
          rel="noopener noreferrer"
          className="hover:text-white transition-colors flex items-center gap-1"
        >
          <span>Source Code</span>
          <ExternalLink className="h-2.5 w-2.5" />
        </a>
      </div>
    </div>
  );
}

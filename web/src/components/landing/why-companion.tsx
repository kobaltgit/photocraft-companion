import { SectionShell, SectionEyebrow } from "@/components/section-shell";
import { ShieldCheck, Zap, Lock, RefreshCw, Terminal, Layers } from "lucide-react";

export function WhyCompanion() {
  const pillars = [
    {
      title: "Zero Core Hardcoding",
      desc: "Anyone can drop a folder with an ONNX model and declarative plugin.json into plugins/ — and it works immediately without recompiling the program.",
      icon: ShieldCheck
    },
    {
      title: "Direct Socket 18492 Bridge",
      desc: "Companion and PhotoCraft communicate over a local JSON-RPC socket. Read active layers, execute selection commands, and push masks in milliseconds.",
      icon: Terminal
    },
    {
      title: "100% Offline & Private",
      desc: "Zero cloud telemetry, zero external APIs. Neural models run directly on your hardware via DirectML and ONNX Runtime. Your art never leaves your machine.",
      icon: Lock
    },
    {
      title: "Seamless In-Place Canvas Edit",
      desc: "Forget manual file saving or folder browsing. Click a toolstrip action, and the result commits directly into your active PhotoCraft layer stack.",
      icon: Zap
    },
    {
      title: "Automatic CPU Fallback",
      desc: "Intelligent execution provider fallback. If DirectML GPU acceleration is unavailable, neural inference runs seamlessly on CPU threads.",
      icon: RefreshCw
    },
    {
      title: "Zero Subscriptions Forever",
      desc: "Free and open-source under the permissive MIT license. Built by creators for creators, with clean Rust architecture and community plugins.",
      icon: Layers
    }
  ];

  return (
    <SectionShell id="why-companion">
      <SectionEyebrow
        index="04"
        label="WHY COMPANION"
        annotation="ARCHITECTURAL FREEDOM"
      />

      <div className="px-6 py-12 md:px-10 md:py-16">
        <h2 className="max-w-3xl font-display text-3xl sm:text-5xl md:text-6xl font-medium tracking-tight text-ink-strong leading-[1.05]">
          Built with an uncompromising{" "}
          <span className="font-serif italic font-normal text-white">philosophy.</span>
        </h2>
        <p className="mt-4 max-w-xl text-base sm:text-lg text-muted leading-relaxed">
          PhotoCraft Companion is not an isolated sandbox or a subscription wrapper. It is a neutral, high-performance desktop runtime.
        </p>

        <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-px border border-line bg-line">
          {pillars.map((pillar, i) => {
            const Icon = pillar.icon;
            return (
              <div
                key={i}
                className="bg-bg p-8 hover:bg-bg-raised transition-colors flex flex-col justify-between"
              >
                <div>
                  <div className="flex items-center justify-between mb-6">
                    <Icon className="h-5 w-5 text-accent" />
                    <span className="font-mono text-xs text-faint">0{i + 1}</span>
                  </div>

                  <h3 className="font-mono text-lg font-bold text-white mb-2.5">
                    {pillar.title}
                  </h3>

                  <p className="text-sm text-muted leading-relaxed">
                    {pillar.desc}
                  </p>
                </div>

                <div className="mt-6 pt-4 border-t border-line hud-label text-[10px] text-faint">
                  RULE // VERIFIED
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </SectionShell>
  );
}

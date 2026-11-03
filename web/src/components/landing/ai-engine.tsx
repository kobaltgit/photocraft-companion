import { SectionShell, SectionEyebrow } from "@/components/section-shell";
import { Sparkles, Cpu, Eye, Sliders } from "lucide-react";

export function AiEngine() {
  const features = [
    {
      title: "AI Remove Background",
      badge: "BRIA RMBG-1.4 · DIRECTML GPU · UP TO 8K",
      desc: "High-fidelity subject cutout preserving hair and fine transparent details up to 8K. Sub-actions allow instant export to a new layer or a non-destructive layer mask in PhotoCraft.",
      icon: Sparkles
    },
    {
      title: "AI Generative Fill (LaMa)",
      badge: "LAMA INPAINTING · ZERO-PROMPT · IN-PLACE",
      desc: "Context-aware object erasure via lasso selection and canvas border expansion without prompt hallucinations or cloud queues. Seamless texture synthesis in seconds.",
      icon: Cpu
    },
    {
      title: "Interactive Depth Studio",
      badge: "DEPTH ANYTHING V2 · MULTI-VIEWPORT GPU",
      desc: "Monocular 3D depth estimation with click-to-focus lens bokeh defocus, aerial perspective, and atmospheric fog simulation calculated directly on your GPU.",
      icon: Eye
    },
    {
      title: "Two-Tier Floating Toolstrip",
      badge: "EGUI 0.36 · DUAL-ORIENTATION · ZERO TASKBAR",
      desc: "Ultra-compact palette hovering over PhotoCraft. Toggle between horizontal (78px) and vertical (50px) layouts with screen coordinate memory and sub-action flyouts.",
      icon: Sliders
    }
  ];

  return (
    <SectionShell id="ai-studio">
      <SectionEyebrow
        index="03"
        label="LOCAL AI STUDIO"
        annotation="DIRECTML GPU ACCELERATION"
      />

      <div className="px-6 py-12 md:px-10 md:py-16">
        <h2 className="max-w-3xl font-display text-3xl sm:text-5xl md:text-6xl font-medium tracking-tight text-ink-strong leading-[1.05]">
          The precision that{" "}
          <span className="font-serif italic font-normal text-white">cloud AI</span>{" "}
          cannot give.
        </h2>
        <p className="mt-4 max-w-xl text-base sm:text-lg text-muted leading-relaxed">
          Text prompts lack spatial control. Work directly on your layers in PhotoCraft with professional selection masks, and let local neural models execute seamlessly in place on your canvas.
        </p>

        {/* 4 Feature Cards Grid */}
        <div className="mt-12 grid grid-cols-1 md:grid-cols-2 gap-px border border-line bg-line">
          {features.map((feat, i) => {
            const Icon = feat.icon;
            return (
              <div
                key={i}
                className="group relative bg-bg p-8 hover:bg-bg-raised transition-colors flex flex-col justify-between"
              >
                <div>
                  <div className="flex items-center justify-between mb-6">
                    <span className="hud-label text-accent flex items-center gap-2 text-[10px]">
                      <Icon className="h-3.5 w-3.5" />
                      <span>{feat.badge}</span>
                    </span>
                    <span className="font-mono text-xs text-faint">0{i + 1}</span>
                  </div>

                  <h3 className="font-mono text-xl font-bold text-white mb-3">
                    {feat.title}
                  </h3>

                  <p className="text-sm text-muted leading-relaxed">
                    {feat.desc}
                  </p>
                </div>

                <div className="mt-8 pt-4 border-t border-line flex items-center justify-between hud-label text-faint group-hover:text-ink transition-colors">
                  <span>STATUS: READY</span>
                  <span>LOCAL ONNX / DIRECTML</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </SectionShell>
  );
}

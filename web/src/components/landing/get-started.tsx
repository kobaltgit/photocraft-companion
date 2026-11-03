import { SectionShell, SectionEyebrow } from "@/components/section-shell";
import { DownloadCta } from "./download-cta";

export function GetStarted() {
  return (
    <SectionShell id="get-started">
      <SectionEyebrow
        index="05"
        label="GET STARTED"
        annotation="NATIVE DESKTOP RUNTIME"
      />

      <div className="px-6 py-14 md:px-10 md:py-20 text-center flex flex-col items-center">
        <h2 className="max-w-3xl font-display text-4xl sm:text-6xl font-medium tracking-tight text-ink-strong leading-[1.05]">
          Your studio. Your GPU.{" "}
          <span className="font-serif italic font-normal text-white">Your control.</span>
        </h2>
        
        <p className="mt-5 max-w-xl text-base sm:text-lg text-muted leading-relaxed">
          PhotoCraft Companion runs locally as a native Windows/macOS/Linux utility.
          No accounts, no cloud tokens, no monthly subscriptions.
        </p>

        {/* Action Buttons with Dynamic Version & OS */}
        <div className="mt-9 flex justify-center">
          <DownloadCta layout="banner" />
        </div>

        {/* 3 Step Onboarding Bar */}
        <div className="mt-14 w-full max-w-4xl grid grid-cols-1 md:grid-cols-3 gap-px border border-line bg-line text-left">
          <div className="bg-bg p-6">
            <span className="hud-label text-accent mb-2 block">STEP 01</span>
            <h4 className="font-mono text-sm font-bold text-white mb-1">Launch PhotoCraft</h4>
            <p className="text-xs text-muted">Start PhotoCraft editor. The local IPC control socket opens on port 18492 automatically.</p>
          </div>

          <div className="bg-bg p-6">
            <span className="hud-label text-accent mb-2 block">STEP 02</span>
            <h4 className="font-mono text-sm font-bold text-white mb-1">Start Companion</h4>
            <p className="text-xs text-muted">Run companion_app.exe. The floating toolstrip docks over your canvas and detects active layers.</p>
          </div>

          <div className="bg-bg p-6">
            <span className="hud-label text-accent mb-2 block">STEP 03</span>
            <h4 className="font-mono text-sm font-bold text-white mb-1">Craft in Real-Time</h4>
            <p className="text-xs text-muted">Click any tool or AI action. Results commit directly to your layer stack with zero latency.</p>
          </div>
        </div>

      </div>
    </SectionShell>
  );
}

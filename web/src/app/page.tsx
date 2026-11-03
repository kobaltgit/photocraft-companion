import { SiteNav } from "@/components/site-nav";
import { Hero } from "@/components/landing/hero";
import { InteractivePlayground } from "@/components/landing/interactive-playground";
import { PluginsGrid } from "@/components/landing/plugins-grid";
import { AiEngine } from "@/components/landing/ai-engine";
import { WhyCompanion } from "@/components/landing/why-companion";
import { GetStarted } from "@/components/landing/get-started";
import { SiteFooter } from "@/components/site-footer";
import { FixedCornerNav } from "@/components/ruler/fixed-corner-nav";

export default function Home() {
  return (
    <div className="relative min-h-screen bg-bg text-ink selection:bg-accent selection:text-black">
      {/* Architectural Sticky Navigation Bar */}
      <SiteNav />

      {/* Main Content Sections */}
      <main id="main">
        <Hero />
        <InteractivePlayground />
        <PluginsGrid />
        <AiEngine />
        <WhyCompanion />
        <GetStarted />
      </main>

      {/* Architectural Footer */}
      <SiteFooter />

      {/* Fixed Bottom-Right Corner Section HUD */}
      <FixedCornerNav />
    </div>
  );
}

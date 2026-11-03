"use client";

import { useEffect, useState } from "react";

const SECTIONS = [
  { id: "hero", label: "PHOTOCRAFT COMPANION" },
  { id: "playground", label: "LIVE PLAYGROUND" },
  { id: "plugins", label: "CRAFTING PLUGINS" },
  { id: "ai-studio", label: "LOCAL AI STUDIO" },
  { id: "why-companion", label: "WHY COMPANION" },
  { id: "get-started", label: "GET STARTED" },
];

export function FixedCornerNav() {
  const [activeId, setActiveId] = useState("hero");

  useEffect(() => {
    const handleScroll = () => {
      const scrollPos = window.scrollY + 250;
      for (let i = SECTIONS.length - 1; i >= 0; i--) {
        const sec = document.getElementById(SECTIONS[i].id);
        if (sec && sec.offsetTop <= scrollPos) {
          setActiveId(SECTIONS[i].id);
          break;
        }
      }
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const scrollTo = (id: string) => {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: "smooth" });
    }
  };

  return (
    <aside
      aria-label="Section Navigation"
      className="hidden xl:flex fixed bottom-8 right-8 z-50 flex-col items-end gap-1.5 select-none pointer-events-auto"
    >
      {SECTIONS.map((sec) => {
        const isActive = activeId === sec.id;
        return (
          <button
            key={sec.id}
            type="button"
            onClick={() => scrollTo(sec.id)}
            className={`group flex items-center justify-end text-right transition-all duration-200 cursor-pointer ${
              isActive
                ? "text-ink-strong font-bold"
                : "text-faint hover:text-ink font-medium"
            }`}
          >
            <span
              className={`font-mono text-[11px] tracking-[0.14em] uppercase transition-colors ${
                isActive ? "text-ink-strong" : "text-muted group-hover:text-ink"
              }`}
            >
              {sec.label}
            </span>
            <span
              aria-hidden
              className={`ml-2.5 h-1 transition-all duration-200 ${
                isActive
                  ? "w-3 bg-accent opacity-100"
                  : "w-1 bg-line-strong opacity-40 group-hover:w-2 group-hover:opacity-80"
              }`}
            />
          </button>
        );
      })}
    </aside>
  );
}

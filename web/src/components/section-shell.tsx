import React from "react";

interface SectionShellProps {
  id: string;
  children: React.ReactNode;
  className?: string;
  ticks?: boolean;
}

export function SectionShell({
  id,
  children,
  className = "",
  ticks = true,
}: SectionShellProps) {
  return (
    <section id={id} className={`relative ${className}`}>
      {/* Top Hairline Rule */}
      <span
        aria-hidden
        className="absolute inset-x-0 top-0 h-px bg-line"
      />
      
      {/* Centered Structural Container (1280px max-width) */}
      <div className="relative mx-auto max-w-[1280px] border-x border-line">
        {ticks && (
          <>
            {/* Left drafting crosshair tick */}
            <span
              aria-hidden
              className="tick tick-half z-30 -top-[5px] -left-[6px]"
            />
            {/* Right drafting crosshair tick */}
            <span
              aria-hidden
              className="tick tick-half z-30 -top-[5px] -right-[5px]"
            />
          </>
        )}
        {children}
      </div>
    </section>
  );
}

interface SectionEyebrowProps {
  index: string;
  label: string;
  annotation?: string;
}

export function SectionEyebrow({
  index,
  label,
  annotation,
}: SectionEyebrowProps) {
  return (
    <div className="sticky top-12 z-20 border-y border-line bg-bg/85 backdrop-blur-md">
      <div className="flex items-center justify-between gap-4 px-6 py-3 md:px-10">
        <p className="hud-label text-muted">
          <span className="text-faint">{index}</span>
          <span aria-hidden className="mx-2 text-faint">
            /
          </span>
          <span className="text-ink">{label}</span>
        </p>
        {annotation && (
          <p className="hud-label hidden text-faint sm:block">{annotation}</p>
        )}
      </div>
    </div>
  );
}

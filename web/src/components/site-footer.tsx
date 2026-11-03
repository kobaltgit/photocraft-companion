import Link from "next/link";
import { assetPath } from "@/lib/asset-path";

export function SiteFooter() {
  return (
    <footer className="relative border-t border-line bg-bg-sunken pt-16 pb-12">
      <div className="relative mx-auto max-w-[1280px] px-6 md:px-10">
        
        {/* Top Footer Row */}
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-8 pb-12 border-b border-line">
          <div className="flex items-center gap-3">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img src={assetPath("/logo.svg")} alt="PhotoCraft Companion" className="h-7 w-auto" />
            <div>
              <span className="font-mono text-xs font-bold tracking-wider text-white block">
                PHOTOCRAFT COMPANION
              </span>
              <span className="text-[11px] text-muted">
                Open-source local neural runtime & plugins ecosystem
              </span>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-6 hud-label text-muted">
            <Link href="#playground" className="hover:text-accent transition-colors">
              Playground
            </Link>
            <Link href="#plugins" className="hover:text-white transition-colors">
              Plugins
            </Link>
            <Link href="#ai-studio" className="hover:text-white transition-colors">
              AI Studio
            </Link>
            <Link href="#why-companion" className="hover:text-white transition-colors">
              Zero-Core
            </Link>
            <a
              href="https://discord.gg/2gzn4Un4Bw"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-[#5865F2] transition-colors flex items-center gap-1.5"
            >
              <svg className="h-3 w-3 fill-current" viewBox="0 0 24 24">
                <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.929 1.793 8.18 1.793 12.061 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.893.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z" />
              </svg>
              <span>Discord</span>
            </a>
            <a
              href="https://github.com/kobaltgit/photocraft-companion"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-white transition-colors flex items-center gap-1.5"
            >
              <svg className="h-3 w-3 fill-current" viewBox="0 0 24 24">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
              </svg>
              <span>GitHub</span>
            </a>
          </div>
        </div>

        {/* Bottom Metadata Row */}
        <div className="mt-8 flex flex-col md:flex-row items-start md:items-center justify-between gap-4 text-xs text-muted font-mono">
          <div className="flex items-center gap-2">
            <span>© 2026 PhotoCraft Companion Team. Permissive MIT License.</span>
          </div>

          <div className="flex items-center gap-6 text-[10px] uppercase tracking-wider text-faint">
            <span className="flex items-center gap-1.5">
              <span className="h-1.5 w-1.5 rounded-full bg-accent" />
              <span>IPC PORT 18492</span>
            </span>
            <span>DIRECTML // ONNX RUNTIME</span>
            <span>EGUI 0.36 // PURE RUST</span>
          </div>
        </div>

      </div>
    </footer>
  );
}

import type { Metadata, Viewport } from "next";
import { Inter, Playfair_Display, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin", "cyrillic"],
  variable: "--font-inter",
  display: "swap",
});

const playfair = Playfair_Display({
  subsets: ["latin", "cyrillic"],
  style: ["normal", "italic"],
  variable: "--font-playfair",
  display: "swap",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  display: "swap",
});

export const metadata: Metadata = {
  title: "PhotoCraft Companion — Local AI Runtime & Plugins",
  description:
    "Open-source companion runtime and precision toolbar for PhotoCraft. Direct in-place neural inference via DirectML GPU and local socket 18492 with zero Wasm limits.",
  icons: {
    icon: "/logo.svg",
  },
};

export const viewport: Viewport = {
  themeColor: "#121316",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${inter.variable} ${playfair.variable} ${mono.variable}`}
    >
      <body className="min-h-screen bg-bg text-ink antialiased">
        {children}
      </body>
    </html>
  );
}

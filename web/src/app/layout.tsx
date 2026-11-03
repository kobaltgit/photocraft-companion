import type { Metadata, Viewport } from "next";
import { Inter, Playfair_Display, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { assetPath } from "@/lib/asset-path";

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

const SITE_URL = "https://kobaltgit.github.io/photocraft-companion";
const SITE_TITLE = "PhotoCraft Companion — Local AI Runtime & Floating Toolbar";
const SITE_DESC =
  "Native, open-source neural engine and floating companion toolbar for PhotoCraft. Direct in-place canvas execution over local JSON-RPC socket 18492 with DirectML GPU acceleration and zero Wasm limits.";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: SITE_TITLE,
    template: "%s | PhotoCraft Companion",
  },
  description: SITE_DESC,
  applicationName: "PhotoCraft Companion",
  authors: [{ name: "kobaltGIT", url: "https://github.com/kobaltgit" }],
  creator: "kobaltGIT",
  publisher: "PhotoCraft Ecosystem",
  keywords: [
    "PhotoCraft",
    "PhotoCraft Companion",
    "local AI photo editor",
    "Photoshop alternative plugins",
    "DirectML GPU inference",
    "ONNX Runtime Rust",
    "Bria RMBG 2.0 remove background",
    "LaMa AI generative fill",
    "Depth Anything V2",
    "open source image editing",
    "socket 18492",
    "desktop floating toolbar",
  ],
  alternates: {
    canonical: SITE_URL,
  },
  icons: {
    icon: [{ url: assetPath("/logo.svg"), type: "image/svg+xml" }],
    apple: [{ url: assetPath("/logo.svg"), type: "image/svg+xml" }],
  },
  openGraph: {
    type: "website",
    locale: "en_US",
    url: SITE_URL,
    siteName: "PhotoCraft Companion",
    title: SITE_TITLE,
    description: SITE_DESC,
    images: [
      {
        url: assetPath("/og-image.png"),
        width: 1200,
        height: 630,
        alt: "PhotoCraft Companion — Local AI Runtime & Floating Toolbar for Artists",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: SITE_TITLE,
    description: SITE_DESC,
    images: [assetPath("/og-image.png")],
    creator: "@kobaltgit",
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
};

export const viewport: Viewport = {
  themeColor: "#121316",
  colorScheme: "dark",
  width: "device-width",
  initialScale: 1,
};

const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "PhotoCraft Companion",
  operatingSystem: "Windows 10, Windows 11, macOS, Linux",
  applicationCategory: "MultimediaApplication",
  applicationSubCategory: "GraphicEditorExtension",
  softwareVersion: "2.0.2",
  license: "https://opensource.org/licenses/MIT",
  url: SITE_URL,
  downloadUrl: "https://github.com/kobaltgit/photocraft-companion/releases",
  image: `${SITE_URL}/og-image.png`,
  description: SITE_DESC,
  offers: {
    "@type": "Offer",
    price: "0",
    priceCurrency: "USD",
  },
  author: {
    "@type": "Person",
    name: "kobaltGIT",
    url: "https://github.com/kobaltgit",
  },
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
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </head>
      <body className="min-h-screen bg-bg text-ink antialiased">
        {children}
      </body>
    </html>
  );
}

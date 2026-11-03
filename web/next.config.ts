import type { NextConfig } from "next";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "export",
  basePath: basePath || undefined,
  images: {
    unoptimized: true,
  },
};

export default nextConfig;

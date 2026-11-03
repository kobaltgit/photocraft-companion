"use client";

import { useState, useEffect } from "react";
import registryData from "../../public/registry.json";
import { assetPath } from "./asset-path";

export interface PluginModel {
  name: string;
  required?: boolean;
  size_mb?: number;
  url?: string;
  sha256?: string;
}

export interface RegistryPlugin {
  id: string;
  dirName: string;
  name: string;
  version: string;
  description: string;
  author: string;
  plugin_type: "native_ai" | "interactive_ui" | "no_code" | "native_algorithm" | string;
  download_url: string;
  icon_url: string;
  icon_local: string;
  models: PluginModel[];
  github_url: string;
}

function extractDirName(id: string, downloadUrl?: string): string {
  if (downloadUrl) {
    const match = downloadUrl.match(/\/([^/]+)\.pcpl$/);
    if (match) return match[1];
  }
  const parts = id.split(".");
  return parts[parts.length - 1];
}

function parseRawPlugins(rawList: Array<Record<string, unknown>>): RegistryPlugin[] {
  return rawList.map((p) => {
    const id = String(p.id || "");
    const dirName = extractDirName(id, String(p.download_url || ""));
    return {
      id: dirName,
      dirName,
      name: String(p.name || dirName),
      version: String(p.version || "1.0.0"),
      description: String(p.description || ""),
      author: String(p.author || "kobaltGIT"),
      plugin_type: String(p.plugin_type || "native_ai"),
      download_url:
        String(p.download_url) ||
        `https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/${dirName}.pcpl`,
      icon_url:
        String(p.icon_url) ||
        `https://github.com/kobaltgit/photocraft-companion/raw/main/plugins/${dirName}/icon.svg`,
      icon_local: assetPath(`/plugins/${dirName}.svg`),
      models: Array.isArray(p.models) ? (p.models as PluginModel[]) : [],
      github_url: `https://github.com/kobaltgit/photocraft-companion/tree/main/plugins/${dirName}`,
    };
  });
}

// Baseline built directly from registry.json at compile time - ZERO HARDCODED VERSIONS
export const INITIAL_PLUGINS: RegistryPlugin[] = parseRawPlugins(
  (registryData.plugins || []) as unknown as Array<Record<string, unknown>>
);

export function usePluginsList() {
  const [plugins, setPlugins] = useState<RegistryPlugin[]>(INITIAL_PLUGINS);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    let isMounted = true;

    async function syncRegistry() {
      try {
        const res = await fetch(assetPath("/registry.json"));
        if (!res.ok) return;

        const data = await res.json();
        const rawPlugins = data.plugins;

        if (Array.isArray(rawPlugins) && rawPlugins.length > 0 && isMounted) {
          const parsed = parseRawPlugins(rawPlugins as unknown as Array<Record<string, unknown>>);
          setPlugins(parsed);
        }
      } catch (err) {
        console.warn("Using compiled registry data, runtime fetch error:", err);
      } finally {
        if (isMounted) setIsLoading(false);
      }
    }

    syncRegistry();

    return () => {
      isMounted = false;
    };
  }, []);

  return {
    plugins,
    totalCount: plugins.length,
    isLoading,
  };
}

"use client";

import { useEffect, useState } from "react";

export type DetectedOS = "windows" | "macos" | "linux" | "other";

export interface ReleaseAssetInfo {
  name: string;
  downloadUrl: string;
  size: number;
}

export interface ReleaseInfo {
  version: string;
  tagName: string;
  releaseUrl: string;
  os: DetectedOS;
  installerUrl: string;
  installerName: string;
  portableUrl: string;
  portableName: string;
  isLoading: boolean;
}

const FALLBACK_VERSION = "v2.0.1";
const REPO = "kobaltgit/photocraft-companion";

export function detectUserOS(): DetectedOS {
  if (typeof window === "undefined" || !navigator) return "windows";
  
  const userAgent = navigator.userAgent || "";
  const platform = (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData?.platform || "";

  if (/win/i.test(platform) || /windows/i.test(userAgent)) {
    return "windows";
  }
  if (/mac/i.test(platform) || /macintosh|mac os x/i.test(userAgent)) {
    return "macos";
  }
  if (/linux/i.test(platform) || /linux|x11/i.test(userAgent)) {
    return "linux";
  }
  return "windows"; // Default fallback
}

export function useReleaseInfo(): ReleaseInfo {
  const [os, setOs] = useState<DetectedOS>("windows");
  const [version, setVersion] = useState<string>(FALLBACK_VERSION);
  const [releaseUrl, setReleaseUrl] = useState<string>(
    `https://github.com/${REPO}/releases`
  );
  const [installerUrl, setInstallerUrl] = useState<string>(
    `https://github.com/${REPO}/releases/download/${FALLBACK_VERSION}/PhotoCraft-Companion-Setup-Windows-x64.exe`
  );
  const [installerName, setInstallerName] = useState<string>(
    "PhotoCraft-Companion-Setup-Windows-x64.exe"
  );
  const [portableUrl, setPortableUrl] = useState<string>(
    `https://github.com/${REPO}/releases/download/${FALLBACK_VERSION}/PhotoCraft-Companion-${FALLBACK_VERSION}-windows-x64-portable.zip`
  );
  const [portableName, setPortableName] = useState<string>(
    `PhotoCraft-Companion-${FALLBACK_VERSION}-windows-x64-portable.zip`
  );
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    const detected = detectUserOS();
    setOs(detected);

    let isMounted = true;

    async function fetchLatestRelease() {
      try {
        const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
          headers: {
            Accept: "application/vnd.github.v3+json",
          },
        });

        if (!res.ok) {
          throw new Error(`GitHub API error: ${res.status}`);
        }

        const data = await res.json();
        const tag: string = data.tag_name || FALLBACK_VERSION;
        const htmlUrl: string = data.html_url || `https://github.com/${REPO}/releases/tag/${tag}`;
        const assets: Array<{ name: string; browser_download_url: string }> = data.assets || [];

        if (!isMounted) return;

        setVersion(tag);
        setReleaseUrl(htmlUrl);

        // Determine installer & portable based on OS and available assets
        let instUrl = "";
        let instName = "";
        let portUrl = "";
        let portName = "";

        if (detected === "windows") {
          const winInstaller = assets.find((a) =>
            /setup.*\.exe$/i.test(a.name) || a.name.includes("Setup")
          );
          const winPortable = assets.find((a) =>
            /portable.*\.zip$/i.test(a.name) || (a.name.includes("portable") && a.name.endsWith(".zip"))
          );

          instUrl = winInstaller?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-Setup-Windows-x64.exe`;
          instName = winInstaller?.name || "PhotoCraft-Companion-Setup-Windows-x64.exe";

          portUrl = winPortable?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-${tag}-windows-x64-portable.zip`;
          portName = winPortable?.name || `PhotoCraft-Companion-${tag}-windows-x64-portable.zip`;
        } else if (detected === "macos") {
          const macDmg = assets.find((a) => a.name.endsWith(".dmg"));
          const macPortable = assets.find((a) => a.name.includes("macos") && a.name.endsWith(".tar.gz"));

          instUrl = macDmg?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-${tag}-macos.dmg`;
          instName = macDmg?.name || `PhotoCraft-Companion-${tag}-macos.dmg`;

          portUrl = macPortable?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-${tag}-macos-portable.tar.gz`;
          portName = macPortable?.name || `PhotoCraft-Companion-${tag}-macos-portable.tar.gz`;
        } else {
          // Linux
          const linuxDeb = assets.find((a) => a.name.endsWith(".deb"));
          const linuxPortable = assets.find((a) => a.name.includes("linux") && a.name.endsWith(".tar.gz"));

          instUrl = linuxDeb?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-${tag}-linux-amd64.deb`;
          instName = linuxDeb?.name || `PhotoCraft-Companion-${tag}-linux-amd64.deb`;

          portUrl = linuxPortable?.browser_download_url ||
            `https://github.com/${REPO}/releases/download/${tag}/PhotoCraft-Companion-${tag}-linux-x86_64-portable.tar.gz`;
          portName = linuxPortable?.name || `PhotoCraft-Companion-${tag}-linux-x86_64-portable.tar.gz`;
        }

        setInstallerUrl(instUrl);
        setInstallerName(instName);
        setPortableUrl(portUrl);
        setPortableName(portName);
      } catch (err) {
        console.warn("Could not fetch latest release from GitHub API, using fallback defaults:", err);
      } finally {
        if (isMounted) setIsLoading(false);
      }
    }

    fetchLatestRelease();

    return () => {
      isMounted = false;
    };
  }, []);

  return {
    version,
    tagName: version,
    releaseUrl,
    os,
    installerUrl,
    installerName,
    portableUrl,
    portableName,
    isLoading,
  };
}

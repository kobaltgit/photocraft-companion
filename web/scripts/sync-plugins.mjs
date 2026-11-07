import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Resolve paths
const repoRoot = path.resolve(__dirname, "../..");
const pluginsDir = path.resolve(repoRoot, "plugins");
const publicDir = path.resolve(__dirname, "../public");
const publicPluginsDir = path.resolve(publicDir, "plugins");
const registryJsonPath = path.resolve(publicDir, "registry.json");
const distRegistryPath = path.resolve(repoRoot, "dist/registry.json");

console.log("=== PhotoCraft Web: Automatic Plugin Synchronization ===");
console.log(`Scanning plugins from: ${pluginsDir}`);

if (!fs.existsSync(pluginsDir)) {
  console.warn(`[WARN] Plugins directory not found at: ${pluginsDir}`);
  process.exit(0);
}

fs.mkdirSync(publicPluginsDir, { recursive: true });

// Check if pre-validated dist/registry.json exists (e.g. from cargo plugin_validator)
let rawRegistry = null;
if (fs.existsSync(distRegistryPath)) {
  try {
    const content = fs.readFileSync(distRegistryPath, "utf-8");
    rawRegistry = JSON.parse(content);
    console.log(`Found compiled dist/registry.json (${rawRegistry.total_plugins || rawRegistry.plugins?.length} plugins)`);
  } catch (e) {
    console.warn(`[WARN] Could not parse dist/registry.json, falling back to scanning plugins/ directory: ${e.message}`);
  }
}

const entries = fs.readdirSync(pluginsDir, { withFileTypes: true });
const scannedPlugins = [];

for (const entry of entries) {
  if (!entry.isDirectory()) continue;
  const dirName = entry.name;
  const dirPath = path.join(pluginsDir, dirName);
  const manifestPath = path.join(dirPath, "plugin.json");
  const iconPath = path.join(dirPath, "icon.svg");

  if (!fs.existsSync(manifestPath)) continue;

  try {
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

    // Copy icon.svg to web/public/plugins/<dirName>.svg
    if (fs.existsSync(iconPath)) {
      const destIcon = path.join(publicPluginsDir, `${dirName}.svg`);
      fs.copyFileSync(iconPath, destIcon);
    }

    // Map manifest to registry entry
    scannedPlugins.push({
      id: manifest.id || `ui.photocraft.plugins.${dirName}`,
      name: manifest.name || dirName,
      version: manifest.version || "1.0.0",
      description: manifest.description || "",
      author: manifest.author || "kobaltGIT",
      plugin_type: manifest.plugin_type || manifest.type || "no_code",
      download_url: `https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/${dirName}.pcpl`,
      icon_url: `https://github.com/kobaltgit/photocraft-companion/raw/main/plugins/${dirName}/icon.svg`,
      models: Array.isArray(manifest.models)
        ? manifest.models.map((m) => ({
            name: m.name || m.role || "",
            required: m.required !== false,
            size_mb: m.size_mb || null,
            url: m.url || null,
            sha256: m.sha256 || null,
          }))
        : [],
    });
  } catch (err) {
    console.error(`[ERROR] Failed to parse manifest in ${dirPath}:`, err.message);
  }
}

// Sort alphabetically by ID
scannedPlugins.sort((a, b) => a.id.localeCompare(b.id));

const finalPlugins = rawRegistry && Array.isArray(rawRegistry.plugins) && rawRegistry.plugins.length >= scannedPlugins.length
  ? rawRegistry.plugins
  : scannedPlugins;

const registryOutput = {
  generated_at: new Date().toISOString(),
  total_plugins: finalPlugins.length,
  plugins: finalPlugins,
};

fs.writeFileSync(registryJsonPath, JSON.stringify(registryOutput, null, 2), "utf-8");
console.log(`[OK] Successfully synchronized ${finalPlugins.length} plugins and icons to web/public/registry.json`);

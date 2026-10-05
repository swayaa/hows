// Facts about the desktop app and its build: app identifier, settings file,
// guides subfolder, export formats of the export sheet, and the minimum Rust
// version.

import { match, readJson } from "./source.mjs";

const tauriConfig = "app/src-tauri/tauri.conf.json";
const settings = "app/src-tauri/src/settings.rs";
const commands = "app/src-tauri/src/commands.rs";
const settingsUi = "app/src/settingsUi.ts";
const manifests = ["core/Cargo.toml", "app/src-tauri/Cargo.toml"];

function rustVersion() {
  const versions = new Set(manifests.map((file) => match(file, /^rust-version = "([^"]+)"/m)));
  if (versions.size !== 1) throw new Error(`${manifests.join(" and ")} name different rust-version values`);
  return [...versions][0];
}

export default function appFacts() {
  const formats = match(settingsUi, /export type ExportFormat = ([^;]+);/);
  return [
    { key: "app.identifier", value: readJson(tauriConfig).identifier, source: tauriConfig },
    { key: "app.settings_file", value: match(settings, /config_dir\.join\("([^"]+)"\)/), source: settings },
    {
      key: "app.guides_subfolder",
      value: match(commands, /fn guides_base_dir[\s\S]*?documents_base_dir\(app\)\?\.join\("([^"]+)"\)/),
      source: commands,
    },
    { key: "app.export_formats", value: [...formats.matchAll(/"([^"]+)"/g)].map((found) => found[1]), source: settingsUi },
    { key: "build.rust_version", value: rustVersion(), source: manifests[0] },
  ];
}

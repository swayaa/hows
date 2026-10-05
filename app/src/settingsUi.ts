/** Settings helpers. */

import type { Messages } from "./locales/index.ts";
import { fillTemplate } from "./template.ts";

/** Values changed through `change_setting`, see `SettingChange` in `settings.rs`. */
export type TunableSettings = {
  typing_pause_ms: number;
  scroll_pause_ms: number;
  recent_limit: number;
  /** Empty means the title template of the current language. */
  title_template: string;
  file_name_template: string;
  pdf_paper: string;
  pdf_margin_mm: number;
  annotation_color: string;
  annotation_stroke: number;
  /** Preset code from `brands.json`. */
  brand: string;
  /** Own accent `#RRGGBB`; empty means the preset accent. */
  accent_color: string;
  /** "Created with Hows" line in HTML, PDF, and Markdown exports. */
  export_credit: boolean;
};

export type TunableField = keyof TunableSettings;

/** One change, serialized like the backend's `{ field, value }` enum. */
export type SettingChange = {
  [Field in TunableField]: { field: Field; value: TunableSettings[Field] };
}[TunableField];

/** `get_settings` / `set_*_folder` / `change_setting` payload from the Tauri backend. */
export type SettingsView = TunableSettings & {
  hotkey: string;
  pause_hotkey: string;
  /** `system` (follow Windows) or a code from `UI_LOCALES`. */
  language: string;
  /** The code `language` stands for right now, with `system` resolved. */
  resolved_language: string;
  export_format: string;
  open_after_export: boolean;
  theme: string;
  export_folder_display: string;
  export_folder_fallback: boolean;
  export_folder_synced: boolean;
  guides_folder_display: string;
  guides_folder_fallback: boolean;
  guides_folder_synced: boolean;
  /** Hotkey slots (`record` | `pause`) another app held at startup. */
  unregistered_hotkeys: string[];
  /** Title template of the current language, used while `title_template` is empty. */
  language_title_template: string;
  /** `{date}` and `{time}` for now, formatted by the backend like in real titles. */
  preview_date: string;
  preview_time: string;
};

/** `language` value that follows the Windows display language. */
export const SYSTEM_LANGUAGE = "system";

export type NumberLimit = { min: number; max: number; step: number };

export type NumericField =
  | "typing_pause_ms"
  | "scroll_pause_ms"
  | "recent_limit"
  | "pdf_margin_mm"
  | "annotation_stroke";

/** `get_settings_defaults`: the only source of default values and limits. */
export type SettingsDefaults = {
  defaults: TunableSettings;
  limits: Record<NumericField, NumberLimit>;
  pdf_papers: string[];
};

/** Title a new recording would get now, as `guide_title_from` fills it. */
export function exampleTitle(view: SettingsView, app: string): string {
  const template = view.title_template.trim() || view.language_title_template;
  return fillTemplate(template, { app, date: view.preview_date, time: view.preview_time }).trim();
}

/** File name (without extension) a guide would get now, as `file_stem` fills it. */
export function exampleFileName(view: SettingsView, title: string, app: string): string {
  return fillTemplate(view.file_name_template, {
    title,
    app,
    date: view.preview_date,
    time: view.preview_time,
  }).trim();
}

export function isDefault(
  view: TunableSettings,
  defaults: SettingsDefaults | null,
  field: TunableField,
): boolean {
  return defaults === null || view[field] === defaults.defaults[field];
}

/** The change that puts `field` back to its default; `null` until the defaults are loaded. */
export function defaultChange(defaults: SettingsDefaults | null, field: TunableField): SettingChange | null {
  return defaults === null ? null : ({ field, value: defaults.defaults[field] } as SettingChange);
}

/** Name of `code` in a table of localized names, the code itself when the table has none. */
export function localizedName<Names extends Record<string, string>>(names: Names, code: string): string {
  return Object.hasOwn(names, code) ? names[code as keyof Names] : code;
}

/** Until `get_settings` answers: neutral placeholders, not the backend defaults. */
export const DEFAULT_SETTINGS_VIEW: SettingsView = {
  hotkey: "",
  pause_hotkey: "",
  language: "",
  resolved_language: "",
  export_format: "html",
  open_after_export: true,
  theme: "system",
  export_folder_display: "",
  export_folder_fallback: false,
  export_folder_synced: false,
  guides_folder_display: "",
  guides_folder_fallback: false,
  guides_folder_synced: false,
  unregistered_hotkeys: [],
  language_title_template: "",
  preview_date: "",
  preview_time: "",
  typing_pause_ms: 0,
  scroll_pause_ms: 0,
  recent_limit: 0,
  title_template: "",
  file_name_template: "",
  pdf_paper: "",
  pdf_margin_mm: 0,
  annotation_color: "",
  annotation_stroke: 0,
  brand: "",
  accent_color: "",
  export_credit: false,
};

export type ExportFormat = "html" | "pdf" | "steps";

export function asExportFormat(raw: string): ExportFormat {
  return raw === "pdf" || raw === "steps" ? raw : "html";
}

/** HTML and PDF land in the export folder, `.steps` in the guides folder. */
export function exportFolderKind(format: ExportFormat): "export" | "guides" {
  switch (format) {
    case "html":
    case "pdf":
      return "export";
    case "steps":
      return "guides";
    default: {
      const _exhaustive: never = format;
      return _exhaustive;
    }
  }
}

export type ThemePreference = "system" | "light" | "dark";

export function asThemePreference(raw: string): ThemePreference {
  return raw === "light" || raw === "dark" || raw === "system" ? raw : "system";
}

/**
 * Shortens a folder path by leaving out whole folders in the middle, so the
 * drive and the folder the path names stay readable: `C:\…\Documents\Hows`.
 */
export function shortenPath(path: string, max = 30): string {
  const trimmed = path.trim();
  if (trimmed.length <= max) return trimmed;
  const separator = trimmed.includes("\\") ? "\\" : "/";
  const parts = trimmed.split(/[\\/]/).filter((part, index) => part !== "" || index === 0);
  const head = `${parts[0]}${separator}…${separator}`;
  let tail = parts[parts.length - 1];
  if (head.length + tail.length > max) {
    return `…${separator}${tail.slice(0, Math.max(1, max - 3))}…`;
  }
  for (let index = parts.length - 2; index > 0; index--) {
    const longer = `${parts[index]}${separator}${tail}`;
    if (head.length + longer.length > max) break;
    tail = longer;
  }
  return `${head}${tail}`;
}

/**
 * A setting number in the UI language's notation, 1.5 in English and 1,5 in German.
 * WebView2 formats `type="number"` fields by the Windows language instead.
 */
export function formatSettingNumber(value: number, localeCode: string): string {
  return new Intl.NumberFormat(localeCode, { maximumFractionDigits: 3, useGrouping: false }).format(value);
}

/** Reads a typed setting number with either decimal mark; `null` when it is none. */
export function parseSettingNumber(text: string): number | null {
  const normalized = text.replace(/\s/g, "").replace(",", ".");
  return /^-?(\d+(\.\d*)?|\.\d+)$/.test(normalized) ? Number(normalized) : null;
}

/** Display a hotkey chord with the locale's key names, Ctrl as Strg and Shift as Umschalt. */
export function formatHotkeyLabel(chord: string, keyNames: Messages["keyNames"]): string {
  return chord
    .split("+")
    .map((part) => {
      const token = part.trim();
      switch (token.toLowerCase()) {
        case "ctrl":
        case "control":
          return keyNames.ctrl;
        case "shift":
          return keyNames.shift;
        case "alt":
          return keyNames.alt;
        case "meta":
        case "win":
        case "super":
          return keyNames.win;
        default:
          return token.length === 1 ? token.toUpperCase() : token;
      }
    })
    .join("+");
}

/** Build canonical chord from a KeyboardEvent (Ctrl+Shift+R form). */
export function chordFromKeyboardEvent(event: KeyboardEvent): string | null {
  const key = event.key;
  if (!key || key === "Escape") return null;
  // Modifier-only presses are not a complete chord.
  if (
    key === "Control" ||
    key === "Shift" ||
    key === "Alt" ||
    key === "Meta"
  ) {
    return null;
  }
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Win");
  let main = key.length === 1 ? key.toUpperCase() : key;
  if (main === " ") main = "Space";
  parts.push(main);
  return parts.join("+");
}

/** Resolve active skin from preference + OS media query. */
export function resolveThemeSkin(
  preference: ThemePreference,
  systemDark: boolean,
): "light" | "dark" {
  switch (preference) {
    case "light":
      return "light";
    case "dark":
      return "dark";
    case "system":
      return systemDark ? "dark" : "light";
    default: {
      const _exhaustive: never = preference;
      return _exhaustive;
    }
  }
}

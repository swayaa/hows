/** Window chrome ("Soft Ledger"): surfaces, window titles and text helpers. */

import type { Messages } from "./locales/index.ts";
import { fillTemplate } from "./template.ts";

export type RecorderPhase = "idle" | "recording" | "paused" | "reviewing";

/** Primary Soft Ledger surfaces. Library, then capture, then editor, then the export sheet, then settings. */
export type ChromeSurface =
  | "library"
  | "capture"
  | "editor"
  | "settings";

export type ChromeResolveInput = {
  phase: RecorderPhase;
  settingsOpen: boolean;
};

/**
 * Resolve which Soft Ledger surface owns the first viewport.
 * Settings overlays Library/Editor; Capture while recording/paused;
 * Editor when reviewing (incl. empty Review); Library when idle.
 */
export function resolveChromeSurface(input: ChromeResolveInput): ChromeSurface {
  if (input.settingsOpen) return "settings";
  switch (input.phase) {
    case "recording":
    case "paused":
      return "capture";
    case "reviewing":
      return "editor";
    case "idle":
      return "library";
    default: {
      const _exhaustive: never = input.phase;
      return _exhaustive;
    }
  }
}

/** Annotate is a mode on the editor. The control shows only when the mode is on and a screenshot is ready. */
export function showAnnotatePill(opts: {
  surface: ChromeSurface;
  annotateMode: boolean;
  hasImage: boolean;
  imageLoaded: boolean;
}): boolean {
  return (
    opts.surface === "editor" &&
    opts.annotateMode &&
    opts.hasImage &&
    opts.imageLoaded
  );
}

/** Document / window title per surface. */
export function windowTitleForSurface(surface: ChromeSurface, messages: Messages): string {
  return `Hows · ${messages.windowTitles[surface]}`;
}

/** `{n}`/`{m}` template fill for count strings. */
export function fillCount(template: string, n: number, m: number): string {
  return fillTemplate(template, { n, m });
}

/** "3 Schritte" / "1 step" from the copy's singular/plural pair. */
export function stepCountLabel(copy: Messages, count: number): string {
  return `${count} ${count === 1 ? copy.stepOne : copy.stepMany}`;
}

const ARIA_KEY_NAMES: Record<string, string> = {
  ctrl: "Control",
  control: "Control",
  shift: "Shift",
  alt: "Alt",
  win: "Meta",
  meta: "Meta",
  super: "Meta",
};

/** A chord from the settings as `aria-keyshortcuts` value: "Ctrl+Shift+F9" becomes "Control+Shift+F9". */
export function ariaKeyShortcuts(chord: string): string {
  return chord
    .split("+")
    .map((part) => part.trim())
    .map((part) => ARIA_KEY_NAMES[part.toLowerCase()] ?? part.toUpperCase())
    .join("+");
}

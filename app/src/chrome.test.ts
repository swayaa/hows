import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ariaKeyShortcuts,
  fillCount,
  resolveChromeSurface,
  showAnnotatePill,
  stepCountLabel,
  windowTitleForSurface,
} from "./chrome.ts";
import { uiLocale } from "./locales/index.ts";

const de = uiLocale("de").messages;
const en = uiLocale("en").messages;

describe("Soft Ledger chrome surface", () => {
  it("routes Library / Capture / Editor / Settings", () => {
    assert.equal(resolveChromeSurface({ phase: "idle", settingsOpen: false }), "library");
    assert.equal(
      resolveChromeSurface({ phase: "recording", settingsOpen: false }),
      "capture",
    );
    assert.equal(resolveChromeSurface({ phase: "paused", settingsOpen: false }), "capture");
    assert.equal(resolveChromeSurface({ phase: "reviewing", settingsOpen: false }), "editor");
    assert.equal(resolveChromeSurface({ phase: "idle", settingsOpen: true }), "settings");
  });

  it("shows annotate pill only in Editor annotate mode with loaded shot", () => {
    assert.equal(
      showAnnotatePill({
        surface: "editor",
        annotateMode: true,
        hasImage: true,
        imageLoaded: true,
      }),
      true,
    );
    assert.equal(
      showAnnotatePill({
        surface: "editor",
        annotateMode: false,
        hasImage: true,
        imageLoaded: true,
      }),
      false,
    );
  });

  it("fills count templates and pluralizes steps", () => {
    assert.equal(fillCount(de.stepPosition, 2, 5), "Schritt 2 von 5");
    assert.equal(stepCountLabel(de, 1), "1 Schritt");
    assert.equal(stepCountLabel(de, 3), "3 Schritte");
    assert.equal(stepCountLabel(en, 1), "1 step");
  });
  it("turns configured chords into aria-keyshortcuts values", () => {
    assert.equal(ariaKeyShortcuts("Ctrl+Shift+F9"), "Control+Shift+F9");
    assert.equal(ariaKeyShortcuts("control+alt+win+x"), "Control+Alt+Meta+X");
    assert.equal(ariaKeyShortcuts("Alt+P"), "Alt+P");
  });

  it("uses per-surface window titles", () => {
    assert.equal(windowTitleForSurface("library", de), "Hows · Bibliothek");
    assert.equal(windowTitleForSurface("editor", de), "Hows · Anleitung bearbeiten");
    assert.equal(windowTitleForSurface("settings", en), "Hows · Settings");
  });
});

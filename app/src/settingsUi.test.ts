import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  asExportFormat,
  chordFromKeyboardEvent,
  exportFolderKind,
  formatHotkeyLabel,
  formatSettingNumber,
  parseSettingNumber,
  resolveThemeSkin,
  shortenPath,
} from "./settingsUi.ts";
import { uiLocale } from "./locales/index.ts";

describe("settings UI helpers", () => {
  it("maps export formats to their target folder", () => {
    assert.equal(asExportFormat("pdf"), "pdf");
    assert.equal(asExportFormat("steps"), "steps");
    assert.equal(asExportFormat("docx"), "html");
    assert.equal(exportFolderKind("html"), "export");
    assert.equal(exportFolderKind("pdf"), "export");
    assert.equal(exportFolderKind("steps"), "guides");
  });

  it("shortens long paths by whole folders from the middle", () => {
    assert.equal(shortenPath("~/Documents"), "~/Documents");
    assert.equal(shortenPath(String.raw`C:\Users\WDAGUtilityAccount\Documents`), String.raw`C:\…\Documents`);
    assert.equal(
      shortenPath(String.raw`C:\Users\Alex\Documents\Archive`),
      String.raw`C:\…\Alex\Documents\Archive`,
    );
    assert.equal(
      shortenPath("/home/user/Documents/Steps/very/deep/nested/path/guide", 28),
      "/…/deep/nested/path/guide",
    );
    const oneLongFolder = shortenPath(`C:\\${"x".repeat(40)}`);
    assert.equal(oneLongFolder.length, 30);
    assert.ok(oneLongFolder.startsWith("…\\x"));
  });

  it("shows setting numbers in the UI language's notation", () => {
    assert.equal(formatSettingNumber(1.5, "en"), "1.5");
    assert.equal(formatSettingNumber(1.5, "de"), "1,5");
    assert.equal(formatSettingNumber(1.5, "fr"), "1,5");
    assert.equal(formatSettingNumber(2500, "en"), "2500");
    assert.equal(formatSettingNumber(2, "de"), "2");
  });

  it("reads setting numbers typed with either decimal mark", () => {
    assert.equal(parseSettingNumber("1.5"), 1.5);
    assert.equal(parseSettingNumber(" 1,5 "), 1.5);
    assert.equal(parseSettingNumber("12"), 12);
    assert.equal(parseSettingNumber(",5"), 0.5);
    assert.equal(parseSettingNumber(""), null);
    assert.equal(parseSettingNumber("1.2.3"), null);
    assert.equal(parseSettingNumber("abc"), null);
  });

  it("formats hotkey labels with the locale's key names", () => {
    const de = uiLocale("de").messages.keyNames;
    const en = uiLocale("en").messages.keyNames;
    assert.equal(formatHotkeyLabel("Ctrl+Shift+R", de), "Strg+Umschalt+R");
    assert.equal(formatHotkeyLabel("Ctrl+Shift+P", en), "Ctrl+Shift+P");
    assert.equal(formatHotkeyLabel("control+alt+meta+x", en), "Ctrl+Alt+Win+X");
  });

  it("builds chords from keyboard-like events", () => {
    const event = {
      key: "r",
      ctrlKey: true,
      altKey: false,
      shiftKey: true,
      metaKey: false,
    } as KeyboardEvent;
    assert.equal(chordFromKeyboardEvent(event), "Ctrl+Shift+R");
    assert.equal(
      chordFromKeyboardEvent({
        key: "F9",
        ctrlKey: false,
        altKey: false,
        shiftKey: true,
        metaKey: false,
      } as KeyboardEvent),
      "Shift+F9",
    );
    assert.equal(
      chordFromKeyboardEvent({
        key: "F9",
        ctrlKey: false,
        altKey: false,
        shiftKey: false,
        metaKey: false,
      } as KeyboardEvent),
      "F9",
    );
    assert.equal(
      chordFromKeyboardEvent({
        key: "Escape",
        ctrlKey: false,
        altKey: false,
        shiftKey: false,
        metaKey: false,
      } as KeyboardEvent),
      null,
    );
  });

  it("resolves theme skin", () => {
    assert.equal(resolveThemeSkin("light", true), "light");
    assert.equal(resolveThemeSkin("dark", false), "dark");
    assert.equal(resolveThemeSkin("system", true), "dark");
    assert.equal(resolveThemeSkin("system", false), "light");
  });
});

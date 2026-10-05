import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { uiLocale } from "./locales/index.ts";
import {
  DEFAULT_SETTINGS_VIEW,
  defaultChange,
  exampleFileName,
  exampleTitle,
  isDefault,
  localizedName,
  type SettingsDefaults,
} from "./settingsUi.ts";
import { fillTemplate } from "./template.ts";

const en = uiLocale("en").messages;

describe("templates", () => {
  it("fills in one pass and keeps unknown placeholders", () => {
    assert.equal(
      fillTemplate(en.stepHeading, { n: 2, text: "Type {n} in {m}" }),
      "Step 2: Type {n} in {m}",
    );
    assert.equal(fillTemplate("{a} {b}", { a: 1 }), "1 {b}");
  });

  it("previews the title with the language template while the own one is empty", () => {
    const view = {
      ...DEFAULT_SETTINGS_VIEW,
      language_title_template: "{app} guide ({date})",
      file_name_template: "{date} {title}",
      preview_date: "2026-09-30",
      preview_time: "14.05",
    };
    assert.equal(exampleTitle(view, "Notepad"), "Notepad guide (2026-09-30)");
    const own = { ...view, title_template: " {app} at {time} " };
    assert.equal(exampleTitle(own, "Notepad"), "Notepad at 14.05");
    assert.equal(exampleFileName(view, "Notepad guide", "Notepad"), "2026-09-30 Notepad guide");
    const timed = { ...view, file_name_template: "{app} {date} {time}" };
    assert.equal(exampleFileName(timed, "Notepad guide", "Notepad"), "Notepad 2026-09-30 14.05");
  });

  it("offers every placeholder in the help text of every language", () => {
    for (const code of ["en", "de", "fr", "es", "it", "pt", "nl"]) {
      const { messages } = uiLocale(code);
      for (const name of ["{app}", "{date}", "{time}"]) {
        assert.ok(messages.titleTemplateHelp.includes(name), `${code} title ${name}`);
      }
      for (const name of ["{title}", "{app}", "{date}", "{time}"]) {
        assert.ok(messages.fileNameTemplateHelp.includes(name), `${code} file ${name}`);
      }
    }
  });

  it("keeps backend defaults out of the placeholder view", () => {
    assert.equal(DEFAULT_SETTINGS_VIEW.brand, "");
    assert.equal(DEFAULT_SETTINGS_VIEW.language, "");
    assert.equal(DEFAULT_SETTINGS_VIEW.export_credit, false);
    assert.equal(DEFAULT_SETTINGS_VIEW.accent_color, "");
  });

  it("names codes from a table and falls back to the code", () => {
    assert.equal(localizedName(en.paperNames, "a4"), en.paperNames.a4);
    assert.equal(localizedName(en.brandNames, "neon"), "neon");
    assert.equal(localizedName(en.brandNames, "toString"), "toString");
  });

  it("compares a value with the backend default", () => {
    const defaults: SettingsDefaults = {
      defaults: { ...DEFAULT_SETTINGS_VIEW, recent_limit: 8 },
      limits: {
        typing_pause_ms: { min: 0, max: 1, step: 1 },
        scroll_pause_ms: { min: 0, max: 1, step: 1 },
        recent_limit: { min: 1, max: 30, step: 1 },
        pdf_margin_mm: { min: 0, max: 1, step: 1 },
        annotation_stroke: { min: 0, max: 1, step: 1 },
      },
      pdf_papers: ["a4"],
    };
    const view = { ...DEFAULT_SETTINGS_VIEW, recent_limit: 8 };
    assert.equal(isDefault(view, defaults, "recent_limit"), true);
    assert.equal(isDefault({ ...view, recent_limit: 3 }, defaults, "recent_limit"), false);
    assert.equal(isDefault(view, null, "recent_limit"), true);
    assert.deepEqual(defaultChange(defaults, "accent_color"), { field: "accent_color", value: "" });
    assert.deepEqual(defaultChange(defaults, "recent_limit"), { field: "recent_limit", value: 8 });
    assert.equal(defaultChange(null, "brand"), null);
  });
});

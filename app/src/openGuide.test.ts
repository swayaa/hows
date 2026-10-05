import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { uiLocale } from "./locales/index.ts";
import { recentFileLabel, recentMeta, recentTitle, type RecentGuideEntry } from "./openGuide.ts";

const de = uiLocale("de").messages;
const en = uiLocale("en").messages;

describe("open guide copy", () => {
  it("recent label is basename", () => {
    assert.equal(recentFileLabel(String.raw`C:\Guides\demo.steps`), "demo.steps");
    assert.equal(recentFileLabel("/tmp/a.steps"), "a.steps");
    assert.equal(recentFileLabel("plain"), "plain");
  });

  const entry: RecentGuideEntry = {
    path: String.raw`C:\Guides\demo.steps`,
    missing: false,
    unreadable: false,
    title: "Connect a network drive",
    step_count: 5,
    // 2026-09-30 12:00 UTC
    created_at_ms: Date.UTC(2026, 8, 30, 12),
  };

  it("titles a card by the guide's title, else the file name", () => {
    assert.equal(recentTitle(entry), "Connect a network drive");
    assert.equal(recentTitle({ ...entry, title: null }), "demo.steps");
  });

  it("describes a card by step count and date in the UI language", () => {
    assert.equal(recentMeta(entry, en, "en"), "5 steps · Sep 30, 2026");
    assert.equal(recentMeta(entry, de, "de"), "5 Schritte · 30.09.2026");
    assert.equal(recentMeta({ ...entry, step_count: 1, created_at_ms: null }, en, "en"), "1 step");
    assert.equal(recentMeta({ ...entry, missing: true }, en, "en"), en.missing);
    assert.equal(recentMeta({ ...entry, unreadable: true }, de, "de"), de.recentUnreadable);
  });
});

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { confirmIntent, stepTextChanged, stepTextConfirms } from "./unsavedWork.ts";

const editedGuide = { dirty: true, phase: "reviewing", capturedSteps: 0 } as const;
const cleanGuide = { dirty: false, phase: "reviewing", capturedSteps: 0 } as const;

function asker(answer: boolean) {
  const asked: string[] = [];
  return {
    asked,
    ask: async (intent: string) => {
      asked.push(intent);
      return answer;
    },
  };
}

describe("unsaved work policy", () => {
  it("asks before a new recording replaces an edited guide", async () => {
    const { asked, ask } = asker(true);
    assert.equal(await confirmIntent("start", editedGuide, ask), true);
    assert.deepEqual(asked, ["start"]);
  });

  it("asks before tray quit loses an edited guide", async () => {
    const { asked, ask } = asker(true);
    assert.equal(await confirmIntent("quit", editedGuide, ask), true);
    assert.deepEqual(asked, ["quit"]);
  });

  it("asks before opening another guide or going back", async () => {
    const { asked, ask } = asker(true);
    await confirmIntent("open", editedGuide, ask);
    await confirmIntent("discard", editedGuide, ask);
    assert.deepEqual(asked, ["open", "discard"]);
  });

  it("cancel keeps the guide", async () => {
    const { ask } = asker(false);
    assert.equal(await confirmIntent("start", editedGuide, ask), false);
    assert.equal(await confirmIntent("quit", editedGuide, ask), false);
  });

  it("a clean guide does not ask", async () => {
    const { asked, ask } = asker(false);
    assert.equal(await confirmIntent("start", cleanGuide, ask), true);
    assert.equal(await confirmIntent("quit", cleanGuide, ask), true);
    assert.equal(await confirmIntent("open", cleanGuide, ask), true);
    assert.deepEqual(asked, []);
  });

  it("pause, resume and stop never ask", async () => {
    const { asked, ask } = asker(false);
    assert.equal(await confirmIntent("pause", editedGuide, ask), true);
    assert.equal(await confirmIntent("resume", editedGuide, ask), true);
    assert.equal(await confirmIntent("stop", editedGuide, ask), true);
    assert.deepEqual(asked, []);
  });

  it("an earlier edit does not ask while a recording runs", async () => {
    const { asked, ask } = asker(false);
    assert.equal(
      await confirmIntent("quit", { dirty: true, phase: "recording", capturedSteps: 0 }, ask),
      true,
    );
    assert.deepEqual(asked, []);
  });

  it("quit asks before it drops a recording that already has steps", async () => {
    const { asked, ask } = asker(true);
    assert.equal(
      await confirmIntent("quit", { dirty: false, phase: "recording", capturedSteps: 2 }, ask),
      true,
    );
    assert.equal(
      await confirmIntent("quit", { dirty: false, phase: "paused", capturedSteps: 1 }, ask),
      true,
    );
    assert.deepEqual(asked, ["quit", "quit"]);
  });

  it("cancel keeps a recording that already has steps", async () => {
    const { ask } = asker(false);
    assert.equal(
      await confirmIntent("quit", { dirty: false, phase: "paused", capturedSteps: 1 }, ask),
      false,
    );
  });
});

describe("step text edits", () => {
  const step = { text: "Click Save" };

  it("unchanged text is not an edit", () => {
    assert.equal(stepTextChanged(step, "Click Save"), false);
    assert.equal(stepTextChanged(step, "Click Save as"), true);
  });

  it("Enter keeps the text and Shift+Enter stays a new line", () => {
    const plain = { key: "Enter", shiftKey: false, isComposing: false };
    assert.equal(stepTextConfirms(plain), true);
    assert.equal(stepTextConfirms({ ...plain, shiftKey: true }), false);
    assert.equal(stepTextConfirms({ ...plain, isComposing: true }), false);
    assert.equal(stepTextConfirms({ ...plain, key: "a" }), false);
  });
});

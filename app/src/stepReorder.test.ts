import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { gapFromClientX, moveTargetForGap, slotAfterKeyboard } from "./stepReorder.ts";

describe("step reorder gap → move_step index", () => {
  it("maps insert-before gaps to final indices", () => {
    assert.equal(moveTargetForGap(1, 3), 2);
    assert.equal(moveTargetForGap(1, 1), null);
    assert.equal(moveTargetForGap(1, 0), 0);
    assert.equal(moveTargetForGap(0, 4), 3);
    assert.equal(moveTargetForGap(2, 2), null);
    assert.equal(moveTargetForGap(3, 1), 1);
  });

  it("keyboard one-slot stays in bounds", () => {
    assert.equal(slotAfterKeyboard(2, "up", 4), 1);
    assert.equal(slotAfterKeyboard(2, "down", 4), 3);
    assert.equal(slotAfterKeyboard(0, "up", 4), null);
    assert.equal(slotAfterKeyboard(3, "down", 4), null);
    assert.equal(slotAfterKeyboard(0, "down", 1), null);
    assert.equal(slotAfterKeyboard(1, "up", 0), null);
  });

  it("gapFromClientX uses column midpoints (Filmstreifen)", () => {
    const cols = [
      { left: 0, right: 40 },
      { left: 40, right: 80 },
      { left: 80, right: 120 },
    ];
    assert.equal(gapFromClientX(-10, cols), 0);
    assert.equal(gapFromClientX(10, cols), 0);
    assert.equal(gapFromClientX(30, cols), 1);
    assert.equal(gapFromClientX(59, cols), 1);
    assert.equal(gapFromClientX(60, cols), 2);
    assert.equal(gapFromClientX(99, cols), 2);
    assert.equal(gapFromClientX(100, cols), 3);
    assert.equal(gapFromClientX(200, cols), 3);
    assert.equal(gapFromClientX(0, []), 0);
  });
});

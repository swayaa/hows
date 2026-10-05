import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { clampCrop, hitCropHandle, moveCrop, resizeCrop } from "./crop.ts";

describe("crop", () => {
  it("keeps a dragged window inside the picture", () => {
    assert.deepEqual(clampCrop({ x: -0.2, y: 0.1, w: 0.4, h: 0.3 }), {
      x: 0,
      y: 0.1,
      w: 0.4,
      h: 0.3,
    });
    assert.deepEqual(moveCrop({ x: 0.2, y: 0.2, w: 0.5, h: 0.5 }, 0.9, 0), {
      x: 0.5,
      y: 0.2,
      w: 0.5,
      h: 0.5,
    });
  });

  it("pulls the west edge and flips a drag that crosses the opposite edge", () => {
    assert.deepEqual(resizeCrop({ x: 0.2, y: 0.2, w: 0.4, h: 0.4 }, "nw", 0.4, 0.2), {
      x: 0.4,
      y: 0.2,
      w: 0.2,
      h: 0.4,
    });
    assert.equal(resizeCrop({ x: 0.2, y: 0.2, w: 0.4, h: 0.4 }, "se", 0.1, 0.6).x, 0.1);
  });

  it("hits a corner before the inside of the frame", () => {
    const crop = { x: 0.2, y: 0.2, w: 0.4, h: 0.4 };
    assert.equal(hitCropHandle(crop, { x: 0.2, y: 0.2 }, { width: 400, height: 300 }), "nw");
    assert.equal(hitCropHandle(crop, { x: 0.4, y: 0.4 }, { width: 400, height: 300 }), "move");
  });
});

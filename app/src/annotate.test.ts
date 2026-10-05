import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  COLOR_PRESETS,
  MARKS,
  arrowHeadPoints,
  circleFromCornerDrag,
  circleRadii,
  hitTestHandle,
  hitTestOverlay,
  moveOverlay,
  paintOrder,
  penStroke,
  recolorOverlay,
  resizeOverlay,
  type OverlayDto,
} from "./annotate.ts";

const size = { width: 200, height: 100 };
const OTHER = COLOR_PRESETS[1];

describe("paintOrder", () => {
  it("puts blur under highlight under ink", () => {
    const marks: OverlayDto[] = [
      {
        type: "rect",
        id: "r",
        color: MARKS.color,
        stroke: 2.5,
        x: 0,
        y: 0,
        w: 0.1,
        h: 0.1,
      },
      {
        type: "highlight",
        id: "h",
        color: MARKS.highlight_color,
        opacity: 0.4,
        x: 0,
        y: 0,
        w: 0.1,
        h: 0.1,
      },
      { type: "blur", id: "b", x: 0, y: 0, w: 0.1, h: 0.1 },
    ];
    assert.deepEqual(
      paintOrder(marks).map((m) => m.type),
      ["blur", "highlight", "rect"],
    );
  });
});

describe("circle geometry", () => {
  it("builds true circle from corner box via min side in pixels", () => {
    const c = circleFromCornerDrag(0.1, 0.1, 0.5, 0.9, size);
    // width 200, height 100, so dx is 80 px and dy is 80 px, side 80, r is 40/200 = 0.2
    assert.ok(Math.abs(c.r - 0.2) < 1e-9);
    assert.ok(Math.abs(c.cx - 0.3) < 1e-9);
  });

  it("circleRadii keep visual roundness on wide shots", () => {
    const { rx, ry } = circleRadii(0.1, size);
    assert.equal(rx, 0.1);
    assert.equal(ry, 0.2); // 0.1 * 200/100
  });
});

describe("mark defaults", () => {
  it("offers the default color first and no color twice", () => {
    assert.equal(COLOR_PRESETS[0], MARKS.color);
    assert.equal(new Set(COLOR_PRESETS).size, COLOR_PRESETS.length);
  });

  it("draws the pen at 3 px for the default line width, as before", () => {
    assert.equal(penStroke(MARKS.stroke), 3);
  });

  it("scales label text from marks.json and keeps it within its limits", () => {
    const label: OverlayDto = {
      type: "text",
      id: "t1",
      color: MARKS.color,
      size: MARKS.text_size,
      x: 0.1,
      y: 0.1,
      text: "Hi",
    };
    const resizeTo = (x: number) =>
      resizeOverlay(label, { kind: "text", handle: "se" }, { x, y: 0.5 }, size);
    const sizeOf = (mark: OverlayDto) => (mark.type === "text" ? mark.size : NaN);
    const { drag_width, min, max } = MARKS.text_resize;
    assert.ok(Math.abs(sizeOf(resizeTo(0.1 + drag_width)) - MARKS.text_size) < 1e-9);
    assert.equal(sizeOf(resizeTo(0.1)), min);
    assert.equal(sizeOf(resizeTo(1)), max);
    assert.deepEqual([drag_width, min, max], [0.12, 10, 48]);
  });

  it("hits a label inside the box estimated from marks.json", () => {
    const label: OverlayDto = {
      type: "text",
      id: "t2",
      color: MARKS.color,
      size: 20,
      x: 0.1,
      y: 0.1,
      text: "abcdefghij",
    };
    const width = (10 * 20 * MARKS.text_box.char_width) / size.width;
    assert.equal(hitTestOverlay([label], { x: 0.1 + width * 0.9, y: 0.15 }, size)?.id, "t2");
    assert.equal(hitTestOverlay([label], { x: 0.1 + width * 1.5, y: 0.15 }, size), null);
  });
});

describe("arrow head", () => {
  it("returns three points for a non-zero arrow", () => {
    const pts = arrowHeadPoints(0.1, 0.5, 0.9, 0.5, size);
    const parts = pts.split(/\s+/);
    assert.equal(parts.length, 3);
  });
});

describe("hit / transform", () => {
  const rect: OverlayDto = {
    type: "rect",
    id: "r1",
    color: MARKS.color,
    stroke: 2.5,
    x: 0.2,
    y: 0.2,
    w: 0.3,
    h: 0.3,
  };

  it("hits rect body and se handle", () => {
    assert.equal(hitTestOverlay([rect], { x: 0.35, y: 0.35 }, size)?.id, "r1");
    const handle = hitTestHandle(rect, { x: 0.5, y: 0.5 }, size);
    assert.deepEqual(handle, { kind: "box", handle: "se" });
  });

  it("moves and resizes rect", () => {
    const moved = moveOverlay(rect, 0.05, -0.05);
    assert.equal(moved.type, "rect");
    if (moved.type === "rect") {
      assert.ok(Math.abs(moved.x - 0.25) < 1e-9);
      assert.ok(Math.abs(moved.y - 0.15) < 1e-9);
    }
    const resized = resizeOverlay(
      rect,
      { kind: "box", handle: "se" },
      { x: 0.6, y: 0.7 },
      size,
    );
    assert.equal(resized.type, "rect");
    if (resized.type === "rect") {
      assert.ok(Math.abs(resized.w - 0.4) < 1e-9);
      assert.ok(Math.abs(resized.h - 0.5) < 1e-9);
    }
  });

  it("moves whole pen polyline", () => {
    const pen: OverlayDto = {
      type: "pen",
      id: "p1",
      color: OTHER,
      stroke: 3,
      points: [
        [0.1, 0.1],
        [0.2, 0.2],
      ],
    };
    const moved = moveOverlay(pen, 0.05, 0.05);
    assert.equal(moved.type, "pen");
    if (moved.type === "pen") {
      assert.ok(Math.abs(moved.points[0][0] - 0.15) < 1e-9);
      assert.ok(Math.abs(moved.points[0][1] - 0.15) < 1e-9);
      assert.deepEqual(moved.points[1], [0.25, 0.25]);
    }
  });

  it("recolors marks but not blur", () => {
    const next = recolorOverlay(rect, OTHER);
    assert.equal(next.type === "rect" ? next.color : "", OTHER);
    const blur: OverlayDto = { type: "blur", id: "b", x: 0, y: 0, w: 0.1, h: 0.1 };
    assert.equal(recolorOverlay(blur, OTHER), blur);
  });
});

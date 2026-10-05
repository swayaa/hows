/**
 * Annotate helpers. Geometry runs from 0 to 1, and redraw stays sharp at any DPI.
 *
 * Coordinate contract: UI norms against the screenshot *content* box (`.shot`
 * shrink-wraps `<img>`, no object-fit padding). Export resolves the same 0 to 1
 * against original PNG width/height. Arrow heads / circles convert via shot
 * CSS size in Review and PNG size at burn-in so aspect stays sharp.
 */

import marks from "../../core/store/marks.json" with { type: "json" };

export type OverlayDto =
  | {
      type: "rect";
      id: string;
      color: string;
      stroke: number;
      x: number;
      y: number;
      w: number;
      h: number;
    }
  | {
      type: "arrow";
      id: string;
      color: string;
      stroke: number;
      x1: number;
      y1: number;
      x2: number;
      y2: number;
    }
  | {
      type: "pen";
      id: string;
      color: string;
      stroke: number;
      points: [number, number][];
    }
  | {
      type: "highlight";
      id: string;
      color: string;
      opacity: number;
      x: number;
      y: number;
      w: number;
      h: number;
    }
  | {
      type: "text";
      id: string;
      color: string;
      size: number;
      x: number;
      y: number;
      text: string;
    }
  | {
      type: "circle";
      id: string;
      color: string;
      stroke: number;
      cx: number;
      cy: number;
      /** Radius relativ zur Bildbreite. */
      r: number;
    }
  | {
      type: "blur";
      id: string;
      x: number;
      y: number;
      w: number;
      h: number;
    };

export type AnnotateTool =
  | "rect"
  | "arrow"
  | "circle"
  | "pen"
  | "highlight"
  | "blur"
  | "text";

export type BoxHandle = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";
export type CircleHandle = "n" | "e" | "s" | "w";
type ArrowHandle = "start" | "end";
type TextHandle = "se";
export type ResizeHandle =
  | { kind: "box"; handle: BoxHandle }
  | { kind: "circle"; handle: CircleHandle }
  | { kind: "arrow"; handle: ArrowHandle }
  | { kind: "text"; handle: TextHandle };

type NormPoint = { x: number; y: number };
export type ShotSize = { width: number; height: number };

/** Mark defaults from `core/store/marks.json`, shared with the backend and the export. */
export const MARKS = marks;
/** Colors offered while marking; the first is the default of the setting. */
export const COLOR_PRESETS: readonly string[] = [marks.color, ...marks.extra_colors];

/** Pen width for a line width; freehand lines need a little more to look as strong. */
export function penStroke(stroke: number): number {
  return stroke * marks.pen_factor;
}

const HIT_PAD_PX = 4;

const BOX_HANDLE_ORDER: BoxHandle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
const CIRCLE_HANDLE_ORDER: CircleHandle[] = ["n", "e", "s", "w"];
const BOX_HANDLE_CURSORS: Record<BoxHandle, string> = {
  nw: "nwse-resize",
  n: "ns-resize",
  ne: "nesw-resize",
  e: "ew-resize",
  se: "nwse-resize",
  s: "ns-resize",
  sw: "nesw-resize",
  w: "ew-resize",
};

/** Paint order is blur, then highlight, then ink. Vectors stay sharp and are never stretched as a bitmap. */
export function paintOrder(marks: OverlayDto[]): OverlayDto[] {
  const blur: OverlayDto[] = [];
  const highlight: OverlayDto[] = [];
  const rest: OverlayDto[] = [];
  for (const mark of marks) {
    if (mark.type === "blur") blur.push(mark);
    else if (mark.type === "highlight") highlight.push(mark);
    else rest.push(mark);
  }
  return [...blur, ...highlight, ...rest];
}

export function overlayPath(points: [number, number][]): string {
  if (points.length === 0) return "";
  return points
    .map((p, i) => `${i === 0 ? "M" : "L"}${p[0]} ${p[1]}`)
    .join(" ");
}

/**
 * Pfeilspitze in Pixelraum rechnen und zurück auf 0 bis 1.
 * Verhindert Aspect-Stretch bei `viewBox="0 0 1 1"` + `preserveAspectRatio="none"`.
 */
export function arrowHeadPoints(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  size: ShotSize,
  strokePx = marks.stroke,
): string {
  const w = Math.max(size.width, 1);
  const h = Math.max(size.height, 1);
  const fx = x1 * w;
  const fy = y1 * h;
  const tx = x2 * w;
  const ty = y2 * h;
  const dx = tx - fx;
  const dy = ty - fy;
  const len = Math.hypot(dx, dy);
  if (len < 1) return `${x2},${y2} ${x2},${y2} ${x2},${y2}`;
  const ux = dx / len;
  const uy = dy / len;
  const head = marks.arrow_head;
  const headLen = Math.min(head.max, Math.max(head.min, strokePx * head.per_stroke));
  const headWidth = headLen * head.width;
  const bx = tx - ux * headLen;
  const by = ty - uy * headLen;
  const px = -uy;
  const py = ux;
  const lx = (bx + px * headWidth) / w;
  const ly = (by + py * headWidth) / h;
  const rx = (bx - px * headWidth) / w;
  const ry = (by - py * headWidth) / h;
  return `${x2},${y2} ${lx},${ly} ${rx},${ry}`;
}

/** Visuell runder Kreis trotz nicht-quadratischem Shot. */
export function circleRadii(rWidthNorm: number, size: ShotSize): { rx: number; ry: number } {
  const w = Math.max(size.width, 1);
  const h = Math.max(size.height, 1);
  return { rx: rWidthNorm, ry: (rWidthNorm * w) / h };
}

export function circleFromCornerDrag(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  size: ShotSize,
): { cx: number; cy: number; r: number } {
  const w = Math.max(size.width, 1);
  const h = Math.max(size.height, 1);
  const dxPx = (x1 - x0) * w;
  const dyPx = (y1 - y0) * h;
  const sidePx = Math.min(Math.abs(dxPx), Math.abs(dyPx));
  const sx = Math.sign(dxPx) || 1;
  const sy = Math.sign(dyPx) || 1;
  const wNorm = sidePx / w;
  const hNorm = sidePx / h;
  const left = sx >= 0 ? x0 : x0 - wNorm;
  const top = sy >= 0 ? y0 : y0 - hNorm;
  return {
    cx: left + wNorm / 2,
    cy: top + hNorm / 2,
    r: sidePx / 2 / w,
  };
}

export function normalizeBox(
  x: number,
  y: number,
  w: number,
  h: number,
): { x: number; y: number; w: number; h: number } {
  return {
    x: Math.min(x, x + w),
    y: Math.min(y, y + h),
    w: Math.abs(w),
    h: Math.abs(h),
  };
}

export function recolorOverlay(mark: OverlayDto, color: string): OverlayDto {
  if (mark.type === "blur") return mark;
  return { ...mark, color };
}

/** Approximate text box in normalized coords (hit-test + SE handle). */
function textNormExtent(
  mark: Extract<OverlayDto, { type: "text" }>,
  size: ShotSize,
): { w: number; h: number } {
  return {
    w: Math.max(0.04, (mark.text.length * mark.size * marks.text_box.char_width) / Math.max(size.width, 1)),
    h: Math.max(0.03, (mark.size * marks.text_box.line_height) / Math.max(size.height, 1)),
  };
}

function hitPad(size: ShotSize, scale = 1): { padX: number; padY: number } {
  return {
    padX: (HIT_PAD_PX * scale) / Math.max(size.width, 1),
    padY: (HIT_PAD_PX * scale) / Math.max(size.height, 1),
  };
}

function distToSegment(
  px: number,
  py: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const len2 = dx * dx + dy * dy;
  if (len2 === 0) return Math.hypot(px - x1, py - y1);
  let t = ((px - x1) * dx + (py - y1) * dy) / len2;
  t = Math.min(1, Math.max(0, t));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
}

function pointInBox(
  p: NormPoint,
  x: number,
  y: number,
  w: number,
  h: number,
  padX: number,
  padY: number,
): boolean {
  const box = normalizeBox(x, y, w, h);
  return (
    p.x >= box.x - padX &&
    p.x <= box.x + box.w + padX &&
    p.y >= box.y - padY &&
    p.y <= box.y + box.h + padY
  );
}

/** Hit-Test top-most zuerst; Pad ≥4 CSS-px. Pen: Treffer = ganze Polyline (Move only). */
export function hitTestOverlay(
  marks: OverlayDto[],
  point: NormPoint,
  size: ShotSize,
): OverlayDto | null {
  const { padX, padY } = hitPad(size);
  const ordered = [...paintOrder(marks)].reverse();
  for (const mark of ordered) {
    if (markHits(mark, point, padX, padY, size)) return mark;
  }
  return null;
}

function markHits(
  mark: OverlayDto,
  p: NormPoint,
  padX: number,
  padY: number,
  size: ShotSize,
): boolean {
  switch (mark.type) {
    case "rect":
    case "highlight":
    case "blur":
      return pointInBox(p, mark.x, mark.y, mark.w, mark.h, padX, padY);
    case "text": {
      const { w, h } = textNormExtent(mark, size);
      return pointInBox(p, mark.x, mark.y, w, h, padX, padY);
    }
    case "circle": {
      const { rx, ry } = circleRadii(mark.r, size);
      const nx = (p.x - mark.cx) / Math.max(rx, 1e-6);
      const ny = (p.y - mark.cy) / Math.max(ry, 1e-6);
      const d = Math.hypot(nx, ny);
      const pad = Math.max(padX / Math.max(rx, 1e-6), padY / Math.max(ry, 1e-6));
      return Math.abs(d - 1) <= 0.2 + pad || d <= 1 + pad * 0.5;
    }
    case "arrow": {
      const d = distToSegment(p.x, p.y, mark.x1, mark.y1, mark.x2, mark.y2);
      return d <= Math.max(padX, padY) * 2.5;
    }
    case "pen": {
      for (let i = 1; i < mark.points.length; i++) {
        const a = mark.points[i - 1];
        const b = mark.points[i];
        if (
          distToSegment(p.x, p.y, a[0], a[1], b[0], b[1]) <=
          Math.max(padX, padY) * 2.5
        ) {
          return true;
        }
      }
      return false;
    }
    default: {
      const _exhaustive: never = mark;
      return _exhaustive;
    }
  }
}

export function moveOverlay(
  mark: OverlayDto,
  dx: number,
  dy: number,
): OverlayDto {
  switch (mark.type) {
    case "rect":
    case "highlight":
    case "blur":
    case "text":
      return { ...mark, x: mark.x + dx, y: mark.y + dy };
    case "circle":
      return { ...mark, cx: mark.cx + dx, cy: mark.cy + dy };
    case "arrow":
      return {
        ...mark,
        x1: mark.x1 + dx,
        y1: mark.y1 + dy,
        x2: mark.x2 + dx,
        y2: mark.y2 + dy,
      };
    case "pen":
      return {
        ...mark,
        points: mark.points.map(([x, y]) => [x + dx, y + dy] as [number, number]),
      };
    default: {
      const _exhaustive: never = mark;
      return _exhaustive;
    }
  }
}

function boxHandles(mark: {
  x: number;
  y: number;
  w: number;
  h: number;
}): Record<BoxHandle, NormPoint> {
  const box = normalizeBox(mark.x, mark.y, mark.w, mark.h);
  const midX = box.x + box.w / 2;
  const midY = box.y + box.h / 2;
  return {
    nw: { x: box.x, y: box.y },
    n: { x: midX, y: box.y },
    ne: { x: box.x + box.w, y: box.y },
    e: { x: box.x + box.w, y: midY },
    se: { x: box.x + box.w, y: box.y + box.h },
    s: { x: midX, y: box.y + box.h },
    sw: { x: box.x, y: box.y + box.h },
    w: { x: box.x, y: midY },
  };
}

function circleHandlePoints(
  mark: Extract<OverlayDto, { type: "circle" }>,
  size: ShotSize,
): Record<CircleHandle, NormPoint> {
  const { rx, ry } = circleRadii(mark.r, size);
  return {
    n: { x: mark.cx, y: mark.cy - ry },
    e: { x: mark.cx + rx, y: mark.cy },
    s: { x: mark.cx, y: mark.cy + ry },
    w: { x: mark.cx - rx, y: mark.cy },
  };
}

export function hitTestHandle(
  mark: OverlayDto,
  point: NormPoint,
  size: ShotSize,
): ResizeHandle | null {
  const { padX, padY } = hitPad(size, 1.5);
  const near = (a: NormPoint) =>
    Math.abs(a.x - point.x) <= padX && Math.abs(a.y - point.y) <= padY;

  switch (mark.type) {
    case "rect":
    case "highlight":
    case "blur": {
      const hs = boxHandles(mark);
      for (const handle of BOX_HANDLE_ORDER) {
        if (near(hs[handle])) return { kind: "box", handle };
      }
      return null;
    }
    case "circle": {
      const hs = circleHandlePoints(mark, size);
      for (const handle of CIRCLE_HANDLE_ORDER) {
        if (near(hs[handle])) return { kind: "circle", handle };
      }
      return null;
    }
    case "arrow": {
      if (near({ x: mark.x1, y: mark.y1 })) return { kind: "arrow", handle: "start" };
      if (near({ x: mark.x2, y: mark.y2 })) return { kind: "arrow", handle: "end" };
      return null;
    }
    case "text": {
      const { w, h } = textNormExtent(mark, size);
      if (near({ x: mark.x + w, y: mark.y + h })) return { kind: "text", handle: "se" };
      return null;
    }
    case "pen":
      return null;
    default: {
      const _exhaustive: never = mark;
      return _exhaustive;
    }
  }
}

export function resizeOverlay(
  origin: OverlayDto,
  handle: ResizeHandle,
  point: NormPoint,
  size: ShotSize,
): OverlayDto {
  switch (handle.kind) {
    case "box": {
      if (origin.type !== "rect" && origin.type !== "highlight" && origin.type !== "blur") {
        return origin;
      }
      const box = normalizeBox(origin.x, origin.y, origin.w, origin.h);
      let { x, y, w, h } = box;
      const right = x + w;
      const bottom = y + h;
      switch (handle.handle) {
        case "nw":
          x = point.x;
          y = point.y;
          w = right - point.x;
          h = bottom - point.y;
          break;
        case "n":
          y = point.y;
          h = bottom - point.y;
          break;
        case "ne":
          y = point.y;
          w = point.x - x;
          h = bottom - point.y;
          break;
        case "e":
          w = point.x - x;
          break;
        case "se":
          w = point.x - x;
          h = point.y - y;
          break;
        case "s":
          h = point.y - y;
          break;
        case "sw":
          x = point.x;
          w = right - point.x;
          h = point.y - y;
          break;
        case "w":
          x = point.x;
          w = right - point.x;
          break;
        default: {
          const _exhaustive: never = handle;
          return _exhaustive;
        }
      }
      const next = normalizeBox(x, y, w, h);
      if (next.w < 0.005) next.w = 0.005;
      if (next.h < 0.005) next.h = 0.005;
      return { ...origin, ...next };
    }
    case "circle": {
      if (origin.type !== "circle") return origin;
      const w = Math.max(size.width, 1);
      const h = Math.max(size.height, 1);
      const dxPx = (point.x - origin.cx) * w;
      const dyPx = (point.y - origin.cy) * h;
      const rPx = Math.max(4, Math.hypot(dxPx, dyPx));
      return { ...origin, r: rPx / w };
    }
    case "arrow": {
      if (origin.type !== "arrow") return origin;
      if (handle.handle === "start") {
        return { ...origin, x1: point.x, y1: point.y };
      }
      return { ...origin, x2: point.x, y2: point.y };
    }
    case "text": {
      if (origin.type !== "text") return origin;
      const base = Math.max(0.02, point.x - origin.x);
      const { drag_width, min, max } = marks.text_resize;
      const sizePx = Math.min(max, Math.max(min, (marks.text_size * base) / drag_width));
      return { ...origin, size: sizePx };
    }
    default: {
      const _exhaustive: never = handle;
      return _exhaustive;
    }
  }
}

export function selectionHandleList(
  mark: OverlayDto,
  size: ShotSize,
): { key: string; x: number; y: number; cursor: string }[] {
  switch (mark.type) {
    case "rect":
    case "highlight":
    case "blur": {
      const hs = boxHandles(mark);
      return BOX_HANDLE_ORDER.map((key) => ({
        key,
        x: hs[key].x,
        y: hs[key].y,
        cursor: BOX_HANDLE_CURSORS[key],
      }));
    }
    case "circle": {
      const hs = circleHandlePoints(mark, size);
      return CIRCLE_HANDLE_ORDER.map((key) => ({
        key,
        x: hs[key].x,
        y: hs[key].y,
        cursor: key === "n" || key === "s" ? "ns-resize" : "ew-resize",
      }));
    }
    case "arrow":
      return [
        { key: "start", x: mark.x1, y: mark.y1, cursor: "move" },
        { key: "end", x: mark.x2, y: mark.y2, cursor: "move" },
      ];
    case "text": {
      const { w, h } = textNormExtent(mark, size);
      return [{ key: "se", x: mark.x + w, y: mark.y + h, cursor: "nwse-resize" }];
    }
    case "pen":
      return [];
    default: {
      const _exhaustive: never = mark;
      return _exhaustive;
    }
  }
}

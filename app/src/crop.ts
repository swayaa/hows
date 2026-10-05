/** Visible window of a screenshot, normalized to the original image. */

export type CropRect = { x: number; y: number; w: number; h: number };

export type CropHandle = "move" | "nw" | "ne" | "sw" | "se";

const MIN = 0.02;

export function clampCrop(rect: CropRect): CropRect {
  let { x, y, w, h } = rect;
  if (w < 0) {
    x += w;
    w = -w;
  }
  if (h < 0) {
    y += h;
    h = -h;
  }
  w = snap(Math.max(MIN, Math.min(1, w)));
  h = snap(Math.max(MIN, Math.min(1, h)));
  x = snap(Math.min(Math.max(0, x), 1 - w));
  y = snap(Math.min(Math.max(0, y), 1 - h));
  return { x, y, w, h };
}

function snap(value: number): number {
  return Math.round(value * 10_000) / 10_000;
}

export function moveCrop(origin: CropRect, dx: number, dy: number): CropRect {
  return clampCrop({ ...origin, x: origin.x + dx, y: origin.y + dy });
}

export function resizeCrop(
  origin: CropRect,
  handle: CropHandle,
  x: number,
  y: number,
): CropRect {
  if (handle === "move") return origin;
  let left = origin.x;
  let top = origin.y;
  let right = origin.x + origin.w;
  let bottom = origin.y + origin.h;
  const west = handle === "nw" || handle === "sw";
  const east = handle === "ne" || handle === "se";
  const north = handle === "nw" || handle === "ne";
  const south = handle === "sw" || handle === "se";
  if (west) left = x;
  if (east) right = x;
  if (north) top = y;
  if (south) bottom = y;
  return clampCrop({ x: left, y: top, w: right - left, h: bottom - top });
}

export function hitCropHandle(
  crop: CropRect,
  point: { x: number; y: number },
  size: { width: number; height: number },
): CropHandle | null {
  const right = crop.x + crop.w;
  const bottom = crop.y + crop.h;
  const handles: { key: CropHandle; x: number; y: number }[] = [
    { key: "nw", x: crop.x, y: crop.y },
    { key: "ne", x: right, y: crop.y },
    { key: "sw", x: crop.x, y: bottom },
    { key: "se", x: right, y: bottom },
  ];
  const threshold = 22;
  for (const handle of handles) {
    const dx = (point.x - handle.x) * size.width;
    const dy = (point.y - handle.y) * size.height;
    if (dx * dx + dy * dy <= threshold * threshold) return handle.key;
  }
  if (point.x >= crop.x && point.x <= right && point.y >= crop.y && point.y <= bottom) {
    return "move";
  }
  return null;
}

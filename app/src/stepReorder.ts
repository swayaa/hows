export type DragSession =
  | { kind: "idle" }
  | { kind: "dragging"; stepId: string; gap: number };

export type ColRect = { left: number; right: number };

/** Map insert-before gap to `move_step` index; null when drop is a no-op. */
export function moveTargetForGap(from: number, gap: number): number | null {
  const to = gap > from ? gap - 1 : gap;
  return to === from ? null : to;
}

/** One-slot keyboard reorder from a focused grip (Alt+↑/↓). */
export function slotAfterKeyboard(
  from: number,
  dir: "up" | "down",
  length: number,
): number | null {
  if (from < 0 || from >= length) return null;
  switch (dir) {
    case "up":
      return from > 0 ? from - 1 : null;
    case "down":
      return from + 1 < length ? from + 1 : null;
    default: {
      const unreachable: never = dir;
      return unreachable;
    }
  }
}

/** Hit-test `clientX` for horizontal Filmstreifen; gap insert-before (0..=cols.length). */
export function gapFromClientX(
  clientX: number,
  cols: ReadonlyArray<ColRect>,
): number {
  const gap = cols.findIndex((col) => clientX < (col.left + col.right) / 2);
  return gap === -1 ? cols.length : gap;
}

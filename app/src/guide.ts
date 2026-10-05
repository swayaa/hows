/** Guide DTOs from the Tauri backend plus the per-step screenshot cache. */

import type { OverlayDto } from "./annotate";
import type { CropRect } from "./crop";

export type StepDto = {
  id: string;
  action: string;
  text: string;
  has_override: boolean;
  has_image: boolean;
  overlays: OverlayDto[];
  /** `null` shows the whole screenshot. */
  crop: CropRect | null;
};

export type GuideDto = { title: string; steps: StepDto[] };

export type OpenGuideResult = { path: string; guide: GuideDto };

/** `data:` URI per step id; missing key = not loaded yet. */
export type ImageById = Record<string, string>;

/** Keep only entries whose step ids are still live. */
export function pruneImageById(
  imageById: ImageById,
  liveIds: Iterable<string>,
): ImageById {
  const live = new Set(liveIds);
  return Object.fromEntries(
    Object.entries(imageById).filter(([id]) => live.has(id)),
  );
}

/** Full screenshots kept at once: the open step, the visible filmstrip, and a little room. */
export const STEP_IMAGE_CACHE_LIMIT = 24;

/**
 * Ids that stay in memory. Wanted ids come first (selected, then visible).
 * Remaining slots keep the newest cached ids. Never more than `limit`.
 */
export function retainStepImageIds(
  wanted: readonly string[],
  cachedOldestFirst: readonly string[],
  limit = STEP_IMAGE_CACHE_LIMIT,
): string[] {
  const keep: string[] = [];
  const seen = new Set<string>();
  const add = (id: string) => {
    if (keep.length >= limit || id.length === 0 || seen.has(id)) return;
    seen.add(id);
    keep.push(id);
  };
  for (const id of wanted) add(id);
  for (let index = cachedOldestFirst.length - 1; index >= 0; index -= 1) {
    add(cachedOldestFirst[index]);
  }
  return keep;
}

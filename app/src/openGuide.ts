/** Library cards for recently opened guides. */

import { stepCountLabel } from "./chrome.ts";
import type { Messages } from "./locales/index.ts";

export type RecentGuideEntry = {
  path: string;
  missing: boolean;
  /** The file exists but holds no readable guide. */
  unreadable: boolean;
  title: string | null;
  step_count: number | null;
  /** Recording time, milliseconds since 1970 (UTC). */
  created_at_ms: number | null;
};

export function recentFileLabel(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

/** Heading of a library card: the guide's title, else its file name. */
export function recentTitle(entry: RecentGuideEntry): string {
  return entry.title ?? recentFileLabel(entry.path);
}

/** Second line of a library card, for example "5 steps · Sep 30, 2026". */
export function recentMeta(entry: RecentGuideEntry, copy: Messages, localeCode: string): string {
  if (entry.missing) return copy.missing;
  if (entry.unreadable) return copy.recentUnreadable;
  const parts: string[] = [];
  if (entry.step_count !== null) parts.push(stepCountLabel(copy, entry.step_count));
  if (entry.created_at_ms !== null) {
    parts.push(new Intl.DateTimeFormat(localeCode, { dateStyle: "medium" }).format(entry.created_at_ms));
  }
  return parts.join(" · ");
}

/** Backend errors: a stable code for the interface, a technical detail for the log. */

import type { Messages } from "./locales/index.ts";

/** Shape of `CommandError` in `app/src-tauri/src/error.rs`. */
export type CommandError = {
  code: string;
  detail: string;
  /** A path the interface may show, for example the default folder. */
  path?: string;
};

/** Reads a rejected invoke or an event payload; anything else counts as `internal`. */
export function asCommandError(error: unknown): CommandError {
  if (error && typeof error === "object" && typeof (error as CommandError).code === "string") {
    const { code, detail, path } = error as CommandError;
    return { code, detail: typeof detail === "string" ? detail : "", ...(typeof path === "string" ? { path } : {}) };
  }
  return { code: "internal", detail: String(error) };
}

/** Text for an error in the interface language; the backend's own words never reach the screen. */
export function errorMessage(error: unknown, messages: Messages): string {
  const { code } = asCommandError(error);
  const known: Record<string, string> = messages.errors;
  return Object.hasOwn(known, code) ? known[code] : messages.errors.internal;
}

/** Writes the technical detail to the developer console, then returns the translated text. */
export function reportError(error: unknown, messages: Messages): string {
  const { code, detail } = asCommandError(error);
  console.error(`Hows ${code}: ${detail}`);
  return errorMessage(error, messages);
}

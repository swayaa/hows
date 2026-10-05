/**
 * UI locale registry. A locale is added by writing its file and listing it
 * here. Tests check that UI and `core/i18n/locales` list the same codes and
 * names.
 */

import { de } from "./de.ts";
import { en, type Messages } from "./en.ts";
import { es } from "./es.ts";
import { fr } from "./fr.ts";
import { it } from "./it.ts";
import { nl } from "./nl.ts";
import { pt } from "./pt.ts";

export type { Messages };

export type UiLocale = {
  /** ISO 639-1 code, the value stored in `settings.language`. */
  code: string;
  /** Native name for the language picker, equal to `locale.name` in core. */
  name: string;
  messages: Messages;
};

/** English comes first and is the fallback. */
export const UI_LOCALES: readonly UiLocale[] = [
  { code: "en", name: "English", messages: en },
  { code: "de", name: "Deutsch", messages: de },
  { code: "fr", name: "Français", messages: fr },
  { code: "es", name: "Español", messages: es },
  { code: "it", name: "Italiano", messages: it },
  { code: "pt", name: "Português (Brasil)", messages: pt },
  { code: "nl", name: "Nederlands", messages: nl },
];

/** Locale for a stored or OS code such as `de-AT`. Unknown codes give English. */
export function uiLocale(code: string): UiLocale {
  const primary = code.trim().toLowerCase().split(/[-_.]/)[0];
  return UI_LOCALES.find((locale) => locale.code === primary) ?? UI_LOCALES[0];
}

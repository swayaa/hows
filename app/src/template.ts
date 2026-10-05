/** `{name}` templates, filled like `steps_i18n::fill` in the backend. */

/** Fills `{name}` placeholders in one pass; unknown placeholders stay as they are. */
export function fillTemplate(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    Object.hasOwn(values, name) ? String(values[name]) : match,
  );
}

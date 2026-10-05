// Fact markers in a doc page. A marked value looks like this and renders as
// plain text:
//
//   <!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact --> s
//
// `fact:` names the fact, then optional display steps separated by spaces
// (see display.mjs; no `|`, which would split a table cell). The text up to
// `<!-- /fact -->` must show that value. A code fact the interface never
// shows is listed once as `<!-- fact-exempt: key key -->`. Markers inside
// code are examples and ignored.

import { lineOf, withoutCode } from "../lib.mjs";

const token = /<!--\s*(?:fact:([^>]*?)|(\/fact)|fact-exempt:([^>]*?))\s*-->/g;

export function markersIn(text) {
  const markers = [];
  const exempt = [];
  const problems = [];
  let open = null;

  for (const found of withoutCode(text).matchAll(token)) {
    const line = lineOf(text, found.index);
    const [whole, rawSpec, close, exemptList] = found;
    const before = text.slice(text.lastIndexOf("\n", found.index - 1) + 1, found.index);
    if (exemptList === undefined && /^\s*(?:[-*+>]|\d+\.)?\s*$/.test(before)) {
      problems.push({ line, message: "a line that starts with a fact marker renders as raw HTML; start it with text" });
    }
    if (rawSpec !== undefined) {
      const spec = rawSpec.trim().replace(/\s+/g, " ");
      if (open) problems.push({ line, message: `fact marker "${spec}" opens before "${open.spec}" is closed` });
      const [key, ...steps] = spec.split(" ");
      open = { spec, key, steps, line, start: found.index + whole.length };
    } else if (close) {
      if (!open) {
        problems.push({ line, message: "<!-- /fact --> without an opening fact marker" });
        continue;
      }
      markers.push({ ...open, text: text.slice(open.start, found.index) });
      open = null;
    } else {
      for (const key of exemptList.split(/\s+/).filter(Boolean)) exempt.push({ key, line });
    }
  }
  if (open) problems.push({ line: open.line, message: `fact marker "${open.spec}" is never closed` });
  return { markers, exempt, problems };
}

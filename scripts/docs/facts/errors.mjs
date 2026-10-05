// Error codes of the Tauri commands (app/src-tauri/src/error.rs). The docs
// must quote the text of every code, or list it as one the interface never
// shows (`<!-- fact-exempt: ui.errors.<code> -->`).

import { match, rustEnum, snakeCase } from "./source.mjs";

const file = "app/src-tauri/src/error.rs";

export default function errorFacts() {
  if (match(file, /#\[serde\(rename_all = "(\w+)"\)\]\s*pub enum ErrorCode/) !== "snake_case") {
    throw new Error(`${file}: ErrorCode is no longer serialized in snake_case`);
  }
  const codes = rustEnum(file, "ErrorCode").variants.map(snakeCase);
  return {
    facts: [{ key: "errors.codes", value: codes, source: file }],
    required: codes.map((code) => `ui.errors.${code}`),
  };
}

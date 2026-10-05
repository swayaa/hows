// The `.steps` format (core/store/src/model.rs): `store.schema_version`; the
// read limits (core/store/src/limits.rs): `store.limits.<constant>` for every
// `pub const`, each required in the docs; the annotation defaults
// (core/store/marks.json): `marks.<field>` for each number or text, and
// `marks.pen_stroke`, the pen width derived from them.

import { read, readJson, rustConstant } from "./source.mjs";

const file = "core/store/src/model.rs";
const limitsFile = "core/store/src/limits.rs";
const marksFile = "core/store/marks.json";

export function marks() {
  return readJson(marksFile);
}

function limitFacts() {
  return [...read(limitsFile).matchAll(/^pub const (\w+):/gm)].map(([, name]) => ({
    key: `store.limits.${name.toLowerCase()}`,
    value: rustConstant(name).value,
    source: limitsFile,
  }));
}

export default function storeFacts() {
  const defaults = marks();
  const scalars = Object.entries(defaults).filter(([, value]) => ["string", "number"].includes(typeof value));
  const limits = limitFacts();
  return {
    facts: [
      ...limits,
      { key: "store.schema_version", value: rustConstant("SCHEMA_VERSION").value, source: file },
      ...scalars.map(([field, value]) => ({ key: `marks.${field}`, value, source: marksFile })),
      {
        key: "marks.pen_stroke",
        value: Number((defaults.stroke * defaults.pen_factor).toFixed(2)),
        source: marksFile,
      },
    ],
    required: limits.map(({ key }) => key),
  };
}

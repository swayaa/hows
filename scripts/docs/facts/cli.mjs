// Defaults of `steps-cli` (core/cli/src/main.rs): the export format without
// `--format` and the mode without `--mode`. The mode code comes from the
// export crate's JSON.

import { match } from "./source.mjs";

const file = "core/cli/src/main.rs";
const exportLib = "core/export/src/lib.rs";

export default function cliFacts() {
  const mode = match(file, /let mut mode = ExportMode::(\w+);/);
  return [
    { key: "cli.default_format", value: match(file, /let mut format = "([^"]+)"/), source: file },
    {
      key: "cli.default_mode",
      value: match(exportLib, new RegExp(`ExportMode::${mode} => "([^"]+)"`)),
      source: `${file}, ${exportLib}`,
    },
  ];
}

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { pruneImageById, retainStepImageIds } from "./guide.ts";

describe("step image cache", () => {
  it("prunes cache entries for deleted steps", () => {
    const cached = { keep: "data:a", gone: "data:b" };
    assert.deepEqual(pruneImageById(cached, ["keep"]), { keep: "data:a" });
    assert.deepEqual(pruneImageById(cached, []), {});
  });

  it("keeps the selected and visible screenshots and drops the rest past the limit", () => {
    const wanted = ["selected", "visible-a", "visible-b"];
    const cached = ["old", "older", "visible-a", "selected"];
    assert.deepEqual(retainStepImageIds(wanted, cached, 3), [
      "selected",
      "visible-a",
      "visible-b",
    ]);
    assert.deepEqual(retainStepImageIds(["selected"], ["a", "b", "c"], 3), [
      "selected",
      "c",
      "b",
    ]);
  });
});

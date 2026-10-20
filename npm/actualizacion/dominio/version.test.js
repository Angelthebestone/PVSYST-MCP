import assert from "node:assert/strict";
import { test } from "node:test";
import { compararVersiones, parsearVersion, versionMasReciente } from "./version.js";

test("should parse version components when text is x.y.z", () => {
  assert.deepEqual({ ...parsearVersion("1.12.3") }, { mayor: 1, menor: 12, parche: 3 });
});

test("should reject version when text is a prerelease or malformed", () => {
  ["1.2.3-beta.1", "1.2", "v1.2.3", "", null, 3].forEach((texto) =>
    assert.equal(parsearVersion(texto), null),
  );
});

test("should compare numerically when components have different digit counts", () => {
  assert.equal(compararVersiones("0.10.0", "0.9.9"), 1);
  assert.equal(compararVersiones("0.1.1", "0.1.1"), 0);
  assert.equal(compararVersiones("0.1.1", "0.2.0"), -1);
});

test("should throw when comparing an invalid version", () => {
  assert.throws(() => compararVersiones("x", "0.1.0"), TypeError);
});

test("should pick newest valid version and ignore invalid ones", () => {
  assert.equal(versionMasReciente(["0.1.0", "basura", "0.2.0", "0.1.9"]), "0.2.0");
});

test("should have no newest version when none is valid", () => {
  assert.equal(versionMasReciente(["", "x.y.z"]), null);
});

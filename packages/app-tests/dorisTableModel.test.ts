import { strict as assert } from "node:assert";
import { test } from "vitest";
import { DORIS_AGGREGATION_TYPES, dorisTableSqlOption, normalizeDorisKeyModel, parseDorisReplicationNum, supportsDorisTableModel } from "../../apps/desktop/src/lib/table/dorisTableModel.ts";

test("Doris table model is only offered while creating a Doris table", () => {
  assert.equal(supportsDorisTableModel("doris", true), true);
  assert.equal(supportsDorisTableModel("doris", false), false);
  assert.equal(supportsDorisTableModel("mysql", true), false);
  assert.equal(supportsDorisTableModel(undefined, true), false);
});

test("normalizes the Doris key model and replica count", () => {
  assert.equal(normalizeDorisKeyModel("unique"), "UNIQUE");
  assert.equal(normalizeDorisKeyModel(" aggregate "), "AGGREGATE");
  assert.equal(normalizeDorisKeyModel("primary"), "DUPLICATE");
  assert.equal(normalizeDorisKeyModel(undefined), "DUPLICATE");
  assert.equal(parseDorisReplicationNum("3"), 3);
  assert.equal(parseDorisReplicationNum(" 1 "), 1);
  assert.equal(parseDorisReplicationNum("0"), undefined);
  assert.equal(parseDorisReplicationNum("abc"), undefined);
  assert.equal(parseDorisReplicationNum(""), undefined);
  assert.ok(DORIS_AGGREGATION_TYPES.includes("SUM"));
  assert.ok(DORIS_AGGREGATION_TYPES.includes("BITMAP_UNION"));
});

test("builds the Doris table option only when supported", () => {
  assert.deepEqual(dorisTableSqlOption("unique", "1", true), { keyModel: "UNIQUE", replicationNum: 1 });
  assert.deepEqual(dorisTableSqlOption("", "", true), { keyModel: "DUPLICATE", replicationNum: undefined });
  assert.equal(dorisTableSqlOption("unique", "1", false), undefined);
});

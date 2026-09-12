import type { DatabaseType } from "@/types/database";
import type { DorisTableOptions } from "@/lib/table/tableStructureEditorSql";

export const DORIS_KEY_MODELS = ["DUPLICATE", "UNIQUE", "AGGREGATE"] as const;
export type DorisKeyModel = (typeof DORIS_KEY_MODELS)[number];
export const DORIS_DEFAULT_KEY_MODEL: DorisKeyModel = "DUPLICATE";
/** Aggregation types a value column may carry in an AGGREGATE KEY table. */
export const DORIS_AGGREGATION_TYPES = ["SUM", "MAX", "MIN", "REPLACE", "REPLACE_IF_NOT_NULL", "HLL_UNION", "BITMAP_UNION", "QUANTILE_UNION"] as const;

/** The table model can only be chosen while creating a Doris table; it is immutable afterwards. */
export function supportsDorisTableModel(databaseType: DatabaseType | undefined, isCreateMode: boolean): boolean {
  return databaseType === "doris" && isCreateMode;
}

export function normalizeDorisKeyModel(value: string | undefined): DorisKeyModel {
  const upper = (value ?? "").trim().toUpperCase();
  return (DORIS_KEY_MODELS as readonly string[]).includes(upper) ? (upper as DorisKeyModel) : DORIS_DEFAULT_KEY_MODEL;
}

/** Parses the replicas input: a positive integer, or `undefined` for "cluster default". */
export function parseDorisReplicationNum(value: string): number | undefined {
  const trimmed = value.trim();
  if (!/^\d+$/.test(trimmed)) return undefined;
  const parsed = Number.parseInt(trimmed, 10);
  return parsed > 0 ? parsed : undefined;
}

export function dorisTableSqlOption(keyModel: string, replicationNum: string, supported: boolean): DorisTableOptions | undefined {
  if (!supported) return undefined;
  return { keyModel: normalizeDorisKeyModel(keyModel), replicationNum: parseDorisReplicationNum(replicationNum) };
}

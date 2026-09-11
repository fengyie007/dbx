1、增加对doris4.x数据库的支持。 ✅ 已完成（2026-09-12）

## 任务1 修改内容

Doris 本身已受支持（`doris` 连接类型走 MySQL 协议），本次补齐 4.x 的方言、类型、索引与编辑器能力：

### 方言 / 类型（`plugins/dialects/doris.yaml`）
- 版本列表改为 `4.0`（RECOMMENDED）、`3.0`、`2.1`（COMPATIBLE）。
- 新增类型 `VARIANT`（半结构化）、`IPV4`、`IPV6`；`DECIMAL` 补充 `max_precision: 38`。
- `create_index` 模板默认由 `USING BITMAP` 改为 `USING INVERTED`（Doris 4.x 已弃用 bitmap 索引）。

### 表结构编辑器
- `apps/desktop/src/lib/table/tableStructureEditorState.ts`：Doris 不再复用 MySQL 类型列表，改为独立的 OLAP 类型集合（largeint/string/variant/ipv4/ipv6/array/map/struct/bitmap/hll/quantile_state 等，去掉 enum/set/blob/year 等 MySQL 专有类型）。
- `apps/desktop/src/components/structure/TableStructureEditor.vue`：Doris 索引类型下拉改为 `INVERTED / NGRAM_BF / ANN / BITMAP`，默认 `INVERTED`（ANN 为 4.0 向量索引）。
- `crates/dbx-core/src/table_structure_sql/dialect.rs`：Doris 开启 create/drop/rebuild index、index_type、index_comment 能力。
- `crates/dbx-core/src/table_structure_sql/indexes.rs`：新增 Doris 索引 DDL 语法 `CREATE INDEX x ON t (c) USING INVERTED|NGRAM_BF|ANN|BITMAP COMMENT '...'`、`DROP INDEX x ON t`；不支持的类型（如 BTREE）和 UNIQUE 索引降级并给出 warning。
- `crates/dbx-core/src/db/mysql_compatible.rs`：新增测试覆盖 `USING ANN PROPERTIES(...)` 形式的 SHOW CREATE TABLE 索引解析。

### SQL 编辑器 / 补全
- `apps/desktop/src/lib/sql/sqlCompletion.ts`：新增 Doris 关键字（DISTRIBUTED BY、BUCKETS、PROPERTIES、SWITCH、CATALOG、USING INVERTED/ANN、MATCH_* 等）与函数签名（bitmap/hll 聚合、array/variant、IP 函数，以及 4.0 新增的 `SEARCH`、`L2_DISTANCE_APPROXIMATE` 等向量距离函数、`AI_*` 函数）。
- `apps/desktop/src/lib/editor/codemirrorSqlDialect.ts`：Doris 在 MySQL 语法高亮基础上叠加上述关键字、类型与内置函数，不影响 MySQL 本身。

### 文档 / 测试
- `docs/content/docs/databases{,.cn,.tr}.mdx`：标注 Doris 支持 2.1 至 4.x。
- `crates/dbx-core/tests/live_doris.rs`：ignore 说明改为 2.1 至 4.x 集群。
- 新增/调整测试：`packages/app-tests/tableStructureEditorState.test.ts`、`packages/app-tests/sqlCompletion.test.ts`、`apps/desktop/src/lib/__tests__/editor/codemirrorSqlDialect.spec.ts`、`crates/dbx-core/src/table_structure_sql/tests.rs`。

### 待维护者验证
- Rust 部分未在本机执行 cargo（按项目约定），需 `make cargo-check-fast` / `make cargo-test-fast` 验证。
- 方言 descriptor 变更可能需要更新 `crates/dbx-core/src/sql_dialect/snapshots/` 下的 insta 快照。

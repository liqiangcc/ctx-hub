# Context Hub SQLite / rusqlite 搜索预研计划

## 1. 背景

Context Hub 的核心价值是让用户和 AI 快速找回可靠上下文。

如果目标是“极致搜索体验”，MVP 阶段不应再以 JSONL 作为主存储和主搜索方案。

技术路线应调整为：

```text
Rust + rusqlite + SQLite FTS5 + 本地数据库 + JSONL 导入导出
```

JSONL 保留为：

1. 导入格式。
2. 导出格式。
3. 备份格式。
4. 数据迁移格式。

SQLite 作为：

1. 主存储。
2. 主索引。
3. 主搜索引擎。

---

## 2. 预研目标

本轮预研重点验证 `rusqlite + SQLite FTS5` 是否可以支撑 Context Hub 的高质量搜索体验。

需要验证：

1. 本地 SQLite 存储是否稳定。
2. FTS5 全文检索是否可用。
3. 中文搜索是否可用。
4. 中文短词搜索是否可用。
5. 英文、服务名、错误码、命令片段搜索是否可用。
6. Key 精确搜索是否足够快。
7. 标签筛选和全文搜索是否可以组合。
8. 搜索结果是否可以按相关性排序。
9. 搜索结果是否可以生成 snippet / highlight。
10. 是否可以支持前缀搜索和子串搜索。
11. 是否可以在 1 万条记录内保持良好体验。
12. 是否方便后续迁移、备份和跨平台分发。

---

## 3. 技术结论

MVP 搜索方案建议调整为：

```text
SQLite records 表 + FTS5 全文索引 + trigram 子串索引 + 应用层排序增强
```

推荐优先级：

1. SQLite 主表负责可靠存储。
2. FTS5 unicode61 索引负责常规全文搜索。
3. FTS5 prefix 索引负责前缀搜索。
4. FTS5 trigram 索引负责子串搜索。
5. 应用层 CJK n-gram 字段增强中文短词搜索。
6. 应用层 ranking 做最终排序加权。
7. JSONL 只负责导入导出。

---

## 4. 推荐 Cargo 依赖

```toml
[dependencies]
rusqlite = { version = "0.40", features = ["bundled", "modern-full", "functions"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["formatting", "parsing"] }
anyhow = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4"] }
```

说明：

- `bundled`：降低用户机器缺少 SQLite 或 SQLite 编译选项不一致的风险。
- `modern-full`：优先启用较完整的现代 SQLite 能力。
- `functions`：为后续自定义 SQL 函数或排序增强预留空间。

预研时必须验证最终二进制体积和跨平台构建是否可接受。

---

## 5. 数据库文件位置

推荐默认路径：

```text
~/.ctx-hub/ctx-hub.db
```

同时支持环境变量覆盖：

```bash
CTX_HUB_DB=/path/to/ctx-hub.db ctx search payment
```

原因：

1. 方便测试。
2. 方便多数据集。
3. 方便 CI。
4. 方便用户临时指定上下文库。

---

## 6. 推荐数据库 Schema

### 6.1 主表 records

```sql
CREATE TABLE IF NOT EXISTS records (
  rowid INTEGER PRIMARY KEY AUTOINCREMENT,
  id TEXT NOT NULL UNIQUE,
  key TEXT UNIQUE,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  tags_json TEXT NOT NULL DEFAULT '[]',
  tags_text TEXT NOT NULL DEFAULT '',
  record_type TEXT,
  service TEXT,
  env TEXT,
  source TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  expires_at TEXT,
  usage_count INTEGER NOT NULL DEFAULT 0,
  search_ngrams TEXT NOT NULL DEFAULT ''
);
```

字段说明：

- `rowid`：SQLite 内部主键，同时作为 FTS5 rowid。
- `id`：业务 ID，例如 `ctx_xxx`。
- `key`：用户可读的唯一 Key，例如 `runbook.payment.failed`。
- `tags_json`：保留结构化标签。
- `tags_text`：用于搜索的标签展开文本。
- `search_ngrams`：应用层生成的中文短词 / n-gram 搜索增强字段。

---

### 6.2 常规全文索引 records_fts

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
  key,
  title,
  content,
  tags_text,
  record_type,
  service,
  env,
  source,
  search_ngrams,
  content='records',
  content_rowid='rowid',
  tokenize = "unicode61 remove_diacritics 0 tokenchars '-_./:@'",
  prefix = '2 3 4'
);
```

目标：

1. 支持常规全文搜索。
2. 支持服务名、Key、路径、命令、URL 片段。
3. 支持前缀搜索。
4. 支持中文整体词搜索。
5. 支持应用层生成的中文短词 n-gram。

`tokenchars '-_./:@'` 的原因：

- 服务名常包含 `-`、`_`。
- Key 常包含 `.`。
- URL / 路径 / 命令常包含 `/`、`:`、`@`。

---

### 6.3 子串索引 records_trigram

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS records_trigram USING fts5(
  title,
  content,
  key,
  tags_text,
  service,
  env,
  source,
  content='records',
  content_rowid='rowid',
  tokenize = "trigram"
);
```

目标：

1. 支持更宽松的子串搜索。
2. 支持命令片段搜索。
3. 支持 URL 片段搜索。
4. 支持中文长词片段搜索。

限制：

```text
trigram 对少于 3 个 unicode 字符的 MATCH 查询不友好。
```

因此中文 1 到 2 字短词搜索不能只依赖 trigram，需要应用层 n-gram 或 LIKE 兜底。

---

## 7. FTS 同步策略

使用外部内容 FTS5 表时，必须保证主表和 FTS 索引一致。

推荐使用触发器同步：

```sql
CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
  INSERT INTO records_fts(rowid, key, title, content, tags_text, record_type, service, env, source, search_ngrams)
  VALUES (new.rowid, new.key, new.title, new.content, new.tags_text, new.record_type, new.service, new.env, new.source, new.search_ngrams);

  INSERT INTO records_trigram(rowid, title, content, key, tags_text, service, env, source)
  VALUES (new.rowid, new.title, new.content, new.key, new.tags_text, new.service, new.env, new.source);
END;
```

```sql
CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
  INSERT INTO records_fts(records_fts, rowid, key, title, content, tags_text, record_type, service, env, source, search_ngrams)
  VALUES ('delete', old.rowid, old.key, old.title, old.content, old.tags_text, old.record_type, old.service, old.env, old.source, old.search_ngrams);

  INSERT INTO records_trigram(records_trigram, rowid, title, content, key, tags_text, service, env, source)
  VALUES ('delete', old.rowid, old.title, old.content, old.key, old.tags_text, old.service, old.env, old.source);
END;
```

```sql
CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
  INSERT INTO records_fts(records_fts, rowid, key, title, content, tags_text, record_type, service, env, source, search_ngrams)
  VALUES ('delete', old.rowid, old.key, old.title, old.content, old.tags_text, old.record_type, old.service, old.env, old.source, old.search_ngrams);

  INSERT INTO records_fts(rowid, key, title, content, tags_text, record_type, service, env, source, search_ngrams)
  VALUES (new.rowid, new.key, new.title, new.content, new.tags_text, new.record_type, new.service, new.env, new.source, new.search_ngrams);

  INSERT INTO records_trigram(records_trigram, rowid, title, content, key, tags_text, service, env, source)
  VALUES ('delete', old.rowid, old.title, old.content, old.key, old.tags_text, old.service, old.env, old.source);

  INSERT INTO records_trigram(rowid, title, content, key, tags_text, service, env, source)
  VALUES (new.rowid, new.title, new.content, new.key, new.tags_text, new.service, new.env, new.source);
END;
```

预研必须验证：

1. insert 后能搜索到。
2. update 后旧内容搜索不到，新内容能搜索到。
3. delete 后搜索不到。
4. rebuild 后索引一致。

---

## 8. 搜索能力分层

### 8.1 Key 精确搜索

Key 查询必须走普通索引，不走 FTS。

```sql
CREATE INDEX IF NOT EXISTS idx_records_key ON records(key);
```

查询：

```sql
SELECT * FROM records WHERE key = ?1 LIMIT 1;
```

目标：

```text
Key 精确查询应接近 O(log n)，并且优先级最高。
```

---

### 8.2 标签查询

标签查询 MVP 可以先用 `tags_text`。

后续如果标签查询复杂，再拆出关系表：

```sql
CREATE TABLE IF NOT EXISTS record_tags (
  record_rowid INTEGER NOT NULL,
  tag TEXT NOT NULL,
  PRIMARY KEY (record_rowid, tag)
);

CREATE INDEX IF NOT EXISTS idx_record_tags_tag ON record_tags(tag);
```

建议：

```text
如果追求极致标签筛选，预研阶段直接验证 record_tags 关系表。
```

---

### 8.3 常规全文搜索

```sql
SELECT
  r.*,
  snippet(records_fts, -1, '[', ']', '...', 32) AS snippet,
  bm25(records_fts, 8.0, 10.0, 3.0, 5.0, 2.0, 4.0, 2.0, 2.0, 1.0) AS score
FROM records_fts
JOIN records r ON r.rowid = records_fts.rowid
WHERE records_fts MATCH ?1
ORDER BY score ASC, r.usage_count DESC, r.updated_at DESC
LIMIT ?2;
```

建议字段权重：

| 字段 | 权重 |
| --- | --- |
| key | 8.0 |
| title | 10.0 |
| content | 3.0 |
| tags_text | 5.0 |
| record_type | 2.0 |
| service | 4.0 |
| env | 2.0 |
| source | 2.0 |
| search_ngrams | 1.0 |

说明：

- 标题命中应明显优先。
- Key 命中应明显优先。
- 标签和服务名命中应强于正文。
- n-gram 命中用于召回，不应过度影响排名。

---

### 8.4 前缀搜索

FTS 表使用：

```sql
prefix = '2 3 4'
```

查询时可以把部分关键词改写成：

```text
pay* order* runbook*
```

适合：

1. 服务名前缀。
2. Key 前缀。
3. 命令前缀。
4. 英文单词前缀。

预研需要验证：

```bash
ctx search pay
ctx search order-serv
ctx search runbook.pay
```

---

### 8.5 子串搜索

trigram 查询：

```sql
SELECT r.*
FROM records_trigram
JOIN records r ON r.rowid = records_trigram.rowid
WHERE records_trigram MATCH ?1
LIMIT ?2;
```

适合：

1. URL 中间片段。
2. 命令中间片段。
3. 较长中文片段。
4. 错误码附近文本。

限制：

```text
少于 3 个 unicode 字符的子串，需要额外兜底策略。
```

---

### 8.6 中文短词搜索增强

中文搜索的关键问题是：

```text
很多有价值查询只有 2 个字，例如：支付、订单、日志、构建、超时。
```

trigram 对 2 字短词不够友好，所以建议在写入记录时生成 `search_ngrams`。

示例：

```text
支付失败排查规则
```

生成：

```text
支付 付失 失败 败排 排查 查规 规则 支付失败 失败排查 排查规则
```

策略：

1. 对 CJK 文本生成 2-gram 和 3-gram。
2. 存入 `search_ngrams` 字段。
3. FTS 查询时同时查原文和 n-gram 字段。
4. n-gram 字段权重较低，只用于召回。

预研必须验证：

```bash
ctx search 支付
ctx search 失败
ctx search 支付失败
ctx search 日志
ctx search 构建
```

---

## 9. 查询改写策略

用户输入不能直接拼接到 MATCH SQL，必须先做安全转义和查询改写。

### 9.1 输入分类

需要识别：

1. Key-like 查询：`runbook.payment.failed`
2. 服务名查询：`order-service`
3. 错误码查询：`401`、`NullPointerException`
4. 中文查询：`支付失败`
5. 命令片段：`mvn clean package`
6. URL / 路径片段：`/api/order/create`
7. 标签查询：`#payment` 或 `tag:payment`

---

### 9.2 查询模式

建议支持：

```bash
ctx search payment
ctx search "支付失败"
ctx search order-service
ctx search tag:runbook payment
ctx search service:payment-service 401
ctx search key:runbook.payment.failed
```

MVP 可以先支持基础模式，预研时验证扩展空间。

---

## 10. 排序策略

最终排序不应完全依赖 FTS5 `bm25`。

建议使用组合评分：

```text
final_score = fts_score
            + exact_key_boost
            + title_boost
            + tag_boost
            + service_boost
            + recency_boost
            + usage_boost
            + status_penalty
            + expired_penalty
```

其中：

- Key 完全匹配：最高优先级。
- 标题完整命中：强加权。
- 标签命中：强加权。
- 服务名命中：中高加权。
- 正文命中：正常相关性。
- 使用次数高：适当加权。
- 过期内容：降权。
- 废弃内容：默认不展示或明显降权。

---

## 11. 搜索结果展示

搜索结果建议展示：

```text
[1] runbook.payment.failed
标题：支付失败排查规则
标签：payment, test, runbook
服务：payment-service
摘要：... [支付失败] 时先查询 payment_callback_log ...
更新时间：2026-06-21T10:00:00+09:00
```

必须支持：

1. 匹配高亮。
2. 片段摘要。
3. Key 展示。
4. 标签展示。
5. 服务和环境展示。
6. 过期 / 废弃状态提示。

---

## 12. 性能验证目标

预研基准数据规模：

| 数据量 | 目标 |
| --- | --- |
| 100 条 | 感知不到延迟 |
| 1,000 条 | 搜索应接近瞬时 |
| 10,000 条 | 常规查询应可接受 |
| 100,000 条 | 作为压力测试，不作为 MVP 必须目标 |

建议目标：

```text
1 万条记录内，常见搜索 P95 < 100ms。
```

测试维度：

1. Key 精确查询。
2. 单关键词查询。
3. 多关键词查询。
4. 中文短词查询。
5. trigram 子串查询。
6. 标签 + 关键词组合查询。
7. 排序 + snippet 查询。

---

## 13. 数据库维护能力

需要预研以下命令：

```bash
ctx db init
ctx db info
ctx db rebuild-index
ctx db optimize
ctx db export --format jsonl
ctx db import records.jsonl
```

其中：

- `rebuild-index`：重建 FTS 索引。
- `optimize`：执行 FTS optimize。
- `export`：导出 JSONL 备份。
- `import`：从 JSONL 导入。

---

## 14. 预研阶段任务

### 阶段 1：rusqlite 基础验证

任务：

1. 引入 `rusqlite`。
2. 使用 `bundled` 构建。
3. 创建本地数据库。
4. 创建 records 表。
5. 插入和查询记录。
6. 验证 Windows / macOS / Linux 构建。

验收：

```bash
ctx db init
ctx add --title "支付失败" --content "先查 payment_callback_log"
ctx show runbook.payment.failed
```

---

### 阶段 2：FTS5 验证

任务：

1. 创建 records_fts。
2. 创建触发器。
3. 插入数据后自动同步索引。
4. 验证 MATCH 查询。
5. 验证 bm25 排序。
6. 验证 snippet 高亮。

验收：

```bash
ctx search 支付失败
ctx search payment
ctx search order-service
```

---

### 阶段 3：中文短词搜索验证

任务：

1. 实现 CJK 2-gram / 3-gram 生成。
2. 写入 `search_ngrams`。
3. 搜索时改写中文短词查询。
4. 验证 1 字、2 字、3 字以上查询效果。

验收：

```bash
ctx search 支付
ctx search 订单
ctx search 日志
ctx search 超时
```

---

### 阶段 4：trigram 子串搜索验证

任务：

1. 创建 records_trigram。
2. 验证 URL 片段。
3. 验证命令片段。
4. 验证中文长片段。
5. 验证少于 3 字符时的兜底策略。

验收：

```bash
ctx search payment_callback
ctx search clean package
ctx search /api/order
```

---

### 阶段 5：组合搜索验证

任务：

1. 标签 + 关键词。
2. 服务 + 关键词。
3. 环境 + 关键词。
4. 状态过滤。
5. 过期内容降权。

验收：

```bash
ctx search tag:runbook 支付
ctx search service:payment-service 401
ctx search env:test 构建
```

---

### 阶段 6：性能基准验证

任务：

1. 构造 100 条测试数据。
2. 构造 1,000 条测试数据。
3. 构造 10,000 条测试数据。
4. 测试 FTS 查询耗时。
5. 测试 snippet 查询耗时。
6. 测试 trigram 查询耗时。
7. 测试索引重建耗时。

验收：

```text
1 万条记录内，常见搜索 P95 < 100ms。
```

---

## 15. 风险与应对

### 15.1 SQLite FTS5 编译选项风险

风险：系统 SQLite 可能没有启用 FTS5。

应对：

```text
优先使用 rusqlite bundled 构建，降低系统 SQLite 差异。
```

---

### 15.2 中文分词风险

风险：SQLite FTS5 默认 tokenizer 不是中文分词器。

应对：

```text
应用层生成 CJK n-gram 字段，解决中文短词召回。
```

---

### 15.3 trigram 短词限制

风险：trigram 对少于 3 个 unicode 字符的查询不适合。

应对：

```text
中文 1-2 字查询使用 n-gram / LIKE 兜底。
```

---

### 15.4 FTS 索引不一致风险

风险：主表和 FTS 表不一致会导致搜索结果异常。

应对：

```text
使用触发器同步，并提供 ctx db rebuild-index。
```

---

### 15.5 排名不符合直觉风险

风险：纯 BM25 不一定符合个人上下文查询习惯。

应对：

```text
BM25 只作为基础相关性，应用层叠加 Key、标题、标签、服务、使用次数、更新时间等权重。
```

---

## 16. 修改后的 MVP 存储结论

原方案：

```text
JSONL 优先，SQLite 第二阶段。
```

新方案：

```text
SQLite + FTS5 优先，JSONL 作为导入导出和备份格式。
```

原因：

1. 用户明确追求极致搜索体验。
2. Context Hub 的核心价值就是搜索。
3. SQLite FTS5 可以同时支持全文检索、排序、snippet、高亮、前缀和子串搜索。
4. Rust + rusqlite 仍然可以保持单文件二进制分发。
5. JSONL 做主搜索会过早遇到体验上限。

---

## 17. 进入 MVP 开发的判断标准

满足以下条件后，可以正式进入 SQLite 搜索 MVP：

1. `rusqlite` bundled 跨平台构建成功。
2. records 主表读写成功。
3. records_fts 可用。
4. records_trigram 可用。
5. 插入、更新、删除后 FTS 索引一致。
6. 中文短词搜索可用。
7. 英文、服务名、Key、错误码、命令片段搜索可用。
8. snippet / highlight 可用。
9. 1 万条记录内搜索体验可接受。
10. JSONL 导入导出路径明确。

---

## 18. 一句话结论

> 如果 Context Hub 追求极致搜索体验，MVP 阶段应直接预研并采用 `rusqlite + SQLite FTS5` 作为主存储和主搜索方案，JSONL 只作为导入、导出和备份格式。

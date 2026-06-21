# Context Hub 技术预研结论

## 1. 结论摘要

Context Hub 技术预研结论：

```text
可以进入 Rust + SQLite 搜索 MVP 开发阶段。
```

推荐技术路线：

```text
Rust + clap + serde + rusqlite bundled + SQLite FTS5 + rmcp + GitHub Actions
```

核心判断：

1. Rust 适合作为 Context Hub 的首期实现语言。
2. SQLite 不需要用户单独安装，可以通过 `rusqlite` bundled 方式随应用一起编译和分发。
3. 为了极致搜索体验，MVP 阶段应直接采用 SQLite 作为主存储和主搜索底座。
4. JSONL 不再作为主存储，仅保留为导入、导出、备份和迁移格式。
5. MCP 首期只做只读查询，不做新增、修改、删除和命令执行。
6. 搜索能力是 Context Hub 的核心能力，应优先于 Web 后台、多用户权限和自动化能力。

---

## 2. 技术栈最终选择

### 2.1 语言

采用 Rust。

原因：

1. 可以生成跨平台可执行文件。
2. 用户不需要安装 Node/npm/Python 等运行时。
3. 适合构建本地优先 CLI 工具。
4. 依赖更可控。
5. 适合长期维护。
6. 与 SQLite 嵌入式模式匹配。

---

### 2.2 CLI 框架

采用 `clap`。

用途：

1. 子命令定义。
2. 参数解析。
3. help 输出。
4. 后续 shell completion。

MVP 命令：

```bash
ctx add
ctx search <keyword>
ctx tag <tag>
ctx show <key|index>
ctx copy <key|index>
ctx list-tags
ctx mcp
ctx db init
ctx db info
ctx db rebuild-index
ctx db export
ctx db import
```

---

### 2.3 存储和搜索

采用 `rusqlite + SQLite FTS5`。

推荐依赖：

```toml
rusqlite = { version = "0.40", features = ["bundled-full", "functions"] }
```

选择原因：

1. `rusqlite` 是 Rust 使用 SQLite 的成熟封装。
2. `bundled-full` 可以降低系统 SQLite 差异风险。
3. SQLite 可作为嵌入式库打包进应用。
4. SQLite FTS5 支持全文检索、BM25 排序、highlight、snippet、prefix index 和 trigram tokenizer。
5. 对个人本地上下文库来说，SQLite 的数据规模和性能足够。

---

### 2.4 MCP

采用 `rmcp`。

MVP 阶段只提供 MCP Server，只走只读工具。

首期 MCP tools：

```text
search_context(keyword: string)
get_context_by_key(key: string)
list_tags()
get_service_context(service: string)
```

明确不提供：

```text
add_context
update_context
delete_context
run_command
connect_server
read_secret
```

设计原则：

```text
MCP 是查询入口，不是执行入口。
```

---

### 2.5 打包和分发

采用 GitHub Actions 构建跨平台二进制。

目标平台：

1. Windows x64
2. macOS arm64
3. macOS x64
4. Linux x64
5. Linux arm64，可选

用户侧体验：

```text
下载 ctx 可执行文件
运行 ctx
自动创建 ~/.ctx-hub/ctx-hub.db
不需要安装 SQLite
不需要安装数据库服务
不需要安装 Node/npm/Python
```

---

## 3. SQLite 是否需要单独安装

结论：不需要。

Context Hub 使用 SQLite 的方式不是连接外部数据库服务，而是把 SQLite 当作嵌入式数据库引擎。

需要区分：

```text
SQLite 引擎：通过 rusqlite bundled 编译进应用。
ctx-hub.db：用户本地数据文件，由应用创建和维护。
```

也就是说，用户不会安装一个独立 SQLite 服务，也不需要执行数据库初始化命令。

推荐应用默认数据文件：

```text
~/.ctx-hub/ctx-hub.db
```

同时支持环境变量覆盖：

```bash
CTX_HUB_DB=/path/to/ctx-hub.db ctx search payment
```

---

## 4. 搜索方案最终结论

Context Hub 的核心价值是搜索，因此 MVP 搜索方案直接采用：

```text
records 主表 + records_fts 全文索引 + records_trigram 子串索引 + 应用层排序增强
```

### 4.1 主表

主表负责可靠存储。

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

### 4.2 FTS5 全文索引

用于常规全文搜索、前缀搜索、snippet、高亮和 BM25 排序。

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

### 4.3 Trigram 子串索引

用于命令片段、URL 片段、错误信息片段和较长中文片段查询。

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

### 4.4 中文短词搜索

SQLite FTS5 默认 tokenizer 不是中文分词器，因此中文搜索需要应用层增强。

策略：

1. 对中文内容生成 2-gram 和 3-gram。
2. 写入 `search_ngrams` 字段。
3. FTS 查询时同时匹配原文字段和 n-gram 字段。
4. n-gram 字段权重较低，只用于召回。

示例：

```text
支付失败排查规则
```

生成：

```text
支付 付失 失败 败排 排查 查规 规则 支付失败 失败排查 排查规则
```

需要重点验证：

```bash
ctx search 支付
ctx search 失败
ctx search 日志
ctx search 构建
ctx search 超时
```

---

## 5. 排序策略

不完全依赖 SQLite BM25。

最终采用：

```text
FTS5 BM25 + 应用层业务加权
```

排序优先级：

1. Key 完全匹配。
2. 标题完整命中。
3. 标签命中。
4. 服务名命中。
5. 正文命中。
6. 使用次数较高。
7. 更新时间较新。
8. 未过期内容优先。
9. active 状态优先。

建议字段权重：

| 字段 | 权重 |
| --- | --- |
| title | 10.0 |
| key | 8.0 |
| tags_text | 5.0 |
| service | 4.0 |
| content | 3.0 |
| env | 2.0 |
| source | 2.0 |
| record_type | 2.0 |
| search_ngrams | 1.0 |

原则：

```text
标题、Key、标签、服务名命中要明显优于正文命中。
```

---

## 6. 搜索结果展示

搜索结果必须提供足够上下文，而不是只显示标题。

推荐展示：

```text
[1] runbook.payment.failed
标题：支付失败排查规则
标签：payment, test, runbook
服务：payment-service
摘要：... [支付失败] 时先查询 payment_callback_log ...
更新时间：2026-06-21T10:00:00+09:00
```

必须支持：

1. 编号。
2. Key。
3. 标题。
4. 标签。
5. 服务和环境。
6. snippet 摘要。
7. 命中高亮。
8. 过期 / 废弃提示。

---

## 7. 数据模型结论

Rust Record 模型：

```rust
pub struct Record {
    pub id: String,
    pub key: Option<String>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub record_type: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
    pub usage_count: u64,
}
```

必填字段：

1. `id`
2. `title`
3. `content`
4. `status`
5. `created_at`
6. `updated_at`

可选字段：

1. `key`
2. `tags`
3. `record_type`
4. `service`
5. `env`
6. `source`
7. `expires_at`
8. `usage_count`

---

## 8. 推荐项目结构

```text
ctx-hub/
  Cargo.toml
  src/
    main.rs
    cli/
      mod.rs
      add.rs
      search.rs
      tag.rs
      show.rs
      copy.rs
      db.rs
      mcp.rs
    core/
      mod.rs
      record.rs
      search.rs
      query.rs
      ranking.rs
      ngram.rs
      sensitive.rs
    storage/
      mod.rs
      sqlite.rs
      schema.rs
      migration.rs
      import_export.rs
    mcp/
      mod.rs
      server.rs
      tools.rs
  tests/
    cli_search.rs
    sqlite_storage.rs
    fts_search.rs
    cjk_search.rs
    mcp_readonly.rs
  docs/
    requirements.md
    mvp.md
    rust-research-plan.md
    sqlite-search-research-plan.md
    technical-research-conclusion.md
```

核心原则：

```text
CLI 和 MCP 共享 core + storage，不重复实现业务逻辑。
```

---

## 9. MVP 开发优先级

进入 MVP 开发后，推荐顺序：

### 阶段 1：项目骨架

1. 初始化 Cargo 项目。
2. 接入 clap。
3. 定义子命令结构。
4. 实现 `ctx --help`。

### 阶段 2：SQLite 存储

1. 接入 rusqlite bundled。
2. 实现 `ctx db init`。
3. 创建 records 表。
4. 创建 FTS5 索引。
5. 创建触发器。

### 阶段 3：新增和查询闭环

1. 实现 `ctx add`。
2. 实现 `ctx search`。
3. 实现 `ctx show`。
4. 实现 snippet 展示。

### 阶段 4：搜索体验增强

1. 中文 n-gram。
2. trigram 子串搜索。
3. prefix 查询。
4. 标签 + 关键词组合查询。
5. 排序增强。

### 阶段 5：复制和标签

1. 实现 `ctx copy`。
2. 实现 `ctx tag`。
3. 实现 `ctx list-tags`。

### 阶段 6：MCP 只读查询

1. 接入 rmcp。
2. 实现 stdio transport。
3. 暴露 search_context。
4. 暴露 get_context_by_key。
5. 暴露 list_tags。
6. 确保 MCP 不提供写入和执行能力。

### 阶段 7：跨平台构建

1. GitHub Actions 构建 Windows / macOS / Linux。
2. 上传 release artifact。
3. 验证无需安装 SQLite 即可运行。

---

## 10. 性能目标

MVP 阶段建议性能目标：

| 数据量 | 目标 |
| --- | --- |
| 100 条 | 感知不到延迟 |
| 1,000 条 | 搜索接近瞬时 |
| 10,000 条 | 常见查询 P95 < 100ms |
| 100,000 条 | 压力测试，不作为 MVP 必须目标 |

必须验证：

1. Key 精确查询。
2. 单关键词搜索。
3. 中文短词搜索。
4. 英文 / 服务名搜索。
5. 错误码搜索。
6. 命令片段搜索。
7. 标签 + 关键词组合搜索。
8. snippet 查询性能。
9. trigram 查询性能。
10. FTS 重建性能。

---

## 11. 风险结论

### 11.1 SQLite FTS5 编译风险

风险：系统 SQLite 编译选项不一致。

结论：使用 `rusqlite` bundled/full 路线规避。

---

### 11.2 中文搜索风险

风险：SQLite 默认 tokenizer 不是中文分词器。

结论：通过应用层 CJK 2-gram / 3-gram 增强解决。

---

### 11.3 Trigram 短词限制

风险：trigram 对少于 3 个 unicode 字符的查询不友好。

结论：中文 1-2 字查询走 n-gram 或 LIKE 兜底。

---

### 11.4 MCP SDK 风险

风险：Rust MCP 生态相对 TypeScript/Python 更年轻。

结论：MCP 模块隔离，core 层不依赖 rmcp 类型；首期只做 stdio 只读查询。

---

### 11.5 剪贴板风险

风险：Linux X11 / Wayland 兼容性存在差异。

结论：复制失败时输出到 stdout，让用户手动复制。

---

### 11.6 数据迁移风险

风险：SQLite schema 后续可能变化。

结论：MVP 开始就引入 schema version 和 migration 表。

建议：

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
```

---

## 12. 需要暂缓的能力

以下能力不进入首期 MVP：

1. Web 管理后台。
2. 多用户权限。
3. 云同步。
4. 自动同步聊天软件。
5. 自动抓取网页。
6. 自动执行命令。
7. 查询远程日志。
8. 查询数据库。
9. Kubernetes 管理。
10. 向量检索。
11. 语义检索。
12. 知识图谱。

原因：

```text
首期必须先把本地记录和搜索体验做到足够好。
```

---

## 13. 进入 MVP 开发判断

满足。

进入 MVP 开发的条件：

1. 技术方向明确：Rust。
2. 存储方案明确：SQLite。
3. 搜索方案明确：FTS5 + trigram + n-gram + ranking。
4. 用户安装成本明确：不需要单独安装 SQLite。
5. AI 接入边界明确：MCP 只读。
6. 不做能力明确：自动执行、远程操作、Web、多用户暂缓。
7. 风险有应对方案。

因此：

```text
Context Hub 可以从技术预研阶段进入 Rust + SQLite MVP 开发阶段。
```

---

## 14. 官方资料依据

- rusqlite 文档：`https://docs.rs/rusqlite/latest/rusqlite/`
- rusqlite feature flags：`https://docs.rs/crate/rusqlite/latest/features`
- SQLite FTS5 文档：`https://www.sqlite.org/fts5.html`
- rmcp 文档：`https://docs.rs/rmcp/latest/rmcp/`
- clap 文档：`https://docs.rs/clap/latest/clap/`

---

## 15. 最终一句话结论

> Context Hub 技术预研完成，推荐进入 Rust + rusqlite + SQLite FTS5 的 MVP 开发阶段；SQLite 随应用打包，搜索体验优先，MCP 首期只读，JSONL 仅作为导入导出和备份格式。

# Context Hub MVP Development Plan

## 1. 背景

SQLite FTS POC 已经验证 Rust + rusqlite bundled + SQLite FTS5 + trigram + CJK n-gram 路线可行。

下一阶段不应继续在 POC 代码上直接堆功能，而应进入正式 MVP 开发，把已验证能力重构成清晰、可维护、可测试的工程结构。

MVP 的目标是交付一个本地优先的个人上下文管理工具，支持低成本记录、高质量搜索、快速查看和 AI 只读查询。

---

## 2. MVP 总体目标

MVP 必须完成以下闭环：

1. 用户可以通过 CLI 新增上下文记录。
2. 用户可以通过关键词、标签、Key 和服务名快速搜索。
3. 用户可以查看完整记录。
4. 用户可以复制记录内容。
5. AI 可以通过 MCP 只读查询上下文。
6. 数据本地持久化到 SQLite。
7. SQLite 随应用打包，不要求用户单独安装。
8. CI 在 Linux、macOS、Windows 上通过。

---

## 3. 技术路线

正式 MVP 采用以下技术路线：

```text
Rust + clap + serde + rusqlite bundled + SQLite FTS5 + rmcp + GitHub Actions
```

设计原则：

```text
CLI 和 MCP 共享 core / storage，不重复实现业务逻辑。
SQLite 是主存储和主搜索引擎。
JSONL 只作为导入、导出和备份格式。
MCP 首期只读，不提供写入和执行能力。
```

---

## 4. 推荐工程结构

```text
ctx-hub/
  Cargo.toml
  src/
    main.rs
    cli/
      mod.rs
      add.rs
      search.rs
      show.rs
      tag.rs
      copy.rs
      db.rs
      mcp.rs
    core/
      mod.rs
      record.rs
      query.rs
      search.rs
      ranking.rs
      ngram.rs
      output.rs
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
    cli_smoke.rs
    storage_sqlite.rs
    search_fts.rs
    search_cjk.rs
    mcp_readonly.rs
  docs/
```

POC 中的单文件 `src/main.rs` 只作为验证代码，不应作为正式结构长期保留。

---

## 5. 开发阶段

### 阶段 1：项目骨架重构

目标：从 POC 单文件结构重构为正式模块结构。

任务：

1. 拆分 `cli`、`core`、`storage`、`mcp` 模块。
2. 保留现有 POC 的可运行能力。
3. 建立统一错误处理。
4. 建立统一输出格式。
5. 保证 CI 继续通过。

验收：

1. `cargo fmt --all -- --check` 通过。
2. `cargo clippy --all-targets --all-features -- -D warnings` 通过。
3. `cargo test --all-features` 通过。
4. `cargo build --release --all-features` 通过。

---

### 阶段 2：SQLite 存储正式化

目标：将 POC 数据库代码改造成可维护的 storage 层。

任务：

1. 定义 `Record` 数据模型。
2. 定义 `Storage` trait。
3. 实现 `SqliteStorage`。
4. 实现 schema 初始化。
5. 实现 schema migration 表。
6. 实现数据库路径解析。
7. 支持 `CTX_HUB_DB` 覆盖数据库路径。

验收：

1. 可以初始化数据库。
2. 可以新增记录。
3. 可以通过 Key 查询记录。
4. 可以列出记录数量和数据库路径。
5. 测试可以使用临时数据库。

---

### 阶段 3：搜索能力正式化

目标：将 POC 搜索能力改造成稳定搜索模块。

任务：

1. 实现 FTS5 查询。
2. 实现 trigram 子串查询。
3. 实现 CJK 2-gram / 3-gram 生成。
4. 实现查询转义。
5. 实现 Key 精确查询优先。
6. 实现标签查询。
7. 实现服务名查询。
8. 实现基础排序。
9. 实现 snippet 展示。

验收：

1. 中文短词搜索可用。
2. 中文组合词搜索可用。
3. 英文搜索可用。
4. 服务名搜索可用。
5. 命令片段搜索可用。
6. Key 精确查询优先返回。
7. 搜索结果包含摘要。

---

### 阶段 4：CLI MVP

目标：完成用户侧核心命令。

首期命令：

```bash
ctx add
ctx search <keyword>
ctx tag <tag>
ctx show <key|id>
ctx copy <key|id>
ctx list-tags
ctx db init
ctx db info
ctx db rebuild-index
```

任务：

1. 完善 `ctx add` 参数。
2. 完善搜索结果展示。
3. 完善 `ctx show`。
4. 实现 `ctx copy`。
5. 实现 `ctx list-tags`。
6. 实现数据库维护命令。
7. 优化错误提示。

验收：

1. 用户可以新增记录。
2. 用户可以搜索记录。
3. 用户可以查看详情。
4. 用户可以复制内容。
5. 用户可以查看标签列表。
6. 用户可以重建搜索索引。

---

### 阶段 5：JSONL 导入导出

目标：提供备份和迁移能力。

任务：

1. 实现 `ctx db export --format jsonl`。
2. 实现 `ctx db import <file>`。
3. 明确导入冲突处理策略。
4. 明确导出字段格式。
5. 为导入导出增加测试。

验收：

1. 可以导出全部记录。
2. 可以从 JSONL 导入记录。
3. 重复 Key 有明确处理结果。
4. 导出数据可以人工查看。

---

### 阶段 6：MCP 只读查询

目标：让 AI 可以只读查询 Context Hub。

首期 MCP tools：

```text
search_context
get_context_by_key
list_tags
get_service_context
```

任务：

1. 接入 rmcp。
2. 实现 stdio transport。
3. MCP 调用 core 搜索能力。
4. MCP 不直接访问数据库细节。
5. MCP 不提供写入能力。
6. MCP 不提供命令执行能力。
7. 增加只读行为测试。

验收：

1. MCP client 可以搜索上下文。
2. MCP client 可以按 Key 获取记录。
3. MCP client 可以列出标签。
4. MCP 入口不能修改记录。

---

### 阶段 7：质量和发布

目标：保证 MVP 可交付、可安装、可持续维护。

任务：

1. 保持 rustfmt 必须通过。
2. 保持 clippy 必须通过。
3. 保持 Linux、macOS、Windows CI 通过。
4. 增加 release build workflow。
5. 产出平台二进制。
6. 编写安装说明。
7. 编写 MCP 配置说明。

验收：

1. 三平台 CI 全部通过。
2. release build 成功。
3. 用户无需单独安装 SQLite。
4. 用户可以根据文档运行 `ctx`。
5. 用户可以根据文档配置 MCP。

---

## 6. 非 MVP 范围

以下能力不进入首期 MVP：

1. Web 管理后台。
2. 多用户权限系统。
3. 云同步。
4. 自动同步聊天工具。
5. 自动抓取网页。
6. 自动执行命令。
7. 查询远程日志。
8. 查询业务数据库。
9. Kubernetes 管理。
10. 向量检索。
11. 语义检索。
12. 知识图谱。

---

## 7. MVP 完成标准

MVP 完成时应满足：

1. CLI 核心命令可用。
2. SQLite 存储稳定。
3. 搜索体验优先满足个人使用。
4. 中文短词、英文、服务名、命令片段都可以搜索。
5. MCP 只读查询可用。
6. JSONL 导入导出可用。
7. CI 全部通过。
8. 文档覆盖安装、CLI、MCP、备份恢复。

---

## 8. 下一步建议

不要直接将 POC 代码合入 main 作为正式实现。

建议下一步创建新的正式开发分支：

```text
feat/mvp-core
```

然后按阶段 1 开始，把 POC 代码重构为正式模块结构。

POC PR 可以继续保留为技术验证记录，等正式 MVP 分支启动后再决定是否关闭或合并文档部分。

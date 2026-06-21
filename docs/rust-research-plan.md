# Context Hub Rust 技术预研计划

## 1. 预研目标

Context Hub 已确定 MVP 阶段采用 Rust 作为主要实现语言。

本预研的目标不是直接完成业务功能，而是验证 Rust 技术路线是否能够稳定支撑 Context Hub 的 MVP：

1. 跨平台 CLI 工具。
2. 本地持久化存储。
3. 关键词、标签和 Key 查询。
4. 快速复制查询结果。
5. MCP 只读查询。
6. 单人可维护、低依赖、易分发。

预研完成后，应能够回答：

```text
Rust 是否适合作为 ctx-hub 的首期实现语言？
采用哪些 crate？
首期存储方案选什么？
MCP 只读查询是否可稳定实现？
跨平台打包和安装是否足够简单？
```

---

## 2. 技术路线结论

MVP 阶段建议技术方向：

```text
Rust + clap + serde + 本地存储 + rmcp + GitHub Actions 跨平台构建
```

首期目标不是复杂架构，而是做一个可以长期使用的本地工具。

推荐优先级：

1. 先完成 CLI 能力。
2. 再完成本地存储和搜索。
3. 再完成 MCP 只读查询。
4. 最后补充跨平台构建和发布。

---

## 3. 预研范围

### 3.1 CLI 框架预研

候选：

- `clap`

需要验证：

1. 子命令是否方便实现。
2. 参数解析是否清晰。
3. help 输出是否友好。
4. 是否支持 shell completion。
5. Windows / macOS / Linux 下表现是否一致。

需要实现的预研命令：

```bash
ctx add
ctx search <keyword>
ctx tag <tag>
ctx show <key|index>
ctx copy <key|index>
ctx list-tags
ctx mcp
```

预研验收：

```text
可以用 clap 定义完整 MVP 命令结构，并输出清晰 help 信息。
```

---

### 3.2 数据模型预研

需要定义核心 Record 模型。

推荐结构：

```rust
pub struct Record {
    pub id: String,
    pub title: String,
    pub content: String,
    pub key: Option<String>,
    pub tags: Vec<String>,
    pub record_type: Option<String>,
    pub service: Option<String>,
    pub env: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
    pub usage_count: u64,
}
```

需要验证：

1. `serde` 序列化 / 反序列化。
2. JSONL 存储是否方便。
3. YAML / Markdown Front Matter 是否有必要。
4. 字段升级是否容易兼容。

预研验收：

```text
可以把 Record 写入本地文件，再完整读回，并支持字段缺失兼容。
```

---

### 3.3 存储方案预研

候选方案：

1. JSONL
2. Markdown + YAML Front Matter
3. SQLite
4. SQLite FTS

MVP 建议优先验证：

```text
JSONL 优先，SQLite 作为第二阶段方案。
```

原因：

1. JSONL 无原生依赖。
2. 数据可人工查看。
3. 容易备份和迁移。
4. 写入简单。
5. 适合个人工具早期阶段。

需要验证：

1. 默认数据目录选择：`~/.ctx-hub/records.jsonl`。
2. Windows 路径兼容。
3. 文件不存在时自动初始化。
4. 追加写入是否简单。
5. 修改 / 删除是否需要重写整个文件。
6. 未来迁移 SQLite 的成本。

预研验收：

```text
可以完成 add/search/show/tag/list-tags 的本地 JSONL 读写闭环。
```

---

### 3.4 搜索方案预研

MVP 搜索先使用内存扫描，不引入复杂全文检索。

搜索范围：

- title
- content
- key
- tags
- service
- env
- source

需要验证：

1. 大小写不敏感搜索。
2. 中文搜索。
3. 英文搜索。
4. 数字和错误码搜索。
5. 多关键词搜索。
6. 搜索结果排序。
7. 匹配摘要展示。

建议排序：

1. Key 完全匹配
2. 标题命中
3. 标签命中
4. 服务名命中
5. 正文命中
6. 更新时间较新
7. 使用次数较高

预研验收：

```text
1000 条记录内搜索响应足够快，结果排序符合预期。
```

---

### 3.5 快速复制预研

候选：

- `arboard`

需要验证：

1. macOS 剪贴板写入。
2. Windows 剪贴板写入。
3. Linux X11 / Wayland 兼容性。
4. 复制正文、URL、命令和整条记录。
5. 失败时是否可以降级为输出到 stdout。

预研验收：

```text
ctx copy 可以在主流平台复制内容；复制失败时给出清晰提示，并输出可手动复制内容。
```

---

### 3.6 MCP Server 预研

候选：

- `rmcp`

MVP 阶段只需要 MCP Server，只读查询 Context Hub。

需要验证的 MCP 能力：

1. stdio transport。
2. tool 定义。
3. search_context 工具。
4. get_context_by_key 工具。
5. list_tags 工具。
6. read-only 限制。
7. 错误返回格式。
8. 与本地 CLI 共享同一套 core 逻辑。

建议首期 MCP tools：

```text
search_context(keyword: string)
get_context_by_key(key: string)
list_tags()
get_service_context(service: string)
```

MCP 明确不提供：

```text
add_context
update_context
delete_context
run_command
connect_server
read_secret
```

预研验收：

```text
可以通过 MCP stdio 调用 search_context，并读取本地 ctx-hub 数据。
```

---

### 3.7 敏感信息提示预研

MVP 不做复杂权限系统，只做新增时提示。

需要验证：

1. 对 `password`、`token`、`secret`、`cookie`、`private key` 等关键词提示。
2. 对疑似私钥格式提示。
3. 用户确认后仍可保存。
4. MCP 查询时是否需要对疑似敏感内容打码。

MVP 建议策略：

```text
CLI 新增时：提示但不强制拦截。
MCP 查询时：默认不返回疑似敏感字段，或者返回前打码。
```

预研验收：

```text
可以识别常见疑似敏感信息，并给出可理解的提示。
```

---

### 3.8 跨平台构建预研

目标平台：

1. Windows x64
2. macOS arm64
3. macOS x64
4. Linux x64
5. Linux arm64，可选

需要验证：

1. GitHub Actions 构建 Rust release。
2. 生成平台对应二进制。
3. Windows 下是否生成 `.exe`。
4. 不依赖 Node/npm/Python 运行时。
5. 下载后是否可以直接运行。
6. 配置目录是否跨平台正确。

预研验收：

```text
GitHub Actions 可以产出 Windows、macOS、Linux 的可执行文件。
```

---

## 4. 推荐 crate 清单

MVP 阶段建议优先验证以下 crate：

| 能力 | 候选 crate | 用途 |
| --- | --- | --- |
| CLI | `clap` | 命令行参数和子命令 |
| 序列化 | `serde` | 数据模型序列化 / 反序列化 |
| JSON | `serde_json` | JSONL 存储 |
| 时间 | `chrono` 或 `time` | created_at / updated_at |
| 用户目录 | `directories` | 获取跨平台配置目录 |
| 错误处理 | `anyhow` / `thiserror` | 错误管理 |
| 剪贴板 | `arboard` | 快速复制 |
| MCP | `rmcp` | MCP Server |
| 异步运行时 | `tokio` | MCP 和异步能力 |
| SQLite | `rusqlite` | 第二阶段存储候选 |

MVP 不建议一开始引入太多依赖。

---

## 5. 推荐目录结构

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
      list_tags.rs
      mcp.rs
    core/
      mod.rs
      record.rs
      search.rs
      sensitive.rs
      error.rs
    storage/
      mod.rs
      jsonl.rs
    mcp/
      mod.rs
      server.rs
      tools.rs
  tests/
    cli_add.rs
    cli_search.rs
    storage_jsonl.rs
  docs/
    requirements.md
    mvp.md
    rust-research-plan.md
```

设计原则：

```text
CLI 和 MCP 不能各写一套业务逻辑。
CLI 和 MCP 都必须调用 core + storage。
```

---

## 6. 预研执行顺序

### 阶段 1：Rust 项目骨架

目标：创建最小可运行 Rust 项目。

任务：

1. 初始化 Cargo 项目。
2. 添加 `clap`。
3. 实现 `ctx --help`。
4. 实现空的子命令结构。

验收：

```bash
ctx --help
ctx add --help
ctx search --help
```

---

### 阶段 2：Record + JSONL 存储

目标：完成本地数据读写闭环。

任务：

1. 定义 Record。
2. 实现 JSONL append。
3. 实现 JSONL load_all。
4. 默认存储到 `~/.ctx-hub/records.jsonl`。
5. 支持测试数据目录覆盖。

验收：

```bash
ctx add --title "支付失败" --content "先查 payment_callback_log"
ctx search 支付失败
```

---

### 阶段 3：搜索和展示

目标：完成人用 CLI 的核心查询体验。

任务：

1. 关键词搜索。
2. 标签查询。
3. Key 查询。
4. 搜索结果摘要。
5. 排序规则。

验收：

```bash
ctx search payment
ctx tag runbook
ctx show runbook.payment.failed
```

---

### 阶段 4：复制能力

目标：快速复制查询结果。

任务：

1. 接入剪贴板 crate。
2. 支持复制 content。
3. 支持复制 title / key / full。
4. 失败时输出到 stdout。

验收：

```bash
ctx copy runbook.payment.failed
ctx copy runbook.payment.failed --field content
```

---

### 阶段 5：MCP 只读查询

目标：验证 AI 可以通过 MCP 查询 ctx-hub。

任务：

1. 接入 `rmcp`。
2. 实现 stdio transport。
3. 暴露只读 tools。
4. 调用 core search。
5. 禁止新增、修改、删除和执行命令。

验收：

```text
MCP Client 可以调用 search_context，并返回本地记录。
```

---

### 阶段 6：跨平台构建

目标：验证发布方式。

任务：

1. 添加 GitHub Actions。
2. 构建 Windows / macOS / Linux 二进制。
3. 上传 artifact。
4. 本地验证运行。

验收：

```text
下载二进制后无需安装 Node/npm/Python，直接运行 ctx。
```

---

## 7. 关键风险

### 7.1 MCP SDK 风险

Rust MCP SDK 生态比 TypeScript/Python 晚一些，需要重点验证：

1. 文档完整度。
2. stdio transport 是否稳定。
3. tool schema 定义是否简单。
4. 与主流 MCP Client 兼容性。
5. 后续协议版本变更成本。

应对策略：

```text
MCP 模块隔离，不让 rmcp 类型污染 core 层。
```

---

### 7.2 剪贴板兼容风险

Linux 桌面环境差异较大，剪贴板可能失败。

应对策略：

```text
复制失败时不阻塞主流程，直接把内容输出到 stdout。
```

---

### 7.3 存储升级风险

JSONL 简单，但后续复杂搜索可能不够。

应对策略：

```text
定义 Storage trait，MVP 使用 JsonlStorage，后续可以增加 SqliteStorage。
```

---

### 7.4 搜索能力不足风险

简单内存扫描不如全文检索。

应对策略：

```text
先验证 1000 条记录内体验；超过规模后再引入 SQLite FTS。
```

---

## 8. 不在本轮预研范围

本轮不预研：

1. Web 管理后台。
2. 多人权限系统。
3. 云端同步。
4. 自动同步聊天软件。
5. 自动抓取网页。
6. 自动执行命令。
7. 查询远程日志。
8. 数据库查询。
9. Kubernetes 管理。
10. 向量检索。
11. 语义检索。

---

## 9. 预研产出物

本轮预研结束后，应该产出：

1. Rust 项目骨架。
2. CLI 命令结构。
3. Record 数据模型。
4. JSONL 存储验证。
5. 搜索验证。
6. copy 验证。
7. MCP stdio 只读查询验证。
8. GitHub Actions 跨平台构建验证。
9. 技术结论文档。
10. 是否进入正式 MVP 开发的结论。

---

## 10. 进入正式开发的判断标准

满足以下条件后，可以进入正式 MVP 开发：

1. Rust CLI 命令结构清晰。
2. JSONL 存储可用。
3. 搜索体验满足个人使用。
4. 复制功能可用，失败时有降级方案。
5. MCP 只读查询跑通。
6. core 层可以被 CLI 和 MCP 复用。
7. 跨平台构建方案明确。
8. 没有发现阻塞性技术风险。

---

## 11. 一句话结论

> ctx-hub 的 Rust 技术预研应优先验证 CLI、本地存储、搜索、复制和 MCP 只读查询闭环；只要这些能力跑通，就可以进入 Rust MVP 正式开发。

# SQLite FTS POC Demo

## 1. 目标

本 POC 用于验证 Context Hub 的工程级技术路线：

```text
Rust + rusqlite bundled + SQLite FTS5 + trigram + CJK n-gram
```

重点验证：

1. Rust CLI 可以正常启动。
2. SQLite 不需要用户单独安装。
3. 应用可以自动创建本地数据库。
4. records 主表可以写入数据。
5. FTS5 全文索引可以搜索。
6. trigram 子串索引可以搜索命令片段和路径片段。
7. 中文短词可以通过应用层 n-gram 召回。
8. 搜索结果可以返回 snippet。

---

## 2. 当前分支

```bash
research/rust-sqlite-fts
```

该分支是预研分支，不直接代表正式 MVP 实现。

---

## 3. 构建

```bash
cargo build
```

如果构建成功，说明 `rusqlite` 的 bundled SQLite 路线可用。

---

## 4. 使用临时数据库测试

建议预研时使用临时数据库，避免污染真实数据：

```bash
export CTX_HUB_DB=/tmp/ctx-hub-poc.db
```

Windows PowerShell：

```powershell
$env:CTX_HUB_DB="$env:TEMP\ctx-hub-poc.db"
```

---

## 5. 初始化数据库

```bash
cargo run -- db init
```

预期输出：

```text
initialized: /tmp/ctx-hub-poc.db
```

---

## 6. 新增测试记录

```bash
cargo run -- add \
  --key runbook.payment.failed \
  --title "支付失败排查规则" \
  --content "支付失败时先查询 payment_callback_log，再查询 payment-service 日志。错误码 401 需要检查 mock token。" \
  --tag payment \
  --tag runbook \
  --service payment-service \
  --env test \
  --source "支付组同步"
```

再新增一条命令记录：

```bash
cargo run -- add \
  --key command.order.build \
  --title "order-service 构建命令" \
  --content "mvn clean package -DskipTests -Ptest" \
  --tag order-service \
  --tag build \
  --service order-service \
  --env test
```

---

## 7. 搜索验证

### 7.1 中文短词搜索

```bash
cargo run -- search 支付
cargo run -- search 失败
cargo run -- search 日志
```

预期：可以找到 `runbook.payment.failed`。

---

### 7.2 中文组合词搜索

```bash
cargo run -- search 支付失败
```

预期：可以找到支付失败排查规则，并返回 snippet。

---

### 7.3 英文和服务名搜索

```bash
cargo run -- search payment
cargo run -- search payment-service
cargo run -- search order-service
```

预期：可以找到对应服务上下文。

---

### 7.4 命令片段搜索

```bash
cargo run -- search "clean package"
cargo run -- search DskipTests
```

预期：可以找到构建命令记录。

---

### 7.5 Key 精确查询

```bash
cargo run -- show runbook.payment.failed
cargo run -- show command.order.build
```

预期：精确展示完整记录。

---

### 7.6 标签查询

```bash
cargo run -- tag runbook
cargo run -- tag build
cargo run -- list-tags
```

预期：可以按标签筛选并统计标签数量。

---

## 8. 数据库信息

```bash
cargo run -- db info
```

预期输出当前数据库路径和记录数量。

---

## 9. 重建索引

```bash
cargo run -- db rebuild-index
```

预期：FTS5 和 trigram 索引可以重建成功。

---

## 10. 当前 POC 边界

当前 POC 只验证搜索核心可行性，不包含：

1. MCP Server。
2. 复制到剪贴板。
3. 完整错误处理。
4. 数据迁移版本管理。
5. JSONL 导入导出。
6. GitHub Actions 跨平台构建。
7. 完整测试用例。
8. 生产级搜索排序。

这些能力应在 POC 验证通过后再进入正式 MVP 开发。

---

## 11. POC 通过标准

满足以下条件即可认为 SQLite FTS 路线工程可行：

1. `cargo build` 成功。
2. `cargo run -- db init` 成功。
3. 新增记录成功。
4. 中文短词搜索成功。
5. 中文组合词搜索成功。
6. 英文、服务名搜索成功。
7. 命令片段搜索成功。
8. Key 精确查询成功。
9. 不需要单独安装 SQLite。

---

## 12. 结论模板

验证完成后，在 PR 或后续文档中记录：

```text
SQLite FTS POC 验证结果：通过 / 不通过
验证平台：Windows / macOS / Linux
Rust 版本：
是否需要单独安装 SQLite：否
中文短词搜索：通过 / 不通过
trigram 子串搜索：通过 / 不通过
snippet 展示：通过 / 不通过
主要问题：
下一步建议：
```

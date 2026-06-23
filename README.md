# Context Hub

Context Hub 是一个面向个人和 AI 的上下文集中管理工具。

它用于集中存储日常工作中有价值的信息，例如服务地址、接口文档、常用命令、构建规则、排查经验、历史问题、他人同步的信息和临时注意事项，并支持通过关键词、标签和唯一 Key 快速查询与复用。

## 项目目标

Context Hub 的目标是把零散信息从聊天记录、浏览器收藏、个人笔记、命令历史和大脑记忆中抽离出来，沉淀为一个可搜索、可复用、可给 AI 使用的外部记忆库。

核心目标：

- 集中管理日常高价值信息
- 通过关键词快速搜索
- 通过标签快速筛选
- 通过唯一 Key 精确定位
- 支持快速复制查询结果
- 为 AI 提供统一上下文入口
- 减少人的记忆负担和 AI 的上下文猜测

## 适合记录的内容

Context Hub 适合记录：

- 服务地址
- 测试环境地址
- 接口文档地址
- 常用命令
- 构建方式
- Maven / JDK / Profile 配置说明
- 日志查询方式
- 数据库和表说明
- 排查流程
- 历史 bug 经验
- 他人同步的重要信息
- 临时注意事项
- 服务负责人和依赖关系

不建议保存明文敏感信息，例如密码、Token、Cookie、SSH 私钥、数据库密码和生产密钥。敏感信息应只保存引用，例如 `credential_ref`。

## 技术方向

Context Hub MVP 阶段优先采用 Rust 实现。

选择 Rust 的主要原因：

- 跨平台分发更简单
- 不依赖 Node/npm/Python 运行时
- 适合构建本地 CLI 工具
- 可以生成单个可执行文件
- 依赖更可控
- 适合长期维护本地工具

MVP 推荐技术路线：

```text
Rust + clap + serde + rusqlite + SQLite FTS5 + stdio MCP + GitHub Actions
```

SQLite 不要求用户单独安装。MVP 阶段应使用 `rusqlite` 的 bundled 构建方式，将 SQLite 作为嵌入式库随应用一起编译和分发。用户安装 Context Hub 后，只需要运行 `ctx` 可执行文件；应用会在本地创建和维护 `ctx-hub.db` 数据文件。

JSONL 不再作为主存储和主搜索方案，仅保留为导入、导出、备份和迁移格式。

## 安装与快速开始

前置要求：

- 已安装 Rust stable 工具链
- 本地可以运行 `cargo`

从源码目录构建 release 版本：

```bash
cargo build --release --all-features
```

构建完成后可以直接运行生成的二进制：

```bash
# Linux / macOS
./target/release/ctx db init

# Windows PowerShell
.\target\release\ctx.exe db init
```

也可以安装到 Cargo bin 目录，之后直接使用 `ctx`：

```bash
cargo install --path .
ctx db init
```

下面的示例默认 `ctx` 已经在 `PATH` 中；如果没有安装，请把 `ctx` 替换为 `./target/release/ctx` 或 `.\target\release\ctx.exe`。

初始化数据库并添加、搜索一条记录：

```bash
ctx db init
ctx add --key runbook.payment.failed --title "支付失败排查规则" --content "先查 payment_callback_log" --tag payment --tag runbook
ctx search payment
ctx show runbook.payment.failed
```

## 核心能力

首期能力：

- 快速新增信息
- 关键词全文搜索
- 标签查询
- 唯一 Key 精确查询
- 查看完整记录
- 快速复制记录内容
- AI 通过 MCP 只读查询上下文

后续能力：

- 命令模板管理
- 构建上下文管理
- 服务上下文管理
- 更完善的结构化查询
- 更好的搜索排序
- 信息过期提醒
- Web 管理界面

## 推荐记录格式

每条记录建议包含标题、正文、创建时间和更新时间，也可以设置唯一 Key、标签、服务、环境、来源、状态、失效时间和使用次数。

示例：

```yaml
id: ctx_001
key: runbook.payment.failed
title: 支付失败排查规则
tags:
  - payment
  - test
  - runbook
service: payment-service
env: test
type: runbook
source: 支付组同步
status: active
created_at: 2026-06-21T10:00:00+09:00
updated_at: 2026-06-21T10:00:00+09:00
expires_at:
usage_count: 0
content: 支付失败时先查询 payment_callback_log，再查询 payment-service 日志。
```

## 当前 CLI

可用命令：

```bash
ctx db init
ctx db info
ctx db rebuild-index
ctx db export --format jsonl
ctx db import <file>
ctx add --title <title> --content <content> [--key <key>] [--tag <tag>]
ctx search <keyword>
ctx tag <tag>
ctx show <key-or-id>
ctx copy <key-or-id> [--field content|command|url|key|title|full]
ctx list-tags
ctx mcp
```

全局数据库路径可以通过 `--db <path>` 指定：

```bash
ctx --db /absolute/path/to/ctx-hub.db db init
ctx --db /absolute/path/to/ctx-hub.db search payment
```

示例：

```bash
ctx db init
ctx add --key runbook.payment.failed --title "支付失败排查规则" --content "先查 payment_callback_log" --tag payment --tag runbook
ctx db export --format jsonl > records.jsonl
ctx db import records.jsonl
ctx search payment
ctx search "支付失败"
ctx tag runbook
ctx show runbook.payment.failed
ctx copy runbook.payment.failed
ctx copy runbook.payment.failed --field full --print
ctx mcp
```

`ctx copy` 默认复制正文到系统剪贴板；`--field` 可以选择复制正文、命令/URL 内容、Key、标题或整条记录。没有可用剪贴板命令时，CLI 会提示原因并把选中的内容输出到 stdout，方便手动复制。`--print` 会直接输出选中内容，不访问剪贴板。

`ctx db export --format jsonl` 会把 active 记录输出为 JSONL。每行包含 `schema_version`、`id`、`key`、`title`、`content`、`tags`、`service`、`env`、`source`、`created_at` 和 `updated_at`。`ctx db import <file>` 导入相同 schema；如果目标库已经存在相同 `id` 或非空 `key`，该行会被跳过并计入 `skipped_duplicates`，不会覆盖现有记录。

## 数据库路径

所有命令使用同一套数据库路径解析规则，优先级从高到低为：

1. 命令行全局参数 `--db <path>`
2. 环境变量 `CTX_HUB_DB`
3. 默认路径

默认路径：

```text
Linux / macOS: ~/.ctx-hub/ctx-hub.db
Windows:       %USERPROFILE%\.ctx-hub\ctx-hub.db
```

打开数据库时会自动创建父目录。使用环境变量可以把所有命令固定到同一个自定义数据库：

```bash
export CTX_HUB_DB="$HOME/.ctx-hub/work.db"
ctx db init
ctx search payment
```

Windows PowerShell：

```powershell
$env:CTX_HUB_DB="$env:USERPROFILE\.ctx-hub\work.db"
ctx db init
ctx search payment
```

`--db` 适合临时指定某个库，优先级高于 `CTX_HUB_DB`：

```bash
ctx --db ./tmp/ctx-hub.db db init
ctx --db ./tmp/ctx-hub.db add --title "本地测试" --content "临时库记录"
```

## 备份和恢复

推荐使用 JSONL 做备份和迁移：

```bash
ctx db export --format jsonl > ctx-hub.backup.jsonl
```

恢复到当前数据库：

```bash
ctx db import ctx-hub.backup.jsonl
```

恢复到新数据库：

```bash
CTX_HUB_DB="$PWD/restored.db" ctx db init
CTX_HUB_DB="$PWD/restored.db" ctx db import ctx-hub.backup.jsonl
CTX_HUB_DB="$PWD/restored.db" ctx search payment
```

导入不会覆盖已有记录。如果目标库已经存在相同 `id` 或非空 `key`，该行会被跳过并计入 `skipped_duplicates`。

如果要直接复制 SQLite 数据库文件，请先停止正在使用该数据库的 `ctx mcp` 进程或其他写入命令。跨版本迁移优先使用 JSONL，因为它是明确支持的导入、导出和备份格式。

`ctx mcp` 通过 stdio 启动只读 MCP server。MVP 只提供这些工具：`search_context`、`get_context_by_key`、`list_tags`、`get_service_context`。MCP 不提供新增、导入、导出、修改、删除或命令执行能力。

示例 MCP 配置：

```json
{
  "mcpServers": {
    "ctx-hub": {
      "command": "ctx",
      "args": ["--db", "/absolute/path/to/ctx-hub.db", "mcp"]
    }
  }
}
```

如果 `ctx` 不在 MCP 客户端的 `PATH` 中，请把 `command` 改成可执行文件的绝对路径，例如 `/Users/me/.cargo/bin/ctx`、`/absolute/path/to/target/release/ctx` 或 `C:\\Users\\me\\.cargo\\bin\\ctx.exe`。建议显式传入 `--db`，这样 MCP 客户端和日常 CLI 命令会读取同一个数据库。

## AI 使用方式

AI 在处理服务、环境、构建、日志、接口和命令相关问题时，应优先查询 Context Hub，而不是直接猜测。

示例流程：

```text
用户提出问题
  ↓
AI 查询 Context Hub
  ↓
获取服务信息、文档地址、命令或排查经验
  ↓
AI 再结合代码、日志或其他工具继续分析
```

MVP 阶段，AI 只允许通过 MCP 读取上下文，不负责新增、修改、删除记录，也不自动执行记录中的命令。

## 项目原则

```text
集中管理 > 完美分类
能搜到 > 结构漂亮
先可用 > 后智能
只读查询 > 自动执行
搜索体验优先 > 存储实现简单
```

Context Hub 的第一目标不是复杂知识库，而是简单、快速、可持续地记录和找回信息。

## 文档

- [需求文档](docs/requirements.md)
- [MVP 范围](docs/mvp.md)
- [Rust 技术预研计划](docs/rust-research-plan.md)
- [SQLite / rusqlite 搜索预研计划](docs/sqlite-search-research-plan.md)

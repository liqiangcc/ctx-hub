# Context Hub MVP 范围

## 1. MVP 目标

Context Hub 的 MVP 目标是验证一个核心假设：

> 用户是否可以用足够低的成本，把日常工作中的高价值上下文沉淀下来，并让人和 AI 都能快速查到。

MVP 不追求完整知识库能力，也不追求复杂自动化能力。

MVP 只解决三个问题：

1. 信息能快速记录下来。
2. 信息能通过关键词、标签和 Key 快速找回来。
3. AI 能通过只读方式查询同一份上下文，减少猜测。

---

## 2. MVP 核心原则

```text
低摩擦录入 > 完整字段
能搜到 > 结构漂亮
只读查询 > 自动执行
本地可控 > 复杂平台化
先可用 > 后智能
```

Context Hub 在 MVP 阶段应保持简单、轻量、可持续维护。

---

## 3. MVP 必须支持的能力

### 3.1 CLI 新增记录

支持通过命令行快速新增一条记录。

新增时必须支持：

- 标题
- 内容
- 创建时间

新增时可选支持：

- 唯一 Key
- 标签
- 类型
- 服务名
- 环境
- 来源

示例：

```bash
ctx add
ctx add --title "order-service 构建命令" --tag order-service --tag build
```

录入时不应强制用户填写大量字段。

---

### 3.2 关键词搜索

支持通过关键词搜索记录。

搜索范围至少包括：

- 标题
- 内容
- Key
- 标签
- 服务名
- 环境
- 来源

示例：

```bash
ctx search payment
ctx search "支付失败"
ctx search "order-service 构建"
```

搜索应支持：

- 中文
- 英文
- 数字
- 错误码
- 服务名
- 命令片段

---

### 3.3 标签查询

支持通过标签筛选记录。

示例：

```bash
ctx tag maven
ctx tag payment
ctx tag runbook
```

一条记录可以有多个标签。

MVP 阶段不强制所有记录必须有标签。

---

### 3.4 Key 精确查询

支持通过唯一 Key 精确查询记录。

示例：

```bash
ctx show runbook.payment.failed
ctx show command.order-service.build
```

Key 用于重要信息的稳定引用，便于用户和 AI 精确获取上下文。

---

### 3.5 查看完整记录

支持查看一条记录的完整内容。

查询结果至少展示：

- 编号
- Key
- 标题
- 标签
- 匹配摘要
- 来源
- 创建时间或更新时间

详情页至少展示：

- 标题
- 内容
- Key
- 标签
- 类型
- 服务名
- 环境
- 来源
- 创建时间
- 更新时间

---

### 3.6 快速复制

支持快速复制查询结果中的内容。

至少支持复制：

- 整条记录
- 正文
- URL
- 命令
- Key
- 标题

示例：

```bash
ctx copy runbook.payment.failed
ctx copy command.order-service.build --field content
```

快速复制是 MVP 的重要能力，因为 Context Hub 的核心价值之一是减少重复查找和重复粘贴。

---

### 3.7 MCP 只读查询

MVP 阶段应支持 AI 通过 MCP 只读查询 Context Hub。

MCP 首期只提供查询能力，不提供修改和删除能力。

AI 可使用的能力包括：

- 关键词搜索上下文
- 标签筛选上下文
- Key 精确获取记录
- 获取某个服务相关上下文

AI 不应通过 MCP：

- 新增记录
- 修改记录
- 删除记录
- 自动执行记录中的命令
- 自动连接服务器
- 自动读取敏感凭证

---

### 3.8 本地持久化存储

MVP 阶段必须支持本地持久化。

可选方案：

- SQLite
- Markdown + YAML Front Matter
- JSONL
- YAML 文件

MVP 不强制最终存储方案，但必须满足：

- 可备份
- 可迁移
- 可人工查看
- 不依赖复杂外部服务
- 单人使用成本低

---

### 3.9 疑似敏感信息提示

MVP 阶段不做复杂权限系统，但应在新增记录时对疑似敏感信息进行提示。

疑似敏感信息包括：

- password
- token
- secret
- cookie
- private key
- ssh key
- api key
- 数据库密码
- 生产密钥

MVP 阶段建议策略：

```text
发现疑似敏感信息时提示用户确认，不默认强制拦截。
```

允许保存凭证引用，例如：

```yaml
credential_ref: vault://order/test-readonly
```

---

## 4. MVP 不支持的能力

MVP 不支持：

1. Web 管理后台
2. 多用户权限系统
3. 自动同步聊天软件
4. 自动抓取网页
5. 自动执行命令
6. 自动连接服务器
7. 自动查询日志文件
8. 自动查询数据库
9. 自动构建代码
10. 自动管理 Kubernetes
11. 向量检索
12. 语义检索
13. 自动分类
14. 复杂知识图谱
15. 自动判断信息真伪
16. 自动解决业务问题

这些能力可以由后续版本或其他独立 MCP 工具提供。

Context Hub 在 MVP 阶段只负责：

```text
记录上下文、查询上下文、给 AI 提供只读上下文。
```

---

## 5. 推荐数据模型

MVP 推荐记录模型：

```yaml
id: ctx_001
title: 支付失败排查规则
content: 支付失败时先查询 payment_callback_log，再查询 payment-service 日志。
key: runbook.payment.failed
tags:
  - payment
  - test
  - runbook
type: runbook
service: payment-service
env: test
source: 支付组同步
status: active
created_at: 2026-06-21T10:00:00+09:00
updated_at: 2026-06-21T10:00:00+09:00
expires_at:
usage_count: 0
```

### 5.1 必填字段

- `id`
- `title`
- `content`
- `created_at`
- `updated_at`

### 5.2 可选字段

- `key`
- `tags`
- `type`
- `service`
- `env`
- `source`
- `status`
- `expires_at`
- `usage_count`

---

## 6. 搜索排序规则

MVP 搜索结果建议按以下优先级排序：

1. Key 完全匹配
2. 标题命中
3. 标签命中
4. 服务名命中
5. 正文命中
6. 更新时间较新
7. 使用次数较高

`usage_count` 可以作为后续优化搜索排序的基础字段。

---

## 7. 推荐 CLI 命令

MVP 推荐命令：

```bash
ctx add
ctx search <keyword>
ctx tag <tag>
ctx show <key|id>
ctx copy <key|id> [--field content|command|url|key|title|full]
ctx list-tags
ctx db export --format jsonl
ctx db import <file>
ctx mcp
```

示例：

```bash
ctx search payment
ctx search "支付失败"
ctx tag maven
ctx show runbook.payment.failed
ctx copy command.order-service.build
ctx copy runbook.payment.failed --field full --print
ctx list-tags
ctx db export --format jsonl > records.jsonl
ctx db import records.jsonl
ctx mcp
```

---

## 8. MVP 验收标准

MVP 完成后，应满足以下验收条件：

1. 用户可以通过 CLI 快速新增一条记录。
2. 用户可以通过关键词找到已记录内容。
3. 用户可以通过标签缩小查询范围。
4. 用户可以通过 Key 精确找到一条记录。
5. 用户可以查看完整记录。
6. 用户可以快速复制正文、命令、URL 或整条记录。
7. AI 可以通过 MCP 只读查询同一份上下文。
8. MCP 不提供修改、删除和执行命令能力。
9. 新增记录时可以提示疑似敏感信息。
10. 无查询结果和查询失败能够明确区分。
11. 数据可以本地持久化保存。
12. 数据可以备份和迁移。

---

## 9. 后续阶段方向

MVP 验证通过后，再考虑：

1. 更完善的修改和删除体验
2. 更好的搜索排序
3. SQLite FTS 或其他全文检索方案
4. 命令模板管理
5. 服务上下文视图
6. 构建上下文视图
7. 信息过期提醒
8. 敏感信息打码
9. Web 管理界面
10. 与其他 MCP 工具协作

---

## 10. MVP 一句话定义

> Context Hub MVP 是一个本地优先的个人上下文管理工具，通过 CLI 低成本记录信息，并通过关键词、标签、Key 和 MCP 只读接口，让用户和 AI 快速找回可靠上下文。

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

## 核心能力

首期能力：

- 快速新增信息
- 关键词全文搜索
- 标签查询
- 唯一 Key 精确查询
- 查看完整记录
- 快速复制记录内容
- AI 只读查询上下文

后续能力：

- CLI 工具
- MCP Server
- 命令模板管理
- 构建上下文管理
- 服务上下文管理
- 更完善的结构化查询

## 推荐记录格式

每条记录建议包含标题、正文、标签、创建时间，也可以设置唯一 Key、服务、环境、来源和失效时间。

示例：

```yaml
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
created_at: 2026-06-21
expires_at:
content: 支付失败时先查询 payment_callback_log，再查询 payment-service 日志。
```

## 计划中的 CLI

预期命令：

```bash
ctx add
ctx search <keyword>
ctx tag <tag>
ctx show <key|index>
ctx copy <key|index>
ctx list-tags
```

示例：

```bash
ctx search payment
ctx search "支付失败"
ctx tag maven
ctx show runbook.payment.failed
ctx copy command.order-service.test
```

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

## 项目原则

```text
集中管理 > 完美分类
能搜到 > 结构漂亮
先可用 > 后智能
```

Context Hub 的第一目标不是复杂知识库，而是简单、快速、可持续地记录和找回信息。

## 文档

- [需求文档](docs/requirements.md)

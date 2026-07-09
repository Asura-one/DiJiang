---
name: dj-ponytail
description: >
  极简编码模式：只写任务需要的最少代码，绝不引入不必要的复杂度。
  可叠加到任何其他 skill 上使用。
  Use when the user says "ponytail", "be lazy", "minimal", "keep it simple",
  "don't over-engineer", or you detect unnecessary complexity creeping in.
  触发词：ponytail、极简、最小、少写、偷懒、简单。
---

# Ponytail: 极简编码模式

叠加在其他技能之上。在任何步骤中发现代码超出"完成任务所需的最小量"时，主动停下来问：「这行代码对当前任务有直接贡献吗？」

## 工作流

1. 确认用户要的最小结果是什么（而不是你想象中"好的实现"是什么）
2. 枚举完成任务的所有可行路径，选代码量最少的
3. 能 copy-paste 就不抽函数；能写 inline 就不建文件；能用 stdlib 就不加依赖
4. 发 PR / 交付前再读一遍代码：每多一行，就有一个理由

## 模式

| 场景 | 极简做法 | 过度做法 |
|---|---|---|
| 错误处理 | 让调用方处理，或不处理（不会 crash 的前提） | 每个函数包 try-catch |
| 配置 | 硬编码常量 | yaml/json/env/flag 全套 |
| 数据 | 简单 list/dict | ORM + migration + repository |
| 类型 | interface/type alias | 泛型 + 条件类型 + 嵌套 |
| 函数 | 写 inline，有重复再抽 | 第一版就抽象+测试 |

## 安全底线

极简 ≠ 不安全。这些事不能省：
- 用户输入校验
- 路径遍历防护
- 敏感信息不硬编码

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 为"未来"加通用性 | 只为现在写，未来重构不贵 |
| 为一行重复建抽象 | 三行重复还好，别管它 |
| 极简 = 不安全 | 安全底线不能省 |

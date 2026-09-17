# 任务分解：垂直切片（Tracer Bullet）

从 spec 到可执行的 ticket 清单。每个 ticket 是一个端到端的垂直切片。

## 核心原则

- 每个 ticket 是一次可以从用户界面验证的增量
- 第一个 ticket 必须是"Hello World"级的最小可用路径
- 后续 ticket 在已有路径上增加功能/覆盖边界
- blocking edge 显式标注：`B → A` 表示 B 依赖 A

## 分解步骤

### 1. 识别用户旅程

找出用户从开始到完成经历的主要步骤。每个步骤 = 一个潜在 ticket。

### 2. 排序为依赖链

```
UI 占位 → 核心逻辑 → 数据持久化 → 错误处理 → 边界情况 → 性能优化
```

### 3. 按 tracer bullet 切片

每个切片是一次垂直贯穿：

```
ticket-1: "用户能输入搜索词并看到结果列表"（全文检索 → 简单 UI）
ticket-2: "点结果能看详情页"（详情路由 → 内容获取 → 展示）
ticket-3: "搜索结果分页"（参数传递 → 翻页  → 状态保持）
```

### 4. 标注 blocking edges

```yaml
tickets:
  - id: ticket-1
    title: 基础搜索功能
    blocks: []               # 无前置依赖
  - id: ticket-2
    title: 搜索详情页
    blocks: [ticket-1]       # 依赖 ticket-1
  - id: ticket-3
    title: 搜索分页
    blocks: [ticket-1]       # 依赖 ticket-1，可与 ticket-2 并行
```

### 5. 每条 ticket 包含

- 标题：什么 + 为什么
- 验收条件：可验证的通过/失败标准
- 涉及文件：已知的文件列表
- 技术备注：可选的方向性提示
- blocking edges：前置依赖列表

---
name: dj-design
description: >
  前端 UI 设计：产出有观点的界面，不是千篇一律的默认样式。
  Use when the user asks for UI design, page layout, component styling, or visual polish.
  触发词：设计、做页面、做组件、不好看、很丑、前端、UI、design、page、component。
---

# Design: 做有观点的界面

产出有明确设计方向的 UI，不是"看起来像 AI 生成的默认样式"。

## 工作流

### 1. 构建设计简报

| 字段 | 要求 |
|---|---|
| Product surface | 页面/组件/flow |
| 目标用户 | 谁会用它 |
| 核心操作 | 用户在此界面完成什么 |
| 密度 | 工具型高密 / 内容型中密 / 展示型低密 |
| 现有系统 | design tokens、组件库、CSS 框架 |
| 视口 | desktop + mobile |
| Hard constraints | 品牌色、禁止元素、无障碍、数据量 |

### 2. 提交一个具体方向

```text
设计方向：<一句话>
布局：<导航/内容/操作层次>
视觉系统：<颜色、字号、间距、图标>
状态覆盖：<空态/加载/错误/成功>
```

不给三个方案让用户选——选一个，说明理由。

### 3. 从现有系统开始实现

1. 现有组件或 token
2. 现有 CSS 工具类或本地模式
3. 浏览器原生控件
4. 新样式（仅当无本地模式可用）

### 4. 验证

```text
Desktop: <pass/fail>
Mobile: <pass/fail>
Overflow: <pass/fail>
Keyboard: <pass/fail>
Contrast: <pass/fail>
States: <pass/fail or n/a>
```

## 设计偏好

暖灰底(#fafaf9) + 等宽数字，参考 Linear/Notion。禁止：玻璃态、渐变文字、过度装饰。

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 默认 AI 样式凑合 | commit 一个具体方向 |
| 给 3 个方案让用户选 | 选一个，说理由 |
| 不考虑移动端 | 至少检查桌面和手机 |
| 渐变背景+阴影堆叠 | 简洁、克制 |

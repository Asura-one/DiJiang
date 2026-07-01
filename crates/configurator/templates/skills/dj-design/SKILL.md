---
name: dj-design
description: >
  前端 UI 设计：产出有观点的界面，不是千篇一律的默认样式。
  Use when the user asks for UI design, page layout, component styling, or visual polish.
  触发词：设计、做页面、做组件、不好看、很丑、前端、UI、design、page、component。
---

# Design: 做有观点的界面

## 职责

产出有明确设计方向的 UI，不是"看起来像 AI 生成的默认样式"。

## 核心原则

**如果看起来像是默认 prompt 生成的，就不够好。**

## 输入 / 输出

| 项目 | 约定 |
|---|---|
| 输入 | User goal, target audience, existing UI/design system, viewport requirements, and implementation surface |
| 输出 | One committed design direction, implemented UI changes when requested, screenshots or review notes, and validation summary |
| 非目标 | Do not produce a marketing landing page unless requested; do not offer multiple vague options for the user to choose |

## 工作流

### 1. Build the Design Brief

Collect or infer exactly these fields:

| Field | Required answer |
|---|---|
| Product surface | 页面、组件或 flow 名称 |
| Primary user | 谁会重复使用它 |
| Job | 用户在此界面完成什么动作 |
| Density | 工具型高密 / 内容型中密 / 展示型低密 |
| Existing system | design tokens、组件库、图标库、CSS 框架 |
| Viewports | 至少 desktop + mobile；复杂工具加 tablet |
| Hard constraints | 禁止元素、品牌色、无障碍、数据量、状态数 |

If a field is missing but does not affect layout or behavior, choose the conservative default and state it. If it changes the product meaning, ask one question before implementation.

### 2. Commit to One Direction

Output one concrete direction:
```text
Design direction: <one sentence>
First principles: <user job + hard constraints + discarded assumptions>
Reference fit: <existing product/system or none>
Layout: <navigation/content/action hierarchy>
Visual system: <color, type scale, spacing, icon approach>
States: <empty/loading/error/success/disabled as applicable>
```

Do not present three decorative alternatives. Pick the direction that best fits the product surface and explain the tradeoff in one sentence.

第一性原理要求：先从用户任务、信息层级、输入输出和设备约束推导布局，再套视觉风格。不要从好看的截图、流行组件或默认模板反推产品界面。

### 3. Implement With Existing System First

Follow this order:

1. Existing component or token
2. Existing CSS utility or local style pattern
3. Browser-native control
4. New style only when no local pattern fits

For operational tools, prefer dense but readable layouts, predictable navigation, restrained color, and stable dimensions. Avoid hero-style composition, decorative cards, and ornamental gradients unless the product explicitly calls for them.

### 4. Validate the UI

Minimum checks:

```text
Desktop viewport: <pass/fail + issue>
Mobile viewport: <pass/fail + issue>
Overflow/text clipping: <pass/fail + issue>
Keyboard/focus path: <pass/fail + issue>
Color contrast: <pass/fail + issue>
Empty/loading/error states: <pass/fail + issue or n/a>
```

Use screenshots or browser inspection when a runnable frontend is available. If no frontend runtime is available, do code review against layout constraints and mark screenshot verification as `not run`.
## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 设计方向用户不满意 | 问清楚哪里不满意，调整方向 | 回退到上一个可用版本，重新确认方向 |
| 响应式布局在某 viewport 崩溃 | 检查 CSS 断点和 flex/grid 配置 | 降级为单列布局，保证内容可读 |
| 项目无设计系统/token | 从用户偏好推断（暖灰底 #fafaf9 + 等宽数字） | 使用系统默认 + 简洁风格 |
| 截图验证发现渲染异常 | 检查浏览器兼容性和 CSS 属性支持 | 标注不支持的浏览器，给出替代方案 |
| 无障碍检查不通过 | 修复对比度、添加 aria 属性 | 标注已知无障碍限制，记录到 debt |

## 🔴 CHECKPOINT · 设计方向确认

实现前必须确认：
```
设计方向：
- 风格：<具体风格>
- 参考：<参考产品/设计系统>
- 配色：<明亮/暗色/中性 + 主色调>

确认实现？(Y/n)
```

## 设计偏好

用户偏好：暖灰底(#fafaf9) + 等宽数字，参考 Linear/Notion 简洁现代感。

禁止：
- 玻璃态
- 渐变文字
- 过度装饰

## 输出格式

```html
<!-- 文件：<组件路径> -->
<!-- 设计方向：<一句话说明> -->
```

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 用默认样式凑合 | commit 到一个具体设计方向 |
| 给用户 3 个方案让他选 | 选一个，说理由 |
| 不考虑移动端 | 至少检查桌面和手机 |
| 用 emoji 做图标 | 用真正的图标系统 |
| 渐变背景 + 阴影堆叠 | 简洁、克制 |
| 布局崩了不检查就交付 | 截图验证每个 viewport |
| 无障碍不达标就上线 | 至少过颜色对比度检查 |

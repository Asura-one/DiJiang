---
name: dj-design
name_cn: 前端视觉设计
description: "前端 UI 设计：产出有观点的界面，不是千篇一律的默认样式。 触发词：设计、做页面、做组件、不好看、很丑、前端、UI、design、page、component、排版、截图改样式。"
description_cn: 产出有观点的界面设计：方向锁定、design token、明暗双主题、程序化验证（溢出/对比度/canvas 响应式），拒绝千篇一律的默认样式与 AI 模板味。
disable-model-invocation: true
---

## Outcome Contract

| 项目 | 内容 |
|---|---|
| **Outcome** | 有明确设计方向的 UI（让人认不出是 AI 做的） |
| **Done when** | 方向确定 → 现有系统实现 → 验证通过 → 无 AI Slop |
| **Evidence** | 设计简报、实现代码、验证检查表（6 项） |
| **Output** | UI 代码 + 设计方向说明 |

# Design: 做有观点的界面

如果可能是默认 prompt 生成的，就不够好。

## 1. 起点：从已有上下文长出来

起点按优先级取（现有 tokens/组件库 → 用户截图/参考站/品牌资产 → 对话品味原话）；品牌色**绝不用记忆猜**，无资产走品牌协议，搜不到就问（参考站抽三属性再命名 → `references/design-details.md`）

## 2. 品牌协议（涉及品牌/logo 强制）

涉及品牌强制走品牌协议（问 → 搜 → 取 → grep 色值 → 写 `brand-spec.md`+CSS 变量；资产 > 规范，五步详见 → `references/design-details.md`）

## 3. 提交一个方向（内部多候选，交付一个）

内部探索 2-3 个候选（不同布局/气质/字体），**交付一个主推方向 + 一句话理由**，不给三个方案盲选；大任务（品牌/首页/新视觉系统）做 2-3 个真实首屏预览摆出来让用户选（方向模板与三行句 → `references/design-details.md`）

## 4. 从现有系统开始实现

优先级：现有组件/token → CSS 工具类 → 原生控件 → 新样式（五维锁定与已有产品追加面细则 → `references/design-details.md`）

## 5. 诚实内容

不编造统计/价格/评价/logo 墙；占位符优于仿制品（禁 AI 营销词、文案纪律清单 → `references/design-details.md`）

## 6. 反 AI Slop 清单（自查非事后 lint）

**测试：** 陌生人看第一屏会说“AI 做的”？会就改掉再交。三清单与家族纪律 → `references/design-details.md`

## 7. 实现质量底线

七个维度实现底线与参数（排版拒 Reflex/OKLCH/命名间距标尺/动效 transform+opacity/CJK 回退/无障碍/双主题变量）→ `references/design-details.md`（双主题方法 `references/dual-theme-tokens.md`）

## 8. 验证

真实验证渲染，不靠类型检查/想象。交付前过：

```text
Desktop/Mobile(375px)/Overflow/Keyboard/Contrast/States: <pass/fail>
```

加验：生产级 UI 过设计体检 7 问（references）+ AI Slop；真图/真数据；实跑浏览器看过。

**程序化验证三件套**（目测截图不可靠）：溢出 `scrollWidth > clientWidth`、对比度（OKLCH→线性 sRGB、双主题各验一轮）、canvas 响应式（ResizeObserver）→ `references/verification-pitfalls.md`
详细坑与脚本见 `references/verification-pitfalls.md`，双主题方法见 `references/dual-theme-tokens.md`。

## 失败模式

| 触发 | 一线修复 | 兜底 |
|---|---|---|
| 方向不明确 | 内部多候选→一个推荐 | 大任务给 2-3 真实预览 |
| 对比度不达标 | 调到 WCAG AA 4.5:1 | 标注未达标项 |
| 用户说"很怪/不协调" | 当审美拒绝，重读论点 | 重走方向锁定 |

## 设计偏好

暖灰底(#fafaf9) + 等宽数字，参考 Linear/Notion。禁止：玻璃态、渐变文字、过度装饰；数字优先 tabular-nums。

## Hard Rules

1. 交付一个明确方向（大任务真实预览例外）；从现有系统开始，绝不猜品牌色
2. 涉及具体品牌：走品牌协议，资产 > 规范
3. 必须覆盖桌面 + 移动端（375px）视口
4. 禁止：玻璃态、渐变文字、过度装饰、AI 模板装饰（全大写标签/中点/箭头装饰）
5. 选字体三词→拒三→选；不用 Reflex 字体；不编造数据，无资产用占位
6. 真实验证渲染，不靠想象

## Gotchas

| 场景 | 后果 | 预防 |
|---|---|---|
| 用记忆猜品牌色 | 品牌识别断裂 | 搜官方资产写 brand-spec |
| 给 3 个方案盲选 | 决策转移 | 内部多候选交一个推荐 |
| 用 Reflex 字体 | 一眼 AI 味 | 三词→拒三→选 |
| 只做桌面端 | 手机上坏 | 桌面 + 375px 移动 |
| 忽略 CJK 排版 | 中文撕裂感 | 拉丁栈+CJK 回退+lang 标注 |
| grid 用裸 1fr | 375px 必横向溢出，截图看不出来 | 一律 minmax(0,1fr)，程序化溢出检查 |
| 双主题只验 light | dark 下暗底暗字（实测 1.24:1） | accent-foreground 语义翻转，两主题各验 |
| 硬编码色值 | dark 下对比度崩（实测 2.71:1） | 所有色走变量，写 light 时就抽 |

*细节参数表（字体拒绝表、OKLCH、动效时长、CJK、表面细节、无障碍、性能、响应式、设计体检 7 问、DESIGN.md 九节）见 `references/design-details.md`。*

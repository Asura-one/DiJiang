# 明暗双主题 token 方法

> 来源：2026-11-11 设计基调 session（A chrome + C canvas 拍板 + 双主题落地，明暗 ×2 页 ×2 视口全矩阵实测）。

## 核心形态：同命名空间两组变量

```css
:root { --background: oklch(0.985 0.002 80); /* light */ }
html.dark { --background: oklch(0.17 0.005 80); }
```

- Tailwind v4 `dark:` 前缀直接可用；shadcn 变量命名约定平移无成本
- **不用**两套独立类名体系（`.btn-light/.btn-dark`）——那是双倍维护

## 五条实测规则

1. **色温连续**：明暗两档用同一色相家族（如暖 80），避免切换跳温感（冷暗 265 ↔ 暖光的切换实测刺眼）。若参考冷石墨暗档（B 候选 0.17/0.21/0.26/0.29/0.64 的亮度结构很好），只换色相不换亮度档位。
2. **彩色角色按主题调档**：品牌/状态色 light 用深档（0.5–0.56）、dark 用亮档（0.68 左右）——同一 oklch 亮度 L 在明暗底上对比度表现完全不同，需各自校准。
3. **`*-foreground` 语义翻转是最大坑**：dark 主题下 `accent-foreground` 若沿用 light 的深色值，形成**暗底暗字**（实测 1.24:1）。规则：light 下 accent 配深字、dark 下 accent-bg 配亮字；accent 与 accent-foreground 各自独立翻转。
4. **所有色走变量，零硬编码**：light 下看着没问题的硬编码色（时间戳灰），dark 主题下原值对比度崩（实测 2.71:1）。写 light 时就把每个色值抽成变量——这是双主题成本最低的时刻。
5. **每主题独立验证**：只验 light 是常见偷懒路径，dark 的 fg/bg 组合是独立集合（实测 A 的 dark 首验 4 处 ✗ 全在 light 验过的元素上）。两主题 × 两视口（1280/375）× 溢出 + 对比度全跑。

## 色彩角色语义模式（多角色产品）

主操作 primary 明暗互为反色（暖黑/石白，无色相——按钮不需要彩色记忆）；领域状态色与交互签名色分开设变量（teal=运行/成功、陶土=会话签名），勿合并成一个 accent。语义色系（severity 四级等）明暗各配 `*-bg` 淡底变体（同色相低 C），tag/背景/边框三层用同一色相家族。

## 切换交互

顶栏 ◐ 按钮 `document.documentElement.classList.toggle('dark')`；正式实现加 `prefers-color-scheme` 初始态与用户选择持久化。切换后必跑一轮溢出 + 对比度复验（实测切换零布局破损，但配色对必须重算）。

## 冻结产物示例

`.dijiang/plans/go-v1/design-baseline.md` 的双主题 token 全表（20 变量 × 2 主题，全 OKLCH、明暗各校对比度）可作为工程侧 `@theme` 平移的直接输入。

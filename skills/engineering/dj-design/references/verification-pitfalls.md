# 设计验证陷阱（程序化）

> 来源：2026-11-11 前端设计基调 session 实测（dj-prototype spike + dj-design 三候选 → 双主题拍板）。四条全部踩过、修过、复验过。

## 1. grid blowout：`1fr` 轨道必炸 375px

- **现象**：`grid-template-columns: 1fr 340px` 的容器在 375px 视口下，`1fr` 轨道解析为子项 min-content 宽度（实测 432px > 容器 323px），`scrollWidth 458 > clientWidth 375` 横向溢出。**三个设计候选全中**，目测截图几乎看不出来（内容居中时溢出藏在两侧）。
- **根因**：CSS grid 的 `1fr` 轨道 auto 最小值 = `minmax(auto, 1fr)`，吃子项 min-content（长中文文本/等宽代码块把 min-content 撑大）。
- **修法**：一律 `minmax(0, 1fr)`。实测 `min-width: 0` 打在容器上不如 minmax 可靠（grid 轨道解析发生在容器 min-width 约束之前）。
- **兜底**：长文本单元加 `overflow-wrap: anywhere`；表格窄屏用卡片内 `overflow-x: auto` + `min-width` 保表头结构，而非压缩列。

## 2. 溢出必须程序化检查 + 修复后复验

- **命令**（浏览器 evaluate）：`document.documentElement.scrollWidth > document.documentElement.clientWidth`
- **为什么目测不可靠**：候选 C 首版截图人眼看不出问题，程序化一测 `scrollW 605`。溢出量小或藏在 flex/grid 边缘时截图完全正常。
- **定位元凶**（溢出时）：`[...document.querySelectorAll('*')].filter(e => e.getBoundingClientRect().right > viewportW + 1)` 逐层下钻，比猜 CSS 快一个数量级。
- **修复后必须复验**：改一处 CSS 可能牵连（实测修完 A 的 grid 后仍报 458，下钻才发现第二个溢出源 nav）。

## 3. OKLCH 对比度计算两连坑（两踩两修）

- **坑 ①**：Chrome `getComputedStyle().color` 对 oklch 变量返回 `oklch(0.5 0.01 90)` 字符串而非 rgb——**rgb 正则解析全部 parse fail**，需先做 OKLCH→线性 sRGB 转换（OKLab 矩阵：`l = L + 0.3963377774a + 0.2158037573b` 等，见 WCAG 对比度脚本）。
- **坑 ②**：转换函数输出**线性** sRGB；若再误套 gamma（`v^2.4`）把显示值当线性亮度算，会得出系统性偏低假比率（实测正文算出 5.79，实际 16.0；KPI 全线误报 ✗）。
- **正解**：WCAG 相对亮度用线性值；oklch 直接给线性，rgb 字符串才需要 `v <= 0.03928 ? v/12.92 : ((v+0.055)/1.055)^2.4`。
- **带 alpha 合成**：前景半透明时先与背景色 alpha 合成再算亮度。
- **结论纪律**：对比度数字异常（全线 ✗ 或全线夸张 ✓）先怀疑计算管线，别先怀疑配色。

## 4. canvas 图表不随容器收缩（G6 vs React Flow）

- **G6 v5**：构造时 `width` 固定，容器收缩后内容溢出容器边框（375px 实测两节点出框）——**必须** ResizeObserver → `graph.setSize()` + `graph.fitView()`，窄屏断点 fitView 保全图可见。
- **React Flow**：DOM 渲染，容器自适应无此问题。
- **通用**：canvas 类库（G6/ECharts/原生 canvas）默认不响应 CSS 尺寸，都要显式接 ResizeObserver；375px 列入验收点。

## 验证顺序（设计交付前）

1. 桌面 1280 + 移动 375 各截一张全页图
2. 两个视口跑程序化溢出检查（本文件 §2 命令）
3. 溢出 → 下钻定位 → 修 → **复验**
4. 关键色对跑对比度脚本（本文件 §3，注意明暗两主题都要跑）
5. Tab 一次看焦点环是否可见


## 程序化验证三件套（正文迁入）

**程序化验证三件套**（目测截图不可靠，溢出与对比度必须跑代码）：
- 溢出：`scrollWidth > clientWidth` 程序化检查，修复后必复验；定位元凶用 `getBoundingClientRect().right` 逐层下钻
- 对比度：OKLCH 计算色需 OKLCH→线性 sRGB 转换，WCAG 亮度用线性值（误用伽马校正值得出假比率）；**明暗双主题各验一轮，只验 light 是常见偷懒**
- canvas 响应式：G6 等 canvas 库不随容器收缩，ResizeObserver + setSize/fitView；React Flow（DOM）无此问题

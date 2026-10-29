# Design Details

细节清单，按需加载。SKILL.md 是入口，本文件承载长清单与参数。

## 技术栈清单（永不组合）

| 永不组合 | 原因 |
|---|---|
| Tailwind + CSS Modules 同一元素 | 优先级冲突，级联不可预测 |
| Framer Motion + CSS transitions 同一元素 | 同一属性双动画导致抖动 |
| styled-components/emotion + Tailwind | 两套类系统争抢同一 DOM 节点 |
| 多套图标库（Heroicons+Lucide+FA） | 视觉不一致、尺寸错位、包体积膨胀 |
| 多个 Google Font 家族做 display | 多个"性格"互相抵消 |
| Glassmorphism 半透明模糊 + 实线 1px 边框 | 实线边框摧毁深度错觉 |
| 深色背景 + 全白 #ffffff 文字 | 太刺眼；用 rgba(255,255,255,0.85) 或 #f0f0f0 |

开工前命名唯一 CSS 策略（Tailwind only / CSS Modules only / CSS-in-JS only），不漂移。

## Reflex 字体拒绝表

LLM 默认会滑向这些字体（训练数据主导），用了就是"没做决定"。禁令针对反射性 display 使用；有意的产品 UI 用（如 Inter 用于密集数据表）可豁免并说明理由。

拒绝：Inter、DM Sans、DM Serif Display、DM Serif Text、Outfit、Plus Jakarta Sans、Instrument Sans、Instrument Serif、Space Grotesk、Space Mono、IBM Plex Sans/Serif/Mono、Syne、Fraunces、Newsreader、Lora、Crimson Pro、Crimson Text、Playfair Display、Cormorant、Cormorant Garamond。

选择程序：
1. 写三个词描述品牌气质（如"精确、极简、快"）
2. 写下会反射性想到的三个字体
3. 全部拒绝
4. 从有性格的 foundry（Klim、Commercial Type、Colophon、Grilli Type、OH no Type、Village 等）或开源可选里挑一个匹配品牌词的字形，能一句话说清为什么

## 颜色：OKLCH 规则

- 用 OKLCH 而非 HSL：感知均匀，相同数值变化 = 相同感知变化
- 亮度接近极端时收紧 chroma：85% 亮度时 chroma 约 0.08 已够（0.15 艳俗）；15% 亮度同样收紧
- 中性色向品牌色相偏移：chroma 0.005-0.01，微弱但感知得到，产生潜意识连贯
- 60-30-10 说的是视觉重量：60% 中性/表面、30% 次级文字和边框、10% 强调
- 禁止彩色背景上灰字：用背景色相的同色阶低亮度代替

## 主题矩阵

| 语境 | 方向 | 理由 |
|---|---|---|
| 交易/分析仪表盘、夜班 | 深色 | 数据密度高；长会话低眩光 |
| 儿童阅读/学习 | 浅色 | 亲和、低疲劳（视力发育期） |
| SRE/可观测工具 | 深色 | 运维语境；低光 NOC 房间一目了然 |
| 周末规划/食谱/日记 | 浅色 | 日间使用；轻快 |
| 音乐/媒体浏览 | 深色 | 内容优先；媒体突出 |
| 医院/临床门户 | 浅色 | 信任与可读性优先 |
| 复古/手工艺品牌 | 暖米白 | 深色与材料气质冲突 |

语境不明默认浅色；双模式语境先浅后深，深色 token 叠加。

## 动效时长表

| 元素 | 时长 | 备注 |
|---|---|---|
| 按压反馈 | 100-160ms | 立即选中/打开/加载状态已算；只在结果有间隙时才加按压反馈 |
| 悬浮/列表导航/disclosure | 最短 | 每天几十次 |
| 输入框/小弹窗 | 120-200ms | |
| 下拉/选择器 | 150-250ms | |
| Modal/Drawer | 200-500ms | 交互元素超 300ms 需要理由 |
| 罕见/一次性（onboarding/完成） | 可带愉悦感 | |

规则：频率决定是否动，先定动不动再定怎么动。每天触发数百次（快捷键、命令面板）无动画，重读起来是卡顿。绝不动画键盘发起的状态切换。尊重 prefers-reduced-motion。

技术：只动 transform/opacity（合成器友好，零布局抖动）；进入禁止 scale(0)，用 scale(0.95)+opacity:0；弹层从触发点缩放（transform-origin 锚定），modal 例外居中；用 transition 而非常量动画 keyframes（可中断重定向）；悬停/按压用 ease-out-quart/quint 或 cubic-bezier(0.16,1,0.3,1)，禁止默认弹跳；手指驱动的拖拽可小弹跳（bounce 0.1-0.3 或 spring dampingFraction 0.6-0.8）；交错进入：语义块 ~100ms、标题分词 ~80ms（opacity 0→1、translateY(12px)→0、blur(4px)→0）；退出用固定 translateY(-12px) ~150ms ease-in，这是唯一允许 ease-in 的地方；图标态切换 scale 0.25→1 + opacity 0→1 + blur 4px→0，无旋转除非语义（chevron）。

## CJK 与多语言排版

- 拉丁字体优先 + 系统 CJK 后置：`font-family: -apple-system, "SF Pro Text", "PingFang SC", "Noto Sans SC", sans-serif`
- CJK 正文 line-height 1.7-1.8（拉丁 1.4-1.5 不够，密集汉字需要更多垂直空间）
- 语言段加 lang="zh"/"ja"/"en"，浏览器才能选对字体和断行
- 衬线阅读模式必须显式 CJK 衬线回退：`"Newsreader", "Songti SC", "Noto Serif SC", serif`；否则拉丁衬线 webfont 无 CJK 字形时中文静默掉回黑体
- 负字距（tracking）不用于 CJK run：大标题 -0.022em 规则仅拉丁；汉字收紧变成渲染 bug

## 表面与布局细节

- 命名圆角标尺：方向锁定时定义 3-4 级（4/8/12/pill 或 6-8 级），全表面同一空间语言，不随手取值
- 同心圆角：outerRadius = innerRadius + padding，嵌套圆角才像有意的；padding 超 24px 视为独立表面各自取值
- 阴影优先于边框：卡片/按钮用分层 box-shadow 提亮，不用边框围栏；边框留给分隔线/单元格/布局分隔（浅色主用；深色见下）
- 深色层级：画布近黑实色（#08090a）；高程靠叠加半透明白：卡片 rgba(255,255,255,0.02)、浮层 0.04、突出面板 0.05；边框同逻辑：0.05 细分 0.08 标准；传统 drop shadow（黑上黑）几乎不可见
- 浅色层级：相邻嵌套表面必须可分辨。最低：侧栏与主区、主区与卡片之间的 background-color 亮度差 ≥4%，或浮卡 shadow ≥0 1px 3px rgba(0,0,0,0.10)。白卡白底 + 0 1px 2px 阴影 = 噪声不是深度
- 图片描边：inset 细描边 `outline: 1px solid rgba(0,0,0,0.1); outline-offset: -1px`（深色 0.1 白），不占布局
- 最小热区 40×40px；可见元素更小用居中伪元素扩展；两个交互元素热区不重叠
- 多卡片对齐：同行卡片等高，矮卡片多一行事实而非多 padding；CTA 底对齐；定价/对比卡特性清单共享 Y 基线；side-by-side 面板（标题/描述/价格/按钮）跨列共享基线；section 顶底 padding 不必对称，光学平衡常需底 > 顶 20-25%；正文行宽 ~65ch
- 间距是系统：整套间距跑一条响应式阶梯；页面太空/太紧就整组按单一系数缩放，不单点调
- 固定高度动作槽（状态栏/动作栏/菜单项）：所有状态同一字号，变化靠填充/描边/透明度/图标，绝不用字号差（1pt 差在切换时抖动）
- 移动端 CTA 静止态：自然宽度、左对齐正文边缘、高度不变；居中读作悬浮，flex:1 读作笨重

## 无障碍基线

- 图标按钮 aria-label；操作用 `<button>`、导航用 `<a>`（不用 div onClick）
- 图片加 alt（装饰图 alt=""）
- 可见焦点：focus-visible:ring-* 或等价；永远不只 `outline: none`
- 正文对比度 ≥4.5:1（大字号 ≥3:1）；触控 target ≥40px
- 表单内联错误与字段相邻，不只在表单顶部

## 性能

- will-change 仅 transform/opacity/filter；绝不用 all；首帧卡顿才加，不预设
- 图片显式 width/height（防 CLS）；折叠下方 loading="lazy"
- 关键字体 font-display: swap
- touch-action: manipulation（防双击缩放延迟）
- 全出血布局 env(safe-area-inset-*)
- Modal/Drawer overscroll-behavior: contain
- hover 态包在 @media(hover:hover) 里，防触摸端永久悬浮

## 响应式验证

- 真实渲染验证，不是类型检查/布局推算：手机 375px（按钮补 320px）+ 桌面 1280px，每个交付语言，长词/本地化字符串放进按钮/标签/导航/紧凑卡片
- 检查行尾孤儿词（含嵌套内联代码）：先查容器宽度和强制断行，不靠缩字号；收紧重复不损意义
- 至少一个窄视口 + 紧凑态

## 参考源优先级

- 源码 + 截图都有：读源码。源码有精确 token 值，截图要猜。从"写下来的"重构，不从"看得见的"猜
- 只有 URL：fetch 返回剥离布局的纯文本。视觉参考（"做成像 X"）要截图，别从剥离 HTML 推断

## DESIGN.md 九节（生产级 UI）

1. 视觉主题与氛围（2-3 句：情绪/密度/设计哲学）
2. 色板与角色（语义名 + 值 + 功能角色）
3. 字体规则（family/字号阶梯/字重/行高/字距；超 4 级用表）
4. 组件样式（按钮全态/卡片/输入/导航，含 default/hover/active/disabled）
5. 布局原则（间距阶梯/栅格列/留白哲学）
6. 深度与层级（阴影系统或背景色阶系统，每级描述）
7. 要做/不要做（5-10 条本项目护栏，不是通用规则）
8. 响应式（断点、导航如何折叠、触摸目标最小尺寸）
9. Agent Prompt Guide（颜色 name:value 对照 + 3-5 条可直接粘贴的组件 prompt，值全内联，无需再查）

## 设计体检 7 问（交付前）

1. 首屏是否品牌/产品一看就认？
2. 有没有一个强视觉锚点（真实图像，不是装饰渐变）？
3. 只扫标题能否理解页面？
4. 每个 section 是否只有一个职责？
5. 卡片是真的需要，还是默认样式？
6. 动效是在改进层级/氛围，还是装饰？
7. 去掉所有装饰性阴影后还高级吗？

## 战略遗漏（audit 提示词，非自动实现范围）

- 自定义 404：任务含路由/生产加固时，品牌页要有回程（首页/搜索/常用导航）
- 返回导航：每个用户可达页面有明确返回路径；死胡同页是 UX 失败
- 表单客户端校验：email 格式、必填内联报错、错误信息贴字段
- Skip-to-content 链接：第一个可聚焦元素
- Cookie 同意 / 隐私与条款：若产品实际埋点与司法管辖区要求，标记缺失并限定范围

这些是产品就绪问题，视觉任务可能暴露它们，但未授权不构建。

## 灵感出处

- Reflex Fonts、Font Selection、OKLCH、Theme Matrix、Absolute Bans、motion 规则、AI Slop Test 改编自 pbakaus/impeccable（Apache 2.0）
- 反 AI slop 清单与模板装饰共识来自 huashu-design（MIT）
- DESIGN.md Scaffold 概念源自 getdesign.md（MIT）
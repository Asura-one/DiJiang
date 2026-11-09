# verify-template: dj-verify 的生成模板与判据

从 SKILL.md 外溢的细节（参照 dj-loop/references/loop-spec.md 模式）。生成 verify-<app> 时逐节对照。

## § Interview：五问定义与判据

从代码库回答（package scripts/Makefile/README 是线索不是结论），只问代码观察不到的：

- **Surface**：用户实际碰什么（web/CLI/API/桌面/库）；可有多面，选主面并记其余
- **Run**：本地怎么启动——命令/端口/env/种子数据/认证；launch 必须能从空数据目录到可用态（含迁移与种子）
- **Drive**：agent 怎么程序化驱动。优先已有 harness（Playwright/Cypress spec、expect 脚本、curl 端点、调试端口）；没有再选通用配方：web/Electron 走 browser/CDP，CLI/TUI 走 PTY/tmux，服务走纯 HTTP
- **Observe**：能抓什么证据（截图/终端 transcript/响应体/日志/退出码/DB 态）
- **Isolate**：两实例能并行吗（端口/数据目录/profiles）？不能就在技能里写明拒双开——宁可拒绝驱动共享实例，别污染用户会话

## § 五段：生成物验收判据

1. **Launch**：确切启动命令 + 就绪判据（日志行/提示符，且达到可用态而非仅进程应答）；含 teardown
2. **Doctor**：一个只读检查回答『这实例值得驱动吗』——进程起、版本对、端口属我、认证有效。任何时候看着不对先跑它
3. **Drive**：harness 配方 + 本 repo 真实选择器/命令（ARIA 标签/data 属性/提示语/路由路径），非通用示例；稳定句柄优先于坐标和 tab 序
4. **Evidence**：为证明要抓什么、存哪。标准：走真实用户路径而非内部 setter/测试端点；动作和结果态都抓；副作用（写文件/插行/发消息）与可见面一起验证；mock 只用在生产边界已隔离外部系统处；dry-run/test 模式要观察它实际跳过了什么（文件/网络/git refs）而非信名字
5. **Cleanup**：拆掉本运行创建的实例与临时态；按句柄杀不按进程名杀；证据留在技能点名的固定位置，永不被 cleanup 清理

## § feature map：文件格式

`features/README.md` 索引 + 每个用户功能一文件（从路由/命令/菜单/文档识别，先 top 3–5）。每文件四个 H2：

- **Sub-features**：子功能拆分
- **How to get to it (user POV)**：用户怎么到达
- **Driving it with <harness>**：用该 harness 怎么驱动
- **Gotchas**：坑。种子数据缺前置条件的，把失败原因写在这，绝不弱化旅程让它通过

## § 先跑通再交付：流程

launch → doctor → drive 一个已映射功能 → 存证据 → cleanup → 确认证据还在（cleanup 吃掉证据 = 生成的 Cleanup 段失败）。每轮失败迭代后都跑 cleanup，防进程/端口滞留。用替身环境（容器替真机/本地栈替远端）验证过的，列出替身做了而目标环境不会做的每一步——这些步骤在真目标上未验证。

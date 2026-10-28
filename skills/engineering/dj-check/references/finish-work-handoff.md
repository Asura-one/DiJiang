# finish-work 数据准备

> 从 dj-check SKILL.md 外溢（DJ-013）。check 第 4 步为 finish-work 准备的全部数据面。

- `git status --short --branch`
- `git diff --stat HEAD`
- 验证报告摘要
- 版本决策建议
- issue 结题评论素材：验收逐条对照 + commit 引用 + 边界如实声明（工单有 Forgejo 镜像时，finish-work 关闭 issue 直接复用本报告，不再另写）

## 三重证据核实

关闭任何 issue 前先核实三样，不信推测：「看起来完成了」不等于完成——

1. 跟踪器状态（Forgejo issue 实际状态）
2. 本地提交记录（commit hash + diff 范围）
3. 代码实证（grep/读文件确认变更在库）

两侧状态一致才收口。

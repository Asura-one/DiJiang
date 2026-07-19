---
name: dj-ask
description: 不知道用哪个 dj-* skill 时的入口。告诉我你想做什么，我推荐对口的 skill。
disable-model-invocation: true
---

不知道从哪个 `dj-*` 开始？告诉我你想做什么，我帮你推荐。

读用户的话，判断意图，然后推荐最对口的 skill。路由逻辑和完整映射表参考 `skill_view(name='dj-dispatch')`。

支持串联推荐（如"需求不明确"→ dj-grill + dj-output + dj-split）。

用户说"够了"、"知道了"或"开始做"时停止推荐，交给对应 skill。

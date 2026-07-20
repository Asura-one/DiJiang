# 证据层级

风险决定证据深度：

| 结论 | 最低证据 |
|---|---|
| "文档链接有效" | 项目自己的 doc-link check 或逐链接存在性 |
| "规则已同源" | realpath/readlink + 平台实际加载顺序 |
| "代码实现是 X" | 当前目标分支代码、schema、配置与相关测试 |
| "PR 已完成" | PR state=merged + merge commit（不等同已部署） |
| "已部署" | deploy marker/release 指向目标 commit + 服务 active |
| "用户已看到新版本" | canonical URL/API 的真实响应 |
| "可安全清场" | merged + production contains change + knowledge receipt + lane clean |
| "整个项目干净" | 所有适用事实面 verified；warning/pending 单列 |

代码直觉、旧 memory、commit message 都只能当线索，不能单独证明终态。

## 发布状态机

```
implemented → locally verified → pushed/PR → CI passed
  → merged → deployed → live verified → knowledge closed
  → full result reported → user approved cleanup → workspace cleaned
  → post-cleanup audit passed
```

跳过的状态必须有项目规则允许的原因。

## 缓存与多表面

当内容经过 CDN、边缘缓存、搜索索引或多客户端时，至少识别 origin 是否为新、canonical URL 是否仍为旧缓存。只验证一个表面时在结论里限制范围。

## 验证失败时

- 同一失败第二次出现，停止盲重试，重新检查假设、环境和命令
- 失败发生在生产写入前，说"尚未影响生产"；在切流后，先确认当前 release 和回滚边界
- 未验证项保持 `pending`

# 发版前清理与文档规范化清单

> 摘要：发版/上线场景的可勾选清理清单——测试残留、文档规范、ADR 完整性、已知缺陷确认；dj-gov 发布收尾时逐项打勾。

发布收尾（merged / deployed / live verified / knowledge closed 四态）中，本清单管「发版前该清没清」：

## 测试与凭据残留
- [ ] 测试脚本无硬编码凭据（grep password/token/key）
- [ ] 无临时测试文件/脚本残留（.temp 之外的 test-*.py、tmp_*.sh）
- [ ] 冗余锁文件/缓存不入包（package-lock 副本、.pyc 目录）

## 文档规范化
- [ ] docs 六目录（references/adr/guide/architecture/design/testing）各自内容现役，无空壳
- [ ] ADR 编号连续无空洞；已 superseded 的有跳转指向
- [ ] CHANGELOG 按六类分组（新增/变更/修复/移除/弃用/安全），无 git log 流水
- [ ] 文档标点规范（全角/半角一致，无混用残留）
- [ ] API 端点文档覆盖度：新端点有文档（对照 `docs/references/api-doc-template.md`）

## 已知缺陷确认
- [ ] 已知缺陷清单明确列出（不是藏着）——发版说明如实披露
- [ ] 每个已知缺陷有：影响面 / 绕行方式 / 修复排期
- [ ] test-plan 里的过期命令已刷新（环境变更后旧命令跑不动 = 清单失真）

## 发布状态四态
- [ ] merged：分支已并入
- [ ] deployed：产物已部署
- [ ] live verified：线上实测通过（不是"部署完就觉得好"）
- [ ] knowledge closed：文档/CHANGELOG/ADR 已同步

任何一项打不了勾 = 发版阻塞项，如实报告不装看不见。

来源：琅嬛「项目发版前清理与文档规范化清单」

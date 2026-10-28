---
AIGC:
  ContentProducer: '001191110102MAD55U9H0F10002'
  ContentPropagator: '001191110102MAD55U9H0F10002'
  Label: '1'
  ProduceID: '0e22a5b9-46cb-4c78-a19f-d4631cf3ebc5'
  PropagateID: '0e22a5b9-46cb-4c78-a19f-d4631cf3ebc5'
  ReservedCode1: 'b8f8d4f2-83a1-4b96-b975-f4c95f2c7d17'
  ReservedCode2: 'b8f8d4f2-83a1-4b96-b975-f4c95f2c7d17'
---

# ADR 007: 文字点火不改拓扑（尾部指针的接力形态）

## 状态

Accepted（2026-10-28，fusion-chain Q3 决策）

## 上下文

fusion-chain 任务要解决"技能执行完无下一步推荐，用户被迫高频回调 dj-ask"。参考项目（mattpocock/skills）的衔接模式是路由图 + 技能尾部指针双载体。DiJiang 引入尾部指针时遇到拓扑约束：CONTEXT.md 规定 **user-invoked 技能互不调用**（user-invoked 负责编排入口，控制权在用户）。

主流程链条 grill→output→split→implement→check→finish-work 大半是 user-invoked 技能。若指针直接「调用」下游，等于 user 边互调，拓扑被打破。

## 选择

**文字点火，不动拓扑**：技能结束时说出建议的下一步（含理由），由用户以一句确认或点名启动。指针措辞分两型——指向 model-invoked 写「完成后调用 X」（agent 可以真调，如 implement→check）；指向 user-invoked 写「建议下一步调用 X（理由）」（如 check→finish-work，agent 点火、用户放行）。

## 放弃的方案

- **拓扑放宽**（主流程 user-invoked 改 model-invoked 实现自动接力）：一次点火全程自动，但 6 个编排技能失去「用户点火」保护——agent 可能自行启动收尾/拆分，节奏失控；CONTEXT.md/AGENTS.md 拓扑表全改；与「控制权在人」设计初衷冲突。
- **折中例外**（只放宽确定性边如 check→finish-work）：例外条款让「user 不互调」不再是干净不变量，每条例外都是下次放宽的先例。参考项目实际形态支持本选择：其 user 边（to-spec→to-tickets→implement）同样靠用户连续点名，从未互调。

## 影响

- 主流程接力多一次点火动作（一句确认的成本），换来编排类技能的节奏控制权始终在用户。
- verify_skills.py 的 check_tail_pointers 按「两型措辞」校验：user 边指针缺「建议」句式即 FAIL——措辞契约机器化，拓扑不动。
- dj-ask 保持战略入口地位不变；指针体系是「每步少问一次路」，不是「不再问路」。

关联：DJ-015（CONTEXT.md 术语预埋）、DJ-016（守卫落地）、DJ-017（本 ADR + grill_mode 配置）。

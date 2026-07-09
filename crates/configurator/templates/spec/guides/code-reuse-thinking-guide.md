# 代码复用思考指南

> **目的**：创建新代码前停下来想一想——它是否已经存在？

---

## 问题

**重复代码是导致不一致性 bug 的头号原因。**

当你复制粘贴或重写已有逻辑时：
- Bug 修复无法传播
- 行为随时间产生分歧
- 代码库更难理解

---

## 编写新代码前

### 第一步：先搜索

```bash
# 搜索相似的函数名
grep -r "functionName" .

# 搜索相似的逻辑
grep -r "关键词" .
```

### 第二步：问这些问题

| 问题 | 如果是... |
|----------|-----------|
| 是否有相似的函数存在？ | 使用或扩展它 |
| 这个模式在其他地方使用吗？ | 遵循现有模式 |
| 这能作为共享工具函数吗？ | 在正确的位置创建它 |
| 我在从另一个文件复制代码吗？ | **停下**——提取到共享位置 |

---

## 常见的重复模式

### 模式 1：复制粘贴函数

**糟糕的**：将校验函数复制到另一个文件

**好的**：提取到共享工具函数，在需要的地方导入

### 模式 2：相似的组件

**糟糕的**：创建一个与现有组件 80% 相似的新组件

**好的**：通过 props/变体扩展现有组件

### 模式 3：重复的常量

**糟糕的**：在多个文件中定义相同的常量

**好的**：单一事实来源，到处导入

### 模式 4：重复的载荷字段提取

**糟糕的**：多个消费者对相同的 JSON/事件字段进行本地转换：

```typescript
const description = (ev as { description?: string }).description;
const context = (ev as { context?: ContextEntry[] }).context;
```

即使代码只有两行，这也是重复的契约逻辑。每个消费者现在都有自己对有效载荷含义的定义。

**好的**：将解码器、类型守卫或投影放在数据所有者的旁边：

```typescript
if (isThreadEvent(ev)) {
  renderThreadEvent(ev);
}
```

**规则**：如果同一个无类型载荷字段在 2 个以上的地方被读取，在添加第三个读取者之前创建一个共享的类型守卫/规范化器/投影。

---

## 何时抽象

**应该抽象**：
- 相同代码出现 3 次以上
- 逻辑足够复杂，可能产生 bug
- 多个人可能需要它

**不要抽象**：
- 只使用一次
- 琐碎的一行代码
- 抽象比重复更复杂

---

## 批量修改后

当你对多个文件做了相似修改：

1. **审查**：所有实例都覆盖了吗？
2. **搜索**：运行 grep 查找遗漏的
3. **考虑**：这应该被抽象吗？

### 归约器应使用穷举结构

当状态从类似 action 的值（`action`、`kind`、`status`、`phase`）派生时，优先使用一个 `switch` 归约器而非分散的 `if/else` 更新。

```typescript
// 糟糕的 - action 特定的状态转换难以审计
if (action === "opened") { ... }
else if (action === "comment") { ... }
else if (action === "status") { ... }

// 好的 - 一个归约器拥有转换表
switch (event.action) {
  case "opened":
    ...
    return;
  case "comment":
    ...
    return;
}
```

当事件日志是事实来源时，这一点很重要。归约器是文档化的重放模型；展示代码和命令不应重复该重放模型的片段。

---

## 提交前清单

- [ ] 搜索了现有的相似代码
- [ ] 没有应被共享的复制粘贴逻辑
- [ ] 在共享解码器之外没有重复的无类型载荷字段提取
- [ ] 常量定义在一个地方
- [ ] 相似的模式遵循相同的结构
- [ ] 归约器/action 转换存在于一个归约器或命令调度器中

---

## 陷阱：Python 的 if/elif/else 穷举检查

**问题**：Python 的 if/elif/else 链没有编译时穷举检查。当你向 `Literal` 类型（如 `Platform`）添加新值时，现有的 if/elif/else 链会静默落到 `else` 并返回错误的默认值。

**症状**：新平台部分工作——某些方法返回 Claude 默认值而非平台特定值。不会抛出错误。

**示例**（`cli_adapter.py`）：
```python
# 糟糕的: "gemini" 落到 else，返回 "claude"
@property
def cli_name(self) -> str:
    if self.platform == "opencode":
        return "opencode"
    else:
        return "claude"  # gemini 静默得到 "claude"！

# 好的: 为每个平台提供显式分支
@property
def cli_name(self) -> str:
    if self.platform == "opencode":
        return "opencode"
    elif self.platform == "gemini":
        return "gemini"
    else:
        return "claude"
```

**预防**：当向 Python `Literal` 类型添加新值时，搜索该类型上的 ALL if/elif/else 链，并为新值添加显式分支。不要依赖 `else` 对新值也是正确的。

---

## 陷阱：产生相同输出的非对称机制

**问题**：当两个不同的机制必须产生相同的文件集时（例如 init 的递归目录复制 vs update 的手动 `files.set()`），结构变更（重命名、移动、添加子目录）只能通过自动机制传播。手动机制会静默漂移。

**症状**：Init 完美工作，但 update 在错误的路径创建文件或完全遗漏文件。

**预防**：
- **最佳**：消除非对称性——让手动路径调用自动路径（例如 `collectTemplateFiles()` 调用 `getAllScripts()` 而非维护自己的列表）
- **如果非对称性不可避免**：添加一个比较两个机制输出的回归测试
- 迁移目录结构时，搜索所有引用旧结构的代码路径

**真实案例**：`trellis update` 有一个手动的 `files.set()` 列表，包含 11 个脚本，而 `getAllScripts()` 已经在追踪它们。修复方法：用手动列表替换为 `for..of getAllScripts()` 循环。参见 v0.4.0-beta.3 中的 `update.ts` 重构。

---

## 模板文件注册（Trellis 特定）

当向 `src/templates/trellis/scripts/` 添加新文件时：

**单一注册点**：`src/templates/trellis/index.ts`

1. 添加 `export const xxxScript = readTemplate("scripts/path/file.py");`
2. 添加到 `getAllScripts()` Map

就是这样。`commands/update.ts` 直接使用 `getAllScripts()`——无需手动同步。

**为什么这很重要**：没有在 `getAllScripts()` 中注册，`trellis update` 不会将文件同步到用户项目。Bug 修复和功能将无法传播。

**历史**：在 v0.4.0-beta.3 之前，`update.ts` 有自己的手动维护文件列表，经常与 `getAllScripts()` 不同步。这导致 11 个 Python 文件在 `trellis update` 期间被静默跳过。修复方法：消除重复列表，使用 `getAllScripts()` 作为单一事实来源。

### 新脚本快速检查清单

```bash
# 添加新的 .py 文件后，验证它是否在 getAllScripts() 中：
grep -l "newFileName" src/templates/trellis/index.ts  # 应该有匹配
```

### 模板同步约定

`.trellis/scripts/`（dogfood 版本）和 `packages/cli/src/templates/trellis/scripts/`（模板版本）必须保持一致。编辑 `.trellis/scripts/` 后，始终同步：

```bash
rsync -av --delete --exclude='__pycache__' .trellis/scripts/ packages/cli/src/templates/trellis/scripts/
```

**陷阱**：使用错误的源/目标路径运行 rsync 可能创建嵌套的垃圾目录（例如 `.trellis/scripts/packages/cli/...`）。运行前务必仔细检查路径。

---
name: dj-tdd
description: >
  测试驱动开发：红绿重构循环，一次一个垂直切片。
  Use when the user wants to build features or fix bugs test-first, mentions
  "red-green-refactor", or wants integration tests.
  触发词：TDD、测试驱动、红绿重构、test-first、先写测试。
---

# TDD: 红绿重构

## 职责

通过测试驱动实现，确保代码行为可验证。一次一个垂直切片，不批量写测试。

## 核心原则

- **测试行为，不测实现**：通过公共接口验证"系统做什么"，不关心"怎么做"
- **垂直切片**：一个测试 → 一个实现 → 重复。不横切（不先写所有测试再写所有代码）
- **测试即规格**：好的测试读起来像规格说明——"用户可以用有效购物车结账"
- **Simplicity First**（karpathy）：测试代码也要简洁。如果一个测试超过 20 行，拆分成多个测试
- **Define Verifiable Success**（karpathy）：每个测试必须有明确的断言，不能只检查"不报错"

## 工作流

### 循环（每个切片）

```
RED  → 写一个失败测试（验证期望行为）
GREEN → 写最少代码让测试通过
REFACTOR → 清理代码，测试仍然通过
```

### 1. RED：写失败测试

- 从用户故事/需求中提取一个行为
- 写测试描述这个行为
- 运行，确认失败（红色）
- 如果不失败 → 测试有问题，修测试

### 2. GREEN：让测试通过

- 写最少的代码让测试通过
- 不考虑设计、不考虑优雅
- 运行，确认通过（绿色）

### 3. REFACTOR：清理

- 消除重复
- 改善命名
- 提取函数/模块
- 运行全量测试，确认仍然通过

## 好测试 vs 坏测试

### ✅ 好测试

```python
# 测试行为：用户可以用有效购物车结账
def test_checkout_with_valid_cart():
    cart = Cart()
    cart.add(Product("苹果", 3.0), quantity=2)
    result = cart.checkout()
    assert result.total == 6.0
    assert result.status == "success"
```

特点：
- 通过公共接口测试（cart.checkout()）
- 描述行为（"用户可以用有效购物车结账"）
- 重构后仍然通过（不关心内部实现）
- 读起来像规格说明

### ❌ 坏测试

```python
# 测试实现：内部计算器被调用了
def test_checkout_calls_calculator():
    cart = Cart()
    cart._calculator = Mock()  # mock 内部实现
    cart.checkout()
    cart._calculator.calculate.assert_called_once()  # 关心内部细节
```

问题：
- mock 内部协作者（_calculator）
- 测私有方法（_calculator）
- 重构就挂（说明测的是实现不是行为）

## 失败处理

| 触发条件 | 一线修复 | 仍失败兜底 |
|---------|---------|-----------|
| 写了测试但不失败（绿色） | 检查测试是否测到了正确行为 | 换一个更精确的断言 |
| GREEN 阶段怎么都过不了 | 缩小实现范围，只让核心断言通过 | 拆成更小的切片，每个切片一个行为 |
| REFACTOR 后测试挂了 | `git stash` 回到 REFACTOR 前 | 跳过这次重构，保持 GREEN 状态 |
| 测试框架配置问题 | 检查 jest/vitest/pytest 配置 | 用最简单的测试方式（如 `assert`） |
| 不确定该测什么行为 | 回到 PRD/需求，提取用户故事 | 从 happy path 开始，逐步加边界情况 |

## 🔴 CHECKPOINT · 切片确认

每个切片完成后：
```
切片：<行为描述>
RED: ✅ 测试失败（确认）
GREEN: ✅ 测试通过
REFACTOR: ✅ 全量测试通过

继续下一个切片？(Y/n)
```

- 用户说"Y" → 进入下一个切片的 RED
- 用户说"n" → 停下来讨论，不强行继续
- 所有切片完成 → 进入 REFACTOR 全量清理，然后告知用户实现完成

## 🔴 CHECKPOINT · 全量确认

所有切片完成后：
```
切片完成：<N> 个
全量测试：<通过/失败>

确认实现完成？(Y/n)
```

## 反例

| ❌ 不要做 | ✅ 正确做法 |
|---|---|
| 先写所有测试再写所有代码 | 一个测试一个实现，垂直切片 |
| mock 内部实现细节 | 通过公共接口测试 |
| 测试通过就不管重构了 | RED-GREEN-REFACTOR 三步都要走 |
| 测试描述写"test function X" | 描述行为："用户可以用有效数据提交" |
| 一个测试验证多个行为 | 一个测试一个行为 |
| 跳过 RED 直接写实现 | 必须先看到测试失败 |

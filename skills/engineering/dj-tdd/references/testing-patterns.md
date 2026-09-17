# 测试组织模式

## 按行为组织

```
tests/
├── test_placeholder.py   # RED: 空的，期望失败
├── test_feature.py        # 核心行为
└── test_edge_cases.py     # 边界
```

## 命名

```
test_<行为>_<场景>
test_cancel_order_when_insufficient_balance_returns_error
```

## 一个测试只测一个行为

```python
def test_cancel_order_restores_inventory():
    # arrange
    # act
    # assert — 只断言这件事
```

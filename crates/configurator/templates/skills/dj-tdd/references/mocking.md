# Mock 策略参考

## 何时 Mock

- **Mock 外部边界**：网络请求、文件系统、系统时间、第三方 API
- **不要 Mock**：值对象、纯数据转换、你模块内的逻辑

## 如何 Mock

优先用依赖注入（接口/trait），其次函数参数。不 mock 具体类。

## 示例

```python
class PaymentService:
    def __init__(self, gateway: PaymentGateway):
        self._gateway = gateway  # 测试时注入 MockPaymentGateway
```

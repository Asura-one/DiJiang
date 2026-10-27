# dj-implement 验证陷阱集

## mock 服务绑定固定端口

后端协议/网络链路验证用本地 mock 服务 + 真实客户端走完整链路（比纯 mock 单测扎实）。mock 服务绑定**固定端口**而非随机分配——企业安全环境下高位随机端口的回环连接可能被间歇性拦截。

## 异步攒批服务的写入可见性

异步攒批服务（接收 → 队列 → 批量 flush 落库，攒批窗口秒级）的写入可见性是最终一致的：固定 sleep 短于攒批窗口必然假阴性（误判"没收到"）。用「轮询 + deadline」替代固定 sleep——0.5s 间隔轮询、deadline 取攒批窗口数倍，轮询到即通过，超时仍查不到才是真失败（区分时序假象与真实缺陷）。

## 框架陷阱

- **Starlette StaticFiles 404 是 raise HTTPException 而非返回 404 response**：SPA 回退逻辑检查 `response.status_code == 404` 永远走不到。catch `StarletteHTTPException` 后回退 index.html，`/api/*` 路径 re-raise 保持 JSON 404 语义。
- **FastAPI 路由函数返回注解写成联合类型**（如 `StreamingResponse | JSONResponse`）：联合类型被当 response model 解析，启动/测试收集即报错。`@app.api_route(...)` 显式加 `response_model=None`，或去掉返回类型注解。

## 观察 SOP（验收含 agent 无法执行的实测项时）

如"30 分钟实测窗口"等真实环境观察项：完成 agent 可验证部分（配置+冒烟+回归）即停，向用户交付**观察 SOP**：

1. 配置生效方式（重启哪个进程/读哪个文件）
2. 观察指标清单
3. 数据回报格式
4. 实测后的岔路选项（提交/回滚/继续调参）

worktree 保留待实测数据回来；实测未发生不进入 finish-work；禁止纸面推演替代实测后直接收尾。
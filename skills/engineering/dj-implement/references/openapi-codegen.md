# OpenAPI 契约先行与代码生成

适用：后端 API 需要「契约唯一事实源 → 双端生成 → 防漂移门禁」的实现方式（Go 后端 + TS 前端）。

## 流程

1. 写 `openapi.yaml` 作为唯一事实源，把通用约定写进 `info.description`：错误模型与错误码、状态码语义、分页与深翻限制、时间格式、CSV 导出转义规则。
2. oapi-codegen 生成 Go 类型 + server 接口；openapi-typescript 生成前端 TS 类型。
3. Makefile 提供 `openapi-gen`（生成）与 `openapi-diff`（防漂移），`check` 依赖 `openapi-diff`。

## oapi-codegen 要点

- 配置文件 `oapi-codegen.yaml`：`package` / `output` / `generate: {models, std-http-server, embedded-spec}`。
- **`output` 相对当前工作目录（CWD），不是相对配置文件目录**；在仓库根执行时写仓库根相对路径。
- **CLI `-o` 会被 config 里的 `output` 覆盖**，别指望用它生成到临时路径。
- 生成代码依赖 `github.com/getkin/kin-openapi/openapi3` 与 `github.com/oapi-codegen/runtime`，需 `go get` + `go mod tidy` 才能编译。
- 安装：`GOPROXY=https://goproxy.cn,direct go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@v2.4.1`。

## 防漂移（openapi-diff）

备份 → 重新生成 → diff → 不一致即 exit 1：

```make
openapi-diff:
	@set -e; tmp=$$(mktemp -d); \
	cp $(OAPI_GO_OUT) $$tmp/gen.go.orig; \
	cp $(OAPI_TS_OUT) $$tmp/types.ts.orig; \
	$(MAKE) --no-print-directory openapi-gen; \
	diff -q $$tmp/gen.go.orig $(OAPI_GO_OUT) >/dev/null || { echo "openapi 漂移(Go)"; exit 1; }; \
	diff -q $$tmp/types.ts.orig $(OAPI_TS_OUT) >/dev/null || { echo "openapi 漂移(TS)"; exit 1; }; \
	echo "openapi diff: 无漂移"
```

- 负向验证：往产物里追加一行 → `make openapi-diff` 必须报红；重生成会自动还原干净产物，无需 `git checkout`。

## YAML 陷阱

- flow mapping 里出现 ASCII `[` 会开启 sequence，报 `did not find expected ',' or '}'`。
  例：`{ type: string, description: 每行 cidr[,note] }` → 改成块式写法或给 description 加引号。
- 中文逗号、中文括号、`/` 在 flow mapping 里安全。

## 覆盖度核对

- OpenAPI `paths` 按路径键去重，一个路径键可含多个 operation（如 `GET+POST /blacklist`）。
- 核对覆盖度要数 **operation 数**（等于 Go `ServerInterface` 方法数），不是路径键数。
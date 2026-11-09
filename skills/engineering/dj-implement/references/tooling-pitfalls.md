# tooling-pitfalls: 编辑器与工具链陷阱集

从 live 副本平台进化回灌（2026-11-09；内容带实测证据，原样保留）。dj-implement 正文指针引入。

## AIGC hook 异步改写与 multiedit 部分落盘

每次成功写入后 mtime 可能被外部进程更新，下一条 edit 就会报 modified since last read；multiedit 失败可能已部分写入，留下不一致中间态——轻则变量已删/引用还在（NameError），重则整文件被截断成碎片。同一文件多处改动优先**整文件重写**。

multiedit 报 modified since last read 时**不是全有或全无**：实测 6 条编辑只落盘了第 1、2 条（两条 import），中间与后续编辑全部未应用——部分应用**不连续、不可预测**。失败后立即 grep/Select-String 逐条核对每条编辑的关键行实际落盘状态，再决定补哪些：既不能假设全部回滚重跑（会重复插入已应用项），也不能假设全部成功。

## 换行符污染：整文件假 diff

edit/write 工具可能把原 LF 文件写成 CRLF，git diff 显示整文件重写（实测 +128/-100 全行变更，真实逻辑改动仅 28 行）。提交前对每个改动文件做字节级核对：

```python
import subprocess
cur = open(f, "rb").read()
head = subprocess.check_output(["git", "show", f"HEAD:{f}"])
print(f, "cur CRLF", cur.count(b"\r\n"), "LF-only", cur.count(b"\n") - cur.count(b"\r\n"))
print(f, "head CRLF", head.count(b"\r\n"), "LF-only", head.count(b"\n") - head.count(b"\r\n"))
```

两侧风格不一致且 diff 全行变更 → 二进制归一回 HEAD 风格（CRLF→LF：`open(p,"wb").write(open(p,"rb").read().replace(b"\r\n", b"\n"))`，反向同理）。逐个检查本次全部改动文件，别只修 diff 最大的那个；归一后重跑该文件相关测试确认无损。

## uv 管理项目的测试运行陷阱

uv 项目（`uv.lock` + `pyproject.toml`）跑测试的三连坑，症状与解法：

- **`uv run pytest` 与 `uv run python -m pytest` 解释器不一致**：前者可能解析到系统 Python 的 pytest（sys.path 指向别处），后者强制走项目 venv。典型症状：`uv run python -c "import app.xxx"` 正常，pytest 收集却对新加模块报 `ModuleNotFoundError`。**统一用 `uv run python -m pytest`**，保证测试与手工导入同一解释器。
- **pytest 在 optional-dependencies 里默认不装**：`uv run` 默认 sync 只装主依赖组，`[project.optional-dependencies]` 的 pytest/pytest-asyncio 不在内 → `python -m pytest` 报 No module named pytest。先 `uv pip install pytest pytest-asyncio` 装进项目 venv；探查性依赖一律 `uv pip install`（不落 pyproject/uv.lock），确认长期需要才 `uv add`。
- **整文件跑挂起超时**：大测试文件整跑可能卡在监听类/重型 fixture 上超时。按测试类分批（`pytest tests/x.py::TestClass`）：先跑触及改动的类，再跑受影响相邻类；不要加长 timeout 硬等整文件。

## 疑似既有失败的 stash 对照归因

改码中途撞见失败、怀疑"本来就坏"（与本次改动无关）时，不要凭直觉断言，也不要顺手修掉。用 **stash 对照法**实测归因：

1. `git stash push -- <本次改动文件>`（只回退改动的已跟踪文件，未跟踪新文件留在原地）
2. 重跑该失败测试：**仍红 = 既有基线失败**（记入已知失败清单，不阻断本次交付，报告中如实声明）；**转绿 = 本次改动引入**，回改码
3. `git stash pop` 恢复改动继续

适用：没做改前基线快照时的补救归因，或失败出现在基线未覆盖的测试里。结论必须写进交付报告（"既有失败，已 stash 对照验证"），别让疑似基线问题污染本次改动的验收，也别把它默默修掉混入本次 diff。

## Go + Vite 前端嵌入（go:embed）的构建顺序

前端产物被 Go `//go:embed` 嵌入时（如 `internal/dashboard/dist`），只改前端不重编 Go，二进制里仍是旧页面。固定顺序：

1. `vite build`（`build.outDir` 指向 embed 目录，`emptyOutDir: true` 会清掉旧 hash 文件，git 里表现为旧 assets 删除 + 新 assets 未跟踪，属正常）
2. 重跑 `go build ./...` + `go test ./...`——embed 在**编译期**读取，dist 变了必须重新编译才生效；顺带验证 embed 目录存在（目录缺失直接编译失败）
3. 浏览器冒烟：静态服务直接托管 dist 目录即可验证 SPA 挂载。无后端时 `/api/*` 返回 404 是**预期**，判据是"无 Vue 运行时错误"而非"无 console 报错"；用注入 localStorage token 绕过路由守卫逐页加载，确认各页模板可挂载

若 `pnpm exec vite` 在 Node 24 下崩溃，直接 `node node_modules/vite/bin/vite.js build`。

## Windows 上 Go 项目走 WSL 构建

Windows 本机常无 `go`（`where go` 空、`GOROOT` 空），而源码在 `E:\`。用 WSL 发行版构建：

```bash
wsl.exe -d Debian -e bash -lc "cd '/mnt/e/<路径>' && export GOPATH=/tmp/gopath GOCACHE=/tmp/gocache GOFLAGS=-mod=mod && go build ./... && go test ./..."
```

- 先 `wsl -l -v` 列发行版，逐个 `wsl -d <distro> -e bash -lc "go version"` 找装了 go 的（各发行版版本不同，按 go.mod 要求挑）。
- **Ubuntu 发行版可能以只读回退方式挂载**（`wsl: ... mounted read-only as a fallback`），此时 `/tmp` 不可写、报 `go: creating work dir: read-only file system`。换一个可写发行版（如 Debian）即可，不必去修 WSL 磁盘。
- `GOCACHE`/`GOPATH` 指到 `/tmp`，避免污染 Windows 侧目录；源码经 `/mnt/e` 访问即可，无需复制。

## 端口影子占用：查询"空闲"但绑定失败/被别的实例应答

WSL2 mirrored 网络下，Windows 侧 `Get-NetTCPConnection -LocalPort <p>` 查不到监听（显示空闲），端口却实际被 WSL 侧服务应答（pentest-core 2026-11-04 实战：8002）。两种症状：

- **绑定失败但健康检查 200**：uvicorn 日志出现 `[Errno 10048] error while attempting to bind` 后进程退出，但 `/health/live` 仍返回 200——是 WSL 里的旧服务在应答，极易误判"服务已就绪"
- **静默错连**：请求打到 WSL 里的同名服务，语义正确但数据源不对（实测：隔离测试库 admin 密码 bcrypt 直连校验通过，API 登录却 401 → 应答者不是我们的实例）

**诊断三步**：
1. 看启动日志尾部：有无 `10048`/bind error、有无 `Application startup complete`——**日志是绑定成败的事实源，健康检查不是**
2. 验证应答者身份：用只有本实例才有的已知状态对照（隔离库种子账号密码直连 DB 校验 vs API 行为）
3. 起服务前后各探一次：`socket.connect_ex(("127.0.0.1", p))`（0=占用，10061=空闲）

**预防**：本地联调避开常用端口（8000/8002/3000），选 8010/3001 等非常规端口；后端起在隔离目录（独立 config.yaml + datas/），失败可整目录删除重试。

## PowerShell 判定原生命令结果：变量捕获的是输出不是退出码

`$x = git merge-base --is-ancestor $b main; $x -eq 0` 是**静默假错**：`$x` 捕获的是 stdout（该命令无输出 → 空串），空串 `-eq 0` 恒为 False——实测把 6 个已并入 main 的分支全部误判为「未合并」。判定原生命令成败 / 布尔结果必须用 `$LASTEXITCODE`，且在下一条命令执行前读取：

```powershell
git merge-base --is-ancestor $b main 2>$null
$anc = $LASTEXITCODE -eq 0   # True = 是 main 祖先
```

**分支是否已并入 main 的核查组合**（单一命令结论与预期矛盾时，换方法交叉验证，别信第一次结果）：

```powershell
git branch --merged main                                          # 初筛：tip 可达 main 的分支
git rev-list --count "main..$b"                                   # 严格：0 = main 已含该分支全部提交
git cherry main $b | Select-String '^\+'                          # cherry-pick 等效新增（应为空）
git log --merges --ancestry-path "$b..main" --format='%h %s' | Select-Object -Last 1   # 合并锚点佐证
```

已完全并入的分支只是遗留引用：`git branch -d`（安全删除，仅对已合并分支生效）清理即可，main 内容零变化。

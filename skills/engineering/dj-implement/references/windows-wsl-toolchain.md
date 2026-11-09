# Windows 主机 + WSL 混合工具链

适用：主机缺某语言工具链（如 Go），但 WSL 发行版里有；或需要在 WSL 的 make 里调用 Windows 侧安装的 node 工具。

## 1. 先探测，再决策

- 主机确认：`go version`、`where.exe go`、常见安装路径（`C:\Go\bin`、`%LOCALAPPDATA%\Programs\Go\bin`）、`$env:GOROOT`/`$env:GOPATH` 全查一遍，再下「没装」的结论。
- WSL 确认：`wsl -l -v` 列发行版，逐个 `wsl -d <distro> -- go version`。kali 可能报 systemd user session 失败、Ubuntu 可能磁盘只读挂载，Debian 常可用。
- 有可用 WSL 工具链时，**先向用户确认「编译/门禁在哪执行」**，不要擅自往主机装工具链。

## 2. WSL 访问 Windows 工作区

- 路径 `/mnt/e/...`；中文路径可用，命令里加引号。
- 输出为空或被截断时，把命令拆小重跑，不要当成失败。

## 3. WSL 里的 npm 可能是 Windows interop

- 症状：`npm install -g` 的日志路径是 `C:\Users\...\npm-cache`，说明调的是 Windows npm，不是 Linux 原生 npm。
- 在 WSL 里跑 Windows node 工具：
  `'/mnt/c/Program Files/nodejs/node.exe' 'C:/Users/<user>/.../node_modules/<pkg>/bin/cli.js' <args>`
- **输出路径必须是 Windows 可见路径**（相对路径或 `E:/...`）；`/tmp/...` 会报 `ERR_INVALID_FILE_URL_PATH: File URL path must be absolute`。
- Windows 全局 npm bin 不在 PATH 时：`npm prefix -g` 找目录，直接调 `<prefix>\<tool>.cmd`。

## 4. Go 模块代理

- `proxy.golang.org` / `github.com` 不可达时用 `GOPROXY=https://goproxy.cn,direct`。
- 先 `curl -sI` 探测哪个可达再决定，不要盲试。

## 5. `go build ./...` 不产出可执行文件

- 它只做编译检查，不把二进制写到源目录。
- 验证子命令行为用 `go run ./path/to/cmd -flag`。

## 6. git 在 exFAT / 非 NTFS 盘

- `fatal: detected dubious ownership` → `git config --global --add safe.directory '<绝对路径>'`（正斜杠）。
- 该错误会中断后续命令：`git remote add` 可能静默未生效，务必 `git remote -v` 复核后再 fetch。

## 7. 把已有目录接入远程仓库（本地文件与远程一致时）

```bash
git init
git remote add origin <url>
git fetch origin main
git reset --mixed origin/main      # 只动 index/HEAD，不覆盖工作区
git checkout -- <远程有而本地缺的文件>
```

- 先 clone 到临时目录比对文件大小/内容，确认一致再接入，避免覆盖。
- `reset --mixed` 后 `git status` 里显示 `D` 的文件是「远程有、工作区没有」，用 `git checkout --` 补回。

## 8. 行尾纪律

- PowerShell `Set-Content` 默认写 CRLF；仓库 `.gitattributes` 若为 `* text=auto`，checkout 侧仍可能转 CRLF。
- 提交前显式转 LF：
  `[System.IO.File]::WriteAllText($f, ($text -replace "`r`n","`n"), (New-Object System.Text.UTF8Encoding($false)))`
- `git add` 时的 "LF will be replaced by CRLF" 警告是 `text=auto` 的预期行为，入库内容仍是 LF。

## 9. 文件写入 hook

- 写入后文件可能被 hook 改写（注入 front matter / 措辞），紧接着 edit 会报 "modified since last read" → 先 read 再 edit。
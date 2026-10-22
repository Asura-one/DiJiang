# 受限网络远程部署链（完整命令）

来源：pentest-core 生产部署会话（2026-10-22）：服务器 docker daemon 出网受限 + 本地企业网挡 pypi/debian 直连，15+ 次试错收敛的可行路径。

## 0. 适用判断

- 服务器部署为 **docker compose 烤入镜像**（代码 `COPY` 进镜像，非挂载源码）→ 只改服务器代码文件**不生效**，必须重建镜像 + recreate 容器
- 部署目录为发布快照（`current -> <时间戳目录>/` symlink，无 `.git`）

## 1. 改前备份（回滚锚点）

```bash
BACKUP=/home/<user>/<proj>/backup-$(date +%Y%m%d-%H%M)
sudo tar czf "$BACKUP/code.tgz" -C <deploy_dir> --exclude=datas --exclude=reports .
sudo cp -a <deploy_dir>/config <deploy_dir>/.env "$BACKUP/"
```

旧镜像**不删**：`docker images` 里的旧 image ID 就是回滚锚点（re-tag 回旧 ID 即回滚）。

## 2. 代码同步

```powershell
# 本地：git archive 只带代码（不带 config/datas/reports/.env）
git archive --format=tar --output=.temp\backend-sync.tar HEAD dispatcher server skills pyproject.toml uv.lock Dockerfile
scp .temp\backend-sync.tar user@host:/tmp/backend-sync.tar
```

```bash
# 服务器：注意未压缩 tar 用 -xf（-xzf 会报 not in gzip format）
tar -xf /tmp/backend-sync.tar -C <deploy_dir>
```

覆盖后 `grep -c "<新代码标记>" <file>` 逐项确认落地；生产 config/.env/datas/reports 不在包内，不会被覆盖。

## 3. 构建决策树

```
先试服务器构建：cd <真实部署目录> && sudo docker compose -f docker-compose.prod.yaml build <svc>
 ├─ 成功 → 跳到 5
 └─ 拉基础镜像超时（shell curl 通而 daemon 不通 = daemon 网络栈受限）
     → 本地 podman 构建 → 走 4 镜像分发
```

本地 podman 构建陷阱（Windows 企业网，全部实测）：

- **代理注入**：podman 把系统代理（如 127.0.0.1:18888）注入 build 环境 → pip `ProxyError`。父进程删 env 变量**无效**（buildah 读自身配置），必须：
  ```
  podman build --env HTTP_PROXY= --env HTTPS_PROXY= --env http_proxy= --env https_proxy= --env ALL_PROXY= --env all_proxy= ...
  ```
- **pypi 直连超时**：`--env PIP_INDEX_URL=https://pypi.tuna.tsinghua.edu.cn/simple --env UV_INDEX_URL=同左`
- **apt/debian**：`deb.debian.org` 不可达或 IPv6 dead → 部署专用 Dockerfile（放 `.temp/`，不污染仓库版）加：
  ```dockerfile
  RUN rm -f /etc/apt/sources.list /etc/apt/sources.list.d/debian.sources \
      && printf 'Acquire::ForceIPv4 "true";\n' > /etc/apt/apt.conf.d/99forcev4 \
      && printf 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ trixie main\ndeb http://mirrors.tuna.tsinghua.edu.cn/debian/ trixie-updates main\n' > /etc/apt/sources.list \
      && apt-get update && apt-get install -y --no-install-recommends curl \
      && rm -rf /var/lib/apt/lists/*
  ```

## 4. 镜像分发链

```powershell
podman build -f .temp\deploy.Dockerfile -t <name>:local .     # 附加上述 env
podman save -o .temp\image.tar <name>:local                   # 260MB 级
scp .temp\image.tar user@host:/tmp/image.tar
```

```bash
sudo docker load -i /tmp/image.tar                                        # 得到 localhost/<name>:local
sudo docker tag localhost/<name>:local <prod-name>:<prod-tag>            # re-tag 到 compose 引用的 tag
```

re-tag 后 compose 引用的 tag 已指向新 image ID。

## 5. 容器 recreate（compose 卡死时手动重建）

compose 三个坑（实测）：

- **tag 不变时 `up -d` 不 recreate**（config-hash 没变）→ 必须 `--force-recreate`
- **从 symlink 目录跑 compose 项目名会变成目录名**（如 `current`），与既有容器的 `com.docker.compose.project` label 不匹配 → 命令挂起或创建平行项目。从真实目录跑，或 `-p <原项目名>`（原项目名从 `docker inspect <ctr>` 的 labels 读）
- daemon 异常时 compose 命令整体挂死（SSH 超时杀不掉）→ 放弃 compose，手动重建

手动重建（docker run 复刻）：

1. `docker inspect <ctr> > /tmp/x.json`，提取：Image、Env、Cmd、RestartPolicy、NetworkMode、PortBindings、Mounts（source/dest/ro|rw）、Healthcheck、compose labels
2. 依赖顺序：**先停依赖方（dispatcher）再停被依赖方（server）；恢复时先起 server 等 healthy 再起 dispatcher**
3. `docker run -d --name <ctr> --restart unless-stopped --network <net> [-p 8000:8000] [--health-cmd/-interval/-timeout/-retries/-start-period] -v <src>:<dst>[:ro] -e <K=V> -l com.docker.compose.project=<原项目> -l com.docker.compose.service=<svc> -l com.docker.compose.project.working_dir=<真实目录> -l com.docker.compose.project.config_files=<yaml> <image> <cmd>`
   - compose labels 必须带上，否则后续 compose 运维识别不到该容器
   - health-cmd 里的内嵌引号最易翻车：`--health-cmd "python -c '...urllib.request.urlopen(\"http://127.0.0.1:8000/health/live\", timeout=5)'"`

## 6. 部署后验证（不验证 = 没部署）

```bash
sudo docker exec <ctr> grep -c "<每个新代码标记>" <对应源文件>   # 全部 >0 才算生效
sudo docker ps --filter name=<前缀> --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'
curl -s http://127.0.0.1:<port>/health/live                      # 200
sudo docker logs --tail 30 <dispatcher>                          # 正常轮询日志
```

## 7. 清理（凭证卫生）

- 服务器 `/tmp`：image tar、代码 tar、`.pw_b64`、inspect dump 全删并复核
- 本地 `.temp`：tar、密码编码文件删（回收站）
- 见 `dj-hunt/references/ssh-sudo-patterns.md` §2 的凭证规则

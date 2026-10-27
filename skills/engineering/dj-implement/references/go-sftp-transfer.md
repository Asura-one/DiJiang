# Go SFTP 文件传输：pkg/sftp 开发与测试知识库

适用：用 `github.com/pkg/sftp` 实现/优化文件传输（scp 类子功能）时的开发与测试参考。
来源：玄枢 scp P1 优化（断点续传 / 覆盖策略 / 元数据保留）实战验证，pkg/sftp v1.13.6。

## 1. 无 sshd 的传输层测试：内存 sftp server

传输核心逻辑（接受 `*sftp.Client` 的纯函数）不需要真实 SSH 服务端即可做**真实协议往返**测试：

```go
func newTestSFTP(t *testing.T) (*sftp.Client, func()) {
    root := t.TempDir()
    c1, c2 := net.Pipe()
    server, err := sftp.NewServer(c1, sftp.WithServerWorkingDirectory(root))
    if err != nil { t.Fatal(err) }
    go server.Serve()
    client, err := sftp.NewClientPipe(c2, c2)
    if err != nil { t.Fatal(err) }
    return client, func() { _ = client.Close(); _ = server.Close() }
}
```

- client 传**相对 POSIX 路径**（`sub/file.txt`），server 以 root 为基准解析；Windows 同样成立（server_windows.go 的 `toLocalPath` 会 join workDir）。
- 本地侧用另一个 `t.TempDir()` + `filepath.Join`。
- 可覆盖：单文件/目录双向、断点续传、冲突策略、preserve、dry-run——全部单测化，无需真实 sshd。
- 传输核心函数应接受可空的进度对象（如 `*progressManager`）并做 nil guard，测试传 nil 即可。

## 2. pkg/sftp 关键 API（v1.13.x）

| API | 行为 | 用途 |
|---|---|---|
| `Client.Create(path)` | `O_RDWR\|O_CREAT\|O_TRUNC`，**总是截断** | 默认覆盖写入 |
| `Client.OpenFile(path, flags)` | 自定义 flags | 续传上传：`O_WRONLY\|O_CREATE`（**不带 O_TRUNC**） |
| `File.Seek / ReadAt / WriteAt / Truncate` | 均可用 | 续传 offset 控制 |
| `Client.Stat / Lstat` | 目标大小探测 | 续传决策 |
| `Client.Chtimes / Chmod` | 设置 mtime / 权限 | `-p/--preserve` 元数据保留 |
| `Client.MkdirAll` | 递归建目录 | 目录传输自动建目标 |

## 3. 断点续传语义（rsync `--partial` 同款假设）

- **下载续传**：本地已存在且大小 < 远端 → `os.OpenFile(local, O_WRONLY|O_CREATE)`（不截断）+ `Seek(本地大小)` 继续写。
- **上传续传**：远端已存在且大小 < 本地 → `OpenFile(remote, O_WRONLY|O_CREATE)` + `Seek(远端大小)` 继续写。
- 目标大小 == 源 → 跳过（或按策略处理）。
- **目标比源大 → 报错不破坏**（续传无意义，需 `--force` 显式截断重传）。
- 续传假定目标已有部分是有效数据——必须在代码注释/文档中说明该假设。

## 4. 覆盖策略决策矩阵

| 策略 | 目标不存在 | 目标存在且不完整 | 目标存在且完整 |
|---|---|---|---|
| 默认 | 写入 | 覆盖 | 覆盖 |
| `--resume` | 写入 | 续传 | 跳过 |
| `--no-clobber` | 写入 | 跳过 | 跳过 |
| `--force` | 写入 | 全量覆盖 | 全量覆盖 |
| `--dry-run` | 打印不写 | 打印不写 | 打印不写 |

CLI 对齐 OpenSSH scp 语义：`-p` 保留时间/权限、`-d` 下载、`-f` 强制、`--no-clobber` 跳过、`--dry-run` 预演。

## 5. 生态事实与常见坑

- **OpenSSH 9.0（2022-04）起官方 scp 默认改用 SFTP 协议**——新实现直接走 SFTP 是正确路线，不要实现 scp 协议。
- 目录传输用 `filepath.Walk`（本地）/ `client.Walk`（远端）递归；symlink 行为需显式定义（`filepath.Walk` 不跟随目录 symlink）。
- 传输逻辑零单测是常见欠账：拆成接受 `*sftp.Client` 的纯函数后，用第 1 节 harness 补测成本极低。
- 进度条（mpb）双层结构：预扫描总量 + 单文件进度；未知大小时 total 设 1 兜底，避免除零。
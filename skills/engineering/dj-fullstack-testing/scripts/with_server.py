#!/usr/bin/env python3
"""起多台服务器 → 等就绪 → 跑命令 → 保证清理（进程组级）。

Adapted from anthropics/skills webapp-testing scripts/with_server.py (Apache-2.0).
DiJiang 改造点：
  ① 进程组清理——原版只 terminate 顶层 shell，`npm run dev` 会残留 node 子进程
  ② 端口 + HTTP 双探活——端口可连 ≠ 服务可用；非 HTTP 服务用 --tcp-only
  ③ 日志落盘 .temp/with-server/，起不来时打印尾部，不吞错误
  ④ 前一台就绪才启动下一台，保证依赖顺序
  ⑤ HTTP 探活显式绕过系统代理（本机代理拦 localhost 的实测陷阱）

用法：
  python3 with_server.py --server "npm run dev" --port 5173 -- python smoke.py
  python3 with_server.py \\
    --server "cd backend && python app.py" --port 8000 \\
    --server "npm run dev" --port 5173 -- pytest tests/ui -q
  python3 with_server.py --server "./bin/agent" --port 9900 --tcp-only -- ./probe.sh

退出码 = 目标命令的返回码；服务器无法就绪 / 用法错误时为 1。
"""

import argparse
import os
import signal
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

WINDOWS = os.name == "nt"
DEFAULT_TIMEOUT = 60
LOG_DIR = Path(".temp/with-server")


def http_ready(port: int, path: str, timeout: float = 3.0) -> bool:
    """GET http://127.0.0.1:port/path，状态码 <500 视为存活（401/404/302 都算活着）。"""
    url = f"http://127.0.0.1:{port}{path}"
    # 显式空代理：系统代理拦 localhost 时 urllib 会走代理导致永远探不到
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open(url, timeout=timeout) as resp:
            return resp.status < 500
    except urllib.error.HTTPError as e:
        return e.code < 500
    except (urllib.error.URLError, OSError, ValueError):
        return False


def wait_ready(port: int, http_path: str, tcp_only: bool, timeout: int):
    """轮询就绪。返回就绪方式描述；超时返回 None。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=1):
                if tcp_only:
                    return f"tcp:{port}"
                if http_ready(port, http_path):
                    return f"http:{port}{http_path}"
        except OSError:
            pass
        time.sleep(0.5)
    return None


def tail_text(path: Path, lines: int = 30) -> str:
    try:
        content = path.read_text(encoding="utf-8", errors="replace").splitlines()
        return "\n".join(content[-lines:])
    except OSError:
        return "(日志不可读)"


def start_server(cmd: str, port: int, log_path: Path):
    log = open(log_path, "ab")
    kwargs = {}
    if WINDOWS:
        kwargs["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    else:
        kwargs["start_new_session"] = True  # 独立进程组，清理时整组收割
    proc = subprocess.Popen(cmd, shell=True, stdout=log, stderr=subprocess.STDOUT, **kwargs)
    return proc, log


def stop_group(proc: subprocess.Popen) -> None:
    """杀整个进程组（SIGTERM → 5s → SIGKILL），不留孤儿。"""
    if proc.poll() is not None:
        return
    try:
        if WINDOWS:
            subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True)
        else:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
    except (ProcessLookupError, PermissionError, OSError):
        try:
            proc.terminate()
        except OSError:
            pass
    for _ in range(50):
        if proc.poll() is not None:
            return
        time.sleep(0.1)
    try:
        if WINDOWS:
            proc.kill()
        else:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
    except (ProcessLookupError, OSError):
        pass


def main() -> int:
    parser = argparse.ArgumentParser(
        description="起多台服务器，等就绪后跑命令，结束时清理全部进程组",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__.split("用法：")[-1].strip(),
    )
    parser.add_argument("--server", action="append", dest="servers", required=True, metavar="CMD",
                        help="服务器启动命令（可重复，含 cd/&& 时整体引号包裹）")
    parser.add_argument("--port", action="append", dest="ports", type=int, required=True, metavar="N",
                        help="每台服务器的端口（与 --server 一一对应）")
    parser.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT, help="单台就绪超时秒数（默认 60）")
    parser.add_argument("--http-path", default="/", help="HTTP 探活路径（默认 /）")
    parser.add_argument("--tcp-only", action="store_true", help="只探 TCP 端口（非 HTTP 服务）")
    parser.add_argument("--log-dir", default=str(LOG_DIR), help="服务器日志目录（默认 .temp/with-server）")
    parser.add_argument("command", nargs=argparse.REMAINDER, metavar="-- COMMAND",
                        help="就绪后执行的命令")
    args = parser.parse_args()

    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("缺少要执行的命令（用 -- 分隔，如 -- pytest -q）")
    if len(args.servers) != len(args.ports):
        parser.error("--server 与 --port 数量必须一致")
    if len(set(args.ports)) != len(args.ports):
        parser.error("端口重复")

    log_dir = Path(args.log_dir)
    log_dir.mkdir(parents=True, exist_ok=True)

    started = []  # (proc, port, log_path, log_handle)
    try:
        for i, (cmd, port) in enumerate(zip(args.servers, args.ports), 1):
            log_path = log_dir / f"server-{port}.log"
            print(f"[{i}/{len(args.servers)}] 启动: {cmd}  (日志 {log_path})")
            proc, log = start_server(cmd, port, log_path)
            started.append((proc, port, log_path, log))
            how = wait_ready(port, args.http_path, args.tcp_only, args.timeout)
            if how is None:
                print(f"✗ 端口 {port} {args.timeout}s 内未就绪。日志尾部：")
                print("  " + tail_text(log_path).replace("\n", "\n  "))
                return 1
            print(f"  就绪 ({how})")

        print(f"全部 {len(started)} 台就绪，执行: {' '.join(args.command)}\n")
        result = subprocess.run(args.command)
        return result.returncode
    except KeyboardInterrupt:
        print("\n中断，清理服务器…")
        return 130
    finally:
        for proc, port, log_path, log in started:
            print(f"停止 :{port}", end=" ")
            stop_group(proc)
            log.close()
            print("done")


if __name__ == "__main__":
    sys.exit(main())

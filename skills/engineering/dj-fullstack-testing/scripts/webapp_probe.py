#!/usr/bin/env python3
"""探测运行中的 web 页面：元素发现 / 控制台+网络监听 / 截图。

Adapted from anthropics/skills webapp-testing examples/
{element_discovery,console_logging,static_html_automation}.py (Apache-2.0)，
合并为单工具并新增网络失败捕获与 --fail-on-error 判定。

用法：
  python3 webapp_probe.py discover http://localhost:5173 [--json]
      # 元素发现：按钮/链接/表单/交互标记 → 文本或 JSON（喂 dj-fullstack-testing 步骤 1 的枚举清单）
  python3 webapp_probe.py console http://localhost:5173 --seconds 8 [--fail-on-error]
      # 监听 console 消息 + pageerror + 网络失败（4xx/5xx 或请求崩溃）
  python3 webapp_probe.py shot http://localhost:5173 --out .temp/probe --viewport mobile --full-page
      # 截图：desktop 1920x1080 / mobile 390x844

约定：动态页 goto 后等 networkidle 再操作（webapp-testing 实测陷阱）。
本地静态 HTML 直接用 file:// URL。缺 playwright 依赖时退出 2 并给安装提示。
"""

import argparse
import json
import sys
from pathlib import Path

VIEWPORTS = {
    "desktop": {"width": 1920, "height": 1080},
    "mobile": {"width": 390, "height": 844},
}
MAX_ITEMS = 60  # 控制台/网络事件打印上限，防刷屏


def load_playwright():
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("缺少 playwright 依赖。安装：pip install playwright && playwright install chromium",
              file=sys.stderr)
        sys.exit(2)
    return sync_playwright


def new_page(browser, viewport):
    page = browser.new_page(viewport=VIEWPORTS[viewport])
    return page


def goto_and_settle(page, url):
    page.goto(url, wait_until="networkidle", timeout=20000)


def clean(text):
    return " ".join((text or "").split())[:80]


# ---------- discover ----------

def cmd_discover(url: str, as_json: bool) -> int:
    with load_playwright()() as p:
        browser = p.chromium.launch(headless=True)
        page = new_page(browser, "desktop")
        goto_and_settle(page, url)

        buttons = []
        for b in page.locator("button").all()[:MAX_ITEMS]:
            try:
                buttons.append(clean(b.inner_text()) or "[无文字]")
            except Exception:
                buttons.append("[detached]")
        links = []
        for a in page.locator("a[href]").all()[:MAX_ITEMS]:
            try:
                links.append({"text": clean(a.inner_text()), "href": a.get_attribute("href")})
            except Exception:
                pass
        inputs = []
        for i in page.locator("input, textarea, select").all()[:MAX_ITEMS]:
            name = i.get_attribute("name") or i.get_attribute("id") or "[unnamed]"
            inputs.append({"name": name, "tag": i.evaluate("e => e.tagName.toLowerCase()"),
                           "type": i.get_attribute("type")})
        data_tests = page.locator("[data-test], [data-testid]").count()

        browser.close()

    if as_json:
        print(json.dumps({"url": url, "buttons": buttons, "links": links,
                          "inputs": inputs, "data_test_count": data_tests},
                         ensure_ascii=False, indent=2))
        return 0
    print(f"页面: {url}")
    print(f"\n按钮 ({len(buttons)}):")
    for i, t in enumerate(buttons):
        print(f"  [{i}] {t}")
    print(f"\n链接 ({len(links)}):")
    for l in links:
        print(f"  - {l['text']} -> {l['href']}")
    print(f"\n表单控件 ({len(inputs)}):")
    for f in inputs:
        print(f"  - {f['name']} <{f['tag']} type={f['type']}>")
    print(f"\ndata-test 标记: {data_tests}")
    return 0


# ---------- console ----------

def cmd_console(url: str, seconds: int, clicks, fail_on_error: bool) -> int:
    with load_playwright()() as p:
        browser = p.chromium.launch(headless=True)
        page = new_page(browser, "desktop")
        events = []

        page.on("console", lambda m: events.append(
            {"kind": f"console.{m.type}", "text": clean(m.text)}))
        page.on("pageerror", lambda e: events.append(
            {"kind": "pageerror", "text": clean(str(e))}))
        page.on("requestfailed", lambda r: events.append(
            {"kind": "request.failed", "text": f"{r.method} {r.url} {r.failure}"}))
        page.on("response", lambda r: events.append(
            {"kind": "http", "status": r.status, "text": f"{r.status} {r.request.method} {r.url}"}
        ) if r.status >= 400 else None)

        goto_and_settle(page, url)
        for selector in clicks or []:
            try:
                page.click(selector, timeout=3000)
            except Exception as e:
                events.append({"kind": "probe.click_failed",
                               "text": f"{selector}: {clean(str(e))}"})
        page.wait_for_timeout(seconds * 1000)
        browser.close()

    problems = [e for e in events
                if e["kind"] in ("pageerror", "console.error", "request.failed")
                or e.get("status", 0) >= 400]

    n_err = sum(1 for e in events if e["kind"] == "console.error")
    n_page = sum(1 for e in events if e["kind"] == "pageerror")
    n_net = sum(1 for e in events if e["kind"] == "request.failed" or e.get("status", 0) >= 400)
    print(f"页面: {url}（静置 {seconds}s）")
    print(f"console.error {n_err} | pageerror {n_page} | 网络失败 {n_net}")
    if problems:
        print("\n问题事件：")
        for e in problems[:MAX_ITEMS]:
            print(f"  [{e['kind']}] {e['text']}")
    else:
        print("无 console error / pageerror / 网络失败")

    if fail_on_error and problems:
        print("\nFAIL：存在页面错误事件（--fail-on-error）")
        return 1
    return 0


# ---------- shot ----------

def cmd_shot(url: str, out: str, viewport: str, full_page: bool) -> int:
    out_dir = Path(out)
    out_dir.mkdir(parents=True, exist_ok=True)
    name = url.split("//", 1)[-1].replace("/", "_").replace(":", "_")[:60]
    path = out_dir / f"{name}-{viewport}{'-full' if full_page else ''}.png"
    with load_playwright()() as p:
        browser = p.chromium.launch(headless=True)
        page = new_page(browser, viewport)
        goto_and_settle(page, url)
        page.screenshot(path=str(path), full_page=full_page)
        browser.close()
    print(f"截图: {path}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="web 页面探测：discover / console / shot",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    d = sub.add_parser("discover", help="元素发现（喂枚举清单）")
    d.add_argument("url")
    d.add_argument("--json", action="store_true", help="输出 JSON")

    c = sub.add_parser("console", help="监听控制台与网络失败")
    c.add_argument("url")
    c.add_argument("--seconds", type=int, default=5, help="点击后静置秒数（默认 5）")
    c.add_argument("--click", action="append", metavar="SELECTOR",
                   help="静置前点击的选择器，可重复（如 #bad-btn、text=提交）")
    c.add_argument("--fail-on-error", action="store_true", help="有错误事件时退出码 1")

    s = sub.add_parser("shot", help="截图")
    s.add_argument("url")
    s.add_argument("--out", default=".temp/probe", help="输出目录（默认 .temp/probe）")
    s.add_argument("--viewport", choices=list(VIEWPORTS), default="desktop")
    s.add_argument("--full-page", action="store_true", help="整页截图")

    args = parser.parse_args()
    if args.cmd == "discover":
        return cmd_discover(args.url, args.json)
    if args.cmd == "console":
        return cmd_console(args.url, args.seconds, args.click, args.fail_on_error)
    return cmd_shot(args.url, args.out, args.viewport, args.full_page)


if __name__ == "__main__":
    sys.exit(main())

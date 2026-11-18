#!/usr/bin/env bash
# wizard-template.sh — dj-wizard 标准模板（每个 wizard 按此结构填写阶段区）
# 库函数区所有 wizard 一致，勿改：进度 / 确认门 / 跨平台 URL / 隐藏输入 / 幂等 .env upsert / 结尾总结
# macOS (bash 3.2+) / Linux / Git Bash 兼容；模板可直接 `bash scripts/wizard-template.sh` 试跑。
set -euo pipefail

STAGES_TOTAL=3   # ← 按实际阶段数改
declare -a SKIPPED=()

say() { STAGE=$((STAGE)); printf '\n[%d/%d] %s\n' "$STAGE" "$STAGES_TOTAL" "$*"; }

confirm() { # confirm <提示> [默认y|n]
  local prompt="$1" def="${2:-y}" ans
  read -r -p "$prompt $([ "$def" = y ] && echo '[Y/n]' || echo '[y/N]') " ans || return 1
  ans="${ans:-$def}"
  [[ "$ans" =~ ^[Yy] ]]
}

open_url() { # 跨平台打开；开不了就打印让用户手动
  local url="$1"
  case "$(uname -s)" in
    Darwin)            open "$url" ;;
    Linux)             xdg-open "$url" >/dev/null 2>&1 || echo "  手动打开: $url" ;;
    MINGW*|MSYS*|CYGWIN*) start "$url" >/dev/null 2>&1 || echo "  手动打开: $url" ;;
    *) echo "  手动打开: $url" ;;
  esac
}

read_secret() { # 隐藏输入，值存 $REPLY
  read -r -s -p "$1" REPLY || return 1
  echo >&2
}

env_upsert() { # env_upsert <file> <KEY> <value> —— 幂等，已存在先确认再覆盖
  local file="$1" key="$2" val="$3"
  touch "$file"
  if grep -q "^${key}=" "$file" 2>/dev/null; then
    if ! confirm "  $key 已存在，覆盖?" n; then
      echo "  跳过 $key（保留旧值）"
      return 0
    fi
    grep -v "^${key}=" "$file" > "${file}.tmp" || true
    mv "${file}.tmp" "$file"
  fi
  printf '%s="%s"\n' "$key" "$val" >> "$file"
  echo "  已写入 $key"
}

# ================= 阶段区（照此结构复制填写，删掉示例）=================

STAGE=1
say "阶段 1：打开控制台"
open_url "https://example.com/console"
confirm "已打开并登录？" || SKIPPED+=("1:控制台")

STAGE=2
say "阶段 2：采集 API Key"
echo "  控制台 Settings → API Keys → 复制 key"
read_secret "  粘贴 API Key: "
env_upsert ".env" "API_KEY" "$REPLY"

STAGE=3
say "阶段 3：（照抄结构继续……）"

# ================= 结尾总结 =================
say "完成"
if [ "${#SKIPPED[@]}" -gt 0 ] 2>/dev/null; then
  echo "未完成阶段：${SKIPPED[*]}"
else
  echo "全部阶段完成"
fi

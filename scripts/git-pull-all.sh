#!/bin/bash
# ============================================================
# 批量 Git Pull 脚本 (优化版)
# 遍历当前目录下所有 git 仓库，批量执行 git pull
# ============================================================

set -uo pipefail

# ============ 默认配置 ============
ROOT_DIR="$(pwd)"
VERBOSE=false
DRY_RUN=false
EXCLUDE_DIRS=(".git" "node_modules" "dist" "build" ".next" "vendor")

# ============ 颜色定义 ============
if [[ -t 1 ]]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    DIM='\033[2m'
    RESET='\033[0m'
else
    GREEN=''
    RED=''
    YELLOW=''
    BLUE=''
    CYAN=''
    DIM=''
    RESET=''
fi

# ============ 帮助信息 ============
show_help() {
    cat << EOF
批量 Git Pull 工具

用法: $(basename "$0") [选项] [目录]

选项:
  -h, --help        显示帮助信息
  -v, --verbose     详细输出，显示完整 git pull 输出
  -n, --dry-run     预览模式，不实际执行
  --include-hidden  包含隐藏目录 (默认跳过 . 开头的目录)

示例:
  $(basename "$0")                        # 在当前目录执行
  $(basename "$0") ~/Projects            # 指定目录
  $(basename "$0") -v ~/Projects         # 详细模式
EOF
}

# ============ 解析参数 ============
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        --include-hidden)
            INCLUDE_HIDDEN=true
            shift
            ;;
        -*)
            echo "未知选项: $1"
            show_help
            exit 1
            ;;
        *)
            if [[ -d "$1" ]]; then
                ROOT_DIR="$(cd "$1" && pwd)"
            else
                echo "目录不存在: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# 默认值
INCLUDE_HIDDEN=${INCLUDE_HIDDEN:-false}

# ============ 初始化 ============
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BLUE}  批量 Git Pull 工具 v1.0${RESET}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "  目录: $ROOT_DIR"
echo ""

# 统计变量
SUCCESS_COUNT=0
FAIL_COUNT=0
UP_TO_DATE=0
HAS_UPDATE=0
FAIL_LIST=()
UPDATE_LIST=()

# ============ 收集所有 git 仓库 ============
REPOS=()
for dir in "$ROOT_DIR"/*/; do
    dir="${dir%/}"  # 去掉末尾斜杠
    repo_name="$(basename "$dir")"
    
    # 隐藏目录过滤
    if [[ "$INCLUDE_HIDDEN" = false && "$repo_name" == .* ]]; then
        continue
    fi
    
    # 排除目录
    skip=false
    for excl in "${EXCLUDE_DIRS[@]}"; do
        if [[ "$repo_name" == "$excl" ]]; then
            skip=true
            break
        fi
    done
    [[ "$skip" = true ]] && continue
    
    # 检查是否是 git 仓库
    if [[ -d "$dir/.git" ]]; then
        REPOS+=("$dir")
    fi
done

if [[ ${#REPOS[@]} -eq 0 ]]; then
    echo -e "${YELLOW}未找到任何 git 仓库${RESET}"
    exit 0
fi

echo -e "  发现 ${#REPOS[@]} 个仓库"
echo ""

# ============ 串行执行 ============
for dir in "${REPOS[@]}"; do
    repo_name="$(basename "$dir")"
    
    if [[ "$DRY_RUN" = true ]]; then
        echo "  [DRY-RUN] $repo_name"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        continue
    fi
    
    # 执行 git pull
    output=$(cd "$dir" && git pull 2>&1)
    exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        echo -e "${RED}✗  $repo_name${RESET}"
        if [[ "$VERBOSE" = true ]]; then
            echo "$output" | head -10 | sed 's/^/    /'
        fi
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAIL_LIST+=("$repo_name")
    else
        if echo "$output" | grep -q "Already up to date"; then
            echo -e "✓  $repo_name ${DIM}(已是最新)${RESET}"
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
            UP_TO_DATE=$((UP_TO_DATE + 1))
        else
            echo -e "${GREEN}↻  $repo_name${RESET} ${DIM}(已更新)${RESET}"
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
            HAS_UPDATE=$((HAS_UPDATE + 1))
            UPDATE_LIST+=("$repo_name")
        fi
    fi
done

# ============ 汇总输出 ============
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "  完成"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""
echo -e "  ✓ 成功:   $SUCCESS_COUNT 个 ($HAS_UPDATE 个有更新, $UP_TO_DATE 个已最新)"

if [[ ${#UPDATE_LIST[@]} -gt 0 ]]; then
    echo ""
    echo -e "${GREEN}  已更新列表:${RESET}"
    for repo in "${UPDATE_LIST[@]}"; do
        echo "    $repo"
    done
fi

if [[ $FAIL_COUNT -gt 0 ]]; then
    echo ""
    echo -e "${RED}  ✗ 失败:   $FAIL_COUNT 个${RESET}"
    echo -e "${RED}  失败列表:${RESET}"
    for repo in "${FAIL_LIST[@]}"; do
        echo "    $repo"
    done
fi

echo ""
[[ $FAIL_COUNT -gt 0 ]] && exit 1 || exit 0

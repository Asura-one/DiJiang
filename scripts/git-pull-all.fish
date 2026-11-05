#!/usr/bin/env fish

# 批量 git pull 脚本 (Fish 版)
# 遍历当前目录下的所有 git 仓库并执行 git pull

# ============== 配置 ==============
set -l ROOT_DIR (pwd)
set -l VERBOSE false
set -l PARALLEL false
# ==================================

# 解析参数
while test (count $argv) -gt 0
    switch $argv[1]
        case -h --help
            echo "批量 git pull 脚本"
            echo ""
            echo "用法: fish git-pull-all.fish [选项] [目录]"
            echo ""
            echo "选项:"
            echo "  -h, --help      显示帮助"
            echo "  -v, --verbose   详细输出"
            echo "  -p, --parallel  并行执行"
            echo ""
            echo "示例:"
            echo "  fish git-pull-all.fish              # 在当前目录执行"
            echo "  fish git-pull-all.fish ~/Projects   # 指定目录"
            exit 0
        case -v --verbose
            set VERBOSE true
        case -p --parallel
            set PARALLEL true
        case '*'
            if test -d $argv[1]
                set ROOT_DIR (realpath $argv[1])
            else
                echo "目录不存在: $argv[1]"
                exit 1
            end
    end
    set argv $argv[2..-1]
end

# 颜色
set -l GREEN (set_color green)
set -l RED (set_color red)
set -l YELLOW (set_color yellow)
set -l BLUE (set_color blue)
set -l CYAN (set_color cyan)
set -l DIM (set_color brblack)
set -l RESET (set_color normal)

echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo $BLUE"  批量 Git Pull 工具"$RESET
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo "目录: $ROOT_DIR"
echo ""

set -gx success_count 0
set -gx fail_count 0
set -gx up_to_date 0
set -gx has_update 0
set -gx fail_list ""

# 遍历所有子目录
for dir in $ROOT_DIR/*
    if not test -d $dir
        continue
    end

    # 检查是否是 git 仓库
    if not test -d $dir/.git
        continue
    end

    set repo_name (basename $dir)

    # 进入目录并 git pull
    cd $dir
    set output (git pull 2>&1)
    set exit_code $status
    cd $ROOT_DIR

    if test $exit_code -ne 0
        echo $RED"✗  $repo_name"$RESET
        if test $VERBOSE = true
            echo $RED"    $output"$RESET | head -5
        end
        set fail_count (math $fail_count + 1)
        set -a fail_list $repo_name
    else
        # 判断是更新了还是已是最新
        if echo $output | grep -q "Already up to date"
            echo "✓  $repo_name" $DIM"(已是最新)"$RESET
            set up_to_date (math $up_to_date + 1)
        else
            echo $GREEN"↻  $repo_name"$RESET $DIM"(已更新)"$RESET
            set has_update (math $has_update + 1)
        end
        set success_count (math $success_count + 1)
    end
end

echo ""
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo "  完成"
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo ""
echo "  ✓ 成功:   $success_count 个 ($has_update 个有更新, $up_to_date 个已最新)"
if test $fail_count -gt 0
    echo $RED"  ✗ 失败:   $fail_count 个"$RESET
    echo $RED"  失败列表:"$RESET
    for item in $fail_list
        echo "    $item"
    end
end

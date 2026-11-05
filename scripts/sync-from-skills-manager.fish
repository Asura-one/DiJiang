#!/usr/bin/env fish

# DiJiang Skill 同步脚本 v2.0
# 从 skill-manager 目录同步更新到 DiJiang 项目
# 环境：macOS + Fish shell

# ============== 默认配置 ==============
set -l SOURCE_DIR ~/.skills-manager/skills
set -l TARGET_DIR ~/Project/DiJiang/skills
set -l CATEGORIES engineering productivity
set -l DRY_RUN false
set -l VERBOSE false
set -l SHOW_DIFF false
set -l BACKUP true
set -l AUTO_COMMIT false
# ====================================

# ============ 帮助信息 ============
function show_help
    echo "DiJiang Skill 同步工具"
    echo ""
    echo "用法: fish sync-from-skills-manager.fish [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help        显示帮助信息"
    echo "  -n, --dry-run     预览模式，不实际执行同步"
    echo "  -v, --verbose     详细输出，显示每个 skill 状态"
    echo "  -d, --show-diff   显示 SKILL.md 内容差异"
    echo "  --no-backup       不创建备份"
    echo "  --auto-commit     同步后自动 git commit"
    echo "  --source DIR      指定源目录"
    echo "  --target DIR      指定目标目录"
    echo ""
    echo "示例:"
    echo "  fish sync-from-skills-manager.fish              # 正常同步"
    echo "  fish sync-from-skills-manager.fish --dry-run   # 预览模式"
    echo "  fish sync-from-skills-manager.fish -v -d       # 详细+显示差异"
end

# ============ 解析参数 ============
while test (count $argv) -gt 0
    switch $argv[1]
        case -h --help
            show_help
            exit 0
        case -n --dry-run
            set DRY_RUN true
        case -v --verbose
            set VERBOSE true
        case -d --show-diff
            set SHOW_DIFF true
        case --no-backup
            set BACKUP false
        case --auto-commit
            set AUTO_COMMIT true
        case --source
            set SOURCE_DIR $argv[2]
            set argv $argv[2..-1]
        case --target
            set TARGET_DIR $argv[2]
            set argv $argv[2..-1]
        case '*'
            echo "未知选项: $argv[1]"
            show_help
            exit 1
    end
    set argv $argv[2..-1]
end

# ============ 颜色定义 ============
set -l GREEN (set_color green)
set -l RED (set_color red)
set -l YELLOW (set_color yellow)
set -l BLUE (set_color blue)
set -l CYAN (set_color cyan)
set -l DIM (set_color brblack)
set -l RESET (set_color normal)

# ============ 目录检查 ============
if not test -d $SOURCE_DIR
    echo $RED"✗ 错误：源目录不存在 $SOURCE_DIR"$RESET
    exit 1
end

if not test -d $TARGET_DIR
    echo $RED"✗ 错误：目标目录不存在 $TARGET_DIR"$RESET
    exit 1
end

# 展开为绝对路径
set SOURCE_DIR (realpath $SOURCE_DIR)
set TARGET_DIR (realpath $TARGET_DIR)

# ============ 统计变量 (全局) ============
set -gx synced_count 0
set -gx unchanged_count 0
set -gx missing_count 0
set -gx new_count 0
set -gx error_count 0
set -gx synced_list ""
set -gx new_list ""
set -gx missing_list ""

# ============ 输出函数 ============
function print_header
    echo $BLUE"┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"$RESET
    echo $BLUE"┃"$RESET"  DiJiang Skill 同步工具 v2.0          "$BLUE"┃"$RESET
    echo $BLUE"┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"$RESET
    echo ""
    echo $DIM"  源目录:   "$RESET $SOURCE_DIR
    echo $DIM"  目标目录: "$RESET $TARGET_DIR
    set mode_str "正常同步"
    if test "$DRY_RUN" = "true"
        set mode_str "预览 (dry-run)"
    end
    echo $DIM"  模式:     "$RESET $mode_str
    echo ""
end

function print_footer
    echo ""
    echo $BLUE"┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓"$RESET
    echo $BLUE"┃"$RESET"  同步完成                            "$BLUE"┃"$RESET
    echo $BLUE"┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛"$RESET
    echo ""
    echo $GREEN"  ✓ 已同步:   "$synced_count" 个"$RESET
    echo "  ✓ 未更新:   "$unchanged_count" 个"
    if test $new_count -gt 0
        echo $CYAN"  ✚ 新增:     "$new_count" 个"$RESET
    end
    if test $missing_count -gt 0
        echo $YELLOW"  ⚠ 源中缺失: "$missing_count" 个"$RESET
    end
    if test $error_count -gt 0
        echo $RED"  ✗ 失败:     "$error_count" 个"$RESET
    end
    echo ""

    if test $synced_count -gt 0
        echo $GREEN"  已同步列表:"$RESET
        for item in $synced_list
            echo "    $item"
        end
        echo ""
    end

    if test $new_count -gt 0
        echo $CYAN"  新增列表 (源中有，目标中无):"$RESET
        for item in $new_list
            echo "    $item"
        end
        echo ""
    end

    if test $missing_count -gt 0
        echo $YELLOW"  缺失列表 (目标中有，源中无):"$RESET
        for item in $missing_list
            echo "    $item"
        end
        echo ""
    end

    if test $synced_count -gt 0
        echo $YELLOW"  提示: 检查变更并提交"$RESET
        echo "    cd ~/Project/DiJiang && git status"
        echo "    cd ~/Project/DiJiang && git diff"
    end
end

# ============ 主逻辑 ============
print_header

# 遍历每个分类目录
for category in $CATEGORIES
    set category_dir $TARGET_DIR/$category

    if not test -d $category_dir
        echo $YELLOW"⚠  分类目录不存在: $category_dir，跳过"$RESET
        continue
    end

    echo $CYAN"━━━ [$category] ━━━"$RESET

    # 遍历目标分类下的所有 skill
    for skill_path in $category_dir/*
        if not test -d $skill_path
            continue
        end

        set skill_name (basename $skill_path)
        set src_skill_dir $SOURCE_DIR/$skill_name
        set dst_skill_dir $category_dir/$skill_name

        # 检查源目录中是否存在
        if not test -d $src_skill_dir
            echo $YELLOW"  ⚠  $skill_name (源目录中不存在)"$RESET
            set missing_count (math $missing_count + 1)
            set -a missing_list "$category/$skill_name"
            continue
        end

        # 检测是否需要同步
        set needs_sync false
        set change_notes ""

        # 1. SKILL.md 内容差异
        if test -f $src_skill_dir/SKILL.md -a -f $dst_skill_dir/SKILL.md
            if not diff -q $src_skill_dir/SKILL.md $dst_skill_dir/SKILL.md > /dev/null 2>&1
                set needs_sync true
                set change_notes "SKILL.md"
            end
        end

        # 2. references 目录差异
        if test -d $src_skill_dir/references -a -d $dst_skill_dir/references
            if not diff -rq $src_skill_dir/references $dst_skill_dir/references > /dev/null 2>&1
                set needs_sync true
                if test -n "$change_notes"
                    set change_notes "$change_notes + references"
                else
                    set change_notes "references"
                end
            end
        end

        # 3. 源中有但目标中没有的文件
        for f in (find $src_skill_dir -type f -not -path "*/.*" -not -name ".DS_Store" 2>/dev/null)
            set rel_path (realpath --relative-to=$src_skill_dir $f)
            if not test -f $dst_skill_dir/$rel_path
                set needs_sync true
                if test -n "$change_notes"
                    set change_notes "$change_notes + 新增文件"
                else
                    set change_notes "新增文件"
                end
                break
            end
        end

        if test $needs_sync = true
            echo $GREEN"  ↻  $skill_name"$RESET $DIM"($change_notes)"$RESET

            if test $VERBOSE = true -o $SHOW_DIFF = true
                # 显示具体变更文件
                diff -rq $src_skill_dir $dst_skill_dir 2>/dev/null | while read -l line
                    echo $DIM"    $line"$RESET
                end
            end

            if test $SHOW_DIFF = true
                echo $DIM"    ────────"$RESET
                diff -u $dst_skill_dir/SKILL.md $src_skill_dir/SKILL.md 2>/dev/null | head -30 | while read -l line
                    echo $DIM"    $line"$RESET
                end
                echo $DIM"    ────────"$RESET
            end

            if test $DRY_RUN = false
                # 备份
                if test $BACKUP = true
                    set backup_dir /tmp/skill-sync-backup/(date +%Y%m%d-%H%M%S)/$category/$skill_name
                    mkdir -p $backup_dir
                    cp -r $dst_skill_dir/* $backup_dir/ 2>/dev/null
                end

                # 执行同步
                rsync -a --delete --exclude=".DS_Store" $src_skill_dir/ $dst_skill_dir/

                if test $status -eq 0
                    set synced_count (math $synced_count + 1)
                    set -a synced_list "$category/$skill_name"
                else
                    set error_count (math $error_count + 1)
                    echo $RED"    ✗ 同步失败"$RESET
                end
            else
                set synced_count (math $synced_count + 1)
                set -a synced_list "$category/$skill_name"
            end
        else
            if test $VERBOSE = true
                echo "  ✓  $skill_name"
            end
            set unchanged_count (math $unchanged_count + 1)
        end
    end

    echo ""
end

# 检测源目录中新增的 skill
echo $CYAN"━━━ [新增检测] ━━━"$RESET
for src_skill_dir in $SOURCE_DIR/*
    if not test -d $src_skill_dir
        continue
    end
    set skill_name (basename $src_skill_dir)

    # 检查是否在目标中存在
    set found false
    for category in $CATEGORIES
        if test -d $TARGET_DIR/$category/$skill_name
            set found true
            break
        end
    end

    if test $found = false
        echo $CYAN"  ✚  $skill_name (新增，未纳入项目)"$RESET
        set new_count (math $new_count + 1)
        set -a new_list "$skill_name (请手动决定分类)"
    end
end

echo ""

# 自动 commit
if test $AUTO_COMMIT = true -a $synced_count -gt 0 -a $DRY_RUN = false
    echo $CYAN"━━━ [自动提交] ━━━"$RESET
    set project_root (dirname (dirname $TARGET_DIR))
    cd $project_root
    git add skills/
    git commit -m "chore: sync skills from skills-manager ($synced_count updated)"
    echo ""
end

print_footer

# 退出码
if test $error_count -gt 0
    exit 1
end
exit 0

#!/usr/bin/env fish

# DiJiang Skill 同步脚本
# 从 skill-manager 目录同步更新到 DiJiang 项目
# 环境：macOS M3 + Fish shell

# ============== 配置区 ==============
set SOURCE_DIR ~/.skills-manager/skills
set TARGET_DIR ~/Project/DiJiang/skills
set CATEGORIES engineering productivity
# ====================================

# 颜色定义
set GREEN (set_color green)
set RED (set_color red)
set YELLOW (set_color yellow)
set BLUE (set_color blue)
set RESET (set_color normal)

# 检查源目录
if not test -d $SOURCE_DIR
    echo $RED"错误：源目录不存在 $SOURCE_DIR"$RESET
    exit 1
end

# 检查目标目录
if not test -d $TARGET_DIR
    echo $RED"错误：目标目录不存在 $TARGET_DIR"$RESET
    exit 1
end

# 统计变量
set synced_count 0
set unchanged_count 0
set missing_count 0
set error_count 0
set synced_list ""
set missing_list ""

echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo $BLUE"  DiJiang Skill 同步工具"$RESET
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo ""
echo "源目录: $SOURCE_DIR"
echo "目标目录: $TARGET_DIR"
echo ""

# 遍历每个分类目录
for category in $CATEGORIES
    set category_dir $TARGET_DIR/$category

    if not test -d $category_dir
        echo $YELLOW"警告：分类目录不存在 $category_dir，跳过"$RESET
        continue
    end

    echo $BLUE"[$category]"$RESET

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
            set missing_list "$missing_list\n  $category/$skill_name"
            continue
        end

        # 对比 SKILL.md 的修改时间
        set src_mtime (stat -f %m $src_skill_dir/SKILL.md 2>/dev/null || echo 0)
        set dst_mtime (stat -f %m $dst_skill_dir/SKILL.md 2>/dev/null || echo 0)

        # 也检查 references 等其他文件
        set needs_sync 0

        # 如果源文件比目标新，需要同步
        if test $src_mtime -gt $dst_mtime
            set needs_sync 1
        end

        # 额外检查：如果内容有差异也同步
        if test -f $src_skill_dir/SKILL.md
            if not diff -q $src_skill_dir/SKILL.md $dst_skill_dir/SKILL.md > /dev/null 2>&1
                set needs_sync 1
            end
        end

        # 检查 references 目录差异
        if test -d $src_skill_dir/references
            if not diff -rq $src_skill_dir/references $dst_skill_dir/references > /dev/null 2>&1
                set needs_sync 1
            end
        end

        if test $needs_sync -eq 1
            # 执行同步：整个 skill 目录
            echo $GREEN"  ↻  $skill_name (有更新)"$RESET

            # 备份目标目录（可选，先直接覆盖）
            # 使用 rsync 保持目录结构
            rsync -a --delete $src_skill_dir/ $dst_skill_dir/

            if test $status -eq 0
                set synced_count (math $synced_count + 1)
                set synced_list "$synced_list\n  $category/$skill_name"
            else
                set error_count (math $error_count + 1)
                echo $RED"  ✗  $skill_name (同步失败)"$RESET
            end
        else
            echo "  ✓  $skill_name (已是最新)"
            set unchanged_count (math $unchanged_count + 1)
        end
    end

    echo ""
end

# 汇总输出
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo $BLUE"  同步完成"$RESET
echo $BLUE"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"$RESET
echo ""
echo $GREEN"已同步: $synced_count 个"$RESET
echo "未更新: $unchanged_count 个"
if test $missing_count -gt 0
    echo $YELLOW"源中缺失: $missing_count 个"$RESET
end
if test $error_count -gt 0
    echo $RED"同步失败: $error_count 个"$RESET
end
echo ""

# 列出已同步的 skill
if test $synced_count -gt 0
    echo $GREEN"已同步列表:"$RESET
    echo $synced_list
    echo ""
end

# 提示 git 操作
if test $synced_count -gt 0
    echo $YELLOW"提示：请在 DiJiang 项目中检查变更并提交 git"$RESET
    echo "  cd ~/Project/DiJiang && git status"
end

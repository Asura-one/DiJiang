<#
.SYNOPSIS
    DiJiang Skill 同步工具 (Windows 版)
.DESCRIPTION
    从 skill-manager 目录同步更新到 DiJiang 项目
    环境: Windows + PowerShell 5.1+
#>

param(
    [switch]$DryRun,
    [switch]$Verbose,
    [switch]$ShowDiff,
    [switch]$NoBackup,
    [switch]$AutoCommit,
    [string]$SourceDir = "$env:USERPROFILE\.skills-manager\skills",
    [string]$TargetDir = "C:\Projects\DiJiang\skills",
    [string[]]$Categories = @("engineering", "productivity"),
    [switch]$Help
)

# ============ 帮助信息 ============
function Show-Help {
    Write-Host "DiJiang Skill 同步工具 (Windows)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "用法: .\sync-from-skills-manager.ps1 [选项]"
    Write-Host ""
    Write-Host "选项:"
    Write-Host "  -DryRun        预览模式，不实际执行同步"
    Write-Host "  -Verbose       详细输出，显示每个文件的变更"
    Write-Host "  -ShowDiff      显示文件内容差异"
    Write-Host "  -NoBackup      不创建备份"
    Write-Host "  -AutoCommit    同步后自动 git commit"
    Write-Host "  -SourceDir     指定源目录 (默认: ~\.skills-manager\skills)"
    Write-Host "  -TargetDir     指定目标目录 (默认: C:\Projects\DiJiang\skills)"
    Write-Host "  -Categories    指定分类目录 (默认: engineering,productivity)"
    Write-Host ""
    Write-Host "示例:"
    Write-Host "  .\sync-from-skills-manager.ps1                    # 正常同步"
    Write-Host "  .\sync-from-skills-manager.ps1 -DryRun           # 预览模式"
    Write-Host "  .\sync-from-skills-manager.ps1 -Verbose -ShowDiff # 详细+差异"
}

if ($Help) {
    Show-Help
    exit 0
}

# ============ 初始化 ============
$ErrorActionPreference = "Stop"

# 统计变量
$script:SyncedCount = 0
$script:UnchangedCount = 0
$script:MissingCount = 0
$script:NewCount = 0
$script:ErrorCount = 0
$script:SyncedList = @()
$script:NewList = @()
$script:MissingList = @()

# ============ 目录检查 ============
if (-not (Test-Path $SourceDir -PathType Container)) {
    Write-Host "✗ 错误：源目录不存在 $SourceDir" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $TargetDir -PathType Container)) {
    Write-Host "✗ 错误：目标目录不存在 $TargetDir" -ForegroundColor Red
    exit 1
}

# 转为绝对路径
$SourceDir = (Resolve-Path $SourceDir).Path
$TargetDir = (Resolve-Path $TargetDir).Path

# ============ 输出函数 ============
function Print-Header {
    Write-Host "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓" -ForegroundColor Blue
    Write-Host "┃" -NoNewline -ForegroundColor Blue
    Write-Host "  DiJiang Skill 同步工具 v2.0 (Win)     " -NoNewline
    Write-Host "┃" -ForegroundColor Blue
    Write-Host "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛" -ForegroundColor Blue
    Write-Host ""
    Write-Host "  源目录:   $SourceDir" -ForegroundColor DarkGray
    Write-Host "  目标目录: $TargetDir" -ForegroundColor DarkGray
    $mode = if ($DryRun) { "预览 (dry-run)" } else { "正常同步" }
    Write-Host "  模式:     $mode" -ForegroundColor DarkGray
    Write-Host ""
}

function Print-Footer {
    Write-Host ""
    Write-Host "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓" -ForegroundColor Blue
    Write-Host "┃" -NoNewline -ForegroundColor Blue
    Write-Host "  同步完成                            " -NoNewline
    Write-Host "┃" -ForegroundColor Blue
    Write-Host "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛" -ForegroundColor Blue
    Write-Host ""
    Write-Host "  ✓ 已同步:   $($script:SyncedCount) 个" -ForegroundColor Green
    Write-Host "  ✓ 未更新:   $($script:UnchangedCount) 个"
    if ($script:NewCount -gt 0) {
        Write-Host "  ✚ 新增:     $($script:NewCount) 个" -ForegroundColor Cyan
    }
    if ($script:MissingCount -gt 0) {
        Write-Host "  ⚠ 源中缺失: $($script:MissingCount) 个" -ForegroundColor Yellow
    }
    if ($script:ErrorCount -gt 0) {
        Write-Host "  ✗ 失败:     $($script:ErrorCount) 个" -ForegroundColor Red
    }
    Write-Host ""

    if ($script:SyncedCount -gt 0) {
        Write-Host "  已同步列表:" -ForegroundColor Green
        $script:SyncedList | ForEach-Object { Write-Host "    $_" }
        Write-Host ""
    }

    if ($script:NewCount -gt 0) {
        Write-Host "  新增列表 (源中有，目标中无):" -ForegroundColor Cyan
        $script:NewList | ForEach-Object { Write-Host "    $_" }
        Write-Host ""
    }

    if ($script:MissingCount -gt 0) {
        Write-Host "  缺失列表 (目标中有，源中无):" -ForegroundColor Yellow
        $script:MissingList | ForEach-Object { Write-Host "    $_" }
        Write-Host ""
    }

    if ($script:SyncedCount -gt 0) {
        Write-Host "  提示: 检查变更并提交" -ForegroundColor Yellow
        Write-Host "    cd (项目根目录) && git status"
    }
}

# ============ 文件对比函数 ============
function Test-DirectoryDifferent {
    param([string]$SrcDir, [string]$DstDir)

    # 获取源目录所有文件
    $srcFiles = Get-ChildItem -Path $SrcDir -Recurse -File -Force | 
        Where-Object { $_.FullName -notmatch '\\\.' -and $_.Name -ne '.DS_Store' }
    
    $hasDiff = $false
    $changeTypes = @()

    foreach ($srcFile in $srcFiles) {
        $relPath = $srcFile.FullName.Substring($SrcDir.Length + 1)
        $dstFile = Join-Path $DstDir $relPath

        # 目标中不存在该文件
        if (-not (Test-Path $dstFile)) {
            $hasDiff = $true
            $changeTypes += "新增: $relPath"
            continue
        }

        # 对比文件内容
        $dstHash = (Get-FileHash $dstFile -Algorithm MD5).Hash
        $srcHash = (Get-FileHash $srcFile.FullName -Algorithm MD5).Hash
        if ($srcHash -ne $dstHash) {
            $hasDiff = $true
            $changeTypes += "修改: $relPath"
        }
    }

    # 检查目标中多余的文件
    if (Test-Path $DstDir) {
        $dstFiles = Get-ChildItem -Path $DstDir -Recurse -File -Force | 
            Where-Object { $_.FullName -notmatch '\\\.' -and $_.Name -ne '.DS_Store' }
        
        foreach ($dstFile in $dstFiles) {
            $relPath = $dstFile.FullName.Substring($DstDir.Length + 1)
            $srcFile = Join-Path $SrcDir $relPath
            if (-not (Test-Path $srcFile)) {
                $hasDiff = $true
                $changeTypes += "删除: $relPath"
            }
        }
    }

    return @{ Different = $hasDiff; Changes = $changeTypes }
}

# ============ 同步函数 ============
function Sync-Directory {
    param([string]$Src, [string]$Dst)
    
    # 创建目标目录
    if (-not (Test-Path $Dst)) {
        New-Item -ItemType Directory -Path $Dst -Force | Out-Null
    }

    #  robocopy 镜像同步
    $robocopyArgs = @(
        $Src,
        $Dst,
        "/MIR",          # 镜像模式
        "/XD", ".git", ".DS_Store",
        "/XF", ".DS_Store", "Thumbs.db",
        "/NFL", "/NDL",  # 不列出文件名/目录名
        "/NJH", "/NJS",  # 不显示头/摘要
        "/NP"            # 不显示进度
    )

    & robocopy @robocopyArgs | Out-Null
    return ($LASTEXITCODE -lt 8)  # robocopy < 8 表示成功
}

# ============ 主逻辑 ============
Print-Header

# 遍历每个分类目录
foreach ($category in $Categories) {
    $categoryDir = Join-Path $TargetDir $category

    if (-not (Test-Path $categoryDir)) {
        Write-Host "⚠  分类目录不存在: $categoryDir，跳过" -ForegroundColor Yellow
        continue
    }

    Write-Host "━━━ [$category] ━━━" -ForegroundColor Cyan

    # 遍历目标分类下的所有 skill
    Get-ChildItem -Path $categoryDir -Directory | ForEach-Object {
        $skillName = $_.Name
        $srcSkillDir = Join-Path $SourceDir $skillName
        $dstSkillDir = $_.FullName

        # 检查源目录中是否存在
        if (-not (Test-Path $srcSkillDir)) {
            Write-Host "  ⚠  $skillName (源目录中不存在)" -ForegroundColor Yellow
            $script:MissingCount++
            $script:MissingList += "$category/$skillName"
            return
        }

        # 对比目录差异
        $result = Test-DirectoryDifferent -SrcDir $srcSkillDir -DstDir $dstSkillDir

        if ($result.Different) {
            Write-Host "  ↻  $skillName" -NoNewline -ForegroundColor Green
            Write-Host " ($($result.Changes.Count) 个变更)" -ForegroundColor DarkGray

            if ($Verbose) {
                $result.Changes | Select-Object -First 10 | ForEach-Object {
                    Write-Host "    $_" -ForegroundColor DarkGray
                }
                if ($result.Changes.Count -gt 10) {
                    Write-Host "    ... 还有 $($result.Changes.Count - 10) 个变更" -ForegroundColor DarkGray
                }
            }

            if ($ShowDiff) {
                Write-Host "    ────────" -ForegroundColor DarkGray
                $skillMdSrc = Join-Path $srcSkillDir "SKILL.md"
                $skillMdDst = Join-Path $dstSkillDir "SKILL.md"
                if ((Test-Path $skillMdSrc) -and (Test-Path $skillMdDst)) {
                    $diff = Compare-Object (Get-Content $skillMdDst) (Get-Content $skillMdSrc) | 
                        Select-Object -First 30
                    $diff | ForEach-Object {
                        $prefix = if ($_.SideIndicator -eq "=>") { "+" } else { "-" }
                        $color = if ($_.SideIndicator -eq "=>") { "Green" } else { "Red" }
                        Write-Host "    $prefix $($_.InputObject)" -ForegroundColor $color
                    }
                }
                Write-Host "    ────────" -ForegroundColor DarkGray
            }

            if (-not $DryRun) {
                # 备份
                if (-not $NoBackup) {
                    $backupDir = "$env:TEMP\skill-sync-backup\$(Get-Date -Format 'yyyyMMdd-HHmmss')\$category\$skillName"
                    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
                    Copy-Item -Path "$dstSkillDir\*" -Destination $backupDir -Recurse -Force
                }

                # 执行同步
                $success = Sync-Directory -Src $srcSkillDir -Dst $dstSkillDir

                if ($success) {
                    $script:SyncedCount++
                    $script:SyncedList += "$category/$skillName"
                } else {
                    $script:ErrorCount++
                    Write-Host "    ✗ 同步失败" -ForegroundColor Red
                }
            } else {
                $script:SyncedCount++
                $script:SyncedList += "$category/$skillName"
            }
        } else {
            if ($Verbose) {
                Write-Host "  ✓  $skill_name"
            }
            $script:UnchangedCount++
        }
    }

    Write-Host ""
}

# 检测新增 skill
Write-Host "━━━ [新增检测] ━━━" -ForegroundColor Cyan

Get-ChildItem -Path $SourceDir -Directory | ForEach-Object {
    $skillName = $_.Name

    # 检查是否在目标中存在
    $found = $false
    foreach ($category in $Categories) {
        $targetPath = Join-Path $TargetDir "$category\$skillName"
        if (Test-Path $targetPath) {
            $found = $true
            break
        }
    }

    if (-not $found) {
        Write-Host "  ✚  $skillName (新增，未纳入项目)" -ForegroundColor Cyan
        $script:NewCount++
        $script:NewList += "$skillName (请手动决定分类)"
    }
}

Write-Host ""

# 自动 commit
if ($AutoCommit -and $script:SyncedCount -gt 0 -and -not $DryRun) {
    Write-Host "━━━ [自动提交] ━━━" -ForegroundColor Cyan
    $projectRoot = Split-Path $TargetDir -Parent
    Push-Location $projectRoot
    git add skills/
    git commit -m "chore: sync skills from skills-manager ($($script:SyncedCount) updated)"
    Pop-Location
    Write-Host ""
}

Print-Footer

# 退出码
if ($script:ErrorCount -gt 0) {
    exit 1
}
exit 0

<#
.SYNOPSIS
    批量 Git Pull 工具 (Windows 版)
.DESCRIPTION
    遍历指定目录下所有 git 仓库，批量执行 git pull
.PARAMETER RootDir
    根目录 (默认: 当前目录)
.PARAMETER VerboseMode
    详细输出，显示完整 git pull 输出
.PARAMETER DryRun
    预览模式，不实际执行
.PARAMETER IncludeHidden
    包含隐藏目录 (默认跳过 . 开头的目录)
.EXAMPLE
    .\git-pull-all.ps1
    .\git-pull-all.ps1 -RootDir D:\Projects
    .\git-pull-all.ps1 -VerboseMode -RootDir C:\Code
#>

param(
    [string]$RootDir = ".",
    [switch]$VerboseMode,
    [switch]$DryRun,
    [switch]$IncludeHidden
)

# ============ 初始化 ============
$ErrorActionPreference = "Continue"

# 转为绝对路径
try {
    $RootDir = (Resolve-Path $RootDir).Path
} catch {
    Write-Host "错误：目录不存在 $RootDir" -ForegroundColor Red
    exit 1
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "  批量 Git Pull 工具 v1.0 (Windows)" -ForegroundColor Blue
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "  目录: $RootDir"
Write-Host ""

# 排除目录
$ExcludeDirs = @(".git", "node_modules", "dist", "build", ".next", "vendor")

# 统计变量
$SuccessCount = 0
$FailCount = 0
$UpToDate = 0
$HasUpdate = 0
$FailList = @()
$UpdateList = @()

# ============ 收集所有 git 仓库 ============
$Repos = @()

Get-ChildItem -Path $RootDir -Directory | ForEach-Object {
    $dir = $_.FullName
    $repoName = $_.Name
    
    # 隐藏目录过滤
    if (-not $IncludeHidden -and $repoName.StartsWith(".")) {
        return
    }
    
    # 排除目录
    if ($ExcludeDirs -contains $repoName) {
        return
    }
    
    # 检查是否是 git 仓库
    $gitDir = Join-Path $dir ".git"
    if (Test-Path $gitDir -PathType Container) {
        $Repos += $dir
    }
}

if ($Repos.Count -eq 0) {
    Write-Host "未找到任何 git 仓库" -ForegroundColor Yellow
    exit 0
}

Write-Host "  发现 $($Repos.Count) 个仓库"
Write-Host ""

# ============ 串行执行 ============
foreach ($dir in $Repos) {
    $repoName = Split-Path $dir -Leaf
    
    if ($DryRun) {
        Write-Host "  [DRY-RUN] $repoName"
        $SuccessCount++
        continue
    }
    
    Push-Location $dir
    try {
        $output = git pull 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    
    if ($exitCode -ne 0) {
        Write-Host "✗  $repoName" -ForegroundColor Red
        if ($VerboseMode) {
            $output | Select-Object -First 10 | ForEach-Object {
                Write-Host "    $_" -ForegroundColor DarkGray
            }
        }
        $FailCount++
        $FailList += $repoName
    } else {
        $outputStr = $output -join "`n"
        if ($outputStr -match "Already up to date" -or $outputStr -match "Already up-to-date") {
            Write-Host "✓  $repoName " -NoNewline
            Write-Host "(已是最新)" -ForegroundColor DarkGray
            $SuccessCount++
            $UpToDate++
        } else {
            Write-Host "↻  $repoName " -NoNewline -ForegroundColor Green
            Write-Host "(已更新)" -ForegroundColor DarkGray
            $SuccessCount++
            $HasUpdate++
            $UpdateList += $repoName
        }
    }
}

# ============ 汇总输出 ============
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "  完成"
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""
Write-Host "  ✓ 成功:   $SuccessCount 个 ($HasUpdate 个有更新, $UpToDate 个已最新)"

if ($UpdateList.Count -gt 0) {
    Write-Host ""
    Write-Host "  已更新列表:" -ForegroundColor Green
    foreach ($repo in $UpdateList) {
        Write-Host "    $repo"
    }
}

if ($FailCount -gt 0) {
    Write-Host ""
    Write-Host "  ✗ 失败:   $FailCount 个" -ForegroundColor Red
    Write-Host "  失败列表:" -ForegroundColor Red
    foreach ($repo in $FailList) {
        Write-Host "    $repo"
    }
}

Write-Host ""
if ($FailCount -gt 0) { exit 1 } else { exit 0 }

# 用高清原图重新生成全部图标资源
#   - 原图：assets/icon-source.png（1240x1240，透明背景，仓库内唯一来源）
#   - 第一步：由原图裁出内容包围盒，生成方形母图 assets/icon-square.png
#     （母图四周留固定边距，作为所有产物的统一起点）
#   - 第二步：用母图生成 MSIX 商店资源 msix/assets/
#     方形资源（StoreLogo / Square*Logo / SmallTile / LargeTile）按画布内缩后等比居中；
#     宽幅资源（Wide310x150Logo / SplashScreen）按高度缩放并水平居中
#   - 第三步：用母图生成 Tauri 图标集 src-tauri/icons/
#     （pnpm tauri icon assets/icon-square.png）
param(
    [string]$Source = '',
    [string]$OutDir = '',
    # 母图四周相对内容包围盒的留白比例
    [double]$Margin = 0.08,
    # 商店资源画布四周留白比例（按画布短边计）
    [double]$Inset = 0.04
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$RepoRoot = Split-Path -Parent $PSScriptRoot
if (-not $Source) { $Source = Join-Path $RepoRoot 'assets\icon-source.png' }
if (-not $OutDir) { $OutDir = Join-Path $RepoRoot 'msix\assets' }
$MasterPath = Join-Path $RepoRoot 'assets\icon-square.png'

if (-not (Test-Path $Source)) { Write-Error "找不到原图: $Source" }
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }

# ---- 读取原图并求内容包围盒（只看 alpha，避免把透明边当内容）----
$src = New-Object System.Drawing.Bitmap $Source
$minX = $src.Width; $minY = $src.Height; $maxX = -1; $maxY = -1
for ($y = 0; $y -lt $src.Height; $y++) {
    for ($x = 0; $x -lt $src.Width; $x++) {
        if ($src.GetPixel($x, $y).A -gt 8) {
            if ($x -lt $minX) { $minX = $x }
            if ($x -gt $maxX) { $maxX = $x }
            if ($y -lt $minY) { $minY = $y }
            if ($y -gt $maxY) { $maxY = $y }
        }
    }
}
$content = New-Object System.Drawing.Rectangle $minX, $minY, ($maxX - $minX + 1), ($maxY - $minY + 1)
Write-Host "原图内容包围盒: $($content.Width) x $($content.Height) @ ($minX,$minY)"

# ---- 第一步：生成方形母图 ----
$masterSide = [int][Math]::Max($content.Width, $content.Height) * (1 + $Margin)
$masterSide = [int][Math]::Round($masterSide)
$master = New-Object System.Drawing.Bitmap $masterSide, $masterSide, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$master.SetResolution(96, 96)
$mg = [System.Drawing.Graphics]::FromImage($master)
$mg.Clear([System.Drawing.Color]::Transparent)
$mg.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$mg.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$mg.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
$mg.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
$masterDrawW = [int][Math]::Round($content.Width * ($masterSide / ($content.Width * (1 + $Margin))))
$masterDrawH = [int][Math]::Round($content.Height * ($masterSide / ($content.Width * (1 + $Margin))))
$masterDest = New-Object System.Drawing.Rectangle `
    ([int][Math]::Round(($masterSide - $masterDrawW) / 2)), `
    ([int][Math]::Round(($masterSide - $masterDrawH) / 2)), $masterDrawW, $masterDrawH
$mg.DrawImage($src, $masterDest, $content, [System.Drawing.GraphicsUnit]::Pixel)
$mg.Dispose()
$master.Save($MasterPath, [System.Drawing.Imaging.ImageFormat]::Png)
$master.Dispose()
$src.Dispose()
Write-Host "母图: $MasterPath ($masterSide x $masterSide)"

# ---- 渲染一个尺寸（源为母图）----
$base = New-Object System.Drawing.Bitmap $MasterPath
function Render-Icon([int]$width, [int]$height, [string]$path) {
    $canvas = New-Object System.Drawing.Bitmap $width, $height, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $canvas.SetResolution(96, 96)
    $g = [System.Drawing.Graphics]::FromImage($canvas)
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality

    $inset = [int][Math]::Round([Math]::Min($width, $height) * $Inset)
    $innerW = [Math]::Max(1, $width - 2 * $inset)
    $innerH = [Math]::Max(1, $height - 2 * $inset)
    $scale = [Math]::Min($innerW / $base.Width, $innerH / $base.Height)
    $drawW = [int][Math]::Round($base.Width * $scale)
    $drawH = [int][Math]::Round($base.Height * $scale)
    $dest = New-Object System.Drawing.Rectangle `
        ([int][Math]::Round(($width - $drawW) / 2)), `
        ([int][Math]::Round(($height - $drawH) / 2)), $drawW, $drawH
    $g.DrawImage($base, $dest)
    $g.Dispose()
    $canvas.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $canvas.Dispose()
}

# ---- 第二步：资源规格（与原 WinUI 3 项目的 MRT scale 表一致）----
$scaleSets = @(
    @{ Name = 'StoreLogo';        W = 50;  H = 50 },
    @{ Name = 'Square44x44Logo';  W = 44;  H = 44 },
    @{ Name = 'Square150x150Logo';W = 150; H = 150 },
    @{ Name = 'Wide310x150Logo'; W = 310; H = 150 },
    @{ Name = 'SmallTile';        W = 71;  H = 71 },
    @{ Name = 'LargeTile';        W = 310; H = 310 },
    @{ Name = 'SplashScreen';     W = 620; H = 300 }
)
$scales = @(100, 125, 150, 200, 400)

foreach ($set in $scaleSets) {
    foreach ($scale in $scales) {
        $w = [int][Math]::Round($set.W * $scale / 100)
        $h = [int][Math]::Round($set.H * $scale / 100)
        Render-Icon $w $h (Join-Path $OutDir "$($set.Name).scale-$scale.png")
    }
    # 清单按裸文件名引用资源：补一份 scale-100 的同名副本
    Render-Icon $set.W $set.H (Join-Path $OutDir "$($set.Name).png")
}

# ---- Square44x44Logo 的 targetsize / altform 变体（任务栏与列表小图标）----
foreach ($size in @(16, 24, 32, 48, 256)) {
    Render-Icon $size $size (Join-Path $OutDir "Square44x44Logo.targetsize-$size.png")
}
Render-Icon 24 24 (Join-Path $OutDir 'Square44x44Logo.targetsize-24_altform-unplated.png')
foreach ($size in @(16, 32, 48, 256)) {
    Render-Icon $size $size (Join-Path $OutDir "Square44x44Logo.altform-unplated_targetsize-$size.png")
}
foreach ($size in @(16, 24, 32, 48, 256)) {
    Render-Icon $size $size (Join-Path $OutDir "Square44x44Logo.altform-lightunplated_targetsize-$size.png")
}

$base.Dispose()
Write-Host ""
Write-Host "完成，资源输出到 $OutDir" -ForegroundColor Green
Get-ChildItem -LiteralPath $OutDir -Filter '*.png' -File | Measure-Object | ForEach-Object { "共 $($_.Count) 个 PNG" }
Write-Host "接下来执行：pnpm tauri icon assets/icon-square.png"

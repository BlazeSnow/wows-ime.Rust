<#
.SYNOPSIS
  将 package.json 中的版本号同步到 tauri.conf.json、Cargo.toml 与 Cargo.lock。

.DESCRIPTION
  版本号只维护在仓库根目录的 package.json 中，避免多处手改导致不一致：
    - version：软件版本（格式 X.Y.Z 或 X.Y.Z-beta.N，如 2026.10.20、1.0.0-beta.1）

  本项目只发布到微软商店（msixbundle），不做 MSI 打包，因此没有 msiVersion，
  也不写 tauri.conf.json 的 bundle.windows.wix.version。

  本脚本写入：
    - tauri.conf.json 的 version（应用版本）
    - src-tauri/Cargo.toml 中 [package] 段的 version（Rust 后端版本）
    - src-tauri/Cargo.lock 中根包的 version（保持锁文件一致）

  脚本只操作自身所在的目录，因此应复制到目标仓库根目录后运行。
  根包名从 Cargo.toml 的 [package] 段读取，无需随项目修改本脚本。

.EXAMPLE
  .\version.ps1        # 按 package.json 同步各版本位置
#>

$ErrorActionPreference = 'Stop'

$RepoRoot = $PSScriptRoot
$PackagePath = Join-Path $RepoRoot 'package.json'
$TauriConfPath = Join-Path $RepoRoot 'src-tauri\tauri.conf.json'
$CargoTomlPath = Join-Path $RepoRoot 'src-tauri\Cargo.toml'
$CargoLockPath = Join-Path $RepoRoot 'src-tauri\Cargo.lock'

foreach ($path in @($PackagePath, $TauriConfPath, $CargoTomlPath, $CargoLockPath)) {
    if (-not (Test-Path -LiteralPath $path)) { throw "缺少文件：$path" }
}

# 1. 读取并校验 package.json 中的版本字段
$package = Get-Content $PackagePath -Raw -Encoding UTF8 | ConvertFrom-Json

$version = $package.version
if (-not $version) {
    throw 'package.json 中缺少 version 字段。'
}
if ($version -notmatch '^\d+\.\d+\.\d+(-beta\.\d+)?$') {
    throw "package.json 的 version 格式不正确：$version（应为 X.Y.Z 或 X.Y.Z-beta.N，例如 2026.10.20、1.0.0-beta.1）"
}

# 2. 读取 Cargo.toml 的 [package] 段，取得根包名（Cargo.lock 中据此定位根包）
$cargoTomlText = [System.IO.File]::ReadAllText($CargoTomlPath)
$packageSection = [regex]::Match($cargoTomlText, '(?ms)^\[package\]\s*$.*?(?=^\[|\z)')
if (-not $packageSection.Success) {
    throw "未在 $CargoTomlPath 中找到 [package] 段。"
}
$rootPackage = [regex]::Match($packageSection.Value, '(?m)^name\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $rootPackage) {
    throw "未在 $CargoTomlPath 的 [package] 段中找到 name。"
}
if ($packageSection.Value -notmatch '(?m)^version\s*=\s*"') {
    throw "未在 $CargoTomlPath 的 [package] 段中找到 version。"
}

# 3. 同步 tauri.conf.json 的 version
$nodeScript = @'
const fs = require('fs');
const ver = process.argv[1];
const t = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
t.version = ver;
fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(t, null, 2) + String.fromCharCode(10));
'@

Push-Location $RepoRoot
try {
    node -e $nodeScript $version
    if ($LASTEXITCODE -ne 0) { throw '同步版本到 tauri.conf.json 失败' }
}
finally {
    Pop-Location
}

# 4. 同步 Cargo.toml 的 [package] version 与 Cargo.lock 中根包的 version。
#    Cargo.toml 只在 [package] 段内替换，避免误改 [dependencies.*] 等子表中同样以
#    version = 开头的行；Cargo.lock 中其他包的 version 行同理不能动，因此按
#    「[[package]] + name 行 + 紧跟的 version 行」定位根包。
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

$newSection = [regex]::Replace($packageSection.Value, '(?m)(^version\s*=\s*")[^"]*"', ('${1}' + $version + '"'))
$newToml = $cargoTomlText.Remove($packageSection.Index, $packageSection.Length).Insert($packageSection.Index, $newSection)
[System.IO.File]::WriteAllText($CargoTomlPath, $newToml, $utf8NoBom)

$lockText = [System.IO.File]::ReadAllText($CargoLockPath)
$lockPattern = '(?ms)(^\[\[package\]\]\r?\nname = "' + [regex]::Escape($rootPackage) + '"\r?\nversion = ")[^"]*"'
if (-not [regex]::IsMatch($lockText, $lockPattern)) {
    throw "未在 $CargoLockPath 中找到根包 $rootPackage 的 version 行。"
}
$newLock = [regex]::Replace($lockText, $lockPattern, ('${1}' + $version + '"'))
[System.IO.File]::WriteAllText($CargoLockPath, $newLock, $utf8NoBom)

Write-Host "已同步 tauri.conf.json / Cargo.toml / Cargo.lock（version=$version）"

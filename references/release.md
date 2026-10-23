# 版本与发布

## 版本号

只维护在根目录 `package.json`，其余位置由 `version.ps1` 同步：

| 位置 | 字段 | 谁改 |
| --- | --- | --- |
| `package.json` | `version` | 手动 |
| `src-tauri/tauri.conf.json` | `version` | `version.ps1` |
| `src-tauri/Cargo.toml` | `[package].version` | `version.ps1` |
| `src-tauri/Cargo.lock` | 根包 `version` | `version.ps1` |

| 字段 | 格式 | 说明 |
| --- | --- | --- |
| `version` | `X.Y.Z` 或 `X.Y.Z-beta.N` | 软件版本 |

**本项目只发布到微软商店，不做 MSI 打包**，因此没有 `msiVersion`（MSI 发布序列号），
也没有 `baseVersion`（版本线基线），`tauri.conf.json` 里也没有 `bundle.windows.wix` 段。
`version.ps1` / `tag.ps1` 已按此裁剪：只同步与校验上表中的四处 `version`。

商店包的版本另有要求：包版本必须是四段式 `x.y.z.w`。本项目用日期式三段版本
（如 `2026.10.20`），打包时自动补 `.0` 得到 `2026.10.20.0`；tag 同样按四段式打，
即 `v2026.10.20.0`。

## 发布流程

1. 在 `package.json` 手动修改 `version`。
2. 运行 `version.ps1` 把版本号同步到其余三处。
3. 更新更新日志（`CHANGELOG.md` 已迁至官网 <https://wows.blazesnow.com/CHANGELOG>，本仓库只留跳转说明）。
4. 提交改动。
5. 运行 `tag.ps1`：校验各版本文件与 `package.json` 一致（不一致则拒绝），确认后打 tag 并推送，触发发布工作流。

tag 格式为 `v` + `version`，三段式自动补 `.0`（如 `v2026.10.20.0`）；含 `-beta` 的 tag 在 GitHub 上标记为 Prerelease。

## 分发方式

**只做 msixbundle 微软商店分发**，不做其它打包形式。打包脚本为 `scripts/make-msix.ps1`（`pnpm msix`）。

应用仅支持 Windows：商店包的身份沿用原 WinUI 3 版本（`BlazeSnow.WOWS-IME`，发布者 `CN=C171AF55-419C-4E73-B34E-CB98C8F1EB78`，商店产品 ID `9P0R5GM2PTKW`），商店列表得以延续。

### 打包结构

| 位置 | 内容 |
| --- | --- |
| `msix/AppxManifest.template.xml` | 清单模板，`@VERSION@` 与 `@ARCH@` 由脚本替换 |
| `msix/assets/` | 商店图标资源（含 `scale` / `targetsize` / `altform-unplated` 全部变体与裸文件名副本） |
| `scripts/make-msix.ps1` | 逐架构 `makeappx pack` → `makeappx bundle` |
| `.github/workflows/release.yml` | 推送 `v*` tag 时构建 x64 + arm64 并发布 bundle |

打包脚本默认从 `package.json` 取版本号并补足四段式（`2026.10.20` → `2026.10.20.0`），也可用 `-Version` 显式指定。`makeappx` 与 `makepri` 来自 Windows SDK，脚本会自行扫描所有固定磁盘下的 `Windows Kits\10\bin` 定位，不假设安装盘符。

单架构与多架构：

```powershell
scripts/make-msix.ps1                                  # x64
scripts/make-msix.ps1 -TargetArch x64,arm64 `
    -RustTarget x86_64-pc-windows-msvc,aarch64-pc-windows-msvc
```

`makeappx bundle` 必须显式传 `/bv`（bundle 版本），否则会用打包时刻的 UTC 时间生成版本号，导致商店显示版本与包内实际版本不一致。

### 清单编码

`msix/AppxManifest.template.xml` 是 UTF-8（无 BOM）文本，打包脚本读它时必须显式 `-Encoding UTF8`：
Windows PowerShell 5.1 的 `Get-Content` 默认按系统 ANSI 代码页读取（简体中文机器为 GBK），
会把清单里的中文读成乱码，乱码字节甚至会吃掉 XML 结束标签的 `<`，产出无法解析的清单。

脚本写清单时用 .NET API 固定写 UTF-8 无 BOM：PS 5.1 的 `Set-Content -Encoding UTF8` 会带 BOM 而 PS 7 不会，
固定编码才能保证本地打包与 CI（`shell: pwsh`）产出完全一致。

## 打包前检查

```powershell
cargo test          # 后端测试
pnpm test           # 前端测试
pnpm check:locales  # 三语资源一致性
pnpm build:web      # 前端构建（含 tsc 类型检查）
```

四项必须全部通过再打包。

## 相关脚本

| 脚本 | 作用 |
| --- | --- |
| `version.ps1` | 按 `package.json` 同步版本号 |
| `tag.ps1` | 版本一致性校验 + 打 tag 推送 |
| `run.ps1` | 一键运行（开发 / 发布 / 打包） |
| `scripts/make-msix.ps1` | 打包 msixbundle |
| `scripts/make-store-assets.ps1` | 由高清原图重新生成商店图标资源 |

## 图标资源

图标由 `assets/icon-source.png`（唯一来源，1240×1240 透明背景）统一生成，改图只改这一张：

```powershell
pnpm icons     # 生成方形母图 + 商店资源 + Tauri 图标集
```

生成物：`assets/icon-square.png`（方形母图）、`msix/assets/`（商店资源全变体）、`src-tauri/icons/`（Tauri 图标集）。详见 [implementation.md](implementation.md)。

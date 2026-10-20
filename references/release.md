# 版本与发布

## 版本号

只维护在根目录 `package.json`，其余位置由 `version.ps1` 同步：

| 位置 | 字段 | 谁改 |
| --- | --- | --- |
| `package.json` | `version` | 手动 |
| `package.json` | `msiVersion` | 手动 |
| `package.json` | `baseVersion` | 手动 |
| `src-tauri/tauri.conf.json` | `version` | `version.ps1` |
| `src-tauri/tauri.conf.json` | `bundle.windows.wix.version` | `version.ps1` |
| `src-tauri/Cargo.toml` | `[package].version` | `version.ps1` |
| `src-tauri/Cargo.lock` | 根包 `version` | `version.ps1` |

字段含义：

| 字段 | 格式 | 说明 |
| --- | --- | --- |
| `version` | `X.Y.Z` 或 `X.Y.Z-beta.N` | 软件版本 |
| `msiVersion` | 三段纯数字 | MSI 发布序列号，固定 `1.0.N`（N 为第 N 次发布） |
| `baseVersion` | `vX.Y.Z` | 目标版本线，约束 `version` 的基线 |

`msiVersion` 与软件版本脱钩的原因：MSI 版本比较只取前三段且不允许字母，无法表达 `-beta.N` 语义；用发布次数绑定序列号（`1.0.N`）可保证单调递增，旧版可直接覆盖升级。

## 发布流程

1. 在 `package.json` 手动修改 `version` 与 `msiVersion`（`msiVersion` 每次发布递增）。
2. 运行 `version.ps1` 把版本号同步到其余位置。
3. 更新 `CHANGELOG.md`。
4. 提交改动。
5. 运行 `tag.ps1`：校验各版本文件与 `package.json` 一致（不一致则拒绝），确认后打 tag 并推送，触发发布工作流。

tag 格式为 `v` + `version`（如 `v1.0.0`、`v1.0.0-beta.1`）；含 `-beta` 的 tag 在 GitHub 上标记为 Prerelease。

## 分发方式

**只做 msixbundle 微软商店分发**，不做其它打包形式。打包脚本为 `scripts/make-msix.ps1`（`pnpm msix`）。

应用仅支持 Windows：商店包的身份为 `BlazeSnow.WOWS-IME`，发布者 `CN=C171AF55-419C-4E73-B34E-CB98C8F1EB78`，与原 WinUI 3 版本一致，商店产品 ID 沿用原版（`9P0R5GM2PTKW`）。

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

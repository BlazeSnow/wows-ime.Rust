# 实现记录

按里程碑记录本仓库的实现决策与取舍。编辑历史以 git 提交信息为准，本文只保留「为什么这么做」。

## M0 · 仓库初始化

- Git 配置：`*.ps1` 用 CRLF、其余文本统一 LF；图片类资产走 Git LFS（`.gitattributes`）。
- 入口脚本：`run.ps1`（一键运行）、`version.ps1`（版本同步）、`tag.ps1`（打 tag 发布），均取自软件开发规范模板。
- 版本号只维护在 `package.json`，`version.ps1` 负责同步到 `tauri.conf.json` / `Cargo.toml` / `Cargo.lock`。

## M1 · 脚手架

- 技术栈：Tauri 2 + Rust + React + TypeScript + Vite + shadcn/ui（Tailwind CSS v4）+ SQLite（rusqlite，bundled）。
- 参考 `MessagesEncrypter.WinUI → MessagesEncrypter.Rust` 的迁移结构：前端 `src/`、后端 `src-tauri/`、`references/` 文档索引、`tools/check-locales.mjs` 本地化校验。
- 图标：用原 WinUI 3 项目的 `Square150x150Logo` 生成 Tauri 全套图标（`pnpm tauri icon`）；应用仅支持 Windows，移动端图标不入库。
- 窗口：默认隐藏，`setup` 中恢复几何并钳制到工作区后再显示，避免启动闪窗。

## M2 · 后端核心

### TSF 扫描

- 原版直接走 vtable 裸调 COM 接口；本版改用 `windows` crate 的 `Win32_UI_TextServices` 绑定（`ITfInputProcessorProfiles`），语义更安全且不必手写 vtable 索引。
- 已核对 `windows` crate 的 vtable 顺序与原版硬编码的索引一致（`GetLanguageList`=15、`EnumLanguageProfiles`=16、`GetLanguageProfileDescription`=12、`IsEnabledLanguageProfile`=18），行为与原版等价。
- 语言 ID、分类关键词、噪音项过滤、警告码全部沿用原版常量，警告码是前端文案契约。

### 配置文件生成

- 原版用 `XDocument` 生成，本版手写字符串序列化：结构、缩进、分组顺序、`Tag` 取值全部对齐原版。
- 明确 UTF-8 无 BOM（BOM 会让部分解析器读不到根元素）。
- 名称做 XML 转义，避免破坏文档结构。

### 持久化

- 原版用 `ApplicationData.Current.LocalSettings` + `Microsoft.Data.Sqlite` 两处存储；本版统一为单一 SQLite：新增 `settings` 键值表承接「简单状态」，列表表结构与原版一致。
- 旧 `config.json` 迁移按原版规则执行，仍用「改名为 `config.json.migrated`」作为已迁移标记，不额外写标记位。
- 所有持久化异常静默吞掉，与原版一致。

### 警告码与异常边界

- 原版的 COM 调用失败会以 `COMException` 抛出并被捕获，上报 `Tsf/ComException`。Rust 侧 COM 调用返回 `Result`，因此改为在枚举语言配置文件失败时上报 `COM_EXCEPTION_WARNING`（带 HRESULT），语义等价。
- 原版的兜底 `catch (Exception)` 对应本版的 `Tsf/GenericException`：扫描命令用 `catch_unwind` 捕获未预期的 panic，转成该警告码，保证「扫描异常不中断界面」。
- **因此 release profile 不能用 `panic = "abort"`**：abort 模式下 `catch_unwind` 失效，panic 会直接终止进程，该保证形同虚设。`Cargo.toml` 的 release profile 保留 `panic = "unwind"`，并由 `panic_is_catchable_in_this_profile` 用例（需用 `cargo test --release` 运行）守住这条约束。

### 依赖与索引镜像

- 本机直连 `index.crates.io:443` 会超时（`static.crates.io` 反而可达），因此 cargo 拉不到 crates.io 稀疏索引。已在 **cargo 全局配置**（`$CARGO_HOME/config.toml`）里配置 rsproxy 镜像做 source replacement：

  ```toml
  [source.crates-io]
  replace-with = 'rsproxy-sparse'
  [source.rsproxy-sparse]
  registry = "sparse+https://rsproxy.cn/index/"
  ```

  这属于**机器级网络配置**，不进仓库——CI 与其它机器不需要，且镜像同步有延迟。
- 注意 cargo 只读 `$CARGO_HOME/config.toml`。若本机 `CARGO_HOME` 与用户目录不一致（例如 scoop 装 Rust 时指向 `D:\...\rustup\.cargo`），写进 `C:\Users\<用户>\.cargo\` 是不生效的。
- 镜像配好后 `cargo update` / `cargo build` 均可正常联网执行；`cargo update` 会先拉一次索引，约十几秒。
- 排查手段：直接用 TCP 探活（`Test-NetConnection index.crates.io -Port 443`）比 cargo 的报错更能定位是网络不通还是 cargo 配置问题。

## M3 · 前端

- 三步向导用组件内 `step` 状态驱动，不用路由，与原版 `NavigationView` + 页面切换的行为对齐。
- 文案资源由原版三份 `Resources.resw` 转换生成（93 键 × 3 语种），键名与文案完全沿用，避免人工转录误差。
- i18next 的 `keySeparator` / `nsSeparator` 关闭（键名含 `.` 与 `/`），插值分隔符改为单花括号以匹配原版 `{0}` 风格。
- 语言切换改为立即生效（原版需重启应用），因为 i18next 支持运行时切换。
- 状态反馈统一用 `sonner` 的 toast。

## M6 · 打包与发布

- 商店包身份沿用原 WinUI 3 版本的 `BlazeSnow.WOWS-IME`，商店列表得以延续；打包脚本与清单模板参照 `MessagesEncrypter` 的成熟方案移植。
- 商店图标资源直接复用原 WinUI 3 项目的 `Assets`（含 `scale` / `targetsize` / `altform-unplated` 全部变体），并补齐清单按裸文件名引用所需的副本。
- `make-msix.ps1` 默认从 `package.json` 取版本号并补足四段式；`makeappx bundle` 必须显式传 `/bv`，否则 bundle 版本会取打包时刻的 UTC 时间，与包内版本不一致。
- GitHub Actions 在推送 `v*` tag 时构建 x64 + arm64 并发布 `.msixbundle`。

## 图标资源

图标改为新的高清 IME 字样 artwork（原 WinUI 3 版用的是游戏锚形徽标）。仓库内保留唯一来源，可完整复现：

| 位置 | 作用 |
| --- | --- |
| `assets/icon-source.png` | 原始高清图（1240×1240，透明背景），唯一来源，改图只改这里 |
| `assets/icon-square.png` | 生成物：按内容包围盒裁出的方形母图 |
| `msix/assets/` | 生成物：MSIX 商店资源（scale / targetsize / altform 全变体） |
| `src-tauri/icons/` | 生成物：Tauri 图标集（含 Appx 徽标、ico、icns） |

重新生成：

```powershell
pnpm icons     # = icons:store + tauri icon assets/icon-square.png
```

- `scripts/make-store-assets.ps1` 先由原图求内容包围盒（只看 alpha），生成带 8% 边距的方形母图，再用母图渲染各规格资源；方形资源等比缩放到画布内，宽幅资源按高度缩放并水平居中。
- 母图统一后，Tauri 图标集与商店资源的留白比例完全一致，不会出现两处图标大小不同的情况。
- 原 WinUI 3 版的 MRT scale 表（50/63/75/100/200、44/55/66/88/176、150/188/225/300/600 等）逐项沿用，只换图像来源。

## 待办

- msixbundle 实际打包未实测：本机未安装 Windows SDK（缺 `makeappx` / `makepri`）。
- 旧 WinUI 3 版本真实数据的迁移实测。

## 脚本编码约定

仓库内所有 `.ps1` 脚本必须满足三点，否则在 PowerShell 5.1 下会出怪问题：

1. **UTF-8 带 BOM**：无 BOM 时 PowerShell 5.1 按系统 ANSI 代码页（本机为 GBK）解码，中文注释会被误解码，轻则报错、重则静默改变脚本语义。
2. **CRLF 换行**：与 `.gitattributes` 的 `*.ps1 text eol=crlf` 一致。
3. **首字符不可丢**：规范化编码时要用「按字节剥离 BOM 三字节再解码」的方式，**不要**用 `Substring(1)` 之类的字符串截断——一旦判断条件与实际不符，会把 BOM 后的第一个字符（常常是 `#` 或 `<`）吃掉，脚本会以莫名其妙的语法错误告终。

校验方式（每次改动脚本后执行）：

```powershell
foreach ($f in (Get-ChildItem -Recurse -Filter *.ps1)) {
    $errors = $null
    [void][System.Management.Automation.Language.Parser]::ParseFile($f.FullName, [ref]$null, [ref]$errors)
    if ($errors) { Write-Error "$($f.Name): $($errors[0].Message)" }
}
```

只做语法解析不够——首字符被吃掉时 `run.ps1` 仍能解析通过，但运行时会报「xx 不是可识别的 cmdlet」。改完脚本务必实跑一次。

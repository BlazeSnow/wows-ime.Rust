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

### 依赖与离线构建

- 本机 crates.io 索引更新会长时间挂起，`cargo build --offline` 可用（本地已有完整缓存）。新增依赖时优先选已在缓存中的版本。

## M3 · 前端

- 三步向导用组件内 `step` 状态驱动，不用路由，与原版 `NavigationView` + 页面切换的行为对齐。
- 文案资源由原版三份 `Resources.resw` 转换生成（93 键 × 3 语种），键名与文案完全沿用，避免人工转录误差。
- i18next 的 `keySeparator` / `nsSeparator` 关闭（键名含 `.` 与 `/`），插值分隔符改为单花括号以匹配原版 `{0}` 风格。
- 语言切换改为立即生效（原版需重启应用），因为 i18next 支持运行时切换。
- 状态反馈统一用 `sonner` 的 toast。

## 待办

- msixbundle 打包脚本与发布工作流（M6）。
- 旧 WinUI 3 版本真实数据的迁移实测。
- `CHANGELOG.md` 与 `README.md`（面向用户）待补。

# references 索引

本目录是 wows-ime（Tauri 2 重构版）的详细规范与实现记录。`DEVELOPMENT.md` 只保留索引与速查。

## 阅读顺序

1. [features.md](features.md) — 产品做什么、用户流程是什么
2. [ui.md](ui.md) — 界面结构与交互约定
3. [game-config.md](game-config.md) — `ime_config.xml` 的生成规则（最硬的兼容约束）
4. [scanning.md](scanning.md) — 输入法扫描（TSF/COM）
5. [storage.md](storage.md) — 持久化与旧数据迁移
6. [i18n.md](i18n.md) — 多语言
7. [release.md](release.md) — 版本号与发布
8. [testing.md](testing.md) — 测试基线
9. [implementation.md](implementation.md) — 实现记录与决策

## 重构总原则

1. **行为兼容优先**：本项目是既有 WinUI 3 应用的替换品，用户可见行为、生成的文件格式、用户数据都要与原版兼容；技术栈可以换，行为不能悄悄变。
2. **原版是基线**：原版 WinUI 3 源码是行为规格的最终依据。移植时先读原版实现，再动手写 Rust / React 代码；原版的注释与常量（如游戏路径、警告码、分类关键词）都是兼容契约。
3. **单一事实源**：版本号只在 `package.json`，其余位置由 `version.ps1` 同步；文案只在 `src/locales/`，三语键集合由 `check:locales` 校验。
4. **平台边界清晰**：应用仅支持 Windows。输入法扫描依赖 COM/TSF，不得引入跨平台依赖；移动端图标等资产不入库。
5. **持久化失败不阻断流程**：与原版一致，存储异常一律静默吞掉，不影响主流程与启动。
6. **文档随代码更新**：每完成一项功能，把细节与决策写进对应主题文档，不在提交信息之外另建开发日志。

## 术语

| 术语 | 含义 |
| --- | --- |
| 原 WinUI 3 版本 | 本仓库的前身，基于 WinUI 3 / Windows App SDK 的桌面应用 |
| 游戏根目录 | 用户选择的游戏安装目录，其下有 `bin` 与游戏可执行文件 |
| 目标配置文件 | `<游戏根目录>\bin\<数字版本>\res_mods\ime_config.xml` |
| 输入法分类 | 简体中文 / 繁体中文 / 日文，决定写入哪个 `<Tag>` 分组 |

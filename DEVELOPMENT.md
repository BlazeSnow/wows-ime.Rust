# DEVELOPMENT

wows-ime（Tauri 2 重构版）的开发索引。**本文件只作索引与速查，详细规范与实现记录全部在 [references/](references/README.md)。**

- 根目录 `AGENTS.md` 是最高优先级约束，禁止修改；与本文件冲突时以 AGENTS.md 为准。
- **约定：每完成一项功能，细节与决策写入 [references/implementation.md](references/implementation.md)；编辑历史以 git 提交信息为准，不再维护开发日志。**

## 1. 项目定位

本仓库是 wows-ime 的 Tauri 2 重构版（原版为 WinUI 3 应用）。产品定位、红线与对外承诺见 [references/product-spec.md](references/product-spec.md)。

## 2. 技术栈

| 层 | 选型 | 备注 |
| --- | --- | --- |
| 应用框架 | Tauri 2 | 仅支持 Windows |
| 后端 | Rust | 输入法扫描、游戏配置读写、持久化 |
| 前端 | React + TypeScript + Vite；组件库 shadcn/ui（Tailwind CSS v4） | |
| 数据库 | SQLite（rusqlite，bundled） | 自定义游戏路径与自定义输入法 |
| 多语言 | 前端 i18next；后端 fluent-i18n | zh-Hans（默认）/ zh-Hant / ja |
| 测试 | cargo test；vitest + @testing-library | `cargo test` / `pnpm test` |
| 包管理 | 前端 pnpm | |

## 3. 常用命令

```bash
pnpm install          # 安装前端依赖
.\run.ps1             # 开发运行（或根目录 run.ps1）
pnpm build            # 构建发布 exe
pnpm test             # 前端测试
cargo test            # 后端测试（src-tauri）
node tools/check-locales.mjs   # 本地化一致性校验（npm run check:locales）
scripts/make-msix.ps1 # 打包 msixbundle（微软商店，唯一分发方式）
version.ps1 / tag.ps1 # 版本同步 / 打 tag 触发发布
```

改动交付前必须全部通过：`cargo test`、`pnpm test`、`check:locales`、`pnpm build`。

## 4. 硬性约束（速查）

1. 生成的 `ime_config.xml` 与原 WinUI 3 版本兼容（UTF-8 无 BOM、分组顺序一致）→ [references/game-config.md](references/game-config.md)
2. 原版用户数据可迁移（`config.json`、游戏路径、自定义输入法）→ [references/storage.md](references/storage.md)
3. 用户可见文本零硬编码（zh-Hans / zh-Hant / ja，键集合与占位符一致）→ [references/i18n.md](references/i18n.md)
4. 输入法扫描走 TSF/COM，仅 Windows 可用 → [references/scanning.md](references/scanning.md)
5. 只做 msixbundle 微软商店分发，不做其他打包 → [references/release.md](references/release.md)

## 5. 文档索引

| 文档 | 内容 |
| --- | --- |
| [references/README.md](references/README.md) | 索引与重构总原则 |
| [references/features.md](references/features.md) | 功能与用户流程 |
| [references/ui.md](references/ui.md) | 界面与交互 |
| [references/game-config.md](references/game-config.md) | 游戏配置文件生成规则 |
| [references/scanning.md](references/scanning.md) | 输入法扫描（TSF/COM） |
| [references/storage.md](references/storage.md) | 持久化与数据迁移 |
| [references/i18n.md](references/i18n.md) | 多语言规范 |
| [references/release.md](references/release.md) | 版本号规则与发布流程 |
| [references/testing.md](references/testing.md) | 测试基线与兼容验收清单 |
| [references/implementation.md](references/implementation.md) | **Tauri 2 实现记录与决策（重构版细节都在这里）** |

文档中禁止出现原项目的本机绝对路径；需要指代时用「原 WinUI 3 版本」或公开 URL。

## 6. 里程碑

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| M0 | 仓库初始化（Git 配置、脚本、文档骨架） | ✅ |
| M1 | 脚手架：Tauri 2 + React + shadcn/ui | ✅ |
| M2 | 后端核心：输入法扫描、游戏配置生成、持久化 | ✅ |
| M3 | 前端页面：Home 三步向导 + 设置页 | ✅ |
| M4 | 多语言与本地化一致性校验 | ✅（三语 92 键，校验通过） |
| M5 | 数据迁移与兼容验收 | ✅（迁移已实现并有单测；真机旧版数据待实测） |
| M6 | msixbundle 打包与发布流程 | ✅（脚本、工作流、商店资源就绪；本机缺 Windows SDK 未实测打包） |

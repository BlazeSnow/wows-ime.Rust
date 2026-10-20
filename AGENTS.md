# AI 编程指导

1. 本文件是 AI 编程的基准指导，默认禁止修改；只有用户明确授权时才可以修订
2. 所有文件都以 UTF-8 格式存储
3. 实际开发中需要注意终端的 GBK 与 UTF-8 的问题
4. 业务逻辑、持久化、架构、构建发布、本地化等详细内容见 [DEVELOPMENT.md](DEVELOPMENT.md)；DEVELOPMENT.md 鼓励随开发进度同步修改，代码或流程变更时无需额外授权即可更新

## 项目简介

本程序是《战舰世界》中文输入法配置文件的修改器（Tauri 2 桌面应用），通过修订游戏配置文件 `ime_config.xml`，让游戏支持更多的中文输入法。原版为 WinUI 3 应用，本仓库是其 Tauri 2 重构版，最终发布至 Microsoft Store。

## 技术栈

1. 应用框架：Tauri 2，仅支持 Windows
2. 后端：Rust
3. 前端：React + TypeScript + Vite，组件库 shadcn/ui（Tailwind CSS v4）
4. 数据库：SQLite（rusqlite，bundled）
5. 多语言：后端 fluent-i18n，前端 i18next；支持简体中文、繁体中文、日语
6. 分发：msixbundle（微软商店，唯一分发方式）

## 关键约定

- 游戏配置写入 `<游戏根目录>\bin\` 下所有数字版本目录的 `res_mods\ime_config.xml`；`8842736` 仅为版本示例，禁止硬编码为回退方案
- 生成的 `ime_config.xml` 必须与原 WinUI 3 版本保持兼容（UTF-8 无 BOM），否则游戏无法识别；分组顺序为简体、日文、繁体
- 输入法扫描依赖 TSF/COM（`ITfInputProcessorProfiles`），仅 Windows 可用，不得引入跨平台依赖
- 用户数据迁移规则（`config.json` → SQLite + LocalSettings 等价物）见 [references/storage.md](references/storage.md)
- 版本号只维护在根目录 `package.json`（`version` / `msiVersion` / `baseVersion`），其余位置一律由 `version.ps1` 同步
- 用户可见文本零硬编码，新增文案需同步补齐全部语种

## 常用命令

```powershell
# 安装前端依赖
pnpm install

# 开发运行（vite 热重载 + cargo 增量编译）
.\run.ps1

# 构建发布 exe
pnpm build

# 测试
cargo test            # 后端（src-tauri）
pnpm test             # 前端

# 本地化一致性校验
node tools/check-locales.mjs

# 版本号同步与打 tag 发布
.\version.ps1
.\tag.ps1
```

改动交付前必须全部通过：`cargo test`、`pnpm test`、`check:locales`、`pnpm build`。

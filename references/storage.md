# 持久化与数据迁移

后端用 SQLite（`rusqlite`，bundled）保存全部用户数据，文件位于应用数据目录下的 `settings.db`。实现见 `src-tauri/src/storage.rs`。

## 存储位置

| 内容 | 位置 |
| --- | --- |
| 设置库 | `<应用数据目录>\settings.db` |
| 旧版配置（只读来源） | `%LOCALAPPDATA%\Packages\BlazeSnow.WOWS-IME\LocalState\config.json` |

应用数据目录由 Tauri 的 `app_data_dir()` 给出（标识符 `com.blazesnow.wowsime`）。

## 表结构

与原 WinUI 3 版本一致，便于迁移。

### `settings`（简单状态，对应原 `ApplicationData.Current.LocalSettings`）

| 键 | 含义 |
| --- | --- |
| `Settings.SchemaVersion` | 配置结构版本，当前 `1` |
| `Settings.Language` | 界面语言模式：`auto` / `zh-Hans` / `zh-Hant` / `ja` |
| `Game.SelectedPath` | 当前选中的游戏根目录 |

### `custom_game_paths`（自定义游戏路径）

| 列 | 说明 |
| --- | --- |
| `display_name` | 显示名（目录名） |
| `path` | 主键，完整路径 |
| `created_at` / `updated_at` | 时间戳 |

### `custom_input_methods`（自定义输入法）

| 列 | 说明 |
| --- | --- |
| `display_name` | 主键，输入法名称 |
| `category` | 分类，取值同 `ImeCategory` 名称 |
| `created_at` / `updated_at` | 时间戳 |

## 写入语义

- 简单设置：upsert。
- 列表数据：**全量同步**——先 upsert 全部，再删除不在提交集合中的行。单条增删也走全量同步，前端始终提交完整列表。
- 列表读取排序与原版一致：`ORDER BY updated_at, 主键`。

## 旧数据迁移

启动时若检测到旧 WinUI 3 版本的 `config.json`，执行一次迁移：

| 旧字段 | 新位置 |
| --- | --- |
| `SelectedGamePath`（或 `GameDir`） | `settings.Game.SelectedPath` |
| `GamePaths[]`（`Path` / `Name`） | `custom_game_paths` |
| `Ime[]`（`Name` / `Category`） | `custom_input_methods` |

规则：

- `GamePaths[].Name` 为空时用目录名兜底，仍为空则用完整路径。
- `Ime[].Category` 未知值归为 `ChineseSimplified`。
- 迁移成功后旧文件改名为 `config.json.migrated`（已存在则先删除旧的改名文件）。
- **不使用额外的迁移标记**：改名本身就是「已迁移」的证据，因此迁移只会发生一次。
- 迁移失败静默忽略，不阻断启动。

## 错误处理

所有持久化异常一律静默吞掉（返回空列表或空串），与原版行为一致：存储问题不应让应用打不开或卡住主流程。

## 相关测试

`src-tauri/src/storage.rs` 的单元测试覆盖：设置默认值与 upsert、列表读写、全量同步的删除语义、旧配置迁移与改名、迁移只发生一次。

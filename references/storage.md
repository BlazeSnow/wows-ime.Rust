# 持久化与数据迁移

后端用 SQLite（`rusqlite`，bundled）保存全部用户数据，文件位于应用数据目录下的 `settings.db`。实现见 `src-tauri/src/storage.rs`。

## 存储位置

| 内容 | 位置 |
| --- | --- |
| 设置库 | `<应用数据目录>\settings.db` |
| 旧版数据（只读来源） | `%LOCALAPPDATA%\Packages\BlazeSnow.WOWS-IME_*\LocalState\` |

应用数据目录由 Tauri 的 `app_data_dir()` 给出（标识符 `com.blazesnow.wowsime`）。

旧包的目录名带发布者哈希后缀（实际形如 `BlazeSnow.WOWS-IME_cavwvnm5yrdtm`），
哈希随发布者证书而定，不能写死整个目录名——代码按包族名**前缀**在 `Packages` 下查找。

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
| `created_at` / `updated_at` | 时间戳，`DEFAULT CURRENT_TIMESTAMP` |

### `custom_input_methods`（自定义输入法）

| 列 | 说明 |
| --- | --- |
| `display_name` | 主键，输入法名称 |
| `category` | 分类，取值同 `ImeCategory` 名称 |
| `created_at` / `updated_at` | 时间戳，`DEFAULT CURRENT_TIMESTAMP` |

## 表结构迁移

早期版本建表时 `created_at` / `updated_at` 是 `NOT NULL` 但**没有默认值**，而写入语句只提供部分列，
每次插入都会报 `NOT NULL constraint failed` —— 保存静默失败，用户看到的就是「重启后内容没了」。
`CREATE TABLE IF NOT EXISTS` 不会改动已存在的表，因此：

- 启动时 `ensure_timestamp_defaults` 检测这两张表的建表语句，缺默认值就**重建表并保留数据**；
- 写入语句同时显式带上 `created_at` / `updated_at`，即使迁移没跑到也能写进去。

两个措施是叠加的：任一生效即可避免丢数据。

## 写入语义

- 简单设置：upsert。
- 列表数据：**全量同步**——先 upsert 全部，再删除不在提交集合中的行。单条增删也走全量同步，前端始终提交完整列表。
- 列表读取排序与原版一致：`ORDER BY updated_at, 主键`。

## 保存失败必须让用户知道

前端持久化失败**不得静默吞掉**。静默失败会让人以为保存成功，下次启动才发现没了——这是最难查的一类问题。
`useGamePaths` / `useInputMethods` 里的保存调用统一：写 `console.error` 并弹 `Status/SaveFailed` 提示。
（与原版「持久化异常不影响 UI」不冲突：UI 不崩，但要让用户看见。）

## 旧数据迁移

旧 WinUI 3 版有两个时代的存储，迁移按「先新后旧」的顺序取：

| 旧版 | 存储位置 | 迁移内容 |
| --- | --- | --- |
| v2026.6.5 起 | `LocalState\settings.db`（SQLite） | `custom_game_paths`、`custom_input_methods` 两张表 |
| 更早的版本 | `LocalState\config.json` | `SelectedGamePath`、`GamePaths[]`、`Ime[]` |

`config.json` 的字段映射：

| 旧字段 | 新位置 |
| --- | --- |
| `SelectedGamePath`（或 `GameDir`） | `settings.Game.SelectedPath` |
| `GamePaths[]`（`Path` / `Name`） | `custom_game_paths` |
| `Ime[]`（`Name` / `Category`） | `custom_input_methods` |

`settings.db` 里的两张表与本移植版**结构完全一致**，按行拷贝即可，
`created_at` / `updated_at` 一并保留。

规则：

- 两种来源都存在时以 `settings.db` 为准（它是新格式，状态更完整）。
- **旧版的标量设置不迁移**：它们在 `ApplicationData.Current.LocalSettings` 里
  （注册表 / `settings.dat`），不在 SQLite 中；`Game.SelectedPath` 与 `Settings.Language`
  需要用户重新选择，界面语言默认 `auto` 会自动跟随系统。
- `GamePaths[].Name` 为空时用目录名兜底，仍为空则用完整路径。
- `Ime[].Category` 未知值归为 `ChineseSimplified`。
- 迁移成功后旧 `config.json` 改名为 `config.json.migrated`（已存在则先删除旧的改名文件）。
- **迁移写入只 upsert、不删除**。全量同步语义（upsert + 删缺失行）只用于用户主动保存；
  迁移若沿用全量同步，会把用户在 Tauri 版里已添加的数据删掉。
- **迁移标记存在库里**（`Settings.LegacyMigrated`），不依赖文件改名是否成功。
  改名失败（文件被占用、目录只读）时旧文件仍在，没有标记就会每次启动重复迁移。
- 数据落库失败时不写标记、不改名，下次启动重试。
- 迁移失败静默忽略，不阻断启动。

## 错误处理

所有持久化异常一律静默吞掉（返回空列表或空串），与原版行为一致：存储问题不应让应用打不开或卡住主流程。

## 相关测试

`src-tauri/src/storage.rs` 的单元测试覆盖：设置默认值与 upsert、列表读写、全量同步的删除语义、旧配置迁移与改名、迁移只发生一次。

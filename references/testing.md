# 测试基线

## 命令

```powershell
cargo test           # 后端（src-tauri），当前 58 项
pnpm test            # 前端（vitest），当前 46 项
pnpm check:locales   # 三语资源一致性
```

交付前必须全部通过。后端 0 warning 也是硬要求。

## 后端测试（Rust）

| 文件 | 项数 | 覆盖内容 |
| --- | --- | --- |
| `error.rs` | 4 | 错误码序列化形状（前端从 rejection 中读 `{ code }`，是对外契约） |
| `state.rs` | 1 | 设置库路径 |
| `storage/tests.rs` | 15 | 建表幂等、设置默认值与 upsert、自定义路径/输入法读写、全量同步的删除语义、同路径改名不新增、旧 `config.json` 迁移与改名、迁移只发生一次、`GameDir` 兜底、缺名用目录名、未知分类回退、缺名输入法跳过、坏 JSON 与非对象根不迁移且不删旧文件、MSIX LocalState 目录形状 |
| `gameconfig/tests.rs` | 14 | 空选择的三个空分组、分组顺序、每项三元素与 Tag、名称 XML 转义、每项归入自己的分组、数字版本目录筛选、缺 bin / 无数字目录 / 形似数字的文件、UTF-8 无 BOM 写入、多目标写入、已存在文件计数、写入失败定位到具体路径、两种游戏 exe |
| `models.rs` | 7 | 分类字符串往返、未知分类回退、字段 camelCase 序列化、分类枚举名沿用原版、扫描结果与持久化模型的 JSON 往返、空警告字段不出现在 JSON 中 |
| `scanner/tests.rs` | 10 | 目标语言 ID 过滤、语言 ID 与名称关键词推断分类、噪音项过滤、HRESULT 格式、同名候选取高置信度且大小写不敏感、通用失败警告码、panic 可捕获 |
| `language.rs` | 3 | 显式语言原样保留、非法值回退 auto、四种受支持模式 |
| `commands/game.rs` | 4 | 未选输入法被拒、无数字版本目录被拒、成功写入并返回数量、写入失败时 detail 带具体路径 |

### 注意事项

- 扫描类测试依赖真实系统输入法配置，只断言结构（唯一性、排序），不断言具体条目。
- 涉及文件系统的测试使用临时目录并在结束时清理。
- `panic_is_catchable_in_this_profile` 必须用 `cargo test --release` 才有意义（普通 `cargo test` 走 dev profile，本来就是 unwind）。这条用例守住 release profile 不用 `panic = "abort"`。

## 前端测试（vitest + Testing Library）

| 文件 | 项数 | 覆盖内容 |
| --- | --- | --- |
| `i18n.test.ts` | 7 | 语言模式归一化、系统语言推断（繁体/简体/日文/回退） |
| `i18n.interpolation.test.ts` | 3 | `{0}` 占位符插值、多占位符、未知键回退 |
| `lib/errors.test.ts` | 7 | 错误 detail 优先于错误码、无法识别的错误转字符串、警告码本地化与参数填充、资源缺失时回退原始警告码 |
| `lib/api.test.ts` | 5 | 每个 Tauri 命令的调用名与参数结构、后端返回值的透传、后端拒绝时错误透传 |
| `pages/HomePage.test.tsx` | 15 | 三步向导：预设路径与默认选中、切换路径即持久化、扫描结果入列、未勾选输入法被拦、勾选后第三步汇总、新增/覆盖两种写入确认、取消写入不落盘、缺游戏 exe 中止、空扫描结果带警告提示、添加自定义输入法、名称重复被拒 |
| `pages/SettingsPage.test.tsx` | 6 | 版本号展示、项目链接入口、打开官网、切换语言即时生效并持久化 |
| App.test.tsx | 3 | 应用外壳：侧边栏图标、切换到设置页、原生窗口标题取自 AppDisplayName |

### 测试写法约定

- Tauri 侧一律 mock：`@tauri-apps/api/core` 的 `invoke`、`@tauri-apps/plugin-dialog` 的 `open`、`@tauri-apps/plugin-opener` 的 `openPath`、`sonner` 的 `toast`。
- mock 工厂用 `vi.hoisted` 拿到 mock 函数实例，再在 `vi.mock` 工厂里引用。
- **Radix Select 走键盘驱动**：`trigger.focus()` → `[Enter]` 展开 → `[ArrowDown]` 移项 → `[Enter]` 提交。指针路径在 jsdom 下不工作。
- 查询文案要用无歧义写法：勾选框用 `getAllByRole("checkbox")[0]`（多个输入法就有多个），label 用 `getByLabelText`（卡片标题与 label 文案常相同，`getByText` 会命中多个）。
- 切换界面的用例要放在文件**最后**：语言一变，同文件其它用例的文案断言都会被污染。

## 手工验收清单

发布前在真机上逐条确认：

- [ ] 扫描能列出系统已启用的中文/日文输入法，分类推断正确
- [ ] 三种预设路径可选择，自定义路径可添加、删除（删除有确认框）
- [ ] 自定义输入法可添加、删除，分类可切换且重启后保留
- [ ] 写入前有覆盖/新增确认，取消后不写文件
- [ ] 写入后 `bin\<数字版本>\res_mods\ime_config.xml` 内容正确、UTF-8 无 BOM
- [ ] 游戏内可切换到所选输入法
- [ ] 三语界面完整，切换立即生效且重启后保留
- [ ] 从旧 WinUI 3 版本启动后，旧的游戏路径与自定义输入法被迁移，旧文件改名为 `config.json.migrated`

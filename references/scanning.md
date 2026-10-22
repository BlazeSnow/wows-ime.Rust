# 输入法扫描（TSF/COM）

扫描系统已启用的中文与日文输入法。实现位于 `src-tauri/src/scanner.rs`，只依赖 Windows 的 COM 与 TSF，因此应用仅支持 Windows。

## 调用链

1. `CoInitializeEx(COINIT_APARTMENTTHREADED)` 初始化 COM。
   - 成功或 `S_FALSE`：本次调用负责 `CoUninitialize`。
   - `RPC_E_CHANGED_MODE`（`0x80010106`）：COM 已按其它套间模型初始化，可继续使用，但不反初始化。
   - 其它失败：返回警告码 `Tsf/CoInitializeFailed`。
2. `CoCreateInstance` 创建 `ITfInputProcessorProfiles`
   （CLSID `33C53A50-F456-4884-B049-85FD643ECFED`）。失败返回 `Tsf/CreateProfilesObjectFailed`。
3. `GetLanguageList` 取全部语言 ID。失败返回 `Tsf/GetLanguageListFailed`，为空返回 `Tsf/GetLanguageListEmpty`。
4. 对每个目标语言 ID 调 `EnumLanguageProfiles` 枚举输入法配置文件，逐项：
   - `IsEnabledLanguageProfile` 为真才保留；
   - `GetLanguageProfileDescription` 取显示名，空名丢弃；
   - 按名称过滤「输入体验 / Input Experience」噪音项；
   - 推断分类后加入结果。
5. 结果按显示名排序去重（不区分大小写），同类同名时保留置信度高者。

## 目标语言 ID

| 语言 ID | 含义 | 默认分类 |
| --- | --- | --- |
| `0x0804` | 中文（简体，中国） | 简体 |
| `0x1004` | 中文（简体，新加坡） | 简体 |
| `0x0404` | 中文（繁体，中国台湾） | 繁体 |
| `0x0C04` | 中文（繁体，中国香港） | 繁体 |
| `0x1404` | 中文（繁体，中国澳门） | 繁体 |
| `0x0411` | 日文 | 日文 |

其它语言 ID 一律跳过。

## 分类推断

先按语言 ID 判断；判断不出时按名称关键词（大小写不敏感）：

| 关键词 | 分类 |
| --- | --- |
| 速成、倉頡、仓颉、注音、Quick、Cangjie | 繁体 |
| Japanese、日文 | 日文 |
| 拼音、五笔 | 简体 |

都判断不出时归为**简体**（与原版一致）。

## 警告码

| 警告码 | 含义 |
| --- | --- |
| `Tsf/CoInitializeFailed` | COM 初始化失败，参数为 HRESULT |
| `Tsf/CreateProfilesObjectFailed` | 无法创建 COM 对象 |
| `Tsf/GetLanguageListFailed` | 取语言列表失败，参数为 HRESULT |
| `Tsf/GetLanguageListEmpty` | 语言列表为空 |
| `Tsf/ComException` | COM 调用异常 |
| `Tsf/GenericException` | 其它异常 |

警告码是稳定契约：前端据此映射本地化文案（`src/locales/*.json` 中以 `Tsf/` 开头），改动需同步三语资源。

## 注意事项

- 扫描结果依赖真实系统输入法配置，单元测试只断言结构（唯一性、排序），不断言具体条目。
- COM 调用必须成对：`CoInitializeEx` 成功后必须 `CoUninitialize`。
- `GetLanguageList` 返回的语言 ID 数组由 COM 分配，用完必须 `CoTaskMemFree` 释放
  （原 WinUI 3 版在 `finally` 里释放，本移植版同样在遍历结束后释放，避免每次扫描泄漏）。

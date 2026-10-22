# 游戏配置文件生成规则

本文件描述写入游戏的 `ime_config.xml` 的结构与生成规则。这是本项目最硬的兼容约束：游戏只认这个结构，改动可能导致输入法在游戏内不可用。

## 目标路径

```
<游戏根目录>\bin\<数字版本>\res_mods\ime_config.xml
```

- `bin` 下的**所有**数字版本目录都要写一份（游戏可能加载其中任意一个）。
- 版本号目录名只含 ASCII 数字才算命中；其它目录忽略。
- 目录按名称的字符串序（忽略大小写）处理，顺序与原版一致。
- **禁止硬编码版本号**。`8842736` 只是某一时期的示例，写死会让新版本游戏写不进去。

## 文档结构

```xml
<data>
  <language>
    <ChineseSimplified>
      <imeName>微软拼音</imeName>
      <displayName>微软拼音</displayName>
      <Tag>GFxIME_Ch_Simp</Tag>
    </ChineseSimplified>
    <Japanese />
    <ChineseTraditional />
  </language>
</data>
```

规则：

- 根节点固定 `<data>`，其下固定 `<language>`。
- 分组顺序固定：**简体 → 日文 → 繁体**（与原版 `BuildConfigDocument` 一致）。
- 每个输入法固定写三个元素，顺序为 `imeName` → `displayName` → `Tag`。
- `imeName` 与 `displayName` 内容相同，均为用户可见的输入法名称。
- 没有输入法的分组写成自闭合空元素 `<Xxx />`。
- 缩进为 2 空格一级（与原版 `XDocument` 的默认缩进一致）。

## 分类与 Tag 的对应

| 分类 | 分组 | Tag |
| --- | --- | --- |
| 中文简体 `ChineseSimplified` | `ChineseSimplified` | `GFxIME_Ch_Simp` |
| 中文繁体 `ChineseTraditional` | `ChineseTraditional` | `GFxIME_Ch_Trad_Array` |
| 日文 `Japanese` | `Japanese` | `GFxIME_Jp` |

## 编码与写入

- 文件以 **UTF-8 无 BOM** 保存。BOM 会让部分解析器把首元素名读成 `\ufeffdata`，导致游戏读不到配置。
- 写入前创建缺失的 `res_mods` 目录。
- 每个目标文件独立写入，任一文件失败即整体报错。

## 名称转义

输入法名称中的 `&`、`<`、`>`、`"`、`'` 必须转义为 XML 实体，否则会破坏文档结构。转义在生成阶段完成。

XML 1.0 不允许除制表/换行/回车以外的控制字符（以及 `U+FFFE`/`U+FFFF`），
这类字符**直接丢弃**而不是原样写入——原样写入会产出非法 XML，游戏解析失败。
原 WinUI 3 版用 `XDocument` 序列化，遇到非法字符会直接抛错（写不进去）；本移植版选择
丢弃，保证产出的始终是合法 XML。

## 相关代码

| 位置 | 职责 |
| --- | --- |
| `src-tauri/src/gameconfig.rs` | 路径解析、文档生成、写入 |
| `src-tauri/src/commands/game.rs` | 对前端暴露的校验与写入命令 |

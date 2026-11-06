# Simplified Chinese (zh-CN) Minesweeper terminology: what the Chinese community actually says

> **How these notes were made (2026-11-06).**
> - **saolei.wang (扫雷网)**, China's main ranking site, was fetched directly with curl. That covered the tutorial index, the terms page, the practice page, the ranking page and twelve tutorial threads by top players.
> - **minesweeper.online**'s Chinese UI was read in two ways. The string bundle `https://minesweeper.online/js/cn-1046.js` was paired key by key with `en-1046.js`, which gives exact English to zh-CN pairs. The Chinese Patterns help page was rendered with headless Chrome.
> - **Metasweeper (元扫雷)** and **ms-toollib** were cloned from GitHub (Metasweeper `master` @ 9a43e78, ms-toollib `main` @ ef98fdb). Metasweeper is written in Chinese, so `src/ui/en_US.ts` maps each Chinese source string to its English translation.
> - **Chinese Wikipedia** was read in its zh-cn variant: 踩地雷 (oldid 95246442) and 微软扫雷.
> - **Windows XP strings:** no Microsoft zh-CN binary or string dump was reachable. The XP strings below come from open-source XP clones that copy the Chinese UI (mainly `crlf0710/charlesmine-rs`), and they are marked **[XP via clones]**. The Wine `winemine` zh_CN strings are shown only as a contrast.
> - **Zhihu** returned 403 (bot check), so 张少武's four Zhihu articles were not read; only their titles were seen, linked from minesweeper.online. **Bilibili** videos were not reached. One Bilibili text column and one TapTap tutorial were read, and both count as secondary sources.
>
> The authors cited are well-known players. On the saolei.wang ranking page, 张少武 is ranked #8, 郭蔚嘉 #7 and 周丹 #12 ([ranking](https://saolei.wang/ranking)). 张砷镓 is the site's founder, user #1, with an Expert record of 37.82 s ([profile block on bbs/362](https://saolei.wang/bbs/362)).
>
> Markers used: **[uncertain]** means the sources disagree, or the term was seen only once or only in a secondary source. **[unattested]** means no source used a term, so the cell holds a proposal. **[proposal]** means a suggestion for Runesweeper, not a community usage.

---

## 1. Takeaway

- The community vocabulary is stable and short. **定式** is a pattern. Patterns are named by their digits with no hyphens: **11定式, 12定式 (= 21定式), 121定式, 1221定式**. **减法** is subtraction or reduction, and **连续减法** is a chained subtraction. **挖坑** is a hole. **数雷** is mine counting. **判雷** means to deduce mines, **猜雷** means to guess, and **死猜** is a 50/50 or dead guess. **双击** is a chord, and **标雷** means to flag. **盲扫** (or NF, 不标雷) is no-flag play. **无猜** means no-guess.
- For the levels, use **初级 / 中级 / 高级 / 自定义**. These appear in XP, saolei.wang, minesweeper.online, Metasweeper and Chinese Wikipedia. Avoid the Wine wording 初学者/中等/专家 and the Windows 10 wording 简单/中等/专家.
- Several words have **two meanings**, and a translator has to pick one sense for each string:
  - **空** means both an opening (a blank region) and, in deduction talk, a safe cell.
  - **双击** means a chord, but also an ordinary double-click.
  - **开局** is the XP "New game" menu item, and also the opening phase of a game.
  - **数雷** usually means counting the remaining mines, but some writers use it for basic counting around a number.
- Hint wording on minesweeper.online follows one consistent template, and Runesweeper's explainable hint can copy it: 「看到…的 1，它周围有两个黄格，所以这两个黄格有 1 个雷。再看到 2，…所以剩下一格必须是雷。」 See section 5.

---

## 2. Game UI

| English | zh-CN | Source | Note |
|---|---|---|---|
| Minesweeper (the game) | 扫雷 | [zh.wikipedia 踩地雷 (zh-cn)](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7) | The infobox gives 中国大陆 扫雷, 台湾 踩地雷. Use 扫雷 for zh-CN. Formal article title: 扫雷游戏. |
| Game (menu) | 游戏(G) | [charlesmine-rs build.rs L90](https://github.com/crlf0710/charlesmine-rs/blob/master/build.rs); [Metasweeper en_US.ts](https://github.com/eee555/Metasweeper/blob/master/src/ui/en_US.ts) ("游戏 => Game") | **[XP via clones]** for the menu label. |
| a game / one round | 局, 一局 (also 盘) | [saolei bbs/1262](https://saolei.wang/bbs/1262) ("整局", "某个局部"); [saolei bbs/11409](https://saolei.wang/bbs/11409) ("7小时55盘"); minesweeper.online `quests_wins_single` "完成一局 %level%" ([cn bundle](https://minesweeper.online/js/cn-1046.js)) | 好局 means a good board, and 痛失好局 means losing a good board ([雷诀八条](https://saolei.wang/bbs/1579)). |
| New game | 新游戏 | minesweeper.online `menu_new_game` ([cn bundle](https://minesweeper.online/js/cn-1046.js)); Metasweeper en_US.ts L2804 | XP's menu item was **开局(N) F2** ([charlesmine-rs build.rs L96](https://github.com/crlf0710/charlesmine-rs/blob/master/build.rs)) **[XP via clones]**. Restarting is 重开 / 重开新局 (Metasweeper "重开新局"; minesweeper.online `settings_new_game_on_middle_click` "中键重开"). |
| Beginner / Intermediate / Expert / Custom | 初级 / 中级 / 高级 / 自定义 | minesweeper.online `level_1..4`; Metasweeper en_US.ts ("初级 => Beginner" …); [saolei.wang/online](https://saolei.wang/online); [zh.wikipedia](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7); XP clones `初级(&B) 中级(&I) 高级(&E) 自定义(&C)...` | Every community source agrees. Do **not** use Wine's 初学者 / 中等 / 专家 ([wine po/zh_CN.po](https://github.com/wine-mirror/wine/blob/master/po/zh_CN.po)) or Windows 10's 简单/中等/专家 ([zh.wikipedia 微软扫雷](https://zh.wikipedia.org/zh-cn/%E5%BE%AE%E8%BD%AF%E6%89%AB%E9%9B%B7)). |
| Level (difficulty setting) | 级别 / 难度 | minesweeper.online `seo_description_new_game` "选择初级、中级、高级或自定义级别"; [Metasweeper README](https://github.com/eee555/Metasweeper/blob/master/README.md) ("快捷键切换难度") | Both are in use. 级别 is closer to Runesweeper's "Level". |
| Custom field dialog | 自定义雷区; 高度(H): 宽度(W): 雷数(M): | [charlesmine-rs res/CharlesMine.rc L79-82](https://github.com/crlf0710/charlesmine-rs/blob/master/res/CharlesMine.rc) | **[XP via clones]**. Metasweeper uses 自定义设置 / 高度 / 宽度 / 雷数. |
| Mine | 雷 (everyday), 地雷 (formal) | [saolei bbs/177](https://saolei.wang/bbs/177); [zh.wikipedia](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7) | Players say 雷 almost everywhere: 雷数, 标雷, 踩雷, 判雷. |
| Mines (count / setting) | 雷数 | minesweeper.online `game_mines` "雷数"; Metasweeper "雷数： => Mines:" | |
| Mines left (counter) | 剩余雷数 / 剩余 N 雷 | [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) ("左上角显示剩余 6 雷"); [zh.wikipedia](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7) ("剩余地雷的数量"); [TapTap guide](https://taptap.cn/moment/492951218495163660) ("剩余雷数判断") | |
| Total mines | 总雷数 | [ms-toollib algorithms.rs L211-214](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) ("未知雷数 = 总雷数 - 已经标出的雷") | |
| Flag (noun) | 旗 / 旗子 / 小红旗 | [saolei bbs/177](https://saolei.wang/bbs/177) ("小红旗"); [saolei.wang/online](https://saolei.wang/online) ("长按格子放置旗子"); [zh.wikipedia](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7) ("小旗") | |
| to flag | 标雷 (also 插旗) | minesweeper.online `homepage_rules_text` "用右键在你认为是雷的地方插旗（称为标雷）"; [saolei bbs/374 怎样节省标雷](https://saolei.wang/bbs/374) | 标雷 is the standard verb. |
| Flags (counter label) | 标记 / 标雷数 | minesweeper.online `game_flags` "标记"; [ms-toollib minesweeper_board.rs L60](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) "标雷数" | |
| Question marks (Marks option) | 问号; XP menu **标记(?)(M)** | [charlesmine-rs build.rs L121](https://github.com/crlf0710/charlesmine-rs/blob/master/build.rs); [saolei.wang/online](https://saolei.wang/online) ("问号标记：右键按 标雷 → 问号 → 取消 三态循环") | The XP label is **[XP via clones]**. For the setting, saolei.wang's 问号标记 is clear. |
| Timer | 计时器; time 时间 / 用时 / 耗时 | minesweeper.online `settings_disable_timer` "隐藏计时器", `stat_column_time` "耗时" | |
| Best times (XP dialog) | 扫雷英雄榜(T)...; button 重新计分(R); default name 匿名 | [charlesmine-rs build.rs L127, CharlesMine.rc L87-99](https://github.com/crlf0710/charlesmine-rs/blob/master/res/CharlesMine.rc); also [richenyunqi/Mine-game](https://github.com/richenyunqi/Mine-game) and yizhuoyan/game-winmine-java (`扫雷英雄榜(T)...`) | **[XP via clones]**. Three independent clones agree on 扫雷英雄榜. |
| New record dialog (XP) | 已破初级记录。请留尊姓大名。 | [CharlesMine.rc L88-90](https://github.com/crlf0710/charlesmine-rs/blob/master/res/CharlesMine.rc) | **[uncertain]** Only one clone has this text. Whether the original used 记录 or 纪录 is unverified. |
| Record / personal best | 纪录 / 个人纪录; best time 时间纪录 | [saolei.wang](https://saolei.wang/) (nav "纪录", news "刷新了 高级 时间纪录"); minesweeper.online `game_pb` "个人纪录", `profile_best_time` "时间纪录"; Metasweeper "个人纪录 => Personal Records" | The community spells a sports record **纪录**. Use **记录** for a log or history: Metasweeper's 历史记录 plugin, and Runesweeper's History → 历史记录. |
| Win / Loss | 胜利 / 失败; "你赢了" / "你输了" | minesweeper.online `game_state_win`, `game_state_defeat`, `champ_game_won`, `champ_game_lost`; Metasweeper en_US.ts L1002-1007 | |
| hit a mine (lose) | 踩雷 (colloquially 爆) | [saolei bbs/177](https://saolei.wang/bbs/177) ("如果不幸踩雷"); [ms-toollib minesweeper_board.rs L332](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) ("按踩雷判负"); [雷诀八条](https://saolei.wang/bbs/1579) ("爆了脸上不留疤") | 判负 means "is judged lost" (technical register). |
| Win rate | 胜率 | [saolei.wang/online](https://saolei.wang/online); minesweeper.online `game_map_win_chance` | |
| How to Play / Rules | 游戏规则 / 基本规则 | minesweeper.online `homepage_rules_title` "游戏规则", `help_gameplay_title` "基本规则" | 教程 means tutorial (saolei nav; Metasweeper "教程 => Tutorial"). 帮助(H) is the Help menu. |
| Hint | 提示 | minesweeper.online `game_ng_hint` "提示", `quests_19_single` "不用提示完成一局"; [saolei.wang/online](https://saolei.wang/online) ("新手提示：当卡住时，提示附近能确定的地雷和空") | |
| Cell | 格子 / 方格 / 方块 (also 格) | [saolei bbs/177](https://saolei.wang/bbs/177) (方格); [saolei bbs/362](https://saolei.wang/bbs/362) (方块); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) (格 / 黄格) | |
| Unopened / unknown cell | 未开格 / 未打开的格子 / 未知格 | [ms-toollib minesweeper_board.rs L613](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) ("周围未开格数"); [TapTap guide](https://taptap.cn/moment/492951218495163660) ("未知格") | |
| to open / reveal | 打开 / 点开 / 翻开 | [saolei bbs/177](https://saolei.wang/bbs/177); [saolei bbs/8243](https://saolei.wang/bbs/8243) ("翻开") | |
| Board / position | 局面 (position); 盘面 (board); 雷区 (minefield, XP); 局部 (a local area) | Metasweeper/ms-toollib (局面); minesweeper.online homepage (盘面); [saolei bbs/1262](https://saolei.wang/bbs/1262) (局部) | Hints should use 局部 for "this area". |
| Chord | **双击** | [saolei bbs/1262](https://saolei.wang/bbs/1262) ("标雷后双击"); minesweeper.online `game_chords` "双击", `settings_chording_2` "传统左右键双击", `settings_chording_1` "左键代替双击"; [saolei.wang/online](https://saolei.wang/online) ("左键双击：左键点击已打开的数字，视为双击操作") | Here 双击 means pressing both buttons, not a double-click. Metasweeper also uses 双击 for a real double-click ("左键双击重开"). The flag-then-chord technique is 标雷双击 / 标雷连双击 ([bbs/285](https://saolei.wang/bbs/285)). |
| First click | 起手 / 首次点击 | [ms-toollib utils.rs L504, L538](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs) ("起手位置非雷", "win7规则：起手位置开空"); [zh.wikipedia](https://zh.wikipedia.org/zh-cn/%E8%B8%A9%E5%9C%B0%E9%9B%B7) ("首次点击总是安全的") | 起手 is the player jargon. Use 首次点击 for plain UI text. |
| Opening (blank region) | **空**; to open one: **开空** | [saolei.wang/online](https://saolei.wang/online) ("开空、标雷时播放声音反馈"); [ms-toollib algorithms.rs L501](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) ("计算开空概率"); Metasweeper "显示空 => Show empty" | **Two senses**: in tutorials, 空 also means a safe cell ("3选2点中间的空", [bbs/187](https://saolei.wang/bbs/187); "隔空判空", [bbs/8367](https://saolei.wang/bbs/8367)). |
| break into an opening | 破空 | [saolei bbs/285](https://saolei.wang/bbs/285) ("标雷一定要能破空，否则就是废雷"); [bbs/11409](https://saolei.wang/bbs/11409) ("破空和IOE"); [ms-toollib analyse_methods.rs L12](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/analyse_methods.rs) | **[uncertain]** on the exact sense. From context it means making a move that opens a region (gets an opening to cascade). It is a speedrun term, not needed in the UI. |
| Island | 岛 (Isl) | [ms-toollib utils.rs L60](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs) ("计算岛") | The count of openings is left as "Op/Ops" in Metasweeper. No Chinese name was found **[unattested]**. |
| 3BV, 3BV/s | 3BV, 3BV/s (kept in Latin script) | [saolei 术语介绍](https://saolei.wang/page/help/word) ("每局将所有非雷的方块点开所需最少左键点击数") | |
| IOE / efficiency | IOE (kept in Latin script); efficiency 效率 | [saolei 术语介绍](https://saolei.wang/page/help/word); minesweeper.online `game_eff`, `eff_description` "效率指一局游戏的3BV值除以总点击数" | |
| NF (no flags) | **盲扫**; NF; 不标雷; 未标雷 | [saolei 术语介绍](https://saolei.wang/page/help/word) ("NF No Flag：一种仅用左键点击完成游戏，不标雷的玩法"); minesweeper.online `ranking_nf_only` "盲扫"; Metasweeper "未标雷（标准） => NF(Standard)", "标准（盲扫）"; [saolei bbs/285 小谈盲扫与标雷](https://saolei.wang/bbs/285) | Flagging play is called 标雷, as opposed to 盲扫. |
| UPK, MB, LC | UPK, MB, LC (LC = 打开最后一个方格时不幸踩雷) | [saolei 术语介绍](https://saolei.wang/page/help/word) | |
| Replay | 录像 | [saolei.wang](https://saolei.wang/) nav; Metasweeper | |
| Mine probability | 是雷的概率 / 雷概率 | Metasweeper en_US.ts L904 "(是雷的概率)"; minesweeper.online `analyzer_mine_probability` "雷概率" | |

---

## 3. Modes and field generation

| English | zh-CN | Source | Note |
|---|---|---|---|
| No-guess mode | **无猜 / 无猜模式** (NG) | minesweeper.online `menu_ng_mode` "无猜模式", `ng` "无猜（NG）"; [Metasweeper README](https://github.com/eee555/Metasweeper/blob/master/README.md) ("包含无猜、可猜模式"); [saolei.wang/online](https://saolei.wang/online) | saolei.wang's own 无猜模式 works after the fact: "踩雷时在不改变已开数字的前提下尝试重排局面". |
| Classic (mode) | 经典模式 | minesweeper.online nav "经典模式 新游戏" | Metasweeper calls its plain mode **标准** (Standard) and offers a separate **Win7** rule. |
| Metasweeper no-guess variants | 经典无猜, 强无猜, 弱无猜, 准无猜, 强可猜, 弱可猜 | Metasweeper en_US.ts L279-339 (Classic/Strict/Weak no guess, "Blessing mode", "Guessable no guess", "Lucky mode"); semantics in [ms-toollib minesweeper_board.rs L312-354](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) | 经典无猜 is a board generated to be solvable. 强/弱/准无猜 and 可猜 re-lay the mines when the player clicks. **Caution**: 经典 here belongs to a no-guess variant, so a Runesweeper "Classic" called 经典 could be confused with Metasweeper's 经典无猜. |
| lay mines (generation) | 埋雷 / 摆雷 | [ms-toollib utils.rs L503-538](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs) ("标准埋雷规则…"); [saolei.wang/online](https://saolei.wang/online) ("（摆雷中）", "手动摆雷") | |
| solvable without guessing | 无猜 / 可解 / "局面是否无猜" | [ms-toollib algorithms.rs L715](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) ("判断局面是否无猜"); [TapTap guide](https://taptap.cn/moment/492951218495163660) ("无猜扫雷是有解题") | |
| Runesweeper "Patterns" mode | 定式模式 **[proposal, unattested]** | no source | The mode's own name borrows the well-known word 定式. |
| Runesweeper "Max Guess" mode | **[unattested]**. Possible wordings: 多猜模式 or 死猜模式 **[proposal]** | no source | Metasweeper's 可猜 means "guesses are forgiven", which is the opposite intent. Avoid 可猜. |

---

## 4. Deduction techniques, patterns and guessing

| English | zh-CN | Source | Note |
|---|---|---|---|
| Pattern | **定式** | [saolei bbs/362 扫雷定式及其变化](https://saolei.wang/bbs/362) ("定式其实是围棋及五子棋中的术语…我们这里借用定式这个名词"); minesweeper.online `help_patterns_title`, `analyzer_pattern` "定式" | 定势 is a rare misspelling (seen in one comment on bbs/362). |
| Pattern variation | 定式变形 / 变种 | [bbs/362](https://saolei.wang/bbs/362) ("11 定式的变形"); [bbs/8367](https://saolei.wang/bbs/8367) ("121定式变种"); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) ("121 定式变形") | |
| Mine shape around a number | 雷型 / 雷的分布形状 | [bbs/8243](https://saolei.wang/bbs/8243); [bbs/11409](https://saolei.wang/bbs/11409) ("大Y的雷型分布帖") | |
| 1-1 | **11定式**; edge form 边部11定式 / 边11定式 / 靠墙11减法 | [bbs/362](https://saolei.wang/bbs/362); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) ("11 定式 (1)", "靠墙 11 减法"); [TapTap](https://taptap.cn/moment/492951218495163660) ("边11定式") | Digits are written run together with no hyphen. A space is optional ("11 定式"). |
| 1-2 | **12定式 = 21定式** | [bbs/362](https://saolei.wang/bbs/362) ("21 定式（或称 12 定式）"); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) ("21 定式也称 12 定式") | Straight-row form: 直排12定式 (TapTap). |
| 1-2-1 | **121定式** | [bbs/362](https://saolei.wang/bbs/362); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns) ("121 定式实际上就是两个 12 定式") | |
| 1-2-2-1 | **1221定式** | [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns); [bbs/11409](https://saolei.wang/bbs/11409) ("最常用的定式比如121和1221是必须掌握的") | |
| other named patterns | 212定式, 靠雷221定式, 靠雷31/41定式, 夹雷22定式, 边部211定式, 角131定式, 角222定式 (角22定式), 隔行的121定式 | [bbs/362](https://saolei.wang/bbs/362); [minesweeper.online/cn/help/patterns](https://minesweeper.online/cn/help/patterns); [TapTap](https://taptap.cn/moment/492951218495163660) | 靠雷 = next to a known mine. 夹雷 = between mines. 靠边/靠墙 = against the edge. 角 = corner. |
| Hole | **挖坑** (also 挖坑定式); T-shaped T形挖坑; multi-layer 多层挖坑 | minesweeper.online `analyzer_pattern_hole` "挖坑"; [help/patterns](https://minesweeper.online/cn/help/patterns); [bbs/362](https://saolei.wang/bbs/362) ("挖坑定式") | |
| Triangle (pattern) | **[unattested]** | not found in any source | The Chinese sources name shapes by their digits or by 挖坑, not by "triangle". |
| Reduction (subtract known mines from a number) | 化归 (section title); "2 实际上是 1"; 减雷; on saolei 附加雷条件下的假设 | [help/patterns](https://minesweeper.online/cn/help/patterns) (section "化归": "在左边一个雷的影响下，2 实际上是 1"); [TapTap](https://taptap.cn/moment/492951218495163660) ("基础减雷，减雷变12定式"); [bbs/362](https://saolei.wang/bbs/362) ("某方块周围附加n颗雷，则该方块的数字在做判断时减去n") | The phrase "X 实际上是 Y" is how tutorials state a reduced number. It works well in hints. |
| Subtraction (shared-cell difference) | **减法 / 减法公式**; chained: **连续减法**; named: 11减法, 12减法, 23减法, 24减法, 41减法 | [bbs/11409 郭嘉NF经验谈](https://saolei.wang/bbs/11409) ("可以学习减法公式…适用于在无法精确得出雷的位置时翻开不是雷的地方"); [help/patterns](https://minesweeper.online/cn/help/patterns); 张少武's Zhihu titles, e.g. "扫雷进阶玩家判雷核心——减法公式的应用实例" ([zhuanlan.zhihu.com/p/28176578](https://zhuanlan.zhihu.com/p/28176578), not read: 403) | |
| Basic single-number rules | 数数 (1)/(2); 基本逻辑; engine term 单集合 | [help/patterns](https://minesweeper.online/cn/help/patterns) ("数数 (1)": "如果一个数字周围只剩下对应数量的格子，那么这些格子必须全是雷"); `analyzer_pattern_basic_logic` "基本逻辑"; [ms-toollib algorithms.rs L137](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) ("单集合判雷引擎") | 双集合 (two-number) is the engine term for overlapping-pair logic ([algorithms.rs L44](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs)). It is technical, not tutorial language. |
| Case analysis / proof by contradiction | 穷举与假设; 假设法; 一格法 / 两格法 | [help/patterns](https://minesweeper.online/cn/help/patterns) ("假设黄格是雷…和剩余雷数矛盾，所以黄格必须安全"); [TapTap](https://taptap.cn/moment/492951218495163660) ("一格法的意思就是对于一个格，它有两个结果：要么是雷，要么就是打开") | Enumeration: 枚举 ([ms-toollib algorithms.rs L580](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) "枚举法判雷引擎"). |
| Mine counting (endgame, using the mine total) | **数雷**; also 剩余雷数判断 | minesweeper.online `analyzer_pattern_mine_counting`, `game_hints_mc_title` "数雷助手"; [help/patterns](https://minesweeper.online/cn/help/patterns) ("当游戏临近尾声时，可能可以通过剩余雷数来判雷"); [TapTap](https://taptap.cn/moment/492951218495163660) | **[uncertain]** because 数雷 can also mean counting mines around a number: [bbs/11409](https://saolei.wang/bbs/11409) "3及以上的数字可以通过鼠标数雷解决"; the [Bilibili column](https://www.bilibili.com/read/cv5809834) "最基本的推理就是数雷". In hints, say "根据剩余雷数" to be unambiguous. |
| to deduce mines / safe cells | **判雷** / 判空 | [bbs/8367 浅谈隔空判雷及标雷](https://saolei.wang/bbs/8367) ("隔空判雷", "隔空判空"); [ms-toollib](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) ("判雷引擎") | Other common verbs: 推出 / 推理 / 判断 / 确定. |
| Safe cell | **安全格** / 非雷 / 不是雷 / (tutorial) 空 | minesweeper.online `homepage_rules_text` ("推导出安全格"); [ms-toollib minesweeper_board.rs L312](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) ("可推出的安全格"); [bbs/362](https://saolei.wang/bbs/362) ("不是雷") | The adjective "safe" in hints: "…必定安全" / "可以打开" (help/patterns). |
| Frontier | 边缘格子 (vs. 内部未知格子 for interior) | [ms-toollib algorithms.rs L214](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs) | **[uncertain]** This is a solver term. Player tutorials name no frontier. The Bilibili solver write-up uses 连通块 / 联通块 for a connected component ([cv5809834](https://www.bilibili.com/read/cv5809834)). |
| Dead cell | **[unattested]** | not found | Describe it instead, e.g. "打开后也不会提供新信息的格子" **[proposal]**. |
| Guess | **猜雷** / 猜 | [saolei bbs/187 猜雷的一些心得](https://saolei.wang/bbs/187); [ms-toollib analyse_methods.rs L16](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/analyse_methods.rs) ("可以判雷时选择猜雷"); [雷诀八条](https://saolei.wang/bbs/1579) | High-risk guess: 高风险的猜雷 (analyse_methods.rs L8). |
| 50/50 (true, unresolvable) | **死猜**; the two-cell case 2选1 / 二猜一 (edge: 边部2选1; enclosed: 封闭式2选1); "几率都是对半开" | minesweeper.online `analyzer_50_50` "你已经选择了最优操作（死猜）", `analyzer_display_50_50` "显示死猜", `analyzer_potential_50_50` "可能的死猜"; [bbs/187](https://saolei.wang/bbs/187) ("死猜：绝对没有其他辅助条件可想的猜雷…死猜没有所谓正确的解法。因为几率都是对半开"; a comment writes "二猜一") | 张砷镓's 死猜 is broader than a strict 50/50: it includes 封闭式3选1 and 4选2. It contrasts with **过程猜**, a guess that later play might resolve. **五五开** and **二选一** were not seen in these sources **[unattested]**, though both are ordinary Chinese. |
| Forced guess | 必须猜 / 只能猜; stuck: 无从下手 | [ms-toollib minesweeper_board.rs L312, L354](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs) ("一个都推不出来（必须猜）", "推不出任何安全格（只能猜）"); [雷诀八条](https://saolei.wang/bbs/1579) ("无从下手不要愣，就近猜雷把心横") | |
| Safest / optimal move | 最安全的操作 / 最优操作 / 更安全的操作 | minesweeper.online `analyzer_safest_move`, `analyzer_safer_move` | |
| Corner / edge / wall | 角 / 角部 / 靠角; 边 / 边部 / 靠边 / 靠墙 / 墙 | [bbs/362](https://saolei.wang/bbs/362), [bbs/374](https://saolei.wang/bbs/374), [help/patterns](https://minesweeper.online/cn/help/patterns), [bbs/11409](https://saolei.wang/bbs/11409) ("下方是墙") | |
| Flag economy (speed jargon) | 标一开二; 废雷; 点一开一; 局部最优化操作 | [bbs/1262](https://saolei.wang/bbs/1262); [bbs/285](https://saolei.wang/bbs/285); [bbs/11409](https://saolei.wang/bbs/11409) | Not needed for the hint feature. Listed for completeness. |

---

## 5. How Chinese tutorials phrase deductions (verbatim)

These quotes are from the primary pages. The minesweeper.online sentences refer to colour-highlighted cells (黄格 = yellow cells, 紫格 = purple cells).

1. Single-number rule (B1): 「如果一个数字周围只剩下对应数量的格子，那么这些格子必须全是雷。」 ([help/patterns, 数数 (1)](https://minesweeper.online/cn/help/patterns))
2. Satisfied number and chord (B2): 「如果一个数字周围已经有了对应数量的雷，那么剩下的格子全部安全。如果那些雷已经标上，则可以通过双击这个数字一次性打开剩下所有格子。」 (same page, 数数 (2))
3. 1-1: 「看到最左边的 1 ，它周围有两个黄格，所以这两个黄格有 1 个雷。再看到左边第二个 1 ，它周围的两个黄格已经有了 1 个雷，所以剩下一格可以打开。」 (same page, 11 定式 (1))
4. 1-2: 「再看到 2 ，它周围的两个黄格有 1 个雷，所以剩下一格必须是雷。」 (same page, 21 定式 (1))
5. "At most" phrasing: 「受到 1 的影响，这两个紫格最多只能有 1 个雷。」 (same page, 21 定式 (2))
6. Exactly-one by squeezing: 「受 1 的影响，这三个紫格最多只有 1 个雷。受 2 的影响，这三个紫格最少也要有 1 个雷，所以这三格中恰好有 1 个雷。那么 2 这一侧剩下一格是雷，1 这一侧剩下三格安全。」 (same page, 12 减法)
7. Reduction: 「在左边一个雷的影响下， 2 实际上是 1 。…所以24实际上是 11 定式。」 (same page, 11 定式变形)
8. 1-2-1: 「121 定式实际上就是两个 12 定式。对左边的 12 应用 12 定式。对右边的 21 应用 21 定式。得到结论。」 (same page)
9. Mine counting: 「左上角显示剩余 6 雷，已经被数完了，所以剩下两格安全。」 (same page, 数雷)
10. Contradiction: 「假设黄格是雷。黄格是雷可以推出这整片区域有 6 雷，和剩余雷数矛盾，所以黄格必须安全。」 (same page, 穷举与假设)
11. Beginner tutorial voice: 「它就表示 它的周围有 2 颗雷，但咱已经分析出已经有一个是雷了…所以 咱就可以判断出 刚才那个雷的上面还是个雷」 ([saolei bbs/177](https://saolei.wang/bbs/177), 门世运, 2007)
12. 1-1 on saolei: 「在直排上出现的11序列，假如已知某个是雷，则另一个也是雷…假如已知某个不是雷，则另一个也不是雷」 ([bbs/362](https://saolei.wang/bbs/362)). The cell icons are images and are missing from the text.
13. Edge assumption: 「靠边方向加一排空格。」 and the known-mine rule 「某方块周围附加n颗雷，则该方块的数字在做判断时减去n，并排除是雷的方块。」 ([bbs/362](https://saolei.wang/bbs/362))
14. Guessing advice: 「死猜没有所谓正确的解法。因为几率都是对半开。所以碰到了就立刻猜，不要浪费时间。」 ([bbs/187](https://saolei.wang/bbs/187))
15. Hint feature on saolei.wang: 「新手提示：当卡住时，提示附近能确定的地雷和空」 ([saolei.wang/online](https://saolei.wang/online))

**Patterns in this phrasing [derived from 1-10]:**
- Each step names a number by its position and digit, then states a fact: 「看到…的 N ，它周围的 K 个…格有 M 个雷」.
- The conclusion follows with 所以: 「所以剩下 X 格必须是雷 / 必定安全 / 可以打开」.
- Bounds are written as 最多 / 最少 / 恰好 (at most / at least / exactly).
- A reduced number is written as 「在…雷的影响下，N 实际上是 N−k」.
- A named pattern is cited as 「这是 121 定式」 or 「应用 12 定式」.
- Classifiers: 1 个雷 / 一雷 / 两雷 / 一颗雷 (saolei often uses 颗). Cells take 格.

---

## 6. Mapping to Runesweeper's GLOSSARY terms [proposal]

| GLOSSARY term | Suggested zh-CN | Basis |
|---|---|---|
| Level | 级别 (values 初级/中级/高级/自定义) | minesweeper.online, Metasweeper, XP |
| Mode | 模式 | minesweeper.online 经典模式 / 无猜模式; Metasweeper 游戏模式 |
| Classic | 经典 | minesweeper.online 经典模式. See the caution about Metasweeper's 经典无猜 in section 3. |
| Patterns (mode) | 定式 (mode name) **[unattested as a mode name]** | 定式 = pattern everywhere |
| No Guess | 无猜 | all sources |
| Max Guess | **[unattested]**: 死猜 / 多猜 | no precedent. Not 可猜. |
| First click | 首次点击 (UI), 起手 (jargon) | zh.wikipedia; ms-toollib |
| Opening | 空 / 开空 (verb) | saolei.wang, ms-toollib. Watch the 空 = safe-cell sense. |
| Forced guess | 只能猜 / 必须猜 | ms-toollib comments |
| True 50/50 | 死猜 (2选1 when there are two cells) | minesweeper.online, saolei bbs/187 |
| Hint | 提示 | minesweeper.online, saolei.wang |
| Assisted game | 使用了提示的对局 **[proposal]** | minesweeper.online uses 不用提示完成 for the opposite |
| Game record | 对局记录 **[proposal]** | 记录 (log) vs 纪录 (best) distinction |
| History | 历史记录 | Metasweeper history plugin "历史记录" |

---

## 7. Unverified and unreachable

- **Windows XP official zh-CN strings**: no Microsoft source was reached. The strings shown (开局(N) F2, 标记(?)(M), 扫雷英雄榜(T), 自定义雷区, 重新计分(R), 匿名, 已破初级记录。请留尊姓大名。) come from clones that copy the XP UI. Four clones agree on 扫雷英雄榜(T): charlesmine-rs, richenyunqi/Mine-game, yizhuoyan/game-winmine-java and hanxi/MineSweeping. Only charlesmine-rs has 开局(N), 标记(?)(M) and the record dialog text; Mine-game has 新游戏(&N) and 标记(&M) instead. The XP Help menu items were not verified.
- **Zhihu** (张少武's four articles on 减法公式 and 121/1221 variations) returned 403 because of a bot check, and they were not read. The titles come from the links on minesweeper.online's Chinese Patterns page.
- **Bilibili videos** by record holders were not reached. The one Bilibili source used is a text column on solver algorithms ([cv5809834](https://www.bilibili.com/read/cv5809834)). It counts as secondary.
- The **TapTap** guide ([moment/492951218495163660](https://taptap.cn/moment/492951218495163660)) was posted by a now-deleted account. It counts as secondary, but its wording matches the primary pages.
- No source used terms for **triangle**, **dead cell**, **五五开** or **二选一**.
- saolei.wang tutorial diagrams are images. Sentences quoted from them lose the cell icons, so some quotes read with gaps.

# zh-CN pattern terms

The name a hint gives each technique and pattern, in English and zh-CN, kept beside the
translations in `src/strings.rs` (the `Tech*` keys). Chinese names the community uses are
taken as they are; a pattern with no Chinese name gets a descriptive one in the same style
(digits run together, then 定式), marked *descriptive*. The grade is minesweeper.online's No
Guess level; a Chinese pattern takes the level of the English pattern it extends.

Sources: minesweeper.online's pattern catalogue and its Chinese pages, 张砷镓's 扫雷定式及变化
(saolei.wang bbs/362), and the zh-CN terminology research notes.

| English | zh-CN | Catalogue entry | Grade | Basis |
|---|---|---|---|---|
| Single number | 单数字判雷 | B1 | Easy | descriptive (the site's 数数 (1)) |
| Satisfied number | 单数字判空 | B2 | Easy | descriptive (the site's 数数 (2)) |
| 1-1 pattern | 11定式 | 1–1 | Medium | community |
| 1-1 at the edge | 边部11定式 | 1–1 against the edge | Medium | community (saolei) |
| 1-1+ pattern | 多格11定式 | 1–1+ | Medium | descriptive |
| 1-2 pattern | 12定式 | 1–2 | Medium | community |
| 1-2+ pattern | 多雷12定式 | 1–2+ | Medium | descriptive |
| 1-2C pattern | 交叉12定式 | 1–2C | Medium | descriptive |
| 1-2C+ pattern | 交叉多雷12定式 | 1–2C+ | Medium | descriptive |
| 1-2-1 pattern | 121定式 | 1–2–1 | Medium | community |
| 1-2-2-1 pattern | 1221定式 | 1–2–2–1 | Medium | community |
| 2-1-2 pattern | 212定式 | (Chinese only) | Medium | community (saolei) |
| 2-1-1 at the edge | 边部211定式 | (Chinese only) | Medium | community (saolei) |
| Reduction to a 1-1 | 减雷变11定式 | 1–1R | Hard | descriptive (减雷, 化归) |
| Reduction to a 1-2 | 减雷变12定式 | 1–2R | Hard | descriptive (减雷, 化归) |
| Reduction to a 1-2-1 | 减雷变121定式 | 1–2–1R | Hard | descriptive (减雷, 化归) |
| 2-2 between mines | 夹雷22定式 | (Chinese only) | Hard | community (saolei) |
| 2-2-1 next to a mine | 靠雷221定式 | (Chinese only) | Hard | community (saolei) |
| 3-1 next to a mine | 靠雷31定式 | (Chinese only) | Hard | community (saolei) |
| 4-1 next to mines | 靠雷41定式 | (Chinese only) | Hard | community (saolei) |
| Hole H1 | 挖坑定式 (1) | H1 | Hard | community (挖坑) |
| Hole H2 | 挖坑定式 (2) | H2 | Hard | community (挖坑) |
| Hole H3 | 挖坑定式 (3) | H3 | Hard | community (挖坑) |
| Triangle T1 | 三角定式 (1) | T1 | Evil | descriptive |
| Triangle T2 | 三角定式 (2) | T2 | Evil | descriptive |
| Triangle T3 | 三角定式 (3) | T3 | Evil | descriptive |
| Triangle T4 | 三角定式 (4) | T4 | Evil | descriptive |
| Triangle T5 | 三角定式 (5) | T5 | Evil | descriptive |
| 1-3-1 corner | 角131定式 | 1–3–1 corner | Evil | community |
| 2-2-2 corner | 角222定式 | 2–2–2 corner | Evil | community |
| 1>2<1 pattern | 隔行的121定式 | 1>2<1 | Evil | community |
| T-pattern | T形定式 | T–pattern | Evil | descriptive |
| Dependency chain | 连续减法 | Dependency Chain | Evil | community (chained subtraction) |
| Mine counting | 剩余雷数 | Mine counting | Evil | community (根据剩余雷数; 数雷 is ambiguous) |
| Combinations | 穷举与假设 | Combinations | Evil | community (the site's Chinese title) |
| Chain of steps | 多步推理 | (no entry: plain logic) | Hard | descriptive |
| Proof by contradiction | 假设法 | (no entry: plain logic) | Evil | community |

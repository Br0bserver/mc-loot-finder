# mc-loot-finder

一个独立 CLI：在**不生成或加载目标世界区块**的情况下，根据 Minecraft Java 世界种子预测结构内方块容器、`LootTableSeed` 和最终战利品。当前只支持 Minecraft Java **26.1.2**。

已支持 16 个结构入口：

- 主世界：远古城市、沙漠神殿、丛林神庙、雪屋、废弃传送门、试炼密室、沉船、海底废墟、村庄、埋藏的宝藏、掠夺者前哨站、林地府邸；
- 下界：堡垒遗迹、下界要塞、废弃传送门；
- 末地：末地城。

村庄会自动处理五种群系变体，沉船和海底废墟也会按原版选择实际变体。只处理方块容器，不处理箱子矿车等实体容器，因此不支持废弃矿井。

它不是模组。程序直接使用 Minecraft 26.1.2 的原版群系、噪声、结构生成、放置和战利品数据来核对结果；遇到尚未实现的原版规则会报错停止，不会返回看似可用的近似结果。

## 立即使用

需要 Java 25。Gradle Wrapper 会复用本机已有的 Fabric Loom/Minecraft 缓存。

```bash
./gradlew installDist

# 主世界：幽静纹饰
build/install/mc-loot-finder/bin/mc-loot-finder find \
  --seed 0 --structure ancient_city --radius 5000

# 下界：下界合金升级模板
build/install/mc-loot-finder/bin/mc-loot-finder find \
  --seed 0 --structure bastion_remnant --radius 2000

# 主世界：沙丘纹饰
build/install/mc-loot-finder/bin/mc-loot-finder find \
  --seed 0 --structure desert_pyramid --radius 5000

# 主世界：恼鬼纹饰
build/install/mc-loot-finder/bin/mc-loot-finder find \
  --seed 0 --structure woodland_mansion --radius 10000
```

种子 `0` 的回归向量包括：

```text
# ancient_city
892 -47 1286
-519 -48 2283

# bastion_remnant（第一条命中，下界坐标）
167 69 -229  minecraft:chests/bastion_other

# desert_pyramid（第一条沙丘纹饰命中）
10 59 -2996  minecraft:chests/desert_pyramid
```

`--radius` 是对应维度中以 `--center-x/--center-z` 为中心的方块半径。默认结构为 `ancient_city`，不指定 `--item` 时会使用该结构预设的代表性战利品。可运行 `explain` 查看所有结构的程序名。

所有搜索命令支持 `--json`。需要纯 JSON 时建议使用 `installDist` 生成的脚本，避免 Gradle 构建输出混入 stdout。

## 命令

```text
candidates --seed N [--structure NAME --center-x X --center-z Z --radius BLOCKS --limit N --json]
chests --seed N [--structure NAME --center-x X --center-z Z --radius BLOCKS --limit N --json]
find --seed N [--structure NAME --item ITEM_ID --center-x X --center-z Z --radius BLOCKS --limit N --json]
loot --loot-seed N [--table LOOT_TABLE_ID --json]
container-seed --seed N --chunk-x X --chunk-z Z [--structure NAME --ordinal N --json]
explain
```

- `candidates` 只做极快的 random-spread 候选筛选。
- `chests` 用原版内存世界生成验证结构并列出方块容器、LootTable 和 LootTableSeed。命令名沿用 `chests`，但试炼密室结果也包括发射器和饰纹陶罐。
- `find` 执行对应战利品表并筛选目标物品。
- `loot` 单独重放一个受支持 LootTable 的非零 LootTableSeed。
- `--limit` 只限制显示条数，不限制搜索与统计。

堡垒和下界要塞共享 `nether_complexes` 结构集。CLI 会复现原版 `fortress:bastion = 2:3` 的加权选择及群系失败回退，而不是把所有候选误判成堡垒。

## 实现边界

主世界、下界和末地共用同一条搜索流程。候选位置先用独立的快速算法筛选，再由原版代码验证群系、结构集选择和结构放置。堡垒遗迹与下界要塞共享同一个原版结构集，程序会复现其加权选择与群系失败回退，不会把同一候选同时算作两种结构。

战利品解释器直接读取 26.1.2 原版数据包 JSON。目前公开支持 46 张实际容器战利品表，包含试炼密室普通箱、奖励箱、三类发射器和饰纹陶罐；奖励箱内部引用的子表由解释器自动处理，不会被误报成地图里的额外容器。

对不在支持清单内或含未实现规则的表会直接报错，不会给近似结果。

当前仍不支持修改世界生成/战利品的数据包和其他 Minecraft 版本。若两个同类结构极罕见地在同一装饰区块内共享容器随机流，CLI 会明确报错；LootTableSeed 恰好为 `0` 时会跳过，因为原版将它视为实时随机序列哨兵。

## 验证

```bash
./gradlew test
```

验证包括：

- 16 个结构入口的 random-spread 参数（含 triangular 分布）与原版实现对拍；
- LegacyRandom、xoroshiro128++、装饰 RNG 对拍；
- 远古城市 9 座、204 只容器的原版实际放置对拍；
- 桥梁、疣猪兽棚、藏宝室样本覆盖四张堡垒表，20 只容器实际放置对拍；
- 3 座沙漠神殿、12 只过程式箱子的实际放置对拍；
- 通用放置后端与快速路径在远古城市、堡垒、沙漠神殿样本上逐箱一致；
- 主世界和下界废弃传送门、试炼密室、沉船、海底废墟、村庄、前哨站等固定结果测试；
- 46 张支持表各 2048 个 seed 与原版 LootTable 比较完整物品序列和数量；
- 运行时临时目录关闭后自动删除，以及极端搜索坐标拒绝测试。

理论与随机链详见 [docs/theory.md](docs/theory.md)。

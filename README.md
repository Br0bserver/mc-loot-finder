# mc-loot-finder

一个独立 CLI：在**不生成或加载目标区块**的情况下，根据 Minecraft Java 世界种子预测结构箱子、LootTableSeed 和最终战利品。当前支持 Minecraft Java **26.1.2**：

- 远古城市：幽静盔甲纹饰等；
- 堡垒遗迹：下界合金升级模板、猪鼻纹饰和四类堡垒战利品表；
- 沙漠神殿：沙丘盔甲纹饰、附魔金苹果等；
- 林地府邸：恼鬼盔甲纹饰等；
- 丛林神庙：荒野盔甲纹饰，包括箱子和发射器；
- 雪屋：地下室箱子；
- 末地城：尖塔盔甲纹饰等。

它不是模组。Java 25 层直接调用同版本原版群系、噪声、结构生成和放置代码；候选定位与随机数核心保持独立，后续可按现有对拍向量迁移热点，无需重写 CLI。

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

`--radius` 是对应维度中以 `--center-x/--center-z` 为中心的方块半径。默认结构为 `ancient_city`；默认目标随结构变化：远古城市为幽静纹饰，堡垒为下界合金升级模板，沙漠神殿为沙丘纹饰，林地府邸为恼鬼纹饰。

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
- `chests` 用原版内存世界生成验证结构并列出箱子、LootTable 和 LootTableSeed。
- `find` 执行对应战利品表并筛选目标物品。
- `loot` 单独重放一个受支持 LootTable 的非零 LootTableSeed。
- `--limit` 只限制显示条数，不限制搜索与统计。

堡垒和下界要塞共享 `nether_complexes` 结构集。CLI 会复现原版 `fortress:bastion = 2:3` 的加权选择及群系失败回退，而不是把所有候选误判成堡垒。

## 最小重构后的边界

版本层现在是可扩展的 `StructureSpec` 目录，描述结构 ID、维度、random-spread 类型与参数、装饰 RNG step/index、共享结构集权重、箱子扫描后端、战利品表和默认目标。主世界、下界和末地共用同一 CLI 管线。远古城市和堡垒保留经过对拍的 Jigsaw 快速路径；通用正确性后端直接执行原版 `StructureStart.placeInChunk`，记录它创建的容器，目前已跨过程式 Piece、普通模板、府邸组合模板和末地城模板工作。增加同类结构通常只需配置，而不再写一套箱子坐标逻辑；碰到新的世界交互时才扩展记录世界。

战利品解释器直接读取 26.1.2 原版数据包 JSON，并只实现、验证当前支持表实际用到的函数子集。除下列原有七张表外，现已加入 `jungle_temple`、`jungle_temple_dispenser`、`igloo_chest` 和 `end_city_treasure`：

- `minecraft:chests/ancient_city`
- `minecraft:chests/bastion_bridge`
- `minecraft:chests/bastion_hoglin_stable`
- `minecraft:chests/bastion_other`
- `minecraft:chests/bastion_treasure`
- `minecraft:chests/desert_pyramid`
- `minecraft:chests/woodland_mansion`

对不在白名单或含未支持语义的表会失败关闭，不会给近似结果。

当前仍不支持修改世界生成/战利品的数据包和其他 Minecraft 版本。若两个同类结构极罕见地在同一装饰区块内共享容器随机流，CLI 会明确报错；LootTableSeed 恰好为 `0` 时会跳过，因为原版将它视为实时随机序列哨兵。

## 验证

```bash
./gradlew test
```

验证包括：

- 七种 random-spread 参数（含府邸、末地城 triangular 分布）与原版实现对拍；
- LegacyRandom、xoroshiro128++、装饰 RNG 对拍；
- 远古城市 9 座、204 只容器的原版实际放置对拍；
- 桥梁、疣猪兽棚、藏宝室样本覆盖四张堡垒表，20 只容器实际放置对拍；
- 3 座沙漠神殿、12 只过程式箱子的实际放置对拍；
- 通用放置后端与快速路径在远古城市、堡垒、沙漠神殿样本上逐箱一致；
- 种子 0 林地府邸固定向量：起点区块 `(-221,-52)`、7 只战利品箱；
- 十一张支持表各 2048 个 seed 与原版 LootTable 比较完整物品序列和数量。

理论与随机链详见 [docs/theory.md](docs/theory.md)。

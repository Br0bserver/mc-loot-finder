# mc-loot-finder

`mc-loot-finder` 是一个针对 Minecraft Java `26.1.2` 的命令行工具。它根据世界种子定位结构，在内存中调用原版结构生成代码，列出结构内的方块容器和考古方块，并重放对应的战利品表。

基于 Pumpkin 的实验性 Rust 版本位于 `experimental/pumpkin-rust` 分支，仅维护源码并进行 CI 验证。

它不读取存档，也不加载目标世界区块。输出包括容器坐标、LootTable、LootTableSeed，以及指定物品是否会出现在战利品中。

## 构建

需要 Java 25。

```bash
./gradlew installDist
```

生成的程序位于：

```text
build/install/mc-loot-finder/bin/mc-loot-finder
```

## 命令

```text
candidates       快速列出可能生成结构的区块
chests           验证结构并列出方块容器
archaeology      列出可疑沙子和可疑沙砾
find             在容器和考古方块中搜索指定物品
loot             用 LootTable 和 LootTableSeed 重放战利品
container-seed   计算部分结构的容器种子快捷结果
explain          查询支持的结构和具体配置
```

所有命令的详细参数和默认值由程序自己提供：

```bash
build/install/mc-loot-finder/bin/mc-loot-finder help
build/install/mc-loot-finder/bin/mc-loot-finder explain
build/install/mc-loot-finder/bin/mc-loot-finder explain --structure trial_chambers
```

`explain --structure NAME --json` 输出单个结构的机器可读配置，包括维度、默认目标物品、放置参数、可用战利品表和 `container-seed` 是否支持。

## 示例

搜索远古城市中的幽静纹饰：

```bash
build/install/mc-loot-finder/bin/mc-loot-finder find \
  --seed 0 \
  --structure ancient_city \
  --item minecraft:silence_armor_trim_smithing_template \
  --radius 5000
```

列出试炼密室容器：

```bash
build/install/mc-loot-finder/bin/mc-loot-finder chests \
  --seed 0 --structure trial_chambers --radius 2000
```

列出沙漠神殿中的可疑沙子：

```bash
build/install/mc-loot-finder/bin/mc-loot-finder archaeology \
  --seed 0 --structure desert_pyramid --radius 5000
```

直接重放一个容器的战利品：

```bash
build/install/mc-loot-finder/bin/mc-loot-finder loot \
  --table minecraft:chests/ruined_portal \
  --loot-seed -6371263386669125558
```

脚本处理时加 `--json`。`--limit` 只限制显示数量，不限制实际搜索。

## LootTableSeed

结构生成时，容器通常只保存 `LootTable` 和 `LootTableSeed`，玩家打开容器时才根据这两项生成物品。相同版本中，表和非零种子相同，生成的物品结果就相同。

`LootTableSeed` 不是世界种子，也不是区块坐标。一个结构里的不同容器通常有不同的种子。

种子为 `0` 是原版的实时随机哨兵，无法精确预测；`find` 会跳过这类容器。

## 支持范围

当前支持：远古城市、堡垒遗迹、沙漠神殿、丛林神庙、雪屋、末地城、要塞、古迹废墟、主世界和下界废弃传送门、试炼密室、沉船、海底废墟、下界要塞、村庄、埋藏的宝藏、掠夺者前哨站、林地府邸，以及沙漠井。

考古方块支持沙漠神殿、冷暖海底废墟、古迹废墟和沙漠井。`archaeology` 列出方块位置、类型、LootTable 和 LootTableSeed；`find` 会同时搜索普通容器和考古方块。沙漠井虽然沿用 `--structure desert_well` 参数，但内部按原版的区块地表特征生成规则搜索，并未把它近似成普通结构。

只处理方块容器，不处理箱子矿车等实体容器，因此不支持废弃矿井。尚不支持地牢；地牢属于依赖真实洞穴地形的 `MonsterRoomFeature`，目前不能按普通结构搜索。只支持原版 Minecraft Java `26.1.2`，不读取自定义世界生成或战利品数据包。

`candidates` 只提供候选区块，不保证结构一定生成。要塞使用原版同心圆和群系修正，因此查询要塞候选时也需要初始化原版 Worldgen。遇到尚未实现的原版语义，程序会停止并报错，不返回近似结果。

## 测试

```bash
./gradlew test
```

测试覆盖结构放置、随机数、固定结果和战利品表与原版的对拍。

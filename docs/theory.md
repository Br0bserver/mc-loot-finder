# 26.1.2 理论链路

## 为什么未生成、未加载也能预测

箱子的最终 `Items` 通常不会在结构模板落地时立刻生成。结构放置把 `LootTable` 写入容器 NBT，并从当前结构装饰随机流调用 `nextLong()` 写入 `LootTableSeed`。玩家首次访问容器时，原版再使用这个 seed 展开战利品表。

因此问题可以拆成五个确定性阶段：

```text
世界种子
  -> random-spread 或同心圆候选起始区块
  -> 群系验证 + 原版结构起点/结构片段
  -> 模板或过程式 Piece 中的箱子坐标与区块内放置顺序
  -> 装饰随机流产生 LootTableSeed
  -> LootTableSeed 重放战利品表
```

只要版本、世界设置和数据包固定，这些阶段都可复现；无需先让存档生成目标区块。

## 结构版本参数

### 远古城市

26.1.2 原版数据给出的关键参数：

| 参数 | 值 |
|---|---:|
| random-spread spacing | 24 chunks |
| separation | 8 chunks |
| salt | 20083232 |
| spread type | linear |
| generation step | `underground_decoration`，序号 7 |
| 该 step 内结构索引 | 0 |
| Jigsaw size | 7 |
| start height | Y = -27 |

程序启动后会用加载出的原版结构注册表再次校验 step/index，避免静态常量静默漂移。

### 堡垒遗迹

| 参数 | 值 |
|---|---:|
| dimension | `minecraft:the_nether` |
| random-spread spacing | 27 chunks |
| separation | 4 chunks |
| salt | 30084232 |
| structure set | fortress 权重 2，bastion_remnant 权重 3 |
| generation step | `surface_structures`，序号 4 |
| 该 step 内结构索引 | 0 |
| Jigsaw size | 6 |
| start height | Y = 33 |

堡垒候选位置属于共享的 `nether_complexes` 结构集。原版以世界种子和候选 chunk 初始化 LegacyRandom，按权重挑选结构；若挑中的结构不能通过群系/结构验证，会移除该项并继续挑选剩余项。通用运行时完整复现此循环。

### 沙漠神殿

| 参数 | 值 |
|---|---:|
| dimension | `minecraft:overworld` |
| random-spread spacing | 32 chunks |
| separation | 8 chunks |
| salt | 14357617 |
| generation step | `surface_structures`，序号 4 |
| 该 step 内结构索引 | 1 |
| structure piece | `DesertPyramidPiece`，21×21 |

它不是 Jigsaw，也没有独立的神殿 NBT 模板。原版会创建一个过程式 `DesertPyramidPiece`，固定按北、东、南、西顺序尝试密室中的四只箱子。通用记录世界直接执行这个原版 Piece 的放置过程，并用原版噪声生成器回答地表高度查询。

### 林地府邸

| 参数 | 值 |
|---|---:|
| dimension | `minecraft:overworld` |
| random-spread spacing | 80 chunks |
| separation | 20 chunks |
| salt | 10387319 |
| spread type | triangular |
| generation step | `surface_structures`，序号 4 |
| 该 step 内结构索引 | 5 |
| structure ID | `minecraft:mansion` |

府邸不是 Jigsaw。结构起点会组装大量模板 Piece，并在数据标记中创建箱子和灾厄村民/悦灵。记录世界执行原版模板放置，只关闭与箱子预测无关的实体类型；箱子坐标、LootTable 和 LootTableSeed 仍由原版放置代码产生。

### 要塞

| 参数 | 值 |
|---|---:|
| dimension | `minecraft:overworld` |
| placement | `concentric_rings` |
| distance | 32 chunks |
| spread | 3 |
| count | 128 |
| loot tables | corridor、crossing、library |

要塞不使用 random-spread 网格。原版先生成同心圆位置，再把候选移动到偏好的群系附近。CLI 直接读取原版 `ChunkGeneratorStructureState` 的最终 128 个位置，按搜索中心和半径过滤，然后执行原版 `StrongholdStructure` 和过程式 `StrongholdPieces`。因此要塞候选查询也需要初始化原版 Worldgen。

## 三条随机流

### 1. 结构候选位置

每个 random-spread 区域用 48 位 Java LCG（原版 `LegacyRandomSource`）选择一个候选 chunk。这类结构的 `candidates` 只执行这一步，因此很快，但候选尚未经过群系、共享结构集选择或 Jigsaw 验证。要塞是例外：它的同心圆位置包含群系修正，需要原版 Worldgen 状态。

### 2. 原版结构布局

候选通过结构集和群系检查后，CLI 调用原版 26.1.2 `Structure.generate`。远古城市和堡垒使用基于世界种子和起始 chunk 的 Jigsaw 布局；沙漠神殿使用原版 `SinglePieceStructure`。CLI 分别加载主世界和下界的注册表、噪声路由、模板池与模板，没有自行近似群系或结构有效性。

### 3. 容器 LootTableSeed

区块装饰使用 `WorldgenRandom(XoroshiroRandomSource)`：

```text
decorationSeed = setDecorationSeed(worldSeed, chunkX * 16, chunkZ * 16)
featureSeed    = decorationSeed + structureIndex + 10000 * decorationStep
LootTableSeed = nextLong()  // 每个被放置的随机容器消耗一次
```

一个容易漏掉的细节是：模板中的容器即使没有 `LootTable`，原版放置代码仍会为它调用一次 `nextLong()`，所以它仍占用 ordinal。CLI 不输出这类不可搜索的装饰箱，但会让它继续占用 ordinal。

另一个细节是 `WorldgenRandom.nextLong()` 继承自 `BitRandomSource`，在 xoroshiro delegate 上由两次 `next(32)` 组合而成；不能简单等同于 xoroshiro 的一次原生 `nextLong()`。

沙漠神殿还有一个结构专属前缀：`DesertPyramidPiece.postProcess` 在尝试箱子前调用 `nextInt(3)`，用于整体高度的 0–2 格下沉偏移。因此它的第一只箱子并不是 feature seed 后的第一个 `nextLong()`；扫描器先精确消费这次 `nextInt(3)`，再按落入同一装饰 chunk 的箱子顺序取 `nextLong()`。

## 战利品表

26.1.2 的 `minecraft:chests/ancient_city` 先从主池抽 5–10 次，再从稀有池抽一次。稀有池权重是：

| 结果 | 权重 |
|---|---:|
| 空 | 75 |
| Ward 纹饰 | 4 |
| Silence 纹饰 | 1 |

所以“每箱 1/80”只描述边际概率，不能直接拿 LootTableSeed 做一个独立的 `nextInt(80)`。主池的数量、耐久、随机附魔和等级附魔都会先消耗 RNG。当前解释器逐个模拟主池函数，并把复杂的附魔 RNG 消耗委托给原版 26.1.2 `EnchantmentHelper`。

堡垒四张表也在模板池之后设置猪鼻纹饰池，以及下界合金升级模板池。普通堡垒表的升级模板池是空项权重 9、模板权重 1；藏宝室表必定给出一张。但这些池之前仍有数量、耐久和随机附魔消费，因此不能只看最后一个 `nextInt(10)`。

当前解释器从 Minecraft JAR 内的原版 JSON 构建池、权重、数量提供器和函数序列；随机附魔的兼容性过滤与复杂等级附魔继续委托原版注册表/`EnchantmentHelper`。测试会把当前十一张支持表与原版 LootTable 对同一批 seed 比较完整物品序列和数量，覆盖古城、堡垒、沙漠神殿、府邸、丛林神庙、雪屋和末地城。这曾实际发现并修复一个边界：原版 `Mth.nextInt(min,max)` 在 `min == max` 时不推进 RNG。

## 当前边界与后续拆分

容器 ordinal 当前按单个 `StructureStart` 在每个装饰 chunk 内计算。若两个同类结构的片段覆盖同一装饰 chunk，需要按原版 structure-reference 集合顺序把两个起点的容器流合并；这部分还未实现。CLI 会检测这种情况并终止，避免把不确定结果标成精确。

集成测试不只扫描模板：通用正确性后端会让原版 26.1.2 的 `StructureStart.placeInChunk` 把结构实际放进一个捕获世界，并读取原版创建的容器方块实体。它已与快速路径在远古城市、堡垒、沙漠神殿样本上逐箱对拍；种子 0 的林地府邸还固定验证起点区块 `(-221,-52)` 的 7 只战利品箱及其坐标、LootTableSeed 和 ordinal。

性能拆分已清楚：

- 纯核心：候选位置、LegacyRandom、xoroshiro、装饰 seed；无 Minecraft 运行时依赖，适合移植到 Rust/C++ 并大规模并行。
- 原版 oracle：群系、结构生成、结构放置、模板、地表高度和附魔规则；启动加载较重，但最大限度减少版本规则复刻错误。
- CLI：稳定输入输出层；后续可以让快速后端批量筛选，再把少量候选交给原版 oracle 验证。

因此“先用 Java 打通，再重构热点”是有价值的：原版对拍测试已经能作为其他语言实现的行为规范，而不必凭猜测重写整套世界生成。

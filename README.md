# mc-loot-finder

> 这是基于 SteelMC `v0.9.0+mc26.1` 的实验性 Rust 分支，只维护源码并通过 GitHub Actions 验证，不发布正式 Release。

`mc-loot-finder` 是一个用于 Minecraft Java 26.1.2 的命令行工具。当前分支仍在审查中：
已按独立 Java 26.1.2 oracle 恢复固定向量，正在等待新的 Linux/Windows CI 验证；
实现目标是精确定位远古城市、堡垒遗迹、沙漠神殿、冰屋、沉船、村庄、埋藏宝藏和
掠夺者前哨站，CI 全绿前不得宣称已完成原版兼容。

程序不读取存档，也不需要安装 Java。GitHub Actions 生成的 Linux 和 Windows 构建产物只用于 CI 验证，可能随时失效或被清理，不作为正式发行版本提供。

## 使用

搜索远古城市中的幽静纹饰：

```bash
mc-loot-finder find \
  --seed 114514 \
  --structure ancient_city \
  --item minecraft:silence_armor_trim_smithing_template \
  --radius 5000
```

搜索堡垒遗迹中的下界合金升级锻造模板：

```bash
mc-loot-finder find \
  --seed 0 \
  --structure bastion_remnant \
  --item minecraft:netherite_upgrade_smithing_template \
  --center-x 1000 \
  --center-z 520 \
  --radius 0
```

列出远古城市内的容器及其 LootTableSeed：

```bash
mc-loot-finder chests \
  --seed 114514 \
  --structure ancient_city \
  --radius 5000
```

搜索埋藏宝藏中的海洋之心：

```bash
mc-loot-finder find \
  --seed 0 \
  --structure buried_treasure \
  --item minecraft:heart_of_the_sea \
  --center-x 8 \
  --center-z -344 \
  --radius 0
```

按结构区块重算同一个宝箱的 LootTableSeed：

```bash
mc-loot-finder container-seed \
  --seed 0 \
  --structure buried_treasure \
  --chunk-x 0 \
  --chunk-z -22
```

搜索沉船中的地图：

```bash
mc-loot-finder find \
  --seed 0 \
  --structure shipwreck \
  --item minecraft:map \
  --center-x 232 \
  --center-z 136 \
  --radius 0
```

运行 `mc-loot-finder help` 查看命令，运行 `mc-loot-finder explain` 查看默认参数和支持范围。脚本调用时可加 `--json`；`--limit` 只限制显示条数，不会缩小实际搜索范围。

## 当前范围

- 目标是精确支持 Minecraft Java 26.1.2 的远古城市、堡垒遗迹、沙漠神殿、冰屋、沉船、村庄、埋藏宝藏和掠夺者前哨站；未通过独立原版向量的结构不得宣称精确支持。
- `candidates` 可以快速计算其他已登记结构的候选区块，但不验证结构是否实际生成。
- 不读取已有世界，不处理数据包，也不处理箱子矿车等实体容器。
- 固定世界种子、箱子位置、LootTableSeed 和战利品结果必须通过原版结果回归测试。

源码使用 Rust，世界生成链路基于固定 revision 的 SteelMC worldgen crates。许可证为 GPL-3.0-only。

## 开发

- 模块结构：`src/catalog.rs` 是扫描能力和装饰种子参数的目录入口；
  `src/worldgen.rs` 只保留扫描器入口，`src/worldgen/{buried_treasure,jigsaw_scan,single_piece,shipwreck}.rs`
  负责各类结构扫描，`surface_probe.rs` 负责 SteelMC 地形方块状态和 surface-rule 探测，
  `template_scan.rs` 统一处理模板旋转、容器随机消耗和数据标记，`chests.rs` 处理
  Jigsaw 容器事件、去重和种子分配。
- 本机内存不足以编译完整依赖树，因此本地只允许 `cargo fmt` 和静态检查；
  编译、测试、clippy 和平台 smoke 全部由 GitHub Actions 验证。
- Linux 和 Windows 均调用 `ci/smoke.py` 执行同一组行为断言；CI 通过后可用
  `gh run download` 拉取 Linux artifact 在本地复验。
- 命令的退出码和 `--json` 输出格式是兼容性契约，改动需同步更新
  `ci/smoke.py` 与 `src/output.rs` 中的线格式测试。
- 后端迁移审查和未解决风险记录在
  [`docs/STEELMC_BACKEND_REVIEW.md`](docs/STEELMC_BACKEND_REVIEW.md)。

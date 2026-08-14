# mc-loot-finder

> 这是基于 Pumpkin fork 的实验性 Rust 分支，只维护源码并通过 GitHub Actions 验证，不发布正式 Release。

`mc-loot-finder` 是一个用于 Minecraft Java 26.1.2 的命令行工具。当前独立版可以根据世界种子精确定位远古城市和堡垒遗迹，列出其中的容器，并搜索指定战利品。

程序不读取存档，也不需要安装 Java。Linux x86_64 和 Windows x86_64 测试二进制由 GitHub Actions 生成，仅用于验证和试用。

Linux 版使用静态链接，下载后运行：

```bash
chmod +x mc-loot-finder-linux-x86_64
./mc-loot-finder-linux-x86_64 help
```

Windows 版直接运行：

```powershell
.\mc-loot-finder-windows-x86_64.exe help
```

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

运行 `mc-loot-finder help` 查看命令，运行 `mc-loot-finder explain` 查看默认参数和支持范围。脚本调用时可加 `--json`；`--limit` 只限制显示条数，不会缩小实际搜索范围。

## 当前范围

- 精确支持 Minecraft Java 26.1.2 的远古城市和堡垒遗迹定位、结构布局、方块容器和战利品。
- `candidates` 可以快速计算其他已登记结构的候选区块，但不验证结构是否实际生成。
- 不读取已有世界，不处理数据包，也不处理箱子矿车等实体容器。
- 固定世界种子、箱子位置、LootTableSeed 和战利品结果均通过原版结果回归测试。

源码使用 Rust，世界生成链路基于项目维护的 Pumpkin fork。许可证为 GPL-3.0-only。

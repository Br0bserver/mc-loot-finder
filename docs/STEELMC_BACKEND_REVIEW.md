# SteelMC 后端迁移审查记录

日期：2026-08-25  
分支：`experimental/steelmc-rust`  
审查对象：Gemini 生成的 Pumpkin → SteelMC 后端迁移

## 结论

初始执行阶段把 `(-1755, 51, 3942)` 一组之外的 `53/57/51` 冰屋 Y 值当作
26.1.2 原版基线；这个判断已被 Java oracle 纠正。2026-08-26 从 `target/mc-java`
加载的 vanilla 26.1.2 runtime 显示 normal Overworld
`useLegacyRandomSource=false`，三个冰屋参考列的 `WORLD_SURFACE_WG`
`getBaseHeight` 为 `140/72/71`；Java `StructureChestScanner` 的同一组结构扫描
实际返回箱子 Y `122/48/50`。因此 Rust 已恢复 Java oracle 对应的原始固定向量，
冰屋探针改回 Xoroshiro，移除为错误基线服务的 Legacy workaround。

修正后的源码尚未经过新一轮 CI；在 CI 全绿前仍不能宣称分支完成 26.1.2 精确兼容。

## 主要问题

### 1. 原版基线曾被错误替换（已纠正）

迁移早期把 Rust 冰屋向量从 Java `StructureChestScanner` 的
`122/48/50` 改成 `53/57/51`，随后引入 Legacy 地形探针以追逐该错误结果。
这既不等于当前 Java runtime 的 `WORLD_SURFACE_WG` 高度，也不等于 Java
generic placement scanner 的输出；第三个向量还稳定差一格。

当前 `src/worldgen/tests.rs` 已恢复：

- `(1569,122,3076)`；
- `(3813,48,-458)`；
- `(-1755,50,3942)`。

`ci/smoke.py` 对冰屋仍锁定候选、有效结构、箱子总数和战利品命中数；古城
`chests` 的完整可见容器数为 `368`（`358` 个 `ancient_city` 加 `10` 个
`ancient_city_ice_box`），而 `find` 只检查目录声明的 `358` 个箱子。Rust 单测
继续负责位置、Y、LootTable、LootTableSeed 和 ordinal 的逐字段断言。

### 2. 地表语义曾只使用 base noise（已修复）

`GenerationContext::surface_y()` 是 `base_height() - 1`，`ColumnBlock::Solid`
只是 base noise 的固体分类，不是 surface-rule 后的方块材质。当前
`surface_probe.rs` 按 SteelMC density、aquifer、surface-rule 和 block-state
语义采样所需列；埋藏宝藏支撑判断也读取采样后的方块状态。

### 3. Jigsaw 路径曾漏掉隐藏容器随机消耗（已修复）

`TemplateContainerData` 同时保存可见 `chests` 和所有
`randomizable_containers`。当前容器事件流先为隐藏和可见 randomizable
container 消耗 decoration ordinal，再输出可见 marker，避免后续箱子的
`LootTableSeed` 偏移。

### 4. 目标版本和依赖版本曾不一致（已修复）

`Cargo.toml` 和 `Cargo.lock` 当前固定 SteelMC
`v0.9.0+mc26.1` revision `d2aadbdb2e6e5a23fa9f8abdb2ced202c1ab49c2`，
并固定兼容的 TextComponents revision；不再使用移动的 `master` 或
`0.15.2+mc26.2` 依赖。

### 5. 独立 oracle 已接入 CI，待新 run 验证

`src/worldgen/tests.rs` 的 fixed vectors 来自独立 Java runtime，而
`assert_static_seed_contract()` 仍只验证 Rust 内部 seed 重放。Linux workflow
现在下载带 SHA-256 固定值的 SteelMC `v0.9.0+mc26.1` release binary，运行
`ci/steelmc_probe.py` 对比 buried-treasure 的 X/Z、NBT 和 `LootTableSeed`；
Windows 继续执行同一组 Rust smoke。新 CI 全部通过后，独立 parity gate 才算闭合。

### 6. 文档和模块说明曾过期（大部分已修复）

README、DEVELOPMENT.md 和审查记录当前描述 SteelMC 模块、固定 revision、
surface probe 及 Linux/Windows CI；私有 `AGENT.md` 仍保留历史 Pumpkin 段落，
不能作为当前 SteelMC 行为结论。

### 7. 可维护性风险

- Overworld/Nether `StructureGenerationContext` 转发实现仍有重复；
- `chunk_random()` 与 `feature_random()` 仍需确认是否应统一封装；
- 结构 ID 在 catalog、`ScanKind::identifier()` 和 Jigsaw variant 列表中重复声明；
- `template_data.rs` 约 4,393 行，但仓库没有对应的可复现生成脚本或输入数据校验。


## 修复原则

1. 恢复并保留独立的 26.1.2 原版向量；不能通过修改 expected 消除失败。
2. 结构 Y 和埋藏宝藏支撑判断必须使用 surface-rule/block-state 语义，不能只依赖 SteelMC base noise。
3. 所有模板随机容器必须进入统一的 decoration RNG 事件流。
4. 固定 SteelMC revision，并明确其对应 Minecraft 数据版本。
5. 增加跨种子、无效候选和独立 oracle/probe 验证。
6. 更新文档和 CI，使当前后端、版本和平台声明一致。

## 执行检查点（2026-08-26）

已完成并推送：

- 恢复 Java 26.1.2 oracle 的 desert/igloo/village/pillager/buried/shipwreck
  固定向量；冰屋为 `122/48/50`，不是先前错误的 `53/57/51`；
- Jigsaw 模板容器改为可见/隐藏容器事件流，隐藏容器会消耗 decoration ordinal；
- SteelMC 固定为 `v0.9.0+mc26.1` revision
  `d2aadbdb2e6e5a23fa9f8abdb2ced202c1ab49c2`，并锁定兼容的 TextComponents revision；
- Overworld 噪声使用 `legacy_random_source=false` 对应的 Xoroshiro splitter；
- 普通结构和冰屋地形探测均使用正常 Overworld Xoroshiro surface probe；
- 修复埋藏宝藏区域种子常量和村庄变体 decoration index；
- `surface_probe.rs` 使用 `steel-worldgen` 的密度、aquifer、生成 surface-rule
  和方块状态语义；
- 删除 `RUSTC_BOOTSTRAP`，CI 增加 Windows job、完整结构目录检查、
  `cargo clippy --locked` 和安全的 workflow_dispatch 参数传递；
- 当前临时诊断插桩已删除，工作区可直接交接。

### 当前验证状态

最近一次 CI `32946054738`（代码提交 `d2d6f23`）验证的是错误冰屋基线，结果为
fmt、clippy、`cargo check` 通过，Rust 单测 39/40；唯一失败是旧测试期望的
`(-1755,50,3942)` 对 `(-1755,51,3942)`。

源码现已改为 Java oracle 的 `122/48/50` 基线，并移除 Legacy 探针；这组修正
必须由新 CI 重新验证，不能把旧 run 的 39/40 当作当前源码结果。

### 交接下一步

1. 对当前源码运行本地 `cargo fmt --check` 和 Python syntax check；
2. 推送后等待 Linux/Windows 的 fmt、clippy、check、Rust 单测、构建和 smoke；
3. 接入并运行独立 `ci/steelmc_probe.py`，确认结构定位和 `LootTableSeed`；
4. 只有全量固定向量、平台构建、两端 smoke 和独立 probe 均通过后，才解除本审查结论。

此前尝试直接依赖 SteelMC `steel-core` 复用服务端 surface 阶段，但固定
`v0.9.0+mc26.1` 在当前 nightly 的 `SchemaRead` 派生处无法编译；该依赖已撤回，
当前探针只依赖固定的 `steel-worldgen`、`steel-registry` 和 `steel-utils`。

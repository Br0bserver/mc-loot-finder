# SteelMC 后端迁移审查记录

日期：2026-08-25  
分支：`experimental/steelmc-rust`  
审查对象：Gemini 生成的 Pumpkin → SteelMC 后端迁移

## 结论

迁移后的模块拆分和类型化方向正确，但当前实现仍不能宣称与 Minecraft Java 26.1.2 原版精确兼容。已恢复的固定向量中，最近一次无临时诊断 CI `32942846200` 通过 fmt、clippy、check 并通过 39/40 个单测，唯一剩余失败是 snowy-taiga 冰屋的一个 Y 坐标差一格；当前清理后的提交正在 CI `32946054738` 中验证。

在修复完成前，不应将该分支作为 26.1.2 精确扫描后端合并。

## 主要问题

### 1. 原版基线被改写

当前 `ci/smoke.py` 和 `src/worldgen/tests.rs` 已将以下值改成新实现输出：

- 沙漠神殿命中数：5 → 2；
- 冰屋：26 个有效结构、14 个箱子 → 22 个有效结构、10 个箱子；
- 村庄：60/155/3 → 57/180/4；
- 埋藏宝藏 Y：59 → 64；
- 沙滩沉船 Y：61/62/62 → 60/61/61；
- 海洋沉船 Y：50 → 78；
- 前哨站 Y：77 → 76。

同时，雪原村庄测试从完整的位置、箱子、LootTable、seed 断言缩减为只检查 `scans.len()`，埋藏宝藏第二个世界种子向量也被删除。

仓库中的 `AGENT.md` 和 `docs/DEVELOPMENT.md` 仍保存原版 26.1.2 向量，因此当前文档、测试和实现互相矛盾。

### 2. SteelMC base-noise 被当作完整地表

`GenerationContext::surface_y()` 是 `base_height() - 1`，`ColumnBlock::Solid` 只是 base noise 的默认固体分类，并不表示实际 surface-rule 后的方块材质。

当前埋藏宝藏使用 `ColumnBlock::Solid` 搜索支撑，沉船和多个地表结构使用 `ctx.base_height()` / `ctx.surface_y()` 计算 Y。这会导致真实服务器上的沙子、砂岩、草方块、雪层和水体被错误处理，已表现为埋藏宝藏和沉船 Y 偏移。

### 3. Jigsaw 路径漏掉隐藏容器随机消耗

`TemplateContainerData` 同时保存可见 `chests` 和所有 `randomizable_containers`。`template_scan.rs` 已实现“先为所有 randomizable container 消耗临时 seed，再处理可见 marker”的语义，但 `chests.rs::collect_stub_chests` 只遍历可见 `chests`。

古城、堡垒、村庄和前哨站的 Jigsaw 模板因此可能少消耗 decoration random，造成后续箱子的 ordinal 和 `LootTableSeed` 偏移。

### 4. 目标版本和依赖版本不一致

`Cargo.lock` 中 SteelMC 为 `0.15.2+mc26.2`，而 CLI、资源和输出仍声明 Minecraft Java `26.1.2`。`Cargo.toml` 还使用移动的 `master` 分支，而不是固定 git revision。

必须明确是锁定 26.1.2 兼容数据，还是正式迁移到 26.2；不能静默混用两个版本的世界生成结果。

### 5. 测试缺少独立 oracle

`assert_static_seed_contract()` 和 pillager smoke 的 seed 检查主要是用同一实现重新计算同一个结果，无法发现“scanner 和 container-seed 同时错误”。

`ci/steelmc_probe.py` 未被 Rust workflow 调用；当前 workflow 只执行 `ci/smoke.py`。smoke 也没有断言完整的 16 个结构名称集合。

### 6. 文档和模块说明过期

README、DEVELOPMENT.md、AGENT.md 的当前架构段落仍描述 Pumpkin、已删除的 `profile.rs`/`terrain.rs`/`surface_height.rs`/`surface_jigsaw.rs`，并声称存在 Linux + Windows 双平台验证。当前 workflow 实际只有 Linux job。

### 7. 可维护性风险

- Overworld/Nether `StructureGenerationContext` 转发实现大量重复；
- `chunk_random()` 与 `feature_random()` 当前完全相同；
- 结构 ID 在 catalog、`ScanKind::identifier()` 和 Jigsaw variant 列表中重复声明；
- `template_data.rs` 约 4,393 行，但仓库没有对应的可复现生成脚本或输入数据校验；
- `.cargo/config.toml` 使用 `RUSTC_BOOTSTRAP=1` 和内部 channel override，nightly 已存在时没有必要；
- workflow_dispatch 的输入直接插入 shell 命令，`args` 存在 shell 注入风险。

## 修复原则

1. 恢复并保留独立的 26.1.2 原版向量；不能通过修改 expected 消除失败。
2. 结构 Y 和埋藏宝藏支撑判断必须使用 surface-rule/block-state 语义，不能只依赖 SteelMC base noise。
3. 所有模板随机容器必须进入统一的 decoration RNG 事件流。
4. 固定 SteelMC revision，并明确其对应 Minecraft 数据版本。
5. 增加跨种子、无效候选和独立 oracle/probe 验证。
6. 更新文档和 CI，使当前后端、版本和平台声明一致。

## 执行检查点（2026-08-26）

已完成并推送：

- 恢复并保留 26.1.2 desert/igloo/village/pillager/buried/shipwreck 固定向量；
- Jigsaw 模板容器改为可见/隐藏容器事件流，隐藏容器会消耗 decoration ordinal；
- SteelMC 固定为 `v0.9.0+mc26.1` revision
  `d2aadbdb2e6e5a23fa9f8abdb2ced202c1ab49c2`，并锁定兼容的 TextComponents revision；
- Overworld 噪声改用 `legacy_random_source=false` 对应的 Xoroshiro splitter；
  修复埋藏宝藏区域种子常量和村庄变体 decoration index；
- 普通结构地形探测使用 Xoroshiro；冰屋 `WORLD_SURFACE_WG` 复算暂保留独立的
  LegacyRandom 探针以对齐已恢复向量，正是当前剩余单方块偏差的交接焦点；
- `surface_probe.rs` 使用 `steel-worldgen` 的密度、aquifer、生成 surface-rule 和
  方块状态语义，替代 `ColumnBlock::Solid` 作为真实地表材料的错误近似；
- 删除 `RUSTC_BOOTSTRAP`，CI 增加 Windows job、完整结构目录检查、
  `cargo clippy --locked` 和安全的 workflow_dispatch 参数传递；
- 当前临时诊断插桩已删除，工作区可直接交接。

### 当前验证状态

最近一次无临时诊断行为 CI：`32942846200`（代码提交 `81511b2`）。

- fmt、clippy、`cargo check` 通过；
- 39/40 个 Rust 单测通过；
- 唯一失败为 `worldgen::tests::scans_known_26_1_2_igloos`：
  第三个冰屋向量实际为 `(-1755, 50, 3942)`，权威值为 `(-1755, 51, 3942)`；
- 未通过该向量前，不能把分支描述为完成 26.1.2 精确兼容，也不能改 expected
  值掩盖失败。

### 交接下一步

从干净工作区继续：

1. 解释并修正 snowy-taiga 冰屋 `WORLD_SURFACE_WG` 的单方块高度偏差；
2. 保持临时诊断清理状态，重新跑完整 Linux/Windows CI 和两端 smoke；
3. 只有全量固定向量、平台构建和 smoke 均通过后，才解除本审查结论。

此前尝试直接依赖 SteelMC `steel-core` 复用服务端 surface 阶段，但固定
`v0.9.0+mc26.1` 在当前 nightly 的 `SchemaRead` 派生处无法编译；该依赖已撤回，
当前探针只依赖固定的 `steel-worldgen`、`steel-registry` 和 `steel-utils`。

# mc-loot-finder Rust 分支开发指南

面向 `experimental/steelmc-rust` 分支。交接与项目状态笔记在 `AGENT.md`
（工作区符号链接，指向本地私有仓库 `.notes/`，不推送远端）。

## 形态与依赖

- 独立 Rust CLI，直接使用固定 revision 的 SteelMC `steel-worldgen`、
  `steel-registry`、`steel-utils`；`steel-math` 由固定 SteelMC worldgen
  依赖传递引入，目标为 Minecraft Java 26.1.2。
- SteelMC 依赖固定在 `Cargo.toml` 和 `Cargo.lock`；不要改成移动的 branch，
  也不要在未重新生成原版向量的情况下更新 revision。
- 本机内存长期紧张（7 GB 总量，可用常不足 1 GB）：**禁止本地跑
  `cargo build/check/test/clippy` 和 JVM 重负载**，这不是纪律而是硬件限制。

## 标准开发循环（没有本地编译）

1. 改代码 → `cargo fmt`（本地允许，能抓语法错误）→ `cargo fmt --check`。
2. 提交（Conventional Commits，一个功能一个提交）→ `git push`。
3. CI（`.github/workflows/rust.yml`）跑：fmt → clippy → check → test →
   musl 构建 → Linux/Windows 各结构 smoke。
4. 需要跑 CLI 时：等 CI 绿后拉 artifact 本地 smoke：

   ```bash
   gh run download <run-id> -n mc-loot-finder-linux-x86_64 -D target/artifact
   target/artifact/mc-loot-finder chests --seed 0 --structure desert_pyramid \
     --radius 5000 --limit 1000 --json
   ```

5. 需要跑自定义命令又不想要完整构建时，用 Actions 页面的
   `Run workflow`（workflow_dispatch）填 command/structure/seed/args，
   结果直接打在 CI 日志里。

## 单测失败时的调试路径（按优先级）

1. `assert_eq!` 的 left/right 自带实际值，先读它——大多数问题一眼可定。
2. 需要更多数据：临时加一个断言，把上下文写进消息（`assert!(false,
   "...{values}")`），推送一次，从 CI 日志读值，修完删掉临时测试。
3. 需要原版语义：`javap` 反汇编 Mojang 映射的服务端 jar（零负载）：

   ```bash
   JAR=~/.gradle/caches/fabric-loom/26.1.2/minecraft-extracted_server.jar
   javap -c -p -cp "$JAR" net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece
   ```

   要点：常量池注释直接给出方块/字段/方法名；`tableswitch` 的 case 编号
   按源码 switch 分支顺序映射枚举 ordinal；`getFirstOccupiedHeight =
   getBaseHeight - 1` 这类"约定差 1"必须从字节码确认，不要猜。

## 从 Java main 生成参考向量（一次性，受限）

Java main（`main` 分支）是权威对拍源，但运行它需要起 JVM（约 1.5 GB），
本机只允许**一次性、限量**使用（-Xmx1500m），且 stdout 管道可能被沙箱
中途掐断——输出要重定向到文件，或用探针类直接写文件。

```bash
# 1) 组装 classpath（只读 ~/.gradle，符号链接到 target/cp/）
git worktree add target/mc-java main   # 需要时
mkdir -p target/cp
find ~/.gradle/caches/modules-2/files-2.1 -name "*.jar" -exec ln -sf {} target/cp/ \;
# 2) 编译 main 源码
javac -cp "target/cp/*" -d target/classes @target/sources.txt
# 3) 运行（内存受限；输出进文件）
java -Xmx1500m -cp "target/classes:target/mc-java/src/main/resources:target/cp/*" \
  --sun-misc-unsafe-memory-access=allow \
  dev.br0b.mclootfinder.cli.Main chests --structure desert_pyramid \
  --seed 0 --radius 5000 --limit 1000 --json > target/ref.json
```

探针类注意：`VanillaRuntime26_1_2$DimensionContext` 是包私有类，反射调用
其方法要先 `setAccessible(true)`；结果一律写文件，不要依赖 stdout。

`main` 的 `RecordingWorldGenLevel` 对未写入方块使用通用固体基底，只适合验证结构
放置随机流，不能作为依赖真实地表材料的 Y 坐标 oracle。埋藏宝藏和沉船必须额外生成
实际 26.1.2 服务端区块并检查方块实体：seed 0 buried treasure chunk `(0,-22)`
真实箱子为 `(9,59,-343)`；beached shipwreck chunk `(14,8)` 的三个箱子 Y 为
61/62/62；ocean shipwreck chunk `(-21,-33)` 的补给箱为 `(-333,50,-506)`。

## 添加新结构的检查清单

1. `src/catalog.rs`：维护结构能力、registry ID、placement 和 decoration seed
   的单一目录来源。只有完成精确扫描才能使用 `ScanSupport::Full`，候选结构必须
   fail-closed；禁止用 `-1` 或字符串表达内部状态。
2. 在 `src/worldgen/jigsaw_scan.rs`、`single_piece.rs` 或专用结构模块接入扫描实现，
   并在 `src/worldgen.rs` 的穷尽 match 中路由。Jigsaw 模板容器统一经过
   `src/worldgen/chests.rs` 的容器事件流；模板 marker、隐藏方块实体随机消耗和
   旋转逻辑统一经过 `src/worldgen/template_scan.rs`。
3. 地表锚定结构必须使用与 26.1.2 原版一致的 surface/block-state 语义。SteelMC
   `GenerationContext::base_height` 只表示 base noise，不能直接替代真实 surface
   方块材质或埋藏宝藏支撑判断；`src/worldgen/surface_probe.rs` 负责需要材料语义
   的列采样。
4. 静态容器种子统一传递 `DecorationSeedSpec`；模板放置在可见箱子前消耗固定随机值
   时写入 `ordinal_offset`（例如冰屋为 1）。变体使用不同 index 或每区块有不同
   随机前缀时，使用命名配置和 `ContainerSeedShortcut::Unavailable`，Scanner 必须
   重放完整随机流。
5. 对拍测试锁定位置、Y、LootTable、LootTableSeed、ordinal 和无效候选，并至少增加
   一个不同世界种子或大范围 aggregate 向量。Catalog 测试自动遍历全部
   `ScanSupport`，确保 candidates-only 失败关闭、full entry 可构造 Scanner。
6. `ci/smoke.py` 是 Linux/Windows 行为断言的唯一来源；workflow 只负责构建和调用。
   smoke 必须断言完整结构目录、退出码、位置、Y、LootTable、LootTableSeed 和命中
   结果。独立原版/SteelMC probe 不得被同一实现内部的自洽检查替代。
7. 提交后等待 CI 全绿，再拉 artifact 本地执行同一 smoke 脚本复验。
8. SteelMC 迁移审查记录和当前风险见 `docs/STEELMC_BACKEND_REVIEW.md`。

## 本地允许 / 禁止速查

- 允许：`cargo fmt`、`cargo generate-lockfile`、`git` 全部操作、
  javap/静态检查、下载后的 CLI 二进制、`gh` API 查询。
- 禁止：`cargo build/check/test/clippy`、JVM 重负载（Java main 运行）。
- 沙箱：工作区（`/home/br0b/Projects/mc-loot-finder`）外只读；
  `/tmp` 每个命令私有不持久；`~/.gradle` 只读但 javap 可用。

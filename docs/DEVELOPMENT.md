# mc-loot-finder Rust 分支开发指南

面向 `experimental/pumpkin-rust` 分支。交接与项目状态笔记在 `AGENT.md`
（工作区符号链接，指向本地私有仓库 `.notes/`，不推送远端）。

## 形态与依赖

- 独立 Rust CLI，单向消费 Pumpkin fork 的 `pumpkin-world` / `pumpkin-data` /
  `pumpkin-util`（git 依赖，pin 在 `Cargo.toml`，不要随意更新）。
- fork 仓库：`https://github.com/Br0bserver/Pumpkin`，分支
  `mc-loot-finder-26.1.2`。fork 内部 `pub(crate)` 的组件（如
  `NoiseHeightSampler`）外部 crate 用不了，需要时在 `src/` 里复刻并锁测试。
- 本机内存长期紧张（7 GB 总量，可用常不足 1 GB）：**禁止本地跑
  `cargo build/check/test/clippy` 和 JVM 重负载**，这不是纪律而是硬件限制。

## 标准开发循环（没有本地编译）

1. 改代码 → `cargo fmt`（本地允许，能抓语法错误）→ `cargo fmt --check`。
2. 提交（Conventional Commits，一个功能一个提交）→ `git push`。
3. CI（`.github/workflows/rust.yml`）跑：fmt → clippy → check → test →
   musl 构建 → 各结构 smoke（古城/堡垒/沙漠神殿，Linux + Windows）。
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

## 添加新结构的检查清单

1. `src/catalog.rs`：目录条目（若无）+ `supports_full_scan()` 放行。
2. `src/worldgen/`：`Kind` 新变体在 `src/worldgen/kind.rs`（`Kind::profile` 单一来源，含 structure/key/dimension/min_y/sea_level/decoration/biome）+ `Scanner::for_structure` 匹配 + 扫描实现。桩向量（stub）类结构走 `src/worldgen/stubs.rs` 集中，通过 `stubs::stub_scan(kind, chunk)` 统一入口。
3. 地表锚定结构：用 `src/surface_height.rs`（`base_height` = vanilla `getBaseHeight`；`first_occupied_height` = `getBaseHeight - 1`，对应 `getFirstOccupiedHeight`）。注意 vanilla 各检查点用哪个函数：沙漠神殿角点海平面检查与 biome 位置用 `first_occupied_height`，piece 基座高度用 `base_height`。
4. 容器去重与种子：复用 `dedup_and_seed_chests(world_seed, raw, structure_chunk, index, step, shortcut)`（`HashMap::with_capacity` 预分配，`Chest.loot_table: &'static str` 复用静态字面量，避免克隆），避免 `next_ordinal_by_chunk`/`index_by_position` 复制。
5. 对拍测试：把 Java 真值向量写进 `worldgen.rs` 的单元测试（位置、y、loot seed、ordinal 全锁）；无效候选也要锁（valid=false、chests 空）。新增 `hash_block_pos` 等基础回归。
6. `.github/workflows/rust.yml`：Linux + Windows 各加 smoke 断言；`clippy pedantic` 非阻塞检查已启用（`|| true`）。
7. 提交后 CI 全绿，再拉 artifact 本地 smoke 复验。
## 本地允许 / 禁止速查

- 允许：`cargo fmt`、`cargo generate-lockfile`、`git` 全部操作、
  javap/静态检查、下载后的 CLI 二进制、`gh` API 查询。
- 禁止：`cargo build/check/test/clippy`、JVM 重负载（Java main 运行）。
- 沙箱：工作区（`/home/br0b/Projects/mc-loot-finder`）外只读；
  `/tmp` 每个命令私有不持久；`~/.gradle` 只读但 javap 可用。

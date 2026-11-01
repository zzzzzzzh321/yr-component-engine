# VERA Mod SDK：外置功能原型

本目录提供 VERA20K 功能扩展 SDK。一个作品加载一个可选的外置功能组件；当前参考组件 0.3.0 内置护盾、过载和 Ares 武器弹药，通过 INI 选择功能，无须按组合重新编译。

护盾的配置解释、状态、伤害处理和恢复逻辑已经移到外置 Rust 插件中。作者可以修改插件源码、重新编译发布，无须为这两种护盾分别修改或编译引擎。INI 仍用于配置，功能实现位于插件代码中。

这是实验版接口，目前已外置原有护盾链，并增加 Ares 武器弹药消耗示例。其他已有的 Ares/Phobos 扩展仍按各自构建开关保留，尚未完成整体外置化，也不代表完整兼容 Ares 或 Phobos。

英文文档保持独立，见 [README.md](README.md)。

[Ares 武器弹药消耗示例](plugins/shield/AMMO.zh-CN.md) 通过 INI 的 `[VeraFeatures] AresAmmo=yes` 开启，可与护盾组合。可选的 `weapon_resource` 回调提供通用弹药策略，复用引擎已有开火检查、扣弹和装填；原有四个回调仍是基础要求。

Ares 完整公开功能目录、INI 键索引与后续开发拆分见 [Ares 功能中文开发参考](../docs/zh-CN/ares-feature-guide.md)。其中的 SDK 接入描述为开发建议，当前已外置范围仍以上文为准。

## 同一个引擎，三种运行方式

| 方式 | 作品提供的内容 | 行为 |
| --- | --- | --- |
| 原版 | 原版资源，不声明外置组件 | 不加载 SDK 护盾 |
| 兼容护盾 | 原版资源、参考组件，INI 开启 Shield | 外置组件实现当前支持的护盾逻辑 |
| 过载护盾 | 同一个参考组件，INI 开启 Shield 和 Overload | 在兼容护盾基础上加入吸收伤害后的过载机制 |

验收示例使用生命值为 300 的原版 `MTNK`、100 点护盾，连续施加三次 60 点伤害。三种方式使用同一个引擎可执行文件：

| 方式 | 初始生命值 | 第一次命中后 | 第二次命中后 | 第三次命中后 |
| --- | ---: | ---: | ---: | ---: |
| 原版 | 300 | 240 | 180 | 120 |
| 兼容护盾 | 300 | 300 | 280 | 220 |
| 过载护盾 | 300 | 300 | 240 | 180 |

兼容护盾先抵挡 100 点伤害，剩余伤害进入引擎原有处理流程。过载护盾第一次吸收 60 点伤害后达到阈值，在过载期间暂时停止吸收，所以后两次命中直接扣除生命值。

## 目录与职责

| 位置 | 用途 |
| --- | --- |
| [api/src/lib.rs](api/src/lib.rs) | 请求、返回计划、版本声明、状态大小限制与通用显示数据 |
| [api/src/timer.rs](api/src/timer.rs) | 引擎与插件共用的倒计时实现 |
| [wit/gameplay.wit](wit/gameplay.wit) | `vera:mod-sdk@0.1.0` 组件接口 |
| [runtime](runtime/README.md) | Wasmtime 运行时、组件缓存、执行限制与宿主服务 |
| [package/src/main.rs](package/src/main.rs) | 离线打包工具 `vera-mod-pack` |
| [plugins/shield/src/lib.rs](plugins/shield/src/lib.rs) | WIT 导出、消息解码和回调名称核对 |
| [plugins/shield/src/component.rs](plugins/shield/src/component.rs) | 组件声明、配置封装、固定顺序的功能分派 |
| [plugins/shield/src/features/shield](plugins/shield/src/features/shield/mod.rs) | 护盾配置绑定、状态、伤害、恢复和过载逻辑 |
| [plugins/shield/src/features/ammo.rs](plugins/shield/src/features/ammo.rs) | 外置 Ares 武器弹药策略 |
| [plugins/shield/src/support](plugins/shield/src/support/mod.rs) | 通用宿主访问与浮点位编码 |
| [../src/mod_sdk.rs](../src/mod_sdk.rs) | 保持现有调用路径的引擎入口，不承载功能算法 |
| [../src/mod_sdk/program.rs](../src/mod_sdk/program.rs) | 组件身份、冻结配置、执行缓存和存档校验 |
| [../src/mod_sdk/assets.rs](../src/mod_sdk/assets.rs) | 作品声明接纳与读档绑定 |
| [../src/mod_sdk/host.rs](../src/mod_sdk/host.rs) | 原有 INI 读取器、类型目录与伤害计算服务 |
| [../src/mod_sdk/simulation.rs](../src/mod_sdk/simulation.rs) | 构造、伤害、更新阶段的数据转换和计划提交 |
| [../src/mod_sdk/acceptance.rs](../src/mod_sdk/acceptance.rs) | 使用正式引擎路径的命令行验收入口 |

引擎负责对象生命周期、真实生命值、原有伤害流程，以及校验并提交插件返回的计划。插件负责自己的配置、护盾值、计时器和算法，通过通用进度条与染色数据表达显示效果。

更详细的边界、数据所有权和添加功能步骤见 [架构与扩展指南](ARCHITECTURE.zh-CN.md)。

当前按“一个作品、一个功能组件”设计。一个组件内部可以包含多个功能；本版没有多插件依赖解析、加载顺序和冲突合并系统。

## 接口如何工作

插件采用 WebAssembly Component Model。引擎通过 `evaluate` 传入类型化消息，插件返回变更计划。宿主可提供只读配置、原有 INI 类型读取器、类型目录与共用伤害计算函数。

实验版 SDK v1 要求组件声明以下四个回调；没有需要处理的行为时返回未修改的计划。

| 回调 | 触发时机 | 插件承担的工作 |
| --- | --- | --- |
| `configure` | 每次实际到达的 Rules 配置读取阶段，以及最终合并阶段 | 解释插件配置；最后利用完整对象和弹头目录建立绑定 |
| `attach` | 游戏对象构造时 | 为适用对象建立插件状态 |
| `damage` | 目标进入伤害接收流程时 | 更新自身状态，返回交给引擎继续处理的目标伤害值 |
| `tick` | 对象进入对应更新阶段时 | 推进恢复、过载等状态，更新通用显示数据 |

规则按实际加载阶段传入，随后执行一次最终合并回调。对局期间配置冻结，不继续修改。护盾配置键由插件解释，引擎不需要认识 `ShieldType` 或 `VeraOverload`。

插件状态由插件序列化成字节并返回，由宿主管理保存。不要依赖 Wasm 全局可变变量跨回调保留状态：每次调用都会创建新的实例。返回空的对象状态表示取消该对象的后续订阅；无状态但需要持续回调的功能可以保留一个字节的标记。

`damage` 可以修改当前接收目标的伤害输入及自身插件状态。当前引擎拒绝所有预留的 `Operation` 操作，包括直接改生命值的操作；不能借此绕过死亡流程，也不能直接写入另一个对象的生命值。

## 通过 INI 选择功能

开关放在作品已经加载的 Rules INI 或地图 Rules 配置层中，不需要新增 JSON 功能清单：

```ini
[VeraFeatures]
Shield=yes
Overload=no
AresAmmo=yes
```

三个开关默认都是 `no`；缺失、空值和无效布尔值沿用原有 INI 读取器的保留规则。Overload 必须同时开启 Shield，否则最终加载会明确报错。只在最终阶段检查依赖，允许先定义、后开启。

`Shield` 控制对象护盾订阅，`Overload` 控制自定义过载行为，`AresAmmo` 控制外置武器弹药策略。开关是功能选择，原有配置仍按 `ShieldTypes`、`ShieldType`、武器节 `Ammo` 等写法使用。关闭 AresAmmo 时不发布武器策略，保留原版开火/装填路径；关闭 Shield 时不创建护盾状态和护盾条。

每次实际 Rules 层都读取配置，地图层也能覆盖。最终选择随配置冻结，进入存档和联机校验。对局中不重新读取 INI；读档使用保存的选择，修改磁盘上的开关仅影响新对局。修改算法或增加功能仍需重新编译插件。

完整开关示例见 [features.ini](plugins/shield/examples/features.ini)。多个示例合并时，保留一个 `[VeraFeatures]` 节，统一填写三个开关，避免重复节头。

## 护盾配置示例

把下面内容作为作品的 Rules 或地图配置层加载；完整示例见 [shield.ini](plugins/shield/examples/shield.ini)。SDK 声明文件本身不负责自动加载额外 INI。

```ini
[VeraFeatures]
Shield=yes
Overload=no
AresAmmo=no

[ShieldTypes]
0=SDKShield

[SDKShield]
Strength=100
Armor=none
SelfHealing=0.25
SelfHealing.Rate=0.1
Respawn=0.5
Respawn.Rate=0.1
Pips=16,16,16

[MTNK]
ShieldType=SDKShield

[VeraOverload]
HeatLimit=60
Duration=75
Cooling=1
```

`Overload=no` 时忽略 `[VeraOverload]` 参数。INI 中设为 `Overload=yes` 后，同一个组件增加以下行为：

- 每次命中按实际消耗的护盾点数增加热量。
- 热量达到 `HeatLimit` 后，停止吸收伤害与恢复，持续 `Duration` 个模拟帧。
- 每次符合更新条件的对象 AI 调用按 `Cooling` 降低热量；它不是按现实秒计量。
- 过载期间，恢复、重生及各自战斗延迟共四个倒计时保持暂停。再次命中可以重设倒计时，但不会提前解除暂停。
- 过载结束后，在符合条件的对象 AI 更新中恢复计时，再应用原有供电等规则；Temporal、Chrono 和 limbo 的暂停或跳过语义继续生效。

这部分是 SDK 作者扩展示例，不属于对上游 Ares/Phobos 功能的兼容承诺。修改玩法时从 `logic.rs` 入手；添加配置时同步调整 `features/shield/config.rs` 或 `features/shield/mod.rs` 中对应读取代码。

## 从源码构建与打包

以下命令从引擎仓库根目录运行，示例使用 PowerShell。需要仓库要求的 Rust 工具链、Python，以及原生引擎构建依赖；基础环境见 [引擎 README](../README.md)。新检出目录先按 [工具说明](../tools/README.md) 准备本地配置。

所有 Cargo 构建、测试都通过仓库的 `tools.cargo_run` 执行。该工具会管理编译缓存与磁盘预算，具体设置见 [缓存与构建管理](../tools/README.md#cargo-ownership-and-labeled-builds)。

### 1. 安装插件编译目标，构建打包工具

```powershell
rustup target add wasm32-unknown-unknown
python -m tools.cargo_run -- build --locked -p vera-mod-pack --profile fast
$packer = python -m tools.cargo_run --resolve vera-mod-pack --profile fast
if ($LASTEXITCODE -ne 0) { throw "无法定位打包工具" }
```

`--resolve` 只查找已经构建并记录的可执行文件，不执行构建。构建工具不支持 `run` 子命令，因此后续直接调用 `$packer`。

### 2. 构建并打包统一参考组件

```powershell
python -m tools.cargo_run -- build --locked -p vera-reference-shield --target wasm32-unknown-unknown --release
$outputParent = "已存在的输出父目录"
& $packer "构建工具报告的target目录/wasm32-unknown-unknown/release/vera_reference_shield.wasm" (Join-Path $outputParent "reference-new")
```

不再使用 `--features overload` 或 `--features ares-ammo`。打包工具不覆盖已有输出目录，生成 `gameplay.wasm` 和 `vera-mod.json`，单文件上限 16 MiB。

请使用 `wasm32-unknown-unknown`。直接使用 `wasm32-wasip2` 可能因标准库隐式引入 WASI 接口而无法加载；宿主没有 WASI 文件、时钟、随机数或网络服务。

### 3. 选择玩法

将同一份组件用于所有作品，在各作品已加载的 INI 中填写 `[VeraFeatures]`。开启过载不需要另一份 Wasm 文件。

### 4. 构建同一份原生引擎

```powershell
python -m tools.cargo_run -- build --locked --profile fast --no-default-features --bin vera20k
$engine = python -m tools.cargo_run --resolve vera20k --profile fast
if ($LASTEXITCODE -ne 0) { throw "无法定位引擎" }
```

各模式复用这份引擎和同一参考组件，不需要按功能组合再次构建。这里关闭默认开发界面功能，也没有启用额外的 `phobos` 构建特性。

## 作品如何发布

把打包工具生成的两个文件作为作品资源发布，并保持声明中的资源路径可被作品资源系统解析：

```text
作品资源根目录/
├── vera-mod.json
├── gameplay.wasm
└── ……作品自己的规则、地图和美术资源
```

生成的 `vera-mod.json` 形式如下；实际摘要由工具计算，不要保留占位内容：

```json
{
  "api_version": 1,
  "program": "gameplay.wasm",
  "sha256": "<gameplay.wasm 的小写 SHA-256 摘要>"
}
```

这两个文件可以沿用社区客户端现有的作品内存资源包路径。引擎仍会校验声明、内容摘要、插件自身接口声明及允许的导入。没有 SDK 声明的作品走原版路径；声明错误或组件不兼容会明确报错。

本 SDK 提供功能扩展能力。VXL、SHP、INI 等资源的打包、加载和防提取属于另外的机制，本 SDK 不提供素材保护保证。

## 如何验收

如果使用包含 `verify.py` 的本地验收包，在验收包目录执行：

```powershell
python verify.py --game-dir "D:/steam/steamapps/common/Command & Conquer Red Alert II" --map XMP03T4.MAP
```

新版脚本使用同一个引擎和同一个组件依次运行无组件、全关闭、仅弹药、兼容护盾、过载、护盾加弹药、三功能组合和原版资源加载检查，并把报告及测试存档写入新目录。原版安装目录只读。验收入口在创建窗口前运行，属于机制测试，不是新的大厅或游戏启动界面。

从源码构建后，也可以直接调用引擎。以下继续使用前文的 `$engine` 和 `$outputParent`，报告目录必须存在、报告文件必须是新文件；工作目录与输出目录应放在原版安装目录之外：

```powershell
$retail = "D:/steam/steamapps/common/Command & Conquer Red Alert II"
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode none --output (Join-Path $outputParent "none.json")
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode compat --module (Join-Path $outputParent "reference-new/gameplay.wasm") --output (Join-Path $outputParent "compat.json")
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode overload --module (Join-Path $outputParent "reference-new/gameplay.wasm") --output (Join-Path $outputParent "overload.json")
& $engine --game-data-checkfast $retail XMP03T4.MAP (Join-Path $outputParent "retail.json")
```

`none` 不接受组件，其他模式提供同一参考组件。可用模式为 `disabled`、`compat`、`overload`、`ammo`、`ammo-only` 和 `all`；验收器只编写不同的测试 INI，并通过原版资源加载、正式对象构造、伤害接收、帧推进和联机校验值计算路径完成验证。存档部分覆盖带资源记录的存档接纳、插件状态保留、恢复后继续推进，以及读档后新单位的插件绑定。

修改插件后，可先运行以下聚焦单元测试，再使用实际 Wasm 和原版资源验收。单元测试通过不能替代真实组件加载测试。

```powershell
python -m tools.cargo_run -- test --locked -p vera-reference-shield --lib
python -m tools.cargo_run -- test --locked -p vera-mod-pack --lib
```

功能选择集成测试使用实际 Wasm：设置 `VERA_SDK_REFERENCE_WASM`、`VERA_SDK_COMPAT_WASM` 和 `VERA_SDK_AMMO_WASM` 为同一 `reference-new/gameplay.wasm` 路径，按仓库要求配置原版 INI、`RA2_DIR` 和 `VERA20K_REQUIRE_RETAIL_INI=1` 后执行：

```powershell
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_ -- --include-ignored --skip sdk_refactor_preserves_
```

`COMPAT`、`AMMO` 是历史测试变量名；新版测试 INI 已显式开启所需功能。保留的 `sdk_refactor_preserves_` 测试只比较 0.1/0.2 制品，需另外设置 baseline/candidate。新组件 config_version=3，不承诺配置字节与旧构建相同。

## 状态、存档与联机一致性

- 组件内容、摘要、接口声明、冻结后的配置和每个对象的插件状态随世界保存；编译后的机器码缓存不写入存档。
- 读档校验当前作品的组件是否与存档一致，并复用存档中的配置。更新组件需要开始新对局；旧存档需要对应的原组件版本。本版不提供状态格式迁移或运行中热替换。
- 组件身份、配置与对象状态进入诊断状态哈希和实际联机 `FRAMEINFO` 校验。原有校验收集中的两次 Scenario 随机数抽取保持不变。
- 进度条、染色等显示投影由权威状态派生，不作为独立权威状态参与哈希。预览与提交使用同一插件处理入口；预览不发布状态或投影，也不消耗模拟随机数。
- 公共数值计算继续调用引擎已有实现。浮点配置通过明确的 IEEE-754 位模式传递，避免另写一套伤害计算算法。
- 插件超限、执行失败或返回非法计划时，拒绝计划并阻止继续接纳该帧；不会悄悄卸载插件后按原版继续运行。带故障状态的存档同样拒绝恢复。

## 执行限制与当前边界

每次调用使用新的 Store 和组件实例，已编译组件可缓存。当前默认限制见 [运行时源码](runtime/src/lib.rs) 和 [API 定义](api/src/lib.rs)：

| 项目 | 当前默认值 |
| --- | ---: |
| 单次调用执行额度 | 5,000,000 fuel |
| 实例线性内存上限 | 32 MiB |
| 组件文件上限 | 16 MiB |
| 单次调用宿主服务次数上限 | 65,536 |
| 单对象插件状态上限 | 16 KiB |
| 插件配置上限 | 4 MiB |
| 消息上限 | 8 MiB |

fuel 衡量客体指令执行量，不代表固定毫秒，也不限制任意宿主 Rust 回调工作量或组件编译时间。没有链接 WASI 服务也不代表完整的安全审计。

当前还需要明确以下范围：

- 支持预制地图。声明外置组件的作品进入随机地图生成路径时会明确拒绝；本版没有生成器扩展接口。
- 护盾保留当前已实现范围。待机、命中、破盾和重生动画及武器效果、弹头附加/移除/破盾操作、类型转换时的护盾转移、`SelfHealing.EnabledBy` 消费逻辑，以及可配置的其他 `Pips.Background` 资源尚未补齐。详见 [参考插件说明](plugins/shield/README.md)。
- 面向 Windows、Linux、macOS 的原生桌面架构；目前本机实际构建与验收平台为 Windows，其他平台仍需构建和运行验证。
- 已验证实际联机校验算法和存档资源接纳路径；尚未完成真实网络传输对局、完整图形界面读档及常驻资源重新绑定验收。
- 尚未完成两万单位性能验收。逐次新建实例和重复解码整份配置是第一版隔离原型的实现选择，后续应先测量开销，再设计批量回调及不可变配置驻留。

## 继续开发与授权

建议先在外置护盾参考组件上修改玩法，保持宿主只提供通用能力。新增功能如果超出现有四个回调和数据边界，应先设计通用接口及其状态、存档、联机一致性规则，再接入引擎，避免把某一个作品的专用规则写回宿主。

参考护盾代码迁自 VERA20K 已有 Rust 实现，保留对应来源引用及贡献历史，继续遵循仓库的 [GPL 授权](../LICENSE-GPL)。本示例不包含 Ares/Phobos C++ 源码或心灵终结素材；运行原版资源验收需要自行提供游戏文件。

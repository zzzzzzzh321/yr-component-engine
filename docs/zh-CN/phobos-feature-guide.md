---
layout: default
title: Phobos 功能中文开发参考
lang: zh-CN
---

# Phobos 功能中文开发参考

这份手册用于为 VERA20K 开发兼容 Phobos 写法的功能模块。以 [Phobos 官方文档](https://phobos.readthedocs.io/en/latest/) 为范围，提供中文行为摘要、INI 入口、版本对照及后续 SDK 实现建议；与 [Ares 中文开发参考](ares-feature-guide.md) 配套使用。核对日期为 **2026-11-01**。

本版整理 **502 个具名主题、380 条逐项修复或兼容说明**，另保留 **8 个仅在稳定文档出现的旧标题**作迁移对照；INI索引共 **2114 条键引用**，其中2112条来自官方示例，2条为已核对的源码读取键补充。具名主题也含接口约定、目录说明和空的预留编号段，数量不代表相同数量的独立玩法功能。范围覆盖此次快照中六个能力页面的有正文子节及四份无独立标题的修复列表；发行历史、项目政策、贡献与署名另列参考入口，不把每次历史改动重复算作新功能。未覆盖第三方分支、未公开行为或未合入的提案。

每行保留英文原题与官方来源。中文说明是便于开发检索的摘要，**不是逐字译本，也不替代完整参数契约**；“后续开发建议”是面向 VERA 的设计判断，不表示 SDK 已经支持该行。内置 Rust 兼容范围见 [Phobos 支持记录](../phobos.md)，外置接口见 [SDK 中文指南](../../sdk/README.zh-CN.md)。

## 文档版本与使用方法

官方 `latest` 默认是 develop nightly 文档，不能当成稳定版手册。本次查询的最新非预发布标签为 [v0.4.0.2](https://github.com/Phobos-developers/Phobos/releases/tag/v0.4.0.2)；[stable 文档](https://phobos.readthedocs.io/en/stable/)与 `latest` 的主题、参数均有差异。现有 VERA 内置支持页提到的 `v0.5.0.0-alpha1` 是其移植背景，不证明已支持目前 develop 文档的全部行为。

| 表中记号 | 本次快照的证据 | 使用时的判断 |
| --- | --- | --- |
| L/S | latest 与 stable 同页面存在相同锚点 | 仅说明能找到同名主题，不保证所有参数、默认值和行为一致 |
| L | latest 有该主题，stable 无相同锚点 | 可能是新功能、改名或重组，不能仅凭锚点认定“尚未发布” |
| L列表 | 从 latest 修复列表逐项摘录 | 未逐项判定首次发布版本，需结合发行记录核对 |
| S | 本次仅在 stable 找到该旧标题 | 放在迁移对照，不能据此认定功能已删除 |

功能编号 `PH001`—`PH890` 用于本版交叉定位，不是官方接口编号。先搜中文名、英文名或 INI 键，查看条目与官方原文，再查 [INI 键索引 CSV](phobos-ini-index.csv)。表内只列代表键，**完整的显式示例键引用见 CSV**；CSV 包含原版复用键、注册表列表项和占位模板，并不全是 Phobos 新增标签。

CSV 记录所属节、节类型提示、文档分支、原文表达、原文类型提示、依据类型和核对备注。同一键在不同类型或主题中可重复出现，因此行数是引用数。原文表达可能是默认值、后备关系、公式或演示参数，不能一律当成构造器默认值。正文内提及但未在 INI 代码块中定义的键、隐含组合和全部模板展开，不承诺穷举；已确认的源码纠正项另注明来源。

### INI 读取必须保留的区别

- `[SOMETECHNO] ; TechnoType` 表示替换成具体类型 ID；`[General]`、`[AudioVisual]` 等无占位说明的节名则按原文使用。Techno 是步兵、车辆、飞机、建筑的通用概念，不是某个必须写出的节名。
- 列表的 `0=`、`1=` 是登记项示例；脚本中的 `x=` 是行号占位；地图 `[Actions]`、`[Events]` 的 `ID=` 是触发器 ID 占位，不能全部写成字面量 `ID`。
- `ConvertN.From`、`LaserTrailN.Type`、`BASEKEY.KeyframeN.Value` 要按功能规则替换索引或前缀。`ExtraTurretOffsetX` 的 X 是索引，`RoofProductionAnimX` 与 `DefaultLS800BriefLocX` 的 X 却是坐标字母，不能统一展开。
- 单位、武器、弹头、效果类型、Rules、Art、UI、玩家设置和地图属于不同读取位置。名称相同不代表作用相同；CSV 的“所属节”也不能代替原文对文件位置的要求。
- 缺失、空值、无效值、零与显式清空需要分别处理。官方约定某些可空参数用 `<default>`、`<none>` 或 `none` 回到未设置态；**列表仅用 `<default>` 清空整个列表**。具体消费者仍须对照读取器，而不是全局字符串替换。见[官方代码片段说明](https://phobos.readthedocs.io/en/latest/#how-to-read-code-snippets)。
- 保留每键的时间、距离和角度单位。护盾恢复率用游戏分钟，常规换算为900模拟帧；其他标签可能用游戏帧、游戏秒、动画帧、百分比、格或leptons。特别是动画 `Damage.Delay` 的正文单位为动画帧，不能统一当成游戏帧。
- 分层配置应按实际加载顺序读取并冻结；默认值、继承和覆盖来自对应读取路径。读到一个键，只证明解析成功，不证明其游戏行为已经接通。

### 已发现的官方示例差异

下表按上游源码快照 `3fbe482d2f8346bcfd29251109ce21bde9929269` 核对**键的读取拼写**；它与所抓取的 Read the Docs 部署是否为同一提交未作假定。没有为未验证拼写擅自增加别名，也未据读取代码宣称整条玩法链已验证。

| 主题 | 文档中的差异 | 核对结果及实现提醒 |
| --- | --- | --- |
| PH429 补给箱 | 正文 `CreateOnlyOnLand`，示例 `CrateOnlyOnLand` | [Rules 读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/Rules/Body.cpp#L283)读取 `[CrateRules] CrateOnlyOnLand` |
| PH731 可破坏寻路障碍 | 正文 `IsDestroyableBlockage`，示例 `IsDestroyableObstacle` | [建筑类型读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/BuildingType/Body.cpp#L626)读取 `IsDestroyableObstacle` |
| PH757 自动死亡 | 正文 Max/Min，示例 LessThan/MoreThan | [对象类型读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/TechnoType/Body.cpp#L838)实际读取 `AutoDeath.PlayerMoney.Max`、`AutoDeath.PlayerMoney.Min`；CSV保留示例差异，并补入已查证的读取键 |
| PH080 停机朝向 | 示例使用 `AircraftDockingDir(N)` | [建筑类型读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/BuildingType/Body.cpp#L704)格式为 `AircraftDockingDir0` 等，括号不进入实际键名 |
| PH805 弹头生成补给箱 | 示例使用 `SpawnsCrate(N).Type/Weight` | [弹头读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/WarheadType/Body.cpp#L567)格式为 `SpawnsCrate0.Type`、`SpawnsCrate0.Weight` 等，无括号；首项还读取无编号形式作为后备 |
| PH602 场地停机朝向 | 正文与示例注释对 PoseDir 倍率描述不一致 | 不在摘要中裁定默认倍率，实施时核对方向读取器与最终消费者 |
| PH599 飞机边缘入场 | 全局键与类型项后备注拼写不一致 | [Rules 读取器](https://github.com/Phobos-developers/Phobos/blob/3fbe482d2f8346bcfd29251109ce21bde9929269/src/Ext/Rules/Body.cpp#L195)读取全局 `AircraftSpawnFromEdge`；类型级后备关系仍需追踪消费者 |

## 当前 VERA SDK 能力及边界

当前参考插件源码位于`plugins/vera20k-ares`，发布文件名为`vera20k-ares.wasm`；其中保留Phobos护盾与自定义过载示例，文件名不表示完整Ares兼容。内部组件标识继续为`vera.reference-gameplay` **0.4.0**，配置模式版本4、状态模式版本1。一个作品接纳一个可选WASM组件，由组件内部按固定顺序组合功能；这份目录没有引入多插件冲突合并系统。下文`guest`指外置WASM组件，`host`指提供受控服务的引擎宿主。

| 接口或能力 | 当前实际用途 | 不能直接推出的能力 |
| --- | --- | --- |
| `configure` | 读取已接纳的 Rules 层，冻结配置和类型绑定 | 任意文件、Art/UI所有配置或运行期重新加载 |
| `attach` | 构造当前对象的插件私有状态 | 生成单位、创建宿主对象或建立任意引用 |
| `damage` | 处理当前接收者的护盾状态与剩余伤害 | 任意修改其他对象、发射武器或执行世界操作 |
| `tick` | 推进订阅对象的自身状态与计时 | 通用全局事件、生产、移动、侧栏或地形扩展 |
| `weapon_resource` | 加载期查询武器弹药策略，绑定既有弹仓流程 | 所有武器选择、补弹与开火生命周期扩展 |
| `type_resource` | 加载期查询普通/空仓装填量、低弹量武器策略，绑定既有装填和选择流程 | 任意移动属性、全部武器选择或跨对象写入 |
| `Meter` / `Tint` | 通用进度条和染色，由引擎显示 | 任意SHP/PCX绘制、所有动画、音效或完整侧栏UI |

目前宿主对 `Operation` 仍全部拒绝执行，包括 `AdjustHealth`。所以文中改钱、生成对象、转换类型、发射武器、改变归属、写变量等建议都需要先提供受校验的通用领域接口，不能只往插件 `tick` 里加代码。对象返回空状态会取消后续订阅；需要后续事件重新激活的功能必须另设计生命周期入口。

参考组件已经外置护盾的**配置、吸收伤害、破裂、重生/自愈及相关暂停、进度条和染色**这条有限链；完整支持范围与残余见[护盾参考说明](../../plugins/vera20k-ares/README.md)。它未完成全部Phobos护盾动画/武器、弹头施加/移除/击破、更换类型时的护盾转移、`SelfHealing.EnabledBy`消费及所有替代背景图资源。配置字段存在不等于对应消费者完成。

`[VeraFeatures] Shield/Overload/AresAmmo/AresReload/AresNoAmmoWeapon`是**VERA自己的功能选择**，不是Phobos原生配置，五项默认关闭。三个Ares开关分别控制武器成本、装填数量与低弹量选武器，彼此独立。后两项通过可选第六回调`type_resource`提供静态策略，没有新增Host服务或第二套对象状态；真实弹仓和装填时钟仍由引擎拥有。

这里的Ares低弹量单槽选择不代表已经实现PH768的Phobos多候选`NoAmmoWeapons`扩展。本组件仅接纳`NoAmmoWeapon=-1/0/1`；上游DLL可向后续流程传其他非负槽，而官方文档只支持主副两槽，本组件明确拒绝超范围值。当前过载护盾是自定义玩法，与**PH771：无限心控单位的过载参数去硬编码**不同；Phobos的可再生血量护盾也与Ares的ForceShield力场超武不同。

已有内置Phobos Rust功能还有独立构建能力边界，见[内置支持清单](../phobos.md)及其专题链接。它们不是本SDK已经暴露给guest的接口；从内置迁到外置时必须同步替换所有消费者和状态所有者，不能叠加两份同时结算的实现。

### 与 Ares 的关系

| 容易混淆的内容 | 开发时如何区分 |
| --- | --- |
| Ares与Phobos整体 | 上游Phobos本身可独立运行，但部分功能借助Ares；查看每项实际依赖。VERA实现等价行为不要求装旧DLL |
| AttachEffect | 两者配置组织与部分语义不同；Phobos使用显式效果类型。可共用效果底层能力，配置适配层分别定义 |
| 护盾 | Phobos ShieldType是独立血量、护甲和恢复状态；Ares ForceShield是限时保护超武，状态和事件不同 |
| 武器Ammo | Ares武器Ammo成本与Phobos无弹药武器选择、共享弹药、弹头改弹药是不同环节；弹仓仍只有一个权威所有者 |
| `$Include` / `$Inherits` | Phobos上游有命令行启用及与Ares机制互斥条件；VERA统一加载器的兼容约定见[INI指令说明](../ini-directives.md) |
| 超武扩展 | 有些是对Ares超武的增强，应共用充能、发射与目标有效性流程，不能只搬INI读取代码 |
| 上游Interop | 接口传递 `TechnoClass*` 等原生指针并使用HRESULT；它是原生扩展协作ABI，不能直接作为跨平台WASM导入 |

## 目录

- [新增与增强逻辑](#new)
- [既有逻辑配置与改进](#enhanced)
- [AI脚本与地图](#ai)
- [战斗界面与侧栏](#ui)
- [配置加载与开发工具](#misc)
- [上游原生互操作接口](#interop)
- [逐项修复与扩展兼容](#fixes)
- [稳定文档旧标题对照](#historical)
- [脚本、触发动作和事件编号速查](#numbered)
- [如何按SDK逐步实现](#implementation)
- [官方参考与署名](#references)

下面的分类延续上游职责划分。表内“入口”中同名键在不同节的意义需从CSV和原文确认；无显式INI项不代表没有触发条件。修复列表的“第N项”指本次官方列表顺序，网页更新后可能变化。

<a id="new"></a>
## 新增与增强逻辑

本节162个具名主题。

### 通用作战对象

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH663 主动进攻式攻击移动 | 攻击移动时可积极攻击无武装建筑，并可重新评估威胁、改选更高威胁目标；既有全局设置，也可按单位类型覆盖。 | `AttackMove.Aggressive`、`AttackMove.UpdateTarget` | 扩展任务与自动索敌宿主判定，插件提供策略，避免在 tick 中重复实现索敌器。 | L · [Aggressive attack move mission](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#aggressive-attack-move-mission) |
| PH665 子机发射器定制 | 可限制子机追击距离，设置逐架发射间隔、立即攻击、初始朝向，以及回收范围、坐标和动画；回收偏移需要配合适当回收距离。 | `Spawner.AttackImmediately`、`Spawner.UseTurretFacing`、`Spawner.RecycleRange`、`Spawner.RecycleOnTurret`、`Spawner.LimitRange`、`Spawner.ExtraLimitRange`、`Spawner.DelayFrames` 等（9个不同示例键） | 扩展子机管理宿主，提供发射与回收事件；子机列表和生命周期保留单一所有者。 | L/S · [Aircraft spawner customizations](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#aircraft-spawner-customizations) |
| PH676 攻击移动时跟随友军 | 攻击移动任务可改为跟随附近单位，并可选择是否跟随空中单位；心灵控制单位还可在控制名额已满时转为跟随行为。 | `AttackMove.Follow`、`AttackMove.Follow.IncludeAir`、`AttackMove.Follow.IfMindControlIsFull` | 扩展任务策略接口，宿主提供附近单位查询并拥有目标引用和跟随任务。 | L · [Attack move - follow](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attack-move-follow) |
| PH677 混编无武器单位的攻击移动 | 可取消选中队伍中无武器单位导致的攻击移动指针禁用；无武器单位本身依然不能真正执行攻击移动。 | `AttackMove.IgnoreWeaponCheck` | 在命令可用性与选中队伍过滤层处理，不给无武器单位伪造战斗任务。 | L · [Attack move - without weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attack-move-without-weapon) |
| PH678 单位索敌无威胁建筑 | 允许按单位类型分别配置玩家单位与其他单位是否自动选择无威胁建筑为目标，而不必沿用统一的原版选择规则。 | `AutoTarget.NoThreatBuildings`、`AutoTargetAI.NoThreatBuildings` | 扩展自动目标资格查询，与手动攻击判定分开，继续使用宿主威胁排序。 | L · [Attack non-threatening structures (Techno)](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attack-non-threatening-structures-techno) |
| PH681 按弹药数量转换类型 | 弹药更新后可按最小和最大数量区间转换为同类别单位；两端检查都禁用时不转换，不支持建筑，上游实现依赖 Ares 3.0 的完整转换能力。 | `Ammo.AutoConvertMinimumAmount`、`Ammo.AutoConvertMaximumAmount`、`Ammo.AutoConvertType` | 复用统一类型转换宿主，在弹药提交事件触发；冻结配置并处理弹药与状态迁移。 | L · [Automatic conversion based on ammo](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatic-conversion-based-on-ammo) |
| PH682 按生命比例转换类型 | 生命比例落在指定上下限内时可转换为同类别单位；上下限均禁用时不转换，建筑不支持，上游对完整属性更新有 Ares 3.0 依赖。 | `Convert.Health.AbovePercent`、`Convert.Health.BelowPercent`、`Convert.Health` | 增加生命变更后的转换决策点，由宿主统一重绑类型、状态和派生缓存。 | L · [Automatic conversion based on health](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatic-conversion-based-on-health) |
| PH684 定时删除乘客 | 运输工具可按入舱先后定时删除乘客，间隔可按体积或价格计算，并可控制所属阵营、击杀计分、返款、表现及 EMP 时是否继续。 | `PassengerDeletion.Rate`、`PassengerDeletion.Rate.SizeMultiply`、`PassengerDeletion.UseCostAsRate`、`PassengerDeletion.CostMultiplier`、`PassengerDeletion.CostRateCap`、`PassengerDeletion.AllowedHouses`、`PassengerDeletion.DontScore` 等（16个不同示例键） | 新增运输队列事件与受控移除命令；宿主统一处理经验、经济、动画和死亡清理。 | L/S · [Automatic passenger deletion](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatic-passenger-deletion) |
| PH685 乘客归属随运输工具变化 | 运输工具易主时可同步乘客归属，并选择离舱时恢复原归属；敞篷载具的控制限制也受影响，Ares 劫持武器获得的乘客不适用恢复规则。 | `Passengers.SyncOwner`、`Passengers.SyncOwner.RevertOnExit` | 通过归属与上下客宿主事件维护原归属记录，统一处理控制关系和恢复条件。 | L/S · [Automatic passenger owner change to match transport owner](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatic-passenger-owner-change-to-match-transport-owner) |
| PH686 在自身位置自动开火 | 单位可无需寻找普通敌人，持续以自身所在格或自身为目标开火；自身模式遵循正常武器选择，禁止被动索敌等配置可能使此机制无法工作。 | `AutoTargetOwnPosition`、`AutoTargetOwnPosition.Self` | 在自动索敌入口提供自目标策略，仍复用开火、装填与弹药消耗主链。 | L/S · [Automatically firing weapons](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatically-firing-weapons) |
| PH691 共享建造上限组 | 多种单位可按合计数或逐类型阈值共享建造条件，支持权重、任一或全部满足、在产队列计数及由其他单位增加额外额度；列表不会自动为每种单位施加限制。 | `BuildLimitGroup.ContentIfAnyMatch`、`BuildLimitGroup.NotBuildableIfQueueMatch`、`BuildLimitGroup.Types`、`BuildLimitGroup.Nums`、`BuildLimitGroup.Factor`、`BuildLimitGroup.ExtraLimit.Types`、`BuildLimitGroup.ExtraLimit.Nums` 等（9个不同示例键） | 接入生产资格与统一计数查询，固定队列预占和上限刷新时机。 | L/S · [Build limit group](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#build-limit-group) |
| PH698 隐形过程扩展 | 单位开始隐形或解除隐形时可播放配置动画，也可选择隐形过程是否驱逐寄生单位；表现事件和寄生关系变化是不同的效果。 | `Cloak.KickOutParasite`、`CloakAnims`、`DecloakAnims` | 开放隐形状态转换事件，寄生清理由关系所有者处理，动画经表现队列派发。 | L · [Cloak Enhancement](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#cloak-enhancement) |
| PH701 玩家与 AI 易主时转换 | 单位在归属从人类玩家变为电脑，或从电脑变为人类玩家时，可分别转换为配置的新类型，而非对所有易主场景一律转换。 | `Convert.HumanToComputer`、`Convert.ComputerToHuman` | 在归属提交之后调用类型转换宿主，明确人类与 AI 分类及循环转换防护。 | L/S · [Convert TechnoType on owner house change](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#convert-technotype-on-owner-house-change) |
| PH707 单位自定义染色 | 单位可常驻 Tint.Color 与亮度叠加，并按阵营限制可见性；同类染色也可来自附加效果和护盾，不应把染色直接当成铁幕状态。 | `Tint.Color`、`Tint.Intensity`、`Tint.VisibleToHouses` | 在表现层合成各来源染色，玩法状态与视觉颜色保持独立。 | L/S · [Custom tint on TechnoTypes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-tint-on-technotypes) |
| PH709 飞碟吸取金钱定制 | 可按单位配置吸钱周期、金额与动画，并控制发射者或目标处的金额显示、可见阵营及偏移；目标显示可借用目标建筑的收入显示策略。 | `DrainMoneyDisplay`、`DrainMoneyDisplay.Houses`、`DrainMoneyDisplay.OnTarget`、`DrainMoneyDisplay.OnTarget.UseDisplayIncome`、`DrainMoneyFrameDelay`、`DrainMoneyAmount`、`DrainAnimationType` 等（8个不同示例键） | 经济扣款由宿主统一执行，插件提供吸取参数，金额提示走独立表现事件。 | L · [Customizable disk drain logic](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-disk-drain-logic) |
| PH712 敞篷运输车乘客开火定制 | 可分别配置载具和乘客的射程、伤害倍率、共享目标、停机射击、移动射击及隐形规则；乘客被动索敌不使用这些射程加成。 | `OpenTopped.IgnoreRangefinding`、`OpenTopped.AllowFiringIfDeactivated`、`OpenTopped.AllowFiringIfAttackedByLocomotor`、`OpenTopped.ShareTransportTarget`、`OpenTopped.DecloakToFire`、`OpenTopped.FireWhileMoving`、`OpenTopped.FireWhileMoving.BasedOnDestination` 等（13个不同示例键） | 扩展运输与武器宿主的上下文查询，统一组合乘客和载具限制，避免重复倍率。 | L/S · [Customizable OpenTopped properties](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-opentopped-properties) |
| PH714 多类型子机队列 | 发射器可按 Spawns.Queue 依次生成不同子机，数量仍由 SpawnsNumber 决定，缺项由 Spawns 补足；混合导弹与普通子机可能使导弹无法命中。 | `Spawns.Queue` | 扩展子机槽位配置与生成策略，宿主验证队列长度及导弹类型兼容性。 | L · [Customizable spawns queue](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-spawns-queue) |
| PH715 Ares 雷达干扰扩展 | 可定制雷达干扰影响的阵营、检查周期与雷达建筑白黑名单；缩短周期能更及时生效，也会增加检查成本。 | `RadarJamHouses`、`RadarJamDelay`、`RadarJamAffect`、`RadarJamIgnore` | 先复用雷达干扰关系宿主，再开放范围和筛选策略，集中更新阵营雷达状态。 | L · [Customize Ares’s radar jam logic](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-ares-s-radar-jam-logic) |
| PH716 出售声音与 EVA | 建筑或单位出售时可单独指定 EVA 提示和出售声音，声音面向拥有者；车辆出售声音也改为与建筑一致的归属可闻行为。 | `EVA.Sold`、`SellSound` | 由出售事务生成面向指定玩家的音频事件，插件只选择声音与 EVA 类型。 | L/S · [Customize EVA voice and SellSound when selling units](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-eva-voice-and-sellsound-when-selling-units) |
| PH718 混编部队展开优先级 | 单位可设置高低展开优先级或指定冲突类型，使混合选中时只向合适单位下达展开；用户侧还可关闭优先级过滤，单对象光标行为不受影响。 | `LowDeployPriority`、`HighDeployPriority`、`DeployForbidTypes`、`PriorityDeployFiltering` | 在输入命令展开前统一筛选选择集，展开是否合法仍交给模拟宿主。 | L · [Customize whether the unit can deploy](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-whether-the-unit-can-deploy) |
| PH719 驾驶员被杀后的乘客处理 | 驾驶员死亡后可选择保留或杀死乘客，保留选项优先；其敞篷运输用途有特殊归属及任务处理，上游注明尚未充分适配其他场景。 | `DriverKilled.KeptPassengers`、`DriverKilled.KillPassengers` | 先完成驾驶员、乘客与操作者关系宿主，再开放策略，明确保留与归属的边界。 | L · [Customize whether transport can kept or kill passengers when driver has been killed](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-whether-transport-can-kept-or-kill-passengers-when-driver-has-been-killed) |
| PH734 禁止自动退回副武器 | 可禁止主武器无法攻击时自动选择副武器，并单独允许防空回退及对墙选择；运输、空弹药、展开及多武器等专用选择路径仍有例外。 | `AllowWeaponSelectAgainstWalls`、`NoSecondaryWeaponFallback`、`NoSecondaryWeaponFallback.AllowAA` | 扩展武器选择策略，保留专用选择入口的优先级并由宿主返回最终武器索引。 | L/S · [Disabling fallback to (Elite)Secondary weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#disabling-fallback-to-elite-secondary-weapon) |
| PH735 伪装闪烁与移动速度 | 可按阵营配置伪装闪烁可见性，并可让单位采用伪装类型的移动速度；即使伪装已被看破，只要未移除，速度规则仍继续生效。 | `DisguiseBlinkingVisibility`、`UseDisguiseMovementSpeed` | 伪装速度由模拟宿主合成，闪烁与军衔外观在表现宿主按本地可见性处理。 | L/S · [Disguise logic additions (disguise-based movement speed, disguise blinking visibility)](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#disguise-logic-additions-disguise-based-movement-speed-disguise-blinking-visibility) |
| PH736 死亡掉落工具箱 | 步兵或车辆配置 DropCrate 后，死亡时可生成指定工具箱，作为独立于普通死亡武器的掉落效果。 | `DropCrate` | 在死亡主链派发工具箱生成请求，由地图宿主验证落点并管理实体生命周期。 | L · [Drop crates on death](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#drop-crates-on-death) |
| PH739 排除基地中心计算 | 单位类型可不参与基地中心计算，影响相关 AI 脚本和超武选点；通常统计建筑，但初始基地中心也可能统计起始单位。 | `IgnoreForBaseCenter` | 修改统一基地中心查询策略，维护计数与缓存失效，避免不同 AI 功能各算一次。 | L · [Exclusion from base center calculations](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#exclusion-from-base-center-calculations) |
| PH741 加特林热量下降扩展 | 可设置开始下降前的延迟、失去或更换目标时是否重置热量，以及弹药低于阈值时改用另一热量下降数值。 | `RateDown.Delay`、`RateDown.Reset`、`RateDown.Cover.Value`、`RateDown.Cover.AmmoBelow` | 由加特林计时与等级宿主消费策略，在失去目标和弹药提交处更新同一状态。 | L · [Extended gattling rate down logic](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#extended-gattling-rate-down-logic) |
| PH743 额外目标威胁评分 | 索敌评分可加入目标是否威胁自身、是否在射程内、距离、朝向差及与上个目标的距离等项；上个目标记录有短暂保留期。 | `ExtraThreat.IsThreat`、`ExtraThreat.InRange`、`ExtraThreatCoefficient.InRangeDistance`、`ExtraThreatCoefficient.Facing`、`ExtraThreatCoefficient.DistanceToLastTarget`、`AlwaysConsideredThreat` | 扩展宿主威胁评分策略并提供只读快照，上个目标和更新频率仍由索敌宿主持有。 | L · [Extra threat](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#extra-threat) |
| PH745 运输工具快速上下客 | 乘客可免排队快速进入或离开车辆及建筑，并可配置登载距离；它面向 Passenger，不能直接等同驻军 Occupier，距离太小可能使乘客不停靠近。 | `NoQueueUpToEnter`、`NoQueueUpToUnload`、`NoQueueUpToEnter.BoardDistance`、`NoQueueUpToEnter.Buildings`、`NoQueueUpToUnload.Buildings` | 扩展上下客宿主的排队和距离策略，保留合法路径、容量与占格验证。 | L · [Fast access vehicle/structure](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#fast-access-vehicle-structure) |
| PH748 逐发 Burst 开火点 | 可为主副武器或编号武器的每发连射配置 FLH，并区分精英、卧倒和展开；序号从零开始，缺项回退基础 FLH，专用偏移不再额外左右交替。 | `FLHKEY.BurstN` | 在宿主最终 FLH 计算处接入逐发配置，统一索引、姿态与精英回退规则。 | L/S · [Firing offsets for specific Burst shots](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#firing-offsets-for-specific-burst-shots) |
| PH749 按目标条件强制选武器 | 可按隐形、伪装、EMP、海军、目标类型或距离强制选武器，并区分防空列表；多个条件满足时遵循规定优先级，而非选择最高伤害武器。 | `ForceWeapon.InRange.TechnoOnly`、`ForceWeapon.InRange.ApplyRangeModifiers`、`ForceAAWeapon.InRange.ApplyRangeModifiers`、`ForceWeapon.Naval.Decloaked`、`ForceWeapon.Cloaked`、`ForceWeapon.Disguised`、`ForceWeapon.UnderEMP` 等（20个不同示例键） | 为武器选择宿主开放有序规则表，统一验证索引及射程倍率的应用位置。 | L/S · [Forcing specific weapon against certain targets](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#forcing-specific-weapon-against-certain-targets) |
| PH752 初始子机数量 | 可设置发射器初始填充的子机数量，而非开局总是满槽；初始数量超过 SpawnsNumber 时此设置不会按超额槽位生效。 | `InitialSpawnsNumber` | 在子机管理宿主初始化时消费配置，保持初始槽位与后续补充共用状态。 | L · [Initial spawns number](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#initial-spawns-number) |
| PH753 初始生命与克隆兵生命 | 单位可设置出生时生命值；克隆兵还可用固定比例或随机比例范围决定初始生命，两者均不会超过类型最大 Strength。 | `InitialStrength`、`InitialStrength.Cloning` | 在单位生成宿主统一应用上限与克隆来源策略，随机比例由宿主取样。 | L/S · [Initial strength for TechnoTypes and cloned infantry](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#initial-strength-for-technotypes-and-cloned-infantry) |
| PH757 条件自动死亡与移除 | 可按弹药、归属变化、延迟、单位存在、电力或资金条件触发摧毁、消失或出售；支持 limbo 状态，但无实体建筑的这些行为都表现为直接移除。 | `AutoDeath.PlayerMoney.Max`、`AutoDeath.PlayerMoney.Min`、`AutoDeath.AllowLimboed`、`AutoDeath.OnOwnerChange.IgnoreRevertOnExit`、`AutoDeath.TechnosDontExist.AllowLimboed`、`AutoDeath.TechnosExist.AllowLimboed`、`AutoDeath.Behavior` 等（20个不同示例键） | 新增条件事件与受控生命周期命令，统一存档计时、容器清理和阵营统计。 | L/S · [Kill Object Automatically](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#kill-object-automatically) |
| PH763 心灵控制距离与容量扩展 | 心控可限制最大控制距离，按目标体积占用名额，并按阵营显示连线；多重心控满额时还可先释放最早受控单位再接纳新目标。 | `MindControl.IgnoreSize`、`MindControlLink.VisibleToHouse`、`MultiMindControl.ReleaseVictim`、`MindControlRangeLimit`、`MindControlSize` | 扩展心控关系宿主，集中管理容量、距离检测、释放顺序和表现连线。 | L/S · [Mind Control enhancement](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#mind-control-enhancement) |
| PH765 按武器选择攻击语音 | 多武器单位可分别配置各 WeaponN 的攻击语音，精英武器也可单独覆盖；精英语音缺省时退回同编号普通武器语音。 | `VoiceWeaponNAttack`、`VoiceEliteWeaponNAttack` | 在最终武器选择确定后派发声音事件，配置解析统一编号与精英回退。 | L · [Multi VoiceAttack](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#multi-voiceattack) |
| PH766 非特定硬编码单位多武器 | 普通单位可启用多个 WeaponX，而不必借用 Gunner 或加特林类型；可指定哪些武器视作副武器，并限制普通选择逻辑遍历的数量以控制成本。 | `MultiWeapon`、`MultiWeapon.IsSecondary`、`MultiWeapon.SelectCount` | 扩展武器槽位与选择宿主，保留显式索引访问并避免对每次攻击无界遍历。 | L · [Multi Weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#multi-weapon) |
| PH768 空弹药多武器候选 | 当 NoAmmoAmount 条件满足时，可从 NoAmmoWeapons 中按目标资格等规则选择武器；没有可用候选时继续回退原来的 NoAmmoWeapon。 | `NoAmmoWeapons`、`NoAmmoWeapons.IgnoreNeverUse` | 在原空弹药选择分支扩展候选查询，复用射击与弹药宿主而不另造弹药状态。 | L · [No Ammo Weapons](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#no-ammo-weapons) |
| PH769 禁止手动移动命令 | NoManualMove 可禁止单位接受移动命令，并减少可收起建筑因设置集结点而收起；弹头强制收起或集结点后出售等路径仍可绕过此限制。 | `NoManualMove` | 在玩家命令准入处过滤，保留脚本及强制生命周期入口的独立权限。 | L/S · [No Manual Move](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#no-manual-move) |
| PH770 EMP 或超时空中停止装填 | 可分别禁止 EMP 或时间武器锁定状态下的武器冷却及弹药补充，并支持全局默认和单位覆盖；不影响机场飞机的补弹流程。 | `NoRearm.UnderEMP`、`NoRearm.Temporal`、`NoReload.UnderEMP`、`NoReload.Temporal` | 在装填与弹药计时所有者统一处理暂停，不用 guest tick 反复修正宿主计时器。 | L · [No rearm and reload in EMP or temporal](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#no-rearm-and-reload-in-emp-or-temporal) |
| PH771 无限心控过载去硬编码 | 对装备 InfiniteMindControl 武器的单位，可自定义控制过载的数量阈值、伤害、间隔、死亡声音和粒子系统；这是心控过载，不是护盾热量过载。 | `Overload.ParticleSysCount`、`Overload.Count`、`Overload.Damage`、`Overload.Frames`、`Overload.DeathSound`、`Overload.ParticleSys` | 接入心控容量与过载计时宿主，受控自伤与表现分别提交，勿复用自定义护盾热量状态。 | L · [Overload characteristic dehardcoded](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#overload-characteristic-dehardcoded) |
| PH781 子机随载体升级 | 发射器的子机可随载体军衔晋升，避免载体已经升级而新旧子机仍全部保持原等级。 | `Promote.IncludeSpawns` | 通过军衔宿主事件通知子机管理器，明确已有子机与新生子机的继承时机。 | L/S · [Promoted Spawns](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#promoted-spawns) |
| PH782 单位晋升动画 | 单位或建筑晋升为老兵或精英时可播独立动画，支持全局与类型覆盖及列表随机选择；精英未配置时可回退老兵动画。 | `Promote.VeteranAnimation`、`Promote.EliteAnimation` | 从军衔提交事件派发表现请求，统一配置回退与随机选择，避免重复晋升动画。 | L/S · [Promotion animation](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#promotion-animation) |
| PH785 单位受伤战斗警报 | 非建筑单位受伤也可触发雷达事件与声音，按屏内外、阵营、单位和弹头抑制条件过滤，并用间隔避免提示过密；声音有反馈、攻击、EVA 选择优先级。 | `CombatAlert`、`CombatAlert.Default`、`CombatAlert.IgnoreBuilding`、`CombatAlert.SuppressIfInScreen`、`CombatAlert.Interval`、`CombatAlert.SuppressIfAllyDamage`、`CombatAlert.MakeAVoice` 等（13个不同示例键） | 从伤害宿主派发警报候选，屏幕内判断和声音限流放在本地表现层。 | L · [Raise alert when technos are taking damage](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#raise-alert-when-technos-are-taking-damage) |
| PH789 重计连发序号 | 单位更换开火武器，或一段时间没有目标后，可重新计算当前 Burst 序号；无目标等待阈值取上次武器 ROF 与三十帧中的较大值。 | `RecountBurst` | 让武器时序宿主独占连发索引与无目标计时，插件提供重置策略。 | L · [Recount burst index](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#recount-burst-index) |
| PH790 晋升调整弹药补充速度 | 老兵和精英能力可分别缩放普通补弹及空弹夹补弹周期，既有修正仍参与；倍率大于一反而变慢，EmptyReload 为负一时不走空弹夹能力路径。 | `VeteranReload`、`VeteranEmptyReload` | 扩展军衔到弹药计时的策略投影，统一最终周期计算和普通、空仓分支。 | L · [Reload speed adjustment on promotion](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#reload-speed-adjustment-on-promotion) |
| PH792 类型转换后重置心控 | 单位转换类型后可重算心控容量：新类型没有心控时释放全部；仍有心控时更新上限，非无限心控还会释放超出容量的受控单位。 | `Convert.ResetMindControl` | 在统一类型转换事务中调用心控宿主迁移，保证控制关系清理完整且有序。 | L/S · [Reset MindControl after transformation](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#reset-mindcontrol-after-transformation) |
| PH796 死亡报复武器 | 单位被其他单位直接伤害杀死时，可向造成致命伤的对象发射 RevengeWeapon，并按阵营和弹头抑制规则过滤；环境等无直接单位来源的死亡不会触发。 | `RevengeWeapon`、`RevengeWeapon.AffectsHouse`、`SuppressRevengeWeapons`、`SuppressRevengeWeapons.Types` | 从死亡归因宿主派发报复请求，统一来源有效性、目标失效和连锁边界。 | L/S · [Revenge weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#revenge-weapon) |
| PH800 敞篷运输共享弹药 | 敞篷运输工具可向启用 Ammo.Shared 的乘客输送自身弹药，按组号筛选；运输工具组号负一可忽略分组，且需要有弹药并能补弹。 | `Ammo.Shared`、`Ammo.Shared.Group` | 新增跨对象受控弹药转移事务，由同一弹药宿主处理扣除、补充与组匹配。 | L/S · [Shared Ammo](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#shared-ammo) |
| PH804 单位创建声音 | 单位创建时可向拥有者播放 VoiceCreated，也可启用全局播放并替代通常的 EVA_UnitReady；需要区分拥有者提示与所有玩家可闻事件。 | `IsVoiceCreatedGlobal`、`VoiceCreated` | 从单位创建完成事件派发音频请求，按接收范围处理，不从 guest 直接访问音频。 | L/S · [Sound entry on unit’s creation](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#sound-entry-on-unit-s-creation) |
| PH809 狂乱单位的目标阵营 | BerzerkTargeting 可限定狂乱单位允许瞄准和攻击哪些阵营，使狂乱不再只能接受统一的原版目标范围。 | `BerzerkTargeting` | 在狂乱索敌与开火资格宿主共享阵营策略，避免只限制显示光标而遗漏自动攻击。 | L · [Targeting limitation for berzerk technos](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#targeting-limitation-for-berzerk-technos) |
| PH811 直接吞矿变现 | 单位可周期性把自身或偏移格的矿石直接转为资金，配置每格数量、售价倍率、显示与动画，并选择 EMP 时是否继续；动画可按朝向或矿种选择。 | `TiberiumEater.TransDelay`、`TiberiumEater.CellN`、`TiberiumEater.CashMultiplier`、`TiberiumEater.AmountPerCell`、`TiberiumEater.Display`、`TiberiumEater.Display.Houses`、`TiberiumEater.Anims` 等（13个不同示例键） | 新增矿物消耗与经济联动事务，固定格遍历、随机动画及资源扣除顺序。 | L · [Tiberium eater](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#tiberium-eater) |
| PH822 晋升调整武器射程 | 老兵与精英可通过 RANGE 能力调整射程，精英能力叠加老兵能力；倍率大于一增加射程，启用 Projectile.ApplyModifiers 时也影响弹体 ProjectileRange。 | `VeteranRange` | 由军衔策略投影到统一射程查询，明确弹体距离是否继承及能力累加顺序。 | L · [Weapon range adjustment on promotion](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#weapon-range-adjustment-on-promotion) |
| PH824 传送出入时触发武器 | 单位传送离开和到达时可触发武器，短距传送可用替代到达武器，还可按传送整格距离覆盖到达伤害；与是否启用 ChronoTrigger 无关。 | `WarpInWeapon`、`WarpInMinRangeWeapon`、`WarpInWeapon.UseDistanceAsDamage`、`WarpOutWeapon` | 从传送宿主阶段事件发武器请求，统一距离单位、短距回退和归属继承。 | L/S · [Weapons fired on warping in / out](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#weapons-fired-on-warping-in-out) |

### 超级武器

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH664 AI 超武检查间隔 | 允许调整 AI 检查并发射超级武器的间隔；原版约为 106—112 逻辑帧，配置为零或负数时改为每帧检查。 | `AISuperWeaponDelay` | 接入 AI 超武调度器，统一持有计时与随机数消耗，插件只返回延迟策略。 | L/S · [AI Superweapon delay timer](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#ai-superweapon-delay-timer) |
| PH687 超武的单位与科技条件 | 超武可要求至少拥有列表中一种辅助单位、不得拥有任何禁用单位，并满足指定科技等级；科技等级负值不承担只允许 AI 的特殊含义。 | `SW.AuxTechnos`、`SW.NegTechnos`、`SW.TechLevel` | 让超武可用性宿主查询统一单位计数与科技状态，插件返回条件判断。 | L · [Aux technos and TechLevel requirement of superweapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#aux-technos-and-techlevel-requirement-of-superweapon) |
| PH699 超武批量类型转换 | 通过超级武器按类型组与阵营转换同类别单位，包含运输工具、驻军建筑等内部单位；不支持建筑，上游完整属性转换依赖 Ares 3.0。 | `Convert.From`、`Convert.To`、`Convert.AffectsHouse`、`ConvertN.From`、`ConvertN.To`、`ConvertN.AffectsHouse` | 先建立统一转换事务与容器遍历查询，再由超武策略发出批量转换请求。 | L/S · [Convert TechnoType](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#convert-technotype) |
| PH717 超级武器侧栏分类 | 超级武器图标可分配到四个侧栏页签，TabIndex 的零至三分别对应建筑、防御、步兵和车辆页。 | `TabIndex` | 由侧栏宿主消费冻结的图标分类配置，不通过战斗插件修改 UI 状态。 | L/S · [Customize SuperWeapon TabIndex](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-superweapon-tabindex) |
| PH737 EMPulse 建筑武器与互斥 | Ares EMPulse 超武可指定发射建筑使用的武器索引，并暂停其他共享发射建筑的 EMPulse；目标射程默认仍取主武器，除非显式设置超武范围。 | `EMPulse.WeaponIndex`、`EMPulse.SuspendOthers` | 接入超武发射调度与建筑武器宿主，明确共享炮台竞争和延迟发射覆盖规则。 | L/S · [EMPulse settings](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#empulse-settings) |
| PH760 离图建筑投送与移除 | 超武可生成不占地图位置但提供部分建筑能力的 limbo 建筑，并按 ID 删除或加权随机投送；工厂、机场等需要实体交互的机制不应使用此形式。 | `LimboDelivery.Types`、`LimboDelivery.IDs`、`LimboDelivery.RollChances`、`LimboDelivery.RandomWeightsN`、`LimboKill.AffectsHouse`、`LimboKill.IDs`、`LimboKill.Counts` | 扩展阵营建筑注册与受控离图实体，明确可支持能力、胜负统计和销毁路径。 | L/S · [LimboDelivery](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#limbodelivery) |
| PH761 关联超武授予与计时 | 一项超武发射后可授予关联超武，或重置、充满其计时，并可按概率和权重选择；重置优先于就绪，还可向玩家显示关联获得提示。 | `SW.Link`、`SW.Link.Grant`、`SW.Link.Ready`、`SW.Link.Reset`、`SW.Link.RollChances`、`SW.Link.RandomWeightsN`、`Message.LinkedSWAcquired` 等（8个不同示例键） | 由超武宿主原子更新授予与计时，插件返回关联策略，统一随机选择和提示事件。 | L · [Linked superweapons](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#linked-superweapons) |
| PH767 同目标连锁发射超武 | 超武可在原目标处继续发射其他超武，支持概率加权、实际拥有及资金校验、指示器与抑制器过滤；强制发射模式可跳过部分正常条件。 | `SW.Next`、`SW.Next.RealLaunch`、`SW.Next.IgnoreInhibitors`、`SW.Next.IgnoreDesignators`、`SW.Next.RollChances`、`SW.Next.RandomWeightsN` | 通过统一超武发射宿主串接有序请求，明确资金检查和循环连锁边界。 | L/S · [Next](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#next) |
| PH788 超武按接收阵营提示 | 超武激活时可分别向自身拥有者、其他盟友和敌人显示信息并播放 EVA；盟友组不包含激活者自己，适合不同阵营接收不同告警。 | `Message.Activated.Owner`、`Message.Activated.Allies`、`Message.Activated.Enemies`、`EVA.Activated.Owner`、`EVA.Activated.Allies`、`EVA.Activated.Enemies` | 从超武提交事件生成按接收组分类的提示，接收方过滤不得影响模拟结果。 | L · [Recipient-specific message and EVA on superweapon activation](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#recipient-specific-message-and-eva-on-superweapon-activation) |
| PH819 超武目标点引爆弹头或武器 | 任意超武可在目标格或发射建筑处引爆弹头或武器，武器优先，并可覆盖伤害；找不到发射建筑时部分效果仍按超武所属方处理，自身引爆则跳过。 | `Detonate.Warhead`、`Detonate.Warhead.Full`、`Detonate.Weapon`、`Detonate.Damage`、`Detonate.AtFirer` | 复用超武和爆炸宿主请求，明确发射者缺失、伤害默认与完整爆炸范围。 | L/S · [Warhead or Weapon detonation at target cell](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#warhead-or-weapon-detonation-at-target-cell) |

### 建筑

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH666 学院的国家过滤 | 在国家启用 Academy.AllowCountryFilter 后，Ares 学院按国家专属的接收与排除类型列表筛选，不再使用普通学院类型列表。 | `Academy.AllowCountryFilter`、`Academy.Country.Types.N`、`Academy.Country.Ignore.N` | 先接入学院授级机制，再让配置策略按国家查询；不要复制一套经验状态。 | L · [Allow Ares Academy add country-level filters](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#allow-ares-academy-add-country-level-filters) |
| PH690 建筑邻接建造范围定制 | 可按建筑类型允许或禁止借用邻接范围，并可让禁用建筑形成严格禁建区域；也可禁止在建造动画尚未完成的建筑旁继续建造。 | `Adjacent.Allowed`、`Adjacent.Disallowed`、`Adjacent.Disallowed.Prohibit`、`Adjacent.Disallowed.ProhibitDistance`、`NoBuildAreaOnBuildup` | 扩展放置资格查询，保持地图占格、邻接与预览使用同一宿主判定。 | L/S · [Build area customizations](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#build-area-customizations) |
| PH692 放置建筑时自动疏散 | 建筑放置或 MCV 展开时可要求可控占位单位让开，等待清空后完成放置，超时或条件变化则失败；另可将升级件自动放到合适的早期建筑上。 | `ExtendedBuildingPlacing`、`AutoUpgrade` | 让放置宿主持有待放置事务、预览和超时，插件只提供疏散或升级匹配策略。 | L · [Building placing and deploying logic enhancement](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#building-placing-and-deploying-logic-enhancement) |
| PH731 可破坏建筑寻路障碍 | 建筑可被视为类似墙的可破坏障碍，只有合适 MovementZone 的单位才尝试穿越；单位若无有效破障武器，可能一直卡在障碍前。 | `IsDestroyableObstacle` | 扩展寻路通行策略并衔接破障攻击任务，地图阻挡状态仍由宿主维护。 | L/S · [Destroyable pathfinding obstacles](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#destroyable-pathfinding-obstacles) |
| PH738 工程师修理额度 | 工程师与建筑可分别限制单次修理量，双方均设置时取较小值；零代表修满，负数表示按比例修理，并可逐建筑指定修复声音。 | `EngineerRepairAmount`、`BuildingRepairedSound` | 在工程师接触修理宿主计算额度，统一提交生命变化和修复表现事件。 | L · [Engineer repair customization](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#engineer-repair-customization) |
| PH740 建筑升级件扩展 | 升级件可放在己方、盟友或敌方建筑上，并支持多个基础建筑类型，放置后归属随建筑；Ares 对通过扩展列表放置的升级件前置条件有已知限制。 | `PowersUp.Owner`、`PowersUp.Buildings` | 扩展升级件放置宿主与归属事务，统一生产上限和前置条件的统计口径。 | L/S · [Extended building upgrades](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#extended-building-upgrades) |
| PH777 发电厂增强建筑 | 增强建筑存在时可为配置范围和类型的发电厂增加产能，支持固定量、倍率与参与数量上限；出售或摧毁增强建筑后恢复产能。 | `PowerPlantEnhancer.PowerPlants`、`PowerPlantEnhancer.Range`、`PowerPlantEnhancer.Amount`、`PowerPlantEnhancer.Factor`、`PowerPlantEnhancer.MaxCount` | 扩展电力统计宿主与失效更新事件，插件提供增益规则，禁止每帧重复加电。 | L/S · [Power plant enhancer](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#power-plant-enhancer) |
| PH798 工厂屋顶生产动画 | 步兵或车辆从工厂屋顶出口离开时，可使用独立 RoofProductionAnim 系列替代普通生产动画；出口方式由 ExitThroughRoof 等单位属性决定。 | `RoofProductionAnim`、`RoofProductionAnimDamaged`、`RoofProductionAnimGarrisoned`、`RoofProductionAnimX`、`RoofProductionAnimY`、`RoofProductionAnimYSort`、`RoofProductionAnimZAdjust` 等（11个不同示例键） | 由生产出厂宿主决定出口上下文，渲染宿主消费相应动画与偏移配置。 | L · [Roof production anim](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#roof-production-anim) |
| PH806 间谍渗透触发超武 | 启用自定义间谍效果后，可在被渗透建筑位置立即为受害者或渗透者发射指定超武；两项分别决定超武归属方。 | `SpyEffect.Custom`、`SpyEffect.VictimSuperWeapon`、`SpyEffect.InfiltratorSuperWeapon` | 从渗透成功事件向统一超武宿主提交请求，明确目标坐标、归属和效果递归边界。 | L/S · [Spy effects](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#spy-effects) |

### 步兵

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH667 步兵展开收起时转换类型 | 步兵完成展开或收起动作后可立即转换为另一类型；无展开序列的可部署步兵收到展开命令后直接执行展开转换。 | `Convert.Deploy`、`Convert.Undeploy` | 增加序列完成事件与类型转换宿主命令，由转换所有者处理状态迁移。 | L · [Allow infantry to perform type conversion when deploying and undeploying](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#allow-infantry-to-perform-type-conversion-when-deploying-and-undeploying) |
| PH710 步兵卧倒与展开开火点 | 可在美术配置中为步兵卧倒或展开状态分别覆盖主副武器 FLH，并与特定连发序号的开火偏移一起使用。 | `PronePrimaryFireFLH`、`ProneSecondaryFireFLH`、`DeployedPrimaryFireFLH`、`DeployedSecondaryFireFLH` | 从动画姿态与武器宿主导出最终 FLH，避免插件和渲染各算一份坐标。 | L/S · [Customizable FLH when infantry is prone or deployed](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-flh-when-infantry-is-prone-or-deployed) |
| PH711 步兵序列播放速率 | 各步兵动作可定义每张图像停留的逻辑帧数，以及是否随游戏速度归一化；零速率代表静止，归一化还存在分档换算规则。 | `Sequence.<Sequence>.DefaultRate`、`Sequence.<Sequence>.DefaultNormalized`、`<Sequence>Rate`、`<Sequence>Normalized` | 由序列播放宿主读取策略并统一推进，涉及开火帧时同步武器时序。 | L · [Customizable infantry sequence rates](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-infantry-sequence-rates) |
| PH713 逐步兵设置奴隶解放声音 | 每种可被奴役的步兵都可独立设置 SlavesFreeSound，用于奴隶解放时的声音，不必共用全局配置。 | `SlavesFreeSound` | 在奴隶解除归属事件附带类型化声音请求，播放由音频宿主处理。 | L/S · [Customizable SlavesFreeSound](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-slavesfreesound) |
| PH727 逐步兵默认伪装 | 步兵可按自身类型覆盖 DefaultDisguise，优先级高于 Ares 的阵营级默认伪装，从而使同阵营不同间谍使用不同初始外观。 | `DefaultDisguise` | 复用伪装状态宿主，并在配置合成时落实单位级高于阵营级的覆盖顺序。 | L/S · [Default disguise for individual InfantryTypes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#default-disguise-for-individual-infantrytypes) |
| PH786 非人类步兵随机死亡动作 | NotHuman 步兵可随机播放 Die1 至 Die5 死亡序列，而非固定 Die1；需要素材实际提供这些动作，否则可能播放不可见序列。 | `NotHuman.RandomDeathSequence` | 死亡宿主选择确定性动作索引，资源加载阶段验证相应序列存在。 | L/S · [Random death animaton for NotHuman infantry](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#random-death-animaton-for-nothuman-infantry) |
| PH803 奴隶主死亡后的归属 | 奴隶矿车被杀后，奴隶可自毁、保留原归属、交给平民或交给击杀者；最后一种对应原版行为，其他模式用于改变解放后阵营关系。 | `Slaved.OwnerWhenMasterKilled` | 在奴隶主死亡主链调用归属或死亡宿主，统一解除奴役关系和后续任务。 | L/S · [Slaves’ house decision customization when owner is killed](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#slaves-house-decision-customization-when-owner-is-killed) |
| PH817 水中使用陆地步兵动作 | 步兵在水中也可只显示陆地动作序列，而不切换到游泳等水中动作；这是动作选择策略，不能自行改变单位的实际地形通行能力。 | `OnlyUseLandSequences` | 放在步兵表现与序列选择宿主，保持动作显示和移动资格分离。 | L/S · [Use land sequences even in water](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#use-land-sequences-even-in-water) |

### 武器

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH668 激光端点跟随更新 | 激光存在期间可让起点跟随发射者、终点跟随目标或两者都跟随；支持转换后停止更新。分裂子弹激光的起点不能跟随原发射者。 | `LaserPositionUpdate.StopOnFirerConvert`、`LaserPositionUpdate` | 放在渲染事件与对象句柄解析层，插件给跟随策略；绘制位置不得改变战斗结算。 | L · [Allow Laser drawing position update](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#allow-laser-drawing-position-update) |
| PH673 AreaFire 目标选择 | AreaFire 武器可朝当前格、发射者自身或射程内随机格开火；自身目标仍遵循正常武器选择，随机格会按目标与阵营过滤。 | `AreaFire.Target` | 在武器目标解析阶段增加策略，随机选格使用宿主确定性随机接口。 | L/S · [AreaFire target customization](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#areafire-target-customization) |
| PH679 武器攻击无威胁建筑 | 武器层面的 AttackNoThreatBuildings 可决定是否攻击无威胁建筑，且该设置对相关其他配置具有覆盖优先级。 | `AttackNoThreatBuildings` | 在统一目标资格合成时明确武器覆盖顺序，插件返回策略而非自行索敌。 | L · [Attack non-threatening structures (Weapon)](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attack-non-threatening-structures-weapon) |
| PH693 连发各发延迟与序列内开火 | Burst.Delays 可逐发定义连发延迟并回退到最后一项；特定步兵和无炮塔 SHP 车辆可在同一射击序列发多枪，飞机不适用该延迟机制。 | `Burst.Delays`、`Burst.FireWithinSequence` | 接入武器射击时序宿主，并同步动画开火帧、连发索引和装填计时。 | L/S · [Burst delay customizations](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#burst-delay-customizations) |
| PH694 同一帧完成连发 | 车辆与步兵可在同一逻辑帧发完整组 Burst，忽略 Burst.Delays；该开关对建筑和飞机没有作用。 | `Burst.NoDelay` | 由武器发射宿主批量提交连发，明确每发的弹药、目标与随机数结算顺序。 | L · [Burst without delay](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#burst-without-delay) |
| PH721 针对无驾驶员载具的目标过滤 | 武器可按驾驶员是否已被杀死筛选车辆目标，补充 Ares 的相关目标限制；上游要求至少 Ares 3.0 才能使用此驾驶员状态。 | `CanTarget.DriverKilled` | 为统一目标资格查询提供驾驶员状态，后续复用驾驶员机制的单一所有者。 | L · [Customize whether weapon can target driver-killed vehicles](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-whether-weapon-can-target-driver-killed-vehicles) |
| PH722 攻击铁幕目标的策略 | 可分别控制玩家与电脑是否允许攻击铁幕对象，以及玩家单位是否主动索敌这类目标；电脑对主动索敌配置采用自己的判定路径。 | `CanTarget.IronCurtained`、`CanTargetAI.IronCurtained`、`AutoTarget.IronCurtained` | 将手动攻击、AI 索敌与玩家自动索敌入口分开接入，再共享基础资格规则。 | L · [Customize whether weapon can target iron-curtained technos](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-whether-weapon-can-target-iron-curtained-technos) |
| PH729 延迟开火与蓄力动画 | 武器可固定或随机延迟后开火，配合蓄力动画、运输状态跳过、步兵序列暂停及仅首发延迟；飞机不会为过长延迟一直停留等待。 | `DelayedFire.Duration`、`DelayedFire.SkipInTransport`、`DelayedFire.Animation`、`DelayedFire.OpenToppedAnimation`、`DelayedFire.AnimIsAttached`、`DelayedFire.AnimOffset`、`DelayedFire.AnimOnTurret` 等（11个不同示例键） | 扩展武器时序宿主与动画事件，保存延迟计时并定义中断、连发续接和目标丢失语义。 | L · [Delayed firing](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#delayed-firing) |
| PH742 条件额外射程 | 武器可在目标移动、自身移动或预开火时获得额外射程，缓解近战与蓄力丢失攻击机会；只影响攻击过程，不改变寻路等其他射程用途。 | `ExtraRange.TargetMoving`、`ExtraRange.TargetMoving.CloseRangeOnly`、`ExtraRange.FirerMoving`、`ExtraRange.Prefiring`、`ExtraRange.Prefiring.IncludeBurst` | 区分攻击资格、寻路与索敌的射程查询，插件只覆盖指定消费者。 | L · [Extra range](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#extra-range) |
| PH744 命中额外弹头 | 同一武器命中后可按顺序引爆额外弹头，独立配置伤害、概率、完整爆炸或加权抽取；弹体需保留武器来源，部分空爆子武器路径不适用。 | `ExtraWarheads`、`ExtraWarheads.DamageOverrides`、`ExtraWarheads.DetonationChances`、`ExtraWarheads.FullDetonation`、`ExtraWarheads.RollChances`、`ExtraWarheads.RandomWeightsN` | 由爆炸宿主生成有序子事务，统一随机抽取、伤害归属与递归深度。 | L/S · [Extra warhead detonations](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#extra-warhead-detonations) |
| PH746 开火反馈武器 | 武器开火后可对发射者额外使用 FeedbackWeapon；敞篷运输内开火时按反馈武器许可决定，若触发则作用于运输工具，且不应用敞篷伤害倍率。 | `FeedbackWeapon` | 增加发射成功后的受控武器请求，明确反馈目标和倍率来源，防止自递归。 | L/S · [Feedback weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#feedback-weapon) |
| PH756 开火后保持距离 | ROF 计时期间可保持指定距离：正值偏向退到距离外，负值尽量贴近该距离；可分别允许玩家与 AI，并在装填剩余时间很短时提前停止。 | `KeepRange.AllowAI`、`KeepRange.AllowPlayer`、`KeepRange.EarlyStopFrame`、`KeepRange` | 扩展战斗移动任务策略，宿主持有 ROF 与移动命令，防止与追击逻辑互相覆盖。 | L · [Keep Range After Firing](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#keep-range-after-firing) |
| PH762 OmniFire 开火仍转向目标 | OmniFire 单位可在不要求先对准目标也能开火的同时，尝试将车体转向目标；Jumpjet 建议协调车体与运动器的转向速率。 | `OmniFire.TurnToTarget` | 由运动及朝向宿主执行转向意图，维持开火许可与朝向更新的独立关系。 | L/S · [Make units try turning to target when firing with OmniFire=yes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#make-units-try-turning-to-target-when-firing-with-omnifire-yes) |
| PH784 默认辐射的扩展能力 | 原版 Radiation 节定义的默认辐射也可使用自定义辐射带来的部分增强，如记录归属与施放者、伤害建筑；具体规则与自定义辐射章节一致。 | 无独立示例键；见原文条件 | 复用辐射场宿主与同一参数模型，不为默认辐射再实现一套伤害循环。 | L/S · [Radiation enhancements](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#radiation-enhancements) |
| PH787 圆柱式射程测量 | 可在全局或武器上启用忽略高度差的射程判断；false 表示回到默认规则，而非强制三维测距，原本空中单位仍按原版圆柱方式测距。 | `CylinderRangefinding` | 为射程查询提供明确几何模式并保留默认继承语义，勿把 false 当反向覆盖。 | L · [Range finding in cylinder](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#range-finding-in-cylinder) |
| PH807 飞机扫射武器定制 | 可配置飞机是否扫射、每轮发数、模拟 Burst、逐发耗弹、改锁目标格及扫射结束延迟；默认通常整轮只扣一次弹药，侦察机超武另有揭图次数用途。 | `Strafing.SimulateBurst`、`Strafing.UseAmmoPerShot`、`Strafing.TargetCell`、`Strafing`、`Strafing.Shots`、`Strafing.EndDelay` | 在飞机攻击任务宿主接入策略，统一任务阶段、耗弹时机、目标失效和返航条件。 | L/S · [Strafing aircraft weapon customization](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#strafing-aircraft-weapon-customization) |
| PH818 光束视觉散布 | 激光、电弧或辐射束可在目标显示位置增加随机偏移，并可与弹道散布共存；只改变视觉效果，不改变实际弹体爆炸位置。 | `VisualScatter.Min`、`VisualScatter.Max`、`VisualScatter` | 以独立表现事件承载散布结果，避免视觉随机数混入战斗 RNG 或修改命中坐标。 | L · [Visual effect scatter](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#visual-effect-scatter) |
| PH823 武器目标过滤 | 武器可按目标类别、阵营、生命比例与军衔限制攻击，并影响通常的武器选择；攻击空格需显式含 all 或 empty，部分固定武器选择路径例外。 | `CanTarget`、`CanTargetHouses`、`CanTarget.MaxHealth`、`CanTarget.MinHealth`、`CanTargetVeterancy` | 给手动攻击、自动索敌及选武器共享资格查询，明确生命阈值和空格目标语义。 | L/S · [Weapon targeting filter](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#weapon-targeting-filter) |

### 弹头与伤害

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH669 合并建筑范围伤害 | 可将一次范围爆炸对同一建筑各基础格产生的伤害合并后处理；它不同于 CellSpread.MaxAffect，合并取整可能使总伤害略有变化。 | `MergeBuildingDamage` | 由范围伤害宿主按建筑聚合，固定取整与结算顺序，避免插件重复扣血。 | L · [Allow merging AOE damage to buildings into one](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#allow-merging-aoe-damage-to-buildings-into-one) |
| PH670 弹头仅影响施放者 | 可限定弹头只影响施放者，或反向排除施放者，并控制施放者失效时的处理；伤害本身仍受自伤许可约束，仅筛选自己并不自动允许自伤。 | `AffectsInvokerOnly.IgnoreInvokerState`、`AffectsInvokerOnly`、`AffectsInvokerOnly.Reverse` | 给统一受影响目标筛选器增加施放者条件，伤害许可仍交由原伤害所有者判断。 | L · [Allow warhead to only affect invoker](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#allow-warhead-to-only-affect-invoker) |
| PH689 命中解除心灵控制 | 弹头可解除普通心灵控制，但不清除永久控制；解除声效可单独静音，永久控制替换普通控制时也有对应的默认声音策略。 | `RemoveMindControl.Silent`、`MindControl.Permanent.ReplaceSilent`、`RemoveMindControl` | 增加心控解除宿主命令，统一移除控制关系并派发声音事件。 | L/S · [Break Mind Control on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#break-mind-control-on-impact) |
| PH695 圆柱范围弹头与空地过滤 | 范围弹头可改用圆柱影响区，并按高度过滤空中或地面对象；不会覆盖 Ares 的 DamageAirThreshold，也不把 Ares 所有弹头效果纳入新过滤。 | `CellSpread.Cylinder`、`AffectsAir`、`AffectsGround` | 扩展统一范围查询和效果筛选器，分别核对伤害与各类弹头效果消费者。 | L · [CellSpread enhancement](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#cellspread-enhancement) |
| PH696 概率暴击与额外爆炸 | 弹头可按整次爆炸或逐目标掷骰，附加伤害或引爆另一弹头；支持阵营、生命区间、暴击免疫、表现及被拦截后的抑制。 | `VeteranCritChance`、`Crit.ApplyChancePerTarget`、`Crit.ExtraDamage.ApplyFirepowerMult`、`Crit.AnimOnAffectedTargets`、`Crit.SuppressWhenIntercepted`、`Crit.Chance`、`Crit.ExtraDamage` 等（18个不同示例键） | 新增受控随机决策与额外爆炸命令，明确逐次与逐目标 RNG 次序及递归边界。 | L/S · [Chance-based extra damage or Warhead detonation / ‘critical hits’](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#chance-based-extra-damage-or-warhead-detonation-critical-hits) |
| PH697 命中改变目标归属 | 弹头可把目标改归弹头所属方，也可按永久心灵控制处理并尊重心控免疫；可指定控制环动画，不能当作普通伤害附加值处理。 | `ChangeOwner`、`ChangeOwner.SetAsMindControl`、`ChangeOwner.MindControlAnim` | 由归属或心控宿主执行变更并维护关联列表，插件仅选择变更策略。 | L · [Change target Owner on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#change-target-owner-on-impact) |
| PH700 弹头命中类型转换 | 弹头按编号转换规则，把受影响的步兵、车辆或飞机转为同类别新类型，并按所属阵营过滤；不支持建筑，上游有 Ares 3.0 依赖。 | `Convert.From`、`Convert.To`、`Convert.AffectsHouse`、`ConvertN.From`、`ConvertN.To`、`ConvertN.AffectsHouse` | 在弹头效果阶段调用同一转换宿主，处理目标失效、状态继承与重复效果顺序。 | L/S · [Convert TechnoType on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#convert-technotype-on-impact) |
| PH704 自定义心灵控制环动画 | 弹头可指定 MindControl.Anim，用于替换心灵控制成功后使用的控制动画，而不必所有心控效果共用同一表现。 | `MindControl.Anim` | 从心控宿主派发带动画类型的表现事件，控制关系本身不由动画持有。 | L/S · [Custom Mind Control Animation](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-mind-control-animation) |
| PH706 自定义水花动画 | 弹头可指定独立的 SplashList，并选择从水花列表随机抽取动画，从而分别表现不同武器的水面爆炸。 | `SplashList`、`SplashList.PickRandom` | 复用爆炸表现宿主，随机抽取使用确定性事件结果并固定调色板与落点。 | L/S · [Custom ‘SplashList’ on Warheads](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-splashlist-on-warheads) |
| PH720 弹头禁止乘员逃生 | 弹头可分别禁止摧毁目标后生成驾驶乘员、运输乘客或驻军步兵幸存者，使不同击杀方式具有独立逃生规则。 | `PreventCrewEscape`、`PreventPassengerEscape`、`PreventOccupantEscape` | 在死亡清理宿主读取统一逃生策略，防止清理前后重复生成幸存者。 | L · [Customize whether warhead can prevent crew escape from techno](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customize-whether-warhead-can-prevent-crew-escape-from-techno) |
| PH723 按阵营与生命计算伤害倍率 | 弹头可分别设置己方、盟友和敌人的伤害倍率，并对狂乱状态及禁伤敌人场景覆盖；还可按发射者与目标生命比例追加倍率，不影响无视防御的直接伤害。 | `DamageOwnerMultiplier`、`DamageAlliesMultiplier`、`DamageEnemiesMultiplier`、`DamageOwnerMultiplier.NotAffectsEnemies`、`DamageAlliesMultiplier.NotAffectsEnemies`、`DamageOwnerMultiplier.Berzerk`、`DamageAlliesMultiplier.Berzerk` 等（10个不同示例键） | 在统一伤害数值核中组合倍率，明确关系分类、求值顺序和绕过防御路径。 | L · [Damage multipliers](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#damage-multipliers) |
| PH724 伤害地下单位及爆炸显示 | AffectsUnderground 允许弹头伤害地下单位；选中地下目标另需弹体 AU，可禁止地下爆炸动画或把动画显示到正上方地表。 | `AffectsUnderground`、`PlayAnimUnderground`、`PlayAnimAboveSurface` | 区分地下伤害资格与表现落点，统一伤害宿主和渲染事件的坐标含义。 | L · [Damage technos underground](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#damage-technos-underground) |
| PH732 命中引爆伊文炸弹 | 弹头可引爆已安放的伊文炸弹，并按同一施放者、炸弹来源类型及运输、驻军、寄生状态过滤；容器内目标的炸弹通常要等离开容器才爆炸。 | `IvanBomb.Detonate`、`IvanBomb.Detonate.SameInvokerOnly`、`IvanBomb.Detonate.PenetratesTransport`、`IvanBomb.Detonate.PenetratesGarrison`、`IvanBomb.Detonate.AffectsParasite`、`IvanBomb.Detonate.AffectTypes` | 新增炸弹标记命令与容器退出事件，宿主管理延迟引爆和施放者有效性。 | L · [Detonate ivan bomb on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#detonate-ivan-bomb-on-impact) |
| PH733 对全图对象逐个引爆 | 由合适弹体引爆的弹头可遍历全图对象，按类别、阵营、类型及 Verses 过滤逐个结算；默认过滤为空，需显式开放，并有明显性能成本。 | `DetonateOnAllMapObjects`、`DetonateOnAllMapObjects.Full`、`DetonateOnAllMapObjects.AffectsTarget`、`DetonateOnAllMapObjects.AffectsHouse`、`DetonateOnAllMapObjects.AffectTypes`、`DetonateOnAllMapObjects.IgnoreTypes`、`DetonateOnAllMapObjects.RequireVerses` | 提供受控全局查询与批量爆炸事务，固定遍历次序、递归边界和过滤成本。 | L/S · [Detonate Warhead on all objects on map](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#detonate-warhead-on-all-objects-on-map) |
| PH747 弹头击杀触发武器 | 弹头杀死单位后可在受害者或发射者处触发武器，并按目标类型、阵营及抑制配置过滤；没有单位发射者时，发射者处的武器不会触发。 | `KillWeapon`、`KillWeapon.OnFirer`、`KillWeapon.AffectsHouse`、`KillWeapon.OnFirer.AffectsHouse`、`KillWeapon.AffectsTarget`、`KillWeapon.OnFirer.AffectsTarget`、`SuppressKillWeapons` 等（8个不同示例键） | 在死亡归因确定后排队附属武器，宿主负责无来源、连锁击杀及目标失效边界。 | L · [Fire weapon when Warhead kills something](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#fire-weapon-when-warhead-kills-something) |
| PH750 弹头命中金钱交易 | 弹头命中时可给自身拥有者增加或扣除资金，并在目标或发射者处显示金额，指定可见阵营与显示偏移；发射者不存在时提示回到目标处。 | `TransactMoney`、`TransactMoney.Display`、`TransactMoney.Display.AtFirer`、`TransactMoney.Display.Houses`、`TransactMoney.Display.Offset` | 经济变更由阵营资金宿主执行，显示请求与交易结果分离。 | L/S · [Generate credits on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#generate-credits-on-impact) |
| PH754 穿透铁幕与力场伤害 | 弹头可分别穿透铁幕或 Force Shield 的无敌保护，但不等于放开所有弹头效果；负伤害默认可穿透，显式或继承的 false 也会阻止对应负伤害。 | `PenetratesIronCurtain`、`PenetratesForceShield` | 接入无敌过滤的伤害阶段，明确正负伤害与附属效果边界，勿与 HP 护盾混淆。 | L/S · [Iron Curtain & Force Shield damage penetration](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#iron-curtain-force-shield-damage-penetration) |
| PH759 弹头命中发射超武 | 弹头爆炸时可在爆炸格发射列出的超武，并控制是否真实拥有、是否忽略抑制器和指示器及资金显示；负资金消耗仍需足额，部分超武类型不适合此路径。 | `LaunchSW`、`LaunchSW.RealLaunch`、`LaunchSW.IgnoreInhibitors`、`LaunchSW.IgnoreDesignators`、`LaunchSW.DisplayMoney`、`LaunchSW.DisplayMoney.Houses`、`LaunchSW.DisplayMoney.Offset` | 通过超武发射宿主提交请求，统一校验资金、归属和递归调用，勿直接改就绪状态。 | L/S · [Launch superweapons on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#launch-superweapons-on-impact) |
| PH764 弹头修改弹药 | 弹头命中时可为受影响目标增加弹药，负值代表扣除；最终不会超过目标类型 Ammo 上限，也不会降至零以下。 | `Ammo` | 新增受控弹药变更命令，由原弹药宿主夹紧并触发相关事件，不能由 guest 直接写。 | L · [Modify ammo on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#modify-ammo-on-impact) |
| PH774 弹头移除寄生物 | 可关闭原版负伤害驱除寄生物的行为，或让任意弹头执行驱除，并按寄生物类型设置白黑名单；原版负伤害判断发生在 Verses 计算前。 | `RemoveParasite`、`RemoveParasite.Allow`、`RemoveParasite.Disallow` | 在伤害阶段提供寄生清理决策，由关系所有者移除并保证原阶段顺序。 | L/S · [Parasite removal](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#parasite-removal) |
| PH775 穿透运输工具伤害乘客 | 弹头按穿透等级比较后，可概率伤及随机乘客或所有乘客，独立设置伤害倍率与即死概率；运输工具可提供抵抗倍率，清空乘客还可播声音。 | `PenetratesTransport.Level`、`PenetratesTransport.PassThrough`、`PenetratesTransport.FatalRate`、`PenetratesTransport.DamageMultiplier`、`PenetratesTransport.DamageAll`、`PenetratesTransport.CleanSound`、`PenetratesTransport.PassThroughMultiplier` 等（8个不同示例键） | 由运输与伤害宿主协调受控乘客遍历，明确随机次序、死亡清理和递归伤害。 | L · [Penetrates damage on transporter](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#penetrates-damage-on-transporter) |
| PH791 弹头移除伪装 | 弹头可去除间谍或幻影坦克等对象的伪装，即使伪装来自 PermaDisguise 默认永久伪装也适用，而不是只临时让敌人看破。 | `RemoveDisguise` | 通过伪装宿主提交移除命令，统一外观、目标资格和速度派生状态刷新。 | L/S · [Remove disguise on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#remove-disguise-on-impact) |
| PH793 返回弹头效果 | 受原弹头影响的目标可作为返回弹头的归属来源，对原施放者引爆另一弹头，并可设伤害、概率和筛选；互相返回的弹头链可能形成无限递归。 | `ReturnWarhead.ApplyChancePerTarget`、`ReturnWarhead`、`ReturnWarhead.Damage`、`ReturnWarhead.Chance`、`ReturnWarhead.FullDetonation`、`ReturnWarhead.AffectsTarget`、`ReturnWarhead.AffectsHouse` | 在爆炸宿主排队反向效果，显式记录归属与来源并防止循环及无界连锁。 | L · [Return warhead](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#return-warhead) |
| PH795 命中为己方揭示地图 | 弹头命中后可为弹头拥有者揭示指定区域或全图，其他阵营不共享该效果；负值表示整张地图，而不是无效或反向范围。 | `Reveal` | 给视野宿主增加阵营限定的揭图命令，区分永久探索和即时可见性。 | L/S · [Reveal map for owner on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#reveal-map-for-owner-on-impact) |
| PH797 弹头逆向工程 | 弹头可利用 Ares 的逆向工程机制获得受害者对应技术；上游明确要求 Ares 3.0 或更高版本，不能仅凭新增 INI 解析宣称兼容。 | `ReverseEngineer` | 先建立技术解锁与逆向工程宿主，再从弹头效果中调用受控解锁请求。 | L · [Reverse engineer warhead](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#reverse-engineer-warhead) |
| PH799 弹头使建筑出售或收起 | 弹头可使有建造图像的建筑出售，或使配置 UndeploysInto 的建筑收起；出售优先，可放宽不可出售条件，收起后可按附近威胁选择移动位置。 | `BuildingSell`、`BuildingSell.IgnoreUnsellable`、`BuildingUndeploy`、`BuildingUndeploy.Leave` | 调用既有出售与展开宿主事务，统一前提验证、收益、乘员处理及移动命令。 | L · [Sell or undeploy building on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#sell-or-undeploy-building-on-impact) |
| PH802 命中遮蔽敌方地图 | 弹头可对其拥有者的敌人重新遮蔽一定区域或整张地图，不影响己方和盟友；CreateGap 为负值表示作用于全图。 | `CreateGap` | 通过视野宿主提交敌方遮蔽请求，明确探索记录与持续迷雾的不同状态。 | L/S · [Shroud map for enemies on impact](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#shroud-map-for-enemies-on-impact) |
| PH805 弹头生成工具箱 | 弹头可在命中格生成工具箱，被其他对象占据时尝试附近格；可按从零编号的类型与权重配置多种候选，并使用首项别名。 | `SpawnsCrate(N).Type`、`SpawnsCrate(N).Weight` | 新增工具箱生成宿主命令，统一权重随机、合法落点与既有地图工具箱逻辑。 | L/S · [Spawn powerup crate](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#spawn-powerup-crate) |
| PH810 嘲讽弹头 | 弹头可覆盖目标当前任务，迫使其按报复行为攻击伤害来源；没有有效来源单位时不生效，因此不是无来源的任意目标命令。 | `Taunt` | 在受伤后通过任务宿主提交报复意图，校验来源句柄与任务覆盖优先级。 | L · [Taunt warhead](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#taunt-warhead) |
| PH812 逐目标弹头效果执行时点 | 可把 Phobos 的逐目标效果从弹体爆炸阶段改到承受伤害阶段，让粒子等无弹体伤害也能触发；伤害被完全阻止时效果也受阻，多格建筑可能重复触发。 | `ApplyPerTargetEffectsOnDetonate` | 明确伤害前后事件和效果阶段，不同来源共用一次主链，并与建筑合并伤害配套。 | L · [Toggle per-target warhead effects apply timing](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#toggle-per-target-warhead-effects-apply-timing) |
| PH813 指定非人类死亡动作 | 弹头可为 NotHuman 步兵指定 Die1 至 Die5 中的死亡序列，使不同击杀弹头采用不同死亡动作，而不完全取决于随机或固定默认。 | `NotHuman.DeathSequence` | 在死亡动作选择宿主增加弹头策略，资源层校验序列并保留其他死亡优先级。 | L/S · [Trigger specific NotHuman infantry Death anim sequence](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#trigger-specific-nothuman-infantry-death-anim-sequence) |
| PH816 LimboLaunch 发射者重新出现 | 使用 LimboLaunch 武器消失的单位可在弹体爆炸后重新出现，选择强制爆点或附近合法格，并保留目标和选中状态；不能与寄生机制一起使用。 | `UnlimboDetonate`、`UnlimboDetonate.ForceLocation`、`UnlimboDetonate.KeepTarget`、`UnlimboDetonate.KeepSelected` | 由单位生命周期与弹体宿主协调复现事务，区分模拟目标和本地选中状态。 | L · [Unlimbo detonate warhead](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#unlimbo-detonate-warhead) |
| PH820 不致死弹头 | 弹头可造成伤害但不把目标生命降至一以下，适合削弱或训练类效果；它限制对应伤害的致死结果，不代表给予目标全局无敌。 | `CanKill` | 在伤害宿主最终生命提交前应用不致死约束，并核对护盾与负伤害顺序。 | L · [Warhead that can not kill](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#warhead-that-can-not-kill) |

### 车辆

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH671 两栖载具水上上下客 | 两栖步兵或车辆乘客可按配置在水面进入或离开两栖运输车辆，从而不必一律返回陆地再进行装卸。 | `AmphibiousEnter`、`AmphibiousUnload` | 扩展运输上下客的地形与落点检查，插件返回许可，宿主完成占格和移动。 | L · [Amphibious access vehicle](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#amphibious-access-vehicle) |
| PH683 按弹药自动展开与限制展开 | 可按弹药上下限自动触发车辆展开，或限制是否允许展开；适用于转换建筑、展开开火、简单展开及载客车辆，负值可禁用对应边界检查。 | `Ammo.AutoDeployMinimumAmount`、`Ammo.AutoDeployMaximumAmount`、`Ammo.DeployUnlockMinimumAmount`、`Ammo.DeployUnlockMaximumAmount` | 为弹药提交和展开资格开放独立钩子，宿主执行展开并防止重复触发。 | L · [Automatic deploy and blocking deploying based on ammo](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#automatic-deploy-and-blocking-deploying-based-on-ammo) |
| PH703 悬浮载具断电落水 | 可自定义悬浮载具在水域失去活动能力后是否溺亡，避免只能接受统一的停机水面死亡规则。 | `HoverDrownable` | 在悬浮运动器停机和地形变化点提供策略，死亡仍由宿主统一提交。 | L · [Custom hover vehicles shutdown drowning death](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-hover-vehicles-shutdown-drowning-death) |
| PH726 车辆受损图像 | 车辆可按黄血、红血阈值更换陆地或水面图像；引用类型须注册，原始与受损外观需保持同为 SHP 或同为 VXL。 | `Image.ConditionYellow`、`Image.ConditionRed`、`WaterImage.ConditionYellow`、`WaterImage.ConditionRed` | 由资源与渲染宿主完成状态选图，加载时校验类型与素材格式兼容性。 | L · [Damaged unit image changes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#damaged-unit-image-changes) |
| PH728 逐车辆默认幻影伪装 | 车辆类型可分别覆盖 DefaultMirageDisguises，指定自己的默认幻影伪装候选，而不必所有幻影车辆共用同一套配置。 | `DefaultMirageDisguises` | 通过伪装宿主读取类型级候选表，选择时统一使用宿主随机数和状态。 | L · [Default mirage disguise for individual VehicleTypes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#default-mirage-disguise-for-individual-vehicletypes) |
| PH751 SHP 车辆独立炮塔素材 | SHP 炮塔车辆可加载独立的 *tur.shp；存在时炮塔从该文件首帧读取，否则仍从车体文件中按行走帧与朝向数量推算起点。 | 无独立示例键；见原文条件 | 由素材解析与车辆渲染宿主支持资源回退，不需给模拟插件暴露文件访问。 | L · [Independent SHP Vehicle Turret Files](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#independent-shp-vehicle-turret-files) |
| PH755 Jumpjet 移动倾斜 | Jumpjet 单位可在前进与转向时产生俯仰和侧倾，幅度由速度、加速度及转向系数决定，最大倾角不超过九十度。 | `JumpjetTilt`、`JumpjetTilt.ForwardAccelFactor`、`JumpjetTilt.ForwardSpeedFactor`、`JumpjetTilt.SidewaysRotationFactor`、`JumpjetTilt.SidewaysSpeedFactor` | 放在运动结果到渲染姿态的转换层，插件给系数，不让视觉倾斜改变模拟坐标。 | L · [Jumpjet Tilts While Moving](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#jumpjet-tilts-while-moving) |
| PH783 红警一式多炮塔多炮管 | 普通 VXL 车辆可在多个偏移位置绘制并轮流使用同款炮塔与炮管，发射位置由 Burst 序号决定；不提供独立瞄准、武器、冷却或生命的多炮塔。 | `BarrelOverTurret`、`BarrelOffset`、`ExtraBarrelCount`、`ExtraBarrelOffsetX`、`ExtraTurretCount`、`ExtraTurretOffsetX`、`BurstPerTurret` | 扩展 FLH 与渲染姿态宿主，复用同一连发状态，避免误做成多个独立战斗实体。 | L · [RA1-Style Multi-Turret and Multi-Barrel](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#ra1-style-multi-turret-and-multi-barrel) |
| PH814 丢失目标后的炮塔归位 | 车辆丢失目标后可选择炮塔是否转回车体朝向；静止车辆或因移动限制无法在当前格移动者默认不归位，其他通常默认归位。 | `TurretResponse` | 在炮塔朝向宿主接入丢失目标策略，使用当前运动能力而非单看基础速度。 | L · [Turret Response](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#turret-response) |
| PH815 无炮塔 SHP 车辆预开火帧 | 无炮塔且非 VXL 的车辆可使用 FireUp 延迟到指定动画帧开火，并选择换目标时是否重置计数；完全失去目标时计数强制重置。 | `FireUp`、`FireUp.ResetInRetarget` | 让动画与武器时序宿主共享预开火进度，保存状态并明确中断与重瞄准行为。 | L · [Turretless Shape Vehicle FireUp](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#turretless-shape-vehicle-fireup) |

### 动画

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH672 动画结束生成单位 | 动画结束后可生成车辆、飞机或步兵，并控制归属、任务、朝向、落地、降落伞与寻路选点；部分步兵死亡动画的归属仍有上游限制。 | `CreateUnit`、`CreateUnit.Owner`、`CreateUnit.RequireOwner`、`CreateUnit.RemapAnim`、`CreateUnit.Mission`、`CreateUnit.AIMission`、`CreateUnit.Facing` 等（15个不同示例键） | 新增动画完成事件与受控生成命令，宿主负责归属、合法落点、注册和随机朝向。 | L/S · [Anim-to-Unit](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#anim-to-unit) |
| PH675 动画附带烟雾粒子系统 | 动画可附带 BehavesLike=Smoke 的粒子系统；播放 Next 动画时会清理旧系统，并在后续动画定义了系统时创建新实例。 | `AttachedSystem` | 将粒子绑定与清理放入动画表现宿主，以动画生命周期事件驱动。 | L/S · [Attached particle system](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attached-particle-system) |
| PH708 动画阵营可见性 | 可按归属方或施放者阵营控制动画可见性，并处理隐形时的隐藏与附着保留；这些只影响显示，动画附带的伤害仍由所有玩家处理。 | `VisibleTo`、`VisibleTo.ConsiderInvokerAsOwner`、`RestrictVisibilityIfCloaked`、`DetachOnCloak` | 将可见性放在渲染宿主本地过滤，禁止因本地不可见而跳过确定性模拟。 | L/S · [Customizable animation visibility settings](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#customizable-animation-visibility-settings) |
| PH776 动画使用分离声音事件 | 动画可用 DetachedReport 播放不绑定音频事件处理器的声音，减少 Next 动画切换导致处理器重置的问题；也失去绑定声音随位置和循环的通常行为。 | `DetachedReport` | 作为动画到音频的表现策略实现，明确声音是否跟随、循环及何时停止。 | L/S · [Play sound as a detached sound event](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#play-sound-as-a-detached-sound-event) |

### 新类型与公共基础

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH674 类型化附加效果 | 定义独立 AttachEffect 类型，支持持续时间、叠加刷新、条件移除、类型转换转移、动画、属性倍率、反伤和武器禁用等；与 Ares 同名机制的结构及细节不同。 | `AttachEffect.AttachTypes`、`Duration`、`SpeedMultiplier`、`ArmorMultiplier`、`FirepowerMultiplier`、`Cumulative`、`DiscardOn` 等（123个不同示例键） | 先新增效果容器与确定性调度宿主；按属性、伤害和表现分批开放能力，避免大而全操作接口。 | L/S · [Attached Effects](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attached-effects) |
| PH705 自定义辐射类型 | 武器可使用独立辐射类型，并控制建筑受伤频率、基础格命中次数、弹头完整爆炸和伤害归属；独立更新频率与建筑辐射会增加性能成本。 | `UseGlobalRadApplicationDelay`、`RadType`、`RadDurationMultiple`、`RadApplicationDelay`、`RadApplicationDelay.Building`、`RadBuildingDamageMaxCount`、`RadLevelMax` 等（18个不同示例键） | 先扩展辐射场与范围伤害宿主，插件定义参数；统一管理周期、归属和随机效果。 | L/S · [Custom Radiation Types](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-radiation-types) |
| PH758 激光尾迹类型 | 单位、弹体与 VoxelAnim 可绑定多条独立尾迹，配置偏移、颜色、透明度、粗细及激光、电弧、辐射束样式；原注入引擎的大量尾迹有性能压力。 | `DrawType`、`FadeDuration`、`SegmentLength`、`IgnoreVertical`、`CloakVisible`、`CloakVisible.DetectedOnly`、`Color` 等（25个不同示例键） | 由表现宿主管理采样、片段缓存和资源预算，避免在玩法状态内存放绘图实例。 | L/S · [Laser Trails](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#laser-trails) |
| PH801 独立生命护盾体系 | 单位拥有独立护甲与生命池的护盾，可吸伤、漏伤、破盾、重生、自愈及按电力暂停，并支持弹头挂卸替换、类型转换继承和专用表现；并非 Ares 的 ForceShield。 | `ShieldType`、`Strength`、`Armor`、`AbsorbPercent`、`PassPercent`、`SelfHealing`、`Respawn` 等（103个不同示例键） | 现有外置样例只覆盖有限吸伤与计时链；挂卸、转换、动画和武器效果仍需新增宿主能力。 | L/S · [Shields](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#shields) |

### 抛射体

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH680 攻击地下单位 | 弹体可通过 AU 接受地下单位为目标，但实际伤害仍需弹头允许；只有适当的隐形或指定轨迹弹体能入地，横向潜地目标还需探测能力。 | `AU` | 同时扩展目标可见性、地下探测和弹体碰撞，不能只新增一个布尔 INI 读取。 | L · [Attack technos underground](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attack-technos-underground) |
| PH688 Bombard 转折轰击轨迹 | 弹体先飞向由高度和距离比例确定的转折点，再转向目标或从目标上方自由落体；可配置散布、提前引爆、目标吸附、预判、偏移和入地行为。 | `Trajectory`、`Trajectory.Bombard.Height`、`Trajectory.Bombard.FallPercent`、`Trajectory.Bombard.FallPercentShift`、`Trajectory.Bombard.FallScatter.Max`、`Trajectory.Bombard.FallScatter.Min`、`Trajectory.Bombard.FallScatter.Linear` 等（22个不同示例键） | 以弹体轨迹策略接入固定步进宿主，统一坐标、碰撞、随机散布及存档状态。 | L/S · [Bombard trajectory](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#bombard-trajectory) |
| PH772 Parabola 抛物线轨迹 | 提供按速度、高度或角度等六种组合求解的抛物线，支持预判、提前引爆、目标吸附、地面或水面弹跳与衰减；部分角度组合无解或代价较高。 | `Trajectory`、`Trajectory.Parabola.DetonationDistance`、`Trajectory.Parabola.TargetSnapDistance`、`Trajectory.Parabola.OpenFireMode`、`Trajectory.Parabola.ThrowHeight`、`Trajectory.Parabola.LaunchAngle`、`Trajectory.Parabola.LeadTimeCalculate` 等（19个不同示例键） | 建立固定步进弹体轨迹接口，统一求解失败回退、碰撞、弹跳状态和跨平台数值规则。 | L · [Parabola trajectory](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#parabola-trajectory) |
| PH773 降落伞炸弹 | 空中单位发射的弹体可带伞下落，设置下降速度、最大速度与伞动画；不宜同时混用旋转追踪或竖直等其他弹体运动逻辑。 | `Parachuted`、`Parachuted.FallRate`、`Parachuted.MaxFallRate`、`BombParachute` | 给弹体运动宿主增加降落模式，降落伞作为绑定表现，不另建第二套位置状态。 | L · [Parabombs](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#parabombs) |
| PH778 弹体拦截 | 单位可定期扫描可拦截弹体并用指定武器攻击，按护甲和强度结算；还可删除、保留或改写被拦截弹体，扫描间隔与射程影响性能和实际拦截能力。 | `ProjectileInterceptable`、`Interceptor.GuardRange.IsCylindrical`、`Interceptor.ApplyFirepowerMult`、`Interceptor`、`Interceptor.Weapon`、`Interceptor.TargetingDelay`、`Interceptor.CanTargetHouses` 等（23个不同示例键） | 新增只读弹体查询与受控拦截命令，弹体生命、改写和爆炸仍由同一宿主持有。 | L/S · [Projectile interception logic](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#projectile-interception-logic) |
| PH779 可配置弹体轨迹总则 | Trajectory 使用独立运动规则，不应混用 Arcing、ROT、Vertical 或 Inviso；Trajectory.Speed 定义在弹体上，各轨迹对该速度的含义不同。 | `Trajectory.Speed` | 建立单一轨迹选择与校验入口，禁止两个运动器同时推进同一弹体。 | L/S · [Projectile trajectories](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#projectile-trajectories) |
| PH780 陆地水面与地形阻挡弹体 | 弹体可将陆地或水面视为阻碍，影响攻击站位并可在碰撞时引爆；地表高度预测有飞机、建筑及敞篷乘客等不适用路径。 | `SubjectToLand`、`SubjectToLand.Detonate`、`SubjectToWater`、`SubjectToWater.Detonate`、`SubjectToGround` | 让射线或路径资格与实际碰撞共享宿主几何查询，明确各发射者支持范围。 | L/S · [Projectiles blocked by land or water](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#projectiles-blocked-by-land-or-water) |
| PH794 弹体爆炸后返回武器 | 弹体爆炸后可向原发射者发射另一武器与弹体；若原发射者已经死亡则不触发，并可选择是否应用发射者火力倍率。 | `ReturnWeapon.ApplyFirepowerMult`、`ReturnWeapon` | 由弹体宿主生成受控子弹请求，发射者句柄失效时跳过，统一倍率计算。 | L/S · [Return weapon](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#return-weapon) |
| PH808 Straight 直线弹道 | 直线弹体可穿透目标，按路程或近炸触发额外弹头，并配置目标吸附、预判、障碍、贴地高度及距离或次数衰减；轨迹引爆并不自动提供 AI 合法站位搜索。 | `Trajectory`、`Trajectory.Straight.ApplyRangeModifiers`、`Trajectory.Straight.DetonationDistance`、`Trajectory.Straight.TargetSnapDistance`、`Trajectory.Straight.PassThrough`、`Trajectory.Straight.PassDetonate`、`Trajectory.Straight.PassDetonateWarhead` 等（31个不同示例键） | 由弹体宿主统一步进、碰撞与近炸，插件返回轨迹参数，严格定义命中顺序与数值单位。 | L/S · [Straight trajectory](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#straight-trajectory) |

### 飞机

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH702 自定义巡航导弹 | 自定义导弹可启用巡航模式，并可调节上升段连续产生起飞烟迹的间隔；后者也能用于原版指定巡航导弹，不必完全改写成自定义导弹。 | `Missile.Cruise`、`Missile.TakeOffSeparation` | 扩展导弹运动与尾迹宿主策略，保持起飞、巡航和爆炸生命周期一致。 | L · [Custom cruise missiles](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#custom-cruise-missiles) |
| PH725 飞机受损图像 | 飞机生命低于全局黄血或红血阈值时可换用相应图像类型；引用的飞机类型仍须列入 AircraftTypes 注册表。 | `Image.ConditionYellow`、`Image.ConditionRed` | 让美术资源宿主按模拟生命状态选图，预加载合法引用而不转换单位类型。 | L · [Damaged aircraft image changes](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#damaged-aircraft-image-changes) |

### 地形对象

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH730 地形物体摧毁表现 | TerrainType 被摧毁时可指定动画与声音；动画列表有多项时会随机选择，用于区分树木等地形物体的死亡效果。 | `DestroyAnim`、`DestroySound` | 由地形死亡宿主派发表现事件，随机动画选择与清理时点保持确定。 | L/S · [Destroy animation & sound](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#destroy-animation-sound) |

### 通用说明

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH821 新弹头效果通用约束 | 新弹头效果一般可结合 CellSpread 及适用的 GenericWarhead，按 Verses 和护盾护甲判断目标；Ares 永久心控等路径存在不兼容，不能把所有效果默认视作通用。 | 无独立示例键；见原文条件 | 在统一弹头效果分发层声明可用入口与筛选策略，逐项验证效果消费者。 | L/S · [Warheads](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#warheads) |

<a id="enhanced"></a>
## 既有逻辑配置与改进

本节195个具名主题。

本页既含原版改进，也含新的可配置能力；逐项无标题修复另列后文。

### 全局配置

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH077 按类别指定 KeepAlive 默认值 | 为建筑、防御、步兵、车辆和飞机分别设置 KeepAlive 的全局默认，方便配合 Ares 的存活判定触发事件，减少逐个单位配置。 | `KeepAlive.Buildings`、`KeepAlive.Defenses`、`KeepAlive.Infantry`、`KeepAlive.Units`、`KeepAlive.Aircraft` | 由存活统计的引擎机制消费类别默认，guest 提供配置政策。 | L · [Add a global default value for KeepAlive](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#add-a-global-default-value-for-keepalive) |
| PH083 单人模式信标快捷键 | 可在单人战役和遭遇战启用放置信标快捷键，解除原版仅多人模式可用的限制，默认仍保持原版行为。 | `AllowBeaconHotKeyInSinglePlayer` | 由客户端命令层读取策略，再通过既有信标命令处理。 | L · [Allow beacon placement hotkey in single player](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-beacon-placement-hotkey-in-single-player) |
| PH084 超时空冻结时的建筑动画 | 控制 Temporal 弹头作用于建筑时，是否像原版一样令其用电动画停转；该开关独立于超时空清除本身。 | `Temporal.KillPoweredAnim` | 在 Temporal 状态到呈现状态的投影处接入配置。 | L · [Allow customize that whether Temporal=yes warhead will cause target building animation poweroff](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-customize-that-whether-temporal-yes-warhead-will-cause-target-building-animation-poweroff) |
| PH085 允许受控基地车展开 | 可允许被心灵控制的车辆展开成 ConstructionYard 建筑，改变原版对这种展开操作的限制。 | `AllowDeployControlledMCV` | 扩展展开资格判定，并保持控制链与所有权清理完整。 | L · [Allow deploy controlled MCV](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-deploy-controlled-mcv) |
| PH103 信标超额时淘汰最早项 | 原版每人三枚信标满额后拒绝新增；启用后自动移除最早的信标给新信标腾位，不改变信标所属方。 | `AutoRemoveEarliestBeacon` | 在信标集合 owner 中应用有序替换政策。 | L · [Auto-remove earliest beacon](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#auto-remove-earliest-beacon) |
| PH104 允许狂暴影响盟军 | 可解除 Psychedelic 对盟军的硬编码排除；具体弹头仍遵守 AffectsAllies 等影响方规则，不是无条件作用所有友军。 | `AllowBerzerkOnAllies` | 合并到弹头受影响方判定，避免与普通伤害筛选分离。 | L · [Berzerk on allies](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#berzerk-on-allies) |
| PH426 超时空闪烁动画位置 | 可配置闪烁实例间隔，并在建筑中心、当前驻军位置或全部驻军槽位显示，支持超过十个枪口且有无驻军后备规则。 | `ChronoSparkleDisplayDelay`、`ChronoSparkleBuildingDisplayPositions` | 模拟只暴露冻结状态，呈现层计算实例位置与节奏。 | L/S · [Chrono sparkle animation customization & improvements](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#chrono-sparkle-animation-customization-improvements) |
| PH429 工具箱生成与奖励改进 | 改进随机分布，支持仅陆地生成、车辆奖励数量上限与免费基地车资金门槛，并避免满血单位因治疗乱动；陆地限制实际读取 CrateOnlyOnLand。 | `CrateOnlyOnLand`、`UnitCrateVehicleCap`、`HealBaseNoScatter`、`FreeMCV`、`FreeMCV.CreditsThreshold` | 扩展工具箱奖励政策，生成与奖励随机均需保持同步次序。 | L/S · [Crate improvements](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#crate-improvements) |
| PH455 在 art 中替换单位图像 | 启用 ArtImageSwap 后 art 的 Image 可替换各类单位素材文件名，但不会继承同名 art 节；原版某些占位 Image 需处理兼容。 | `ArtImageSwap`、`Image` | 资源解析层区分素材别名与节继承，按支持的 SHP/VXL 类型校验。 | L/S · [Customizable unit image in art](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-unit-image-in-art) |
| PH471 资源储存矿物类别 | 为 Ares 的建筑 Storage 机制指定所用矿物列表索引，配合 Refinery.UseStorage；官方原扩展用法明确依赖 Ares。 | `Storage.TiberiumIndex` | 先提供完整储存与经济事务机制，再由 guest 选择矿物类别。 | L/S · [Customize resource storage](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-resource-storage) |
| PH473 围墙连锁伤害 | 可覆盖围墙受损时对相邻四格围墙传递的伤害值，调整连续围墙比单块围墙更易受损的程度。 | `AdjacentWallDamage` | 围墙伤害机制消费策略，防止重复递归造成多次结算。 | L · [Customize the chained damage of the wall](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-chained-damage-of-the-wall) |
| PH475 补充原版与 Ares 全局默认 | 为默认区域警戒、防散开、心控连线和动画绘制等原本只有局部设置的能力增加全局默认；局部键语义仍沿用原机制。 | `DefaultToGuardArea`、`Warhead.PreventScatter`、`LeptonMindControlOffset`、`MindControlRingOffset`、`Anim.ShouldUseCellDrawer` | 配置解析层集中实现默认继承，各机制消费冻结结果。 | L · [Customize the global default values of some vanilla/Ares flags](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-global-default-values-of-some-vanilla-ares-flags) |
| PH483 受控非重要单位的自动索敌 | 可控制被心灵控制的 Insignificant 非建筑单位是否进入自动目标候选，选择 RA2 或尤复的差异行为。 | `AutoTarget.InsignificantWhenMindControlled` | 在统一索敌过滤器中应用规则并覆盖改属与控制释放。 | L · [Customize whether mind-controlled Insignificant technos can be auto-targeted](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-mind-controlled-insignificant-technos-can-be-auto-targeted) |
| PH490 空中对象高度光照 | 可分别设置飞机与 Jumpjet 步兵、车辆在高于所在格时的高度光照倍率，使两类空中对象不必共享相同亮度变化。 | `AircraftLevelLightMultiplier`、`JumpjetLevelLightMultiplier` | 呈现层按对象移动类别应用高度光照，不写模拟高度。 | L/S · [Customizing effect of level lighting on air units](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-effect-of-level-lighting-on-air-units) |
| PH511 狂暴任务行为 | 可替换单位进入狂暴时自动分配的 Hunt 任务，避免只追远处目标；同时补齐停止和清除目的地处理，修正移动中的 Jumpjet。 | `BerzerkMission` | 狂暴状态 owner 通过统一任务接口提交转换与清理。 | L · [Enhanced berzerk behavior](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#enhanced-berzerk-behavior) |
| PH573 铁幕与力场额外染色强度 | 分别配置铁幕和 Force Shield 状态的额外染色强度；属于外观调整，不改变持续时间、免伤判定或护盾生命。 | `IronCurtain.ExtraTintIntensity`、`ForceShield.ExtraTintIntensity` | 由状态到呈现的投影输出染色样式，模拟只提供无敌状态快照。 | L/S · [Iron Curtain & Force Shield extra tint intensity](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#iron-curtain-force-shield-extra-tint-intensity) |
| PH586 伊文炸弹附着建筑中心 | 可将建筑上的伊文炸弹图像及爆点从地基最左上格移到建筑中心；官方说明不能逐武器单独配置。 | `IvanBombAttachToCenter` | 统一炸弹逻辑爆点与显示锚点，避免只移动图标。 | L/S · [Move IvanBomb Position](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#move-ivanbomb-position) |
| PH597 范围指示圈可见方 | 建筑范围圈可向观察者及指定关系方显示，不再只限当前玩家自己的建筑；这是指示圈外观权限而非实际作用范围。 | `RadialIndicatorVisibility` | 在客户端可见性层过滤范围圈，禁止泄露不可见对象信息。 | L/S · [RadialIndicator visibility](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#radialindicator-visibility) |
| PH598 恢复 Jumpjet 全局控制节 | 重新启用 JumpjetControls 作为各单位 Jumpjet 参数默认，并补充坠毁和摆动设置；部分旧键名称与单位级名称并不完全相同。 | `Crash`、`NoWobbles` | 配置层显式映射旧名称，移动器只消费统一的冻结参数。 | L/S · [Re-enable obsolete [JumpjetControls]](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#re-enable-obsolete-jumpjetcontrols) |
| PH602 区分飞机出厂与野外降落方向 | 为飞机出厂和野外降落分别提供 PoseDir 默认覆盖，统一方向制；原文正文与代码注释对 Field 默认倍数不一致，须核对源码。 | `PoseDir.Production`、`PoseDir.Field` | 配置层保留不同场景，先确认换算再冻结兼容行为。 | L · [Separate the definitions of default direction for aircraft production and landing in the field](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#separate-the-definitions-of-default-direction-for-aircraft-production-and-landing-in-the-field) |
| PH607 遭遇战 AI 卖家与总攻 | 可分别关闭 AI 无工厂后的出售和全军猎杀，设置出售延迟，并禁止展开基地车时召集全军。 | `AIFireSale`、`AIFireSaleDelay`、`AIAllToHunt`、`GatherWhenMCVDeploy` | 扩展 AI 战略政策，计时、出售与任务提交仍由原 owner 执行。 | L/S · [Skirmish AI behavior dehardcode](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#skirmish-ai-behavior-dehardcode) |
| PH618 ColorAdd 使用八位 RGB | 可让 ColorAdd 接受每通道零至二百五十五的 RGB，而非原 RGB565 输入，影响空袭标记、铁幕、狂暴及 Force Shield 染色。 | `ColorAddUse8BitRGB` | 资源配置层转换颜色，保留格式开关避免误读旧配色。 | L/S · [Use 8-bit RGB parameters for [ColorAdd]](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#use-8-bit-rgb-parameters-for-coloradd) |
| PH619 更精确的修理费用 | 可避免先对最大生命除以维修步长向下取整，再用该值作费用除数造成的资金误差；属于修理成本算法选择。 | `FixRepairStepCost` | 修正统一经济数值核，验证舍入、最低收费与扣费顺序。 | L · [Use more precise calculation of repair costs](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#use-more-precise-calculation-of-repair-costs) |
| PH620 恢复藤蔓怪机制 | 恢复藤蔓怪生长、收缩、伤害与免疫，依赖对应地形、动画、弹头和特定覆盖物内部序号；序号由注册顺序决定而非等号左侧编号。 | `VeinholeGrowthRate`、`VeinholeShrinkRate`、`MaxVeinholeGrowth`、`VeinDamage`、`VeinholeTypeClass`、`VeinholeWarhead`、`Veinhole` 等（12个不同示例键） | 需矿物类网格、伤害与对象生命周期机制，不能只增加 INI 读取。 | L/S · [Veinholes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#veinholes) |
| PH622 体素光源方向 | 可调整体素相对光源向量及固定光照开关；只使用光线方向，不使用光源距离，外部预览工具未必匹配固定光照模式。 | `UseFixedVoxelLighting`、`VoxelLightSource` | 渲染层统一归一化和光照坐标系，不引入模拟依赖。 | L/S · [Voxel light source customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#voxel-light-source-customization) |
| PH625 建筑使用路径点 | 可允许建筑接受路径点，解除原版建筑禁用路径点的限制；该开关本身不赋予静止建筑新的移动能力。 | `BuildingWaypoints` | 客户端命令路由与建筑任务消费者需一起核对。 | L · [Waypoints for buildings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#waypoints-for-buildings) |
| PH626 消耗藤蔓资源充能超武 | 超武可使用阵营藤蔓储备充能，并配置所需数量、侧栏储量百分比和建筑就绪动画门槛；不能据此使用已移除的 ChemMissile 类型。 | `UseWeeds`、`UseWeeds.Amount`、`UseWeeds.StorageTimer`、`UseWeeds.ReadinessAnimationPercentage` | 超武与资源账目通过统一事务结算，呈现独立读取充能快照。 | L/S · [Weed-consuming superweapons](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#weed-consuming-superweapons) |
| PH627 藤蔓采集与阵营储量 | Weeder 车辆可采集并向 Weeder 建筑卸载藤蔓；资源存于阵营而非建筑仓库，可用超时空移动器及自定义采集动画。 | 无独立示例键；见原文条件 | 资源 owner 区分阵营储量与载具库存，需采集、卸载完整链。 | L/S · [Weeds & Weed Eaters](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#weeds-weed-eaters) |

### 建筑

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH078 AI 基地建设行为 | 控制 AI 已有建造厂后是否继续展开基地车、更新基地中心，是否偏好出生点建造厂，以及墙节点的连接和清理规则。 | `AIAutoDeployMCV`、`AISetBaseCenter`、`AIBiasSpawnCell`、`AIForbidConYard`、`AINodeWallsOnly`、`AICleanWallNode` | 扩展 AI 建造与基地中心政策接口，基地状态仍由原有 owner 管理。 | L · [AI base construction modification](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#ai-base-construction-modification) |
| PH080 停机位朝向 | 建筑可按停机位分别指定飞机降落方向，覆盖飞机自身设置；支持退回 PoseDir 默认或保留地图预置建筑方向。 | `AircraftDockingDir.DefaultToPoseDir`、`AircraftDockingDir(N)` | 由停靠机制解析停机位配置，渲染消费最终朝向。 | L/S · [Aircraft docking direction](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#aircraft-docking-direction) |
| PH091 矿厂同时播放多组工作动画 | 矿厂可像普通建筑一样同时使用多组 ActiveAnim，不再仅按存储量切换一组；这是建筑动画选择行为的扩展。 | `Refinery.UseNormalActiveAnim` | 由建筑呈现状态选择器消费 art 配置。 | L · [Allows refineries to use multiple ActiveAnim simultaneously](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allows-refineries-to-use-multiple-activeanim-simultaneously) |
| PH097 建造动画应用深度偏移 | 可在建筑建造动画期间启用 ZShapePointMove，修正深度缓冲造成的建筑边角裁切；原版仅在常规显示阶段应用。 | `ZShapePointMove.OnBuildup` | 在建筑渲染深度计算中消费 art 配置，无需模拟写操作。 | L/S · [Apply ZShapePointMove during buildups](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#apply-zshapepointmove-during-buildups) |
| PH100 武装建筑攻击重试间隔 | 建筑已有目标时重新检查能否攻击的等待可按全局或建筑类型配置，接受固定帧数或随机范围，涵盖驻军建筑。 | `BuildingGuardRetryDelay`、`GuardRetryDelay` | 由建筑攻击调度 owner 保存计时，随机范围使用同步随机流。 | L · [Armed building guard retry delay](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#armed-building-guard-retry-delay) |
| PH421 建筑炮塔多帧动画 | SHP 炮塔支持普通、停电、射击及停电射击帧段与帧速；射击帧段不自动阻塞武器开火，需与延迟发射自行配合。 | `TurretAnim.IdleFrames`、`TurretAnim.LowPowerIdleFrames`、`TurretAnim.FiringFrames`、`TurretAnim.LowPowerFiringFrames`、`TurretAnim.IdleRate`、`TurretAnim.FiringRate` | 将射击状态投影给呈现层，不能以动画播放结果驱动模拟。 | L · [Building turret animations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#building-turret-animations) |
| PH422 建筑按车辆处理 | ConsideredVehicle 可独立控制建筑是否计入车辆统计、批量选择及受击告警等行为，不再由可收起与 1x1 地基隐式决定。 | `ConsideredVehicle` | 建立统一对象分类政策并审查所有统计与界面消费者。 | L/S · [Buildings considered as vehicles](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#buildings-considered-as-vehicles) |
| PH432 兵营自定义出口格 | 步兵工厂可直接指定生成步兵的出口格偏移，覆盖原版按三类兵营标志硬编码的格点；偏移属于地图格坐标。 | `BarracksExitCell` | 扩展工厂出厂定位政策，保留占格与出厂路径校验。 | L · [Custom exit cell for infantry factory](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-exit-cell-for-infantry-factory) |
| PH435 建筑改属时同步关联单位 | 可禁止建筑所有者变化时同步改变 RadioLink 相连单位的归属，支持全局与单建筑覆盖，适用于工厂和机场等。 | `BuildingRadioLink.SyncOwner` | 归属事务统一处理建筑、停靠关系与关联单位。 | L · [Custom whether to synchronously change the owner of the RadioLink-linked units when the owner of a building changes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-whether-to-synchronously-change-the-owner-of-the-radiolink-linked-units-when-the-owner-of-a-building-changes) |
| PH442 驻军与坦克碉堡修正 | 建筑可独立覆盖驻军、坦克碉堡的伤害与射速倍率，并指定坦克进入和退出时的声音，未设置时沿用全局参数。 | `OccupyDamageMultiplier`、`OccupyROFMultiplier`、`BunkerDamageMultiplier`、`BunkerROFMultMultiplier`、`BunkerWallsUpSound`、`BunkerWallsDownSound` | 统一武器倍率来源，进出事件交音频层消费。 | L · [Customizable garrison and bunker properties](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-garrison-and-bunker-properties) |
| PH445 回收站资格与奖励武器 | 回收站可限定所有者、盟军及类型白黑名单，调整死亡语音和回收音效；积累回收收入后可按资金门槛与 ROF 触发武器。 | `Grinding.AllowAllies`、`Grinding.AllowOwner`、`Grinding.AllowTypes`、`Grinding.DisallowTypes`、`Grinding.PlayDieSound`、`Grinding.Sound`、`Grinding.Weapon` 等（8个不同示例键） | 需回收事务、收入累计和受控发射接口，不让 guest 直接改钱。 | L/S · [Customizable & new grinder properties](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-new-grinder-properties) |
| PH452 可收起建筑出售帧数 | 为可 Undeploy 的建筑指定出售时倒放建造动画的帧数，替代原版固定倒放末尾二十三帧；不直接改变收起时动画。 | `SellBuildupLength` | 在出售状态的呈现映射中使用独立长度，避免改变收起动画路径。 | L/S · [Customizable selling buildup sequence length for buildings that can undeploy](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-selling-buildup-sequence-length-for-buildings-that-can-undeploy) |
| PH467 克隆中心供电要求 | 可指定克隆中心是否需要供电工作，以选择原版停电仍克隆或 Ares 引入的需要电力行为。 | `Cloning.Powered` | 在克隆资格判定中消费电力状态，生成仍由克隆机制执行。 | L · [Customize if cloning need power](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-if-cloning-need-power) |
| PH470 建筑过充阈值 | 按弹头设置充电等级，并分别设定建筑保持在线和切换武器所需阈值；负值禁用相应能力，不影响 Ares Battery 超武。 | `ElectricAssaultLevel`、`Overpower.KeepOnline`、`Overpower.ChargeWeapon` | 扩展建筑受充电状态政策，勿与护盾过载机制混为一谈。 | L · [Customize overpower logic](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-overpower-logic) |
| PH472 对全体揭示的视野半径 | 建筑 RevealToAll 可配置独立揭示半径，未设置沿用 Sight，不再必须与单位普通视野使用相同半径。 | `RevealToAll.Radius` | 由视野网格 owner 统一增删揭示源。 | L · [Customize reveal radius of RevealToAll](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-reveal-radius-of-revealtoall) |
| PH476 建筑初始朝向 | 建筑可设置固定或随机初始朝向，支持全局默认；该朝向使用二百五十六方向制，与泰伯利亚之日同名键的八方向不同。 | `BuildingStartFacing`、`BuildingStartFacing.Random`、`StartFacing`、`StartFacing.Random` | 由建筑创建 owner 决定朝向，随机方向必须进入同步状态。 | L · [Customize the initial facing of buildings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-initial-facing-of-buildings) |
| PH501 建筑部署开火 | 建筑可使用 DeployFire 和 DeployFireWeapon，并设置重试间隔；工厂、裂缝、载员等部署功能优先，武器仍受冷却和目标资格约束。 | `DeployFireDelay` | 统一部署命令分派，受控发射复用武器机制。 | L · [DeployFire supports buildings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#deployfire-supports-buildings) |
| PH504 建造期间隐藏 AlphaImage | 可全局或按建筑指定建造完成前不显示 AlphaImage，改变建造阶段的附加透明图像表现。 | `NoAlphaImageOnBuildup` | 在建筑呈现状态过滤素材层，不改建造进度模拟。 | L · [Disable AlphaImage during Buildup](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#disable-alphaimage-during-buildup) |
| PH505 关闭建筑受损音效 | 建筑类型可禁止播放 DamageSound，避免特定对象每次受损都播放通用声音；不影响受伤判定本身。 | `DisableDamageSound` | 由音频事件过滤层消费建筑配置，沿用统一的受击音效路由。 | L · [Disable DamageSound](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#disable-damagesound) |
| PH514 工厂排除多工厂加成 | 可将指定工厂建筑排除在 MultipleFactory 数量加成之外，使其仍能生产但不额外贡献多工厂速度奖励。 | `ExcludeFromMultipleFactoryBonus` | 由生产速度统计 owner 统一计数，改属和销毁时更新。 | L/S · [Exclude Factory from providing multiple factory bonus](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#exclude-factory-from-providing-multiple-factory-bonus) |
| PH517 工业工厂增益筛选与叠加上限 | FactoryPlant 可按类型白黑名单限制受益对象，并给每种增益建筑设置有效叠加次数上限；有效时仍考虑 Ares 对应倍率。 | `FactoryPlant.AllowTypes`、`FactoryPlant.DisallowTypes`、`FactoryPlant.MaxCount` | 统一生产费用政策与阵营建筑统计，guest 只提供增益决策。 | L · [FactoryPlant customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#factoryplant-customizations) |
| PH567 城门连接围墙 | 通过 AI 的东西、南北城门列表注册新城门，使其连接围墙；游戏北向为屏幕右上，相关地基方向应按游戏坐标理解。 | 无独立示例键；见原文条件 | 统一围墙连接与城门注册，校验地基宽高和方向。 | L · [Gates connecting with Walls](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#gates-connecting-with-walls) |
| PH593 电厂受损发电衰减 | 可缩放电厂因损伤损失的发电量，零倍率使掉血不减产；结果不会降至零以下，不是直接将总发电量乘该倍率。 | `PowerPlant.DamageFactor` | 统一电力数值核处理衰减，再更新阵营电力账目。 | L · [Power plant damage factor](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#power-plant-damage-factor) |
| PH606 建筑连射的蓄力动画等待 | 可调整建筑连射与蓄力动画等待的关系，实现一次蓄力后连续攻击；棱镜塔自身的硬编码延迟不受该开关控制。 | `IsAnimDelayedBurst` | 由开火调度 owner 管理蓄力与 Burst 时序，动画只消费阶段。 | L · [Skip anim delay for burst fire](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#skip-anim-delay-for-burst-fire) |
| PH611 坦克碉堡兼容与更新频率 | 改善奇数正方形地基的碉堡，允许缩短状态更新等待；可跳过部分入堡限制，但仍要求 Bunkerable、未被寄生且移动器实际兼容。 | `BunkerStateUpdateDelay`、`BunkerableAnyway` | 碉堡 owner 统一资格、停靠与解除，不能只取消表面检查。 | L · [Tank bunker improvements](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#tank-bunker-improvements) |
| PH615 维修建筑的单位修理 | 维修厂、机场和医院可配置周期、恢复量、费用倍率及是否收费；机场负周期禁用维修，显式周期会覆盖按机型补弹节奏。 | `Units.RepairRate`、`Units.RepairStep`、`Units.RepairPercent`、`Units.UseRepairCost` | 维修事务统一协调生命、资金和停靠，需受控经济与修复接口。 | L/S · [Unit repair customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#unit-repair-customization) |

### 抛射体

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH079 空爆与弹体分裂扩展 | 重做空爆和分裂目标选择，可控制数量、散布、重复目标、重选原射手、武器筛选、火力继承及发射特效；部分同名 Ares 设置由 Phobos 优先。 | `Splits.TargetingDistance.Cylindrical`、`Splits.AllowRepeatTargets`、`Splits.UseWeaponTargeting`、`Airburst.UseCluster`、`Airburst.TargetAsSource.SkipHeight`、`AirburstWeapon.ApplyFirepowerMult`、`AirburstWeapon.UseFiringEffects` 等（21个不同示例键） | 需弹体生命周期、目标查询、确定性随机与受控武器发射接口。 | L/S · [Airburst & Splits](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#airburst-splits) |
| PH098 抛射弹道命中与射程 | 可控制高差造成的固有误差，并决定射程判断是否额外考虑建筑地基尺寸；均有全局默认和单弹体覆盖。 | `Arcing.AllowElevationInaccuracy`、`Arcing.ConsiderBuildingFoundation` | 在统一弹道与射程数值核中实现，避免 guest 复制几何计算。 | L · [Arcing projectile customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#arcing-projectile-customization) |
| PH427 集束附加爆炸散布 | Cluster 大于一时，可指定额外爆炸相对原始爆点的最小、最大距离，单位为格；不等于修改分裂弹体的目标距离。 | `ClusterScatter.Min`、`ClusterScatter.Max` | 由引爆流程统一采样散布，复用同步随机和坐标数值核。 | L/S · [Cluster scatter distance customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#cluster-scatter-distance-customization) |
| PH449 弹体独立重力 | 每种弹体可指定重力；零重力可能导致反向飞行和高差目标无法命中，需要直线弹道时应使用专门的 Straight 轨迹。 | `Gravity` | 扩展共享弹道数值核，保留类型、单位与边界校验。 | L/S · [Customizable projectile gravity](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-projectile-gravity) |
| PH469 导弹失去空中目标后的行为 | 可按弹体配置安全高度，并选择失去空中目标后爬升自爆，或飞往目标最后坐标引爆；可继承全局默认。 | `MissileKeepTargetCoord`、`MissileSafetyAltitude` | 弹体 owner 保留最后目标坐标并执行明确失效转移。 | L · [Customize MissileSafetyAltitude and whether missiles fly to the target or climb when losing target](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-missilesafetyaltitude-and-whether-missiles-fly-to-the-target-or-climb-when-losing-target) |
| PH491 弹体初始垂直朝向 | 可显式控制弹体是否以向下方向起步，解除该行为与体素图像的绑定；未设置时体素或 Vertical 弹体默认启用。 | `VerticalInitialFacing` | 弹体初始状态由统一发射 owner 设置，区分图像与运动朝向。 | L · [Customizing initial facing behavior](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-initial-facing-behavior) |
| PH564 高射散布距离 | Inviso 弹体的 FlakScatter 可设最小、最大散布参数；原版的二倍系数仍保留，不能将配置值直接当最终散布半径。 | `BallisticScatter.Min`、`BallisticScatter.Max` | 共享散布数值核保留换算顺序和同步随机消耗。 | L/S · [FlakScatter distance customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#flakscatter-distance-customization) |
| PH603 破片触发与目标筛选 | 破片可在地面或建筑命中时触发，遵守武器及附加效果筛选，避免同建筑重复命中；启用武器筛选会替代原有仅敌军限制。 | `Shrapnel.AffectsGround`、`Shrapnel.AffectsBuildings`、`Shrapnel.UseWeaponTargeting`、`Shrapnel.IgnoreHitBuildings`、`Shrapnel.ObeyWarheadTriggerConditions` | 需弹体引爆、目标查询与受控发射接口，统一同建筑去重规则。 | L/S · [Shrapnel enhancements](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#shrapnel-enhancements) |
| PH629 SHP 弹体绘制深度 | SHP 弹体可通过 art 的 ZAdjust 修正遮挡，例如从建筑发射时被建筑遮住；体素弹体使用另一种逐像素深度算法，不适用此键。 | `ZAdjust` | 分离 SHP 与 VXL 深度管线，不提供误导性的通用 Z 偏移。 | L · [ZAdjust for Projectiles](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#zadjust-for-projectiles) |

### 通用作战对象

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH081 空袭标记外观 | 为呼叫空袭的单位分别配置目标染色索引与指示线、光点颜色，并提供全局默认；不改变空袭伤害和目标筛选。 | `AirstrikeLineColor`、`LaserTargetColor` | 呈现层读取冻结样式，guest 无须直接操纵渲染器。 | L · [Airstrike flare visual customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#airstrike-flare-visual-customizations) |
| PH082 空袭目标资格 | 将目标能否被空袭与 CanC4 解耦，并为弹头限定允许的目标类别；空袭对准目标对象，支持正确指定非建筑目标。 | `AllowAirstrike`、`AirstrikeTargets` | 在空袭目标判定 owner 加策略入口，保持建筑与移动目标一致。 | L/S · [Airstrike target eligibility](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#airstrike-target-eligibility) |
| PH086 关闭索敌提前退出优化 | 可关闭发现较近有效目标后提前结束搜索的规则，使更远但威胁更高的目标仍参与比较；可能改变单位目标选择结果。 | `DisableOveroptimizationInTargeting` | 修改统一索敌评分流程，避免 guest 另建竞争性索敌系统。 | L · [Allow disable an over-optimization in targeting](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-disable-an-over-optimization-in-targeting) |
| PH088 编队征召类型替代 | 现存单位可在 AI 或触发创建编队时被视为其他类型征召；仅影响地图上已有对象，不使 AI 自动生产替代类型。 | `TeamMember.ConsideredAs` | 需编队征召候选查询接口，不能仅改变类型注册表。 | L · [Allow techno type considered as other type when recruiting techno for teams](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-techno-type-considered-as-other-type-when-recruiting-techno-for-teams) |
| PH092 替代发射坐标计算 | 控制 AlternateFLH 相对炮塔还是车体计算，并决定车辆乘客是否使用运输载具的替代发射点，影响开放式运输等坐标逻辑。 | `AlternateFLH.OnTurret`、`AlternateFLH.ApplyVehicle` | 统一发射坐标计算入口，明确车体、炮塔和乘客坐标系。 | L/S · [Alternate FLH customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#alternate-flh-customizations) |
| PH420 建筑赋予自愈的范围与上限 | 配置步兵和车辆自愈建筑的叠加上限、盟军及被动阵营受益，按对象指定自愈类别，并调整自愈图标与偏移。 | `InfantryGainSelfHealCap`、`UnitsGainSelfHealCap`、`GainSelfHealAllowMultiplayPassive`、`GainSelfHealFromPlayerControl`、`GainSelfHealFromAllies`、`Pips.SelfHeal.Infantry`、`Pips.SelfHeal.Units` 等（12个不同示例键） | 自愈模拟、阵营资格和图标显示分层，计时与生命写入仍归引擎。 | L/S · [Building-provided self-healing customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#building-provided-self-healing-customization) |
| PH437 超时空传送前后等待 | 为 ChronoSphere/Warp 单位传送设置跳跃前后延迟，支持全局和类型覆盖；不适用于 Ares 自定义超时空传送的建筑。 | `ChronoSphereDelay`、`ChronoSpherePreDelay` | 由传送状态机管理阶段计时，显式区分单位与建筑路径。 | L/S · [Customizable ChronoSphere teleport delays for units](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-chronosphere-teleport-delays-for-units) |
| PH444 按矿物指定采矿动画 | 矿车可为不同矿物 ID 配置采集动画和每方向帧数，让同一采矿单位针对矿石类型呈现不同采集效果。 | `OreGathering.Anims`、`OreGathering.FramesPerDir`、`OreGathering.Tiberiums` | 在采矿状态到动画的投影处接入，校验并行列表映射。 | L/S · [Customizable harvester ore gathering animation](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-harvester-ore-gathering-animation) |
| PH448 空投后的初始任务 | 可分别指定人类与 AI 空投单位落地后的任务，支持全局与类型覆盖；车辆也采用与步兵一致的人类警戒、AI 猎杀默认。 | `ParadropMission`、`AIParadropMission` | 由空投生成流程提交任务，避免创建后再由插件抢写任务。 | L · [Customizable paradrop missions](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-paradrop-missions) |
| PH453 编队索敌的地图区域筛选 | 非飞机编队的攻击任务可只选同地图连通区、任意区域，或仅在射程内跨区选目标；并不等于改变寻路可达性。 | `TargetZoneScanType` | 在编队索敌候选阶段加政策，仍由寻路 owner 判定移动能力。 | L/S · [Customizable target evaluation map zone check behaviour](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-target-evaluation-map-zone-check-behaviour) |
| PH454 单位独立超时空移动参数 | 按单位覆盖超时空移动计时与出入场动画，并可为超时空传送仪单独指定动画；清除动画可覆盖 Ares 弹头的对应设置。 | `WarpOut`、`WarpIn`、`Chronoshift.WarpOut`、`Chronoshift.WarpIn`、`WarpAway`、`ChronoTrigger`、`ChronoDistanceFactor` 等（10个不同示例键） | 统一传送状态机与表现事件，随机动画使用规定的随机来源。 | L/S · [Customizable Teleport/Chrono Locomotor settings per TechnoType](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-teleport-chrono-locomotor-settings-per-technotype) |
| PH456 军衔图标与模式覆盖 | 可按等级、IFV 武器模式和载员数量配置军衔素材、帧及调色板，控制敌方可见和选中显示；各覆盖层存在明确优先级。 | `EnemyInsignia`、`DrawInsignia.OnlyOnSelected`、`DrawInsignia.UsePixelSelectionBracketDelta`、`DrawInsignia.AdjustPos.Infantry`、`DrawInsignia.AdjustPos.Units`、`DrawInsignia.AdjustPos.Buildings`、`DrawInsignia.AdjustPos.BuildingsAnchor` 等（47个不同示例键） | 呈现层解析样式继承，模拟仅提供等级、武器模式和载员快照。 | L/S · [Customizable veterancy insignias](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-veterancy-insignias) |
| PH457 单位尾波动画 | 单位可覆盖普通、寄生缠绕和下沉尾波，按移动器设全局是否产生尾波，并能让步行移动器也显示水面尾波。 | `WalkLocomotorMakesWake`、`HoverLocomotorMakesWake`、`DriveLocomotorMakesWake`、`ShipLocomotorMakesWake`、`MakesWake`、`Wake`、`Wake.Grapple` 等（8个不同示例键） | 由移动状态投影触发表现，避免尾波创建反向影响模拟。 | L/S · [Customizable wake anim](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-wake-anim) |
| PH460 断桥跌落伤害 | 可为落地、落水分别配置固定或百分比伤害，决定 EMP 失能单位是否采用该规则；负百分比按当前生命计算。 | `FallingDownDamage`、`FallingDownDamage.AllowEMP`、`FallingDownDamage.Water` | 落地事务调用统一伤害入口，冻结百分比语义与水陆分支。 | L · [Customize bridge falling down damage](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-bridge-falling-down-damage) |
| PH461 飞机坠毁旋转倍率 | Fly 移动器对象可调整坠毁时旋转速度倍率，使大型飞机不必只在旋转和完全不旋转间选择。 | `CrashSpin.Multiplier` | 由坠毁表现状态消费倍率，保持坠毁生命和碰撞的引擎权威。 | L · [Customize crash spin multiplier](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-crash-spin-multiplier) |
| PH462 按 IFV 模式区域警戒 | Gunner 单位可限定哪些武器模式使用默认区域警戒或 GUARD_AREA 升级能力，并能区分人类与 AI 的模式名单。 | `DefaultToGuardArea.Modes`、`DefaultToGuardArea.AIModes` | 将模式资格接入任务选择政策，复用同一警戒实现。 | L · [Customize DefaultToGuardArea per gunner mode](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-defaulttoguardarea-per-gunner-mode) |
| PH474 视野共享的所属方范围 | 可按全局、国家、单位类型指定 Sight 向哪些所属方提供视野，单位类型设置覆盖国家，再覆盖全局默认。 | `RevealHouses` | 由视野共享 owner 解析阵营关系，存档及改属时更新来源。 | L · [Customize the country displayed in Sight](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-country-displayed-in-sight) |
| PH477 飞行移动器降落动画 | Fly 对象可按类型覆盖降落动画，并按普通、IsDropship、Carryall 分别设置全局后备，不再只用硬编码的两种动画。 | `DefaultLandingAnim`、`DefaultLandingAnim.Dropship`、`DefaultLandingAnim.Carryall`、`LandingAnim` | 在降落状态事件与资源样式间建立映射，统一类型后备规则。 | L · [Customize the landing animation of technos that have Locomotor=Fly](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-landing-animation-of-technos-that-have-locomotor-fly) |
| PH484 飞行移动器摆动 | 可独立启用或禁止 Fly 对象摆动，不必借用 IsDropship 及其附带行为；未设置时仍按原版类型标志判断。 | `FlyNoWobbles` | 呈现层读取独立摆动政策，保留移动状态唯一来源。 | L · [Customize whether technos with Locomotor=Fly wobble](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-technos-with-locomotor-fly-wobble) |
| PH485 心灵探测可检测性 | 为单位指定是否被具备 PsychicDetectionRadius 的建筑检测，默认可检测；该设置针对心灵探测而非一般隐形探测。 | `PsychicDetectable` | 在心灵探测候选机制中应用，勿替代其他探测规则。 | L · [Customize whether the unit can be detected by psychic detector](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-the-unit-can-be-detected-by-psychic-detector) |
| PH486 单位是否从屋顶出厂 | 单位可显式选择从工厂屋顶离开，覆盖由 BalloonHover 或 JumpJet 自动决定的默认出厂路径。 | `ExitThroughRoof` | 工厂出厂 owner 统一处理屋顶坐标、占格和初始任务。 | L · [Customize whether the unit exits from the roof](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-the-unit-exits-from-the-roof) |
| PH487 死亡后触发标签转移 | 可选择将单位的触发标签移交给车辆窃贼、逃生乘员或乘客，修正扩展环境中只转给乘员的局限。 | `Survivors.ReplaceTagTo` | 由死亡清理事务负责标签迁移，处理多幸存者引用关系。 | L · [Customize which of vehicle thief, pilot, and passenger the tag can be transferred to when the unit is destroyed](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-which-of-vehicle-thief-pilot-and-passenger-the-tag-can-be-transferred-to-when-the-unit-is-destroyed) |
| PH496 受损移动速度 | 可覆盖车辆或船只的受损速度倍率，最大有效值为一；不能借此让黄血单位比未受损时移动更快。 | `DamagedSpeed` | 由统一速度计算 owner 读取配置并钳制边界。 | L · [Damaged speed customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#damaged-speed-customization) |
| PH497 单位体素碎片数量策略 | 按类型配置碎片最小、最大数量，并选择不足总数时是否重新遍历类型；启用限制会在一轮结束后停止生成。 | `DebrisTypes.Limit`、`DebrisMaximums`、`DebrisMinimums` | 复用统一碎片生成器，明确总数和分类型配额的先后顺序。 | L · [Debris voxel animations limitation](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#debris-voxel-animations-limitation) |
| PH498 寄生击杀水面目标后出场 | 可让非 Naval 寄生者也在水面击杀后重新出场，或禁止 Naval 寄生者如此出场；仍保留所在格是否有建筑的检查。 | `Parasite.AllowWaterExit` | 由寄生释放事务执行出场资格与占格检查，统一失败清理。 | L · [Dehardcode of parasites unlimboing after killing naval targets](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#dehardcode-of-parasites-unlimboing-after-killing-naval-targets) |
| PH508 空降舱类型参数 | 非建筑对象可覆盖空降舱角度、高度、速度、拖尾和武器；舱体 AirImage 仅对步兵绘制，较小速度受原有下降限速影响。 | `DropPod.Angle`、`DropPod.AtmosphereEntry`、`DropPod.GroundAnim`、`DropPod.AirImage`、`DropPod.Height`、`DropPod.Puff`、`DropPod.Speed` 等（12个不同示例键） | 需空降舱任务、落地与武器接口，表现素材与模拟状态分离。 | L/S · [DropPod](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#droppod) |
| PH513 进入回收站确认语音 | 单位收到进入回收站命令时可播放 VoiceEnterGrinder，替代原版 VoiceSpecialAttack；与回收发生时的音效分别配置。 | `VoiceEnterGrinder` | 在命令确认音频层接入，不改变回收结算时机。 | L · [Enter the grinder voice](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#enter-the-grinder-voice) |
| PH515 爆炸单位的乘客与建造阶段 | Explodes 单位可允许乘客正常逃生；建筑可在建造或出售阶段视为未启用 Explodes，避免未完成建筑也触发同类爆炸。 | `Explodes.KillPassengers`、`Explodes.DuringBuildup` | 死亡事务一次决定爆炸、载员和阶段分支，不能分别抢写。 | L/S · [Exploding object customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#exploding-object-customizations) |
| PH565 按类别禁止 AI 并行生产 | 可按工厂类别或单位类型禁止 AI 并行队列，避免全面禁用；若 Ares 已设 AllowParallelAIQueues=no，这些细分设置无效。 | `AllowParallelAIQueues`、`ForbidParallelAIQueues.Infantry`、`ForbidParallelAIQueues.Vehicle`、`ForbidParallelAIQueues.Navy`、`ForbidParallelAIQueues.Aircraft`、`ForbidParallelAIQueues.Building`、`ForbidParallelAIQueues` | 由生产队列 owner 统一仲裁并行资格，避免插件另建队列。 | L/S · [Forbid parallel AI queues](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#forbid-parallel-ai-queues) |
| PH566 索敌分散到不同帧 | 创建时给索敌计时引入零至十五帧的随机初始偏移，使同帧大量生成的对象分散索敌；可只作用于 AI。 | `DistributeTargetingFrame`、`DistributeTargetingFrame.AIOnly` | 调度优化由索敌 owner 执行，初始化随机必须可重放。 | L · [Force techno targeting in distributed frames to improve performance](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#force-techno-targeting-in-distributed-frames-to-improve-performance) |
| PH572 铁幕和力场对有机体的效果 | 可将铁幕或 Force Shield 对有机体的行为选为杀死、无敌或忽略，并指定击杀弹头；这里的 Force Shield 不是再生生命护盾。 | `IronCurtain.EffectOnOrganics`、`IronCurtain.KillOrganicsWarhead`、`ForceShield.EffectOnOrganics`、`ForceShield.KillOrganicsWarhead`、`IronCurtain.Effect`、`IronCurtain.KillWarhead`、`ForceShield.Effect` 等（8个不同示例键） | 在无敌状态施加事务中接入，死亡通过既有伤害与清理流程。 | L/S · [Iron Curtain & Force Shield effects on organics customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#iron-curtain-force-shield-effects-on-organics-customization) |
| PH576 Jumpjet 提前爬升 | 可提前五格预测障碍高度、允许边升高边前进，或忽略建筑高度；单独预测不足以越过障碍时仍可能停下。 | `JumpjetClimbPredictHeight`、`JumpjetClimbWithoutCutOut`、`JumpjetClimbIgnoreBuilding` | 扩展同一移动器的障碍预测与运动政策，保持碰撞与高度一致。 | L · [Jumpjet climbing logic enhancement](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#jumpjet-climbing-logic-enhancement) |
| PH577 Jumpjet 坠毁转向 | 可关闭 Jumpjet 坠毁时失控转向，并配置坠毁转向速率；官方提示这组设置后续仍可能变动。 | `RotateOnCrash`、`JumpjetRotateOnCrash`、`CrashROT` | 由坠毁移动状态 owner 处理，版本冻结并避免依赖未确认默认。 | L/S · [Jumpjet rotating on crashing toggle](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#jumpjet-rotating-on-crashing-toggle) |
| PH578 接近目标时直接追踪对象 | 接近目标时可将目标对象本身设为目的地，而非射程圆弧上的点；可配合入射程即停，并保留旧键兼容别名。 | `ApproachTarget.PursuitTarget` | 统一接近目标任务与停止政策，不另建插件追击循环。 | L · [Keep pursuing the target during ApproachTarget](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#keep-pursuing-the-target-during-approachtarget) |
| PH579 低电量销毁生成飞机 | 需电建筑可在低电量时停止对敌选取目标，并令其正在飞行的生成飞机自毁，适用于建筑使用航母式生成机机制。 | `Powered.KillSpawns` | 生成机 owner 协调电力、目标和销毁，清理母体引用。 | L/S · [Kill spawns on low power](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#kill-spawns-on-low-power) |
| PH591 弹药、载机与矿仓图标 | 可配置状态点的素材帧、间距、偏移、空槽和顺序，分别处理建筑与其他对象；弹药环绕与载机图标仍有独立规则。 | `Pips.Generic.Size`、`Pips.Generic.Buildings.Size`、`Pips.Ammo.Size`、`Pips.Ammo.Buildings.Size`、`Pips.Tiberiums.EmptyFrame`、`Pips.Tiberiums.Frames`、`Pips.Tiberiums.DisplayOrder` 等（19个不同示例键） | 呈现层读取状态快照并解析类型样式，无需 guest 任意 UI 写操作。 | L/S · [PipScale pip customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#pipscale-pip-customizations) |
| PH592 移动单位产电与耗电 | 启用全局开关后步兵、车辆和飞机也可贡献 Power；正值产电、负值耗电，扩展了阵营电力来源的对象范围。 | `UnitPowerDrain`、`Power` | 由电力统计 owner 跟踪创建、销毁、改属等生命周期。 | L · [Power drain for units](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#power-drain-for-units) |
| PH596 按所属方隐藏雷达目标 | 可指定 RadarInvisibleToHouse，使雷达隐形也作用于盟军等关系方；未配置仍依原 RadarInvisible 推导默认。 | `RadarInvisibleToHouse` | 雷达可见性统一使用当前阵营关系，改盟后及时更新。 | L · [RadarInvisible for non-enemy house](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#radarinvisible-for-non-enemy-house) |
| PH608 真正固定不动的单位 | 固有 Speed 为零或受移动地形限制的步兵、车辆不再尝试移动，索敌限于射程；临时速度倍率为零与固有静止的命令行为不同。 | 无独立示例键；见原文条件 | 移动、任务和光标共享静止资格，避免可造单位卡死工厂。 | L · [Stationary units](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#stationary-units) |
| PH609 接近途中入射程即停 | 单位接近目标时可在目标进入射程立即停止，避免继续多走；支持全局、类型设置和旧版 AttackMove 兼容别名。 | `ApproachTarget.StopWhenInRange` | 由接近目标任务统一检查射程并清理移动目的地。 | L · [Stop immediately if the target enters the range during ApproachTarget](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#stop-immediately-if-the-target-enters-the-range-during-approachtarget) |
| PH610 潜地高度与水平速度 | 可按全局或单位设置地下移动高度及水平速度；不改变钻入钻出的垂直速度，高度需为负且接近地表可能异常。 | `SubterraneanHeight`、`SubterraneanSpeed` | 由 Tunnel 移动器消费配置，区分水平和垂直运动单位。 | L/S · [Subterranean unit travel height and speed](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#subterranean-unit-travel-height-and-speed) |
| PH612 索敌与区域警戒范围 | 可覆盖目标扫描的硬上限和区域警戒范围；有效范围先加倍再施加上限，不能直接把 GuardRange 当实际扫描半径。 | `MaxGuardRange`、`AreaGuardRange` | 复用统一搜索范围数值核，分别解析普通和区域警戒政策。 | L · [Target scan/guard range customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#target-scan-guard-range-customizations) |
| PH613 按对象与控制方设索敌间隔 | 可分别为 AI 和人类对象设置普通、区域警戒及攻击移动索敌延迟，并细化到类型，在响应速度与计算开销间调整。 | `AINormalTargetingDelay`、`PlayerNormalTargetingDelay`、`AIGuardAreaTargetingDelay`、`PlayerGuardAreaTargetingDelay`、`AIAttackMoveTargetingDelay`、`PlayerAttackMoveTargetingDelay` | 索敌调度 owner 管理计时和缓存失效，guest 提供冻结间隔政策。 | L · [Target scanning delay optimization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#target-scanning-delay-optimization) |
| PH621 体素多分段影子 | 车辆和飞机可同时为多个体素分段绘制影子，并指定各段使用的 HVA 帧；负一表示跟随当前动画帧。 | `ShadowIndices`、`ShadowIndex.Frame`、`ShadowIndices.Frame` | 呈现层处理 VXL 分段和 HVA 采样，模拟仅提供姿态。 | L/S · [Voxel body multi-section shadows](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#voxel-body-multi-section-shadows) |
| PH623 空中体素影子缩放 | 可按基础比例或高度公式缩放空中影子，设最小比例与类型特征高度；适用于车辆和飞机体素，不是单位本体缩放。 | `AirShadowBaseScale`、`HeightShadowScaling`、`HeightShadowScaling.MinScale`、`ShadowSizeCharacteristicHeight` | 呈现层根据高度快照计算影子，明确高度单位与指数边界。 | L/S · [Voxel shadow scaling in air](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#voxel-shadow-scaling-in-air) |

### 车辆

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH087 矿车区域警戒 | 玩家矿车可执行区域警戒，另可指定警戒范围内无有效目标时恢复采矿；不对 AI 矿车开放以免阻断经济。 | `Harvester.CanGuardArea`、`Harvester.CanGuardArea.RequireTarget` | 在矿车任务转换策略中接入，保留采矿任务的唯一 owner。 | L · [Allow miners do area guard](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-miners-do-area-guard) |
| PH434 工具箱单位重抽概率 | 为可由工具箱奖励的车辆设置再次抽取概率，间接降低特定类型的奖励权重；不是直接给每种车辆设最终概率。 | `CrateGoodie.RerollChance` | 在工具箱单位选择 owner 中实现重抽并校验终止条件。 | L/S · [Custom Unit Crate Reroll Chance](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-unit-crate-reroll-chance) |
| PH463 矿车每次卸矿数量 | 可限制一次卸矿步骤交出的矿物量，形成分批卸矿；数值小于等于零时不限制，仍一次卸完。 | `HarvesterDumpAmount` | 在矿物转移 owner 中处理分批事务，避免资源重复结算。 | L · [Customize harvester dump amount](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-harvester-dump-amount) |
| PH464 按矿车配置卸矿速率 | 每种矿车可覆盖全局 HarvesterDumpRate，控制卸矿过程的节奏；未设置仍继承全局值。 | `HarvesterDumpRate` | 由卸矿任务计时机制读取类型配置，明确原始单位。 | L · [Customize HarvesterDumpRate](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-harvesterdumprate) |
| PH465 按矿车配置采矿速率 | 每种矿车可覆盖全局 HarvesterLoadRate，调整采集节奏；与每次卸矿数量和卸矿速率分别配置。 | `HarvesterLoadRate` | 由采矿计时 owner 消费配置，保持采集量结算路径唯一。 | L · [Customize HarvesterLoadRate](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-harvesterloadrate) |
| PH466 步兵闲置动作间隔 | 单值沿用全局频率换算随机间隔；双值直接表示最小、最大游戏帧数，因此两种写法的数值单位不同。 | `IdleActionFrequency` | 解析时区分标量和区间，闲置表现采用明确随机策略。 | L · [Customize IdleActionFrequency](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-idleactionfrequency) |
| PH480 IFV 按模式编组选择 | 同型号 IFV 可按武器模式分组选取，相同分组一起选择；支持规则默认及玩家 RA2MD.INI 开关，武器模式编号从一开始。 | `TypeSelectUseIFVMode`、`WeaponGroupAsN` | 客户端选择层消费模式快照，保持玩家偏好不改变战斗模拟。 | L · [Customize type selection for IFV](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-type-selection-for-ifv) |
| PH488 碾压倾斜与减速 | 分别配置碾车辆和墙时的倾斜与减速；完全跳过减速逻辑不等于倍率设为一，可避免加速车辆偶发掉速。 | `CrushSlowdownMultiplier`、`SkipCrushSlowdown`、`TiltsWhenCrushes.Vehicles`、`TiltsWhenCrushes.Overlays`、`CrushForwardTiltPerFrame`、`CrushOverlayExtraForwardTilt` | 将姿态表现与移动减速分开，由移动 owner 应用速度政策。 | L/S · [Customizing crushing tilt and slowdown](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-crushing-tilt-and-slowdown) |
| PH502 载员车辆与矿车部署 | 车辆可跳过卸客或在空载后执行其他部署动作；矿车可部署并选择是否要求无矿，但正在卸矿时禁止部署。 | `Deploy.SkipPassengerUnload`、`Deploy.NoPassenger`、`Deploy.NoTiberium` | 由部署事务协调载员、资源和类型转换，避免多套任务竞争。 | L · [Deployment Enhancement](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#deployment-enhancement) |
| PH503 车辆销毁动画 | DestroyAnim 扩展到车辆并可随机选动画，动画保留所属方与朝向，供动画生成单位等后续逻辑使用。 | `DestroyAnim.Random`、`DestroyAnim` | 死亡流程发出含归属的事件，需受控动画及单位创建接口。 | L/S · [Destroy animations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#destroy-animations) |
| PH574 简单部署的地形与动画 | 可校验部署后类型的通行能力、禁用地形或建筑重叠，配置部署朝向、多方向动画与收起倒放；DeployDir 为八方向或负一。 | `IsSimpleDeployer.ConsiderPathfinding`、`IsSimpleDeployer.DisallowedLandTypes`、`IsSimpleDeployer.DisallowOnBuildings`、`DeployDir`、`DeployingAnims`、`DeployingAnim.KeepUnitVisible`、`DeployingAnim.ReverseForUndeploy` 等（8个不同示例键） | 部署资格由模拟 owner 判定，动画与素材选择留在呈现层。 | L · [IsSimpleDeployer customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#issimpledeployer-customizations) |
| PH575 部署时增减弹药 | 车辆展开或收起后可增减弹药，并限制在零与最大容量之间；若同时换型，容量采用转换后的类型。 | `Ammo.AddOnDeploy` | 由部署事务调用弹药 owner，现有武器策略查询不能代替该写接口。 | L/S · [IsSimpleDeployer vehicle ammo change on deploy](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#issimpledeployer-vehicle-ammo-change-on-deploy) |
| PH583 矿车卸矿后重新找近矿 | 卸矿后可额外搜索更近矿物，替代总是返回上次装满位置的行为，避免矿车逐渐钻进远矿而忽略附近资源。 | `HarvesterScanAfterUnload` | 采矿任务 owner 统一搜索候选与保留地点，不建立平行矿区缓存。 | L · [Make harvesters do addtional scan after unload](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#make-harvesters-do-addtional-scan-after-unload) |
| PH594 部署换型保留无敌状态 | DeploysInto/UndeploysInto 转换可保留铁幕或 Force Shield 并重新应用对应修正；二者全局默认不同，不能共用一个开关。 | `IronCurtain.KeptOnDeploy`、`ForceShield.KeptOnDeploy` | 由类型转换事务迁移状态和剩余计时，确保单一权威。 | L/S · [Preserve Iron Curtain / Force Shield status on type conversion](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#preserve-iron-curtain-force-shield-status-on-type-conversion) |
| PH601 移动命令保留攻击目标 | 车辆移动时可保留目标，并指定越距判断用的武器、额外距离和停车后是否追击；该规则不保证单位实际能移动射击。 | `KeepTargetOnMove`、`KeepTargetOnMove.Weapon`、`KeepTargetOnMove.NoMorePursuit`、`KeepTargetOnMove.ExtraDistance` | 由命令与目标 owner 协作管理目标，复用开火资格判断。 | L · [Retain target on movement command](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#retain-target-on-movement-command) |
| PH605 沉船资格与下沉速度 | 车辆可独立设置是否沉没、每帧下沉速度，并控制乌贼击杀时直接沉没还是先造成致命伤再按沉船规则处理。 | `Sinkable`、`SinkSpeed`、`Sinkable.SquidGrab` | 死亡与寄生释放事务统一选择下沉路径，保留伤害事件差异。 | L · [Sinking behavior dehardcode](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#sinking-behavior-dehardcode) |
| PH614 炮塔与炮管后坐 | 车辆可配置炮塔、炮管压缩、保持和恢复帧数及距离，单武器可抑制后坐；官方注明与泰伯利亚之日实现可能存在差异。 | `TurretRecoil`、`TurretTravel`、`TurretCompressFrames`、`TurretHoldFrames`、`TurretRecoverFrames`、`BarrelTravel`、`BarrelCompressFrames` 等（10个不同示例键） | 呈现层从开火事件驱动后坐，不以动画计时影响弹药与伤害。 | L · [Turret recoil](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#turret-recoil) |
| PH616 无炮塔车辆冷却中转向 | 无炮塔车辆可在武器尚未冷却完时转向目标；满足相应全向开火设置的 Jumpjet 也可受影响。 | `NoTurret.TrackTarget` | 由朝向与开火 owner 协调更新顺序，避免插件直接写姿态。 | L · [Unit Without Turret Always Turn To Target](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#unit-without-turret-always-turn-to-target) |
| PH624 车辆体素炮塔影子 | 可全局启用车辆炮塔影子，并在 art 中按车型覆盖，使炮塔与车体一起参与阴影呈现。 | `DrawTurretShadow`、`TurretShadow` | 扩展体素渲染资源选择，复用同一阴影绘制管线。 | L/S · [Voxel turret shadow](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#voxel-turret-shadow) |

### 弹头与伤害

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH089 超时空清除倍率 | Temporal 清除时间可选考虑弹头护甲倍率、卧倒倍率以及火力等修正；支持全局默认与单弹头覆盖。 | `Temporal.ApplyVersus`、`Temporal.ApplyMultiplier` | 复用伤害数值内核，并为 Temporal 清除计时开放明确政策。 | L · [Allow Temporal warhead to apply ratio and bonus](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allow-temporal-warhead-to-apply-ratio-and-bonus) |
| PH090 允许弹头伤害射手 | 即使未启用 DamageSelf，也可允许弹头伤害自身射手并应用相邻伤害效果；狂暴仍受所属方限制，不能据此认定所有效果都能自施。 | `AllowDamageOnSelf` | 需弹头命中归因和受影响方策略，伤害写入交由既有 owner。 | L/S · [Allowing damage dealt to firer](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#allowing-damage-dealt-to-firer) |
| PH105 狂暴持续时间叠加 | 可选覆盖、仅首次、取较短或较长、加减乘除等时间叠加方式；部分方式在目标尚未狂暴时不能产生首次效果。 | `Psychedelic.StackingMode` | 由狂暴状态 owner 应用 guest 返回的叠加政策并处理除零边界。 | L · [Berzerk ( Psychedelic ) duration stacking customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#berzerk-psychedelic-duration-stacking-customization) |
| PH428 战斗光照条件与颜色 | 可按细节等级、彩色光效是否参与等级筛选及概率控制命中闪光；CLIsBlack 可强制黑色闪光。 | `CombatLightDetailLevel`、`CombatLightDetailLevel.CheckColored`、`CombatLightChance`、`CLIsBlack` | 将效果意图交给呈现层，明确表现随机不能影响模拟随机流。 | L/S · [Combat light customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#combat-light-customizations) |
| PH430 弹头自定义碎片动画 | 弹头未配置体素碎片时可替换默认金属碎片动画，亦可限制碎片只在非水面产生；并非替代所有死亡动画。 | `Debris.Conventional`、`DebrisAnims` | 由弹头爆炸事件生成呈现意图，保持水陆判定一致。 | L/S · [Custom debris animations and additional debris spawn settings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-debris-animations-and-additional-debris-spawn-settings) |
| PH431 弹头体素碎片数量限制 | 启用 DebrisTypes.Limit 后，各碎片类型可配置最小与最大数量；默认是否启用与最大值列表长度有关，兼顾旧配置。 | `DebrisTypes.Limit`、`DebrisMaximums`、`DebrisMinimums` | 在统一碎片生成器实现数量政策与确定性抽样。 | L · [Custom debris voxel animations limitation](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-debris-voxel-animations-limitation) |
| PH450 震荡倾斜幅度 | Rocker 弹头的摇晃幅度可由固定值替代原伤害推导值，再应用独立倍率；固定值和倍率同时设置时都会生效。 | `Rocker.AmplitudeMultiplier`、`Rocker.AmplitudeOverride` | 在受击表现计算处应用政策，不以摇晃结果修改伤害。 | L/S · [Customizable rocker amplitude](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-rocker-amplitude) |
| PH458 弹头爆炸与水花动画策略 | 爆炸和水花可随机选取、全部播放、按引爆次数节流及散布，支持零伤害动画；计数可能按射手共享，不能一律当作弹头私有状态。 | `CreateAnimsOnZeroDamage`、`Conventional.IgnoreUnits`、`AnimList.PickRandom`、`AnimList.CreateAll`、`AnimList.CreationInterval`、`AnimList.ScatterMin`、`AnimList.ScatterMax` 等（13个不同示例键） | 需明确计数 owner 与生命周期，再生成动画意图。 | L/S · [Customizable Warhead animation behaviour](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-warhead-animation-behaviour) |
| PH459 弹头效果触发资格 | 可按生命比例、等级及中立所属方限制弹头作用，并决定 Phobos 特效是否依赖护甲倍率有效；零生命目标有独立的比例检查规则。 | `AffectsBelowPercent`、`AffectsAbovePercent`、`AffectsVeterancy`、`AffectsNeutral`、`EffectsRequireVerses` | 在统一弹头上下文中过滤，分清伤害许可与扩展效果许可。 | L · [Customizable Warhead trigger conditions](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-warhead-trigger-conditions) |
| PH489 受击是否显形 | 可全局或逐弹头决定造成伤害时是否让目标解除隐形，覆盖原版默认受击显形行为，不替代其他探测与显形机制。 | `DecloakDamagedTargets` | 由隐形状态 owner 接收伤害事件后的策略决策。 | L/S · [Customizing decloak on damaging targets](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-decloak-on-damaging-targets) |
| PH492 移动器弹头的 Jumpjet 参数 | 赋予 Jumpjet 移动器的弹头可覆盖目标转向、速度、高度、爬升、摆动等参数，缺省值继承目标类型对应参数。 | `JumpjetTurnRate`、`JumpjetSpeed`、`JumpjetClimb`、`JumpjetCrash`、`JumpjetHeight`、`JumpjetAccel`、`JumpjetWobbles` 等（9个不同示例键） | 需受控移动器切换接口与参数恢复生命周期，不能只改速度字段。 | L · [Customizing locomotor warhead](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-locomotor-warhead) |
| PH493 寄生目标与缠绕表现 | 可指定寄生攻击会直接清除的目标类别，替换缠绕动画与粒子系统，或关闭默认粒子；这些设置属于寄生弹头机制。 | `Parasite.DisableParticleSystem`、`Parasite.GrappleAnim`、`Parasite.ParticleSystem`、`Parasite.CullingTarget` | 寄生状态、伤害与表现分层，需目标挂接和脱离事件。 | L · [Customizing parasite](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-parasite) |
| PH499 弹头动画深度偏移 | 可覆盖弹头 AnimList 动画原有固定深度偏移；显式设置为零时改用动画自身的 ZAdjust，零不等于无偏移。 | `WarheadAnimZAdjust`、`AnimZAdjust` | 呈现层保留未设置、零和非零三种语义，再计算最终深度。 | L · [Dehardcode the ZAdjust of warhead anim](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#dehardcode-the-zadjust-of-warhead-anim) |
| PH500 心控后自动反击宽限 | 单位刚被临时心控时，可使其原友军一段时间内不自动攻击它；不阻止手动指定攻击，对永久心控也不起作用。 | `MindControl.ThreatDelay` | 由目标威胁资格 owner 跟踪心控宽限，存档保留剩余帧数。 | L · [Delay automatic attack on the controlled unit](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#delay-automatic-attack-on-the-controlled-unit) |
| PH588 非挑衅弹头 | 可抑制受击告警、受攻击触发、AI 防御响应和报复；实际伤害事件仍可能触发，且不抑制 Ares EMP 警告。 | `Nonprovocative` | 弹头上下文携带响应政策，各告警、AI 和触发消费者统一遵守。 | L/S · [Nonprovocative Warheads](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#nonprovocative-warheads) |
| PH600 仅视野内爆炸震屏 | 可让弹头仅在当前屏幕可见区域引爆时触发画面震动，减少远处战斗干扰；有全局默认与单弹头覆盖。 | `ShakeIsLocal` | 只由客户端呈现判断屏幕区域，不将摄像机状态输入模拟。 | L/S · [Restricting screen shaking to current view](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#restricting-screen-shaking-to-current-view) |

### 武器

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH093 环境伤害弹头与排除主目标 | AmbientDamage 可使用独立弹头，或排除武器的主要目标；排除设置可由全局默认再被单个武器覆盖。 | `AmbientDamage.IgnoreTarget`、`AmbientDamage.Warhead` | 扩展武器环境伤害政策，继续通过统一伤害执行路径。 | L/S · [AmbientDamage customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#ambientdamage-customizations) |
| PH099 超时空监狱与驾驶员死亡状态 | 针对 Ares 3.0 起的绑架逻辑，可在归属改变时重置已杀驾驶员状态，或直接阻止无驾驶员车辆改属，避免释放后无法操作。 | `Abductor.ChangeOwner.ResetDriverKilled`、`Abductor.ChangeOwner.IgnoreDriverKilled` | 需绑架、归属和驾驶员状态的事务接口，不能只改国别字段。 | L · [Ares’ Chrono Prisons / Abductors Expansion](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#ares-chrono-prisons-abductors-expansion) |
| PH423 武器覆盖友军攻击规则 | AttackFriendlies 和友军攻击光标可直接写在武器上，并覆盖射手的同名设置，使不同武器拥有不同友军交互行为。 | `AttackFriendlies`、`AttackCursorOnFriendlies` | 统一命令光标与开火资格的政策来源，避免界面允许却不能开火。 | L · [Can attack allies](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#can-attack-allies) |
| PH425 蓄力炮塔分次延迟 | 武器可按连射次数指定蓄力炮塔动画等待，超出列表使用最后一项；显式延迟不再应用 ROF 等修正，非正值不重启原延迟。 | `ChargeTurret.Delays` | 计时由炮塔状态 owner 管理，配置列表冻结并界定 Burst 索引。 | L/S · [Charge turret delays](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#charge-turret-delays) |
| PH441 飞碟激光半径 | 武器可以覆盖飞碟环形激光动画的半径，使不同体型或用途的飞碟采用独立环形外观；默认示例保留原版半径。 | `DiskLaser.Radius` | 呈现层读取武器样式，避免把视觉半径误作武器射程。 | L/S · [Customizable disk laser radius](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-disk-laser-radius) |
| PH451 射速随机附加延迟 | 可全局或按武器覆盖 ROF 额外随机等待，接受固定值或闭区间帧数；改变的不只是动画节奏，也影响实际开火时机。 | `ROF.RandomDelay` | 由武器冷却 owner 抽取同步随机并保存最终计时。 | L/S · [Customizable ROF random delay](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-rof-random-delay) |
| PH468 伊文炸弹图标可见方 | 可全局及按武器限定哪些关系方可以看见伊文炸弹图标，该设置也影响炸弹探测器的显示判断。 | `IvanIconVisibility`、`IvanBomb.Visibility` | 在可见性投影中统一阵营关系判定，不泄露隐藏炸弹状态。 | L · [Customize ivan bomb visibility](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-ivan-bomb-visibility) |
| PH494 飞机开火是否强制卸载乘客 | 可按武器覆盖全局规则，决定飞机尝试开火时强制抛出乘客，还是正常发射武器，解除原版行为绑定。 | `AircraftWeapon.KickOutPassengers`、`KickOutPassengers` | 由飞机攻击任务协调载员与发射事务，保证二者不会重复发生。 | L/S · [Customizing whether passengers are kicked out when an aircraft fires](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizing-whether-passengers-are-kicked-out-when-an-aircraft-fires) |
| PH506 单次开火后保留步兵动作 | FireOnce 武器可不再重置步兵动画序列，让剩余射击帧继续播放；目标仍会像原版一样被忘记。 | `FireOnce.ResetSequence` | 分离武器目标清理与动画序列切换，不让呈现影响目标状态。 | L/S · [Disable FireOnce resetting infantry sequence](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#disable-fireonce-resetting-infantry-sequence) |
| PH509 电弧外观 | 武器可关闭部分电弧、改变分段数和持续时间、选择起点跟随 FLH；仅改变外观，不适用于部分棱镜支援和空爆电弧。 | `Bolt.Disable1`、`Bolt.Disable2`、`Bolt.Disable3`、`Bolt.Arcs`、`Bolt.Duration`、`Bolt.FollowFLH` | 扩展呈现层电弧样式，不据此改变武器命中或伤害。 | L/S · [Electric bolt customizations](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#electric-bolt-customizations) |
| PH510 电弧深度修正 | 可为电弧设置 Z 偏移，并取消建筑电弧初始深度不得为正的限制；该限制开关与额外偏移是两步独立处理。 | `EBoltZAdjust`、`EBoltZAdjust.ClampInitialDepthForBuilding` | 在电弧渲染管线中保留计算顺序与建筑分支。 | L · [Electric bolt Z-adjust](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#electric-bolt-z-adjust) |
| PH581 激光深度偏移 | 可按全局和武器设置激光绘制 Z 偏移，调节遮挡；不适用于棱镜支援武器，不改变激光武器伤害计算。 | `LaserZAdjust` | 呈现层扩展激光样式并保留受限武器分支，统一深度换算。 | L · [Laser Z-adjust](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#laser-z-adjust) |
| PH604 单色激光 | 武器可只使用 LaserInnerColor 绘制可变粗细的单色激光，类似阵营色激光但不取玩家颜色；不适用于棱镜支援武器。 | `IsSingleColor` | 扩展呈现层激光样式，伤害链继续使用原武器机制。 | L/S · [Single-color lasers](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#single-color-lasers) |
| PH617 射击动画跟随朝向 | 射击动画可随射手转向更新类型和位置；与动画 Next 串接合用可能反复改回原类型，导致动画无法结束。 | `FiringAnim.Update`、`Anim.Update` | 呈现层统一管理动画状态转换，对 Next 组合提供校验或诊断。 | L · [Updateable firing anim](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#updateable-firing-anim) |

### 地形对象

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH094 一般地形对象动画 | 解除树木等 TerrainType 动画必须生成矿石的限制，并允许指定动画长度；默认长度还受受损帧布局影响。 | `AnimationLength` | 呈现层扩展地形动画状态，生成矿石仍是独立模拟机制。 | L/S · [Animated TerrainTypes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#animated-terraintypes) |
| PH433 地形对象自定义调色板 | TerrainType 可指定不含扩展名及地形后缀的调色板；它按对象调色板工作，不继承所在地图格的地形染色。 | `Palette` | 资源层解析调色板，呈现层区分对象与地形光照路径。 | L/S · [Custom palette](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#custom-palette) |
| PH446 矿源种类与生成范围 | TerrainType 可指定矿物种类、扩散范围、生成阶段及每次动画填充格数，后两者可用闭区间随机值，并可关联粒子。 | `SpawnsTiberium.Type`、`SpawnsTiberium.Range`、`SpawnsTiberium.GrowthStage`、`SpawnsTiberium.CellsPerAnim`、`SpawnsTiberium.Particle` | 由矿物网格 owner 执行生成政策，保持随机抽取和占格规则。 | L/S · [Customizable ore spawners](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-ore-spawners) |
| PH495 地形受损帧与崩塌动画 | 地形对象可按黄血比例切换受损帧，死亡先播崩塌再移除，最后播放销毁特效；素材帧段须与动画长度正确配套。 | `ConditionYellow.Terrain`、`HasDamagedFrames`、`HasCrumblingFrames`、`CrumblingSound` | 模拟管理死亡和移除，呈现时序通过受控完成事件协调。 | L/S · [Damaged frames and crumbling animation](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#damaged-frames-and-crumbling-animation) |
| PH584 地形对象雷达颜色 | TerrainType 可配置独立 RGB 小地图颜色，使树木等地形对象在雷达上区别显示；与矿物类型的同名键分属不同对象。 | `MinimapColor` | 雷达呈现层按类型域解析，避免跨域误用同名键。 | L/S · [Minimap color customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#minimap-color-customization) |
| PH590 可通行与可覆盖建造地形 | 地形对象可允许穿过或在其上放建筑，后者放置成功后移除对象；矿树与普通地形可分别设置全局默认。 | `Terrain.IsPassable`、`Tibtree.IsPassable`、`Terrain.CanBeBuiltOn`、`Tibtree.CanBeBuiltOn`、`IsPassable`、`CanBeBuiltOn` | 寻路与建造资格共享策略，移除交由建造事务执行。 | L/S · [Passable & buildable-upon TerrainTypes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#passable-buildable-upon-terraintypes) |

### 动画

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH095 动画调色板与归属染色 | AltPalette 按动画所属方染色，支持调用方后备归属与光照；TheaterPalette 可选择地形调色板，并具有明确覆盖优先级。 | `AnimRemapDefaultColorScheme`、`AnimRemapUseInvokerFallback`、`AltPalette.ApplyLighting`、`AltPalette.UseInvokerFallback`、`TheaterPalette` | 建立动画样式和归属投影，避免 guest 持有调色板对象。 | L · [Animation palette customizations & improvements](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#animation-palette-customizations-improvements) |
| PH096 动画武器与伤害时序 | 动画可直接引爆武器，指定伤害归因、火力继承及每循环一次伤害；Damage.Delay 使用动画帧，不能当作游戏帧处理。 | `AnimDamage.DealtByInvoker`、`AnimDamage.ApplyFirepowerMult`、`Weapon`、`Damage.Delay`、`Damage.DealtByInvoker`、`Damage.ApplyOncePerLoop`、`Damage.ApplyFirepowerMult` | 需动画事件与受控引爆接口，保留调用方死亡后的阵营归因。 | L/S · [Animation weapon and damage settings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#animation-weapon-and-damage-settings) |
| PH101 附着动画定位 | 附着动画可选对象逻辑位置、视觉中心或中心的地面投影；地面投影仍保留附着关系，遮挡层级并不会自动变为地面对象。 | `AttachedAnimPosition` | 呈现层统一处理附着点、地面高度与遮挡排序。 | L/S · [Attached animation position customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#attached-animation-position-customization) |
| PH436 动画透明度与关键帧 | 为隐形对象上的附着动画指定单独透明度，并允许关键帧变化；插值最终仍钳制为 0、25、50、75 等合法档位。 | `Translucency.Cloaked` | 由动画呈现层插值并量化，不能把连续值当作任意透明度支持。 | L · [Customizable animation transparency settings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-animation-transparency-settings) |
| PH439 动画碎片与陨石落地行为 | 控制落水爆炸、完整弹头引爆、尾波与水花随机选择及额外影子；完整引爆与仅造成伤害、闪光是不同路径。 | `ExplodeOnWater`、`Warhead.Detonate`、`WakeAnim`、`SplashAnims`、`SplashAnims.PickRandom`、`ExtraShadow` | 分离碰撞模拟和落地表现，受控引爆需宿主能力。 | L/S · [Customizable debris & meteor impact and warhead detonation behaviour](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-debris-meteor-impact-and-warhead-detonation-behaviour) |
| PH479 平铺动画间距与锚点 | Tiled 动画可显式指定像素间距，或继续按首帧非透明区域估算，并可由底部中心改为画布中心对齐。 | `Tiled.Interval`、`Tiled.AlignToCenter` | 呈现层处理素材边界和锚点，不影响地图坐标。 | L · [Customize Tiled drawing interval and centering](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-tiled-drawing-interval-and-centering) |
| PH482 弹坑动画是否清除矿物 | 可禁止 Crater 动画摧毁所在格矿物，提供全局默认与 art 中单动画覆盖；不会因此取消弹坑其他行为。 | `AnimCraterDestroyTiberium`、`Crater.DestroyTiberium` | 模拟地形效果从动画事件中显式执行，不能由渲染结果决定。 | L · [Customize whether Crater=yes animation would destroy tiberium](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-crater-yes-animation-would-destroy-tiberium) |
| PH507 动画绘制偏移与选框锚定 | 动画可指定横向偏移，并按所属对象选框的宽高调整位置；可反转偏移方向，建筑还有独立附加修正。 | `XDrawOffset`、`XDrawOffset.ApplyBracketHeight`、`XDrawOffset.InvertBracketShift`、`XDrawOffset.BracketAdjust`、`XDrawOffset.BracketAdjust.Buildings`、`YDrawOffset.ApplyBracketHeight`、`YDrawOffset.InvertBracketShift` 等（9个不同示例键） | 由呈现层统一计算选框与动画锚点，使用像素单位。 | L · [Draw offset customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#draw-offset-customization) |
| PH518 燃烧动画派生火焰 | 重现并扩展 Scorch/Flamer 派生大小火焰，可配置数量、概率、散布、地形排除和附着；空中的 Flamer 不生成这些派生动画。 | `ConstrainFireAnimsToCellSpots`、`FireAnimDisallowedLandTypes`、`AttachFireAnimsToParent`、`SmallFireCount`、`SmallFireAnims`、`SmallFireChances`、`SmallFireDistances` 等（11个不同示例键） | 需动画事件与子效果生成能力，列表后备和随机次序须明确。 | L/S · [Fire animations spawned by Scorch & Flamer](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fire-animations-spawned-by-scorch-flamer) |
| PH582 附着动画层级 | 附着动画可选择跟随对象层或使用动画自身 Layer；未设置时采用地面层，不能把未设置简单等同于 false。 | `Layer.UseObjectLayer` | 呈现层保留可空布尔值并统一遮挡层解析，不以缺省代替否定。 | L/S · [Layer on animations attached to objects](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#layer-on-animations-attached-to-objects) |
| PH589 无矿隐藏的生长阶段门槛 | HideIfNoOre 动画可要求所在格矿物至少达到指定生长阶段才显示，区别于只检查有没有矿物。 | `HideIfNoOre.Threshold` | 呈现层读取矿物阶段快照，不能据显示状态改变矿物。 | L/S · [Ore stage threshold for HideIfNoOre](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#ore-stage-threshold-for-hideifnoore) |

### 步兵

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH102 AI 步兵自动部署 | 可恢复类似美国大兵的 AI 步兵交战时自动部署行为；支持全局默认及具体步兵覆盖，针对尤复中失效的原版路径。 | `InfantryAutoDeploy` | 在步兵战斗任务中加入部署政策，复用既有部署机制。 | L · [Auto deploy for GI-like infantry](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#auto-deploy-for-gi-like-infantry) |
| PH512 水中步兵副武器动画 | 默认让水中步兵副武器使用 WetAttack，避免错误使用陆地 SecondaryFire/SecondaryProne；可全局及按 art 恢复原版行为。 | `SecondaryFireSequenceLandOnly` | 呈现状态映射统一选择陆地、水中与主副武器序列。 | L · [Ensure infantry use correct firing sequences on water](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#ensure-infantry-use-correct-firing-sequences-on-water) |
| PH595 步兵卧倒移动速度 | 可按步兵类型覆盖卧倒速度倍率；未设置时按 Crawls 选择对应全局倍率，解除原版固定速度限制。 | `ProneSpeed.Crawls`、`ProneSpeed.NoCrawls`、`ProneSpeed` | 统一速度 owner 消费姿态与类型政策，保留 Crawls 默认分支。 | L · [Prone speed customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#prone-speed-customization) |

### 飞机

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH424 运输机吊运语音 | Carryall 飞机接到拾取车辆命令时可播放独立 VoicePickup，替代普通移动语音，不修改实际吊运任务。 | `VoicePickup` | 在命令确认的音频事件层处理，并复用载入的声音注册表。 | L/S · [Carryall pickup voice](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#carryall-pickup-voice) |
| PH447 空投间隔与离场延迟 | 空投单位间隔及最后一个单位落下后的等待可全局或逐机型配置；末尾延迟为负时飞机直线飞离，忽略常规地图边缘规则。 | `ParadropDelay`、`ParadropEndDelay` | 在空投任务状态机维护计时与离场分支，保存当前卸载进度。 | L · [Customizable paradrop delay](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-paradrop-delay) |
| PH478 飞机攻击导致目标格散开 | 可禁止飞机攻击任务强制让目标所在地图格的单位散开，支持全局和机型覆盖；不改变飞机武器本身的伤害。 | `AircraftFiringForceScatter`、`FiringForceScatter` | 在攻击任务提交散开命令前检查政策，保留散开机制唯一入口。 | L · [Customize the scatter caused by aircraft attack mission](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-the-scatter-caused-by-aircraft-attack-mission) |
| PH481 飞机 IsALoaner 覆盖 | 飞机类型可显式决定出场时 IsALoaner 标志，未设置时保留引擎原有的货运飞机等分类判断。 | `IsALoaner` | 由飞机出场 owner 应用类型政策，追踪该标志的各任务消费者。 | L · [Customize whether aircraft is a cargo plane](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customize-whether-aircraft-is-a-cargo-plane) |
| PH516 飞机扩展任务 | 飞机支持路径点、警戒与攻击移动，并可提前下降、调整进场和无机场损伤；启用后需持续拥有可用停机位，任务预置飞机尤其需校验。 | `ExtendedAircraftMissions`、`ExtendedAircraftMissions.UnlandDamage`、`ExtendedAircraftMissions.SmoothMoving`、`ExtendedAircraftMissions.EarlyDescend`、`ExtendedAircraftMissions.RearApproach`、`ExtendedAircraftMissions.FastScramble` | 需飞机任务与停靠生命周期接口，是完整机制链而非单个 tick 回调。 | L · [Extended Aircraft Missions](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#extended-aircraft-missions) |
| PH571 按机型覆盖 CurleyShuffle | CurleyShuffle 可从全局设置细化到具体飞机类型；官方本节将详细行为指向外部参考，不能仅从键名推定新增任务能力。 | `CurleyShuffle` | 先核对飞机攻击机制，再在既有 owner 增加类型覆盖。 | L · [Implement CurleyShuffle for AircraftTypes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#implement-curleyshuffle-for-aircrafttypes) |
| PH580 飞机自身降落朝向 | 飞机类型可指定降落方向，但建筑停机位设置优先；非机场绑定飞机允许负值表示按当前朝向降落。 | `LandingDir` | 停靠和野外降落共享明确的朝向优先级解析。 | L/S · [Landing direction](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#landing-direction) |
| PH599 增援飞机生成位置 | 空袭、侦察和空投飞机可指定进出地图边、距目标固定距离和初始高度，且出生朝向目标；负生成距离反转来袭方向。 | `AircraftSpawnFromEdge`、`AircraftRetreatToEdge`、`SpawnFromEdge`、`RetreatToEdge`、`SpawnDistanceFromTarget`、`SpawnHeight` | 增援任务统一选择边界与坐标，保留来源和同步随机。 | L · [Reinforcement aircraft spawn settings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#reinforcement-aircraft-spawn-settings) |

### 国家与阵营

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH438 按国家指定逃生乘员 | 国家可以覆盖所属阵营的 Crew 步兵类型，为同阵营不同国家提供独立逃生乘员；未设置时继承 Side 定义。 | `Crew` | guest 解析国别覆盖，由逃生生成机制消费最终类型。 | L · [Customizable crew type per country](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-crew-type-per-country) |

### 体素碎片动画

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH440 体素碎片拖尾间隔 | 可为 VoxelAnim 配置拖尾动画实例的生成间隔，单位为游戏帧；不直接改变碎片飞行速度或拖尾动画自身帧速。 | `Trailer.SpawnDelay` | 在体素碎片呈现状态中维护拖尾节奏，保留游戏帧单位。 | L · [Customizable debris trailer anim spawn delay](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-debris-trailer-anim-spawn-delay) |
| PH570 体素碎片落地行为 | 体素碎片可采用与普通碎片动画相近的落水和弹头引爆配置；关键区别是键写在 rulesmd.ini 的 VoxelAnim 节，而非 art。 | 无独立示例键；见原文条件 | 复用落地效果决策内核，保留 VoxelAnim 独立资源解析位置。 | L/S · [Customizable debris & meteor impact and warhead detonation behaviour](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#id2) |

### 粒子

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH443 毒气粒子漂移速度 | BehavesLike=Gas 的粒子可指定最大漂移速度，改变原版固定的漂移表现；不是通用粒子系统速度开关。 | `Gas.MaxDriftSpeed` | 由对应粒子更新机制消费配置，区分表现与伤害步进。 | L/S · [Customizable gas particle speed](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#customizable-gas-particle-speed) |

### 粒子系统

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH519 旋转时火焰粒子终点 | Fire 粒子系统可禁止随着创建它的射手旋转而偏移目标坐标，避免持续火焰终点跟转；默认保留原行为。 | `AdjustTargetCoordsOnRotation` | 粒子系统呈现 owner 消费发射起点、终点及跟随政策。 | L/S · [Fire particle target coordinate adjustment when firer rotates](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fire-particle-target-coordinate-adjustment-when-firer-rotates) |

### 矿物

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH568 矿物在斜坡生长扩散 | 矿物类型可允许在斜坡上生长和扩散，解除原版禁止并清除斜坡矿物的限制；需要按矿物类型启用。 | `AllowRamps` | 由矿物网格更新 owner 应用坡面资格，不改纯显示层。 | L · [Grow and spread on slopes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#grow-and-spread-on-slopes) |
| PH569 矿物雷达颜色 | 可为每种矿物指定 RGB 小地图颜色，使不同矿物在雷达上以独立色彩显示，不再共用固定表现。 | `MinimapColor` | 雷达呈现层读取矿物类型与冻结颜色，保留地图可见性限制。 | L/S · [Minimap color customization](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#id1) |

### 覆盖物

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH585 扩展覆盖物类型数量 | 支持的 OverlayType 从最多二百五十五种扩展到六万五千五百三十五种；地图文件和编辑器还需要对应的扩展格式支持。 | 无独立示例键；见原文条件 | 资源、地图序列化和运行时 ID 必须统一扩容，不能只改注册表。 | L · [More than 255 OverlayTypes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#more-than-255-overlaytypes) |
| PH628 覆盖物绘制深度 | OverlayType 可从 art 的图像节读取 ZAdjust，手动调整覆盖物绘制深度，解决特定素材的遮挡需求。 | `ZAdjust` | 在覆盖物呈现层消费偏移，不改变逻辑占格或碰撞。 | L · [ZAdjust for OverlayTypes](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#zadjust-for-overlaytypes) |

### 通用说明

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH587 全局设置分类说明 | 本分类收录无需逐个对象定义即可生效的全局设置；它是后续具体主题的组织说明，不是额外的独立游戏功能。 | 无独立示例键；见原文条件 | 配置层集中管理全局默认，具体能力仍由各机制 owner 消费。 | L/S · [Newly added global settings](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#newly-added-global-settings) |

<a id="ai"></a>
## AI脚本与地图

本节62个具名主题。

### 小队

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH001 小队解散后恢复可招募标记 | 原版把单位加入小队时覆盖可招募标记，离队后不恢复；新设置允许强制恢复为可招募或不可招募，负数保留原行为，并支持小队覆盖全局。 | `SetRecruitableOnLiberate` | 由小队成员关系所有者在释放时更新原有标记，避免插件另存副本。 | L · [Adjust recruitable status on team member liberate](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#adjust-recruitable-status-on-team-member-liberate) |
| PH024 小队专属空降运输机 | Droppod=yes小队可用ParaDropAircraft覆盖原版和Ares的全局运输机类型。 | `ParaDropAircraft` | 空降任务读取最终类型绑定，生成和乘员装载由宿主统一负责。 | L · [Customized transport plane for teams](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#customized-transport-plane-for-teams) |

### 地图触发事件

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH002 事件606：附加效果施加 | 在触发器启用后，目标获得或重施附加效果时触发；自身效果的首次延迟、重建延迟也影响事件时机，启用前已有的效果不会补触发。 | 无独立示例键；见原文条件 | 先有附加效果生命周期事件，再接地图触发系统；仅查询状态不够。 | L · [606 AttachEffect is attaching to a Techno](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#attacheffect-is-attaching-to-a-techno) |
| PH020 事件604—605：指定对象入格 | 604检查进入单元格的对象类型，605检查其是否在AITargetTypes列表；阵营参数可选任意阵营、指定阵营或触发器所属阵营。 | 无独立示例键；见原文条件 | 由地图进入格事件提供对象与阵营上下文，插件只作过滤。 | L/S · [604-605 Checking if a specific Techno enters in a cell](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#checking-if-a-specific-techno-enters-in-a-cell) |
| PH037 事件601—602：阵营拥有类型 | 检查指定阵营是否拥有至少一个、或完全没有指定类型对象；与Ares81/82不同，阵营由参数选择，原文多人编码只有八个位置。 | 无独立示例键；见原文条件 | 查询阵营对象计数的权威索引，30人编码须另作兼容映射。 | L/S · [601-602 House owns/doesn’t own Techno Type](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#house-owns-doesn-t-own-techno-type) |
| PH067 事件600：关联对象护盾破裂 | 关联对象的可再生护盾破裂时触发，与强制力场超武或普通生命值归零不同。 | 无独立示例键；见原文条件 | 护盾guest输出语义事件，经宿主验证后进入地图事件总线。 | L/S · [600 The shield of the attached object is broken](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#the-shield-of-the-attached-object-is-broken) |
| PH072 事件500—511：变量与常量比较 | 分别对局部和全局变量执行大于、小于、等于、大于等于、小于等于或按位与检查；按位与不是逻辑与。 | 无独立示例键；见原文条件 | 在场景变量查询器实现相同位宽与比较语义，事件只读。 | L/S · [500-511 Variable comparation](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#variable-comparation) |
| PH073 事件524—535：与全局变量比较 | 将目标局部或全局变量与另一个全局变量比较，比较和位运算种类对应500—511。 | 无独立示例键；见原文条件 | 共享变量事件比较器，明确左右操作数的作用域。 | L/S · [524-535 Variable comparation with global variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#variable-comparation-with-global-variable) |
| PH074 事件512—523：与局部变量比较 | 将目标局部或全局变量与另一个局部变量比较，比较和位运算种类对应500—511。 | 无独立示例键；见原文条件 | 共享变量事件比较器，避免按编号各复制一份逻辑。 | L/S · [512-523 Variable comparation with local variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#variable-comparation-with-local-variable) |

### AI脚本行动

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH003 脚本10000—10049：攻击行动族 | 按内置目标类别或AITargetTypes列表选目标，区分远近、威胁和单次或重复攻击；正文实际列出10000—10017。飞机攻击空中目标后会结束脚本。 | 无独立示例键；见原文条件 | 目标枚举、威胁和寻路保留宿主所有者，策略与完成条件可外置。 | L/S · [10000-10049 Attack Actions](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#attack-actions) |
| PH021 脚本10104：超时空突袭基地 | 借首个可用超时空传送超武，把小队送往敌方基地附近；距离在AISafeDistance基础上调整，充能须达到AIMinorSuperReadyPercent。 | 无独立示例键；见原文条件 | 经超武领域提交使用请求，复用就绪、目标和传送有效性检查。 | L/S · [10104 Chronoshift to Enemy Base](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#chronoshift-to-enemy-base) |
| PH025 脚本14002：降低AI权重 | 降低当前AI触发器的权重，仍服从其最小和最大权重限制，参数表示变化量。 | 无独立示例键；见原文条件 | 复用AI触发器唯一权重所有者，与增加权重使用同一算术路径。 | L/S · [14002 Decrease AI Trigger Current Weight](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#decrease-ai-trigger-current-weight) |
| PH030 脚本18000—18023：变量运算 | 脚本参数低16位存变量索引，高16位存操作数；与触发动作的32位变量操作有区别，此处文档限定16位有符号数值。 | 无独立示例键；见原文条件 | 读取器明确打包和符号扩展规则，再由场景变量所有者执行。 | L/S · [18000-18023 Edit Variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#edit-variable) |
| PH031 脚本18048—18071：全局变量操作数 | 沿用脚本变量操作族，但操作数改从全局变量读取；低16位为目标索引，高16位为来源全局变量索引。 | 无独立示例键；见原文条件 | 与基础变量操作共享执行器，仅替换操作数来源。 | L/S · [18048 - 18071 Edit Variable using Global Variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#edit-variable-using-global-variable) |
| PH032 脚本18024—18047：局部变量操作数 | 沿用脚本变量操作族，但操作数改从局部变量读取；参数打包两个变量索引。 | 无独立示例键；见原文条件 | 与基础变量操作共享执行器，读写顺序和位宽须固定。 | L/S · [18024 - 18047 Edit Variable using Local Variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#edit-variable-using-local-variable) |
| PH035 脚本14004：强制敌方阵营过滤 | 覆盖Phobos新增攻击和移动脚本的OnlyTargetHouseEnemy策略，可恢复未覆盖、强制开关或同步随机选择；不改变所有原版脚本。 | 无独立示例键；见原文条件 | 策略上下文绑定小队及脚本执行周期，随机取值必须可重放。 | L · [14004 Force Global OnlyTargetHouseEnemy value in Teams for new attack / move actions introduced by Phobos](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#force-global-onlytargethouseenemy-value-in-teams-for-new-attack-move-actions-introduced-by-phobos) |
| PH039 脚本14001：提高AI权重 | 执行时立即增加AI触发器当前权重并夹在上下限内；同一触发器的多个小队都可能更新该权重，不只原版计算时采用的首个小队。 | 无独立示例键；见原文条件 | 只有AI触发器拥有权重，小队提交调整请求，避免各存一份。 | L/S · [14001 Increase AI Trigger Current Weight](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#increase-ai-trigger-current-weight) |
| PH041 脚本16005：返回前一脚本 | 供16004随机进入的脚本使用，返回调用脚本并继续执行16004之后一行。 | 无独立示例键；见原文条件 | 由AI脚本执行器保存明确的返回位置，存档包含调用上下文。 | L/S · [16005 Jump Back To Previous Script](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#jump-back-to-previous-script) |
| PH042 脚本10103：同队乘员装载 | 让TaskForce中可搭乘的单位进入同队运输工具；单人任务后面还须接43,0等待满载，脚本才会正常继续。 | 无独立示例键；见原文条件 | 装载仍走运输关系所有者，脚本只调度并等待结果。 | L/S · [10103 Load onto Transports](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#load-onto-transports) |
| PH044 脚本19000—19999：预留分类 | 官方将该编号段归为未分类脚本，但当前正文明确为空，不能据此宣称一千个动作已经可用。 | 无独立示例键；见原文条件 | 保留编号命名空间；仅实现并注册有具体行为契约的编号。 | L/S · [19000-19999 Miscellanous/Uncategorized](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#miscellanous-uncategorized) |
| PH045 脚本12001：临时接近距离 | 为下一次移动到对象行动覆盖CloseEnough，执行一次后后续移动恢复原设定。 | 无独立示例键；见原文条件 | 在脚本上下文保存一次性参数，消费后清除并参与存档。 | L/S · [12001 Modify Target Distance](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#modify-target-distance) |
| PH046 原版脚本的可读目标参数 | 原版46、47、56、58可用建筑类型名和near、far、high、low扫描模式，免去人工计算建筑索引与偏移。 | 无独立示例键；见原文条件 | 在同一脚本读取器兼容新旧参数，执行仍复用原版行动。 | L · [More convenient vanilla action script target specification](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#more-convenient-vanilla-action-script-target-specification) |
| PH047 脚本10050—10099：移动到对象族 | 按目标类别或自定义列表靠近敌我对象，可按远近、威胁或随机选取；正文实际列出10050—10061。 | 无独立示例键；见原文条件 | 复用目标与移动所有者，策略结果必须通过同一可达性检查。 | L/S · [10050-10099 Move Team to Techno Location actions](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#move-team-to-techno-location-actions) |
| PH048 脚本16004：随机切换脚本 | 按AIScriptsList中从零开始的列表索引随机选择一个ScriptType，替换小队当前脚本，可用16005返回。 | 无独立示例键；见原文条件 | 冻结类型列表，宿主同步RNG选择，执行器保存返回上下文。 | L/S · [16004 Pick a Random Script](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#pick-a-random-script) |
| PH050 脚本16003：随机跳过下一步 | 以参数给出的百分概率跳过下一条脚本行动；0从不跳，100总会跳，正文的范围提示应结合这两端语义读取。 | 无独立示例键；见原文条件 | 统一同步随机抽样规则及抽取次数，结果进入脚本执行状态。 | L/S · [16003 Randomly Skip Next Action](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#randomly-skip-next-action) |
| PH051 脚本10102：队长周围集结 | 在给定时间内围绕队长执行区域警戒，队长基本停留，范围使用CloseEnough；不同于原地等待。 | 无独立示例键；见原文条件 | 小队调度与各单位任务所有者协作，明确队长失效的结束规则。 | L/S · [10102 Regroup Temporarily Around the Team Leader](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#regroup-temporarily-around-the-team-leader) |
| PH057 脚本12002：移动结束条件 | 下一次移动行动可在队长、任一成员或全部成员达到最小距离时结束；是一次性覆盖，后续行动恢复默认。 | 无独立示例键；见原文条件 | 小队执行器保存一次性结束策略，移动所有者提供完成状态。 | L/S · [12002 Set Move Action End Mode](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-move-action-end-mode) |
| PH063 脚本16001：定时进入下一步 | 计时结束时终止当前行动并开始下一条，时间采用游戏秒。 | 无独立示例键；见原文条件 | 脚本执行器维护模拟帧计时，明确当前行动取消清理。 | L/S · [16001 Start a Timed Jump to the Next Line](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#start-a-timed-jump-to-the-next-line) |
| PH064 脚本16000：定时重复本步 | 计时到期重新开始当前行动并不断重复，直到16002停止或小队销毁；使用游戏秒。 | 无独立示例键；见原文条件 | 在脚本执行器维护计时和重启语义，避免重新创建竞争计时器。 | L/S · [16000 Start a Timed Jump to the Same Line](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#start-a-timed-jump-to-the-same-line) |
| PH065 脚本16002：停止定时跳转 | 停止此前16000或16001建立的定时跳转过程。 | 无独立示例键；见原文条件 | 由同一脚本计时所有者取消订阅与待执行跳转。 | L/S · [16002 Stop the Timed Jumps](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#stop-the-timed-jumps) |
| PH066 脚本14000：成功攻击权重奖励 | 为后续新增攻击行动登记奖励；小队成员击杀指定目标后增加AI触发器权重，仍受最小和最大值限制。 | 无独立示例键；见原文条件 | 把奖励绑定行动目标和击杀事件，防止重复结算。 | L/S · [14000 Team’s Trigger Weight Reward](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#team-s-trigger-weight-reward) |
| PH068 脚本10100：定时区域警戒 | 按游戏秒执行区域警戒，可主动对附近威胁作出反应；与原版5,n停留在原地的警戒行动不同。 | 无独立示例键；见原文条件 | 复用单位区域警戒任务，小队执行器只负责持续时间。 | L/S · [10100 Timed Area Guard](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#timed-area-guard) |
| PH071 脚本14003：撤销小队成功 | 与原版49,0登记成功相反，使小队按未成功处理。 | 无独立示例键；见原文条件 | 复用AI小队成功标记及触发器结算机制。 | L/S · [14003 Unregister Team Success](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#unregister-team-success) |
| PH075 脚本12000：无目标时重试 | 为后续新增攻击行动设置无目标时按一秒间隔重试的次数；0为无限重试，达到限制才跳下一步。 | 无独立示例键；见原文条件 | 小队保存剩余次数与模拟计时，不阻塞游戏主线程。 | L/S · [12000 Wait if No Target Found](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#wait-if-no-target-found) |
| PH076 脚本10101：等待弹药满载 | 小队含有弹药单位时，等待这些单位全部补满才继续脚本。 | 无独立示例键；见原文条件 | 查询引擎原有弹仓状态；Ares武器Ammo成本与弹仓容量分开处理。 | L/S · [10101 Wait Until Ammo is Full](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#wait-until-ammo-is-full) |

### 单人任务地图

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH004 任务AI基地节点维修 | 战役中可控制AI是否维修基地节点和由超武交付的建筑；阵营的三难度配置覆盖全局开关。 | `RepairBaseNodes` | 由AI维修所有者消费统一配置，与建筑归属和超武交付接通。 | L/S · [Base node repairing](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#base-node-repairing) |
| PH026 默认加载画面与简报偏移 | missionmd.ini可提供战役加载图、调色板与简报偏移后备值；图像或地图配置缺失时使用。DefaultLS800BkgdPal也适用于640图。 | `DefaultLS640BriefLocX`、`DefaultLS640BriefLocY`、`DefaultLS800BriefLocX`、`DefaultLS800BriefLocY`、`DefaultLS640BkgdName`、`DefaultLS800BkgdName`、`DefaultLS800BkgdPal` | 由任务元数据与资源加载器实现后备链，不放入对象tick。 | L/S · [Default loading screen and briefing offsets](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#default-loading-screen-and-briefing-offsets) |
| PH043 战役MCV重新部署 | 地图Basic节可决定战役中的MCV是否允许重新部署，覆盖该模式下MultiplayerDialogSettings的同名设置。 | `MCVRedeploys` | 冻结模式最终配置，再由部署规则所有者查询。 | L/S · [MCV redeploying](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#mcv-redeploying) |
| PH059 missionmd中的任务评级参数 | 可在任务条目定义三难度标准用时及达标或超时文案，优先于地图Ranking节，未定义项仍回退到地图。 | `Ranking.ParTimeEasy`、`Ranking.ParTimeMedium`、`Ranking.ParTimeHard`、`Ranking.UnderParTitle`、`Ranking.UnderParMessage`、`Ranking.OverParTitle`、`Ranking.OverParMessage` | 任务元数据加载器逐项合并，保留时间字符串解析语义。 | L/S · [Set par times and related string labels in missionmd.ini](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-par-times-and-related-string-labels-in-missionmd-ini) |
| PH061 开局简报对话框 | 战役开局可显示简报并配置音乐和按钮文字；地图ShowBriefing优先于missionmd，但玩家RA2MD设置可关闭，主题还有阵营后备层。 | `ShowBriefing`、`BriefingTheme`、`ShowBriefingResumeButtonLabel`、`ShowBriefingResumeButtonStatusLabel` | 任务启动流程负责时机与音频，区分作品默认和玩家偏好。 | L/S · [Show briefing dialog on startup](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#show-briefing-dialog-on-startup) |
| PH062 SkipMapSelect跨阵营后续任务 | 跳过地图选择时可绕过原版下一关阵营匹配限制，使用NextScenario或AltNextScenario；特定名称的局部变量也可选备用关。 | 无独立示例键；见原文条件 | 由战役分支所有者统一解析胜利后的目标与条件。 | L · [SkipMapSelect Enhancement](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#skipmapselect-enhancement) |

### 地图触发动作

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH005 动作504：变量间运算 | 以另一个变量的值作为操作数，对指定变量执行与动作501同族的算术或位运算，并分别指定两变量属于局部还是全局。 | 无独立示例键；见原文条件 | 扩展场景变量所有者的校验命令，明确位宽与除零行为。 | L/S · [504 Binary Operation](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#binary-operation) |
| PH022 动作607：清除仇恨 | 清除触发器阵营对指定阵营的仇恨值；参数-1作用于所有阵营。 | 无独立示例键；见原文条件 | 由阵营关系所有者修改仇恨，保持强制敌人策略的独立优先级。 | L · [607 Clear Hate-Value](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#clear-hate-value) |
| PH023 动作600：覆盖掉落箱 | 为关联对象设置、清空或恢复默认DropCrate类型；仅用于挂在对象上的触发器。 | 无独立示例键；见原文条件 | 在对象死亡掉落策略处接入覆盖值，并保存触发器的覆盖状态。 | L · [600 Configure Drop Crate](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#configure-drop-crate) |
| PH027 动作800—802：屏幕横幅 | 800/801按局部或全局变量创建或替换同ID横幅，802删除；支持PCX、SHP或文本，并配置持续、重现、锚点和屏幕约束。 | `PCX`、`SHP`、`SHP.Palette`、`SHP.RefreshAfterDelay`、`CSF`、`CSF.Color`、`CSF.Background` 等（13个不同示例键） | 引擎管理横幅实例与资源，场景命令引用类型和实例ID，呈现不反写模拟。 | L · [800-802 Display Banner](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#display-banner) |
| PH029 动作606：编辑仇恨值 | 修改触发器阵营对目标阵营的仇恨，支持赋值、四则、取模、移位和位运算；-1选择所有阵营。 | 无独立示例键；见原文条件 | 由阵营关系所有者执行已校验操作，定义溢出和移位范围。 | L · [606 Edit Hate-Value](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#edit-hate-value) |
| PH033 动作505：向坐标发射超武 | 按SuperWeaponTypes索引和阵营选择向地图坐标发射；阵营可随机选择，坐标-1从可见地图范围取随机值。旧多人阵营编码仅列4475—4482。 | 无独立示例键；见原文条件 | 经超武所有者校验发射；所有随机选择用同步RNG，另设计30人阵营映射。 | L/S · [505 Fire Super Weapon at specified location](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#fire-super-weapon-at-specified-location) |
| PH034 动作506：向路径点发射超武 | 使用指定阵营的超级武器攻击路径点，超武以列表索引指定，阵营选择规则参照坐标发射动作。 | 无独立示例键；见原文条件 | 复用505的超武命令入口，先把路径点解析为合法坐标。 | L/S · [506 Fire Super Weapon at specified Waypoint](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#fire-super-weapon-at-specified-waypoint) |
| PH036 动作502：生成随机整数 | 从包含两端点的范围生成随机整数，写入指定局部或全局变量。 | 无独立示例键；见原文条件 | 通过宿主同步RNG与变量命令实现，禁止读取系统随机源。 | L/S · [502 Generate random number](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#generate-random-number) |
| PH038 动作501：编辑变量 | 对局部或全局32位有符号变量执行赋值、算术、移位及位运算；超范围数值不能按普通无限精度整数处理。 | 无独立示例键；见原文条件 | 在场景变量所有者定义精确数值语义，异常输入拒绝或按兼容规则处理。 | L/S · [501 Edit Variable](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#id1) |
| PH049 动作503：打印变量 | 把指定局部或全局变量的值输出到游戏消息列表。 | 无独立示例键；见原文条件 | 场景读取值，交给表现消息通道；消息显示不得改变变量。 | L/S · [503 Print variable value](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#print-variable-value) |
| PH052 动作500：立即存档 | 立即保存游戏并组合地图说明与CSF文字为描述；上游多人模式还依赖支持多人存档的YRpp spawner。 | 无独立示例键；见原文条件 | VERA需接自身对局一致性与存档协调流程，不能照搬DLL依赖当作已支持。 | L/S · [500 Save Game](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#save-game) |
| PH053 动作512：设置关联车辆跟随者 | 为关联车辆设置Follower，参数是被跟随关系中的车辆索引；非车辆对象无效。 | 无独立示例键；见原文条件 | 由车辆跟随关系所有者校验引用，并处理销毁和读档重绑定。 | L · [512 Set Follower for Associated Unit](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-follower-for-associated-unit) |
| PH054 动作608：固定敌对目标 | 强制阵营敌人，不再随仇恨值改变；-1解除固定敌人，-2表示永不拥有敌人。 | 无独立示例键；见原文条件 | 在阵营外交策略中规定覆盖优先级，状态由关系所有者保存。 | L · [608 Set Force Enemy](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-force-enemy) |
| PH055 动作610：设置组队间隔 | 修改触发器阵营的TeamDelays，负数退回全局或动态规则；在该阵营下次创建小队后开始使用新值。 | 无独立示例键；见原文条件 | 由AI调度所有者记录待生效值和明确生效阶段。 | L · [610 Set house’s TeamDelays value](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-house-s-teamdelays-value) |
| PH056 动作513：任务计时显示 | 可切换普通、百分比、数字或局部/全局变量显示并支持反向显示；变量缺失显示0，显示方式不会改变任务计时实际到期。 | 无独立示例键；见原文条件 | 任务计时仍由模拟拥有，界面只读取显示模型。 | L · [513 Set mission timer properties](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-mission-timer-properties) |
| PH058 动作611：指定下一任务 | 仅用于战役且要求SkipMapSelect=yes，胜利后加载指定地图。 | 无独立示例键；见原文条件 | 经战役流程所有者设置下一关，不授予guest任意文件读取权。 | L · [611 Set Next Scenario](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-next-scenario) |
| PH060 动作609：雷达模式 | 修改触发器阵营的雷达，可恢复正常供电条件、使用FreeRadar或强制开关。 | 无独立示例键；见原文条件 | 雷达可用性由阵营能力所有者统一计算，UI读取结果。 | L · [609 Set Radar Mode](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#set-radar-mode) |
| PH069 动作510：切换MCV重部署 | 通过动作参数强制设置MCV能否重新部署。 | 无独立示例键；见原文条件 | 修改对局规则覆盖值，部署消费者查询同一权威结果。 | L/S · [510 Toggle MCV redeployablility](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#toggle-mcv-redeployablility) |
| PH070 动作511：建筑收起并移向路径点 | 让指定阵营和类型的建筑收起成车辆并移往路径点，类型参数可用<All>选择全部建筑。 | 无独立示例键；见原文条件 | 部署转换与移动命令必须经既有所有者，失败不留下半转换状态。 | L · [511 Undeploy Building to Waypoint](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#undeploy-building-to-waypoint) |

### 其他地图与调度能力

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH028 按参战情况调整AI组队间隔 | 可按起始点、玩家、盟友、敌人及存活数量选择TeamDelays；排除中立与观察者，CountN在文档中仅定义1—8，修改在下一次建队后生效。 | `TeamDelays.DynamicType`、`TeamDelays.CountN` | 扩展AI调度策略时显式设计30人映射，不能机械沿用8人索引。 | L · [Dynamic Team Delays](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#dynamic-team-delays) |
| PH040 扩展Overlay索引 | 地图NewINIFormat=5允许OverlayType索引最高65535；这种地图不再能被原版加载，编辑器及打包工具也须支持新格式。 | 无独立示例键；见原文条件 | 统一地图编解码和资源索引位宽，再验证编辑器与客户端分发链。 | L · [Increased Overlay Limit](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#increased-overlay-limit) |

<a id="ui"></a>
## 战斗界面与侧栏

本节50个具名主题。

### 战场界面

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH825 单人聊天输入框 | 可在单人场景开放原本禁用的游戏内聊天输入框。 | `AllowChatBoxInSinglePlayer` | 输入和消息界面处理，开发命令须另行校验权限。 | L · [Allow chat box in singleplayer](https://phobos.readthedocs.io/en/latest/User-Interface.html#allow-chat-box-in-singleplayer) |
| PH826 超级武器百分比计时 | 超级武器ShowTimer可全局或按类型改为百分比显示；不会单凭此项解决多人计时器数量过多。 | `SuperWeaponTimer.Percentage`、`ShowTimer.Percentage` | 计时数据从超武所有者只读获取，布局与筛选在UI层处理。 | L · [Allow draw SuperWeapon timer as percentage](https://phobos.readthedocs.io/en/latest/User-Interface.html#allow-draw-superweapon-timer-as-percentage) |
| PH840 生命条显示策略 | 可隐藏整个生命条及外框、只藏格子，或常显生命和附加格子/组号；这些显示策略不改变实际血量。 | `HealthBar.Hide`、`HealthBar.HidePips`、`HealthBar.Permanent`、`HealthBar.Permanent.PipScale` | 呈现层读取生命快照，SDK通用Meter不足以直接覆盖全部原版布局。 | L · [Custom health bars display](https://phobos.readthedocs.io/en/latest/User-Interface.html#custom-health-bars-display) |
| PH842 资金显示步长上限 | 调整资金指示器每帧最大变化量；非正数移除上限，但若平滑仍开启，显示依旧按差值渐近逼近实际资金。 | `CreditsIndicator.MaxStep` | 资金真实值由经济层拥有，显示插值在侧栏独立处理。 | L · [Customize the step limit of the credits indicator](https://phobos.readthedocs.io/en/latest/User-Interface.html#customize-the-step-limit-of-the-credits-indicator) |
| PH846 对象数字显示 | 定义数值显示类型，可展示生命、护盾、弹药、生产或超武状态；支持数字图像或文字、百分比/计时、阵营可见性和锚点。 | `Buildings.DefaultDigitalDisplayTypes`、`Infantry.DefaultDigitalDisplayTypes`、`Vehicles.DefaultDigitalDisplayTypes`、`Aircraft.DefaultDigitalDisplayTypes`、`DigitalDisplay.Health.FakeAtDisguise`、`InfoType`、`InfoIndex` 等（32个不同示例键） | 先提供只读属性查询和表现资源协议；通用Meter并非完整数字显示兼容。 | L/S · [Digital display](https://phobos.readthedocs.io/en/latest/User-Interface.html#digital-display) |
| PH847 关闭资金渐近显示 | 关闭平滑后，资金显示每步按MaxStep追赶；若要立即跳到实际值，还须把MaxStep设为非正数。 | `CreditsIndicator.Smooth` | 复用同一侧栏显示状态，避免经济层受显示插值影响。 | L · [Disable the credits indicator smooth transition](https://phobos.readthedocs.io/en/latest/User-Interface.html#disable-the-credits-indicator-smooth-transition) |
| PH850 选中对象闪光 | 对玩家控制的对象，选中时可闪光指定帧数；作品提供效果后玩家仍可关闭。 | `SelectionFlashDuration`、`ShowFlashOnSelecting` | 选择事件触发表现计时，避免改变对象模拟订阅。 | L/S · [Flashing Technos on selecting](https://phobos.readthedocs.io/en/latest/User-Interface.html#flashing-technos-on-selecting) |
| PH853 框选低优先级 | 框内有普通单位时可排除低优先级对象；仅有低优先级时正常选择，追加选择按新增池判断，玩家可关闭过滤。 | `LowSelectionPriority`、`PrioritySelectionFiltering` | 选择控制器统一实现普通、同型与追加框选条件。 | L/S · [Low priority for box selection](https://phobos.readthedocs.io/en/latest/User-Interface.html#low-priority-for-box-selection) |
| PH855 建筑放置预览 | 支持建筑预览、透明度、帧与剧场资源；缺省取建造动画末个非阴影帧，否则取主体首帧，不包括附属动画和bib。 | `PlacementPreview`、`PlacementPreview.Translucency`、`PlacementGrid.Translucency`、`PlacementGrid.TranslucencyWithPreview`、`PlacementPreview.Shape`、`PlacementPreview.ShapeFrame`、`PlacementPreview.Offset` 等（10个不同示例键） | 由建筑资源与放置呈现链实现；放置合法性仍由模拟所有者判断。 | L/S · [Placement preview](https://phobos.readthedocs.io/en/latest/User-Interface.html#placement-preview) |
| PH857 保留CSF空白 | 可跳过原版合并连续空格及去除首尾空白的规范化，保留简报等文本的排版。 | `PreserveCSFWhitespace` | 文本加载器保留原始值和显示策略，不能用普通trim替代全部读取规则。 | L · [Preserve whitespace in CSF](https://phobos.readthedocs.io/en/latest/User-Interface.html#preserve-whitespace-in-csf) |
| PH860 现实秒计时显示 | 按目标帧率换算计时，可自适应实际FPS；无限帧及自定义速度强制自适应，无限帧或超60FPS自定义速度时显示可能波动。 | `RealTimeTimers`、`RealTimeTimers.Adaptive` | 本地时间换算只影响呈现，模拟计时保持固定帧语义。 | L/S · [Real time timers](https://phobos.readthedocs.io/en/latest/User-Interface.html#real-time-timers) |
| PH862 可定制选择框 | 为步兵、车辆、飞机及指定建筑定制选择图、地面投影和连线，可按血量换帧并限定阵营/观察者可见性。 | `DefaultInfantrySelectBox`、`DefaultUnitSelectBox`、`Shape`、`Palette`、`Frames`、`Offset`、`Translucency` 等（23个不同示例键） | 表现层绑定选框资源并读取高度、血量与可见性，避免泄露不可见状态。 | L · [Select Box](https://phobos.readthedocs.io/en/latest/User-Interface.html#select-box) |
| PH864 选建筑切换侧栏分类 | 选择工厂可自动切到相应标签，建筑还能指定标签；负值自动推断，0—3选具体页，≥4不操作。 | `SetTabBySelectingFactory`、`SetTabBySelecting` | 输入/选择层通知侧栏，生产类别仍查询工厂类型。 | L · [Set sidebar tab by selecting factory](https://phobos.readthedocs.io/en/latest/User-Interface.html#set-sidebar-tab-by-selecting-factory) |
| PH865 超武指定器与抑制器范围 | 超武瞄准时显示己方指定器与敌方抑制器范围，可全局、按超武和玩家设置控制。 | `ShowDesignatorRange` | 使用已验证的超武范围查询，绘制过滤遵守玩家可见信息策略。 | L/S · [Show designator & inhibitor range](https://phobos.readthedocs.io/en/latest/User-Interface.html#show-designator-inhibitor-range) |
| PH866 显示已进行游戏时间 | 需要作品和玩家两侧开关共同启用，显示游戏已进行时间和可调背景；观察者沿用其既有时间显示。 | `ShowGameTime`、`ShowGameTime.BoardOpacity` | 由UI把模拟帧转换为游戏时间，避免新建独立运行计时器。 | L · [Show game time](https://phobos.readthedocs.io/en/latest/User-Interface.html#show-game-time) |
| PH867 发电增强器范围显示 | 放置建筑时可显示发电增强器作用范围，由作品与玩家设置控制。 | `ShowPowerPlantEnhancerRange` | 范围数据来自建筑/电力领域，放置呈现负责绘制。 | L · [Show power plant enhancer range](https://phobos.readthedocs.io/en/latest/User-Interface.html#show-power-plant-enhancer-range) |
| PH871 超武倒计时排序 | 可按ShowTimer.Priority升序从上向下排列计时器；只改顺序，不折叠或减少数量。 | `ShowTimer.Priority` | 超武UI统一排序和布局，30人场景另设计分组与筛选。 | L/S · [SuperWeapon ShowTimer sorting](https://phobos.readthedocs.io/en/latest/User-Interface.html#superweapon-showtimer-sorting) |
| PH873 居中任务字幕与历史消息 | 可把任务字幕放中部并保留历史，设置悬停不消失、背景和可见条数；显示条数不等于实际历史存储上限。 | `MessageApplyHoverState`、`MessageDisplayInCenter`、`MessageDisplayInCenter.BoardOpacity`、`MessageDisplayInCenter.LabelsCount`、`MessageDisplayInCenter.RecordsCount` | 消息历史和存档由应用服务负责，呈现与输入遮挡单独处理。 | L · [Task subtitles display in the middle of the screen](https://phobos.readthedocs.io/en/latest/User-Interface.html#task-subtitles-display-in-the-middle-of-the-screen) |
| PH879 建筑同型选择 | 扩展原版有限的建筑同型选择能力；上游因技术限制要求Ares存在，这并不代表VERA应加载Ares.dll。 | `BuildingTypeSelectable` | 在VERA选择系统直接提供等价策略，复用对象分类和权限过滤。 | L · [Type select for buildings](https://phobos.readthedocs.io/en/latest/User-Interface.html#type-select-for-buildings) |
| PH880 光效、震屏和轨迹开关 | 玩家可关闭指定闪光、震屏与可隐藏的Phobos激光轨迹；作品可把某轨迹设成不可隐藏，光效还受细节等级影响。 | `WarheadParticleAlphaImageIsLightFlash`、`LightFlashAlphaImageDetailLevel`、`AlphaImage.IsLightFlash`、`Particle.AlphaImageIsLightFlash`、`IsHideable`、`HideLightFlashEffects`、`HideLaserTrailEffects` 等（8个不同示例键） | 渲染层分类过滤效果，关闭视觉不取消模拟中的武器或伤害。 | L · [Visual effects toggling](https://phobos.readthedocs.io/en/latest/User-Interface.html#visual-effects-toggling) |
| PH881 回收与矿石收入浮字 | 显示建筑回收、收矿或修理厂出售所得，短间隔收入合并，支持阵营过滤与AI开关；Delay=0会调整为1。 | `DisplayIncome`、`DisplayIncome.Delay`、`DisplayIncome.Houses`、`DisplayIncome.AllowAI`、`DisplayIncome.Offset` | 经济结果发出只读收入事件，呈现层合并显示并遵守可见性。 | L/S · [Visual indication of income from grinders and refineries](https://phobos.readthedocs.io/en/latest/User-Interface.html#visual-indication-of-income-from-grinders-and-refineries) |

### 侧栏

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH827 可切换自动维修 | 维修按钮或热键改为切换自动维修状态，不再切换鼠标模式；关闭时停止建筑维修。 | `ExtendedPlayerRepair` | 玩家命令进入维修所有者，记录开关并校验资金与建筑状态。 | L · [Allow replacing vanilla repairing with togglable auto repairing](https://phobos.readthedocs.io/en/latest/User-Interface.html#allow-replacing-vanilla-repairing-with-togglable-auto-repairing) |
| PH837 建筑生产队列 | 建筑可像单位一样排入生产队列；当前建筑造好但未放置前，不开始建造下一座。 | `BuildingProductionQueue` | 扩展现有生产队列与放置状态，避免UI另存生产进度。 | L · [Building Production Queue](https://phobos.readthedocs.io/en/latest/User-Interface.html#building-production-queue) |
| PH838 建造图标排序 | 类型优先级在原版排序规则前生效；还可用内部Name替代本地化UIName作末级比较，避免语言包改变顺序。 | `SortCameoByName`、`CameoPriority` | 侧栏排序读取冻结类型数据，排序不改变生产或类型索引。 | L/S · [Cameo Sorting](https://phobos.readthedocs.io/en/latest/User-Interface.html#cameo-sorting) |
| PH839 暂停菜单背景居中 | 可把暂停背景在可用区域居中，以适应超过1024×768的分辨率。 | `CenterPauseMenuBackground` | 在暂停菜单布局层处理，使用明确像素与缩放坐标。 | L/S · [Center pause menu background](https://phobos.readthedocs.io/en/latest/User-Interface.html#center-pause-menu-background) |
| PH841 缺失建造图标后备资源 | 可用指定SHP或PCX替代缺失图标的XXICON.SHP后备图。 | `MissingCameo` | 由资源目录与侧栏加载器统一处理回退，校验扩展名和调色板。 | L/S · [Custom Missing Cameo ( XXICON.SHP )](https://phobos.readthedocs.io/en/latest/User-Interface.html#custom-missing-cameo-xxicon-shp) |
| PH851 矿工数量统计 | 显示活跃和总矿工数，可自定义计入的对象及经济建筑、阈值颜色和位置；玩家可关闭，默认位置与Weeds计数可能冲突。 | `HarvesterCounter.Show`、`HarvesterCounter.Label`、`HarvesterCounter.ConditionYellow`、`HarvesterCounter.ConditionRed`、`Harvester.Counted`、`Sidebar.HarvesterCounter.Offset`、`Sidebar.HarvesterCounter.HideMaxValue` 等（12个不同示例键） | 统计从经济/任务所有者派生，侧栏只读，不额外拥有矿工状态。 | L/S · [Harvester counter](https://phobos.readthedocs.io/en/latest/User-Interface.html#harvester-counter) |
| PH856 电力盈余显示 | 显示电力产出与消耗差值，按负载比例着色，停电状态有独立显示；位置、对齐及玩家开关可配置。 | `PowerDelta.Show`、`PowerDelta.ConditionYellow`、`PowerDelta.ConditionRed`、`Sidebar.PowerDelta.Offset`、`Sidebar.PowerDelta.ColorGreen`、`Sidebar.PowerDelta.ColorYellow`、`Sidebar.PowerDelta.ColorRed` 等（10个不同示例键） | 从阵营电力快照派生文本，停电与零产出规则显式处理。 | L/S · [Power delta counter](https://phobos.readthedocs.io/en/latest/User-Interface.html#power-delta-counter) |
| PH858 侧栏生产进度 | 通过额外tab0xpp.shp资源在侧栏显示工厂生产状态，并可按阵营调整位置。 | `ProducingProgress.Show`、`Sidebar.ProducingProgress.Offset` | 读取生产所有者的进度快照，资源绑定与绘制归侧栏。 | L/S · [Producing Progress](https://phobos.readthedocs.io/en/latest/User-Interface.html#producing-progress) |
| PH868 侧栏雷达水平对齐 | 修正原版雷达图的水平偏移，可总开关或按阵营覆盖；只影响本地玩家侧栏。 | `UseSidebarRadarOffsetFix`、`Sidebar.RadarOffsetFix` | 在侧栏坐标布局中集中修正，避免修改地图或雷达模拟数据。 | L · [Sidebar radar horizontal offset](https://phobos.readthedocs.io/en/latest/User-Interface.html#sidebar-radar-horizontal-offset) |
| PH870 阵营侧栏坐标风格 | 按阵营选择原版GDI或另一组硬编码按钮坐标，默认首阵营采用GDI布局。 | `Sidebar.GDIPositions` | 资源与布局系统映射兼容样式，跨分辨率适配留给UI。 | L/S · [Specify Sidebar style](https://phobos.readthedocs.io/en/latest/User-Interface.html#specify-sidebar-style) |
| PH872 屏幕左侧超级武器栏 | 可把超武图标放在屏幕左边，配置列数、间距、布局及重要性阈值；作品限制与玩家筛选共同决定哪些超武出现。 | `SuperWeaponSidebar`、`SuperWeaponSidebar.Pyramid`、`SuperWeaponSidebar.Interval`、`SuperWeaponSidebar.LeftOffset`、`SuperWeaponSidebar.CameoHeight`、`SuperWeaponSidebar.Max`、`SuperWeaponSidebar.MaxColumns` 等（20个不同示例键） | UI建立只读超武视图模型，点击仍提交原有使用命令；需高分辨率和30人布局验收。 | L · [SuperWeapon Sidebar](https://phobos.readthedocs.io/en/latest/User-Interface.html#superweapon-sidebar) |
| PH882 Weeds储量显示 | 侧栏显示储存的Weeds数值，可调偏移和颜色，玩家可关闭；默认位置与矿工计数重叠。 | `WeedsCounter.Show`、`Sidebar.WeedsCounter.Offset`、`Sidebar.WeedsCounter.Color`、`ShowWeedsCounter` | 只读资源仓储快照，布局时统一处理计数器冲突。 | L/S · [Weeds counter](https://phobos.readthedocs.io/en/latest/User-Interface.html#weeds-counter) |

### 通用说明

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH828 胜负结算音乐 | 可为每个阵营指定获胜与失败时播放的主题曲。 | `IngameScore.WinTheme`、`IngameScore.LoseTheme` | 结算流程发出表现事件，音频层解析主题引用。 | L/S · [Audio](https://phobos.readthedocs.io/en/latest/User-Interface.html#audio) |
| PH852 加载画面增强 | 支持PCX、较大加载图的居中裁切及隐藏空出生位黑点；上游实际窗口尺寸还取决于CnCNet5 spawner，缺少时仍是800×600。 | `DisableEmptySpawnPositions` | 由VERA启动和资源层实现图像布局，不能把上游spawner能力视为SDK已具备。 | L/S · [Loading screen](https://phobos.readthedocs.io/en/latest/User-Interface.html#loading-screen) |
| PH878 扩展侧栏提示 | 建造图标可显示名称、费用、电力、时间、说明和超武次数；支持长文本、锚定与本地化格式，玩家可控制描述显示。 | `ExtendedToolTips`、`AnchoredToolTips`、`CostLabel`、`PowerLabel`、`PowerBlackoutLabel`、`TimeLabel`、`SWShotsFormat` 等（14个不同示例键） | 读取最终类型与生产数据，费用计算复用经济所有者，文字绘制留在UI。 | L/S · [Tooltips](https://phobos.readthedocs.io/en/latest/User-Interface.html#tooltips) |

### 热键

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH843 热键：轮换单个选择 | 在启动轮换时的选中池内逐个选择并循环；改变选择会重置轮换，空池给出提示。 | `CycleSelectionKeyEnabled` | 选择控制器管理池与失效对象，纯本地选择不进入玩法状态。 | L · [[ ] Cycle Selection](https://phobos.readthedocs.io/en/latest/User-Interface.html#cycle-selection) |
| PH844 热键：按类型轮换选择 | 按类型选择整组选中对象，单一类型时转为逐个轮换；顺序按优先级、原始Cost及逆INI加载顺序，支持Ares GroupAs。 | `CycleTypeSelectionKeyEnabled`、`CycleTypeSelectionPrintSummary`、`TypeCyclePriority` | 选择控制器复用类型归组，排序只读且不受玩家造价倍率误导。 | L · [[ ] Cycle Type Selection](https://phobos.readthedocs.io/en/latest/User-Interface.html#cycle-type-selection) |
| PH845 热键：移出部分选择 | 从当前选择中取消1个或5个对象，需开启对应热键并提供本地化文字。 | `DeselectObjectKeysEnabled` | 由选择控制器更新本地集合，处理对象已死亡的情况。 | L · [[ ] Deselect Object(s)](https://phobos.readthedocs.io/en/latest/User-Interface.html#deselect-object-s) |
| PH848 热键：伤害数字 | 开关伤害浮字显示，属于开发热键，启用条件见开发工具的DebugKeysEnabled。 | 无独立示例键；见原文条件 | 仅切换本地调试视图，不改变伤害结算。 | L/S · [[ ] Display Damage Numbers](https://phobos.readthedocs.io/en/latest/User-Interface.html#display-damage-numbers) |
| PH849 热键：对象信息 | 把当前悬停或最后选中的对象信息写入日志并提示；属于开发诊断。 | 无独立示例键；见原文条件 | 宿主诊断入口读取对象快照，校验对象仍有效。 | L/S · [[ ] Dump Object Info](https://phobos.readthedocs.io/en/latest/User-Interface.html#dump-object-info) |
| PH854 热键：下一空闲矿工 | 选择并居中到下一名当前空闲、且被矿工计数规则识别的对象。 | `NextIdleHarvesterKeyEnabled` | 复用矿工分类与任务快照，镜头操作留在应用层。 | L/S · [[ ] Next Idle Harvester](https://phobos.readthedocs.io/en/latest/User-Interface.html#next-idle-harvester) |
| PH859 热键：快速存档 | 保存当前游戏并生成本地化描述；上游多人可用性取决于支持多人存档的YRpp spawner版本。 | `QuickSaveKeyEnabled` | 调用VERA统一存档协调器，多人模式须先有一致的暂停/快照协议。 | L/S · [[ ] Quicksave](https://phobos.readthedocs.io/en/latest/User-Interface.html#quicksave) |
| PH861 热键：保存变量 | 把局部和全局变量导出到文件；与游戏存档不是同一种操作，要求开发热键权限。 | 无独立示例键；见原文条件 | 宿主诊断服务负责导出，不给guest通用磁盘权限。 | L/S · [[ ] Save Variables](https://phobos.readthedocs.io/en/latest/User-Interface.html#save-variables) |
| PH863 热键：选择受控单位 | 选择当前屏幕内被非永久心控捕获的单位，并显示本地化结果提示。 | `SelectCapturedKeyEnabled` | 选择控制器查询心控关系与屏幕可见性，不改归属状态。 | L · [[ ] Select Captured Units](https://phobos.readthedocs.io/en/latest/User-Interface.html#select-captured-units) |
| PH874 热键：范围指示开关 | 开关超级武器指定器范围显示，作品可禁用热键，文字由CSF提供。 | `ToggleDesignatorRangeKeyEnabled` | 切换玩家本地显示偏好，复用既有范围绘制入口。 | L/S · [[ ] Toggle Designator Range](https://phobos.readthedocs.io/en/latest/User-Interface.html#toggle-designator-range) |
| PH875 热键：数字显示开关 | 开关对象数字显示类型，作品可配置热键是否可用。 | `ToggleDigitalDisplayKeyEnabled` | 只改变本地显示筛选，不删除模拟数据。 | L/S · [[ ] Toggle Digital Display](https://phobos.readthedocs.io/en/latest/User-Interface.html#toggle-digital-display) |
| PH876 热键：逐帧模式开关 | 在普通推进与开发逐帧模式之间切换，与逐帧推进命令配合使用。 | 无独立示例键；见原文条件 | 由应用主循环控制模式，联网时须有明确协调规则。 | L/S · [[ ] Toggle Frame By Frame Mode](https://phobos.readthedocs.io/en/latest/User-Interface.html#toggle-frame-by-frame-mode) |
| PH877 热键：任务消息面板 | 开关中部任务字幕历史面板；该普通界面键与后面列出的开发热键权限不能混为一谈。 | `ToggleMessageListKeyEnabled` | 输入与消息面板控制器统一处理显示和鼠标命中区域。 | L · [[ ] Toggle Message Label](https://phobos.readthedocs.io/en/latest/User-Interface.html#toggle-message-label) |

### 其他界面配置

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH869 战役开局自动存档开关 | 玩家可决定开始新战役时是否执行原版自动存档。 | `SaveGameOnScenarioStart` | 任务启动流程读取玩家偏好并调用统一存档入口。 | L/S · [Skip saving game on starting a new campaign](https://phobos.readthedocs.io/en/latest/User-Interface.html#skip-saving-game-on-starting-a-new-campaign) |

<a id="misc"></a>
## 配置加载与开发工具

本节16个具名主题。

### 开发与诊断

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH647 额外失步日志 | 出现联机失步时增加随机调用、朝向、目标与目的地等信息，帮助比较双方模拟分歧。 | 无独立示例键；见原文条件 | 由同步诊断层记录，避免日志和格式化影响模拟状态。 | L/S · [Additional sync logging](https://phobos.readthedocs.io/en/latest/Miscellanous.html#additional-sync-logging) |
| PH649 开发工具：伤害浮字 | 热键显示伤害与治疗数值，普通生命和护盾使用不同配色；需DebugKeysEnabled启用开发键。 | 无独立示例键；见原文条件 | 损伤结果生成只读调试事件，浮字由呈现层处理。 | L/S · [Display Damage Numbers](https://phobos.readthedocs.io/en/latest/Miscellanous.html#display-damage-numbers) |
| PH650 开发工具：对象信息导出 | 热键导出选中或鼠标悬停对象信息，要求启用开发键。 | 无独立示例键；见原文条件 | 由宿主诊断服务输出有限快照，guest无需文件写权限。 | L/S · [Dump Object Info](https://phobos.readthedocs.io/en/latest/Miscellanous.html#dump-object-info) |
| PH651 开发工具：逐帧执行 | 可进入逐帧模式，按一次命令推进1、5、10、15、30或60帧以观察行为。 | 无独立示例键；见原文条件 | 由主循环调度控制，联机模式需明确同步与权限条件。 | L/S · [Frame Step In](https://phobos.readthedocs.io/en/latest/Miscellanous.html#frame-step-in) |
| PH653 可复用军衔标识类型 | 注册InsigniaType后可按军衔、武器槽或乘员条件复用图像、调色板和帧配置。 | `Insignia`、`Insignia.Rookie`、`Insignia.Veteran`、`Insignia.Elite`、`InsigniaPalette`、`InsigniaPalette.Rookie`、`InsigniaPalette.Veteran` 等（15个不同示例键） | 加载期解析资源与类型，表现层按只读军衔和武器状态选择。 | L · [Insignia Type](https://phobos.readthedocs.io/en/latest/Miscellanous.html#insignia-type) |
| PH655 缺失音频提示 | 解析soundmd.ini时把找不到的音频采样写入调试日志。 | 无独立示例键；见原文条件 | 由音频资源加载器统一诊断，保留逻辑名称与解析来源。 | L · [Logging missing audio files (samples)](https://phobos.readthedocs.io/en/latest/Miscellanous.html#logging-missing-audio-files-samples) |
| PH656 场景变量导出文件 | 开发热键可导出局部和全局变量到locals.ini与globals.ini，也可在场景结束自动导出；按任务名保存分节并保留已有其他节。 | `SaveVariablesOnScenarioEnd` | 宿主诊断导出服务读取变量快照，文件写入不放给guest。 | L/S · [Save variables to file](https://phobos.readthedocs.io/en/latest/Miscellanous.html#save-variables-to-file) |
| PH658 移动器名称别名 | Locomotor可写Drive、Hover、Jumpjet等语义名称替代CLSID；Chrono不是标准别名，其看似有效来自错误值回退。 | 无独立示例键；见原文条件 | 类型绑定器显式解析合法别名，不把偶然回退扩展成承诺。 | L/S · [Semantic locomotor aliases](https://phobos.readthedocs.io/en/latest/Miscellanous.html#semantic-locomotor-aliases) |

### 通用说明

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH648 移除Blowfish DLL启动依赖 | 上游不再需要BLOWFISH.DLL才能启动；这项部署变化不等于Mod包加密或素材保护。 | 无独立示例键；见原文条件 | 资源解码依赖由跨平台加载器管理，不引入旧DLL启动约束。 | L · [Blowfish Dependency](https://phobos.readthedocs.io/en/latest/Miscellanous.html#blowfish-dependency) |

### INI与复用配置

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH652 INI文件包含 | 启用-Include后按[$Include]列表深度优先包含文件，后读值覆盖先读值，编号须跨包含文件唯一；与Ares[#include]互斥，并有首行可能丢失的上游提示。 | 无独立示例键；见原文条件 | 复用VERA统一INI加载器，明确兼容策略；不把解析职责复制到插件。 | L/S · [Include files](https://phobos.readthedocs.io/en/latest/Miscellanous.html#include-files) |
| PH654 关键帧动画配置 | 支持按百分比或绝对帧指定值，绝对帧优先，并可重置和选择插值；重复时点无效，类型是否可插值须逐功能确定。 | `BASEKEY.KeyframeN.Value`、`BASEKEY.KeyframeN.Percentage`、`BASEKEY.KeyframeN.Absolute`、`BASEKEY.Keyframe.ResetValues`、`BASEKEY.Interpolation` | 提供通用关键帧资源描述，模拟与表现插值分别定义数值规范。 | L · [Keyframe animations](https://phobos.readthedocs.io/en/latest/Miscellanous.html#keyframe-animations) |
| PH657 INI节继承 | 启用-Inheritance后以$Inherits按从左到右、深度优先查找父节；当前节缺失或空值时回退，最后才用默认；与Ares对应机制互斥。 | `$Inherits` | 由统一INI解析器处理循环、优先级和来源，冻结结果供组件读取。 | L/S · [Section inheritance](https://phobos.readthedocs.io/en/latest/Miscellanous.html#section-inheritance) |
| PH660 异常处理与崩溃报告 | 上游可切换异常处理，输出崩溃报告和转储，支持符号与错误数据库；这是原生进程诊断设施，不是玩法钩子。 | 无独立示例键；见原文条件 | 用各平台宿主崩溃诊断设施实现，不向WASM暴露原生异常地址。 | L · [Turning off/on in-game exception handling](https://phobos.readthedocs.io/en/latest/Miscellanous.html#turning-off-on-in-game-exception-handling) |
| PH662 Windows控件视觉样式 | 上游为进程启用Common Controls v6清单，让游戏和Phobos原生对话框使用较新的系统样式。 | 无独立示例键；见原文条件 | 跨平台应用采用自己的窗口适配层，不在玩法组件里处理Windows清单。 | L · [Visual styles](https://phobos.readthedocs.io/en/latest/Miscellanous.html#visual-styles) |

### 游戏速度

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH659 单人游戏速度 | 可设战役默认速度，并用周期性交替帧延迟构成自定义速度；官方稳定可达范围是10—60FPS，最快档以上实际速度随机器变化。 | `CustomGS`、`CustomGSN.ChangeInterval`、`CustomGSN.ChangeDelay`、`CustomGSN.DefaultDelay`、`CampaignDefaultGameSpeed` | VERA主循环直接管理模拟步频，兼容配置需映射语义而非照搬忙等方式。 | L/S · [Single player game speed](https://phobos.readthedocs.io/en/latest/Miscellanous.html#single-player-game-speed) |

### 玩家颜色

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH661 扩展多人颜色 | SkirmishUnlimitedColors让外置启动器传来的颜色索引对应Colors列表，突破八色；原版大厅不适用，且与Ares下拉颜色配置互斥。 | `SkirmishUnlimitedColors` | 客户端、spawn协议、模拟与渲染共用颜色索引契约，30人还需可辨识方案。 | L/S · [Unlimited skirmish colors](https://phobos.readthedocs.io/en/latest/Miscellanous.html#unlimited-skirmish-colors) |

<a id="interop"></a>
## 上游原生互操作接口

本节17个具名主题。

以下是上游原生DLL之间的接口说明。VERA当前没有加载这些原生导出，表中SDK建议是重新设计等价能力的方向。

### 导出能力

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH630 接口AE_Attach：施加附加效果 | 上游原生接口按名称列表向目标施加效果，带施加者、来源及延迟覆盖；输出成功数量，无有效类型时返回S_FALSE。 | 无独立示例键；见原文条件 | 只作API设计参考；WASM须使用稳定对象句柄与类型ID，不能传原生指针。 | L · [AE_Attach](https://phobos.readthedocs.io/en/latest/Interoperability.html#ae-attach) |
| PH631 接口AE_Detach：按类型移除 | 按效果类型名称列表从目标移除效果，输出移除数量；没有匹配效果与参数错误返回不同状态。 | 无独立示例键；见原文条件 | 未来用经过权限检查的效果移除命令，保留可诊断结果。 | L · [AE_Detach](https://phobos.readthedocs.io/en/latest/Interoperability.html#ae-detach) |
| PH632 接口AE_DetachByGroups：按组移除 | 按附加效果组名批量移除目标上的效果并返回数量。 | 无独立示例键；见原文条件 | 先定义组注册与叠加语义，再经效果所有者执行。 | L · [AE_DetachByGroups](https://phobos.readthedocs.io/en/latest/Interoperability.html#ae-detachbygroups) |
| PH633 接口AE_TransferEffects：转移效果 | 把来源对象的全部附加效果转移到目标；要求来源和目标指针有效。 | 无独立示例键；见原文条件 | 类型转换与效果转移共用生命周期规则，检查来源引用和计时。 | L · [AE_TransferEffects](https://phobos.readthedocs.io/en/latest/Interoperability.html#ae-transfereffects) |

### 通用说明

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH634 原生API返回约定 | 使用HRESULT表示成功、无工作、空指针、非法参数或内部失败，输出值通过参数返回。 | 无独立示例键；见原文条件 | SDK可借鉴结构化结果，ABI采用可序列化值，避免复用COM内存布局。 | L · [API Convention](https://phobos.readthedocs.io/en/latest/Interoperability.html#api-convention) |
| PH635 原生导出API目录 | 列出效果、弹体归属、事件、类型转换、威胁回调和场景变量接口及可用版本；变量接口从Interop1.1.0列出。 | 无独立示例键；见原文条件 | 作为能力清单参考，与VERA的宿主导入版本独立管理。 | L · [Available APIs](https://phobos.readthedocs.io/en/latest/Interoperability.html#available-apis) |

### 对象与场景接口

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH636 接口Bullet_SetFirerOwner：弹体归属 | 更新弹体扩展记录的开火阵营；可有意传空阵营清除记录，无对应扩展实例会报错。 | 无独立示例键；见原文条件 | 弹体所有者提供验证后的归属更新命令，不能直接写对象内存。 | L · [Bullet_SetFirerOwner](https://phobos.readthedocs.io/en/latest/Interoperability.html#bullet-setfirerowner) |
| PH637 接口ConvertToType_Phobos：类型转换 | 把FootClass对象转成目标类型，不兼容类型返回非法参数；实际转交TechnoExt的转换逻辑。 | 无独立示例键；见原文条件 | 先提供完整转换事务，包含身份、状态、容器与引用迁移。 | L · [ConvertToType_Phobos](https://phobos.readthedocs.io/en/latest/Interoperability.html#converttotype-phobos) |
| PH639 接口EventExt_AddEvent：提交事件 | 调用已有事件扩展的AddEvent，按实际触发结果返回S_OK或S_FALSE，空事件指针被拒绝。 | 无独立示例键；见原文条件 | SDK事件以类型化数据入队，由场景事件所有者验证和分派。 | L · [EventExt_AddEvent](https://phobos.readthedocs.io/en/latest/Interoperability.html#eventext-addevent) |
| PH641 额外威胁值回调 | 注册多个回调并按注册顺序处理，后一个接收前一个处理后的总威胁值；顺序影响结果。 | 无独立示例键；见原文条件 | 外置威胁策略须固定执行顺序和确定性数值，热循环先评估调用成本。 | L · [RegisterCalculateExtraThreatCallback](https://phobos.readthedocs.io/en/latest/Interoperability.html#registercalculateextrathreatcallback) |
| PH643 接口读取全局变量 | 按索引读全局变量；缺失时输出0并返回S_FALSE，不自动创建，场景未初始化则失败。 | 无独立示例键；见原文条件 | 提供只读场景变量查询，保留缺失与实际0的区别。 | L · [Variables_GetGlobal_Phobos](https://phobos.readthedocs.io/en/latest/Interoperability.html#variables-getglobal-phobos) |
| PH644 接口读取局部变量 | 按索引读局部变量，缺失返回0和S_FALSE，不创建条目，输出参数为空被拒绝。 | 无独立示例键；见原文条件 | 与全局变量共享查询模型，作用域显式传入。 | L · [Variables_GetLocal_Phobos](https://phobos.readthedocs.io/en/latest/Interoperability.html#variables-getlocal-phobos) |
| PH645 接口设置全局变量 | 设置全局变量，不存在时创建；需要场景扩展已初始化。 | 无独立示例键；见原文条件 | 由场景变量所有者执行命令并纳入存档与联机校验。 | L · [Variables_SetGlobal_Phobos](https://phobos.readthedocs.io/en/latest/Interoperability.html#variables-setglobal-phobos) |
| PH646 接口设置局部变量 | 设置或创建局部变量；场景尚未初始化时失败。 | 无独立示例键；见原文条件 | 复用同一变量命令入口，生命周期限定到当前场景。 | L · [Variables_SetLocal_Phobos](https://phobos.readthedocs.io/en/latest/Interoperability.html#variables-setlocal-phobos) |

### 协议版本

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH638 原生API弃用处理 | 官方设计保留废弃入口并给出终止性诊断，提示替代接口和版本范围；正文示例并非当前确有接口被删除。 | 无独立示例键；见原文条件 | VERA宜在加载时检查能力与版本，拒绝不兼容组件并给出迁移信息。 | L · [Deprecated API Handling](https://phobos.readthedocs.io/en/latest/Interoperability.html#deprecated-api-handling) |
| PH640 查询Interop接口版本 | GetInteropAPIVersion返回major、minor、patch三部分，调用者据此判断可用接口。 | 无独立示例键；见原文条件 | 插件manifest和宿主协商使用自己的版本，不借Phobos版本冒充兼容。 | L · [GetInteropAPIVersion](https://phobos.readthedocs.io/en/latest/Interoperability.html#getinteropapiversion) |
| PH642 原生API版本规则 | 破坏兼容的变更升主版本，兼容新增升次版本，未改接口的修正升补丁版本；大版本变化重置较低位。 | 无独立示例键；见原文条件 | 区分宿主协议、插件配置和存档状态三个版本域。 | L · [Semantic Versioning Rules](https://phobos.readthedocs.io/en/latest/Interoperability.html#semantic-versioning-rules) |

<a id="fixes"></a>
## 逐项修复与扩展兼容

本节380条来自官方四份修复列表，每个顶层列表项各占一行；嵌套条件保留在对应摘要内，不把子句另算一项。此处也包含行为改进和扩展交互。原版错误应修正在对应引擎所有者；有兼容差异时，再明确作品策略。当前是否已在VERA实现须查具体支持记录，不能由“上游已修复”直接推出。

### AI与地图逐项修复

本组14条。

| 编号及条目 | 中文说明 | 版本与官方来源 |
| --- | --- | --- |
| PH006 按玩家槽位绑定地图对象 | 遭遇战和联机地图可用 `<Player @ X>` 指定预置对象及触发所属方；所属玩家未参战时，相应触发会被销毁。 | L列表 · [Bugfixes and Miscellanous 第1项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH007 修正移动到单元格的编码 | 脚本“Move to cell”改用尤复的 `1000*Y+X` 坐标编码，避免按旧版 `128*Y+X` 移动到错误位置。 | L列表 · [Bugfixes and Miscellanous 第2项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH008 扩大地图航路点编号范围 | 航路点可读取至 `2147483647`，突破原先最高 701 的范围；编辑器和地图工具仍需配合支持。 | L列表 · [Bugfixes and Miscellanous 第3项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH009 触发动画支持实际作用 | 地图动作 41 可通过参数创建会发声、造成伤害及触发矿石连锁反应的动画，编辑器动作配置也需更新。 | L列表 · [Bugfixes and Miscellanous 第4项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH010 触发建造支持建起和维修 | 地图动作 125 可选择播放建起动画，并允许单人 AI 维修所建建筑；需同步调整编辑器动作定义。 | L列表 · [Bugfixes and Miscellanous 第5项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH011 取消地图变量数量上限 | 规则中的全局变量与地图中的局部变量不再受原有数量限制，适合更复杂的任务状态与触发逻辑。 | L列表 · [Bugfixes and Miscellanous 第6项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH012 部署失败后寻找空地 | 脚本部署 `DeploysInto` 载具时，当前位置不可展开会继续寻找可用空间，避免队伍永久卡在部署动作。 | L列表 · [Bugfixes and Miscellanous 第7项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH013 触发生成队伍的载客逻辑 | 地图动作 7、80、107 生成的队伍可以正常使用 IFV 与 OpenTopped 乘员机制。 | L列表 · [Bugfixes and Miscellanous 第8项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH014 移除预置建筑粒子强制创建 | 取消地图开始时为预置建筑一律生成 `NaturalParticleSystem` 的处理，避免不符合状态的粒子系统出现。 | L列表 · [Bugfixes and Miscellanous 第9项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH015 明确 AI 超时空脚本所用超武 | 脚本 56、57、10104 可由 `AIChronoSphereSW`、`AIChronoWarpSW` 指定超武；未指定时仍按文档规则回退选择。 | L列表 · [Bugfixes and Miscellanous 第10项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH016 修正 AI 队伍征募不满员 | 修正队伍征募过程的不一致，减少本可补齐成员却以人数不足状态形成队伍的问题。 | L列表 · [Bugfixes and Miscellanous 第11项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH017 恢复脚本播放语音动作 | 恢复《泰伯利亚之日》的脚本动作 24“Play speech”，让队伍脚本能够播放指定语音。 | L列表 · [Bugfixes and Miscellanous 第12项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH018 允许地图和模式覆盖矿石类型 | 修正 Tiberium 类型不能被地图及游戏模式 INI 覆盖的问题，使相应规则层的设置能够生效。 | L列表 · [Bugfixes and Miscellanous 第13项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |
| PH019 修正任务简报显示越界 | 修正固定窗口尺寸造成的任务简报越界，避免较多内容超出原有显示区域。 | L列表 · [Bugfixes and Miscellanous 第14项](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#bugfixes-and-miscellanous) |

### 游戏逻辑逐项修复

本组314条。

| 编号及条目 | 中文说明 | 版本与官方来源 |
| --- | --- | --- |
| PH106 正确保存游戏模式选项 | 修正模式选项的存档恢复，例如读档后 `BuildOffAlly` 不再出现错误值。 | L列表 · [Bugfixes and miscellaneous 第1项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH107 读档后正确移除建筑光源 | 修正读档后建筑光照染色无法清除的问题，建筑出售或摧毁时不再留下永久残光。 | L列表 · [Bugfixes and miscellaneous 第2项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH108 兼容预览节的不同排列 | 地图中 Preview/PreviewPack 位于 Map 节之后时，也能正确读取并绘制地图预览。 | L列表 · [Bugfixes and miscellaneous 第3项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH109 地图重新着色保留光源 | 修正地图动作改变环境色时破坏光源的行为；可用 `UseRetintFix=no` 关闭此项以权衡性能。 | L列表 · [Bugfixes and miscellaneous 第4项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH110 心控载具展开保留控制关系 | 修正被心灵控制的载具展开成建筑后，所有权被永久转交给控制方的问题。 | L列表 · [Bugfixes and miscellaneous 第5项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH111 工程师占领解除建筑心控 | 工程师占领被心控建筑时正确解除旧的心控关系，避免原控制链继续影响新所有者。 | L列表 · [Bugfixes and miscellaneous 第6项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH112 抑制载具式建筑的占领提示 | 占领被游戏视为载具的建筑时不再播放 `EVA_BuildingCaptured`，避免不合适的建筑占领提示。 | L列表 · [Bugfixes and miscellaneous 第7项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH113 防止死亡对象再次死亡 | 对仍在执行沉没、坠毁或死亡动画的对象，避免重复进入死亡流程及其伴随效果。 | L列表 · [Bugfixes and miscellaneous 第8项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH114 隐形辐射工兵可部署开火 | 修正隐形状态下的 Desolator 不能发射部署武器的问题，使该状态不再阻断部署攻击。 | L列表 · [Bugfixes and miscellaneous 第9项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH115 修正超时空抹除对象清理 | 被 Temporal 武器作用的单位可正确完成抹除与清理，避免对象残留并导致错误。 | L列表 · [Bugfixes and miscellaneous 第10项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH116 恢复建筑快捷键放置模式 | 放置失败并退出放置模式后，建筑与防御标签快捷键仍可再次进入放置，避免快捷键失效。 | L列表 · [Bugfixes and miscellaneous 第11项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH117 收起建筑不误报集合点 | 带有收起转换功能的建筑收起时不再播放 `EVA_NewRallypointEstablished`，避免错误的集合点提示。 | L列表 · [Bugfixes and miscellaneous 第12项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH118 海军建筑尊重非水上放置 | 修正 `Naval=yes` 强制水上放置的问题，使明确的 `WaterBound=no` 能够发挥作用。 | L列表 · [Bugfixes and miscellaneous 第13项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH119 串行 AI 队列的机场对接 | 使用 Ares `AllowParallelAIQueues=no` 时修正 AI 飞机对接机场的问题，属于扩展间交互修正。 | L列表 · [Bugfixes and miscellaneous 第14项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH120 所属方颜色激光支持加粗 | 修正所属方颜色模式的激光绘制，使较大的激光粗细设置也能正常表现。 | L列表 · [Bugfixes and miscellaneous 第15项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH121 修正死亡武器引爆流程 | 使 `DeathWeapon` 更正确地引爆；`PreImpactAnim` 等部分设置仍不受支持，不能视为全部引爆行为兼容。 | L列表 · [Bugfixes and miscellaneous 第16项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH122 修正高编号驻军枪口位置 | 驻军建筑的 `MuzzleFlashX` 在编号超过 10 时仍使用正确位置，避免枪口闪光落在建筑中心。 | L列表 · [Bugfixes and miscellaneous 第17项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH123 修正被牵引跳跃单位坠毁 | 可坠毁的 Jumpjet 单位在被 Locomotor 弹头牵引期间死亡时，正确落向地面。 | L列表 · [Bugfixes and miscellaneous 第18项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH124 修正跳跃单位开火朝向 | Jumpjet 单位从不同方向开火时可以正常转向目标，避免射击方向与目标方向不一致。 | L列表 · [Bugfixes and miscellaneous 第19项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH125 坠毁中的跳跃单位停止开火 | 修正 Jumpjet 单位进入坠毁流程后仍持续向敌人开火的问题，统一死亡过程中的攻击状态。 | L列表 · [Bugfixes and miscellaneous 第20项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH126 跳跃炮塔单位停车朝向 | 修正带炮塔 Jumpjet 单位停下后总朝右下方的问题，使停车后的朝向不再被错误重置。 | L列表 · [Bugfixes and miscellaneous 第21项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH127 跳跃单位使用探测器 | 修正 Jumpjet 对象不能使用 `Sensors` 的限制，让其探测能力能够正常参与判定。 | L列表 · [Bugfixes and miscellaneous 第22项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH128 敞篷乘员使用替代枪口 | 载具作为 OpenTopped 乘员时可使用 `AlternateFLH`，使从运输载具中发射的枪口位置更符合配置。 | L列表 · [Bugfixes and miscellaneous 第23项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH129 优化地图箱子分布 | 改善可见地图区域内随机箱子的空间分布，减少原算法导致的地图边缘偏好。 | L列表 · [Bugfixes and miscellaneous 第24项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH130 高编号地块的桥梁维修 | 允许使用编号 255 及以上的 TileSet，同时避免东北至西南方向的断桥因此无法维修。 | L列表 · [Bugfixes and miscellaneous 第25项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH131 任务失败后可直接读档 | 任务失败的重试对话框加入“Load Game”按钮，便于直接恢复存档重新尝试。 | L列表 · [Bugfixes and miscellaneous 第26项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH132 体素炮塔偏移支持三轴 | 体素炮塔的 `TurretOffset` 可读取 F、L、H 分量，支持前后、左右与高度方向的位置调整。 | L列表 · [Bugfixes and miscellaneous 第27项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH133 SHP 载具支持炮塔偏移 | 将 `TurretOffset` 支持扩展到 SHP 载具，使这类载具也能按配置偏移炮塔。 | L列表 · [Bugfixes and miscellaneous 第28项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH134 无限心控支持单伤害配置 | `InfiniteMindControl` 配合 `Damage=1` 时也能控制多个单位，避免被错误限制为一个目标。 | L列表 · [Bugfixes and miscellaneous 第29项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH135 飞机后续射击尊重连发数 | 非 Fighter 飞机或使用扫射模式的飞机，其后续各次射击也会考虑武器 `Burst`，不再只处理第一次。 | L列表 · [Bugfixes and miscellaneous 第30项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH136 载具部署开火遵守武器规则 | 载具部署开火尊重显式 `DeployFireWeapon`、`FireOnce` 与停止命令；单次开火还按文档规则设置短暂部署锁定。 | L列表 · [Bugfixes and miscellaneous 第31项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH137 步兵按目标选择部署武器 | `DeployFireWeapon=-1` 的步兵无论是否部署，都可依目标选择主副武器，不再被部署状态固定限制。 | L列表 · [Bugfixes and miscellaneous 第32项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH138 展开攻击保留原目标 | 载具通过 `DeployToFire` 展开为建筑时保留当前目标，避免转换后丢失原本的攻击任务。 | L列表 · [Bugfixes and miscellaneous 第33项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH139 连发效果使用正确枪口 | 修正使用 `Burst` 的武器在绘制激光等效果时采用错误发射偏移的问题。 | L列表 · [Bugfixes and miscellaneous 第34项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH140 简单部署音效只播放一次 | `IsSimpleDeployer` 完成展开或收起时仅播放一次对应音效，避免转向和部署动画期间不断重复。 | L列表 · [Bugfixes and miscellaneous 第35项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH141 AI 触发识别建筑升级 | AITrigger 条件能够识别建筑升级对象，让升级建筑可作为有效的 AI 条件参与判断。 | L列表 · [Bugfixes and miscellaneous 第36项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH142 回收建筑遵守吸收类型 | 修正 `UnitAbsorb`、`InfantryAbsorb` 与 `Grinding` 的交互，回收建筑只接收配置允许的对象类型。 | L列表 · [Bugfixes and miscellaneous 第37项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH143 不可回收载具显示禁入光标 | 载具不能进入某回收建筑时显示“No Enter”光标，让指针反馈与实际能否进入一致。 | L列表 · [Bugfixes and miscellaneous 第38项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH144 工程师遵守回收建筑准入 | 阻止工程师进入本应禁止进入的回收建筑，例如不允许进入的满血盟友建筑。 | L列表 · [Bugfixes and miscellaneous 第39项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH145 空中单位应用速度加成 | 飞机和 Jumpjet 单位可以获得国家、老兵、箱子及 Ares 附加效果提供的相应速度修正。 | L列表 · [Bugfixes and miscellaneous 第40项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH146 载具正确应用自定义色盘 | 修正体素和 SHP 载具对 `Palette` 的处理，使配置的自定义色盘按预期参与绘制。 | L列表 · [Bugfixes and miscellaneous 第41项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH147 地形可从雷达隐藏 | 地形类型设置 `RadarInvisible=true` 后，不再显示在小地图中，统一其雷达可见性。 | L列表 · [Bugfixes and miscellaneous 第42项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH148 解除隐形恢复心控标记 | 被心控对象解除隐形时，正确恢复心控指示动画，避免控制状态存在但标记缺失。 | L列表 · [Bugfixes and miscellaneous 第43项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH149 弹头和触发动画继承所属方 | 弹头动画列表、溅射列表及地图动作 41 创建的动画，默认获得与其来源相符的所属方。 | L列表 · [Bugfixes and miscellaneous 第44项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH150 核弹各阶段尊重亮光设置 | 核弹载运武器和载荷武器在适用时都读取 `Bright`，不再只在特定发射方式下使载荷亮光生效。 | L列表 · [Bugfixes and miscellaneous 第45项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH151 自愈标记尊重选择框偏移 | 通过步兵或载具自愈加成显示的标记也使用 `PixelSelectionBracketDelta`，与血条位置保持一致。 | L列表 · [Bugfixes and miscellaneous 第46项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH152 修正超时空目标血条残留 | 鼠标移开被 Temporal 武器攻击的单位后，血条可按正常规则隐藏，避免永久保留悬停显示。 | L列表 · [Bugfixes and miscellaneous 第47项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH153 建筑自愈恢复完好图像 | 建筑经 `SelfHealing` 恢复到完好状态时，正确切回未损伤图像，避免仍显示破损外观。 | L列表 · [Bugfixes and miscellaneous 第48项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH154 零占地地形不再崩溃 | 地形类型可使用 `Foundation=0x0` 而不崩溃，获得类似建筑零占地配置的效果。 | L列表 · [Bugfixes and miscellaneous 第49项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH155 弹体保留发射方所属信息 | 发射者在引爆前被摧毁，弹体仍保留其所属方；Ares EMP 等部分扩展弹头效果尚不适用。 | L列表 · [Bugfixes and miscellaneous 第50项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH156 敞篷运输按乘员武器算射程 | 运输载具扫描威胁和接近目标时考虑乘员的 `OpenTransportWeapon`，使采用的射程与乘员武器一致。 | L列表 · [Bugfixes and miscellaneous 第51项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH157 尾迹动画继承对象所属方 | 尾迹动画继承所附属动画、弹体或飞机的所属方，避免相关效果缺少正确归属。 | L列表 · [Bugfixes and miscellaneous 第52项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH158 建筑副武器使用自身激光参数 | 建筑以副武器发射激光时读取副武器的激光配置，不再误用主武器的参数。 | L列表 · [Bugfixes and miscellaneous 第53项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH159 无来源类型伤害也记录击杀 | 已知伤害所属方但没有来源 TechnoType 时，也正确记录载具被击杀并触发相应地图事件。 | L列表 · [Bugfixes and miscellaneous 第54项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH160 嵌套运输死亡归属正确传递 | 相互嵌套的运输对象死亡时，继续向更深层乘员传递击杀者信息，避免第二层之后丢失归属。 | L列表 · [Bugfixes and miscellaneous 第55项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH161 改进半透明压缩图像绘制 | 支持 Z 信息的半透明 RLE SHP 改用更准确的绘制，减少绿偏和色带；可用 `FixTransparencyBlitters=no` 关闭。 | L列表 · [Bugfixes and miscellaneous 第56项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH162 隐形弹体组合攻击更准确 | 修正 `Inviso=true` 弹体配合 `Airburst=yes` 或 `EMEffect=true` 弹头时可能出现的落点不准确。 | L列表 · [Bugfixes and miscellaneous 第57项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH163 炸弹生成步兵保留所属方 | 修正 BombClass 的 `MakeInfantry` 生成中立方步兵的问题，使生成单位获得正确所属关系。 | L列表 · [Bugfixes and miscellaneous 第58项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH164 磁轨粒子命中坐标正确 | 目标建筑有自定义 `TargetCoordOffset` 或对桥强制攻击时，磁轨粒子使用正确的绘制坐标。 | L列表 · [Bugfixes and miscellaneous 第59项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH165 多处计算采用建筑目标偏移 | 射击角度、目标线等机制考虑建筑 `TargetCoordOffset`，避免可配置命中点只在部分路径生效。 | L列表 · [Bugfixes and miscellaneous 第60项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH166 任务中可见盟友隐形对象 | 单人任务允许玩家看见盟友所属的隐形对象，使盟友隐形单位的呈现更符合协作关系。 | L列表 · [Bugfixes and miscellaneous 第61项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH167 长建起动画的地基显示 | 修正长建起动画中 `BibShape` 短暂提前绘制的问题，避免建造过程中出现地基闪现。 | L列表 · [Bugfixes and miscellaneous 第62项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH168 平铺动画支持自定义色盘 | 动画使用 `Tiled=yes` 时也可读取 `CustomPalette`，不再忽略该类动画的色盘配置。 | L列表 · [Bugfixes and miscellaneous 第63项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH169 所有权改变时清理旧命令 | 尝试减少心控、劫持等所有权变更后仍保留旧攻击、回收或驻军命令的情况；官方没有保证消除所有路径。 | L列表 · [Bugfixes and miscellaneous 第64项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH170 修正预置建筑自然粒子创建 | 调整地图预置建筑的 `NaturalParticleSystem` 创建处理，避免开局时无条件生成不合适的粒子系统。 | L列表 · [Bugfixes and miscellaneous 第65项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH171 跳跃单位坠毁倾斜与翻转 | 修正 `TiltCrashJumpjet=no` 时 Jumpjet 单位无法表现倾斜或翻转的问题。 | L列表 · [Bugfixes and miscellaneous 第66项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH172 取消替代枪口条目五个上限 | 单位可配置五条以上的 `AlternateFLH`，为更多乘员发射位置或替代枪口提供支持。 | L列表 · [Bugfixes and miscellaneous 第67项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH173 碎片上限包含最大值 | 弹头生成碎片时将 `MaxDebris` 本身作为数量上限，避免实际最大数量始终少一个。 | L列表 · [Bugfixes and miscellaneous 第68项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH174 双对空武器可选择主武器 | 主副武器都可对空时，允许选择主武器；Gattling 单位对应地允许采用奇数号武器槽。 | L列表 · [Bugfixes and miscellaneous 第69项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH175 盖特武器可回退到副槽 | Gattling 主槽因装甲、目标过滤或护盾等条件不能攻击时，可以回退到对应偶数号副武器槽。 | L列表 · [Bugfixes and miscellaneous 第70项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH176 陆地禁攻涵盖地形对象 | `LandTargeting=1` 也会阻止攻击陆地上的树木等 TerrainType，补齐对地形对象的目标限制。 | L列表 · [Bugfixes and miscellaneous 第71项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH177 水面禁攻涵盖空格和地形 | `NavalTargeting=6` 会阻止以空水格和水上地形对象为目标，补齐原先不一致的水面限制。 | L列表 · [Bugfixes and miscellaneous 第72项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH178 特殊目标限制不误用主武器 | 修正 `NavalTargeting=7` 或 `LandTargeting=2` 仍以主武器攻击陆地地形对象的问题。 | L列表 · [Bugfixes and miscellaneous 第73项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH179 可涉水步兵正确落入水中 | 能够正常入水但没有 `C4=true` 的步兵，空降或超时空进入水面后不再被错误杀死。 | L列表 · [Bugfixes and miscellaneous 第74项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH180 任务单独控制基地车重新部署 | 战役中可通过独立开关允许 MCV 重新部署，无需借用遭遇战设置 `MCVRedeploys`。 | L列表 · [Bugfixes and miscellaneous 第75项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH181 可收起建筑正常出售 | `UndeploysInto.Sellable=true` 可使适用的非基地建筑正常出售，并恢复收起型建筑出售时的结构出售语音。 | L列表 · [Bugfixes and miscellaneous 第76项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH182 水上建筑收起移动点正确 | `WaterBound=true` 的建筑收起成载具时，正确指定转换后载具应移动到的位置。 | L列表 · [Bugfixes and miscellaneous 第77项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH183 建筑允许零伤害并正确治疗 | `CanC4.AllowZeroDamage=true` 可取消 `CanC4=false` 的最低一点伤害；经过倍率后的负伤害始终绕过此最低伤害检查。 | L列表 · [Bugfixes and miscellaneous 第78项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH184 纯对空主武器显示攻击光标 | 建筑主武器使用 `AG=false` 弹体时仍有攻击光标，避免界面误示其完全不能攻击。 | L列表 · [Bugfixes and miscellaneous 第79项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH185 弹体可限制为纯对空 | `AAOnly=true` 可让 `AA=true` 弹体仅攻击空中目标；`AG=false` 本身仍只限制地面单元格目标。 | L列表 · [Bugfixes and miscellaneous 第80项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH186 敞篷乘员枪口不受载具连发干扰 | 运输载具自身武器 `Burst>1` 时，不再错误地按连发序号横移 OpenTopped 乘员的发射位置。 | L列表 · [Bugfixes and miscellaneous 第81项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH187 伪装步兵应用所需色盘 | 修正步兵伪装外观在需要自定义色盘时未正确应用的问题，避免伪装图像颜色异常。 | L列表 · [Bugfixes and miscellaneous 第82项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH188 伪装单位军衔按可见身份显示 | 伪装可见时按伪装类型及所属方显示军衔；观察者及启用伪装闪烁的玩家仍看到原单位军衔。 | L列表 · [Bugfixes and miscellaneous 第83项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH189 预置超武建筑不误播就绪动画 | 地图上预先放置的超武建筑，不再在开局错误显示 `SuperAnimThree`。 | L列表 · [Bugfixes and miscellaneous 第84项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH190 建筑升级可授予间谍卫星 | `SpySat=yes` 可以通过建筑升级应用，使卫星能力能够由升级提供而非仅由基础建筑提供。 | L列表 · [Bugfixes and miscellaneous 第85项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH191 AI 并行建造海陆载具 | AI 可与人类玩家一样，同时生产 `Naval=true` 与 `Naval=false` 的载具。 | L列表 · [Bugfixes and miscellaneous 第86项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH192 跳跃单位起步不突转方向 | 修正 Jumpjet 单位刚收到移动命令时突然转向右下方的表现，尤其可见于开局生成的跳跃单位。 | L列表 · [Bugfixes and miscellaneous 第87项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH193 自定义色盘支持完整所属颜色 | 使用自定义 `Palette` 的对象，无论所属颜色在 Colors 列表哪一部分，都能正确接受超武和地图染色。 | L列表 · [Bugfixes and miscellaneous 第88项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH194 展开攻击检查建筑放置规则 | `DeployToFire` 判断是否可以展开时考虑目标建筑的放置要求，改善 `WaterBound` 建筑展开异常。 | L列表 · [Bugfixes and miscellaneous 第89项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH195 展开受阻时重算地面位置 | 载具当前无法展开时，`DeployToFire` 会重新计算其在陆地上的适合位置，避免沿用错误位置。 | L列表 · [Bugfixes and miscellaneous 第90项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH196 围墙读取自定义色盘 | 围墙 Overlay 在适用时使用 artmd.ini 中 `Palette` 指定的色盘，不再一律采用默认颜色。 | L列表 · [Bugfixes and miscellaneous 第91项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH197 运输舱内可继续装填弹药 | `ReloadInTransport=true` 允许有 Ammo 的单位在舱内按 Reload/EmptyReload 计时装填，也可设置全局默认值。 | L列表 · [Bugfixes and miscellaneous 第92项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH198 治疗量可应用弹头倍率 | `ApplyModifiersOnNegativeDamage=true` 可让 Verses 与 PercentAtMax 作用于负伤害；支持弹头配置及全局默认。 | L列表 · [Bugfixes and miscellaneous 第93项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH199 空中附属动画及时更新图层 | 飞行单位的附属动画在父对象更新后立即调整图层；二者同层时，动画始终绘制在父对象之上。 | L列表 · [Bugfixes and miscellaneous 第94项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH200 占领后继续更新供电动画 | `Powered` 或 `PoweredSpecial` 建筑被敌方占领后，其供电相关动画仍会正确更新。 | L列表 · [Bugfixes and miscellaneous 第95项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH201 修正子机导弹目标设置 | 修正生成导弹的错误目标设置，并改变相应爆心；可用 `MissileSpawnAttackCell=false` 关闭此修正。 | L列表 · [Bugfixes and miscellaneous 第96项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH202 修正战役地图头部读取崩溃 | 修正读取战役地图 Header 节时可能发生的崩溃，避免任务载入在头部解析阶段中断。 | L列表 · [Bugfixes and miscellaneous 第97项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH203 EMP 状态炮塔停止转动 | 单位受 EMP 时不再旋转炮塔，使炮塔活动与瘫痪状态保持一致。 | L列表 · [Bugfixes and miscellaneous 第98项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH204 EMP 状态跳跃单位停止摇摆 | Jumpjet 单位受 EMP 时停止摇摆，避免瘫痪后仍保留正常悬浮的晃动表现。 | L列表 · [Bugfixes and miscellaneous 第99项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH205 跳跃坠毁不因建筑减速 | 移除 Jumpjet 单位坠向建筑时的不正常减速，使坠毁运动不再受该处特殊减速影响。 | L列表 · [Bugfixes and miscellaneous 第100项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH206 磁轨环境伤害跨越高度变化 | 使用 `IsRailgun=yes` 时，`AmbientDamage` 不再因为沿途地形高度变化而被提前截断。 | L列表 · [Bugfixes and miscellaneous 第101项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH207 磁轨和火焰粒子跨越高差 | 修正磁轨及火焰粒子被地形高度变化截断的问题，使显示与其传播路径更加一致。 | L列表 · [Bugfixes and miscellaneous 第102项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH208 读档保留传送后的冻结时间 | 传送单位的静止冻结计时在读档后正确恢复，避免例如超时空军团兵因此提前结束冻结。 | L列表 · [Bugfixes and miscellaneous 第103项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH209 传送单位在斜坡正确倾斜 | 使用传送运动方式的单位也可按斜坡显示倾斜，避免始终保持不合地形的水平外观。 | L列表 · [Bugfixes and miscellaneous 第104项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH210 修正火箭阴影位置 | 修正火箭影子的定位，使阴影按相应对象的位置关系绘制，避免明显错位。 | L列表 · [Bugfixes and miscellaneous 第105项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH211 更多运动方式支持视觉翻转 | Teleport、Tunnel、Fly 运动方式的单位也可像其他运动方式一样表现视觉翻转。 | L列表 · [Bugfixes and miscellaneous 第106项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH212 子机降落对齐母体朝向 | 生成的飞机返回降落时与生成者当前朝向对齐，减少回收阶段的方向不一致。 | L列表 · [Bugfixes and miscellaneous 第107项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH213 途经建筑不误触进入事件 | 无武装间谍、工程师或驻军步兵设置建筑航路点时，不再因错误执行占领任务而触发 EnteredBy。 | L列表 · [Bugfixes and miscellaneous 第108项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH214 升级建筑动画支持电力属性 | 建筑 `PowerUpN` 动画可使用 Powered 及 PoweredLight/Effect/Special，支持与供电状态关联的表现。 | L列表 · [Bugfixes and miscellaneous 第109项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH215 部署光标避免引发失步 | 修正鼠标悬停在选中的 `DeploysInto` 单位上时，光标显示可能触发联机不同步的问题。 | L列表 · [Bugfixes and miscellaneous 第110项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH216 工厂收起不绘制集合线 | 工厂建筑收起时跳过集合点连线，避免在收起过程中显示不合适的路径线。 | L列表 · [Bugfixes and miscellaneous 第111项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH217 各类对象正确应用染色 | SHP 载具、所有飞机以及不同位置的建筑动画，均能正确接收应有的染色效果。 | L列表 · [Bugfixes and miscellaneous 第112项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH218 无敌对象使用正确染色 | 铁幕或力场护盾作用中的对象始终使用其对应染色，避免两类无敌效果的颜色被错误应用。 | L列表 · [Bugfixes and miscellaneous 第113项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH219 无效坐标不参与中心计算 | 排除地图坐标无效的对象，避免其影响开局视野位置以及 AI 基地中心的计算。 | L列表 · [Bugfixes and miscellaneous 第114项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH220 不显形武器装填时仍可隐形 | 使用 `DecloakToFire=false` 武器的单位及建筑，在锁定目标与装填期间也能正常进入隐形。 | L列表 · [Bugfixes and miscellaneous 第115项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH221 探测器不揭露盟友建筑 | `Sensors=true` 单位不再揭露盟友建筑，避免探测机制错误改变友方建筑的隐形表现。 | L列表 · [Bugfixes and miscellaneous 第116项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH222 大范围查询包含空中单位 | 较大射程的目标扫描与大 CellSpread 弹头引爆，更可靠地纳入空中单位，减少漏选和漏伤。 | L列表 · [Bugfixes and miscellaneous 第117项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH223 桥上双方仍可正常对空 | 发射者与空中目标均位于桥上方时，使用 `AA=true` 弹体的武器仍能正常发射。 | L列表 · [Bugfixes and miscellaneous 第118项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH224 伪装目标自定义色盘正确 | 伪装目标具有自定义色盘时，伪装单位采用正确色盘，避免外观与被模仿对象不一致。 | L列表 · [Bugfixes and miscellaneous 第119项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH225 升级动画与升级层级一致 | 建筑升级尽可能使用与 `PowersUpToLevel` 对应的 `PowerUpN` 动画设置，减少层级映射不一致。 | L列表 · [Bugfixes and miscellaneous 第120项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH226 地下单位先出土再部署 | 地下或正在入地的单位不能直接部署开火或简单部署，而是先出土，再执行相应部署功能。 | L列表 · [Bugfixes and miscellaneous 第121项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH227 恢复 AI 电力盈余目标 | 启用 `EnablePowerSurplus` 可恢复 AI 盈余电力目标，并可用 ScaleToDrainAmount 按当前耗电量缩放。 | L列表 · [Bugfixes and miscellaneous 第122项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH228 显示受控单位规划路径 | 规划路径可显示给玩家控制的所有单位；单人模式也可由 `DebugPlanningPaths=yes` 扩大调试显示。 | L列表 · [Bugfixes and miscellaneous 第123项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH229 超时空攻击奴隶步兵不崩溃 | 修正 Temporal 弹头攻击 `Slaved=true` 步兵时可能导致的崩溃。 | L列表 · [Bugfixes and miscellaneous 第124项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH230 高速运动不再卡死 | 修正 Tunnel、Walk、Mech 等运动方式在速度过快时可能无法继续移动的问题。 | L列表 · [Bugfixes and miscellaneous 第125项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH231 生成步兵动画接受地块光照 | 使用单位色盘且 `UseNormalLight=false` 的 MakeInfantry 动画，会应用所在单元格的光照变化。 | L列表 · [Bugfixes and miscellaneous 第126项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH232 移除跳跃步兵零伤害副作用 | `InfDeath=9` 弹头对 Jumpjet 步兵造成零伤害时，不再产生原有不合适的效果。 | L列表 · [Bugfixes and miscellaneous 第127项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH233 核弹与心控全图光照作用飞机 | 修正 Nuke、Dominator 级环境光照未应用到 AircraftType 的问题，使飞机也接受对应光照变化。 | L列表 · [Bugfixes and miscellaneous 第128项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH234 无武装对象跳过目标扫描 | 对没有武器的 Techno 跳过目标扫描调用，减少无意义的目标查找工作。 | L列表 · [Bugfixes and miscellaneous 第129项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH235 空爆弹体保留武器类型 | AirburstWeapon 创建的弹体记录来源 WeaponType，因此能够执行辐射等依赖武器信息的效果。 | L列表 · [Bugfixes and miscellaneous 第130项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH236 装填机场正确维修受损飞机 | 受损飞机在 `UnitReload=true` 对接点的维修不再依赖必须先完成一次落靠，修正相应维修遗漏。 | L列表 · [Bugfixes and miscellaneous 第131项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH237 月球地形可读取更多地块索引 | `ApplyLunarFixes=true` 可读取月球剧场的岸、水、崖及桥索引；启用前须修正指向空地块集等无效配置。 | L列表 · [Bugfixes and miscellaneous 第132项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH238 允许攻击盟友时保留目标 | `AttackFriendlies=true` 且当前目标为盟友的对象，不再每帧清空目标，尤其改善 AI 建筑的持续攻击。 | L列表 · [Bugfixes and miscellaneous 第133项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH239 地图跟随载具编号不被打乱 | 预置载具的 Follower 索引严格对应地图 Units 列表，不再被创建失败或初始乘员生成的载具扰乱。 | L列表 · [Bugfixes and miscellaneous 第134项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH240 解除运动器叠加后恢复供能 | Drive、Jumpjet、Ship、Teleport 运动器脱离 piggyback 包装时正确恢复启用状态，避免单位失去正常移动能力。 | L列表 · [Bugfixes and miscellaneous 第135项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH241 统一停止命令的多种异常 | 停止命令可正确结束攻击移动、区域警戒及桥上任务，并修正 Jumpjet 和飞机的停车、返航异常；桥下仍禁止停止移动。 | L列表 · [Bugfixes and miscellaneous 第136项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH242 空中队伍按平面距离到点 | 空中队员以二维距离判断是否到达任务地点，避免停在建筑上方等情况下因高度差导致脚本停滞。 | L列表 · [Bugfixes and miscellaneous 第137项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH243 单位速度支持小数并规范换算 | Speed 接受小数，按文档的百分比到内部 0—255 速度值流程截断与限幅；并非直接将小数当内部速度。 | L列表 · [Bugfixes and miscellaneous 第138项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH244 空爆武器支持更多射击效果 | AirburstWeapon 支持 IsLaser、IsElectricBolt、IsRadBeam 与 AttachedParticleSystem，使空爆分支可绘制相应效果。 | L列表 · [Bugfixes and miscellaneous 第139项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH245 地下移动应用速度加成 | 地下行进速度也应用老兵、附加效果等来源的速度倍率，避免地下阶段忽略这些修正。 | L列表 · [Bugfixes and miscellaneous 第140项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH246 飞机依运动属性选择表面 | 飞机按 MovementZone 与 SpeedType 处理不同表面的移动，修正点选水面却异常转向附近岸边的问题。 | L列表 · [Bugfixes and miscellaneous 第141项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH247 寄生未命中时不凭空消失 | 寄生物攻击未命中且原单元格被占用时，避免其直接消失，修正失败攻击后的对象保留。 | L列表 · [Bugfixes and miscellaneous 第142项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH248 限制问题运动器进入坦克碉堡 | 阻止使用不兼容运动器的单位进入坦克碉堡，避免这些组合产生已知运行问题。 | L列表 · [Bugfixes and miscellaneous 第143项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH249 超时空碰撞不留隐形障碍 | 超时空传送到不可碾压单位上而自毁时，正确清理原位置占用，避免留下无法通行的隐形屏障。 | L列表 · [Bugfixes and miscellaneous 第144项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH250 死亡单位正确清理探测范围 | 单位被摧毁时移除其残留 Sensors 影响，避免死亡后仍保留探测作用。 | L列表 · [Bugfixes and miscellaneous 第145项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH251 免费单位按自身速度类型找位置 | FreeUnit 使用生成单位自身的 SpeedType 搜索出生位置，减少按错误移动属性选取不适合地块。 | L列表 · [Bugfixes and miscellaneous 第146项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH252 开局海军单位不生成在陆地 | 修正海军载具作为 StartUnit 时可能出生在陆地上的问题，使起始位置符合其通行属性。 | L列表 · [Bugfixes and miscellaneous 第147项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH253 收起后按目标载具判断通行 | 建筑收起时，使用目标 VehicleType 的 SpeedType 与 MovementZone 判断能否进入目的格。 | L列表 · [Bugfixes and miscellaneous 第148项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH254 两栖矿车可返回水上矿厂 | 修正两栖移动矿车不能自动返回 WaterBound 矿厂的问题；Teleporter 单位不属于此项修正路径。 | L列表 · [Bugfixes and miscellaneous 第149项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH255 未建成工厂不提前推出单位 | 工厂仍处于建造阶段时不推出单位，避免单位因此长期困在工厂内部。 | L列表 · [Bugfixes and miscellaneous 第150项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH256 修正过早移除对象引用 | 单位隐形或进入运输载具后，先前发出的子机、C4 或弹体等仍可按正确引用关系归属经验。 | L列表 · [Bugfixes and miscellaneous 第151项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH257 远距离声波绘制避免崩溃 | 修正 IsSonic 波传播距离超过一定范围时的绘制崩溃，提升长距离声波武器的稳定性。 | L列表 · [Bugfixes and miscellaneous 第152项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH258 大占地建筑正确回收子机 | 占地超过 1×1 的建筑也能正常回收所生成的对象，避免生成者尺寸导致回收失败。 | L列表 · [Bugfixes and miscellaneous 第153项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH259 多条电弧持续跟随枪口 | 需跟随单位当前枪口坐标的电弧，在同一时刻存在多条时也正确更新位置，不再只更新其中一条。 | L列表 · [Bugfixes and miscellaneous 第154项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH260 修正部分射角失效情况 | 修正 `FireAngle` 在某些条件下不正确工作的情况，使配置射角在相应路径中生效。 | L列表 · [Bugfixes and miscellaneous 第155项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH261 两栖破坏型单位可进入水上建筑 | 使用 AmphibiousDestroyer 或 AmphibiousCrusher 移动区域的对象可正常进入适用水上建筑。 | L列表 · [Bugfixes and miscellaneous 第156项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH262 桥下航母正确选择攻击位置 | 航母独自位于高架桥下时，可以寻找合适的攻击位置，避免目标定位失败阻断攻击。 | L列表 · [Bugfixes and miscellaneous 第157项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH263 空中航母近桥仍可攻击 | 修正空中航母靠近高架桥时不能攻击的问题，避免桥梁邻近关系错误阻断子机出击。 | L列表 · [Bugfixes and miscellaneous 第158项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH264 桥上航母可回收舰载机 | 航母位于桥上时仍能正常收回生成的飞机，修正桥层位置与回收判断的不一致。 | L列表 · [Bugfixes and miscellaneous 第159项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH265 桥上跳跃单位阴影定位正确 | Jumpjet 位于高架桥上方时，阴影不再错误停留在地面高度，改善桥层与阴影位置关系。 | L列表 · [Bugfixes and miscellaneous 第160项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH266 瞬时武器效果支持散布 | 激光、电弧与辐射束配合 `Inviso=true`、`FlakScatter=true` 弹体时，也能表现配置的散布。 | L列表 · [Bugfixes and miscellaneous 第161项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH267 治疗武器自动搜寻空中目标 | 修正治疗武器不能自动获取空中目标的问题，使适用空中单位可以进入治疗目标选择。 | L列表 · [Bugfixes and miscellaneous 第162项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH268 体素弹体支持动画和发射方色盘 | 体素弹体可使用 AnimPalette 与 FirersPalette，扩展其色盘选择并保持相应颜色来源。 | L列表 · [Bugfixes and miscellaneous 第163项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH269 AI 不再锁定不可达功能建筑 | AI 搜索坦克碉堡、生化反应炉等建筑时排除无法抵达的目标，避免持续卡在无效寻路。 | L列表 · [Bugfixes and miscellaneous 第164项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH270 修正运输船首位登船者 | 地面单位呼叫运输船登船时，首个呼叫者仍会正常上船，避免其被登船流程遗漏。 | L列表 · [Bugfixes and miscellaneous 第165项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH271 读取单位自定义进出建筑音效 | TechnoType 上的 EnterBioReactorSound、LeaveBioReactorSound、EnterGrinderSound 能够按配置使用。 | L列表 · [Bugfixes and miscellaneous 第166项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH272 被牵引矿车正确中断卸矿 | 矿车受 IsLocomotor 弹头抬起时正确处理卸货状态，避免持续卸货或之后无法再卸货。 | L列表 · [Bugfixes and miscellaneous 第167项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH273 修正斜坡单位过度倾斜 | 单位位于斜坡时不再出现过大的视觉倾斜角度，使姿态变化与坡面更加一致。 | L列表 · [Bugfixes and miscellaneous 第168项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH274 连续登车不留隐形障碍 | 修正步兵连续进入载具时产生不可通行隐形屏障的问题，正确处理相关位置占用。 | L列表 · [Bugfixes and miscellaneous 第169项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH275 桥上传送登车正确清理占用 | 传送单位在桥上进入运输载具时，不再产生可能导致卡死或崩溃的隐形阻挡。 | L列表 · [Bugfixes and miscellaneous 第170项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH276 传送基地车移动不引发失步 | 修正使用 Teleport 运动方式的 MCV 移动时引起联机重连错误的问题。 | L列表 · [Bugfixes and miscellaneous 第171项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH277 悬浮载具被抬起时阴影正确 | 使用 Hover 运动器的载具受到 IsLocomotor 弹头抬升时，修正其阴影绘制位置。 | L列表 · [Bugfixes and miscellaneous 第172项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH278 驻军目标提前移除不崩溃 | 步兵即将进入可驻军建筑，而建筑已移除但未完成死亡处理时，避免访问失效对象导致崩溃。 | L列表 · [Bugfixes and miscellaneous 第173项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH279 子机提前移除不崩溃 | 生成对象已被移除但尚未完成实际死亡处理时，正确处理残留关联，避免游戏崩溃。 | L列表 · [Bugfixes and miscellaneous 第174项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH280 区分空袭发起方与目标引用 | 将攻击者或飞机和空袭目标使用的 AirstrikeClass 指针分离，避免两侧引用互相覆盖。 | L列表 · [Bugfixes and miscellaneous 第175项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH281 修正建筑空袭标色所属方 | 修正建筑总被染成空袭所属方颜色的问题，使目标染色不再错误继承该归属。 | L列表 · [Bugfixes and miscellaneous 第176项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH282 AI 击败或结盟后重新找敌人 | AI 当前敌人被击败或与其结盟后，会继续搜索新敌人，避免保留失效的敌对目标。 | L列表 · [Bugfixes and miscellaneous 第177项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH283 步兵入建筑遵守容量体积 | 步兵进入建筑时也检查 Passengers 与 SizeLimit，避免绕过建筑的乘员数量及体积限制。 | L列表 · [Bugfixes and miscellaneous 第178项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH284 快捷部署播放单位部署语音 | 通过快捷键或命令栏部署单位时也播放 VoiceDeploy，使音效反馈与其他部署操作一致。 | L列表 · [Bugfixes and miscellaneous 第179项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH285 舰船不能驶上高架桥 | 修正舰船能在高架桥面行驶的问题，使桥层通行判断符合舰船的移动属性。 | L列表 · [Bugfixes and miscellaneous 第180项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH286 修正向低处空袭标线裁切 | 空袭指示线指向较低地形目标时正确绘制，避免高差造成的线条裁切。 | L列表 · [Bugfixes and miscellaneous 第181项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH287 精英单位散开由能力控制 | 精英对象默认不再自动获得散开行为，改由 SCATTER 老兵能力明确控制。 | L列表 · [Bugfixes and miscellaneous 第182项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH288 电力支援副武器不无条件攻击 | `ElectricAssault=yes` 的副武器不再无条件攻击己方 Overpowerable 建筑，避免忽略本应检查的条件。 | L列表 · [Bugfixes and miscellaneous 第183项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH289 步兵可使用盖特武器逻辑 | Infantry 支持 `IsGattling=yes`，可应用盖特武器随阶段变化的相关机制。 | L列表 · [Bugfixes and miscellaneous 第184项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH290 范围伤害跨越桥面和地面 | 修正桥上或地面引爆的范围伤害不能影响另一层对象的问题，使适用跨层目标正常受影响。 | L列表 · [Bugfixes and miscellaneous 第185项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH291 空军支持自身伤害设置 | 修正 DamageSelf 与 AllowDamageOnSelf 对空中单位不生效的问题，使自伤规则按配置工作。 | L列表 · [Bugfixes and miscellaneous 第186项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH292 工程师修复清除破损粒子 | 建筑被工程师修复后正确移除原有受损粒子，避免建筑已完好仍持续显示损坏效果。 | L列表 · [Bugfixes and miscellaneous 第187项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH293 修正体素动画尾迹位置 | 修正 VoxelAnim 中 TrailerAnim 的错误坐标，使尾迹与体素动画的当前位置对应。 | L列表 · [Bugfixes and miscellaneous 第188项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH294 敞篷传送距离考虑建筑目标 | 修正目标为建筑时 OpenToppedWarpDistance 的计算，避免运输乘员传送攻击采用错误距离。 | L列表 · [Bugfixes and miscellaneous 第189项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH295 飞行矿车可手动返回矿厂 | MovementZone=Fly 的矿车可以接收手动进入矿厂的命令，不再被错误的进入判断阻止。 | L列表 · [Bugfixes and miscellaneous 第190项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH296 跳跃矿车出厂自动采矿 | Jumpjet 矿车离开战车工厂后能自动开始采矿，修正生产完成后的任务衔接。 | L列表 · [Bugfixes and miscellaneous 第191项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH297 跳跃矿车入矿厂不重叠崩溃 | 修正 Jumpjet 矿车手动进入矿厂时重叠并导致崩溃的问题，协调对接及位置状态。 | L列表 · [Bugfixes and miscellaneous 第192项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH298 母体 EMP 不再让子机飞出地图 | 生成者受 EMP 时，其生成飞机不再错误飞向地图边缘，修正瘫痪状态下的子机控制。 | L列表 · [Bugfixes and miscellaneous 第193项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH299 飞机垂直弹体向下落 | Aircraft 发射 Vertical 弹体时按垂直下落处理，避免异常轨迹；可用 Vertical.AircraftFix=false 关闭。 | L列表 · [Bugfixes and miscellaneous 第194项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH300 工程师进入无需维修的建筑 | 建筑不需要修复时工程师仍可正常进入，也可按 Alt 强制执行相应进入操作。 | L列表 · [Bugfixes and miscellaneous 第195项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH301 玩家间谍不被强制改派任务 | 玩家控制的间谍对建筑执行攻击操作时，不再被强制切换到其他任务。 | L列表 · [Bugfixes and miscellaneous 第196项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH302 无拆弹弹头不显示拆弹动作 | 工程师所有武器弹头均不具 BombDisarm 能力时，不再提供对应拆弹鼠标操作，避免界面误导。 | L列表 · [Bugfixes and miscellaneous 第197项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH303 修正步兵部署武器异常用法 | 调整 InfantryType 的 DeployFireWeapon 处理，修正文档指出的异常使用路径。 | L列表 · [Bugfixes and miscellaneous 第198项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH304 运输载具冻结时停止乘员抹除 | OpenTopped 载具被 Temporal 武器冻结后，其乘员的 Temporal 攻击也正确停止。 | L列表 · [Bugfixes and miscellaneous 第199项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH305 AI 载具隐形不自动散开 | 修正电脑所属载具在进入隐形时错误散开的行为，避免隐形操作干扰原有位置或任务。 | L列表 · [Bugfixes and miscellaneous 第200项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH306 地图改所属不让潜艇固定左转 | 潜艇经地图事件改变所属方后不再总向左转，修正所有权转换带来的朝向异常。 | L列表 · [Bugfixes and miscellaneous 第201项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH307 快捷撤出驻军刷新建筑 | 部署快捷键清空驻军后立即更新建筑绘制，避免已撤出的驻军仍影响建筑显示。 | L列表 · [Bugfixes and miscellaneous 第202项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH308 驻军无处撤出时不消失 | 建筑周围没有空位时，撤出流程不再让驻军凭空消失，避免把放置失败当成已成功离开。 | L列表 · [Bugfixes and miscellaneous 第203项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH309 牵引者被冻结时终止牵引 | Locomotor 武器的攻击者受到 Temporal 弹头作用时，正确停止正在持续的运动器牵引效果。 | L列表 · [Bugfixes and miscellaneous 第204项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH310 桥上悬浮目标保持可交互 | 修正 IsLocomotor 弹头使高架桥上 Hover 单位变得无法选择、无法受伤的问题。 | L列表 · [Bugfixes and miscellaneous 第205项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH311 非载具停止射击也终止牵引 | 除载具外的发射者停止开火时，也会结束 Locomotor 弹头效果，避免牵引失去正常终止条件。 | L列表 · [Bugfixes and miscellaneous 第206项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH312 悬浮载具桥上死亡不沉没 | 修正 Hover 载具在桥上被摧毁时错误进入沉没流程的问题。 | L列表 · [Bugfixes and miscellaneous 第207项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH313 装填中攻击光标遵守目标条件 | 修正选中单位处于重新装填状态时，对任意目标都无条件显示攻击操作的问题。 | L列表 · [Bugfixes and miscellaneous 第208项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH314 地图覆盖不混淆炮管炮塔动画 | 地图中存在对象类型节时，炮管动画数据不再被炮塔动画数据错误覆盖。 | L列表 · [Bugfixes and miscellaneous 第209项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH315 扩大大地图寻路节点缓冲 | 修正大地图因寻路节点缓冲过小导致的崩溃，为相关路径搜索提供足够的节点容量。 | L列表 · [Bugfixes and miscellaneous 第210项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH316 悬浮简单部署尊重不落地设置 | IsSimpleDeployer、BalloonHover 单位设置 DeployToLand=false 时，不再在悬浮阶段被强制降落。 | L列表 · [Bugfixes and miscellaneous 第211项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH317 空中部署动画阴影落在地面 | 空中单位播放带 Shadow 的 DeployingAnim 时，将阴影绘制到对应地面位置而非错误高度。 | L列表 · [Bugfixes and miscellaneous 第212项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH318 部署动画支持归一化与倒放 | DeployingAnim 支持 Normalized 与 Reverse；倒放的帧数使用 LoopEnd，即使没有多次循环也如此。 | L列表 · [Bugfixes and miscellaneous 第213项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH319 部署动画随单位染色 | 采用单位绘制器的 DeployingAnim 也应用单位对应的染色，保持转换过程的视觉一致。 | L列表 · [Bugfixes and miscellaneous 第214项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH320 空中跳跃单位正确生成导弹 | 修正 Jumpjet 在空中无法正确生成导弹的问题，使生成型武器不再被错误的位置状态阻断。 | L列表 · [Bugfixes and miscellaneous 第215项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH321 规划节点悬停状态及时更新 | 通过快捷键改变所选对象等操作后，当前悬停规划节点会及时刷新，避免继续使用过期状态。 | L列表 · [Bugfixes and miscellaneous 第216项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH322 跳跃载具射程内正确停止 | Jumpjet 载具获得射程内目标时能正确停下，不再因目标任务更新而异常继续移动。 | L列表 · [Bugfixes and miscellaneous 第217项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH323 跳跃步兵射程外继续接近 | 修正 Jumpjet 步兵获得射程外目标时错误停车的问题，使其可按攻击需求继续接近。 | L列表 · [Bugfixes and miscellaneous 第218项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH324 隐形跳跃步兵隐藏阴影 | Jumpjet 步兵处于隐形状态时不再始终绘制影子，避免阴影直接暴露不可见对象。 | L列表 · [Bugfixes and miscellaneous 第219项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH325 非对接任务不驶向建筑对接口 | 对象未准备执行对接时，不再错误朝建筑 dock 位置移动，避免无关任务被对接逻辑干扰。 | L列表 · [Bugfixes and miscellaneous 第220项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH326 狂暴跳跃载具可以正常停下 | 修正 Jumpjet 载具进入狂暴后不能正确停止的问题，保持该状态下的移动控制一致。 | L列表 · [Bugfixes and miscellaneous 第221项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH327 步兵穿树不破坏他方通行 | 步兵经过有树的单元格后，不再令该格对其他所属方不可通行，避免占用标记残留。 | L列表 · [Bugfixes and miscellaneous 第222项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH328 地形伪装对象染色正确 | 伪装成 Terrain 或 Overlay 的单位，铁幕与空袭染色的颜色和强度不再被错误应用。 | L列表 · [Bugfixes and miscellaneous 第223项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH329 友伤和无伤不使 AI 进入战斗 | AI 建筑受到友方伤害或不大于零的伤害时，不再错误触发其战斗状态。 | L列表 · [Bugfixes and miscellaneous 第224项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH330 纯对空武器可搜寻下落单位 | 开启 FallingDownTargetingFix 后，AA=yes、AG=no 的武器可自动搜寻伞兵等正在下落的目标。 | L列表 · [Bugfixes and miscellaneous 第225项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH331 SHP 炮塔载具支持铁幕染色 | 带炮塔的 SHP 载具可正确表现铁幕及自定义染色，补齐其绘制路径中的效果支持。 | L列表 · [Bugfixes and miscellaneous 第226项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH332 恢复未使用的触发事件 | 重新启用原先未使用的地图触发事件 2、53、54；具体参数与语义仍应对照对应事件定义。 | L列表 · [Bugfixes and miscellaneous 第227项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH333 载具落向步兵不清空整格对象 | 修正载具落到步兵上时将该单元格所有内容移除的问题，避免波及同格其他对象。 | L列表 · [Bugfixes and miscellaneous 第228项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH334 建造中改所属保留建起流程 | 建筑在建起动画期间改变所属方，不再跳过建起流程或留下未正确清理的建造状态。 | L列表 · [Bugfixes and miscellaneous 第229项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH335 可见区外预置飞机不误判坠毁 | 修正预置在可见地图范围外的飞机在特定条件下被错误标记为正在坠毁的问题。 | L列表 · [Bugfixes and miscellaneous 第230项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH336 地下矿车跨封闭区域寻找矿厂 | Subterranean 矿车即使位于被水域、悬崖等包围的区域，也能正确寻找可对接矿厂。 | L列表 · [Bugfixes and miscellaneous 第231项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH337 改所属正确清理对象关联效果 | 对象改变所属方时清理相关指向效果，避免部分效果仍按旧关系引用或影响该单位。 | L列表 · [Bugfixes and miscellaneous 第232项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH338 战役其他玩家可使用揭图箱 | 战役中另一玩家控制的所属方拾取 Reveal 箱子时，也可获得对应揭图效果。 | L列表 · [Bugfixes and miscellaneous 第233项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH339 AI 脚本可纳入跳跃单位 | 启用 AIAirTargetingFix 后修正原版脚本忽略 Jumpjet 的行为，使适用跳跃单位参与对应判断。 | L列表 · [Bugfixes and miscellaneous 第234项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH340 空中舰船死亡不错误沉没 | 海军舰船在空中被摧毁时不再仍执行水面沉没流程，使死亡处理符合实际所在状态。 | L列表 · [Bugfixes and miscellaneous 第235项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH341 隐藏调试信息同时隐藏计时器 | 关闭调试可见性时也不显示 MPDebug 计时器，避免隐藏设置只作用于部分调试内容。 | L列表 · [Bugfixes and miscellaneous 第236项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH342 最小射程内不逃向最远位置 | 目标距离小于 MinimumRange 时，单位不再错误地前往极远位置，修正退避目的地选择。 | L列表 · [Bugfixes and miscellaneous 第237项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH343 不可训练对象不获触发晋升 | 触发动作创建 Trainable=false 对象时，不再错误地为其授予晋升等级。 | L列表 · [Bugfixes and miscellaneous 第238项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH344 攻击友方隐形目标不强制探测器 | 修正非维修单位攻击友方隐形单位时必须具备 Sensors 的额外限制，保持该目标操作一致。 | L列表 · [Bugfixes and miscellaneous 第239项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH345 火箭爬升考虑目标高度 | 火箭在爬升阶段纳入目的地海拔，避免只按未考虑目标高度的轨迹计算。 | L列表 · [Bugfixes and miscellaneous 第240项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH346 更新中删除不跳过下个对象 | 对象在 Update 中从 LogicClass 移除时，正确继续迭代，不再漏掉紧接着的对象更新。 | L列表 · [Bugfixes and miscellaneous 第241项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH347 AI 不积累对友方的敌意 | 修正 AI 向友方所属方设置怒气的问题，避免错误敌意干扰其正常决策。 | L列表 · [Bugfixes and miscellaneous 第242项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH348 AI 无敌人时优先最近目标 | AI 没有现存敌人时不再直接采用数组首位所属方，而按相应距离规则寻找邻近敌人。 | L列表 · [Bugfixes and miscellaneous 第243项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH349 副武器选择检查运动器弹头 | 武器选择阶段检查副武器弹头的 IsLocomotor，避免忽略牵引类武器的特殊条件。 | L列表 · [Bugfixes and miscellaneous 第244项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH350 坦克碉堡内载具不受牵引 | 禁止对坦克碉堡中的载具使用 IsLocomotor 弹头，避免对受驻入状态约束的对象强行牵引。 | L列表 · [Bugfixes and miscellaneous 第245项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH351 带乘员或部署攻击矿车可卸矿 | 修正受 Passengers 或 DeployFire 设置影响的矿车无法正常卸矿的问题。 | L列表 · [Bugfixes and miscellaneous 第246项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH352 矿车离开坦克碉堡可移动 | 矿车退出坦克碉堡后恢复正常移动，避免沿用驻入阶段的状态而卡住。 | L列表 · [Bugfixes and miscellaneous 第247项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH353 更多工厂支持生产动画 | 步兵工厂和非建造厂型建筑工厂可使用 ProductionAnim，播放期间 IdleAnim 也按规则暂停。 | L列表 · [Bugfixes and miscellaneous 第248项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH354 永久心控替换不丢失选择 | 普通心控被永久心控或心灵支配器控制替换时，已选对象不再因此意外失去选中状态。 | L列表 · [Bugfixes and miscellaneous 第249项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH355 超时空不错误传送到原点 | 修正 ChronoSphere 将对象错误传送至地图单元格 0,0 的问题，避免目的地计算异常。 | L列表 · [Bugfixes and miscellaneous 第250项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH356 攻击移动不追逐无法攻击目标 | 攻击移动中的对象无法攻击某目标时，不再仍朝该目标移动，避免不可攻击对象牵引行军。 | L列表 · [Bugfixes and miscellaneous 第251项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH357 AI 超时空脚本不写死超武序号 | 修正脚本 56、57 强制使用序号 3、4 超武的问题，使其不再依赖特定超武列表顺序。 | L列表 · [Bugfixes and miscellaneous 第252项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH358 被动阵营待工程师建筑无威胁 | NeedsEngineer 建筑属于 MultiplayPassive 所属方时，威胁按零处理，不再采用其 ThreatPosed 值。 | L列表 · [Bugfixes and miscellaneous 第253项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH359 载具重叠围墙不出现出售操作 | 载具与 Wall Overlay 重叠时，不再显示出售光标或允许出售载具，避免墙体售卖判断串入载具。 | L列表 · [Bugfixes and miscellaneous 第254项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH360 树木伪装不泄漏军衔标记 | 修正伪装成树木的载具在不应显示军衔时仍绘制标记的问题，避免通过军衔暴露伪装。 | L列表 · [Bugfixes and miscellaneous 第255项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH361 修正 AI 常规目标选择 | 修正 AI 常规目标选择将可驻军建筑也作为目标的相应问题，避免目标分类混入不适用对象。 | L列表 · [Bugfixes and miscellaneous 第256项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH362 跳跃移动任务正确结束 | 修正 Jumpjet 移动任务未正确结束的问题，使到达及任务切换能正常完成。 | L列表 · [Bugfixes and miscellaneous 第257项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH363 AI 驻军失效后立即重选 | 驻军目的地不再可驻军时，AI 队伍脚本立即重新评估目标，不先尝试进入失效建筑。 | L列表 · [Bugfixes and miscellaneous 第258项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH364 展开收起不损失一点生命 | 受损对象在 DeploysInto 与 UndeploysInto 转换时，不再因换算错误额外丢失一点生命值。 | L列表 · [Bugfixes and miscellaneous 第259项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH365 人类单位及时切换后续任务 | 人类控制对象可在移动中更及时接受下一任务，改善 Jumpjet/Hover；ReadyToNextMission.MovingCheck=true 可恢复旧检查。 | L列表 · [Bugfixes and miscellaneous 第260项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH366 降落中断桥不误杀伞降单位 | 单位伞降期间下方桥梁被摧毁，落地时不再因过期桥面状态被错误杀死。 | L列表 · [Bugfixes and miscellaneous 第261项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH367 跳过完全不可见的体素部件 | 体素变换矩阵三轴缩放均为零的部件不再进入绘制，减少部分体素模型的无效渲染工作。 | L列表 · [Bugfixes and miscellaneous 第262项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH368 被牵引放下不播放坠毁语音 | 单位被 IsLocomotor 弹头放下时，不再误播坠毁语音及音效，避免将释放误判为坠毁。 | L列表 · [Bugfixes and miscellaneous 第263项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH369 反击不在等威胁目标间抖动 | 修正反击逻辑在多个同等威胁目标间不断切换的问题，使当前攻击选择更稳定。 | L列表 · [Bugfixes and miscellaneous 第264项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH370 伞降可坠毁对象正确死亡 | Crashable=yes 的伞降对象在空中被摧毁后，会正确完成死亡流程，避免落到地面仍不死亡。 | L列表 · [Bugfixes and miscellaneous 第265项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH371 非人类伞兵尊重禁止坠毁 | NotHuman 步兵伞降时被击杀，也会尊重 Crashable=no，不再强制按坠毁方式落地。 | L列表 · [Bugfixes and miscellaneous 第266项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH372 修正跳跃牵引悬浮目标落点 | Jumpjet 运动器弹头牵引 BalloonHover 目标时，避免错误地让目标从攻击者头顶上方跌落。 | L列表 · [Bugfixes and miscellaneous 第267项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH373 低生命建筑维修避免崩溃 | Strength 小于 RepairStep 时，维修流程不再触发相应崩溃，正确处理单次维修量大于生命上限的情况。 | L列表 · [Bugfixes and miscellaneous 第268项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH374 矿石再生后矿车恢复工作 | 非 Teleporter 矿车在矿石枯竭后又再生时，可重新开始采集，不再长期停留在闲置状态。 | L列表 · [Bugfixes and miscellaneous 第269项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH375 地图外坠毁对象完成清理 | Fly、Jumpjet、Rocket 对象在地图外坠毁后彻底清理，避免永久占用生产槽或单位数量限制。 | L列表 · [Bugfixes and miscellaneous 第270项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH376 裂缝与卫星交互避免失步 | 修正 GapGenerator 与 SpySat 交互造成战争迷雾状态不一致的问题，避免由此引发联机失步。 | L列表 · [Bugfixes and miscellaneous 第271项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH377 矿石耗尽仍保留采矿任务 | 矿车不会仅因矿物耗尽而退出 Harvest，之后会周期性尝试恢复采矿，适应矿物再生。 | L列表 · [Bugfixes and miscellaneous 第272项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH378 修正步兵空闲模式任务切换 | 调整步兵 EnterIdleMode 中错误的任务切换，使进入空闲状态时不再转入不适当任务。 | L列表 · [Bugfixes and miscellaneous 第273项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH379 悬浮寻路可忽略地面障碍因素 | BalloonHoverPathingFix=true 修正 BalloonHover 对象设置目的地及分发移动命令时错误考虑地面因素的问题。 | L列表 · [Bugfixes and miscellaneous 第274项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH380 暂存空间死亡刷新侧栏 | 单位在 limbo 状态死亡时也正确刷新侧栏，避免界面沿用已过期的生产或可用状态。 | L列表 · [Bugfixes and miscellaneous 第275项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH381 非战役模式可播放侧栏影片 | 地图动作 100、117 可在非战役模式播放侧栏影片，并按动作定义选择是否暂停。 | L列表 · [Bugfixes and miscellaneous 第276项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH382 电力支援自动寻找盟友防御 | ElectricAssault 武器可自动获取盟友可充能防御建筑，使电力支援不只依赖手动目标操作。 | L列表 · [Bugfixes and miscellaneous 第277项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH383 区域警戒更及时重新索敌 | 区域警戒消灭目标后，不再比其他任务明显更晚重新搜寻目标，修正不一致的索敌等待。 | L列表 · [Bugfixes and miscellaneous 第278项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH384 纯视觉对象不参与同步校验 | 纯视觉动画与粒子不计入帧 CRC，避免其呈现差异引起联机同步校验；并非排除有逻辑作用的效果。 | L列表 · [Bugfixes and miscellaneous 第279项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH385 低空起降对象不重复受伤 | 修正正在低空起飞或降落的对象可能被重复计伤的问题，避免同次效果结算两份伤害。 | L列表 · [Bugfixes and miscellaneous 第280项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH386 超武辅助建筑条件包含升级 | AuxBuilding 条件能够计入建筑升级，使升级对象也可满足对应超武的辅助建筑需求。 | L列表 · [Bugfixes and miscellaneous 第281项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH387 修正体素弹体与动画光照 | 调整体素弹体和体素动画的光照处理，修正其与场景照明不一致的相应问题。 | L列表 · [Bugfixes and miscellaneous 第282项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH388 隧道内改所属不再卡死 | 对象在地下隧道状态改变所属方后，能够继续正常工作，避免运动状态与所有权转换冲突。 | L列表 · [Bugfixes and miscellaneous 第283项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH389 侧栏部署恢复通用音效 | 从侧栏执行 Deploy 时恢复播放 AudioVisual.DeploySound 的原有 TS 行为，补齐部署操作反馈。 | L列表 · [Bugfixes and miscellaneous 第284项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH390 敞篷乘员遵守移动开火限制 | 乘员能否从 OpenTopped 运输载具射击，会尊重其武器 FireWhileMoving，避免移动中绕过开火限制。 | L列表 · [Bugfixes and miscellaneous 第285项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH391 更多飞行对象阴影位置正确 | 修正非 Aircraft 的 Fly 单位，以及被 IsLocomotor 弹头牵引的 Aircraft，其阴影使用错误位置的问题。 | L列表 · [Bugfixes and miscellaneous 第286项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH392 生成单位不接受不适用玩家命令 | 修正子机或奴隶执行某些玩家命令的问题，避免这类从属对象绕过其专属控制流程。 | L列表 · [Bugfixes and miscellaneous 第287项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH393 停用对象清理建筑连接 | Techno 停用时正确重置与所连接建筑的关系，避免保留不再有效的连接状态。 | L列表 · [Bugfixes and miscellaneous 第288项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH394 带炮塔建筑支持全向开火 | `Turret=yes` 建筑上的 OmniFire 能正确生效，避免炮塔属性使全向开火设置失效。 | L列表 · [Bugfixes and miscellaneous 第289项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH395 主工厂切换不进入卸载状态 | 将生产建筑设为 Primary 时，不再错误进入卸载任务，避免切换主工厂影响其工作状态。 | L列表 · [Bugfixes and miscellaneous 第290项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH396 密集辐射区减少光照卡顿 | 短时间累积大量 radsite 时，减少频繁光照更新造成的明显卡顿，改善辐射密集场景的表现。 | L列表 · [Bugfixes and miscellaneous 第291项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH397 生产动画支持供电属性 | 建筑的 PreProductionAnim、ProductionAnim 可使用 Powered 及 PoweredLight/Effect/Special 等供电属性。 | L列表 · [Bugfixes and miscellaneous 第292项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH398 AI 建筑工厂播放生产动画 | AI 所有的 Factory=BuildingType 建筑在放置生产完成的建筑时，正确播放 ProductionAnim。 | L列表 · [Bugfixes and miscellaneous 第293项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH399 建筑乘员可快捷卸载 | 带乘员的建筑可通过部署快捷键或命令栏卸载乘员，补齐原先不能触发的操作路径。 | L列表 · [Bugfixes and miscellaneous 第294项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH400 炮塔立即转向新攻击目标 | 载具炮塔复位过程中获得新的攻击目标时，立即转向该目标，不必先完成无意义的复位。 | L列表 · [Bugfixes and miscellaneous 第295项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH401 非英文模式正常记录 AI 日志 | 修正非英文模式下电脑玩家记录无法正常输出的问题，保持相应调试记录可用。 | L列表 · [Bugfixes and miscellaneous 第296项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH402 零步行帧率不再除零崩溃 | WalkRate=0 不再在单位起步时触发除零；其含义为不推进步行动画和脚步计时，类似 IdleRate=0。 | L列表 · [Bugfixes and miscellaneous 第297项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH403 观察者可见所有伊文炸弹 | 观察者能够看见任何所属方附着的 IvanBomb，避免观察视角漏掉部分已放置炸弹。 | L列表 · [Bugfixes and miscellaneous 第298项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH404 矿石生长扩散避免崩溃卡死 | 修正 Tiberium 生长与扩散中的崩溃、卡死路径，使持续增长的矿物区域更稳定。 | L列表 · [Bugfixes and miscellaneous 第299项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH405 矿脉使用对应怪物色盘 | Tiberium veins Overlay 使用与 Veinhole Monster 相匹配的色盘，避免矿脉与主体颜色不一致。 | L列表 · [Bugfixes and miscellaneous 第300项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH406 降落伞可解除固定所属色盘 | 降落伞动画可用 ShouldUseCellDrawer=no 解除强制所属方色盘，并继承动画所有者当前国家。 | L列表 · [Bugfixes and miscellaneous 第301项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH407 停用恢复不打断运动器效果 | 载具受 IsLocomotor 弹头影响时，停用或重新激活不再错误中断该效果。 | L列表 · [Bugfixes and miscellaneous 第302项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH408 重开任务正确处理地形图像缓存 | 任务使用带缓存 SHP 图像的 TerrainType 时，重新开始任务不再因相关缓存处理导致崩溃。 | L列表 · [Bugfixes and miscellaneous 第303项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH409 围墙消失后停止攻击空地 | 目标围墙被邻格伤害、碾压或波伤害摧毁后，攻击单位不再继续攻击原地面位置。 | L列表 · [Bugfixes and miscellaneous 第304项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH410 攻击脚本支持工程师和间谍进入 | 工程师、间谍执行原版攻击脚本时可以正确进入目标建筑，修正相应任务分支的进入失败。 | L列表 · [Bugfixes and miscellaneous 第305项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH411 屏幕顶部声波绘制避免崩溃 | 修正 IsSonic 波靠近屏幕上边缘绘制时发生的罕见崩溃，处理相关边界坐标。 | L列表 · [Bugfixes and miscellaneous 第306项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH412 同平面位置波绘制避免错误 | 波攻击发射者与目标处于相同水平位置时，不再导致绘制崩溃或内存破坏。 | L列表 · [Bugfixes and miscellaneous 第307项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH413 征募排除坠毁和沉没对象 | 正在坠毁或沉没的单位不能再被征募进 TaskForce 队伍，避免把死亡对象当作可用队员。 | L列表 · [Bugfixes and miscellaneous 第308项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH414 退出坦克碉堡不误执行部署 | 拥有部署功能的单位离开 Tank Bunker 时，不再顺带执行其部署功能。 | L列表 · [Bugfixes and miscellaneous 第309项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH415 暂存发射步兵正确设置装填计时 | 步兵武器同时使用 LimboLaunch 与弹药机制时，正确设置 ReloadTimer，避免装填流程失效。 | L列表 · [Bugfixes and miscellaneous 第310项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH416 未定义碎片类型不强制生成金属碎片 | 单位配置 MinDebris、MaxDebris 而没有 DebrisTypes 时，不再在死亡时无条件额外生成一次 MetallicDebris。 | L列表 · [Bugfixes and miscellaneous 第311项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH417 指令不再只处理前一百二十八个对象 | 移除单次执行命令时的 128 个 Techno 限制，让更大的选中编组能够共同接收命令。 | L列表 · [Bugfixes and miscellaneous 第312项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH418 规划节点和总单位上限提高 | 规划模式的节点上限由 16 提高至 256，单个规划命令的总单位上限由 128 提高至 256。 | L列表 · [Bugfixes and miscellaneous 第313项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |
| PH419 维修出售模式可始终切换 | 维修及出售模式不再依赖场上存在 Insignificant=no 建筑，避免建筑构成变化导致模式不能切换。 | L列表 · [Bugfixes and miscellaneous 第314项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#bugfixes-and-miscellaneous) |

### 与其他扩展的逐项兼容

本组44条。

| 编号及条目 | 中文说明 | 版本与官方来源 |
| --- | --- | --- |
| PH520 EMP 超武采用建筑正确枪口 | Ares EMPulse 超武发出的武器完整使用发射建筑 FLH，修正扩展交互中的枪口偏移。 | L列表 · [Fixes / interactions with other extensions 第1项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH521 EMP 超武武器可生成辐射 | Ares EMPulse 超武在不是 TargetSelf 模式时，其发射武器也能创建辐射效果。 | L列表 · [Fixes / interactions with other extensions 第2项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH522 EMP 超武弹体尊重浮落和重力 | Ares EMPulse 超武发射的武器会读取 Floater 与 Phobos Gravity，保持弹体运动配置一致。 | L列表 · [Fixes / interactions with other extensions 第3项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH523 悬浮简单部署正确恢复移动 | Hover 运动器的 IsSimpleDeployer 单位使用 DeployToLand 后不再卡住，也不再无限播放移动音效。 | L列表 · [Fixes / interactions with other extensions 第4项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH524 类型部署转换显示禁部署光标 | Convert.Deploy 的目标类型因 SpeedType 等条件不能进入当前格时，显示 NoDeploy 光标而非可部署。 | L列表 · [Fixes / interactions with other extensions 第5项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH525 类型转换更新传送出现延迟 | 包括 Ares 在内的类型转换，在传送单位出现延迟尚未结束时，正确更新转换后的延迟状态。 | L列表 · [Fixes / interactions with other extensions 第6项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH526 移动中类型转换更新移动音效 | 包括 Ares 在内的类型转换，在对象移动中更新 MoveSound，避免新类型继续使用旧类型移动音效。 | L列表 · [Fixes / interactions with other extensions 第7项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH527 运输载具转换更新敞篷乘员状态 | 运输对象类型转换后，正确更新其乘员的 OpenTopped 状态，避免保留转换前的射击条件。 | L列表 · [Fixes / interactions with other extensions 第8项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH528 特殊动画期间销毁不恢复旧动画 | 修正 Ares 引入的问题：建筑播放 SpecialAnim 时出售、抹除或摧毁，不再错误恢复 ActiveAnim。 | L列表 · [Fixes / interactions with other extensions 第9项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH529 劫持移动载具不残留占用 | Ares Abductor 武器劫持正在移动的载具时，正确清理放置占用，避免留下永久阻挡状态。 | L列表 · [Fixes / interactions with other extensions 第10项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH530 抑制特定指针重定位误警告 | 解析 Tags、TaskForces 时抑制 Ares 对特定空指针重定位产生的警告，减少这类非致命日志干扰。 | L列表 · [Fixes / interactions with other extensions 第11项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH531 初始乘员获得学院加成 | 战车工厂生产的载具，其 Ares InitialPayload 初始乘员也能获得 Academy 效果。 | L列表 · [Fixes / interactions with other extensions 第12项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH532 触发生成载具创建初始载荷 | 经地图触发动作生成的载具也能创建 Ares InitialPayload，避免初始乘员只在部分生产路径生成。 | L列表 · [Fixes / interactions with other extensions 第13项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH533 减少静态对象透明附图重建 | 接管 Ares AlphaImage 重建处理，避免建筑、静止对象及无炮塔对象每帧重复创建附图造成卡顿。 | L列表 · [Fixes / interactions with other extensions 第14项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH534 修正 Ares 特定地图事件判定 | 修正 Ares 地图触发事件 75、77 部分情况下无法正确判定的问题，避免条件满足却不触发。 | L列表 · [Fixes / interactions with other extensions 第15项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH535 修正部署类型转换后崩溃 | 修正部分单位在部署转换后导致崩溃的问题，改善相应转换路径与对象状态的衔接。 | L列表 · [Fixes / interactions with other extensions 第16项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH536 进入隧道清理透明附图 | 单位进入隧道后正确移除 AlphaImage，避免地下单位的附图仍留在原位置显示。 | L列表 · [Fixes / interactions with other extensions 第17项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH537 部署转换不在转向移动时重复触发 | 修正 Ares Convert.Deploy 在单位转向或移动时反复触发的问题，避免一次部署重复进入转换。 | L列表 · [Fixes / interactions with other extensions 第18项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH538 配合启动器保存多份联机存档 | 配合支持联机存档的 Spawner 与 XNA CnCNet，使用 SVGM_000—999.NET 并保存 spawnSG.ini，减少频繁保存覆盖；非通用独立存档保证。 | L列表 · [Fixes / interactions with other extensions 第19项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH539 类型转换按射角重置炮管 | Ares 类型转换时按 FireAngle 正确重置炮管方向，避免新类型沿用不适用的旧炮管角度。 | L列表 · [Fixes / interactions with other extensions 第20项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH540 闪光时长覆盖维修闪光 | Ares Flash.Duration 可以覆盖武器维修闪光效果，修正设置在该路径中被忽略的问题。 | L列表 · [Fixes / interactions with other extensions 第21项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH541 克隆战车工厂避免重复克隆 | 同时有 CloningFacility 与 WeaponsFactory 的建筑，不再错误克隆多辆载具并将其堵在出口。 | L列表 · [Fixes / interactions with other extensions 第22项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH542 收起型建筑可显示替代图标 | 修正 Ares 引入的问题：渗透敌方载具工厂后，带 UndeploysInto 的建筑可显示 AltCameo/AltCameoPCX。 | L列表 · [Fixes / interactions with other extensions 第23项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH543 修正幸存者的非概率生成失败 | 修正对象被摧毁后，幸存者因非概率因素不能生成的问题；并不代表取消原有生成概率。 | L列表 · [Fixes / interactions with other extensions 第24项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH544 运输载具幸存者生成位置正确 | 运输对象被摧毁时，载具幸存者使用正确出生位置，避免沿用错误的运输或乘员坐标。 | L列表 · [Fixes / interactions with other extensions 第25项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH545 爆炸废墟建筑不阻断战败 | Explodes=yes 建筑使用 Ares 废墟逻辑后，不再因状态残留使其所属方无法正常战败。 | L列表 · [Fixes / interactions with other extensions 第26项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH546 调整隐形敞篷运输开火限制 | 修改 Ares 中隐形 OpenTopped 运输载具的开火阻止钩子；具体是否可开火须结合当前运输与武器规则核对。 | L列表 · [Fixes / interactions with other extensions 第27项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH547 修正寄生物反复重置到受害者 | 修正 Ares 特定情况下令寄生物频繁回到受害者位置的错误交互，减少由此引发的崩溃。 | L列表 · [Fixes / interactions with other extensions 第28项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH548 单位投送超武建筑初始朝向正确 | Ares UnitDelivery 超武放置建筑时使用正确初始方向，避免投送结果的朝向被错误初始化。 | L列表 · [Fixes / interactions with other extensions 第29项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH549 敞篷初始乘员不受自身地形误限 | InitialPayload 或 Full 队伍生成的 OpenTopped 乘员，不会仅因载具驶入乘员自身不能通行的地形而失去开火能力。 | L列表 · [Fixes / interactions with other extensions 第30项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH550 转换移除透明附图后可正常读档 | 对象转换或动画 Next 转为不带 AlphaImage 的类型后，正确处理附图状态，避免之后读档崩溃。 | L列表 · [Fixes / interactions with other extensions 第31项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH551 类型转换正确更新炸弹观察属性 | Techno 类型转换后正确更新 BombSight，避免继续沿用旧类型的炸弹可见性相关设置。 | L列表 · [Fixes / interactions with other extensions 第32项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH552 国家 EVA 配置读档后保留 | EVA.Tag 支持按国家设置，EVAIndex 在读档后不再重置，使国家语音选择保持一致。 | L列表 · [Fixes / interactions with other extensions 第33项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH553 禁用武器与盖特敞篷逻辑协调 | DisableWeapons.Duration 让盖特阶段回落并停声，不再干扰索敌，同时可配合 OpenTopped.CheckTransportDisableWeapons。 | L列表 · [Fixes / interactions with other extensions 第34项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH554 超武建筑条件纳入建筑升级 | Ares SW.AuxBuildings、SW.NegBuildings 可统计建筑升级，使升级也能满足或阻止超武使用条件。 | L列表 · [Fixes / interactions with other extensions 第35项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH555 步兵部署转换无需简单部署标记 | 步兵可使用 Convert.Deploy 而不必再设置 IsSimpleDeployer=true，减少与不相干部署机制的耦合。 | L列表 · [Fixes / interactions with other extensions 第36项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH556 自定义巡航导弹使用爬升速率 | 允许自定义巡航导弹，从而使 Ares Missile.RaiseRate 在适用导弹上发挥实际作用。 | L列表 · [Fixes / interactions with other extensions 第37项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH557 EMP 暂停 AI 工厂生产 | 修正 Ares EMP 不暂停电脑工厂生产的问题，使 AI 工厂受电磁瘫痪时正确中止生产推进。 | L列表 · [Fixes / interactions with other extensions 第38项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH558 步兵可使用初始载荷 | 移除 InfantryType 不能使用 InitialPayload 的限制，使初始载荷机制可应用于步兵类型。 | L列表 · [Fixes / interactions with other extensions 第39项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH559 弹体航程可应用武器射程倍率 | ProjectileRange 大于零时应用武器射程修正；WeaponType 可用 ProjectileRange.ApplyModifiers=false 禁用。 | L列表 · [Fixes / interactions with other extensions 第40项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH560 可配置弹头禁止散开默认值 | CombatDamage.Warhead.PreventScatter 可设置弹头 PreventScatter 的全局默认，减少逐弹头重复配置。 | L列表 · [Fixes / interactions with other extensions 第41项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH561 超武图标手动发射独立配置 | SW.ShowCameo 与 SW.ManualFire 可独立于 SW.AutoFire 生效，避免自动发射设置决定其他操作入口。 | L列表 · [Fixes / interactions with other extensions 第42项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH562 隧道建筑支持快捷卸载 | Ares 隧道类建筑可通过部署快捷键或命令栏卸载对象，修正仅该建筑类别失效的操作路径。 | L列表 · [Fixes / interactions with other extensions 第43项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |
| PH563 驾驶员死亡正确停止载具活动 | 修正 Ares 中驾驶员被杀后载具偶尔仍继续开火和移动的问题，保持无驾驶员状态一致。 | L列表 · [Fixes / interactions with other extensions 第44项](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html#fixes-interactions-with-other-extensions) |

### 界面逐项修复

本组8条。

| 编号及条目 | 中文说明 | 版本与官方来源 |
| --- | --- | --- |
| PH829 加载全彩非索引 PCX | 允许加载全彩、非调色板 PCX，并覆盖游戏与 Ares 支持的 PCX 加载路径，不再仅限原有索引图像。 | L列表 · [Bugfixes and miscellanous 第1项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH830 命令行可指定窗口图标 | gamemd.exe 可用 -icon 后接绝对或相对 ico 路径，指定自定义程序窗口图标。 | L列表 · [Bugfixes and miscellanous 第2项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH831 免注册加载 Blowfish 库 | 系统未注册 Blowfish.dll 时直接加载而不尝试注册，避免无管理员权限等原因触发字符串管理器初始化失败。 | L列表 · [Bugfixes and miscellanous 第3项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH832 扩展非输入法键盘字符支持 | 修正非 IME 输入对 Basic Latin、Latin-1 之外字符的处理；实际显示仍需字库包含对应字形。 | L列表 · [Bugfixes and miscellanous 第4项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH833 修正建筑信息提示与生产图标 | 选中建筑的信息提示及揭示生产图标采用正确位置和图层，避免相互遮挡或显示错位。 | L列表 · [Bugfixes and miscellanous 第5项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH834 可配置计时器闪烁配色 | AudioVisual.TimerBlinkColorScheme 指定超武、任务等计时器的闪烁色组，默认采用 Colors 列表第三项。 | L列表 · [Bugfixes and miscellanous 第6项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH835 暂停生产时队列数量及时刷新 | 生产暂停期间添加或移除排队单位，侧栏正确更新队列数量，不再显示过期计数。 | L列表 · [Bugfixes and miscellanous 第7项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |
| PH836 提高光标刷新频率 | 将光标更新间隔从 16 毫秒缩短为 1 毫秒，提升指针更新响应；不代表游戏模拟帧率同步提高。 | L列表 · [Bugfixes and miscellanous 第8项](https://phobos.readthedocs.io/en/latest/User-Interface.html#bugfixes-and-miscellanous) |

<a id="historical"></a>
## 稳定文档旧标题对照

下列8个旧锚点在此次latest中不存在。部分能力已改名或并入更大主题；此表保留稳定文档的定位，既不判定已删除，也不保证latest与stable逐键等价。这里不是独立的稳定版全量差异清单。

| 编号及主题 | 中文说明 | 代表性INI入口 | 后续开发建议 | 版本与官方原文 |
| --- | --- | --- | --- | --- |
| PH883 旧稳定文档：工业工厂类型筛选 | 稳定文档以独立主题说明 FactoryPlant 的类型白黑名单；最新文档有更广的对应主题，迁移时应比较参数而非认定功能消失。 | `FactoryPlant.AllowTypes`、`FactoryPlant.DisallowTypes` | 对照新主题合并配置政策，不并列实现两套增益统计。 | S · [Allowed / disallowed types for FactoryPlant](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#allowed-disallowed-types-for-factoryplant) |
| PH884 旧稳定文档：默认染色方案 | 稳定文档说明 AltPalette 动画和不能正常染色的体素弹体使用可配置默认色组；最新文档对动画染色进行了更细分组织。 | `AnimRemapDefaultColorScheme` | 迁移时核对动画和弹体各自读取范围，避免按旧标题推定删除。 | S · [Customizing default ColorScheme](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#customizing-default-colorscheme) |
| PH885 旧稳定文档：空袭飞机出生参数 | 稳定文档单列空袭及侦察机距目标固定距离和初始高度；最新文档扩展为增援飞机设置，旧配置需对照新主题验证。 | `SpawnDistanceFromTarget`、`SpawnHeight` | 复用增援生成机制，显式保留旧键与版本差异。 | S · [Fixed spawn distance & spawn height for airstrike / SpyPlane aircraft](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#fixed-spawn-distance-spawn-height-for-airstrike-spyplane-aircraft) |
| PH886 旧稳定文档：弹药驱动部署 | 可按弹药上下限自动部署或解锁部署；稳定文档警告自动部署需结合 Ares 类型转换，否则可能反复下达部署命令。 | `Ammo.AutoDeployMinimumAmount`、`Ammo.AutoDeployMaximumAmount`、`Ammo.DeployUnlockMinimumAmount`、`Ammo.DeployUnlockMaximumAmount` | 核对新版弹药转换政策，通过部署和弹药 owner 实现一次事务。 | S · [IsSimpleDeployer vehicle auto-deploy / deploy block on ammo change](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#issimpledeployer-vehicle-auto-deploy-deploy-block-on-ammo-change) |
| PH887 旧稳定文档：部署动画与方向 | 稳定文档提供地面部署任意方向、保持本体可见、收起倒放和阵营色绘制；最新主题已扩展部署方向与动画组织。 | `DeployingAnim.AllowAnyDirection`、`DeployingAnim.KeepUnitVisible`、`DeployingAnim.ReverseForUndeploy`、`DeployingAnim.UseUnitDrawer` | 迁移时核对旧方向键与新版 DeployDir，避免猜测兼容别名。 | S · [IsSimpleDeployer vehicle deploy animation / direction customization](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#issimpledeployer-vehicle-deploy-animation-direction-customization) |
| PH888 旧稳定文档：静止车辆 | 稳定文档说明 Speed=0 的车辆按静止对象处理，并警告可造车辆无法离开工厂；最新文档还覆盖步兵与移动区域限制。 | 无独立示例键；见原文条件 | 统一采用静止资格 owner，再按版本保留行为差异。 | S · [Stationary vehicles](https://phobos.readthedocs.io/en/stable/Fixed-or-Improved-Logics.html#stationary-vehicles) |
| PH889 稳定文档旧主题：隐藏生命条 | 稳定文档以独立主题说明HealthBar.Hide隐藏生命条及外框；latest同键并入更完整的生命条显示主题，不能据锚点缺失判定删除。 | `HealthBar.Hide` | 迁移时转查PH840及其原文，保持同键读取语义。 | S · [Hide health bars](https://phobos.readthedocs.io/en/stable/User-Interface.html#hide-health-bars) |
| PH890 稳定文档旧主题：光效开关 | 稳定文档单列闪光开关，latest扩大为光效、震屏和轨迹控制；旧参数仍需逐项核对，而非直接当成被移除。 | `WarheadParticleAlphaImageIsLightFlash`、`LightFlashAlphaImageDetailLevel`、`Particle.AlphaImageIsLightFlash`、`HideLightFlashEffects` | 对照PH880迁移，玩家偏好与模拟效果分离。 | S · [Light flash effect toggling](https://phobos.readthedocs.io/en/stable/User-Interface.html#light-flash-effect-toggling) |

<a id="numbered"></a>
## 脚本、触发动作和事件编号速查

**这三种编号不是同一张表。** AI脚本行写在ScriptType中，地图触发动作写在`[Actions]`，地图事件写在`[Events]`；例如动作600与事件600完全不同。下面区分官方分类预留范围和正文实际定义，避免把整个编号段都注册成“已支持”。

### AI 脚本

| 实际编号或范围 | 用途 | 主题入口 |
| --- | --- | --- |
| 10000—10017 | 按目标类别/列表、远近、威胁等攻击 | PH003；10000—10049是所属攻击分类范围 |
| 10050—10061 | 移动接近敌我对象或目标列表 | PH047；10050—10099是所属移动分类范围 |
| 10100、10101、10102、10103、10104 | 区域警戒、等弹满、队长附近集结、装载、超时空袭击基地 | PH068、PH076、PH051、PH042、PH021 |
| 12000、12001、12002 | 无目标重试、一次性接近距离、一次性移动结束策略 | PH075、PH045、PH057 |
| 14000—14004 | 成功奖励、加权、减权、撤销成功、OnlyTargetHouseEnemy覆盖 | PH066、PH039、PH025、PH071、PH035 |
| 16000—16005 | 定时重复、定时下一行、停止计时、概率跳行、随机脚本、返回脚本 | PH064、PH063、PH065、PH050、PH048、PH041 |
| 18000—18023 | 打包16位索引/操作数的变量操作 | PH030 |
| 18024—18047 | 以局部变量为操作数 | PH032 |
| 18048—18071 | 以全局变量为操作数 | PH031 |
| 19000—19999 | 预留未分类区，官方正文当前为空 | PH044，不应生成该段所有动作 |
| 原版46、47、56、58的新参数写法 | 类型名与扫描模式替代索引/偏移手算 | PH046 |

10000—10999、10100—10999、12000—12999、14000—14999、16000—16999、18000—18999等大范围是文档组织或编号分配，不等于每个数都已定义。变量脚本的参数打包和16位数值尤其不能与动作501的32位路径混用。[官方脚本行动表](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#script-actions)

### 地图触发动作

| 编号 | 行为 | 主题入口 |
| --- | --- | --- |
| 500 | 立即存档 | PH052 |
| 501、502、503、504 | 编辑变量、随机数、打印变量、变量间运算 | PH038、PH036、PH049、PH005 |
| 505、506 | 在坐标或路径点发射超武 | PH033、PH034 |
| 510、511、512、513 | MCV重部署开关、收起建筑移向路径点、设置车辆Follower、计时显示 | PH069、PH070、PH053、PH056 |
| 600 | 配置关联对象掉落补给箱 | PH023 |
| 606、607、608、609、610、611 | 仇恨编辑/清除、强制敌人、雷达模式、组队间隔、下一任务 | PH029、PH022、PH054、PH060、PH055、PH058 |
| 800、801、802 | 局部变量横幅、全局变量横幅、删除横幅 | PH027 |

未列出的空隙不能据此推断功能存在。多人存档在上游有spawner支持条件，VERA须验证自身存档协调路径。[官方地图动作表](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#trigger-actions)

### 地图事件

| 编号 | 条件 | 主题入口 |
| --- | --- | --- |
| 500—511 | 局部/全局变量与常量比较或按位与 | PH072 |
| 512—523 | 局部/全局变量与局部变量比较或按位与 | PH074 |
| 524—535 | 局部/全局变量与全局变量比较或按位与 | PH073 |
| 600 | 关联对象护盾破裂 | PH067 |
| 601、602 | 指定阵营拥有或没有指定类型 | PH037 |
| 604、605 | 指定类型或类型列表中的对象进入格 | PH020；没有据此补出603 |
| 606 | 在有效时间内施加或重施附加效果 | PH002 |

同一个事件范围的前后半段区分变量作用域，具体顺序查原表。PH002不补触发启用前已经存在的效果；PH037的旧多人阵营编码并不自动覆盖30人。[官方事件表](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html#trigger-events)

<a id="implementation"></a>
## 如何按 SDK 逐步实现

### 按职责拆，不按每个 INI 键增加宿主函数

| 工作类别 | 引擎保留什么 | 功能组件负责什么 | 当前需要补的接口 |
| --- | --- | --- | --- |
| 简单策略与数值 | 弹仓、实际伤害、移动、开火的既有流程 | 读取配置并返回成本、倍率或选择策略 | 已有武器成本、装填量与低弹量选择策略可参考；其他决策点按需增加 |
| 附加效果与持续状态 | 共享能力查询、生命周期事件、命令校验 | 类型配置、持续时间、叠加、失效、效果策略 | 效果描述/查询及创建、移除、转换、死亡事件 |
| 弹头、超武与世界操作 | 对象、阵营、经济、传送、生产等单一状态所有者 | 策略及一次性意图 | 类型化领域命令、权限、顺序、幂等与结果 |
| 弹体轨迹与拦截 | 弹体身份、坐标、碰撞、世界查询与同步RNG | 受限轨迹/命中策略 | 专用阶段接口与批量查询，先测热循环成本 |
| 数字显示、侧栏、加载与暂停界面 | 资源、输入、绘制、音频与本地偏好 | 可以声明只读展示数据或配置 | 呈现/资源协议，不能用对象tick替代界面框架 |
| 解析、错误修复和性能 | INI及资源加载器、原版机制、内存与索引安全 | 必要时声明明确的兼容政策 | 修正既有所有者，避免做成需要每作品重复带的补丁 |

对象真实生命值、弹药、所有权、位置和生产进度都只由相应引擎领域写入。插件可以拥有自己的效果持续时间或护盾点数；其他系统通过经宿主验证的查询或效果描述使用它，不能再维护一份相同的权威状态。

### 示例一：现有护盾参考组件

假设作品已按[SDK说明](../../sdk/README.zh-CN.md)接纳参考WASM组件，下面是Rules片段，使用已外置的基本护盾能力；INI本身不会下载或加载一个未声明的组件。

```ini
[VeraFeatures]
Shield=yes
Overload=no

[ShieldTypes]
0=DemoShield

[DemoShield]
Strength=100
AbsorbPercent=1.0
PassPercent=0.0
SelfHealing=25%
SelfHealing.Rate=0.1
Respawn=100%
Respawn.Rate=0.5

[MTNK]
ShieldType=DemoShield
```

加载时组件冻结类型配置；MTNK创建时得到100点护盾状态。伤害回调先处理护盾，剩余伤害交给原有生命值结算。`tick`推进自愈和重生计时，`Meter`显示护盾；护盾值随组件状态保存。这里的0.1和0.5是**游戏分钟**，基础换算分别为90和450模拟帧，不是现实秒。战斗重置、供电、超时空等条件会影响实际恢复时机，不能据此承诺固定现实秒恢复。

增加 `Overload=yes` 选择的是我们写的护盾热量玩法；不是启用PH771的原版心控过载，也不代表完整Phobos护盾功能都已兼容。

### 示例二：规划一个限时加速效果

下面按Phobos配置组织展示**待实现**的附加效果，当前参考组件不会仅因这些键就给坦克加速：

```ini
[AttachEffectTypes]
0=DemoHaste

[DemoHaste]
Duration=150
SpeedMultiplier=1.25

[MTNK]
AttachEffect.AttachTypes=DemoHaste
AttachEffect.Delays=-1
```

目标行为是MTNK初次获得效果后，在有效的150游戏帧内使用速度倍率。`AttachEffect.Delays=-1`禁止自然到期后自动重施；若省略而沿用默认0，自带效果到期后会立即重新获得，便不再是此处的单次限时效果。实现应按这样的链完成：

1. 配置阶段登记效果类型、把150帧和1.25倍率冻结到组件配置，明确自带效果延迟、重新施加、叠加与过期规则。
2. 对象建立或效果施加时构造效果状态；宿主发出必要生命周期事件，避免把仅有对象`attach`误认为已经覆盖所有施加来源。
3. 增加通用的**移动属性策略查询**或受校验效果描述，由引擎移动所有者在计算最终速度时消费倍率。该接口目前没有，不能拿`Operation`越权写速度。
4. 效果过期后取消该倍率；不把速度强制写回“最初值”，因为老兵、箱子、其他效果可能在期间改变基础结果。
5. 存档保存剩余时间和效果身份；联机校验覆盖影响模拟的配置与状态，跨平台使用一致数值语义。测有效前后速度，并测试中途存读档和类型转换。

这样后续作者修改持续时间、倍率、叠加或触发条件，主要改插件；引擎保留能被不同功能复用的移动查询，而不是新增一个仅认识`DemoHaste`的特殊钩子。[Phobos附加效果原文](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html#attached-effects)

### 建议的推进次序

1. **先把现有护盾缺口拆成可验收的链。** 例如破盾动画/武器、弹头施加与移除、类型转换保留，分别查依赖；不要同时宣布“支持全部护盾”。
2. **再扩充有明确决策点的小功能。** 已有Ares低弹量单槽选择可作为基础；Phobos多候选选择、武器目标过滤或其他加成查询仍需逐项实现。复用原有开火和弹仓，验证选用、拒绝、消耗和装填整条路径。
3. **建立有限的效果协议。** 限时加速作为一条闭环完成后，再增加叠加、失电暂停、换归属、死亡和解除条件；不先造一个任意写世界的总接口。
4. **界面及资源扩展单独推进。** 数字显示、建造队列、加载画面与超武侧栏参考现有内置所有者；先决定哪些配置要给作者开放，再扩展呈现协议。
5. **最后做高耦合的世界机制。** 类型转换、拦截、轨迹、超武连锁与跨对象效果，需要对象引用、时序、碰撞和生命周期依赖，开发量不能按INI键个数估计。

### 每条功能的完成条件

- 明确作品需要哪一个官方版本的行为；记录配置所在文件、节、键、默认来源、读取器、覆盖顺序、单位及非法输入策略。
- 找到引擎里相关状态的唯一所有者，定义插件输入、返回值以及哪些领域命令可执行，不能只实现配置字段。
- 接通创建、使用、暂停、解除、销毁和读档；列出换归属、运输、钻地、隐形、超时空、部署/类型转换等真正相关的交互。
- 规定同帧阶段、同步RNG流与抽取次数，选择跨平台一致的数学语义；表现效果不得更改模拟。
- 检查组件字节、manifest、冻结配置、状态版本及存档绑定。当前原型没有热替换或自动状态迁移，旧存档不能随意换成重编译组件。
- 通过正式加载与游戏路径验证一个代表性闭环，保留少量能发现实际回归的测试；联机、存读档和资源表现按功能需要验收。
- 把仍未完成的触发条件和交互写在功能支持页；“键已解析”“目录已列出”“插件成功加载”均不能替代行为验收。

当前代码入口见 [组件分派](../../plugins/vera20k-ares/src/component.rs)、[护盾配置](../../plugins/vera20k-ares/src/features/defense/shield/config.rs)、[护盾逻辑](../../plugins/vera20k-ares/src/features/defense/shield/logic.rs)、[Ammo示例](../../plugins/vera20k-ares/src/features/weapons/ammo_cost.rs)、[类型化API](../../sdk/api/src/lib.rs)、[宿主服务](../../src/mod_sdk/host.rs)、[模拟接入](../../src/mod_sdk/simulation.rs)和[架构指南](../../sdk/ARCHITECTURE.zh-CN.md)。

<a id="references"></a>
## 官方参考与署名

| 来源 | 在本手册中的用途 |
| --- | --- |
| [项目首页](https://phobos.readthedocs.io/en/latest/) / [官方仓库](https://github.com/Phobos-developers/Phobos) | 版本入口、项目关系、安装与代码定位；上游DLL注入部署方式不直接套给VERA |
| [General Info](https://phobos.readthedocs.io/en/latest/General-Info.html) | 构建类型、存档版本与兼容约定 |
| [New / Enhanced Logics](https://phobos.readthedocs.io/en/latest/New-or-Enhanced-Logics.html) | 新类型与玩法机制 |
| [Fixed / Improved Logics](https://phobos.readthedocs.io/en/latest/Fixed-or-Improved-Logics.html) | 已有逻辑改进、配置与逐项修复 |
| [AI Scripting and Mapping](https://phobos.readthedocs.io/en/latest/AI-Scripting-and-Mapping.html) | AI行动、地图动作、事件与任务参数 |
| [User Interface](https://phobos.readthedocs.io/en/latest/User-Interface.html) | 输入、侧栏、显示与加载画面 |
| [Miscellanous](https://phobos.readthedocs.io/en/latest/Miscellanous.html) | INI、开发工具、玩家颜色与速度 |
| [Interoperability](https://phobos.readthedocs.io/en/latest/Interoperability.html) | 原生扩展导出ABI，与VERA WASM SDK分开处理 |
| [What’s New / Migration](https://phobos.readthedocs.io/en/latest/Whats-New.html) | 逐版本默认行为变化、键迁移与发行历史；实施具体版本时逐段核对 |
| [Project guidelines and policies](https://phobos.readthedocs.io/en/latest/Project-guidelines-and-policies.html) / [Contributing](https://phobos.readthedocs.io/en/latest/Contributing.html) | 上游项目政策与贡献流程 |
| [Credits](https://phobos.readthedocs.io/en/latest/CREDITS.html) / [License](https://phobos.readthedocs.io/en/latest/License.html) | 原作者贡献与上游许可原文 |

Phobos来自Phobos Developers及其贡献者；本手册是面向VERA开发者的独立中文归纳，保留原题、标识符和可追溯链接，不冒充官方翻译或授权证明。复用上游实现时按对应文件的许可和署名要求处理，功能来源与VERA重写的实现范围分开记录。

文档更新时，按官方页面与锚点核对增删、正文变化、示例键和迁移记录；保持现有PH编号的交叉引用，新增内容分配新编号，不为排序而重排既有编号。对改名主题建立旧新映射，再决定是否合并，不能仅凭标题变化判定删除。

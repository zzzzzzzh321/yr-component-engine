---
layout: default
title: Ares 功能中文开发参考
lang: zh-CN
---

# Ares 功能中文开发参考

这份手册用于为 VERA20K 编写兼容 Ares 写法的功能模块。按 [Ares 3.0 官方文档](https://ares-developers.github.io/Ares-docs/) 的功能页、修复页和参考页归纳，保留英文名称与 INI 拼写，中文解释行为及后续实现边界。核对日期为 **2026-11-01**。

本版列出 **239 个功能主题、143 项修复、23 个目录或参考入口**。功能主题中也包含官方注明未完成或有限制的项目；目录页数量不代表独立能力数量。范围对应官方目录和搜索索引中的公开文档，未收录第三方分支、未公开行为或 Phobos、NPatch 独有扩展。

功能表的“后续开发建议”是面向 VERA20K 的设计建议。官方行为说明与当前引擎支持状态分别查阅：内置兼容范围见 [Ares 支持记录](../ares.md)，外置功能接口见 [SDK 中文指南](../../sdk/README.zh-CN.md)。

## 使用与实现边界

先按功能名或 INI 键搜索下方目录，读清适用对象和限制，再打开该行官方原文核对完整参数、默认值、单位及历史变更。独立的 [INI 键索引 CSV](ares-ini-index.csv) 按功能编号、所属节和键名定位来源；它收录官方正文中显式标注节名的关联键，包含复用的原版键及原文占位模板，未将所有条目都宣称为 Ares 新增键。索引中的 `*`、`X`、`#` 等模板须按原文展开；`Weapon`、`TechnoType` 等节类别须替换为实际对象ID，不能直接照抄为节名。

### 当前 SDK 的实际能力

当前统一参考组件为 `vera.reference-gameplay` 0.3.0。已外置的 **Ares 武器弹药成本 `Ammo`** 使用 `weapon_resource`；可再生血量护盾示例属于 Phobos 兼容链，过载为自定义玩法。Ares 的 ForceShield 力场护盾超武另见[超级武器](#超级武器)，与这项血量护盾分开处理。

| 现有接口 | 实际用途 |
| --- | --- |
| `configure` | 读取已接纳的 Rules 层，最终冻结配置和类型绑定 |
| `attach` | 构造对象的插件状态 |
| `damage` | 处理当前接收者的状态和伤害输入 |
| `tick` | 在对象更新阶段推进自身状态 |
| `weapon_resource` | 加载时查询武器弹药策略，绑定到引擎已有资源流程 |
| `Meter` / `Tint` | 描述通用进度条和染色，由引擎呈现 |

目前 `Operation` 仍全部拒绝执行，不能借它直接改血、改钱、生成单位、控制天气或修改其他对象。本手册中的这些能力需要先扩展受校验的通用接口。对象返回空状态会取消后续订阅；一个作品仍只接纳一个可选组件，内部按明确顺序组合功能。

### 选择正确的实现位置

| 功能性质 | 合适的位置与边界 |
| --- | --- |
| 配置解释、数值或选择策略 | 尽量留在功能模块；引擎提供已有决策点上的通用查询 |
| EMP、附加效果、禁武器等持续状态 | 引擎维护共享状态与能力查询，插件给出可校验的效果描述，明确暂停、叠加和移除 |
| 所有权、运输、类型转换、超武与世界操作 | 经领域所有者提交命令，完成初始化、引用、归属、清理和存档整条链 |
| 菜单、鼠标、声音、调色板和加载画面 | 应用、输入、呈现和资源层；不能假设对象的 `tick` 能驱动这些功能 |
| INI、MIX、CSF、BAG 与美术格式 | 统一加载器和资源目录；插件引用逻辑资源名，保持原有优先级 |
| 原版缺陷、越界、泄漏和算法性能 | 修正相应引擎所有者；需要历史兼容差异时再明确配置行为 |

原版对象的真实生命值、弹仓、所有权、电力、移动和生产状态都保留一个权威所有者。已有内置 Ares Rust 实现可作为迁移参考，迁移时同步替换消费者；避免出现两份可独立变化的状态。

## INI 读取与单位约定

- **节与键的位置有意义。** 单位节的 `Ammo` 是弹仓容量，武器节的 `Ammo` 是每次消耗。把名字一样的键放错节，不会得到同一种功能。
- **保留原拼写。** 例如 `CanPassiveAquire` 是文档采用的键名，不应仅按英文拼写习惯自行改名。
- **类型按每个标签确定。** 有的比例用浮点或带百分号表示，有的使用整数百分比；布尔、整数、枚举、类型名和资源引用需要不同读取器。
- **分清游戏时间与现实时间。** 官方术语以15帧标定一秒游戏时间，游戏分钟通常乘900换算为帧；部分标签的分钟指现实时间，应逐项核对。游戏时间对应的现实耗时随速度变化。距离单位中，一格是256 leptons。详见 [官方术语表](https://ares-developers.github.io/Ares-docs/glossary.html)。
- **缺失、空值、无效值、零和 `none` 分别处理。** 默认值可能来自构造、另一标签或最后的合并层；逐功能复用引擎现有读取器，不能把所有值统一交给标准字符串转换。
- **同时检查配置与运行链。** 能读到键，只证明配置被接收；状态的创建、消费、恢复、清理、存档和联机校验都需要接通。

### VERA 原型的功能选择

下面是 VERA SDK 自己的选择配置，**不属于 Ares 原生标签**：

```ini
[VeraFeatures]
Shield=yes
Overload=no
AresAmmo=yes
```

它用于选择统一参考组件中已经实现的功能。AresAmmo 开启后，武器仍按原有 `Ammo` 写法配置。尚未实现的 Ares 项目不会因为增加一个 INI 开关就自动可用。选择在对局加载时冻结，读档沿用保存的配置。

## 功能目录

- [恢复的泰伯利亚之日机制](#恢复的泰伯利亚之日机制)
- [单位与通用玩法](#单位与通用玩法)
- [隐形与探测](#隐形与探测)
- [武器](#武器)
- [抛射体](#抛射体)
- [弹头与伤害](#弹头与伤害)
- [建筑与基地](#建筑与基地)
- [阵营与国家](#阵营与国家)
- [超级武器](#超级武器)
- [矿物与资源](#矿物与资源)
- [资源与配置加载](#资源与配置加载)
- [界面与诊断](#界面与诊断)
- [原版缺陷修复](#原版缺陷修复)
- [模组逻辑修复](#模组逻辑修复)
- [性能修复](#性能修复)

下列编号用于本版手册定位。每行都保留对应官方页面，完整默认值与细分条件以该页面为准。所有“后续开发建议”都描述实现方向；当前外置完成范围以本手册开头的 SDK 能力表为准。


## 恢复的泰伯利亚之日机制

本节 9 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR373 出售单位动作修复 | 超级武器的出售单位动作可对停靠单位执行类似侧栏出售的操作。原本对坦克碉堡内的单位使用时，会立即或在碉堡稍后被摧毁时引发内部错误，Ares修复了该路径，使恢复的出售动作不会留下此类故障。 | `Action` | 建议由出售所有者统一处理停靠与碉堡关系，在对象移除时完成容器清理。 | [Action=SellUnit](https://ares-developers.github.io/Ares-docs/restored/actionsellunit.html) |
| AR374 两栖单位水中图像 | 两栖车辆入水后可借用另一已登记车辆类型的图像，形成沉入水中的不同外观，支持SHP或体素资源。此机制替换显示图像，不负责完整类型转换；原对象与水中图像必须同属SHP或同属体素，不能混用两类资源。 | `WaterImage`、`Image`、`UnloadingClass` | 建议由美术资源所有者按地形选择图像，模拟类型与显示替身保持明确区分。 | [Amphibious Image Changes](https://ares-developers.github.io/Ares-docs/restored/amphibiousimagechanges.html) |
| AR375 电磁脉冲停用机制 | EMP可让易感单位与建筑停工，并联动移动、武器、电力、生产、雷达和子机等功能。弹头独立配置持续时间及叠加上限，也支持缩短或清除EMP；类型可调整免疫、时长倍率和毁坏门槛，空中对象还可能立即坠毁。 | `EMP.Duration`、`EMP.Cap`、`ImmuneToEMP`、`EMP.Modifier`、`EMP.Threshold`、`EMP.Sparkles`、`EMPAIRecoverMission`、`Verses` | 建议由失能效果所有者维护EMP计时，各领域能力统一查询并处理停用和恢复。 | [Electromagnetic Pulse (EMP)](https://ares-developers.github.io/Ares-docs/restored/emp.html) |
| AR376 火风暴防御墙 | 指定建筑墙段可自动连接，并在超级武器开启时阻挡敌方弹体、摧毁接触的敌我对象；忽略火风暴的对象可穿越，被焚毁者不释放乘员。可定义焚毁弹头、表现及受击折损持续时间，地图强开动作仅支持非人类阵营。 | `Firestorm.Wall`、`IgnoresFirestorm`、`FirestormWarhead`、`DamageToFirestormDamageCoefficient`、`FirestormActiveAnim`、`FirestormIdleAnim`、`FirestormGroundAnim`、`FirestormAirAnim` | 建议协调防御墙覆盖、弹体通行与销毁所有者，并由超级武器管理有效时段。 | [Firestorm Wall](https://ares-developers.github.io/Ares-docs/restored/firestormwall.html) |
| AR378 激光围栏 | 激光围栏柱之间可自动放置围栏段，阻止地面对象通过且免疫普通伤害；柱子被摧毁会移除关联围栏。失电时围栏关闭可自由通行，恢复供电后会重新激活并毁坏正处于围栏位置的对象，必须同时配置柱和围栏两种建筑。 | `LaserFencePost`、`LaserFence` | 建议由建筑连接与通行所有者维护围栏段，供电变化统一更新阻挡和接触伤害。 | [Laser Fences](https://ares-developers.github.io/Ares-docs/restored/laserfences.html) |
| AR379 多工程师占领 | 可要求建筑生命降到指定比例后才能由工程师占领，未达门槛时改为按最大生命比例造成伤害，科技建筑可例外直接占领。伤害值应与占领门槛协调，避免工程师毁掉目标；未开放菜单开关时强制使用规则文件配置。 | `EngineerCaptureLevel`、`EngineerDamage`、`EngineerAlwaysCaptureTech`、`MultiplayPassive` | 建议由工程师进入所有者统一裁决占领或伤害，并复用建筑归属与伤害流程。 | [Multi Engineer](https://ares-developers.github.io/Ares-docs/restored/multiengineer.html) |
| AR380 多工程师选项界面 | 可在遭遇战菜单显示多工程师复选框，让玩家决定是否采用规则文件中的占领门槛与工程师伤害。关闭多工程师时单名工程师可直接占领；不显示开关则强制规则配置，目前不支持通过此菜单扩展到网络或在线对局。 | `AllowMultiEngineer`、`EngineerDamage`、`EngineerCaptureLevel` | 建议由对局选项所有者保存模式开关，工程师行为查询统一生效配置。 | [Multi Engineer Checkbox](https://ares-developers.github.io/Ares-docs/restored/multiengineercheckbox.html) |
| AR381 单位与建筑探照灯 | 四类可作战对象都能拥有探照灯，建筑维持旋转搜索，其他单位固定朝前并可对齐车体或炮塔。可设置高度、距离、颜色通道及暗化效果；隐形、停用、超时空或钻地期间隐藏，解除隐形后可正确恢复，建筑不支持炮塔对齐。 | `HasSpotlight`、`Spotlight.StartHeight`、`Spotlight.Distance`、`Spotlight.AttachedTo`、`Spotlight.DisableRed`、`Spotlight.DisableGreen`、`Spotlight.DisableBlue`、`Spotlight.DisableColor` | 建议由探照灯表现所有者读取朝向与可用状态，探测事件交给侦察机制处理。 | [Spotlights](https://ares-developers.github.io/Ares-docs/restored/spotlights.html) |
| AR382 恢复车辆劫持者 | 可创建类似变异劫持者的车辆窃取步兵，而不必额外开启偷窃、渗透或间谍标记，避免把不相关行为绑定在一起。该页作为恢复机制的说明入口，音效、目标许可、心控转移和幸存处理等具体选项应参见劫持者专题。 | `Thief`、`Infiltrate`、`Agent` | 建议由劫持任务所有者独立处理车辆接管，具体约束复用劫持者扩展配置。 | [Vehicle Thief](https://ares-developers.github.io/Ares-docs/restored/vehicle-thief.html) |

## 单位与通用玩法

本节 87 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR153 额外护甲类型与弹头倍率 | 新增护甲可继承原有护甲的弹头倍率，或指定统一默认倍率，再由弹头逐类覆盖。强制攻击、还击和自动索敌可分别配置，不再被零至百分之二的特殊倍率绑定，也可组合出可被瞄准但免伤的对象。 | `Versus.magic`、`Versus.magic.ForceFire`、`Versus.steel.Retaliate`、`Versus.clingfilm.PassiveAcquire`、`Verses`、`Armor` | 建议让规则解析定义护甲关系，由伤害与索敌所有者共同消费同一结果。 | [Additional ArmorTypes and Verses](https://ares-developers.github.io/Ares-docs/new/additionalarmortypesandverses.html) |
| AR154 飞机独立装填间隔 | 为每种飞机单独指定停靠补充一发弹药所需的分钟数，便于区分少量重弹药和大量轻弹药的补给节奏。仅在可装填建筑中生效；每架飞机独立计时，避免先降落者少等一个周期的问题。 | `ReloadRate`、`UnitReload` | 建议接入飞机弹药与停靠补给所有者，按实例维护装填计时。 | [Aircraft-Specific ReloadRate](https://ares-developers.github.io/Ares-docs/new/aircraftreload.html) |
| AR155 飞机受损与坠毁烟雾 | 飞机可独立选择重伤和坠毁时产生的烟雾动画，并分别设置每帧生成概率。适用于飞机损伤表现的差异化；这些概率描述烟雾生成频率，不负责改变坠毁规则或飞机受到的伤害。 | `Smoke.Anim`、`Smoke.ChanceRed`、`Smoke.ChanceDead` | 建议由飞机状态提供损伤阶段，由表现层按配置生成烟雾。 | [Aircraft Smoke](https://ares-developers.github.io/Ares-docs/new/aircraftsmoke.html) |
| AR156 空中动画播放间隔 | 为空中状态增加独立动画推进间隔，覆盖移动与待机动画速度，使起飞、降落和悬停中的直升机式单位仍能转动旋翼。数值必须大于零才启用；设为零则继续使用原有动画节奏。 | `AirRate`、`WalkRate`、`IdleRate` | 建议接入单位动画状态选择，由表现层统一决定每次帧推进。 | [AirRate](https://ares-developers.github.io/Ares-docs/new/airrate.html) |
| AR157 空袭呼叫与中止语音 | 空袭指示单位和执行飞机可分别定制入场呼叫语音及指示者过早死亡时的中止语音。指示单位设置优先于飞机，均未设置时回退到全局配置；可使用空声音条目关闭相应提示。 | `VoiceAirstrikeAttack`、`VoiceAirstrikeAbort`、`AirstrikeAttackVoice`、`AirstrikeAbortSound` | 建议由空袭任务生命周期产生事件，音频层按指示者优先级选声。 | [Airstrike Voices](https://ares-developers.github.io/Ares-docs/new/airstrike.html) |
| AR158 按战区替换单位美术 | SHP单位可按当前地图战区选择附加战区字母的美术节，缺少对应节时回退到基础图像节。它把原本步兵雪地替图推广到更多战区和单位；体素不支持此机制，不能借此切换体素外观。 | `AlternateTheaterArt`、`AlternateArcticArt`、`Image`、`Prerequisite.RequiredTheaters` | 建议在资源解析所有者中实现战区候选与回退，向绘制层交付结果。 | [AlternateTheaterArt](https://ares-developers.github.io/Ares-docs/new/alternatetheaterart.html) |
| AR159 动画伤害的武器与间隔 | 动画伤害可改用已登记武器创建并立即引爆弹体，以触发部分武器效果，伤害量仍取自动画。另可按动画帧设置两次伤害的间隔，要求伤害至少为一；该武器路径目前不继承动画创建者的玩家归属。 | `Weapon`、`Damage.Delay`、`Damage`、`Warhead`、`WeaponTypes`、`Rate` | 建议连接动画伤害与武器引爆所有者，并明确伤害计时和归属边界。 | [Animation Damage](https://ares-developers.github.io/Ares-docs/new/animations.html) |
| AR160 附加效果系统 | 单位或弹头可附加限时属性倍率、隐形能力及跟随动画，支持自刷新、重施与叠加。运输和超时空状态会暂停更新；隐形会移除动画但保留属性，射速倍率在装填开始时结算，可能持续影响到效果结束之后。 | `AttachEffect.Animation`、`AttachEffect.Duration`、`AttachEffect.SpeedMultiplier`、`AttachEffect.ArmorMultiplier`、`AttachEffect.FirepowerMultiplier`、`AttachEffect.ROFMultiplier`、`AttachEffect.Cumulative`、`AttachEffect.DiscardOnEntry` | 建议由持续效果所有者管理实例与生命周期，战斗及移动查询共享修正值。 | [AttachEffect](https://ares-developers.github.io/Ares-docs/new/attacheffect.html) |
| AR161 击杀赏金 | 单位和建筑可因射击或碾压消灭敌人获得赏金，金额由受害者类型和军衔定义，也可为负数。可要求持有特定建筑才启用，并控制金额浮字；国家可禁止其所属对象被击杀时提供赏金。 | `BountyEnablers`、`BountyDisplay`、`Bounty`、`Bounty.Display`、`Bounty.Value`、`Bounty.RookieValue`、`Bounty.EliteValue`、`GivesBounty` | 建议让击杀归属所有者触发经济结算，再由界面消费赏金显示事件。 | [Bounty](https://ares-developers.github.io/Ares-docs/new/bounty.html) |
| AR190 按类型定制建造时间 | 每类单位或建筑可独立配置基础建造速度、用于计时的虚拟造价、缺电惩罚及其上下界，并调整额外同类工厂的加速倍率。设置专属基础速度后，墙体原有的建造速度系数不再另行叠加。 | `BuildTime.Speed`、`BuildTime.Cost`、`BuildTime.LowPowerPenalty`、`BuildTime.MinLowPower`、`BuildTime.MaxLowPower`、`BuildTime.MultipleFactory` | 建议在生产队列所有者集中计算建造耗时，SDK仅提供类型配置入口。 | [Build Time](https://ares-developers.github.io/Ares-docs/new/buildtime.html) |
| AR191 警戒状态自动索敌控制 | 可让人类玩家单位在普通警戒中停止自行索敌，而在区域警戒中仍主动攻击，也可仅对具备主动隐形能力的单位如此处理。要求开启原有自动索敌，不适用于建筑或AI；行进射击已保留的目标不会因此自动清除。 | `CanPassiveAquire`、`CanPassiveAquire.Guard`、`CanPassiveAquire.Cloak`、`OpportunityFire`、`Cloakable` | 建议接入任务与目标选择所有者，在自动索敌入口统一判定警戒条件。 | [CanPassiveAquire Settings](https://ares-developers.github.io/Ares-docs/new/canpassiveaquire.html) |
| AR192 运输机吊运规则 | 吊运飞机可限制被运车辆的体积，车辆也能显式允许或拒绝吊运，覆盖有机体和非车辆的默认排除。感染寄生物的单位不能吊起；吊运机不能绑定机场或停靠列表，舰船落地还会同时毁掉自身和运输机。 | `Carryall`、`Carryall.SizeLimit`、`Carryall.Allowed`、`AirportBound`、`Dock`、`Size` | 建议由运输与地形放置所有者共同检查吊运资格、承载状态和安全落点。 | [Carryalls](https://ares-developers.github.io/Ares-docs/new/carryalls.html) |
| AR193 超时空监狱与绑架武器 | 绑架武器可把符合体积和生命门槛的目标收入乘员舱，容纳失败时改造成常规伤害，并支持超时空抹除结束后再收容及可选归属转换。车辆和飞机是主要适用对象；建筑与步兵载具可能异常，必须正确设置乘员容量。 | `Abductor`、`Abductor.Temporal`、`Abductor.ChangeOwner`、`Abductor.AbductBelowPercent`、`Abductor.MaxHealth`、`ImmuneToAbduction`、`PassengerTurret`、`SizeLimit` | 建议协调武器命中、乘员容器和归属所有者，统一处理收容与释放。 | [Chrono Prisons / Abductors](https://ares-developers.github.io/Ares-docs/new/chronoprisons.html) |
| AR194 超时空传送对象规则 | 可按类型禁止超时空传送，将部署后的建筑视为车辆处理，并决定单位能否被传送落点碾压。免疫落点碾压时抵达者会被摧毁；建筑不受此免疫项影响，另可全局禁止步兵借传送碾压车辆。 | `Chronoshift.Allow`、`Chronoshift.IsVehicle`、`Chronoshift.Crushable`、`ChronoInfantryCrush`、`Chronosphere.ReconsiderBuildings` | 建议在超时空能力所有者内统一筛选目标、部署形态与落点冲突结果。 | [Chronoshift](https://ares-developers.github.io/Ares-docs/new/chronoshift.html) |
| AR195 中立敌人与协防 | 类型可在多人游戏中被视为自动索敌的敌人，避免危险平民始终保持中立。全局还可分别让AI和人类单位主动攻击正在伤害友军的中立对象；未启用协防时，通常只有实际遭攻击者会还击。 | `CivilianEnemy`、`AutoRepel`、`PlayerAutoRepel`、`MultiplayPassive`、`Insignificant` | 建议由外交与索敌所有者判断中立威胁，附近单位复用统一敌对判定。 | [Civilian Enemies](https://ares-developers.github.io/Ares-docs/new/civilianenemies.html) |
| AR203 飞机坠毁与旋转 | 飞机被摧毁时可选择直接在空中消失，或继续落向地面产生坠毁效果；保留坠毁时还能关闭失控旋转，使大型飞机表现为滑落。旋转开关仅支持飞机类型，跳跃喷气式车辆不受此选项控制。 | `Crashable`、`CrashSpin`、`JumpJet` | 建议由飞机死亡运动所有者确定坠毁流程，绘制层读取旋转开关。 | [Crashing](https://ares-developers.github.io/Ares-docs/new/crashableaircraft.html) |
| AR204 金钱箱随机金额 | 金钱箱奖励可以自定义额外随机金额的上限，用于调整拾取收入的波动。实际奖励会在既有金额之上再增加零到此上限的随机数；设为零即可消除这部分随机波动，默认额外上限为九百资金。 | `RandomCrateMoney` | 建议由箱子奖励所有者统一使用同步随机源，并通过经济所有者入账。 | [Crate Settings](https://ares-developers.github.io/Ares-docs/new/crates.html) |
| AR205 碾压反伤 | 被碾压对象可以向碾压者反向造成指定弹头伤害，适用于让碾压步兵付出代价，并能按新兵、老兵和精英分别设置数值。基础值重设时会先覆盖既有军衔值，再读取同一配置文件中的军衔覆盖项。 | `CrushDamage`、`CrushDamage.Rookie`、`CrushDamage.Veteran`、`CrushDamage.Elite`、`CrushDamage.Warhead` | 建议接入碾压伤害所有者，按受害者军衔读取反伤并沿正常伤害链结算。 | [Crush Damage](https://ares-developers.github.io/Ares-docs/new/crushdamage.html) |
| AR206 单位与建筑的动作光标 | 每种对象可选移动、禁止移动、部署和进入等命名光标，建筑还可定制间谍渗透光标。进入动作取鼠标指向对象的配置；占领建筑、破坏者和劫持者使用其他动作路径，不能把所有进入样式都靠此页设置。 | `Cursor.Move`、`Cursor.NoMove`、`Cursor.Deploy`、`Cursor.NoDeploy`、`Cursor.Enter`、`Cursor.NoEnter`、`Cursor.Spy` | 建议由命令判定所有者返回动作资格，界面层按对象类型解析命名光标。 | [Cursors on Units and Structures](https://ares-developers.github.io/Ares-docs/new/cursors.html) |
| AR207 动画与弹体自定义调色板 | 动画和弹体可指定独立调色板，不再只在原有动画与单位色板间选择。文件名中的三个波浪号会替换成当前战区的三字母后缀；对弹体使用时，必须将其同时登记进动画列表，才能正确应用该设置。 | `CustomPalette` | 建议由资源所有者解析战区色板，动画和弹体绘制复用同一色板选择能力。 | [Custom Animation and Projectile Palettes](https://ares-developers.github.io/Ares-docs/new/customanimationandprojectilepalettes.html) |
| AR208 侧边栏图标自定义调色板 | 单位建造图标与超级武器图标可分别选择独立调色板，配置位置分别位于美术与规则文件。普通和升级图标必须共用同一色板；默认仍使用标准图标色板，索引零在游戏中透明，不应作为可见颜色使用。 | `CameoPalette`、`SidebarPalette`、`Cameo`、`AltCameo`、`SidebarImage` | 建议由图标资源所有者解析色板与透明索引，侧边栏只消费已绑定资源。 | [Custom Cameo Palettes](https://ares-developers.github.io/Ares-docs/new/customcameopalettes.html) |
| AR209 军衔徽记与敌方可见性 | 各类型可为新兵、老兵和精英指定不同徽记图像及帧号，也可通过全局或类型选项隐藏敌军徽记。自定义图像需采用战区调色板，未指定的升级徽记保持原有帧；观察者始终可以看到所有玩家的军衔。 | `Insignia.Rookie`、`Insignia.Veteran`、`Insignia.Elite`、`InsigniaFrame.Rookie`、`InsigniaFrame.Veteran`、`InsigniaFrame.Elite`、`EnemyInsignia`、`Insignia.ShowEnemy` | 建议由经验所有者提供军衔，表现层负责徽记资源和敌我可见性选择。 | [Veterancy Insignia](https://ares-developers.github.io/Ares-docs/new/customizableinsignia.html) |
| AR210 按类型设置降落伞动画 | 每种单位或建筑可在下落时选择自己的降落伞动画，使空降外观不再完全依赖统一资源。配置引用美术文件中的动画节，未设置时回退到标准降落伞；对应SHP图像应使用单位调色板。 | `Parachute.Anim` | 建议由空降状态触发表现事件，资源层按类型解析降落伞动画与默认值。 | [Customizable Parachute Animations](https://ares-developers.github.io/Ares-docs/new/customizableparachuteanimations.html) |
| AR211 经验来源与分配 | 可配置乘员、空袭、心控对象及子机击杀时的经验归属和倍率，用于在载具与乘员、指示者与飞机、母体与子机间分配收益。不可训练对象始终不获经验，部分分配要求双方可训练，敌对心控关系还会限制乘员经验。 | `Experience.FromPassengers`、`Experience.PromotePassengers`、`Experience.PassengerModifier`、`Experience.FromAirstrike`、`Experience.MindControlSelfModifier`、`Experience.MindControlVictimModifier`、`Experience.SpawnOwnerModifier`、`Experience.SpawnModifier` | 建议将击杀经验集中交给经验所有者，按乘员、心控和子机关系统一分配。 | [Customizable Veterancy](https://ares-developers.github.io/Ares-docs/new/customizableveterancy.html) |
| AR212 自定义导弹飞机 | 可把飞机类型定义为导弹，独立配置发射停顿、俯仰、加速、巡航轨迹、伤害和尾烟，突破三种原有导弹的固定组合。启用后必须填写有效参数；精英武器不会自动回退到普通武器，引用的武器还需事先登记。 | `Missile.Custom`、`Missile.PauseFrames`、`Missile.PitchFinal`、`Missile.Acceleration`、`Missile.LazyCurve`、`Missile.Warhead`、`Missile.Weapon`、`Missile.EliteWeapon` | 建议由导弹运动所有者驱动飞行阶段，并复用武器引爆与动画资源能力。 | [Custom Missiles](https://ares-developers.github.io/Ares-docs/new/custommissiles.html) |
| AR213 受损粒子系统 | 受损冒烟和持续火花可按对象类型分别指定候选粒子系统，并把火花开关从机械步兵专属行为中独立出来。未启用新列表时维持原有分类规则；文档仅保证烟雾和火花系统完整支持，其他行为还缺少专门处理。 | `DamageSparks`、`DamageParticleSystems`、`DamageSmokeParticleSystems`、`DamageSparksParticleSystems`、`BehavesLike`、`Cyborg` | 建议由损伤状态触发粒子表现，资源所有者统一解析候选系统及兼容限制。 | [Damage Particle Systems](https://ares-developers.github.io/Ares-docs/new/damageparticlesystems.html) |
| AR214 部署朝向 | 带有部署动画的单位在部署前会先转向指定八方向，不再要求同时开启部署落地逻辑，从而便于用动画掩盖形态转换。没有部署动画的单位不再无谓转向；类型未设置时沿用全局部署朝向。 | `DeployDir`、`DeployToLand`、`DeployingAnim` | 建议在部署任务所有者中安排转向与动画阶段，统一消费方向配置。 | [DeployDir](https://ares-developers.github.io/Ares-docs/new/deploydir.html) |
| AR215 钻地与出土动画声音 | 使用地下运动方式的单位可分别定制入地、出土时的动画和声音，替代全体地下单位共用同一组表现的限制。四项均可单独配置为可选效果，未设置时仍分别回退到原有全局钻地动画与声音。 | `DigIn`、`DigOut`、`DigInSound`、`DigOutSound`、`Dig`、`DigSound` | 建议由地下运动所有者发出进出地事件，表现层按类型解析动画与声音。 | [Dig Animation and Sound](https://ares-developers.github.io/Ares-docs/new/dig.html) |
| AR216 停用体素车辆的变暗程度 | 可分别设置EMP、缺少操作员和缺电导致停用时的体素车辆亮度，按首个适用原因取值，当前EMP优先。该效果仅改变外观，不覆盖SHP车辆、建筑、飞机或步兵；一表示保持原亮度，更大数值会增亮。 | `DeactivateDimEMP`、`DeactivateDimOperator`、`DeactivateDimPowered` | 建议由失能状态所有者提供原因，体素绘制层只读取对应亮度倍率。 | [Dimming Deactivated Units](https://ares-developers.github.io/Ares-docs/new/dimming.html) |
| AR217 飞碟激光命中动画 | 飞碟激光自动遵循武器的明亮设置，并可通过全局开关显示弹头动画列表中的命中动画。由于原版飞碟武器已配置动画，新增显示必须显式启用，避免默认外观随扩展改变；此项不修改吸取或伤害逻辑。 | `DiskLaserAnimEnabled`、`DiskLaser`、`AnimList`、`Bright` | 建议在飞碟激光效果所有者中统一处理亮度和可选命中动画事件。 | [Disk Laser Animations](https://ares-developers.github.io/Ares-docs/new/disklaser.html) |
| AR218 泰伯利亚恶魔行为 | 步兵可获得泰伯利亚恶魔的野生动物行为：警戒时趴在矿区，重伤时中止进攻并逃向矿区，且容易被激怒。需要允许爬行才会趴下，非人类设置用于匹配死亡表现；自动抵御其中立攻击还需配合平民敌对规则。 | `Doggie`、`Crawls`、`NotHuman` | 建议让步兵任务所有者处理矿区休息和逃跑，并复用中立敌对判定。 | [Tiberian Fiends](https://ares-developers.github.io/Ares-docs/new/doggie.html) |
| AR219 局部电力吸取 | 吸取单位或被吸取建筑任一方开启局部模式后，吸取只针对目标建筑的发电量，不再拖垮所属玩家的整个电网。吸取量从建筑发电值起算，先应用攻击者再应用建筑的限值，正数限制上限、负数限制下限。 | `Drain.Local`、`Drain.Amount`、`Power` | 建议由电力所有者结算吸取占用，武器关系仅提供目标与类型限值。 | [Drain Weapon Options](https://ares-developers.github.io/Ares-docs/new/drain.html) |
| AR220 空降舱全局设置 | 为空降舱运动和超级武器提供尾烟、随机步兵候选与生成数量上下限，未设置攻击武器也能正常产生尾烟。超级武器只支持步兵，候选可重复以调整抽取权重；数量与类型默认值不能直接构成可用配置。 | `DropPodTrailer`、`DropPodTypes`、`DropPodMinimum`、`DropPodMaximum`、`DropPodWeapon`、`Report` | 建议由空降生成所有者处理同步抽取与落地，表现层消费舱体尾烟配置。 | [Drop Pods](https://ares-developers.github.io/Ares-docs/new/droppod.html) |
| AR221 地图编辑器支持 | 地图编辑器需要先识别Ares新增脚本、事件和动作的名称及参数格式。FinalAlert2可合并官方提供的定义来编辑触发器；脚本界面另依赖未随Ares提供的FA2Ext扩展，已添加参数类型的环境还要调整定义兼容。 | 无独立键或见原文 | 建议提供独立编辑器元数据适配，脚本与触发语义仍由引擎所有者执行。 | [Editor Support](https://ares-developers.github.io/Ares-docs/new/editorsupport.html) |
| AR222 敌方建筑显示名称 | 建筑可为敌对观察者提供替代提示名称，用于隐藏真实身份或假建筑的名称。盟友、观察者和已渗透该建筑的玩家仍看到真实名称；该功能目前仅适用于建筑类型，不能直接用于车辆、飞机或步兵。 | `EnemyUIName`、`UIName` | 建议由情报与界面所有者共同判定名称可见性，统一查询文本资源。 | [EnemyUIName](https://ares-developers.github.io/Ares-docs/new/enemyuiname.html) |
| AR223 自定义EVA播报类型 | 可在EVA配置文件登记任意数量的播报类型，再为各EVA事件按类型指定音频文件，并供阵营选择使用。三种原有播报默认保留；文件名最多八字符，某类型未为事件填写文件名时，该事件不会播放声音。 | 无独立键或见原文 | 建议由语音资源所有者登记播报类型，阵营选择与事件查音共用注册表。 | [EVA Types](https://ares-developers.github.io/Ares-docs/new/evatypes.html) |
| AR224 SHP车辆方向数量 | SHP车辆可使用多于八个方向的图像，让车辆转向显示更细致。方向数在美术配置中设置，只支持不小于八的二次幂；少于八会只显示首帧。该设置针对SHP车辆，不用于体素车辆的方向计算。 | `Facings` | 建议由车辆美术资源所有者校验帧布局，绘制层统一映射朝向到图像帧。 | [Facings on Units](https://ares-developers.github.io/Ares-docs/new/facings.html) |
| AR225 工业工厂折扣倍率 | 各类型可调整工业工厂等建筑对自身产生的降价效果，支持削弱、消除或反转折扣，便于对不同单位单独平衡。该倍率线性缩放折扣的幅度；超出文档完整支持范围的较大值可能把最终造价压到零。 | `FactoryPlant.Multiplier`、`FactoryPlant`、`CostBonus` | 建议在经济造价所有者统一合成工厂折扣，生产与界面共享最终价格。 | [Factory Plant Effect](https://ares-developers.github.io/Ares-docs/new/factoryplant.html) |
| AR226 伪装类型的选择估值 | 假单位或诱饵可指定被模仿的对象类型，使依据生命和军衔进行单位选择的快捷键使用该类型的价格计算。它只处理这些选择命令的估值，避免价格差异暴露身份；没有指定时维持原有行为。 | `FakeOf` | 建议在选择命令的估值所有者中读取伪装参考，不改动实际经济造价。 | [FakeOf](https://ares-developers.github.io/Ares-docs/new/fakeof.html) |
| AR227 空降下落速度 | 可按类型分别设置带降落伞和不带伞下落时的逐帧重力加速度，以及各自的最大下降速度，用于区分不同空投对象。最大速度必须使用负数；未设置时沿用全局速度限制，两个加速度默认均为一。 | `FallRate.Parachute`、`FallRate.NoParachute`、`FallRate.ParachuteMax`、`FallRate.NoParachuteMax` | 建议由空降运动所有者管理垂直速度，SDK提供按类型的参数配置。 | [Fall Rate](https://ares-developers.github.io/Ares-docs/new/fallrate.html) |
| AR228 森林火灾蔓延 | 易燃树木被可点火弹头命中后可燃烧，并逐步引燃邻近树木，恢复森林火灾。树木必须明确设为易燃，还要为全局火焰列表配置足量有效动画以免崩溃；小火动画的灼痕选项可能妨碍树木烧毁。 | `Sparky`、`IsFlammable`、`OnFire`、`TreeFire`、`SmallFire`、`Scorch` | 建议由地形燃烧所有者管理点火与传播，动画伤害和资源解析复用现有能力。 | [Forest Fires](https://ares-developers.github.io/Ares-docs/new/forestfires.html) |
| AR229 盖特武器循环 | 盖特单位可在最高武器阶段完成后回到第一阶段继续升速，同时保留当前攻击目标，避免用单次开火绕行导致丢目标。必须开启盖特逻辑；实际各阶段的开火次数仍受升降速参数和实例状态影响，不能保证固定次数。 | `Gattling.Cycle`、`IsGattling`、`Stage`、`EliteStage`、`RateUp`、`RateDown`、`FireOnce` | 建议接入盖特阶段所有者，在阶段边界循环计数并保留任务目标。 | [Gattling Enhancements](https://ares-developers.github.io/Ares-docs/new/gattlingcycle.html) |
| AR230 多种步兵战车与乘员武器 | 所有炮手车辆都能按首名乘员选择武器、炮塔和提示名称，并定制维修应答，扩展原本仅特定步兵战车读取的映射。炮塔数量可超过十八但不支持超过一百二十七；武器序号从一开始，乘员模式从零开始。 | `Gunner`、`TurretCount`、`WeaponTurretIndexX`、`WeaponUINameX`、`IFVMode`、`WeaponCount`、`VoiceIFVRepair` | 建议由炮手载具所有者解析乘员模式，武器、炮塔与界面共用选定模式。 | [Gunner IFVs](https://ares-developers.github.io/Ares-docs/new/gunner.html) |
| AR231 解除单位编号专属行为 | 原本绑定奶牛和辐射工兵编号的行为可改为各步兵类型的独立选项，也能关闭原单位默认能力。奶牛选项影响频繁待机和随机移动；辐射工兵选项影响部署开火时机及相关地面辐射行为，仍需合适的部署与武器配置。 | `IsCow`、`IsDesolator`、`ImmuneToRadiation`、`RadLevel` | 建议由步兵任务与部署所有者读取显式能力，避免在行为代码中分散编号判断。 | [Hard-coded Unit Properties](https://ares-developers.github.io/Ares-docs/new/hardcodedunitproperties.html) |
| AR232 矿车与奴隶采集参数 | 矿车、奴隶矿场和奴隶可按类型覆盖长短扫描距离，奴隶矿场还可调整前移收益判断和自动唤醒间隔。矿车另能定义预定矿厂的过远门槛；各项只对相应采集角色有效，唤醒间隔设为负一可禁用自动唤醒。 | `Harvester.LongScan`、`Harvester.ShortScan`、`Harvester.ScanCorrection`、`Harvester.TooFarDistance`、`Harvester.KickDelay` | 建议让采集任务所有者集中维护搜索、预定与唤醒行为，按角色读取配置。 | [Harvesters, Slave Miners, and Slaves](https://ares-developers.github.io/Ares-docs/new/harvesters.html) |
| AR233 劫持者扩展 | 车辆劫持者可配置进出音效、一次性消耗、击杀驾驶员数量与能否切断心控，目标也可独立禁止被劫持。通常不能劫持友军或直接驾驶中立化车辆；幸存劫持者保留军衔，并按规则恢复随机生命，心控链接会随转移清理。 | `VehicleThief`、`VehicleThief.EnterSound`、`VehicleThief.LeaveSound`、`VehicleThief.BreakMindControl`、`VehicleThief.OneTime`、`VehicleThief.KillPilots`、`VehicleThief.Allowed`、`CanDrive` | 建议协调劫持、归属与心控所有者，统一处理进入、毁灭逃生和链接迁移。 | [Hijackers](https://ares-developers.github.io/Ares-docs/new/hijackers.html) |
| AR234 猎杀者飞行器 | 猎杀者由超级武器从指定建筑发射，可配置起降速度、接近目标下降与引爆距离，并按类型排除不应被随机选中的目标。仅支持使用飞行运动方式的体素车辆；没有合格发射建筑时仍会消耗超级武器，但不会生成猎杀者。 | `HunterSeeker`、`HSBuilding`、`HunterSeeker.DetonateProximity`、`HunterSeeker.DescendProximity`、`HunterSeeker.AscentSpeed`、`HunterSeeker.DescentSpeed`、`HunterSeeker.EmergeSpeed`、`HunterSeeker.Ignore` | 建议由猎杀者任务所有者管理选敌与飞行，引爆和超级武器发射复用各自能力。 | [Hunter Seeker](https://ares-developers.github.io/Ares-docs/new/hunterseeker.html) |
| AR236 步兵电击死亡动画 | 全局可明确指定步兵遭电击死亡时播放的动画，替代固定取动画列表第二项的编号依赖。该选项对应弹头电击死亡方式；未设置时先查找标准电击动画名称，仍找不到才回退到列表第二项，便于保持旧配置兼容。 | `InfantryElectrocuted`、`InfDeath` | 建议让步兵死亡表现所有者统一解析动画回退，避免依赖资源注册顺序。 | [InfantryElectrocuted](https://ares-developers.github.io/Ares-docs/new/infantryelectrocuted.html) |
| AR237 初始乘员与驻军 | 单位和部分建筑创建时可自带指定类型、数量的乘员或驻军，形成出厂即满载的运兵载具。建筑仅支持可驻军或吸收步兵类型且只装步兵；不支持步兵容器、飞机或建筑载荷及嵌套初始载荷，体积适配需自行保证。 | `InitialPayload.Types`、`InitialPayload.Nums`、`CanBeOccupied`、`InfantryAbsorb`、`MaxNumberOccupants`、`Passengers`、`Size`、`SizeLimit` | 建议由对象创建与乘员容器所有者协同生成载荷，复用容量和驻军限制。 | [Initial Payload (Troop Crawlers)](https://ares-developers.github.io/Ares-docs/new/initialpayload.html) |
| AR238 短局存活判定 | 可按单位、步兵、飞机或建筑类型决定其是否维持所属玩家在短局中的存活，从而添加特殊存活对象或排除重要建筑。类型选项独立于计分和重要性标记；但全局基地单位列表中的车辆始终能保活，不能靠此选项取消。 | `KeepAlive`、`DontScore`、`Insignificant`、`BaseUnit` | 建议由胜负判定所有者集中统计保活对象，归属和销毁变化同步更新。 | [Keep Alive](https://ares-developers.github.io/Ares-docs/new/keepalive.html) |
| AR239 击杀驾驶员与接管 | 特定弹头可按概率和生命门槛击杀驾驶员、驱逐其他乘员并中立化车辆，另可移除军衔。对象可保护驾驶员或限制接管，有机和自然单位始终免疫；驾驶步兵可接管合格中立车辆，有操作员要求时会作为可卸载乘员进入。 | `KillDriver`、`KillDriver.Owner`、`KillDriver.KillBelowPercent`、`KillDriver.Chance`、`ProtectedDriver`、`ProtectedDriver.MinHealth`、`CanDrive`、`CanBeDriven` | 建议协调弹头效果、乘员与归属所有者，统一处理失能、驱逐和重新接管。 | [Killing Drivers](https://ares-developers.github.io/Ares-docs/new/killingdrivers.html) |
| AR240 单位损失EVA播报 | 单位或建筑被摧毁时，可按类型选择所属玩家听到的损失播报，也可关闭提示。默认建筑不播报，其他类型沿用单位损失提示；不计分、不重要或生成子机等对象即使配置了该项，也不会触发这类EVA消息。 | `EVA.Lost`、`DontScore`、`Insignificant`、`Spawned` | 建议由销毁归属所有者产生损失事件，语音层执行类型选择与过滤。 | [Unit or Structure Lost EVA Report](https://ares-developers.github.io/Ares-docs/new/losteva.html) |
| AR241 动画生成步兵的归属 | 生成步兵的特定死亡动画和地图触发动画，可选择施加者、击杀者、受害者、固定中立阵营或随机玩家作为新步兵主人，并影响重映射颜色。默认施加者含义随来源变化；动画链中生成项放末段，归属项必须放首段。 | `MakeInfantryOwner`、`MakeInfantry`、`Next`、`InfDeathAnim`、`DeathAnims` | 建议由动画上下文所有者传递来源归属，对象生成时统一决定主人和配色。 | [MakeInfantryOwner](https://ares-developers.github.io/Ares-docs/new/makeinfantryowner.html) |
| AR242 手动控制限制 | 可限制手动选敌、卸载或显示运输进入光标，并调整治疗步兵自身的区域警戒光标优先级。禁手动开火仍允许自动索敌和还击；隐藏进入光标只影响界面，脚本和AI仍可登载，禁止手卸也不改变被摧毁时的逃生规则。 | `NoManualFire`、`NoManualUnload`、`NoManualEnter`、`NoSelfGuardArea` | 建议在命令入口和界面光标层分别接入限制，运输准入仍交给容器所有者。 | [Manual Control](https://ares-developers.github.io/Ares-docs/new/manual.html) |
| AR252 命名鼠标光标 | 鼠标光标集中以名称登记，可定义动画帧、间隔、小地图版本和点击热点，再供单位或武器引用。原有光标隐式可用，新增特殊动作光标可独立覆盖；热点使用描述文字，不能直接照搬其他补丁数值，也不支持修改默认光标。 | `Name`、`Cursor` | 建议建立界面资源的光标注册表，动作判定仅返回名称并统一解析。 | [Mouse Cursors](https://ares-developers.github.io/Ares-docs/new/mousecursors.html) |
| AR253 电影列表与独立解锁 | 可在专门配置文件中按顺序列出电影，为每项指定显示名称及是否需要解锁，解锁状态按电影独立保存。电影名称不带扩展名且说明文字必填；只有列表为空时，才继续使用原版固定列表和阵营进度记录方式。 | `Description`、`Unlockable` | 建议由媒体目录与用户进度所有者管理清单和解锁，菜单读取统一结果。 | [Movies List](https://ares-developers.github.io/Ares-docs/new/movies.html) |
| AR254 多来源供电单位 | 单位可列出多个建筑类型作为供电来源，从而突破每个阵营只能有一种受供电单位和一种供电建筑的限制。建筑列表按任一满足计算，逗号表示或关系，不提供要求全部建筑同时存在的与关系。 | `PoweredBy`、`PowersUnit` | 建议由单位供电所有者查询建筑资格与有效状态，SDK只表达候选来源列表。 | [New Powered Unit Logic](https://ares-developers.github.io/Ares-docs/new/newpoweredunitlogic.html) |
| AR255 低弹药时强制换武器 | 单位或建筑弹药降到指定门槛时，可强制改用主武器或副武器，适合切换备用武器或限制射击。此选择会覆盖普通选武器流程，负一关闭；不能覆盖车辆或步兵部署开火所固定选择的武器，仍需遵循所选武器的弹药要求。 | `NoAmmoWeapon`、`NoAmmoAmount`、`DeployFireWeapon`、`DeployFire`、`OpenTransportWeapon` | 建议在武器选择所有者中查询弹药门槛，并与部署开火优先级统一裁决。 | [NoAmmoWeapon](https://ares-developers.github.io/Ares-docs/new/noammoweapon.html) |
| AR256 全能碾压的进攻性 | 全能碾压单位可以关闭主动贴近敌人以实施碾压的行为，避免攻击或还击命令使重型载具不断向敌群深处追撞。必须先具有全能碾压能力；此设置只限制主动靠近策略，不负责取消单位原有的碾压资格。 | `OmniCrusher.Aggressive`、`OmniCrusher` | 建议由攻击接近与碾压所有者共同选择距离，保持碾压资格判定唯一。 | [OmniCrusher Enhancements](https://ares-developers.github.io/Ares-docs/new/omnicrusher.html) |
| AR257 操作员要求 | 建筑和车辆可要求舱内至少有指定类型或任意乘员才能运作；缺员车辆不能移动和开火，建筑只禁武器，不影响生产供电等功能。需正确配置舱位和吸收能力，多种特殊运输或部署类型不兼容，AI使用也不稳定。 | `Operator`、`Passengers`、`SizeLimit`、`InfantryAbsorb`、`UnitAbsorb`、`DeploysInto`、`BalloonHover` | 建议由乘员所有者提供操作资格，失能与开火所有者复用同一判定。 | [Operator](https://ares-developers.github.io/Ares-docs/new/operator.html) |
| AR258 粒子优化、伤害范围与色板 | 粒子处理自动优化兼容的火花、轨道炮和烟雾组合；气体粒子可扩大伤害范围，形状粒子可选独立色板。优化要求系统与所含粒子行为一致且无特殊附加图像或拖线；范围伤害每对象至多一次，零范围保留本格判定。 | `BehavesLike`、`HoldsWhat`、`DamageRange`、`Palette` | 建议由粒子所有者维护模拟与批处理，伤害筛选和绘制色板分别接入共享能力。 | [Particles](https://ares-developers.github.io/Ares-docs/new/particles.html) |
| AR259 乘员类型与容量计算 | 运输容器可用允许和拒绝名单限制乘员类型，补充原有体积与体积上限检查；非空允许名单会排除未列出的类型。车辆还可改为每对象只占一个座位，但最大可进入体积限制始终有效，不能靠忽略占位体积绕过。 | `Passengers.Allowed`、`Passengers.Disallowed`、`Passengers.BySize`、`Size`、`SizeLimit`、`Passengers` | 建议由乘员容器所有者统一执行类型准入、单体上限与总容量判定。 | [Specific Passengers](https://ares-developers.github.io/Ares-docs/new/passengers.html) |
| AR260 PCX建造图标 | 单位普通和升级建造图标以及超级武器侧栏图标，可以使用PCX文件替代SHP图像。单位选项位于美术配置，超级武器选项位于规则配置，文件名包含扩展名；图片必须为二百五十六色且尺寸为六十乘四十八像素。 | `CameoPCX`、`AltCameoPCX`、`SidebarPCX` | 建议在图标资源所有者中增加PCX来源解析，侧栏复用统一图标接口。 | [PCX Cameos](https://ares-developers.github.io/Ares-docs/new/pcxcameos.html) |
| AR261 点标图像帧索引 | 点标配置除原有颜色或人物名称外，也可直接填入整数作为点标图像文件的帧序号，以便使用自定义点标。不能识别的非整数值会记录解析错误；整数越界却不会自动记错，因此配置方仍需保证索引落在实际图像帧数内。 | `Pip`、`OccupyPip` | 建议由点标资源所有者统一解析命名值与帧索引，并在资源绑定时检查范围。 | [Pips](https://ares-developers.github.io/Ares-docs/new/pips.html) |
| AR262 建造前置条件扩展 | 前置系统可限制战区、排除特定建筑、接受多组替代清单、要求偷取科技或特定国家原建工厂，也能自定义通用组及非建筑替代物。替代清单按任一组满足；战区和负前置不受前置覆盖绕过，AI会忽略建筑的工厂来源限制。 | `Prerequisite.RequiredTheaters`、`Prerequisite.Negative`、`Prerequisite.Lists`、`Prerequisite.List#`、`Prerequisite.StolenTechs`、`FactoryOwners`、`FactoryOwners.Forbidden`、`PrerequisiteXAlternate` | 建议由建造资格所有者统一组合全部条件，SDK提供规则表达与诊断入口。 | [Prerequisites](https://ares-developers.github.io/Ares-docs/new/prerequisites.html) |
| AR269 升级提示与乘员同步军衔 | 可定制升级到老兵或精英时的声音、EVA及闪烁时长，并在运输单位升级时把所有可训练乘员设为同一军衔。同步转发的是军衔而非经验比例，会覆盖乘员原有经验甚至降低其等级，因此更适合固定载荷的模拟多武器单位。 | `VeteranFlashTimer`、`Promote.VeteranSound`、`Promote.EliteSound`、`Promote.VeteranFlash`、`Promote.EliteFlash`、`EVA.VeteranPromoted`、`EVA.ElitePromoted`、`Promote.IncludePassengers` | 建议由经验所有者处理军衔同步并发出升级事件，声音与闪烁由表现层消费。 | [Promotion settings](https://ares-developers.github.io/Ares-docs/new/promotion.html) |
| AR270 范围雷达干扰 | 干扰对象只需靠近敌方雷达或间谍卫星建筑，就可在指定半径内压制其侦察功能，无需开火或部署。干扰仅影响雷达和卫星能力，不会让建筑整体停工；友军不受影响，敌方仍可依靠范围之外的其他雷达保持雷达可用。 | `RadarJamRadius`、`Radar`、`SpySat` | 建议由侦察覆盖所有者维护建筑干扰状态，电力和其他建筑功能保持各自职责。 | [Radar Jammers](https://ares-developers.github.io/Ares-docs/new/radarjammers.html) |
| AR271 自定义范围指示圈 | 单位与建筑可独立指定范围指示圈半径，既能覆盖自动推导的范围，也能为原本没有此类范围的对象添加显示。必须开启范围圈显示标志；单位仅支持普通静态圈，不支持扫描旋转或同心圈样式。 | `RadialIndicatorRadius`、`HasRadialIndicator` | 建议由表现层读取范围配置，能力所有者仅提供缺省有效范围供回退。 | [Radial Indicators](https://ares-developers.github.io/Ares-docs/new/radialindicators.html) |
| AR272 每次装填的弹药数量 | 单位可分别设置普通装填周期和空弹药装填周期恢复的弹数，补充原有仅能定义装填间隔的机制。普通数量允许负数，空弹数量为零或负数时无法恢复；空弹数量不要求单独设定空弹间隔，且此逻辑不支持飞机。 | `ReloadAmount`、`EmptyReloadAmount`、`Reload`、`EmptyReload` | 建议由弹药所有者在计时完成时统一应用恢复量，并明确空弹分支优先级。 | [ReloadAmount and EmptyReloadAmount](https://ares-developers.github.io/Ares-docs/new/reload.html) |
| AR273 逆向工程解锁 | 具备逆向工程能力的回收建筑可从送入的步兵或车辆获得可生产类型，目标可禁止逆向或指定解锁为另一类型。当前要求建筑支持回收；解锁可绕过多种普通科技条件，但仍遵循建造上限、战区、负前置和工厂等限制。 | `ReverseEngineersVictims`、`Grinding`、`CanBeReversed`、`ReversedAs`、`BuildLimit`、`Prerequisite.RequiredTheaters`、`Prerequisite.Negative` | 建议由科技解锁所有者持有逆向成果，回收链提交结果并复用建造资格查询。 | [Reverse Engineer logic](https://ares-developers.github.io/Ares-docs/new/reverseengineerlogic.html) |
| AR274 自爆破坏者 | 步兵可进入敌方可破坏建筑并以自身消耗换取建筑立即毁坏，建筑也能独立设置免疫。需要开启渗透能力，不能与劫持、驾驶、C4或驻军逻辑混用；只可对敌方建筑生效，原设想中的临时隐形尚未实现。 | `Saboteur`、`ImmuneToSaboteurs`、`Infiltrate`、`CanDrive`、`C4`、`Occupier`、`CanC4` | 建议由渗透任务所有者完成准入与牺牲，建筑毁坏复用正常C4销毁流程。 | [Saboteurs](https://ares-developers.github.io/Ares-docs/new/saboteur.html) |
| AR275 动画结束后的余火 | 动画结束时可在同一附着对象上续接小火，或在周围随机产生大小火焰，恢复燃烧弹余火机制。附着小火有地形限制，火焰资源缺失不再导致崩溃；小火自身不能再开启续接，否则会无限循环并令燃烧树木无法烧尽。 | `Scorch`、`Flamer`、`SmallFire`、`LargeFire` | 建议由动画生命周期所有者派发后续火焰，地形过滤与资源回退集中处理。 | [Scorch and Flamer](https://ares-developers.github.io/Ares-docs/new/scorchflamer.html) |
| AR276 小队脚本动作扩展 | 小队脚本增强了敌我基地集结距离和铁幕分组选择，并新增辅助供电、击杀驾驶员、接管车辆、类型转换、声呐显形及禁武器动作。各动作的作用对象与参数不同，持续时间为零可结束相应限制，编号与参数须按逐项定义使用。 | `AISafeDistance`、`AIFriendlyDistance`、`SW.AITargetingMode`、`CanDrive`、`VehicleThief` | 建议由脚本调度所有者派发领域命令，电力、乘员和效果状态仍归各自所有者。 | [Script Actions](https://ares-developers.github.io/Ares-docs/new/scripts.html) |
| AR277 自修复规则 | 对象可按类型配置自修复间隔、每次恢复量及最大恢复生命比例，并按军衔细分上限和数量。统一值会覆盖对应军衔项，恢复量不得为负；受到伤害后还可暂停一段帧数，治疗武器不会触发这一战斗延迟。 | `SelfHealing.Rate`、`SelfHealing.Max`、`SelfHealing.RookieMax`、`SelfHealing.EliteMax`、`SelfHealing.Amount`、`SelfHealing.RookieAmount`、`SelfHealing.EliteAmount`、`SelfHealing.CombatDelay` | 建议由生命与修复所有者统一维护计时、上限和受击延迟，避免独立回血状态。 | [Self Heal](https://ares-developers.github.io/Ares-docs/new/selfheal.html) |
| AR278 单位出售资格 | 可按建筑类型决定是否允许出售停靠单位，也能按单位类型禁止出售，并设定全局默认值。人类玩家使用出售按钮指向停靠对象时才获得相应操作光标；建筑默认资格沿用维修功能，单位禁售默认取全局配置。 | `UnitSell`、`Unsellable`、`UnitsUnsellable`、`UnitRepair` | 建议由出售命令所有者统一校验建筑与单位资格，再委托经济所有者退款。 | [Sell Units](https://ares-developers.github.io/Ares-docs/new/sellunit.html) |
| AR286 扩大视野范围 | 单位视野可超过原有十格上限，允许配置更大侦察范围，但大范围会增加运算负担，不宜把它作为全图揭示工具。超过十格时视域呈明显八边形，这是该范围算法的表现，后续版本也可能调整其计算方式。 | `Sight`、`CellSpread` | 建议由战争迷雾所有者统一计算视野覆盖，SDK暴露范围并保留性能边界。 | [Sight](https://ares-developers.github.io/Ares-docs/new/sight.html) |
| AR287 隐形与飞行母舰的子机 | 子机母体隐形不再导致所有子机自毁，并初步支持气球悬停式飞行母舰：非导弹子机从空中母体起飞，也能返回停靠。母体移动时仍可能出现错误降落格，子机飞行高度宜高于母体；移动音效等表现仍有已知不足。 | `BalloonHover`、`FlightLevel` | 建议由子机关系所有者协调母体位置与回收航线，隐形状态不应销毁有效关系。 | [Spawners](https://ares-developers.github.io/Ares-docs/new/spawners.html) |
| AR288 动画粒子生成半径 | 动画生成的粒子可在最小与最大半径之间分散，而非全部挤在动画中心；圆周按粒子数量分段，每段生成一个，且保留动画离地高度。最大半径不得小于最小值，负最小值会取随机距离的绝对值，从而提高靠近中心的概率。 | `SpawnsParticle`、`NumParticles`、`SpawnsParticle.RangeMinimum`、`SpawnsParticle.RangeMaximum` | 建议由动画粒子所有者统一使用同步随机和坐标规则，生成后交给粒子系统。 | [Particle Spawning](https://ares-developers.github.io/Ares-docs/new/spawnsparticle.html) |
| AR289 侦测警报重复间隔 | 隐形与地下单位被侦测的EVA警报，可分别设置按分钟计算的重复间隔，避免每次探测都立即播报。两类警报独立于普通缺电和资金不足的间隔选项；默认零间隔，需显式配置才能节制重复提示。 | `StealthSpeakDelay`、`SubterraneanSpeakDelay`、`SpeakDelay` | 建议由EVA提示所有者维护各警报冷却，侦测所有者仅提交发现事件。 | [Speak Delays](https://ares-developers.github.io/Ares-docs/new/speakdelays.html) |
| AR323 载具与建筑幸存者 | 被摧毁载具可生成驾驶员并让乘员逃生，按载具军衔分别设定机会；类型和阵营还能决定幸存者兵种、技师与工程师概率。空中逃生及周边落点必须可用，失败者通常记作击杀；驾驶员以半血继承经验，乘员保留原状态。 | `Survivor.Side#`、`Crew.TechnicianChance`、`Survivor.Pilots`、`Survivor.RookiePilotChance`、`Survivor.ElitePilotChance`、`Survivor.RookiePassengerChance`、`Survivor.ElitePassengerChance`、`Crew.EngineerChance` | 建议由销毁与乘员所有者统一处理逃生抽取、落点和击杀归属，再生成幸存者。 | [Survivors](https://ares-developers.github.io/Ares-docs/new/survivors.html) |
| AR324 小队集体还击 | 空闲小队成员受攻击且未立即死亡时，可让整队向攻击者还击，恢复早期游戏的协防行为。只提供全局开关，不支持逐小队定制；排除飞机、视为飞机的单位和寄生物，第二个攻击者出现时不会重置成员原有目标。 | `TeamRetaliate`、`ConsideredAircraft` | 建议由小队任务所有者接收受击事件，复用目标资格且保留已有攻击任务。 | [Team Retaliate](https://ares-developers.github.io/Ares-docs/new/teamretaliate.html) |
| AR332 地图触发动作 | 新增地图动作可开关非人类阵营的火风暴、永久增减辅助电力、击杀绑定单位驾驶员、切换EVA和修改对象招募组。火风暴强制开启不消耗充能但会因缺电关闭；驾驶员操作仍遵守保护与铁幕，阵营参数决定后续归属。 | 无独立键或见原文 | 建议由触发调度所有者解析编号并发命令，实际状态变更归对应机制所有者。 | [Trigger Actions](https://ares-developers.github.io/Ares-docs/new/triggeractions.html) |
| AR333 地图触发事件 | 新增事件覆盖EMP、探照灯、驾驶员、绑架、超级武器、逆向工程、类型数量和保活状态。事件区分瞬时或持续条件及是否记忆，组合触发时不可混淆；多个瞬时非记忆事件无法同时满足，特定阵营事件通常先于通用事件。 | `ReversedAs`、`KeepAlive` | 建议由事件所有者集中定义顺序与持久语义，领域机制仅提交发生事实。 | [Trigger Events](https://ares-developers.github.io/Ares-docs/new/triggerevents.html) |
| AR334 炮塔独立转速 | 单位炮塔可设置独立于车体转向速度的转速，便于让重型炮塔转得更慢或让快速跟踪炮塔转得更快。合法取值为零到一百二十七，未设置时继续使用原有整体转速；此项不改变武器射速或单位移动速度。 | `TurretROT`、`ROT` | 建议让朝向所有者区分车体与炮塔转向参数，瞄准和绘制复用其结果。 | [Turret ROT](https://ares-developers.github.io/Ares-docs/new/turretrot.html) |
| AR335 单位类型转换 | 单位可在升级、简易部署、水陆过渡或小队脚本中切换同一对象类别的类型，并更新生命、弹药及相关能力。建筑不支持，转换后仍须容纳于当前位置和容器；不能凭转换新增特殊武器内部状态，脚本即时转换也不支持更换运动方式。 | `Promote.VeteranType`、`Promote.EliteType`、`Promote.VeteranExperience`、`Promote.EliteExperience`、`Convert.Deploy`、`Convert.Water`、`Convert.Land`、`Convert.Script` | 建议由类型转换所有者统一迁移状态与刷新依赖，复用经验、部署和乘员资格。 | [Type Conversion](https://ares-developers.github.io/Ares-docs/new/typeconversion.html) |
| AR336 同类选择分组 | 不同单位类型可共享选择组名，便于把各阵营的犬类、工程师或混合类别对象视为同类选择。组名最长三十字符，默认是类型编号；部署与收起形态可自动归组，也能全局关闭该行为，命名时要避免无意吸收同编号类型。 | `GroupAs`、`TypeSelectUseDeploy`、`DeploysInto`、`UndeploysInto` | 建议由选择所有者解析显式分组及部署关联，保持选组逻辑与单位类型独立。 | [Type Selection Groups](https://ares-developers.github.io/Ares-docs/new/typeselect.html) |
| AR337 新增老兵与精英能力 | 升级能力增加EMP免疫、辐射免疫、驾驶员保护、不可抹除、毒素免疫及两类心灵免疫，可在获得相应军衔后生效。不同能力对应不同弹头机制；心控免疫仍可能被心灵控制器覆盖，不能把所有精神伤害视为同一种效果。 | `ImmuneToRadiation`、`Warpable`、`ImmuneToPoison`、`ImmuneToPsionicWeapons`、`ImmuneToPsionics` | 建议由经验所有者授予能力标记，伤害和特殊效果入口统一查询对应免疫。 | [Veteran and Elite Abilities](https://ares-developers.github.io/Ares-docs/new/veteranabilities.html) |

## 隐形与探测

本节 6 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR196 离地单位的隐形高度 | 主动隐形单位不再因为悬空而无法重新隐形，也不受隐形发生器的高度门槛限制。只有依赖发生器被动隐形的单位需要低于指定高度，默认使用悬浮高度，因此可正常覆盖贴近地面的悬浮车辆。 | `CloakHeight`、`HoverHeight`、`Cloakable` | 建议由隐形所有者区分主动与区域来源，再统一读取单位离地高度。 | [Cloaking and Units Above Ground](https://ares-developers.github.io/Ares-docs/new/cloak/cloakheight.html) |
| AR197 隐形与显形声音 | 将进入隐形和解除隐形的提示音分开，并允许每个单位或建筑覆盖全局设置。主动隐形与隐形发生器赋予的隐形共用这些类型选项，潜水和浮出同样适用；未覆盖的条目仍按全局声音回退。 | `CloakSound`、`DecloakSound` | 建议在隐形状态转换时发出表现事件，由音频层处理类型与全局回退。 | [Cloak Sounds](https://ares-developers.github.io/Ares-docs/new/cloak/cloaksound.html) |
| AR198 按类型设置隐形阶段 | 每种单位或建筑可以单独指定进入或解除隐形所经历的阶段数量，替代所有对象共享一个全局值的限制。该设置调整隐形过渡过程，不负责授予隐形能力；未配置时仍沿用全局隐形阶段数。 | `Cloakable.Stages`、`CloakingStages` | 建议将阶段参数交给隐形状态所有者，表现层按其进度绘制过渡。 | [Cloaking Stages](https://ares-developers.github.io/Ares-docs/new/cloak/cloakstages.html) |
| AR199 按状态限制隐形 | 可让单位仅静止时主动隐形，让部署步兵仅展开后隐形，或令建筑缺电时显形；还能彻底禁止某类型接受任何隐形来源。各条件对主动与发生器隐形的覆盖不同，静止限制不支持气球悬停，也不约束升级所得隐形能力。 | `CloakStop`、`Cloakable.Deployed`、`Cloakable.Powered`、`Cloakable.Allowed`、`Cloakable`、`Deployer`、`BalloonHover` | 建议让隐形所有者按能力来源与单位状态集中计算资格，避免重复状态。 | [Cloak Depending On State](https://ares-developers.github.io/Ares-docs/new/cloak/cloakstates.html) |
| AR200 失能与寄生物的隐形交互 | 依靠供电逻辑被停用或驾驶员死亡的单位会自动显形并失去主动隐形能力，但仍可接受友军隐形发生器的遮蔽。寄生物不再因宿主隐形而被弹出，而是留在宿主体内继续造成伤害。 | `PoweredBy`、`PoweredUnit` | 建议由隐形所有者订阅失能状态，寄生关系仍由寄生生命周期负责维护。 | [General Changes and Updates](https://ares-developers.github.io/Ares-docs/new/cloak/general.html) |
| AR202 传感阵列 | 传感阵列建筑以传感视野探测敌方隐形对象和地下移动，并用相同范围绘制指示圈；摧毁、供电、转手和超时空交互得到修正。可关闭某类型被探测时的EVA和雷达警告，但不会阻止其显形，新增语音需另行配置。 | `SensorArray`、`SensorsSight`、`SensorArray.Warn` | 建议由侦测所有者维护覆盖与可见性，提示事件交给雷达和语音层。 | [Sensor Arrays](https://ares-developers.github.io/Ares-docs/new/cloak/sensorarray.html) |

## 武器

本节 12 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR359 武器逐次弹药成本 | 每种武器可消耗不同数量弹药，也可零消耗；不足成本时等待装填，零成本射击不重置装填。官方不支持飞机，自动换武器另见NoAmmoWeapon。 | `Ammo` | 现有外置示例用 WeaponResource 返回策略；扣弹和装填继续由引擎负责。 | [Ammo](https://ares-developers.github.io/Ares-docs/new/weapons/ammo.html) |
| AR360 按方向选择开火动画 | 武器 Anim 可使用16或32方向组；不足8项时只使用第一项，不能把列表数量任意视为方向数量。 | `Anim` | 在开火呈现事件中携带朝向，统一选帧，勿改变弹体生成时序。 | [Anim for Facings](https://ares-developers.github.io/Ares-docs/new/weapons/anim.html) |
| AR361 武器攻击与超距鼠标 | 为单选单位设置攻击和超距鼠标；多选及无法攻击的目标不保证使用该鼠标，部署后不能移动也有原动作优先级。 | `Cursor.Attack`、`Cursor.AttackOutOfRange` | 扩展输入动作解析及鼠标资源映射，不放进逐对象 Tick。 | [Cursors](https://ares-developers.github.io/Ares-docs/new/weapons/cursors.html) |
| AR362 特殊武器是否造成普通伤害 | 让声波、火焰粒子等特殊武器选择是否额外造成普通伤害，也可关闭普通武器伤害；关闭时可能同时影响依赖非零伤害的效果。 | `ApplyDamage` | 在原开火和伤害入口提供策略，分别验证普通伤害与附加效果门槛。 | [Ordinary Damage for IsSonic and UseFireParticles](https://ares-developers.github.io/Ares-docs/new/weapons/damage.html) |
| AR363 电弧颜色与终点粒子 | 电弧三层颜色可独立配置，并可指定或关闭命中位置的火花粒子系统；缺失时仍使用原调色板与全局默认。 | `Bolt.Color3`、`Bolt.ParticleSystem` | 在电弧呈现描述和粒子生成接口扩展，保持模拟与绘制分离。 | [Electric Bolts](https://ares-developers.github.io/Ares-docs/new/weapons/electricbolts.html) |
| AR365 自定义伊文炸弹和死亡炸弹 | 逐武器定制定时或死亡触发炸弹、手动引爆、出售时引爆、工程师拆除及桥梁行为，并可更换炸弹图标。 | `IvanBomb.Delay`、`IvanBomb.Warhead`、`IvanBomb.Damage`、`IvanBomb.AttachSound`、`IvanBomb.TickingSound`、`IvanBomb.DeathBomb`、`IvanBomb.DeathBombOnAllies`、`IvanBomb.CanDetonateTimeBomb` | 需要独立附着炸弹生命周期、拥有者与引爆命令，不可只在普通 Damage 内扣血。 | [Ivan Bombs](https://ares-developers.github.io/Ares-docs/new/weapons/ivanbombs.html) |
| AR366 激光宽度 | 可配置激光线宽；官方注明该实现仅对 IsHouseColor=yes 生效，且存在质量及兼容缺陷，不能当作所有激光的通用效果。 | `LaserThickness` | 由渲染层处理宽度及玩家颜色；兼容模式与新的视觉增强应分开定义。 | [LaserThickness](https://ares-developers.github.io/Ares-docs/new/weapons/laser.html) |
| AR367 武器指定抛射体最大航程 | 给 Ranged=yes 的抛射体指定耗尽燃料前的最大距离；该值不是武器可选目标的 Range。 | `ProjectileRange` | 复用弹体距离预算与中途爆炸入口，区分射程和实际飞行航程。 | [ProjectileRange](https://ares-developers.github.io/Ares-docs/new/weapons/projectilerange.html) |
| AR368 辐射与时空光束外观 | 为辐射光束和辐射喷发配置颜色、玩家色、持续帧数及振幅；这是光束显示扩展，不等于新增辐射伤害类型。 | `Beam.Color`、`Beam.IsHouseColor`、`Beam.Duration`、`Beam.Amplitude` | 扩展光束呈现数据，数值单位以官方含义和原绘制行为确认。 | [Radiation Beams](https://ares-developers.github.io/Ares-docs/new/weapons/radiation.html) |
| AR369 独立轨道炮粒子束 | 让新的轨道炮射击不必等待上一束粒子消失；不能与 IsRailgun=yes 混用，大量粒子可能增加开销。 | `IsDetachedRailgun` | 将开火许可与呈现实例寿命解耦，同时保证粒子伤害的权威归属。 | [Detached Railgun Beams](https://ares-developers.github.io/Ares-docs/new/weapons/railgun.html) |
| AR370 波束效果 颜色 方向与沿途伤害 | 扩展激光式波束、颜色和强度混合、按目标类别反转绘制方向，并让不同波束支持沿途伤害；外观与伤害需分别实现。 | `Wave.IsLaser`、`Wave.IsBigLaser`、`Wave.Color`、`Wave.Intensity`、`Wave.IsHouseColor`、`Wave.ReverseAgainstVehicles`、`Wave.ReverseAgainstBuildings`、`Wave.ReverseAgainstInfantry` | 渲染处理混色和方向，模拟沿途收集目标并走统一伤害流程。 | [Waves](https://ares-developers.github.io/Ares-docs/new/weapons/waves.html) |
| AR371 独立武器类型注册表 | 通过 WeaponTypes 注册需要解析的武器，不再必须给虚拟单位挂武器才能加载，可供子武器及模式扩展引用。 | `WeaponTypes` | 在配置目录分配阶段注册稳定武器身份，再建立所有引用。 | [[WeaponTypes] Section](https://ares-developers.github.io/Ares-docs/new/weapons/weapontypes.html) |

## 抛射体

本节 5 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR263 旋转抛射体的逐方向动画 | 让 Rotates=yes 的抛射体在每个方向使用一组动画帧，通过 AnimLength 和 AnimRate 播放；不能把 AnimRate 设为零。 | `AnimLength` | 在抛射体美术解码和帧选择层实现，方向与播放时间不改变弹道。 | [Animations and Rotates=yes](https://ares-developers.github.io/Ares-docs/new/projectiles/animations.html) |
| AR264 抛射体附着粒子系统 | 抛射体飞行途中可携带粒子拖尾；官方仅支持 BehavesLike=smoke 的粒子系统，不代表任意粒子类型都能附着。 | `AttachedSystem` | 把附着、随动及销毁交给统一粒子实例管理，插件只提供描述。 | [AttachedSystem Trailers](https://ares-developers.github.io/Ares-docs/new/projectiles/attachedsystem.html) |
| AR265 每种抛射体的弹道散布 | 把全局散布拆为每种抛射体的最小、最大距离；要求 Inaccurate=yes 与 Arcing=yes。某些 FlakScatter/Inviso 附加散布仍有全局规则。 | `BallisticScatter.Min`、`BallisticScatter.Max` | 在既有弹道采样处扩展参数，固定同一随机流的调用数和顺序。 | [BallisticScatter](https://ares-developers.github.io/Ares-docs/new/projectiles/ballisticscatter.html) |
| AR267 抛射体燃料与中途引爆 | 恢复飞行距离耗尽后自行爆炸的机制。抛射体设置 Ranged，武器用 ProjectileRange 指定可飞行距离，二者配套使用。 | `Ranged` | 复用抛射体生命周期、距离和统一爆炸入口，不让插件绕过清理。 | [Ranged](https://ares-developers.github.io/Ares-docs/new/projectiles/ranged.html) |
| AR268 导弹分裂与空爆扩展 | 支持分裂、重选目标、自身排除和空爆覆盖范围。AirburstSpread 不与 Splits 同用，且官方指出分裂武器 Speed 低于19会异常。 | `Splits`、`RetargetAccuracy`、`RetargetSelf`、`AirburstSpread`、`AroundTarget` | 补充受校验的子弹生成与重选目标能力，验证目标顺序及随机数。 | [Splits and Airburst](https://ares-developers.github.io/Ares-docs/new/projectiles/splits.html) |

## 弹头与伤害

本节 20 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR338 狂暴射速与独立免疫 | 可全局或逐单位修改狂暴状态的重装倍率，并把狂暴免疫与精神控制免疫分开配置；倍率越小代表射击越快。 | `BerserkROFMultiplier`、`Berserk.ROFMultiplier`、`ImmuneToBerserk` | 接入已有状态免疫与重装时间计算，区分未设置和显式布尔值。 | [Berserk](https://ares-developers.github.io/Ares-docs/new/warheads/berserk.html) |
| AR339 范围爆炸与单对象命中次数 | 扩大 CellSpread 可用范围，并限制同一次爆炸对多格建筑的重复命中次数。扩大半径会明显增加遍历成本。 | `CellSpread.MaxAffect` | 先复用区域目标收集器，再稳定去重或选最近命中；避免另写伤害遍历。 | [CellSpread Enhancements](https://ares-developers.github.io/Ares-docs/new/warheads/cellspread.html) |
| AR340 低生命目标处决 | 把寄生攻击的斩杀规则扩展到普通弹头，可按开火者等级配置目标生命阈值和逐次命中的处决概率。 | `Culling.EliteBelowHealth`、`Culling.EliteChance` | 需要终结伤害及击杀归属接口；随机判定由引擎的确定性随机流执行。 | [Culling](https://ares-developers.github.io/Ares-docs/new/warheads/culling.html) |
| AR341 地面爆炸影响空中对象 | 按爆炸点离地高度决定是否收集空中目标，并可选择以桥面高度判断。设负阈值可让地面爆炸也考虑空中目标。 | `DamageAirThreshold`、`DamageAirConsiderBridges` | 先扩展区域目标筛选上下文，保留桥面高度坐标，随后走统一接收伤害。 | [Damage Airborne Units](https://ares-developers.github.io/Ares-docs/new/warheads/damageair.html) |
| AR342 抑制目标死亡武器 | 弹头造成击杀时，可按单位类型列表或步兵、车辆类别抑制死亡武器；仅造成过伤害而非最终击杀不生效。 | `DeathWeapon.Suppress`、`DeathWeapon.SuppressVehicles`、`DeathWeapon.SuppressInfantry` | 在死亡武器的唯一触发点接入策略，携带最终击杀弹头并验证连锁爆炸。 | [Suppress Death Weapons](https://ares-developers.github.io/Ares-docs/new/warheads/deathweapon.html) |
| AR343 覆盖死亡音效和语音 | 弹头击杀对象时替换或静音其死亡声音；原对象没有相应声音时，不会仅因覆盖设置而凭空播放。 | `DieSound.Override`、`VoiceDie.Override` | 从击杀结果向音频事件层传递覆盖值，音频不成为模拟状态权威。 | [Override Die Sounds and Voices](https://ares-developers.github.io/Ares-docs/new/warheads/diesound.html) |
| AR344 暂时禁用目标武器 | 命中后让对象在指定帧数内不能开火，但装填、寻敌、死亡武器和C4等特殊能力并不因此全部停用。 | `DisableWeapons.Duration` | 增加受校验的开火禁用状态，在通用 CanFire 处消费，不能简单暂停整个AI。 | [Disable Weapons](https://ares-developers.github.io/Ares-docs/new/warheads/disableweapons.html) |
| AR345 命中闪烁效果 | 可让被命中的对象按指定帧数闪烁；是否必须造成实际伤害，还受通用弹头效果门槛控制。 | `Flash.Duration` | 输出带持续时间的呈现事件或投影，不能用显示计时器驱动伤害判定。 | [Flashing](https://ares-developers.github.io/Ares-docs/new/warheads/flash.html) |
| AR346 弹头关系门槛与非伤害效果 | 分别控制敌方、拥有者、零伤害、Verses门槛、采矿车警告与受击散开。可否选中目标与命中后是否影响目标是两道判断。 | `AffectsEnemies`、`AffectsOwner`、`EffectsRequireDamage`、`EffectsRequireVerses`、`AllowZeroDamage`、`Malicious`、`PreventScatter` | 形成共享弹头准入决策，供EMP等效果复用；避免每个效果复制阵营判断。 | [General Settings](https://ares-developers.github.io/Ares-docs/new/warheads/general.html) |
| AR348 部署步兵倍率与自定义死亡动画 | 对已部署步兵使用专门伤害倍率，并按动画ID选择步兵死亡、变异效果；部署姿态不等于卧倒姿态。 | `Damage.Deployed`、`InfDeathAnim` | 分别接伤害倍率与死亡动画解析点，变异还需处理新对象及拥有者。 | [Infantry-related Settings](https://ares-developers.github.io/Ares-docs/new/warheads/infantry.html) |
| AR349 离子炮弹头与冲击视觉 | 让任意弹头触发离子炮效果，独立指定光束、爆炸、伤害和涟漪，并可设置命中桥梁的绝对摧毁行为；涟漪本身是视觉。 | `IonCannon`、`IonCannon.Blast`、`IonCannon.Beam`、`IonCannon.Warhead`、`IonCannon.Damage`、`IonCannon.Rock`、`IonCannon.Ripple`、`BridgeAbsoluteDestroyer` | 拆成世界伤害、桥梁破坏与呈现三类命令，统一归属和生命周期。 | [Ion Cannon](https://ares-developers.github.io/Ares-docs/new/warheads/ioncannon.html) |
| AR350 弹头施加和削减铁幕 | 按帧数施加、削减或叠加铁幕，可设持续上限及对象倍率。同次普通伤害先处理；步兵和有机对象有特殊死亡语义。 | `IronCurtain.Duration`、`IronCurtain.Cap`、`IronCurtain.Modifier`、`IronCurtainFlash`、`IronCurtain.Flash` | 复用现有无敌状态所有者，明确伤害与赋予效果的顺序并测试有机目标。 | [Iron Curtain](https://ares-developers.github.io/Ares-docs/new/warheads/ironcurtain.html) |
| AR351 维修与治疗的类型限制解除 | 允许步兵、车辆按弹头倍率治疗或维修不同类别目标，并支持跟随保护；车辆仍不能维修停在停机坪上的飞机。 | 无独立键或见原文 | 让目标合法性、鼠标动作与负伤害接收一致，不能只改变最终治疗量。 | [Mechanical and Organic Warheads](https://ares-developers.github.io/Ares-docs/new/warheads/mechanical.html) |
| AR352 永久精神控制 | 在精神控制弹头上开启永久控制，按支配者式规则重新取得已有控制关系的目标；可作用于弹头能选中的建筑。 | `MindControl.Permanent` | 通过统一所有权与控制关系管理转移对象，覆盖原控制者死亡和建筑功能。 | [Permanent Mind-Control](https://ares-developers.github.io/Ares-docs/new/warheads/mindcontrol.html) |
| AR353 核爆全图闪光 | 解除只有NUKE弹头才能触发全图闪光的限制，可逐弹头配置持续时间或关闭；多个闪光叠加有官方已知限制。 | `NukeFlash.Duration` | 作为全局呈现效果管理，不能把画面闪光当作伤害或战争迷雾状态。 | [Nuke Flash](https://ares-developers.github.io/Ares-docs/new/warheads/nukeflash.html) |
| AR354 爆炸前置动画与延迟伤害 | 先播放不可循环的前置动画，结束后才爆炸和伤害；可随动画移动更新弹体及最终目标格。 | `PreImpactAnim`、`PreImpactAnim.Moves` | 需要延迟爆炸生命周期与动画完成事件，读档保留待处理爆炸。 | [PreImpactAnim](https://ares-developers.github.io/Ares-docs/new/warheads/preimpactanim.html) |
| AR355 按最大或当前生命计算伤害 | 按目标类别计算比例伤害；正比例基于最大生命，负比例基于当前生命，治疗方向另由武器Damage符号决定；若干特殊弹头不支持。 | `RelativeDamage`、`RelativeDamage.Terrain` | 在原伤害入口提供数值策略，继续经过装甲、等级与Verses，不直接写血量。 | [MAD Tank Style Relative Damage](https://ares-developers.github.io/Ares-docs/new/warheads/relativedamage.html) |
| AR356 声纳暴露与禁隐形 | 让目标在指定帧数内不能使用主动隐形；零伤害或免疫倍率是否允许施加，取决于通用弹头效果设置。 | `Sonar.Duration` | 在隐形状态所有者中记录禁用到期帧，并从呈现层读取可见性结果。 | [Sonar / Decloak](https://ares-developers.github.io/Ares-docs/new/warheads/sonar.html) |
| AR357 时空抹除动画与生命权重 | 可替换抹除动画，并让目标当前生命按可配置权重影响剩余抹除量；权重取0到1，不能当作直接伤害倍率。 | `Temporal.WarpAway`、`Temporal.HealthFactor` | 复用时空锁定、剩余抹除量及释放链，避免新增一套死亡状态。 | [Temporal](https://ares-developers.github.io/Ares-docs/new/warheads/temporal.html) |
| AR358 抑制单位损失播报 | 弹头造成实际击杀时可抑制拥有者的单位损失EVA；只造成伤害而未击杀不会静音后续其他来源的击杀。 | `UnitLost.Suppress` | 在最终击杀事件中传递播报策略，不修改全局EVA开关。 | [Suppress Unit Lost Message](https://ares-developers.github.io/Ares-docs/new/warheads/unitlost.html) |

## 建筑与基地

本节 27 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR162 军事学院与初始经验 | 学院建筑可按步兵、飞行器、车辆和建筑分别授予生产时的初始经验，并用类型白名单与排除表缩小对象范围。多个学院只取最高奖励，不叠加；仅可训练对象有效，也不会降低其他来源已经给予的更高军衔。 | `Academy.InfantryVeterancy`、`Academy.AircraftVeterancy`、`Academy.VehicleVeterancy`、`Academy.BuildingVeterancy`、`Academy.Types`、`Academy.Ignore` | 建议由引擎生产与经验所有者结算奖励，SDK需类型筛选和初始经验能力。 | [Academies](https://ares-developers.github.io/Ares-docs/new/buildings/academy.html) |
| AR163 突击占领等级 | 为突击步兵和目标建筑增加可比较的突击等级，使建筑能按等级抵挡特定突击兵。只有启用突击能力的步兵才参与此逻辑，步兵等级必须不低于建筑等级；建筑等级为负数时，无论步兵等级多高都不能突击。 | `Assaulter.Level`、`Assaulter` | 建议由引擎驻军与突击判定所有者执行比较，SDK需读取类型等级配置。 | [Assaulter](https://ares-developers.github.io/Ares-docs/new/buildings/assaulter.html) |
| AR164 AI基地范围与内圈选址 | 可分别决定建筑是否扩展AI的基地建造空间，以及AI是否优先把它放在基地中心附近，便于保护关键设施或覆盖范围效果。内圈选址只是偏好，建筑占地过大或中心已拥挤时，AI仍会退回外围寻找可建位置。 | `AIBaseNormal`、`AIInnerBase`、`BaseNormal`、`CloakGenerator` | 建议由引擎AI基地规划所有者管理范围与选址，SDK需声明建筑规划偏好。 | [Extending Base Space of the AI](https://ares-developers.github.io/Ares-docs/new/buildings/basebuilding.html) |
| AR165 AI重复建造数量 | 允许AI对普通建筑规划多个实例，并按困难、中等、简单分别配置基础数量与随机追加上限。基础数量至少为一，追加数量不能为负；额外建筑会与后续规划随机混排，并不保证连续建完同一种建筑。 | `AIBuildCounts`、`AIExtraCounts` | 建议由引擎AI建造计划所有者统一抽样，SDK需按难度配置数量及确定性随机支持。 | [Build More Buildings of a Type](https://ares-developers.github.io/Ares-docs/new/buildings/buildcount.html) |
| AR166 建筑放置提示音 | 把建筑落地时的提示音从全局设置拆到类型配置，允许不同建筑拥有各自的放置音效。声音面向建筑所有者播放，未指定时沿用全局建筑放置音；适合让不同设施的建造反馈具有区别，而无需改动全局声音。 | `SlamSound`、`BuildingSlam` | 建议由引擎建筑放置事件触发音频表现，SDK需类型音效引用与本地播放能力。 | [Building Slam](https://ares-developers.github.io/Ares-docs/new/buildings/buildingslam.html) |
| AR167 建成与出售动画时长 | 建筑可分别指定建成动画和出售动画的播放时长，默认出售时长继承建成时长。动画帧数多于原定时长时会自动延长，避免跳帧异常；部分建筑功能仍在落地时立即生效，不能把动画结束等同于功能开放时刻。 | `BuildupTime`、`SellTime` | 建议由引擎建筑生命周期所有者维护计时，表现层按确定的时长播放建造资源。 | [Buildup and Sell Time](https://ares-developers.github.io/Ares-docs/new/buildings/builduptime.html) |
| AR168 自定义建筑占地 | 建筑地基不再局限于原版预设，可设计任意大小与形状，但须同时提供占地轮廓。战车工厂的出厂路径仍沿用原版硬编码，优先出口固定取轮廓第十一格，因此修改工厂地基容易造成出厂位置不匹配。 | 无独立键或见原文 | 建议由引擎占地与通行所有者接收地基数据，SDK需形状资源及出口约束校验。 | [Custom Building Foundations](https://ares-developers.github.io/Ares-docs/new/buildings/custombuildingfoundations.html) |
| AR169 敌方维修扳手可见性 | 新增全局开关，控制非盟友能否看到建筑正在点击维修时的扳手动画，可减少敌人从维修表现获得的信息。观察者始终可见；受到超时空武器攻击的建筑，以及处于隐形状态的敌方建筑，本来就不显示该动画。 | `EnemyWrench`、`EnemyHealth` | 建议由UI依据阵营关系和建筑状态过滤维修覆盖层，SDK需表现可见性策略。 | [Enemy Wrench](https://ares-developers.github.io/Ares-docs/new/buildings/enemywrench.html) |
| AR170 专用工厂与克隆生产 | 支持把单位限定到指定工厂，并让工厂只接收明确列出的产品；克隆还可指定设施、替代类型和禁用对象。车辆克隆需匹配海陆属性，步兵与车辆不能跨类别替换；指定克隆设施会覆盖通用克隆机制，出口堵塞不退款。 | `BuiltAt`、`Factory.ExplicitOnly`、`CloningFacility`、`Cloneable`、`ClonedAs`、`ClonedAt`、`Naval`、`Cloning` | 建议由引擎生产与生成所有者统一处理出厂和克隆，SDK需通用对象生成及设施查询能力。 | [Factories and Cloning](https://ares-developers.github.io/Ares-docs/new/buildings/factoriesandcloning.html) |
| AR171 伪装建筑标记 | 可把建筑标为仿冒设施，使拥有者与盟友选中时看到假建筑提示；敌人若通过揭示生产情报的间谍效果渗透，也能识破身份。该机制用于外观近似真建筑、较脆弱且无实际功能的诱饵，并可配合敌方名称设置隐藏真实名称。 | `Fake`、`EnemyUIName` | 建议由引擎建筑类型保存真假标记，UI结合关系与渗透情报选择名称和提示。 | [Fakes](https://ares-developers.github.io/Ares-docs/new/buildings/fakes.html) |
| AR172 建筑力场护盾时长系数 | 允许每种建筑调整所受力场护盾的持续时长，以原有秒数乘以类型系数，默认不增不减。系数为零或更低的建筑不能成为力场护盾发射目标；该设置适合区分关键设施与普通设施的保护强度，而非改变范围。 | `ForceShield.Modifier` | 建议由引擎保护效果所有者计算持续时间，SDK需目标资格与效果时长修正规则。 | [Force Shield](https://ares-developers.github.io/Ares-docs/new/buildings/forceshield.html) |
| AR173 闸门开关音效 | 闸门允许盟友通过，同时对敌人充当墙体，本页把开门和关门音效拆成建筑类型选项。名称沿用早期下沉式闸门约定：向下表示开启，向上表示关闭；未指定时分别继承全局闸门声音，避免按字面把开关配反。 | `GateDownSound`、`GateUpSound`、`GateDown`、`GateUp` | 建议由引擎闸门状态转换触发音频事件，SDK需建筑音效引用与通行状态联动。 | [Gates](https://ares-developers.github.io/Ares-docs/new/buildings/gates.html) |
| AR175 避雷建筑与雷击倍率 | 建筑可充当避雷设施，吸引附近随机生成的雷云，并对自身受到的雷击伤害应用独立倍率。吸引要求它是该随机雷云附近最近的对象，云会改为在建筑上方生成；倍率只对启用避雷标记的建筑生效，默认不改变伤害。 | `LightningRod`、`LightningRod.Modifier` | 建议由引擎天气与伤害所有者共同处理选点和倍率，SDK需天气目标筛选能力。 | [Lightning Rods](https://ares-developers.github.io/Ares-docs/new/buildings/lightningrods.html) |
| AR176 建筑参与框选 | 允许按类型控制建筑是否像车辆一样加入批量框选，可让大型可收起建筑一起选中，也可排除本来会被当作车辆的一格建筑。原版只自动处理一格且可收起的建筑；新开关决定框选资格，不会自行赋予收起能力。 | `MassSelectable`、`UndeploysInto` | 建议由UI选择系统读取建筑框选资格，SDK需声明选择属性并保留引擎对象权限。 | [Mass-Selection](https://ares-developers.github.io/Ares-docs/new/buildings/massselect.html) |
| AR177 单位可穿行建筑 | 新增独立标记，让步兵和车辆能进入或站在建筑占用的格子上，适合制作地雷等需要透明占地的设施。该方案避免借用激光围墙或火墙标记造成免伤、特殊死亡或火墙联动；开关默认关闭，仍须另配建筑实际功能。 | `IsPassable` | 建议由引擎地图占用与寻路所有者统一解释可通行性，SDK需建筑占地规则配置。 | [Passable Structures](https://ares-developers.github.io/Ares-docs/new/buildings/passablestructures.html) |
| AR178 光棱支援网络 | 重建光棱塔支援逻辑，支持多级传递、混合塔型、盟友供能及链长、分支和总规模限制，并可用加算与倍率调整主炮伤害。支援光束只读取武器的部分属性；塔不应收起，充能中收起可能崩溃，支援配置也不直接等于常规开火。 | `PrismForwarding`、`PrismForwarding.Targets`、`PrismForwarding.MaxFeeds`、`PrismForwarding.MaxChainLength`、`PrismForwarding.MaxNetworkSize`、`PrismForwarding.SupportModifier`、`PrismForwarding.DamageAdd`、`PrismForwarding.SupportWeapon` | 建议由引擎武器与支援网络所有者管理拓扑、充能和伤害，SDK需网络规则及光束资源能力。 | [Prism Forwarding](https://ares-developers.github.io/Ares-docs/new/buildings/prismforwarding.html) |
| AR179 周期收入与金额浮字 | 建筑附加组件也可独立计时产生收入，启动金额支持负数，资金不足仍不阻止占领或放置；被中立被动国家占领时不发启动收入。主建筑可显示汇总金额浮字，组件沿用主建筑开关；颜色与带正负号的格式仍为固定表现。 | `ProduceCashDelay`、`ProduceCashAmount`、`ProduceCashStartup`、`ProduceCashDisplay`、`DisplayCreditsDelay` | 建议由引擎经济所有者结算建筑及组件收入，SDK需受控经济操作与金额提示资源能力。 | [Oil Derricks ( ProduceCash )](https://ares-developers.github.io/Ares-docs/new/buildings/producecash.html) |
| AR180 矿厂卸货位置与朝向 | 可在矿厂图像配置中指定矿车卸货停靠的格子偏移与转向，使自定义矿厂美术能配合实际卸货动作。配置属于建筑图像而非规则类型，停靠格必须可通行；默认偏移为三、一，默认朝向为三十二方向制的八，即朝东。 | `DockUnloadCell`、`DockUnloadFacing` | 建议由引擎采集与停靠所有者读取图像锚点，SDK需停靠参数及资源关联能力。 | [Refineries](https://ares-developers.github.io/Ares-docs/new/buildings/refineries.html) |
| AR181 缺钱时暂停维修与工程师限制 | 可让人类玩家缺钱时保留建筑维修模式，维修暂停但扳手仍显示，资金恢复后自动继续，默认行为仍是直接终止维修。另可逐建筑禁止工程师瞬间修满，且不妨碍该建筑继续通过普通点击维修获得修复。 | `RepairStopOnInsufficientFunds`、`EngineerRepairable`、`ClickRepairable`、`Repairable` | 建议由引擎维修与经济所有者维护暂停状态，SDK需维修模式和工程师交互资格规则。 | [Repair and Insufficient Funds](https://ares-developers.github.io/Ares-docs/new/buildings/repair.html) |
| AR182 秘密实验室科技奖励 | 秘密实验室可从建筑、步兵、车辆和飞机清单随机授予科技，只选择当前拥有者尚不能生产的奖励。可在每次有效占领时重抽，并对奖励限定国家；若仅首次抽取，之后易主不会重新校验国家限制，须留意共享结果。 | `SecretLab`、`SecretLab.PossibleBoons`、`SecretLab.GenerateOnCapture`、`SecretLab.RequiredHouses`、`SecretLab.ForbiddenHouses` | 建议由引擎科技解锁与占领所有者执行抽取，SDK需生产资格查询及受控科技授权能力。 | [Secret Labs](https://ares-developers.github.io/Ares-docs/new/buildings/secretlabs.html) |
| AR183 建筑阻挡弹道 | 建筑可按高度与坚固等级拦截弹丸，攻击者据弹丸可破坏等级决定先拆障碍还是换位置。须同时启用弹丸受建筑阻挡；开启的闸门不阻挡，盟友穿透由全局开关控制，非矩形建筑效果有限，部分光束显示仍可能穿过实际受击障碍。 | `SubjectToBuildings`、`SolidLevel`、`SolidHeight`、`AlliedSolidTransparency` | 建议由引擎弹道碰撞与攻击选位所有者共用阻挡判定，SDK需建筑碰撞及友军穿透规则。 | [Solid Buildings](https://ares-developers.github.io/Ares-docs/new/buildings/solidbuildings.html) |
| AR184 可组合间谍渗透效果 | 启用自定义渗透后，可组合雷达情报、停电、偷钱、超武重置或授权、初始军衔、生产侦察及延时爆破等效果。偷取科技扩为零至三十一的索引，能与生产前置要求联动；启用新逻辑的原版科技建筑不再自动满足旧偷科技标记。 | `SpyEffect.Custom`、`SpyEffect.RevealRadar`、`SpyEffect.PowerOutageDuration`、`SpyEffect.StolenMoneyPercentage`、`SpyEffect.SuperWeapon`、`SpyEffect.StolenTechIndex`、`Prerequisite.StolenTechs`、`SpyEffect.SabotageDelay` | 建议由引擎渗透所有者协调经济、科技、超武与情报状态，SDK需受控世界操作及事件能力。 | [Spy Effects](https://ares-developers.github.io/Ares-docs/new/buildings/spyeffects.html) |
| AR185 科技建筑归还中立与占领通知 | 多人游戏可让最初属于被动国家的科技建筑或可驻守建筑，在玩家战败或退出时归还最初拥有者，而非摧毁；单人任务忽略此设置。还可定制科技建筑丢失语音及得失文字，通知要求建筑具备需要工程师的标记。 | `ReturnStructures`、`Returnable`、`LostEvaEvent`、`Message.Capture`、`Message.Lost`、`NeedsEngineer` | 建议由引擎玩家退场与占领所有者决定归属，UI按建筑事件播放配置的提示。 | [Tech Structures Return to Neutral](https://ares-developers.github.io/Ares-docs/new/buildings/techstructures.html) |
| AR186 建筑电源切换 | 恢复键盘电源切换命令、专用光标与断电覆盖动画，默认须全局启用后才可操作，且只能切换符合条件的自有建筑。AI可按智商和检查间隔应对发电不足，尽量保留防御；间谍停电或护盾停电等情况不按此方案处理。 | `TogglePowerAllowed`、`TogglePower`、`TogglePowerDelay`、`Power`、`Powered` | 建议由引擎电力所有者接受切换请求，UI负责光标和覆盖层，SDK需受控电源操作。 | [Toggle Power](https://ares-developers.github.io/Ares-docs/new/buildings/togglepower.html) |
| AR187 低电量建筑衰损 | 可恢复基地缺电时建筑周期掉血，分别设置发电类与耗电类损伤，并允许建筑覆盖伤害量和最低生命比例。伤害使用既定C4弹头，可降至指定阈值甚至摧毁；还能逐国家或单人任务玩家关闭，被动国家默认不受影响。 | `Degrade.Enabled`、`Degrade.Percentage`、`Degrade.AmountNormal`、`Degrade.AmountConsumer`、`Degrade.Amount`、`Degrades`、`DamageDelay`、`C4Warhead` | 建议由引擎电力与伤害所有者按统一时序执行衰损，SDK需周期规则及国家适用配置。 | [Trivial Structure Damage](https://ares-developers.github.io/Ares-docs/new/buildings/trivialstructuredamage.html) |
| AR188 共享乘员隧道网络 | 同一玩家的同类隧道入口共享一份乘员队列，进入后可立即从其他入口离开，容量按单位数而非体积累计。最后入口被毁会杀死内部单位，出售则尝试撤离；入口不能带武器或炮塔，也不能被占领或心控，进入还受体积限制。 | `Passengers`、`MaxSize`、`Tunnel`、`Size`、`SizeLimit`、`EnterTransportSound`、`LeaveTransportSound` | 建议由引擎运输所有者维护共享乘员及入口生命周期，SDK需通用装载、转移与撤离能力。 | [Tunnel Networks](https://ares-developers.github.io/Ares-docs/new/buildings/tunnels.html) |
| AR189 战壕、驻军穿透与废墟修复 | 组合驻军受击穿透、空堡临时夺占、建筑毁坏转废墟与工程师复原，以及相邻同类战壕间转移。各功能可独立使用；复原不消耗工程师，废墟与原建筑必须地基匹配且禁止转换循环，战壕转移要求一格地基，并可限制允许进驻的步兵类型。 | `UC.PassThrough`、`UC.FatalRate`、`UC.DamageMultiplier`、`SubjectToTrenches`、`Bunker.Raidable`、`Rubble.Destroyed`、`Rubble.Intact`、`IsTrench` | 建议由引擎驻军与建筑转换所有者统一维护乘员、归属及地基，SDK需受控转移和类型转换能力。 | [Urban Combat - Trenches](https://ares-developers.github.io/Ares-docs/new/buildings/urbancombattrenches.html) |

## 阵营与国家

本节 7 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR279 国家默认单位与AI电厂 | 国家可覆写阵营默认的空降内容、数量、运输机与降落伞，并指定出生即为老兵的建筑。AI电厂清单也可按国家定制，但至少要有一种电厂始终能满足前置条件；空降车辆必须使用新增的类型与数量字段，旧列表仅识别步兵。 | `AI.PowerPlants`、`ParaDrop.Types`、`ParaDrop.Num`、`ParaDrop.Aircraft`、`Parachute.Anim`、`VeteranBuildings` | 建议由引擎国家规则所有者处理默认值继承，SDK需国家配置、生产奖励及空降参数能力。 | [Country Defaults](https://ares-developers.github.io/Ares-docs/new/sidescountries/defaultcountry.html) |
| AR280 阵营默认单位与基地规则 | 把幸存驾驶员、工程师、技师、初始伪装、防御建筑清单及数量等规则下放到阵营，并提供空降与猎杀者默认类型。幸存者数量依出售或退款金额换算；空降支持步兵和车辆，猎杀者仍须使用符合专门移动规则的车辆类型。 | `Crew`、`Engineer`、`Technician`、`DefaultDisguise`、`SurvivorDivisor`、`AI.BaseDefenses`、`AI.BaseDefenseCounts`、`HunterSeeker` | 建议由引擎阵营规则所有者统一提供默认值，SDK需类型引用、继承规则与基地规划配置。 | [Side Defaults](https://ares-developers.github.io/Ares-docs/new/sidescountries/defaultside.html) |
| AR281 阵营与国家扩展及数量边界 | 开放阵营和国家的增删、排序与定制，阵营声明数量本身不受限制。可玩国家建议最多十六个，总国家最多三十二个，并把可玩项放前面；部分功能超过十六国会失效，嘲讽也只支持十六国，多人标记决定国家是否进入可选与随机池。 | `Multiplay` | 建议由引擎国家注册与大厅UI共同校验容量，SDK需阵营国家定义及选择资格配置。 | [Sides & Countries](https://ares-developers.github.io/Ares-docs/new/sidescountries/index.html) |
| AR282 阵营结算画面与音乐 | 可按阵营替换战役结算背景、过渡、循环动画和调色板，并按是否低于标准时间选择音乐；多人结算另有背景、十条图像栏及胜败音乐。胜败横幅也可定制，至少提供战役与多人胜败四帧，颜色需经过场景调色板映射。 | `CampaignScore.Background`、`CampaignScore.Transition`、`CampaignScore.Animation`、`CampaignScore.Palette`、`CampaignScore.UnderParTheme`、`MultiplayerScore.Background`、`MultiplayerScore.Bars`、`GraphicalText.Image` | 建议由UI结算页面按阵营加载图像、调色板和音乐，SDK需资源引用与结算状态选择能力。 | [Score Screens and Themes](https://ares-developers.github.io/Ares-docs/new/sidescountries/scorescreens.html) |
| AR283 多人初始部队与预展开基地 | 可逐国家指定开局步兵和车辆候选清单，也可完全取消初始部队；开局数量预算可改用固定的平均单位价格。国家还可直接以首个可建造的基地建筑开局，替代基地车，适合固定基地玩法，且不必与之后获得的基地类型一致。 | `StartInMultiplayer.Types`、`StartInMultiplayerUnitCost`、`StartInMultiplayer.WithConst`、`AllowedToStartInMultiplayer`、`BuildConst`、`BaseUnit` | 建议由引擎对局初始化所有者统一生成开局对象，SDK需初始预算、候选池和基地方案配置。 | [Start in Multiplayer](https://ares-developers.github.io/Ares-docs/new/sidescountries/startinmultiplayer.html) |
| AR284 国家选择、载入画面与嘲讽 | 国家可设置列表顺序、人工与AI随机权重、说明文字、旗帜、观察者素材、载入图文音乐及嘲讽路径。隐藏列表项不会自动排除随机选择，须另设零权重；嘲讽文件模式必须包含双波浪号，以替换为零一至零八编号。 | `MenuText.Status`、`ListIndex`、`RandomSelectionWeight`、`AIRandomSelectionWeight`、`File.Flag`、`File.LoadScreen`、`LoadingTheme`、`File.Taunt` | 建议由大厅与载入UI读取国家资源配置，SDK需资源检索、列表排序及确定性权重选择支持。 | [User Interface](https://ares-developers.github.io/Ares-docs/new/sidescountries/uicountry.html) |
| AR285 阵营侧栏、界面与播报员 | 可按阵营选择侧栏资源包及尤里文件命名方式，设置提示文字、消息颜色、对话框图像、载入音乐和EVA播报员。对话框图像与调色板须配套；播报员名称对应语音配置中的类型标签，也可设为无以关闭该阵营播报。 | `Sidebar.MixFileIndex`、`Sidebar.YuriFileNames`、`ToolTipColor`、`MessageTextColor`、`DialogBackground.Image`、`DialogBackground.Palette`、`LoadingTheme`、`EVA.Tag` | 建议由UI与音频资源所有者按阵营解析配置，SDK需侧栏、调色板、音乐及播报资源绑定能力。 | [User Interface and Loading Theme](https://ares-developers.github.io/Ares-docs/new/sidescountries/uiside.html) |

## 超级武器

本节 32 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR290 超武目标动画与声音 | 可为超武分别指定目标格动画、播放高度、可见阵营及目标位置音效，并设置核弹发射或可延迟超武激活时的声音。各项默认值取决于实际超武类型；普通声音与激活声音处于不同阶段，配置时应依对应类型的时序核对。 | `SW.Animation`、`SW.AnimationHeight`、`SW.AnimationVisibility`、`SW.Sound`、`SW.ActivationSound` | 建议由引擎超武事件驱动表现层，SDK需动画、位置音频和按阵营过滤可见性的能力。 | [Animation/Sound](https://ares-developers.github.io/Ares-docs/new/superweapons/animsound.html) |
| AR291 超武获取条件与次数上限 | 可按国家、人类或AI身份、辅助建筑和排斥建筑控制超武可用性，也可限制发射次数或在无提供建筑时持续授予。辅助建筑列表只需拥有其一，但建筑升级组件不算有效前置；发射次数限制不适用充放电型，常驻授予仍受其他资格限制。 | `SW.RequiredHouses`、`SW.ForbiddenHouses`、`SW.AuxBuildings`、`SW.NegBuildings`、`SW.AllowPlayer`、`SW.AllowAI`、`SW.Shots`、`SW.AlwaysGranted` | 建议由引擎超武可用性所有者集中评估条件，SDK需国家、建筑存在性及使用次数规则。 | [Availability](https://ares-developers.github.io/Ares-docs/new/superweapons/availability.html) |
| AR292 建筑附带多个超武与动画 | 建筑可用列表提供原有两个槽位以外的更多超武，并支持各项条件授予；首个可用超武决定放置时的EVA播报和建筑充能动画状态。充能、临近完成、就绪与发射动画扩展到自定义超武，但原先两个槽位仍有特殊处理，部分机制未全面扩展。 | `SuperWeapons`、`SuperWeapon`、`SuperWeapon2` | 建议由引擎建筑与超武所有者统一关联授权，UI按选定超武状态展示建筑动画及播报。 | [Building Options](https://ares-developers.github.io/Ares-docs/new/superweapons/buildings.html) |
| AR293 超武图标状态文字 | 侧栏超武图标可分别覆盖停机、就绪、可提前启动的充能中、激活消耗中及准备中的文字。充能中与激活状态专用于充放电逻辑，普通超武尚未充满时使用准备文字；停机文字还要求该超武受电力影响且建筑已关闭。 | `Text.Hold`、`Text.Ready`、`Text.Charging`、`Text.Active`、`Text.Preparing`、`UseChargeDrain` | 建议由UI从超武状态机派生图标文本，SDK需本地化标签引用和只读状态订阅能力。 | [Cameo Overlay Texts](https://ares-developers.github.io/Ares-docs/new/superweapons/cameooverlay.html) |
| AR294 超武充能与持续消耗 | 可逐超武设置充能时间与激活持续时间的倍率，并控制激活后能否手动中止，倍率不能为零。这些选项仅针对充放电逻辑；通用页旧说明只认可火墙，但三点零的电池类型页另明确采用同一机制，支持范围应结合具体类型核对。 | `UseChargeDrain`、`SW.ChargeToDrainRatio`、`SW.Unstoppable`、`ChargeToDrainRatio` | 建议由引擎超武状态机实现充能、持续与中止，SDK需类型能力校验及受控激活操作。 | [Charge/Drain Super Weapons](https://ares-developers.github.io/Ares-docs/new/superweapons/chargedrain.html) |
| AR295 超武发射费用与持续收支 | 发射超武可给玩家账户加钱或扣钱，负值表示费用；余额不足会取消发射并提示，图标也会变暗且工具提示显示所需金额。充放电超武还可按帧间隔持续产生正负收支，发射一次的金额与激活期间的周期金额分别配置。 | `Money.Amount`、`Money.DrainAmount`、`Money.DrainDelay`、`UseChargeDrain` | 建议由引擎经济所有者原子校验并结算费用，SDK需受控收支请求与UI费用状态读取能力。 | [Cost](https://ares-developers.github.io/Ares-docs/new/superweapons/cost.html) |
| AR296 超武允许与禁止光标 | 可为超武指定允许发射与禁止发射两套鼠标光标，并引用统一的光标定义资源。目标类型或拥有者不符、超出射程、缺少附近指示器等情况会使用禁止光标；没有类型特殊默认时，分别沿用攻击和禁止操作光标。 | `Cursor`、`NoCursor` | 建议由UI复用引擎发射资格结果选择光标，SDK需光标资源引用与目标资格查询能力。 | [Cursors](https://ares-developers.github.io/Ares-docs/new/superweapons/cursors.html) |
| AR297 超武EVA语音事件 | 可分别定制超武被发现、充满、激活、未就绪点击、资金不足以及请求选择目标时的EVA语音。发现播报不向建筑拥有者播放，就绪播报只给拥有者；各事件可设为无以关闭，资金不足事件默认沿用通用余额不足播报。 | `EVA.Detected`、`EVA.Ready`、`EVA.Activated`、`EVA.Impatient`、`EVA.InsufficientFunds`、`EVA.SelectTarget` | 建议由引擎超武事件标明接收对象，音频层解析EVA资源，SDK需事件和播报标签配置。 | [EVA Events](https://ares-developers.github.io/Ares-docs/new/superweapons/evaevents.html) |
| AR298 超武范围、影响对象与充能 | 通用配置提供圆形或矩形效果区、延迟、雷达事件、计时器可见性、分组及对象和阵营筛选，圆形单值通常表示半径。还支持首次即就绪与失去提供建筑后继续后台充能；后台充能不支持充放电型，延迟及范围细节须以具体类型为准。 | `SW.Range`、`SW.Deferment`、`SW.TimerVisibility`、`SW.Group`、`SW.AffectsHouse`、`SW.AffectsTarget`、`SW.InitialReady`、`SW.VirtualCharge` | 建议由引擎超武所有者维护充能与效果规则，SDK需区域查询、对象筛选及状态配置能力。 | [General Settings](https://ares-developers.github.io/Ares-docs/new/superweapons/general.html) |
| AR299 超武后续步骤关联 | 原先控制点击前后阶段与前置依赖的三个字段不再生效，改由后续超武关联字段指定发射后的下一步骤。主要用途是把超时空源点选择切换到对应的目的地选择，可让多个超时空球共用一个后续步骤，也可各自绑定独立超武。 | `SW.PostDependent`、`PreClick`、`PostClick`、`PreDependent` | 建议由引擎超武交互状态机维护后续步骤引用，SDK需有类型校验的阶段关联配置。 | [Hardcoded Values](https://ares-developers.github.io/Ares-docs/new/superweapons/hardcoded.html) |
| AR300 超武扩展总览与类型覆写限制 | 汇集超武获取条件、建筑关联、选点、费用、提示与各类型规则。超武类型和动作不能在游戏模式配置或地图中直接改写；需先在主规则定义另一超武，再让提供建筑改为引用它，避免把局部覆盖误当成类型切换。 | `Type`、`Action`、`SuperWeapon` | 建议由引擎规则加载所有者校验类型固定边界，SDK需超武注册与建筑引用配置能力。 | [Super Weapons](https://ares-developers.github.io/Ares-docs/new/superweapons/index.html) |
| AR301 主超武环境光变化 | 三类主要超武可临时改变环境亮度与红绿蓝分量，各分量可单独设置，无需整组覆写。亮度或颜色值设为负一时采用场景原值，是否启用另有开关；目前只有主要超武支持此能力，过高亮度可能降低运行速度。 | `Light.Enabled`、`Light.Ambient`、`Light.Red`、`Light.Green`、`Light.Blue` | 建议由表现层依据引擎超武阶段混合环境光，SDK需灯光参数资源与类型能力声明。 | [Lighting](https://ares-developers.github.io/Ares-docs/new/superweapons/lighting.html) |
| AR302 超武通知文字与颜色 | 可为发现、就绪、发射、延迟激活、冲突中止、费用不足和自动选点失败分别配置文字，接收者按事件区分为所有玩家或发射者。消息可用发射国家颜色，也可指定固定配色；若启用发射者颜色，固定配色不会生效。 | `Message.Detected`、`Message.Ready`、`Message.Launch`、`Message.Activate`、`Message.Abort`、`Message.CannotFire`、`Message.FirerColor`、`Message.Color` | 建议由UI按超武事件与接收权限展示本地化文本，SDK需消息标签、颜色和失败原因能力。 | [Messages](https://ares-developers.github.io/Ares-docs/new/superweapons/messages.html) |
| AR303 超武射程、指示器与抑制器 | 超武可同时受提供建筑的最小最大射程、己方指示器覆盖和敌方抑制器区域约束。至少一个提供建筑须同时满足两项射程，有一个敌方抑制器覆盖便禁止发射；建筑抑制器还须在线并满足工程师条件，指示和抑制半径默认取视野。 | `SW.RangeMaximum`、`SW.RangeMinimum`、`SW.Designators`、`SW.AnyDesignator`、`DesignatorRange`、`SW.Inhibitors`、`SW.AnyInhibitor`、`InhibitorRange` | 建议由引擎超武选点所有者统一查询空间限制，SDK需实体区域检索及可用状态判定能力。 | [Range Limits](https://ares-developers.github.io/Ares-docs/new/superweapons/range.html) |
| AR304 超武手动与自动选点 | 可让人类超武采用AI选点或充满自动发射，并分别限制手动目标、阵营与迷雾；自动选点提供基地、敌军、缺电等模式及约束、地图指定位置偏好。模式与类型默认值需配合，旧地图首选目标类型仅支持一，使用它时可能跳过射程和指示器检查。 | `SW.UseAITargeting`、`SW.AutoFire`、`SW.ManualFire`、`SW.RequiresTarget`、`SW.RequiresHouse`、`SW.AITargeting`、`SW.AITargeting.Constraints`、`SW.AITargeting.Preference` | 建议由引擎目标选择所有者统一执行约束，SDK需只读态势查询与可配置选点策略。 | [Targeting](https://ares-developers.github.io/Ares-docs/new/superweapons/targeting.html) |
| AR305 Type=Battery：备用供电 | 电池超武在激活期间提供恒定备用电力，并可让指定建筑过充使用副武器，或无视基地电量保持在线；过充不要求建筑原本允许充能。效果按充放电持续时间自动结束后重新充能，默认AI在缺电时使用，但并非所有建筑效果都能在全基地停电时正常运作。 | `SW.AITargeting`、`SW.ChargeToDrainRatio`、`Battery.Power`、`Battery.Overpower`、`Battery.KeepOnline`、`Overpowerable` | 建议由引擎电力与超武所有者维护临时增益，SDK需供电修正、保持在线及过充规则能力。 | [Type=Battery](https://ares-developers.github.io/Ares-docs/new/superweapons/types/battery.html) |
| AR306 Type=ChronoSphere / Type=ChronoWarp：超时空转移 | 超时空球选择源区，配套转移超武选择目的地；可筛选对象、转移建筑、控制有机体与乘员死亡，以及铁幕或不可转移对象的处理。普通建筑放不下时默认在原地炸毁，可收起建筑有车辆放置选项；AI不会使用，建筑转移仍有建造动画重播与炮塔朝向重置问题。 | `SW.PostDependent`、`SW.Range`、`Chronosphere.ReconsiderBuildings`、`Chronosphere.KillOrganic`、`Chronosphere.AffectsUndeployable`、`Chronosphere.BlowUnplaceable`、`Chronosphere.KillCargo`、`Chronosphere.AffectsIronCurtain` | 建议由引擎移动与建筑所有者执行原子转移，SDK需受控传送、放置校验及两阶段交互能力。 | [Type=ChronoSphere](https://ares-developers.github.io/Ares-docs/new/superweapons/types/chronosphere.html) |
| AR307 Type=DropPod：空降舱 | 空降舱在配置的最小与最大数量间投放随机步兵，每项等概率，重复列出类型可增加权重，并能设初始经验。没有内置默认兵种，须自行配置；落点需要净空，所有单位的放置重试总数最多为计划数量三倍，超限后停止继续投放。 | `DropPod.Types`、`DropPod.Veterancy`、`DropPod.Minimum`、`DropPod.Maximum`、`SW.AITargeting` | 建议由引擎增援与放置所有者负责随机选兵和落点，SDK需受控生成、经验及空降资源能力。 | [Type=DropPod](https://ares-developers.github.io/Ares-docs/new/superweapons/types/droppod.html) |
| AR308 Type=EMPulse：电磁脉冲炮联射 | 使一个或多个指定建筑同时攻击目标或在自身位置引爆主武器，可限制炮数并让一门炮射程合格后联动其余炮。专用电磁炮会转向并延迟投弹，普通建筑直接开火且不支持炮塔；伤及自身需另允自伤，AI目前不检查射程，默认禁用自动选点。 | `EMPulse.Cannons`、`EMPulse.TargetSelf`、`EMPulse.Linked`、`EMPulse.PulseBall`、`EMPulse.PulseDelay`、`SW.MaxCount`、`SW.RangeMinimum`、`SW.RangeMaximum` | 建议由引擎武器与超武所有者协调炮组及时序，SDK需受控开火、区域判定和自伤策略。 | [Type=EMPulse](https://ares-developers.github.io/Ares-docs/new/superweapons/types/empulse.html) |
| AR309 Type=Firestorm：火墙防御 | 恢复火墙防御系统，激活后按充能与消耗比例维持防御，耗尽后自动关闭重充，也可再次点击图标提前关闭以更早恢复。此类型强制使用充放电，并忽略动作字段以实现单击激活；AI不会主动使用，但任务可通过地图动作九十二和九十三控制。 | `SW.ChargeToDrainRatio`、`UseChargeDrain`、`Action` | 建议由引擎火墙与超武所有者统一维护激活状态，SDK需防御网络控制及图标交互能力。 | [Type=Firestorm](https://ares-developers.github.io/Ares-docs/new/superweapons/types/firestorm.html) |
| AR310 Type=GenericWarhead：通用弹头 | 在指定目标格直接引爆配置的弹头，并使用超武伤害值，默认以进攻模式自动选点。爆炸落在格子而非直接作用于单位，尤其要影响步兵时通常需要弹头扩散；若希望同时伤及提供超武的建筑，该建筑还必须允许自身伤害。 | `SW.Damage`、`SW.Warhead`、`SW.AITargeting`、`CellSpread`、`DamageSelf` | 建议由引擎伤害所有者提供格点引爆入口，SDK需受控区域伤害、弹头资源及自伤资格能力。 | [Type=GenericWarhead](https://ares-developers.github.io/Ares-docs/new/superweapons/types/genericwarhead.html) |
| AR311 Type=GeneticConverter：基因转换 | 基因转换可选择整次弹头爆炸，或逐个处理范围内步兵；爆炸模式忽略范围及其他专属筛选，非爆炸模式遵守装甲倍率与免疫。可排除改造人或非人类，并让自然生物直接死亡；影响目标选项只支持陆地和水域限制，不能用来选择单位类别。 | `SW.Range`、`SW.Damage`、`SW.Warhead`、`SW.AffectsTarget`、`Mutate.Explosion`、`Mutate.IgnoreCyborg`、`Mutate.IgnoreNotHuman`、`Mutate.KillNatural` | 建议由引擎伤害与变异所有者处理模式及免疫，SDK需步兵筛选、弹头效果和受控类型转换能力。 | [Type=GeneticConverter](https://ares-developers.github.io/Ares-docs/new/superweapons/types/geneticconverter.html) |
| AR312 Type=HunterSeeker：猎杀者 | 从合格建筑发射自动追击并撞击敌人的车辆型猎杀者，每座最多一架，并可限制发射建筑数。类型可覆盖阵营默认值，也可对所有敌物等概率抽取；没有发射建筑仍会消耗充能，飞行中可能换目标，铁幕与正在被超时空抹除的对象不会入选。 | `SW.MaxCount`、`SW.AffectsHouse`、`SW.AffectsTarget`、`HunterSeeker.Buildings`、`HunterSeeker.Type`、`HunterSeeker.RandomOnly` | 建议由引擎生成、导航与目标选择所有者管理猎杀者，SDK需发射点查询及受控生成追击能力。 | [Type=HunterSeeker](https://ares-developers.github.io/Ares-docs/new/superweapons/types/hunterseeker.html) |
| AR314 Type=LightningStorm：闪电风暴 | 可调雷击伤害、延迟、持续时长、雷达干扰、定点与随机雷云频率和间隔，并自定义云、闪电、碎片及声音。此类型范围单值表示直径，区别于通用半径规则；云列表为空则风暴不生效，闪电动画不得带弹跳标记，否则击中建筑可能崩溃。 | `SW.Range`、`Lightning.Duration`、`Lightning.RadarOutage`、`Lightning.HitDelay`、`Lightning.ScatterDelay`、`Lightning.ScatterCount`、`Lightning.Clouds`、`Lightning.Bolts` | 建议由引擎天气所有者维护确定性雷云与伤害时序，SDK需天气控制、资源引用及雷达状态能力。 | [Type=LightningStorm](https://ares-developers.github.io/Ares-docs/new/superweapons/types/lightningstorm.html) |
| AR315 Type=MultiMissile：核导弹 | 核导弹可独立指定载荷、伤害、弹头、发射动画与预警，并选择由发射井或地图外投送，支持多个合格发射建筑。载荷武器必须列入武器类型表；上升武器须启用核弹生成标记才能产生下落阶段，弹体还可用垂直选项改变飞行方式。 | `SW.Damage`、`SW.Warhead`、`Nuke.Payload`、`Nuke.TakeOff`、`Nuke.PsiWarning`、`Nuke.SiloLaunch`、`WeaponType`、`NukeMaker` | 建议由引擎导弹与超武所有者串联发射和载荷，SDK需受控投射物生成及阶段资源配置能力。 | [Type=MultiMissile](https://ares-developers.github.io/Ares-docs/new/superweapons/types/multimissile.html) |
| AR316 Type=ParaDrop / Type=AmerParaDrop：空降增援 | 两种空降类型仅默认值不同，均可发送多架自选运输机，并按超武、国家、阵营及飞机序号逐级覆盖内容。新字段允许步兵与车辆混合，旧兵种列表仅支持步兵；类型和数量必须配套覆写，飞机类型则可单独继承或替换。 | `ParaDrop.Types`、`ParaDrop.Num`、`ParaDrop.Aircraft`、`ParaDrop.Count`、`SW.AITargeting` | 建议由引擎增援所有者解析层级默认值并投放，SDK需飞机生成、载荷列表和空降资源能力。 | [Type=ParaDrop and Type=AmerParaDrop](https://ares-developers.github.io/Ares-docs/new/superweapons/types/paradrop.html) |
| AR317 Type=IronCurtain / Type=ForceShield：区域保护 | 铁幕与力场护盾共享保护机制，差异主要为范围、对象、阵营、停电等默认值，建筑保护色仍按类型固定。可设持续时间、发射后停电和结束前提示音，提示时间须小于总时长；AI铁幕可按分组响应队伍脚本，但尚未完整支持AI触发器。 | `Protect.Duration`、`Protect.PowerOutage`、`Protect.PlayFadeSoundTime`、`SW.Group`、`SW.Range`、`SW.AffectsHouse`、`SW.AffectsTarget`、`SW.RequiresTarget` | 建议由引擎保护与电力所有者维护效果计时，SDK需受控保护、阵营筛选及脚本分组能力。 | [Type=IronCurtain and Type=ForceShield](https://ares-developers.github.io/Ares-docs/new/superweapons/types/protect.html) |
| AR318 Type=PsychicDominator：心灵控制 | 心灵控制器先造成可选伤害，再在单独捕获范围内夺取步兵或车辆，可关闭夺取只保留伤害。允许控制动画、触发帧比例、涟漪和永久性，并分别处理已心控、已永久心控及通常免疫心控的对象；伤害范围不由捕获范围决定。 | `SW.Range`、`SW.Damage`、`Dominator.FireAtPercentage`、`Dominator.Capture`、`Dominator.CaptureMindControlled`、`Dominator.CapturePermaMindControlled`、`Dominator.CaptureImmuneToPsionics`、`Dominator.PermanentCapture` | 建议由引擎伤害与心控所有者按阶段处理归属，SDK需受控控制权变更、筛选和动画时序能力。 | [Type=PsychicDominator](https://ares-developers.github.io/Ares-docs/new/superweapons/types/psychicdominator.html) |
| AR319 Type=PsychicReveal：心灵探视 | 揭示目标周围地图，默认范围从原版全局值读取时仍封顶十格，显式指定超武范围即可解除此限制。范围设为负一时改为全图揭示，点击图标直接生效，无需再选目标；可另配激活声音，AI默认沿用空降式选点。 | `SW.Range`、`SW.Sound`、`SW.AITargeting`、`Cursor` | 建议由引擎战争迷雾所有者处理区域揭示，SDK需受控探索状态操作及免选点激活能力。 | [Type=PsychicReveal](https://ares-developers.github.io/Ares-docs/new/superweapons/types/psychicreveal.html) |
| AR320 Type=SonarPulse：声纳反隐 | 让范围内或全图的隐形对象暂时显形，即使隐形来自发生器也有效。默认只影响敌方水域对象，手动与AI目标条件也默认限水面；范围负一时点击图标即可全图触发，不必选择位置，显形持续时间可独立按帧设置。 | `SW.Range`、`SW.AITargeting`、`SW.AffectsHouse`、`SW.AffectsTarget`、`SW.RequiresTarget`、`SW.AIRequiresTarget`、`SonarPulse.Delay` | 建议由引擎隐形状态所有者维护临时显形，SDK需区域实体筛选与受控反隐效果能力。 | [Type=SonarPulse](https://ares-developers.github.io/Ares-docs/new/superweapons/types/sonarpulse.html) |
| AR321 Type=SpyPlane：侦察机支援 | 可指定派出的飞机类型、数量与初始任务，从传统侦察扩展为以守卫、攻击或移动等任务到达的飞机支援。默认发送一架原版侦察机并执行侦察接近任务，自动选点沿用空降模式；作者需同时配置飞机类型本身的行为与资源。 | `SpyPlane.Type`、`SpyPlane.Count`、`SpyPlane.Mission`、`SW.AITargeting`、`Cursor` | 建议由引擎增援与任务所有者派遣飞机，SDK需受控飞机生成、初始任务及资源引用能力。 | [Type=SpyPlane](https://ares-developers.github.io/Ares-docs/new/superweapons/types/spyplane.html) |
| AR322 Type=UnitDelivery：对象投送 | 延迟后在目标区投放步兵、车辆、飞机或建筑，可选择归属及建筑是否扩展建造空间。落点含随机选择且顺序不定，飞机也先放地面，常驻飞行单位会起飞；海军须合法地形，AI单位进入猎杀、建筑进入守卫，人类单位进入区域守卫。 | `SW.Deferment`、`SW.ActivationSound`、`SW.AITargeting`、`Deliver.Types`、`Deliver.Owner`、`Deliver.BaseNormal` | 建议由引擎生成与地图放置所有者统一创建对象，SDK需受控生成、归属、任务和基地范围能力。 | [Type=UnitDelivery](https://ares-developers.github.io/Ares-docs/new/superweapons/types/unitdelivery.html) |

## 矿物与资源

本节 6 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR325 矿物爆炸与连锁反应 | 恢复动画命中矿格引发爆炸，以及覆盖物受损后向邻格传播动画的两种连锁机制。每种矿物可设爆炸弹头、动画命中伤害和碎片概率；另可提高可爆覆盖物的触发门槛，只有单次伤害高于阈值才爆炸，弹头扩散须自行配置。 | `ExplosionWarhead`、`ExplosionDamage`、`Debris.Chance`、`OverlayExplodeThreshold`、`TiberiumChainReaction`、`Explodes` | 建议由引擎矿物与覆盖物所有者传播反应，SDK需地图资源操作、区域伤害及碎片事件能力。 | [Chain Reactions](https://ares-developers.github.io/Ares-docs/new/tiberium/chainreactions.html) |
| AR326 矿物行走伤害与变异生物 | 启用后，不免疫的单位进入矿格时受伤，停在格中不会因停留时长持续受伤，高于悬浮高度也不受影响。矿种可分别配置伤害与弹头；矿伤致死有机会生成中立小变异生物，但地图还须启用死亡变异，小型个体相遇可合并为大型。 | `TiberiumDamageEnabled`、`TiberiumProof`、`Damage`、`Warhead`、`TiberiumTransmogrify`、`TiberiumDeathToVisceroid`、`SmallVisceroid`、`LargeVisceroid` | 建议由引擎移动入格与伤害所有者触发矿伤，SDK需地表查询、受控伤害和死亡生成能力。 | [Tiberium Damage](https://ares-developers.github.io/Ares-docs/new/tiberium/damage.html) |
| AR327 载矿单位死亡爆炸 | 恢复载矿矿车被毁时引爆货物的机制，伤害取决于所载矿物能量，只结算正伤害。必须配置专用爆炸弹头，并由弹头自身提供扩散；启用矿车停战时不造成伤害。独立爆炸动画仍可显示，且本机制不同于地面矿格连锁爆炸。 | `TiberiumExplosive`、`TiberiumExplosiveWarhead`、`TiberiumExplosiveAnim`、`Power`、`CellSpread` | 建议由引擎死亡与货物所有者计算载矿爆炸，SDK需库存读取、区域伤害及死亡表现事件。 | [Tiberium Explosive](https://ares-developers.github.io/Ares-docs/new/tiberium/explosive.html) |
| AR328 矿物治疗与死亡遗矿 | 恢复单位站立或移动在矿格上时周期回血，可按矿种独立设间隔及步兵、车辆、飞机恢复量。须先开启全局功能并赋予单位资格或老兵能力，高于悬浮高度不治疗；死亡遗矿另有开关，最多尝试在邻格留下五份矿物。 | `TiberiumHealEnabled`、`TiberiumHeal`、`Heal.Delay`、`Heal.Step`、`Heal.IStep`、`Heal.UStep`、`TiberiumRemains` | 建议由引擎地表效果与生命所有者统一计时，SDK需矿物查询、治疗及死亡地图资源操作能力。 | [Tiberium Heal](https://ares-developers.github.io/Ares-docs/new/tiberium/heal.html) |
| AR330 死亡时矿物溢出 | 有正储量的单位或建筑可在毁坏时把部分内容洒回地图，默认不开启。矿厂和筒仓按实际库存种类与数量溢出；矿车和奴隶则依据装载比例最多洒出九份，且不超过自身容量，洒出类型固定取矿物列表第一项，不能混同建筑库存规则。 | `TiberiumSpill`、`Storage` | 建议由引擎库存与地图资源所有者结算死亡溢出，SDK需货物访问及受控矿格写入能力。 | [Tiberium Spill](https://ares-developers.github.io/Ares-docs/new/tiberium/spill.html) |
| AR331 矿物储存、筒仓与容量提示 | 矿厂可把卸下资源存入建筑容量而非直接兑钱，净化器额外收益仍直接进账户。支持容量色格、筒仓不足语音与文字；语音须自备资源，可收起矿厂不支持储存，原矿厂启用容量显示还须明确开启储存，文字取消需使用空文本标签。 | `Refinery.UseStorage`、`Message.SilosNeeded`、`PipScale`、`Storage`、`Refinery`、`ResourceDestination` | 建议由引擎经济与库存所有者维护资源容量，UI显示容量及警报，SDK需受控库存操作。 | [Storage and Silos](https://ares-developers.github.io/Ares-docs/new/tiberium/storage.html) |

## 资源与配置加载

本节 8 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR243 随机地图的缺省地形参数 | 官方为缺少配置的雪地地形文件提供 Medians=71，避免启用城市区域或地形扩展时触发内部错误。 | `Medians` | 在地形或生成器配置校验层处理，不是战斗对象插件功能。 | [Avoidable Internal Errors](https://ares-developers.github.io/Ares-docs/new/misc/avoidableinternalerrors.html) |
| AR244 额外音频索引与音频包 | 允许在基础 audio.idx/bag 后加载 audio01 到 audio99，后加载索引可覆盖前面的音效，便于小体积声音补丁。 | 无独立键或见原文 | 由资源加载与音频目录维护覆盖顺序，不在Wasm中读取磁盘包。 | [Additional Bag Files](https://ares-developers.github.io/Ares-docs/new/misc/bagfiles.html) |
| AR245 INI拆分与递归包含 | 用 include 节把Rules拆成多个文件，递归采用深度优先顺序，后读到的同一配置会覆盖先前值；子路径和MIX查找有各自规则。 | `#include` | 由配置加载器展开并保留物理读取层与顺序，插件只消费已接纳的视图。 | [[#include]](https://ares-developers.github.io/Ares-docs/new/misc/include.html) |
| AR247 列表配置长度扩展 | 对接收列表的配置放宽长度，官方上限按整段标签名和等号一起计入；这不是所有字符串值都统一扩容。 | 无独立键或见原文 | 在对应读取器设容量并保留截断策略，禁止以无限长字符串替换兼容行为。 | [List Lengths](https://ares-developers.github.io/Ares-docs/new/misc/listlengths.html) |
| AR248 散装音效文件 | 支持从游戏目录或MIX直接读取受支持的WAV，soundmd仍按不带扩展名的名字引用，不要求先放入BAG。 | 无独立键或见原文 | 接入资源解析与音频解码层，插件只引用逻辑资源名。 | [Loose Audio Files](https://ares-developers.github.io/Ares-docs/new/misc/looseaudiofiles.html) |
| AR249 MIX覆盖顺序调整 | 调整expandmd扩展包与语言包的加载优先级，使扩展内容可以覆盖langmd中的同名资源。 | 无独立键或见原文 | 由单一资源搜索顺序实现；验证同名文件最终来源和读档资源身份。 | [MIX Loading Order](https://ares-developers.github.io/Ares-docs/new/misc/mixloadingorder.html) |
| AR250 压缩鼠标图像的未完成项目 | 页面提出为mouse.sha使用SHP压缩，但官方同时明确标注尚未成功实现，不能把它列为Ares已完成能力。 | 无独立键或见原文 | 作为待研究的资源解码项目记录，不能用不确定行为作为兼容验收基准。 | [Mouse.sha](https://ares-developers.github.io/Ares-docs/new/misc/mousesha.html) |
| AR251 附加字符串表与直接显示名 | 支持附加CSF覆盖、NOSTR直接文本、更多字符串和语言中立表头；NOSTR长度与CSF语言标记仍有规则。 | `UIName` | 由本地化资源目录统一合并，UI只读取稳定字符串ID。 | [String Table Enhancements](https://ares-developers.github.io/Ares-docs/new/misc/stringtableenhancements.html) |

## 界面与诊断

本节 20 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR384 位图截图与时间命名 | 截图使用BMP并按时间命名，避免已有截图较多时逐个查找编号；这是截图输出能力，与地图快照不同。 | 无独立键或见原文 | 放在应用和图像导出层，保持文件写入不影响模拟帧。 | [Bitmap Screenshots](https://ares-developers.github.io/Ares-docs/ui-features/bitmapscreenshots.html) |
| AR385 可滚动战役列表 | 用列表替代固定战役按钮，从battlemd读取战役、描述、摘要及选中音效，并可显示调试战役；列表可视数量受摘要布局影响。 | `CampaignList`、`CampaignListSize`、`ShowSummary`、`ShowDebugCampaigns`、`HoverSound`、`Summary` | 扩展战役目录和前端视图，不能依赖游戏对象 Tick 实现。 | [Campaign List](https://ares-developers.github.io/Ares-docs/ui-features/campaignlist.html) |
| AR386 战役加载文字与音乐 | 按任务配置加载画面文字颜色，并从地图 Basic 指定加载音乐；任务名前缀只用于未显式配置时的默认选择。 | `LoadScreenText.Color`、`LoadingTheme` | 由任务元数据、加载界面和音频场景共同消费。 | [Campaign Load Screen](https://ares-developers.github.io/Ares-docs/ui-features/campaignloadscreen.html) |
| AR387 旧引擎启动与调试参数 | 提供跳过片头、日志、严格检查、离线AI控制与处理器亲和性等旧引擎选项；部分是Windows兼容措施，并非跨平台SDK要求。 | 无独立键或见原文 | 按需要映射到启动器或应用诊断；不照搬DLL注入及旧系统设置。 | [Command Line Arguments](https://ares-developers.github.io/Ares-docs/ui-features/commandlinearguments.html) |
| AR388 自定义战役按钮 | 可配置最多四个战役按钮的目标任务、图像、调色板、副标题和提示；隐藏按钮会影响布局组合。 | `CampaignX`、`CampaignX.Image`、`CampaignX.Palette`、`CampaignX.Subline`、`CampaignX.Tooltip` | 建立战役按钮的前端描述及资源绑定，与模拟逻辑分开。 | [Customizable Campaign Buttons](https://ares-developers.github.io/Ares-docs/ui-features/customizablecampaignbuttons.html) |
| AR389 玩家颜色下拉项 | 允许配置颜色预览、游戏颜色方案和提示，官方Count只允许8到16；它不是30名玩家或30种玩家颜色的支持承诺。 | `Count`、`SlotX.DisplayColor`、`SlotX.ColorScheme`、`SlotX.Tooltip` | 房间前端和颜色方案目录统一映射；VERA更大人数另设协议与UI能力。 | [Customizable Dropdown Colors](https://ares-developers.github.io/Ares-docs/ui-features/customizabledropdowncolors.html) |
| AR390 退出时跳过确认 | 在uimd中选择主菜单退出按钮是否立即关闭游戏；默认仍显示确认对话框。 | `QuickExit` | 由应用退出流程处理，不能让插件直接终止进程。 | [Exit Without Confirmation](https://ares-developers.github.io/Ares-docs/ui-features/exit.html) |
| AR391 旧DirectDraw绘制选项 | ares.ini可选择旧绘图路径，但软件模拟在较新Windows中不可用；官方把该文件定位为用户设置，而非Mod应随意覆盖的配置。 | `DirectX.Force` | 评估现代渲染后端的等效需求，不复制旧DirectDraw路径为SDK接口。 | [Graphics / Surface Drawing](https://ares-developers.github.io/Ares-docs/ui-features/graphicssurfacedrawing.html) |
| AR393 内部错误与诊断报告 | 增强异常文本、日志和崩溃转储，可在加载阶段提前发现部分配置错误；严格检查会对某些非致命错误也报错。 | 无独立键或见原文 | 由宿主诊断与配置校验管理；组件错误要说明调用和对象身份。 | [Internal Errors / Debugging](https://ares-developers.github.io/Ares-docs/ui-features/internalerrorsdebugging.html) |
| AR394 调试快捷键与禁用开关 | 增加地图快照、日志切换、类型及AI计划导出、FPS、AI控制和断电命令；部分调试键可禁用，官方明确这不构成素材防提取保护。 | `DebugKeysEnabled` | 由输入、诊断和编辑工具注册命令；会改变游戏状态的命令必须进入同步输入。 | [Keyboard Commands (Hotkeys)](https://ares-developers.github.io/Ares-docs/ui-features/keyboardcommandshotkeys.html) |
| AR395 加载画面提示调整 | 启动加载画面显示Ares版本信息；国家加载文本为空时不再为不存在的文字绘制暗色区域。 | 无独立键或见原文 | 加载界面按资源与文本实际内容布局，不影响战斗模拟。 | [Loading Screen](https://ares-developers.github.io/Ares-docs/ui-features/loadingscreen.html) |
| AR396 单人及同队遭遇战 | 允许没有电脑对手的单人测试，也允许所有玩家同队；官方说明同队设置下仍有特定败北结束条件。 | 无独立键或见原文 | 调整房间准入与胜负规则，不能只去掉界面中的人数检查。 | [Lone Player and Single Team Skirmish](https://ares-developers.github.io/Ares-docs/ui-features/loneplayerskirmish.html) |
| AR397 主菜单按钮行为自定义 | 可让指定菜单按钮保留、禁用、隐藏或改为消息、片头及鸣谢等动作；隐藏后要重新布局。 | `SinglePlayerButton`、`NetworkButton`、`WWOnlineButton`、`MoviesAndCreditsButton`、`CampaignButton`、`SkirmishButton`、`SneakPeeksButton`、`PlayMoviesButton` | 前端使用白名单动作描述，不允许INI提供任意宿主代码。 | [Menu Button Customization](https://ares-developers.github.io/Ares-docs/ui-features/menubuttoncustomization.html) |
| AR398 作品名称与版本标识 | 在uimd设置作品名和版本，供日志与存档兼容识别；版本不应仅作显示文本使用。 | `Name`、`Version` | 把作品身份纳入包、对局和存档元数据；不要只依靠可随意改写的名称。 | [Mod-specific Information](https://ares-developers.github.io/Ares-docs/ui-features/modinfo.html) |
| AR399 更多游戏分辨率 | 开放旧选项界面的更多分辨率，官方范围到4096×4096，并使旧高分辨率作弊开关失去必要性。 | 无独立键或见原文 | 按窗口及渲染设备能力提供设置，不把历史范围当现代后端硬上限。 | [New Game Resolutions](https://ares-developers.github.io/Ares-docs/ui-features/newgameresolutions.html) |
| AR400 单次加入多单位生产队列 | Shift加左键可一次加入五个同类单位，与Shift加右键清空同类队列对应；不支持建筑。 | 无独立键或见原文 | 由前端生成确定性的排队命令，再交现有生产队列校验资金和限制。 | [Queue More Than One Unit with a Single Click](https://ares-developers.github.io/Ares-docs/ui-features/queueunts.html) |
| AR401 随机地图样式与城市区域 | 新增群岛、沙漠、随机桥梁及城市区域对象列表；官方注明预览与实际结果可能不一致，并有部分设置失效报告。 | `Structures`、`Vehicles`、`Infantry` | 属于生成器与预览管线；当前SDK声明组件时的生成地图限制需单独扩展。 | [Random Map Generator](https://ares-developers.github.io/Ares-docs/ui-features/randommapgenerator.html) |
| AR402 开发版本屏幕说明 | 通过附加字符串表中的TXT_RELEASE_NOTE显示开发或测试版本说明；不要求新增Rules功能键。 | 无独立键或见原文 | 用作品元数据和本地化文字呈现，不加入模拟状态。 | [Release Note](https://ares-developers.github.io/Ares-docs/ui-features/releasenote.html) |
| AR403 Mod存档版本兼容 | 识别游戏、Mod和Ares版本兼容性并隐藏不兼容存档；音效目录顺序或美术文件变化也可能影响恢复。 | 无独立键或见原文 | 复用宿主存档和资源身份验证，所有功能必须提供稳定状态及版本策略。 | [Save Games](https://ares-developers.github.io/Ares-docs/ui-features/savegames.html) |
| AR404 界面文字 边框与选中颜色 | 在uimd中定制控件、边框、选中和禁用颜色；观察者控件有独立颜色规则，不完全随全局颜色变化。 | `Color.Border1`、`Color.Border2`、`Color.Text`、`Color.X.Text`、`Color.Selection`、`Color.X.Selection`、`Color.Disabled`、`Color.X.Disabled` | 由主题与UI控件层消费，保持观战语义和可读性。 | [User Interface Colors](https://ares-developers.github.io/Ares-docs/ui-features/userinterfacecolors.html) |

## 原版缺陷修复

本节 54 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR002 允许手动指定飞机停靠建筑 | 原版不允许玩家手动选定飞机的停靠建筑。修复后可向飞机停靠列表中的建筑下达降落命令，并以进入或禁止进入光标提示可用性。 | `Dock` | 建议由停靠命令所有者统一验证候选建筑并生成光标反馈。 | [Aircraft Can Be Manually Docked to Structures](https://ares-developers.github.io/Ares-docs/bugfixes/type1/aircraftdock.html) |
| AR003 大型超级武器忽略隐形目标 | AI为闪电风暴、核弹、心灵控制器和基因突变选点时曾把隐形对象算作目标，显得能够作弊侦察；修复使这些超级武器不再据此选点。 | `Type` | 建议由超级武器目标筛选所有者统一执行隐形可见性判断。 | [AI Targeting Cloaked Objects With Major Super Weapons](https://ares-developers.github.io/Ares-docs/bugfixes/type1/aitargetingcloakedobjectswithmajorsuperweapons.html) |
| AR004 启用多帧鼠标光标 | 部分操作光标虽有多帧图像却只显示首帧。修复启用伞兵、超时空、铁幕、引爆及伊文炸弹光标动画，使既有素材按预期播放。 | 无独立键或见原文 | 建议由光标绘制所有者维护帧推进，随当前操作切换动画。 | [Animated Cursors](https://ares-developers.github.io/Ares-docs/bugfixes/type1/animatedcursors.html) |
| AR005 弹跳动画缺失弹头时不结算伤害 | 弹跳动画即使没有设置弹头也会尝试伤害结算，既无法安全关闭伤害又可能崩溃。修复先检查弹头是否存在，使无弹头动画可安全使用。 | `Warhead` | 建议由动画伤害所有者在结算前检查弹头引用并跳过空值。 | [Bouncing Animation Damage Issues](https://ares-developers.github.io/Ares-docs/bugfixes/type1/animdamage.html) |
| AR006 红血战斗碉堡仍可进驻 | 普通中立驻军建筑红血后会逐出步兵并禁止再进入，战斗碉堡虽不逐人却误用同一入驻限制；修复允许新步兵进入红血碉堡。 | 无独立键或见原文 | 建议由建筑进驻所有者区分碉堡和普通驻军建筑的准入规则。 | [Infantry Cannot Enter Battle Bunkers in Red Health](https://ares-developers.github.io/Ares-docs/bugfixes/type1/battlebunkerredhealth.html) |
| AR007 开放式载具乘客不显示尾随动画 | 步兵在开放式载具内时仍会显示其尾随动画，使乘客的视觉效果错误地出现在车外。修复抑制这一状态下的动画显示。 | `Behind`、`OpenTopped` | 建议由附属动画绘制所有者检查乘坐状态，抑制舱内尾随效果。 | [Behind Anim and OpenTopped Transports](https://ares-developers.github.io/Ares-docs/bugfixes/type1/behindanimopentopped.html) |
| AR008 禁止敌车堵塞矿厂和战车工厂出口 | 敌方车辆曾能停在矿厂及战车工厂的可通行地基上堵门，攻击堵门车辆又会误伤下方建筑；修复禁止敌车穿行这些建筑格。 | 无独立键或见原文 | 建议由通行判定所有者结合建筑归属检查地基通行权限。 | [Blocking Enemy Refineries and War Factories](https://ares-developers.github.io/Ares-docs/bugfixes/type1/blockedrefinerywarfactory.html) |
| AR009 切换目标不重置连发计数 | 多发武器在目标死亡、换目标或停止时会清零连发计数，却不重新装填，因而可快速刷新整轮射击。修复保留计数以封堵此漏洞。 | `Burst` | 建议由武器射击状态所有者跨目标变更保留当前连发进度。 | [Burst Abuse](https://ares-developers.github.io/Ares-docs/bugfixes/type1/burstabuse.html) |
| AR010 混乱状态不接受停止命令 | 混乱气体本应令单位自主攻击，但玩家仍可用停止命令压住其行为，减轻误伤己方的代价。修复使混乱中的单位也拒绝停止指令。 | `Psychedelic` | 建议由命令准入所有者统一拦截混乱状态下的玩家命令。 | [Chaos Gas ( Psychedelic=yes ) and the Stop Command](https://ares-developers.github.io/Ares-docs/bugfixes/type1/chaosgasandstopcommand.html) |
| AR011 修复同步校验器越界写入 | 用于比较游戏状态的校验器曾写出自身对象边界，可能破坏其他内存。修复消除该写越界；官方未确认它此前具体造成何种故障。 | 无独立键或见原文 | 建议由同步校验所有者限定缓冲区边界并验证写入长度。 | [Checksummer Implementation Error](https://ares-developers.github.io/Ares-docs/bugfixes/type1/checksumimpl.html) |
| AR012 取消超时空选点不重复播报就绪 | 超时空传送选好起点后取消终点选择，原版会再次播报装置就绪，尽管武器从未消耗。修复去除这条不对应真实充能变化的提示。 | 无独立键或见原文 | 建议由超级武器交互所有者区分选点取消和实际充能完成。 | [Aborting the Chronoshift announces the Chronosphere as ready](https://ares-developers.github.io/Ares-docs/bugfixes/type1/chronoshiftabort.html) |
| AR013 超时空传送入水的悬浮跳跃单位会沉没 | 夜鹰或展开后的武装直升机等使用悬浮速度类型的跳跃单位，被传送进水后原可存活并起飞。修复按此情形的落点规则令其沉没。 | `SpeedType` | 建议由传送落点所有者按运动类型判定入水生存与销毁。 | [Chronoshift Will Sink Jumpjet Units](https://ares-developers.github.io/Ares-docs/bugfixes/type1/chronoshiftwillsinkjumpjetunits.html) |
| AR014 建造防御建筑时被夺取建造场不崩溃 | 玩家在防御建筑分页生产建筑时，建造场若被敌人夺取，原版可能崩溃。此修复使生产队列与建造场归属变化能够安全衔接。 | 无独立键或见原文 | 建议由建筑生产所有者在工厂易主时安全迁移或终止队列。 | [Construction Yard Capture Crash](https://ares-developers.github.io/Ares-docs/bugfixes/type1/conyardcapture.html) |
| AR015 停靠搜索距离溢出采用饱和值 | 寻找矿厂、机场或维修厂时，距离运算溢出可能令极远建筑比近处更优。修复把超大距离限制为最大值，使它们同为最差候选。 | 无独立键或见原文 | 建议由空间距离所有者使用饱和运算，供所有停靠搜索共用。 | [Distance Calculation Causes Overflow](https://ares-developers.github.io/Ares-docs/bugfixes/type1/distanceoverflow.html) |
| AR016 禁止向敌方矿车下达警戒命令 | 原版可像控制己方单位一样命令敌矿车进入警戒，令其停止正常经济活动。修复在命令权限上排除敌方矿车，封堵这一干扰手段。 | 无独立键或见原文 | 建议由玩家命令所有者在分派前验证单位控制权与目标归属。 | [Enemy Harvester Guard Mode Exploit](https://ares-developers.github.io/Ares-docs/bugfixes/type1/enemyharvesterguardmodeexploit.html) |
| AR017 精英开火语音回退到普通武器语音 | 精英武器的专用攻击语音为空时，原版直接回退到通用攻击语音，导致一些单位升级后失去特色。修复优先采用对应普通武器语音。 | `VoicePrimaryWeaponAttack`、`VoicePrimaryEliteWeaponAttack`、`VoiceSecondaryWeaponAttack`、`VoiceSecondaryEliteWeaponAttack`、`VoiceAttack` | 建议由语音选择所有者按武器槽和等级建立一致的回退顺序。 | [Firing Voices and Veterancy](https://ares-developers.github.io/Ares-docs/bugfixes/type1/firingvoicesandveterancy.html) |
| AR018 AI力场护盾仅响应真正发射的超武 | 敌方超级武器因同类效果已存在而未能发射时，AI仍可能启动力场护盾并停电，玩家可反复利用假攻击耗盾。修复只响应真实发射。 | 无独立键或见原文 | 建议由超级武器发射所有者在成功提交后才通知防御AI。 | [Force Shield and AI Exploit](https://ares-developers.github.io/Ares-docs/bugfixes/type1/forceshieldexploit.html) |
| AR019 突变步兵无法落地时删除转换动画 | 突变动画结束后若生成格被占，原版每帧分配新步兵却不释放，动画越久内存泄漏越重。修复在无法生成时删除动画，终止反复尝试。 | `MakeInfantry` | 建议由突变生成所有者处理失败清理，结束动画及临时单位生命期。 | [Frozen Mutation ( MakeInfantry ) Animations](https://ares-developers.github.io/Ares-docs/bugfixes/type1/frozenmutationmakeinfantryanimations.html) |
| AR020 警戒单位不再追逐出射程飞机 | 警戒中的防空单位可能追逐敌机而离开防区，容易被诱走。修复使普通警戒任务在飞机离开射程后守住位置，区域警戒不受此项改变。 | 无独立键或见原文 | 建议由任务状态所有者区分警戒和区域警戒的追击条件。 | [Units in Guard Mission Will not Chase Aircraft](https://ares-developers.github.io/Ares-docs/bugfixes/type1/guardapproachtarget.html) |
| AR022 补全步兵部署任务状态 | 可部署步兵在解除部署延迟非负时可能陷入任务状态空洞，例如尤里复制人范围攻击后长时间悬空。修复补全转换路径，避免异常停顿。 | `Deployer`、`UndeployDelay` | 建议由步兵部署状态机所有者补全延迟阶段的合法后继状态。 | [Infantry Deploy Glitch](https://ares-developers.github.io/Ares-docs/bugfixes/type1/infantrydeploy.html) |
| AR023 伊文炸弹跨部署和易主保留 | 被安装炸弹的单位变成建筑、建筑变回单位或建筑易主时，炸弹曾无故丢失。修复在对象转换中移交炸弹；出售带弹对象会引爆它。 | 无独立键或见原文 | 建议由附着炸弹所有者处理对象替换、易主及出售时的转移结算。 | [Disappearing Ivan Bombs](https://ares-developers.github.io/Ares-docs/bugfixes/type1/ivanbombsdisappearing.html) |
| AR024 疯狂伊文默认进入友方回收厂 | 伊文面对友方回收厂曾总显示装弹光标，必须先埋炸弹才能进入。修复默认允许进入友方回收厂，敌方仍可装弹，强制攻击可覆盖默认行为。 | 无独立键或见原文 | 建议由交互动作所有者按关系选择进入或装弹，并支持强制攻击。 | [Crazy Ivans Cannot Enter Grinders](https://ares-developers.github.io/Ares-docs/bugfixes/type1/ivanentergrinder.html) |
| AR025 目标销毁时清除残留目标引用 | 单位或建筑有时不会在目标销毁后正确放弃目标，残留引用常使飞机发生随机崩溃。修复确保销毁对象不再被继续当作有效目标使用。 | 无独立键或见原文 | 建议由目标引用所有者在销毁通知中同步清理当前及历史目标。 | [Last Target not Cleared Correctly](https://ares-developers.github.io/Ares-docs/bugfixes/type1/lasttargetbug.html) |
| AR026 按阵营选择载入界面文字颜色 | 原版战役总用苏军红色，多人中非盟军也用红色。修复使盟军战役使用盟军色，尤里多人使用尤里色，并在缺省时回退为紫色。 | `SovietLoad`、`AlliedLoad`、`YuriLoad` | 建议由载入界面所有者按模式和阵营解析文字颜色及回退值。 | [Load Screen Colors](https://ares-developers.github.io/Ares-docs/bugfixes/type1/loadscreencolors.html) |
| AR027 脱离心控的建筑恢复警戒 | 建筑从心灵控制释放后曾被安排单位使用的猎杀任务，因建筑不能移动而停止工作甚至不还击。修复改派警戒任务，使建筑正常响应。 | 无独立键或见原文 | 建议由易主任务所有者按对象类别选择释放后的合法任务。 | [Mind-Controlled Buildings Stopped Working When Freed](https://ares-developers.github.io/Ares-docs/bugfixes/type1/mindcontrolbuildings.html) |
| AR028 海军生产图标正确显示初始老兵标记 | 渗透战车工厂曾给所有可训练载具的图标加老兵标记，海军船坞产物实际却不享受加成。修复只标记真正会以老兵等级出厂的载具。 | 无独立键或见原文 | 建议由生产资格所有者提供初始等级，供侧栏图标统一读取。 | [Misleading Veteran Naval Cameos](https://ares-developers.github.io/Ares-docs/bugfixes/type1/misleadingveterannavalcameos.html) |
| AR029 AI不把被动中立势力当主要敌人 | AI按受损情况选择重点敌人时，油井爆炸等中立伤害可能使其转攻平民。修复排除多人被动势力，避免中立对象干扰战略敌人选择。 | `MultiplayPassive` | 建议由AI敌人评估所有者在仇恨排名前过滤被动中立势力。 | [MultiplayPassive=yes Houses Anger Players](https://ares-developers.github.io/Ares-docs/bugfixes/type1/multiplaypassiveanger.html) |
| AR030 时空冻结保留正确的无子机替代外形 | 带无子机替代模型的母舰遭时空武器攻击时，原版总切到无子机外形，驱逐舰上停靠的飞机会突然消失。修复按实际子机状态选模型。 | `NoSpawnAlt`、`Temporal` | 建议由载机外形所有者依据真实挂载状态绘制冻结中的母舰。 | [NoSpawnAlt and Temporal Warheads](https://ares-developers.github.io/Ares-docs/bugfixes/type1/nospawnalttemporal.html) |
| AR031 驻军步兵即时晋升 | 驻军步兵虽能靠击杀积累经验，却原须离开建筑才晋升，再入驻后才会用精英武器。修复在建筑内即时、静默晋升，无须进出切换。 | `OpenTopped`、`Gunner` | 建议由经验等级所有者即时更新驻军等级并使武器选择同步。 | [Occupiers Are Promoted Only After Exiting Building](https://ares-developers.github.io/Ares-docs/bugfixes/type1/occupierpromotion.html) |
| AR032 开放式乘客攻击不阻塞防御建筑充能 | 载有高频射击步兵的开放式载具可能扰乱需要充能的防御建筑，让后者一直无法开火。修复消除这一攻击反馈造成的充能干扰。 | 无独立键或见原文 | 建议由防御建筑充能所有者避免乘客命中反馈错误重置射击状态。 | [Open-Topped Vehicles and Defensive Structures Exploit](https://ares-developers.github.io/Ares-docs/bugfixes/type1/opentoppedjambuildings.html) |
| AR033 海军寄生单位击杀友军不获经验 | 海军寄生单位如巨型乌贼抓住并杀死友方船只时，原版也会获得经验。修复使寄生击杀遵守敌我关系，不能靠攻击盟友升级。 | `Naval` | 建议由击杀经验所有者为寄生伤害统一应用敌我资格检查。 | [Parasites Gaining Experience From Killing Allies](https://ares-developers.github.io/Ares-docs/bugfixes/type1/parasitesexperience.html) |
| AR034 空中宿主死亡后寄生者从原高度坠落 | 寄生者杀死空中宿主后原会完好出现在地面。修复从宿主死亡高度释放寄生者，使其坠毁；此页明确跳跃式寄生者也仍按同样方式坠落。 | `JumpJet` | 建议由宿主销毁所有者继承释放高度，并交坠落机制处理寄生者。 | [Parasites In Airborne Units](https://ares-developers.github.io/Ares-docs/bugfixes/type1/parasitesinairborneunits.html) |
| AR035 未配置乘员标记时保留正确默认值 | 配置节存在但未写乘员或驻军标记键时，错误查找会破坏默认值，导致运输工具和建筑使用错误小图标。修复恢复缺键时的正常解析。 | `Pip`、`OccupyPip` | 建议由配置解析所有者区分缺键与显式值，保留类型默认标记。 | [Pip Distortion Bug](https://ares-developers.github.io/Ares-docs/bugfixes/type1/pipdistortionbug.html) |
| AR036 修复光棱支援倍率与连接生命周期 | 修复配置层缺键导致倍率放大百倍、失效塔仍贡献伤害和施工出售时可支援的问题。已用UMP显式修正的地图仍可能覆盖全局倍率，可用建筑支援倍率另行覆盖。 | `PrismSupportModifier`、`PrismForwarding.SupportModifier` | 建议由光棱支援所有者统一维护倍率继承、连线资格与失效清理。 | [Prism Support Bugs](https://ares-developers.github.io/Ares-docs/bugfixes/type1/prismsupportbugs.html) |
| AR037 显示范围指示圈时最小化不崩溃 | 选中建筑等操作显示范围指示圈时，若最小化游戏，原版可能发生崩溃。修复使这一界面状态能够安全跨越窗口最小化过程。 | 无独立键或见原文 | 建议由范围叠加绘制所有者在窗口资源失效时暂停并重建绘制。 | [Minimizing While Radial Indicator Visible](https://ares-developers.github.io/Ares-docs/bugfixes/type1/radialindicatorcrash.html) |
| AR038 存档恢复保留补给箱且领取后消失 | 原版读档会清空全部补给箱，掩盖存档中箱子领取后无法消失的问题，也会破坏任务关键箱子。修复领取清理并取消读档时的整体删除。 | 无独立键或见原文 | 建议由补给箱所有者恢复完整存档状态，并统一执行领取后销毁。 | [Crates Disappear When Loading A Game](https://ares-developers.github.io/Ares-docs/bugfixes/type1/savegamecrates.html) |
| AR039 秘密实验室奖励独立随机选择 | 原版奖励列表的后部项目较难被抽中，先前实验室的选择还会排除后来实验室的某些结果。修复使每座实验室独立、等机会地随机选择。 | 无独立键或见原文 | 建议由实验室奖励所有者使用完整候选集独立抽取各实例奖励。 | [Secret Lab Boon Weighting](https://ares-developers.github.io/Ares-docs/bugfixes/type1/secretlabboonweighting.html) |
| AR040 出售建筑流程封堵重复乘员和幽灵基地车 | 出售建造场时改下移动命令可白拿乘员，失败落地的基地车还可能阻止战败。修复禁止出售改移动、避免二次放人，并清除无法放置的基地车。 | 无独立键或见原文 | 建议由建筑出售所有者独占流程，统一处理乘员生成和基地车落地。 | [Selling Buildings Exploits](https://ares-developers.github.io/Ares-docs/bugfixes/type1/sellingbuildingsexploits.html) |
| AR041 船坞维修中的船只仍受常规伤害 | 原版船只只要正在船坞接受维修，就不会受到常规武器伤害，形成维修期间的非预期无敌。修复使维修不再屏蔽正常伤害结算。 | 无独立键或见原文 | 建议由伤害准入所有者排除船坞维修状态造成的错误免疫。 | [Units Being Repaired on Shipyards are Immune to Damage](https://ares-developers.github.io/Ares-docs/bugfixes/type1/shipyardrepairimmune.html) |
| AR042 新对局载入时不播旧局机器人停机声 | 上一局出现过遥控坦克后，再开始一局可能在载入画面播放它们的停机声音。修复避免旧对局清理状态触发新对局的音效。 | 无独立键或见原文 | 建议由对局音频所有者在切局清理时抑制单位状态变化音效。 | [Robot Tank Sounds On Loading Screen](https://ares-developers.github.io/Ares-docs/bugfixes/type1/soundsplayingwhenloading.html) |
| AR043 间谍不得渗透己方或盟友建筑 | 原版虽然不应允许间谍渗透己方和盟友建筑，却仍能下令进入并施加渗透效果。修复阻止这些友方目标，避免通过合作或自我渗透获利。 | 无独立键或见原文 | 建议由渗透所有者在命令及进入结算时共同校验敌对关系。 | [Spying on Allied Buildings](https://ares-developers.github.io/Ares-docs/bugfixes/type1/spyingalliedbuildings.html) |
| AR044 多人禁超武选项不影响战役 | 多人模式关闭超级武器的设置曾残留到战役，连相关建筑也可能无法生产。修复限制选项作用域，使战役仍能正常取得和使用超级武器。 | `DisableableFromShell` | 建议由模式规则所有者仅在多人对局应用大厅超级武器限制。 | [Super Weapon Option Affected Campaigns](https://ares-developers.github.io/Ares-docs/bugfixes/type1/superweaponoption.html) |
| AR045 未占领科技建筑受损不播放工作动画 | 需要工程师启用的科技建筑在受损变黄并切换图像时，会错误启动工作动画，仿佛已被占领。修复只纠正动画状态，原问题不影响功能。 | `NeedsEngineer` | 建议由建筑动画所有者在损伤换图后重新核对启用状态。 | [Tech Structures and Animations](https://ares-developers.github.io/Ares-docs/bugfixes/type1/techstructuresneedsengineer.html) |
| AR046 时空抹除清理正在前往的驻军目标 | 步兵被命令进入中立建筑后，若到达前该建筑被时空抹除，原版会发生内部错误。修复使目标消失能安全取消尚未完成的进驻意图。 | `Temporal` | 建议由进驻目标所有者订阅建筑销毁并取消在途步兵引用。 | [Temporal Warheads and Potential Occupation Targets](https://ares-developers.github.io/Ares-docs/bugfixes/type1/temporalwarheadsandpotentialoccupationtargets.html) |
| AR047 时空武器击杀友军不获经验 | 普通击杀不会因杀死己方或盟友对象而获得经验，但时空武器曾绕过这条限制。修复把同一敌我资格判断应用到时空抹除结算。 | `Temporal` | 建议由击杀经验所有者统一接收时空结算并核验敌我关系。 | [Temporal Warheads Earn Experience By Killing Friendly Units](https://ares-developers.github.io/Ares-docs/bugfixes/type1/temporalwarheadsearnexperiencebykillingfriendlyunits.html) |
| AR048 时空武器击杀只结算一次经验 | 时空击杀原先先自行奖励经验，又调用会再奖励一次的公共销毁逻辑，使攻击者获得双份经验。修复取消时空路径的重复等级更新。 | `Temporal` | 建议由击杀经验所有者成为唯一发奖入口，移除时空重复写入。 | [Temporal Warheads Gain Double Experience](https://ares-developers.github.io/Ares-docs/bugfixes/type1/temporalwarheadsgaindoubleexperience.html) |
| AR049 时空抹除期间隐藏建筑维修扳手 | 建筑被时空武器冻结并抹除时不能维修，生命值也不会因维修改变，但扳手动画仍曾播放。修复隐藏这段时间的维修指示，避免误导。 | 无独立键或见原文 | 建议由维修显示所有者按实际可维修状态决定扳手可见性。 | [Repair Wrench Playing When Warping Out](https://ares-developers.github.io/Ares-docs/bugfixes/type1/temporalwrench.html) |
| AR050 非法地图坐标不再写越界数组 | 部分路径会用无效格坐标写入数组，造成内存破坏并最终崩溃。修复对这一越界写入进行纠正；官方没有确认其是否会引发不同步。 | 无独立键或见原文 | 建议由地图数组所有者统一校验坐标并拒绝超出边界的写入。 | [Out of Bounds Array Access Could Lead to Crashes](https://ares-developers.github.io/Ares-docs/bugfixes/type1/threatposedestimatescrash.html) |
| AR051 实验室多于奖励种类时仍各有奖励 | 地图秘密实验室多于可选奖励数时，原版会让所有实验室都没有奖励。修复让各实验室独立随机选择，数量超过候选数也能正常使用。 | 无独立键或见原文 | 建议由实验室奖励所有者允许独立重复抽取，避免全局耗尽状态。 | [Too Many Secret Labs](https://ares-developers.github.io/Ares-docs/bugfixes/type1/toomanysecretlabs.html) |
| AR052 按实际停靠位置出售维修厂内单位 | 单位出售原依赖停在建筑中心，造成盟军维修厂能卖而苏军维修厂不能。修复改用每个建筑的实际停靠位置，并支持逐建筑控制出售能力。 | 无独立键或见原文 | 建议由停靠出售所有者读取实际泊位，统一校验单位与建筑状态。 | [Selling Unit on Dock Structures](https://ares-developers.github.io/Ares-docs/bugfixes/type1/unitselldock.html) |
| AR053 被磁力拖动时不播放自主移动声 | 单位受磁电坦克牵引时原会播放自己的移动声音，尽管并未靠自身动力运动。修复仅在自主移动时触发该音效，使牵引表现符合状态。 | `MoveSound` | 建议由移动音频所有者区分自主推进与外部牵引后触发声音。 | [Unit Sounds Played At Inappropriate Times](https://ares-developers.github.io/Ares-docs/bugfixes/type1/unitsoundsplayedatinappropriatetimes.html) |
| AR054 低游戏速度不再误判为性能不足 | 原版按帧率关闭激光、波束及高细节动画，低游戏速度也会被当成持续掉帧而无法恢复特效。修复使慢速游戏仍可显示高质量效果。 | `DetailLevel` | 建议由细节等级所有者结合目标游戏帧率判断性能回退阈值。 | [Visual Effects Switched Off Erratically](https://ares-developers.github.io/Ares-docs/bugfixes/type1/visualeffectsdisabled.html) |
| AR055 体素动画伤害不修改类型且允许无弹头 | 体素动画直接传出类型的伤害字段，结算可能把后续实例伤害越改越低；缺弹头还会崩溃。修复使用伤害副本，并在没有弹头时跳过伤害。 | `Damage`、`Warhead` | 建议由体素动画伤害所有者复制结算参数并检查弹头有效性。 | [Voxel Animation Damage Issues](https://ares-developers.github.io/Ares-docs/bugfixes/type1/voxelanimdamage.html) |
| AR056 装甲倍率特殊值稳定控制攻击行为 | 弹头倍率0%禁止强攻、反击及自动索敌，1%禁止后两项，2%只禁自动索敌，浮点比较却曾使规则失常。修复这些判断，并允许相关行为独立于倍率配置。 | `Verses` | 建议由弹头攻击资格所有者解析特殊倍率并统一提供行为判定。 | [Warhead Verses ‘ Special Values](https://ares-developers.github.io/Ares-docs/bugfixes/type1/warheadversesspecialvalues.html) |

## 模组逻辑修复

本节 79 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR057 飞机轨道炮击杀目标不崩溃 | 飞机装备轨道炮武器时，原版在摧毁其预定目标后可能发生内部错误。修复使飞机的轨道炮击杀路径能够安全完成目标销毁后的处理。 | `IsRailgun` | 建议由轨道炮射击所有者在目标销毁后清理飞机的相关引用。 | [AircraftTypes and Railguns](https://ares-developers.github.io/Ares-docs/bugfixes/type2/aircrafttypesandrailguns.html) |
| AR058 飞机可使用防空武器空战 | 原版飞机即使装备能攻击空中的弹道，也不会攻击其他空中单位。修复让飞机正确使用现有防空配置，不要求额外增加启用开关。 | `AA` | 建议由飞机目标与射击所有者接入通用防空弹道资格检查。 | [Air-To-Air Combat](https://ares-developers.github.io/Ares-docs/bugfixes/type2/airtoaircombat.html) |
| AR059 建筑战区换图支持任意字母前缀 | 建筑按战区替换图像文件名时，原只支持图像标识以G、N、Y、C开头，其他前缀失效。修复扩展到所有大小写英文字母，并保留缺图回退。 | `NewTheater`、`Image` | 建议由建筑资源解析所有者统一按战区变换文件名并执行回退。 | [Alternate Theater Art For BuildingTypes](https://ares-developers.github.io/Ares-docs/bugfixes/type2/alternatetheaterartforbuildingtypes.html) |
| AR060 轨道炮环境伤害为零时跳过结算 | 环境伤害设零的轨道炮仍走受击流程，部分建筑甚至强制扣一点血，无法仅作视觉效果。修复直接跳过环境伤害调用，避免触发攻击副作用。 | `AmbientDamage` | 建议由轨道炮环境伤害所有者在零值时停止伤害事件分发。 | [AmbientDamage=0 Still Dealt Damage](https://ares-developers.github.io/Ares-docs/bugfixes/type2/ambientdamage.html) |
| AR061 两栖单位传送到水面不再沉没 | 正常能够直接进入水面的非悬浮两栖单位，原版被超时空装置传送到水上却会沉没。修复让这类单位按其真实水陆通行能力生存。 | 无独立键或见原文 | 建议由传送落点所有者依据两栖通行能力决定水面落地结果。 | [Amphibious Objects Sink When Chronoshifted Onto Water](https://ares-developers.github.io/Ares-docs/bugfixes/type2/amphibiousobjectssinkwhenchronoshiftedontowater.html) |
| AR062 普通动画遵守自身伤害弹头 | 原版动画伤害多用全局火焰弹头，只有特定动画用炸药弹头，忽略动画自身配置。修复优先使用动画所设弹头，未配置才保留原默认。 | `FlameDamage2`、`C4Warhead`、`Warhead` | 建议由动画伤害所有者统一解析显式弹头与历史默认回退。 | [Animation Damage Warheads](https://ares-developers.github.io/Ares-docs/bugfixes/type2/animationdamagewarheads.html) |
| AR063 军械库步兵正常离开且不堵队 | 步兵进入军械库后可能无法正常离开，继而阻塞后续单位进入。修复完善军械库出口处理，使服务结束的步兵能出场并释放入口队列。 | `Armory` | 建议由建筑服务队列所有者统一处理服务完成、出口放置和出队。 | [Infantry Getting Stuck in Armory](https://ares-developers.github.io/Ares-docs/bugfixes/type2/armoryexit.html) |
| AR064 突击驻军建筑时正确区分攻击者和死者 | 突击步兵清理驻军时原把突击者与受害步兵颠倒，导致错误登记损失，甚至多次把关键突击单位记为阵亡而触发任务失败。修复身份混淆。 | `Assaulter` | 建议由驻军突击所有者传递正确攻击者与受害者，统一记账。 | [Assaulter Mix Up](https://ares-developers.github.io/Ares-docs/bugfixes/type2/assaultermixup.html) |
| AR065 基地车列表不再只识别前三项 | 原版只把基地车列表的前三种载具识别为基地车，额外阵营因此在非短局等规则中受限。修复遍历整个列表，使新增项目同样参与判定。 | `BaseUnit` | 建议由基地车类型所有者解析完整列表，供存活与阵营逻辑共用。 | [BaseUnit](https://ares-developers.github.io/Ares-docs/bugfixes/type2/baseunit.html) |
| AR066 取消侧栏建造的建造场不崩溃 | 把建筑工厂型建造场设为可从侧栏直接建造后，开始生产又取消曾触发内部错误。修复让这种建筑类型也能安全撤销未完成的生产。 | `Factory` | 建议由建筑生产所有者支持工厂型建筑取消时的完整资源回收。 | [Buildable Construction Yards](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildableconstructionyards.html) |
| AR067 玩家新建秘密实验室也生成奖励 | 实验室奖励原仅在地图载入时分配，玩家后来建造的实验室没有奖励。修复在建筑放置时，从该玩家当时无法建造的项目中随机选择。 | 无独立键或见原文 | 建议由实验室放置所有者按当前玩家建造能力建立奖励候选集。 | [Buildable Secret Labs](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildablesecretlabs.html) |
| AR068 低电量播报检查完整建造场列表 | 原版低电量语音只检查建造场列表的前三种建筑，额外阵营即使拥有自己的建造场也可能无提示。修复使列表中任一建筑都能满足播报条件。 | `BuildConst` | 建议由电力提示所有者读取完整建造场列表并统一判定播报。 | [BuildConst](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildconst.html) |
| AR069 建筑和步兵正常恢复弹药 | 原版建筑与步兵的弹药恢复逻辑不完整，难以实用。修复其补弹流程；飞机仍需停靠补弹，医院和军械库建筑仍不恢复弹药。 | `Ammo`、`Hospital`、`Armory` | 建议由弹药所有者按对象类别推进装填，并保留特殊建筑例外。 | [BuildingTypes and InfantryTypes Do Not Reload Ammo Properly](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildingtypesandinfantrytypesdonotreloadammoproperly.html) |
| AR070 全部建筑上的升级可满足前置条件 | 原版只检查最近建造建筑的升级，已有升级常不满足前置条件。修复让所有建筑升级都能参与普通前置判断，也可满足超级武器辅助建筑要求。 | `AuxBuilding` | 建议由前置条件所有者汇总全部有效升级，供普通和超武共用。 | [BuildingType Upgrades Are Not Viable Prerequisites](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildingtypeupgradesarenotviableprerequisites.html) |
| AR071 生产队列不重复计算建造上限 | 原版把正在生产的载具或步兵计数两次，已有同类生产时会过早达到上限，清空队列后才又能排最后一项。修复消除这次重复计数。 | `BuildLimit` | 建议由生产计数所有者区分排队与现存实例，统一进行上限判断。 | [BuildLimit and Queues](https://ares-developers.github.io/Ares-docs/bugfixes/type2/buildlimitqueues.html) |
| AR072 老兵获得的炸药能力可摧毁建筑 | 步兵通过晋升获得炸药能力后，原版虽允许下达使用命令，却不能炸毁建筑。修复让这种等级奖励真正生效，恢复其预期的建筑破坏用途。 | 无独立键或见原文 | 建议由老兵能力所有者将炸药资格接入建筑爆破动作的准入结算。 | [C4 Veteran Ability](https://ares-developers.github.io/Ares-docs/bugfixes/type2/c4veteranability.html) |
| AR073 运输机卸货恢复载具状态 | 运输机曾永久改写货物转向速度，卸下后还可能冻结或无法落地即丢失。修复恢复转向，以飞机朝向卸货，恢复警戒或小队，放置失败则收回。 | `ROT` | 建议由吊运所有者保存并恢复货物状态，原子处理卸载失败。 | [Carryall Cargo Fixes](https://ares-developers.github.io/Ares-docs/bugfixes/type2/carryallcargothings.html) |
| AR074 超时空运输车显形中禁止卸客 | 传送后尚在显形的武装运输车，键盘与指令栏禁止部署，鼠标却能卸客。修复使鼠标也遵守同一限制，避免不同输入途径给出相反结果。 | 无独立键或见原文 | 建议由运输部署所有者统一准入条件，供鼠标及快捷命令调用。 | [Chrono-Transport Deploy Inconsistency](https://ares-developers.github.io/Ares-docs/bugfixes/type2/chronotransportdeploy.html) |
| AR075 飞机和建筑支持自身隐形配置 | 飞机与建筑原先忽略可隐形配置，修复后会按该标志隐形。飞机接到攻击命令就显形，而不是等进入射程，此行为边界需与步兵区分。 | `Cloakable` | 建议由隐形状态所有者纳入飞机与建筑，并明确攻击命令的显形时点。 | [Cloakable AircraftTypes and BuildingTypes](https://ares-developers.github.io/Ares-docs/bugfixes/type2/cloakableaircrafttypesandbuildingtypes.html) |
| AR076 载机母舰隐形不再摧毁子机 | 母舰一隐形，原版把失去视野当作母舰被摧毁，导致全部子机坠毁重生。修复区分隐藏与死亡，使可隐形航母、驱逐舰正常保留子机。 | `Spawner` | 建议由子机关系所有者区分宿主隐藏与销毁通知，保持挂载引用。 | [Cloakable Spawners](https://ares-developers.github.io/Ares-docs/bugfixes/type2/cloakablespawners.html) |
| AR077 治疗或零伤害命中不再破隐 | 隐形对象受到无伤害甚至治疗弹头命中时，原版仍会短暂显形，友方支援也可能泄露位置。修复使治疗和零伤害武器不触发这种破隐。 | 无独立键或见原文 | 建议由受击隐形所有者依据有效伤害结果决定是否解除隐形。 | [Cloaked Objects no Longer Uncloak if Unaffected](https://ares-developers.github.io/Ares-docs/bugfixes/type2/cloakdamage.html) |
| AR078 隐形建筑维修扳手不泄露位置 | 原版维修扳手对所有玩家可见，隐形建筑维修时会因此暴露坐标。修复隐藏隐形建筑的维修动画，但观察者仍能看到该指示。 | 无独立键或见原文 | 建议由维修叠加绘制所有者结合隐形与观察者身份筛选可见性。 | [Repair Wrench Revealing Cloaked Structures](https://ares-developers.github.io/Ares-docs/bugfixes/type2/cloakwrench.html) |
| AR079 解除自定义调色板数量限制 | 原版多个对象使用自定义调色板会触发限制崩溃，且默认雕像已占用名额。修复解除此限制；模组仍须为使用的各战区提供对应调色板资源。 | `Palette` | 建议由调色板资源所有者按名称与战区管理实例、共享及释放。 | [Custom Palettes](https://ares-developers.github.io/Ares-docs/bugfixes/type2/custompalettes.html) |
| AR080 普通武器也可摧毁可塌陷悬崖 | 原版可破坏悬崖仍可选为目标，却仅能被粒子或声波武器击塌。修复让其他武器也能使其成为斜坡，并继续遵守全局坍塌概率设置。 | `CollapseChance` | 建议由地形伤害所有者统一处理武器命中及悬崖坍塌概率。 | [Destroyable Cliffs](https://ares-developers.github.io/Ares-docs/bugfixes/type2/destroyablecliffs.html) |
| AR081 建筑销毁动画不再引发联机失步 | 原版使用建筑销毁动画配置时，会在多人游戏中引发重连错误。修复纠正这一动画路径，使启用该配置不再造成已知的同步问题。 | `DestroyAnim` | 建议由销毁动画生命周期所有者保持各端相同的创建与清理事件。 | [DestroyAnim s Cause Reconnection Errors](https://ares-developers.github.io/Ares-docs/bugfixes/type2/destroyanimscausereconnectionerrors.html) |
| AR082 建筑销毁动画使用所属玩家颜色 | 建筑销毁动画的可换色部分原始终为红色，修复后使用建筑所属玩家颜色。若素材本身用了普通红色色阶而非换色色阶，仍需修改图像。 | `DestroyAnim` | 建议由销毁动画绘制所有者继承建筑归属，并采用对应换色调色板。 | [DestroyAnim s Don’t Remap](https://ares-developers.github.io/Ares-docs/bugfixes/type2/destroyanimsdontremap.html) |
| AR083 核弹警报与钻地音效可分别配置 | 钻地单位和核弹警报原共用全局钻地声音，改一个会影响另一个。修复允许超级武器独立指定启动声音，核弹未配置时仍回退到全局钻地声。 | `DigSound`、`Locomotor`、`SW.ActivationSound` | 建议由超级武器音频所有者解析启动声，保留核弹的默认回退。 | [DigSound](https://ares-developers.github.io/Ares-docs/bugfixes/type2/digsound.html) |
| AR084 只有乘客控炮载具保留最后乘员 | 带炮塔数量的运输车原卸客时总留下最后一人，要再次部署才能放出，波及蓄能和盖特载具。修复只对乘客控炮载具保留作为驾驶员的末人。 | `TurretCount`、`IsChargeTurret`、`IsGattling`、`Gunner` | 建议由运输乘员所有者按控炮身份决定保留人数及卸载顺序。 | [Passengers and TurretCount](https://ares-developers.github.io/Ares-docs/bugfixes/type2/ejectlastpassenger.html) |
| AR085 建筑乘员类型异常不死循环或崩溃 | 出售建筑时若候选乘员全是工程师，原版会反复抽取直到卡死；乘员类型未配置也可在出售或摧毁时崩溃。修复终止无解抽取并先验证类型。 | `Engineer`、`Technician`、`AlliedCrew` | 建议由建筑乘员生成所有者限定抽取流程并验证可空类型引用。 | [Crew with Engineer=yes could freeze the game](https://ares-developers.github.io/Ares-docs/bugfixes/type2/engineertechniciancrew.html) |
| AR086 工厂出口分流按生产类别匹配 | 主工厂出口拥塞时原仅找同型建筑分流。修复把候选扩展为工厂类别与海军属性相同的其他建筑；依赖旧行为的狗窝技巧因此失效。 | `Factory`、`Naval` | 建议由工厂出口所有者按生产类别与海陆属性统一搜索可用出口。 | [Factory Load Sharing](https://ares-developers.github.io/Ares-docs/bugfixes/type2/factoryloadsharing.html) |
| AR087 免费赠送的非矿车进入区域警戒 | 矿厂附赠单位原一律被派去采矿，即使赠送对象根本不是矿车。修复保留矿车的采矿任务，让其他免费单位以区域警戒开始活动。 | 无独立键或见原文 | 建议由免费单位生成所有者按采矿能力分派合法初始任务。 | [Free Unit Initial Mission](https://ares-developers.github.io/Ares-docs/bugfixes/type2/freeunit.html) |
| AR088 裂缝半径可超过一百二十七格 | 裂缝发生器原用过小数据类型保存半径，无法超过一百二十七格。修复扩大普通和强化半径的表示范围；覆盖格太多仍会增加运行负担。 | `GapRadiusInCells`、`SuperGapRadiusInCells` | 建议由裂缝覆盖所有者扩大半径表示，并增量维护受影响网格。 | [Gap Generator Radius can be greater than 127 Cells](https://ares-developers.github.io/Ares-docs/bugfixes/type2/gapradius.html) |
| AR089 自定义围墙也允许覆盖建造闸门 | 原版闸门覆盖围墙仅识别盟苏两种墙，尤里及新墙类型不适用。修复改按围墙属性准入；闸门与围墙的自动连接仍仅支持原有两种墙。 | `Wall` | 建议由建筑放置所有者依据围墙属性允许闸门覆盖，另管连接规则。 | [Hardcoded Wall/Gate Interactions](https://ares-developers.github.io/Ares-docs/bugfixes/type2/hardcodedwallgateinteractions.html) |
| AR090 回收被劫持载具包含劫车者退款 | 原版把被劫持载具送进回收厂时，只结算车辆退款而漏掉车内劫车者。修复将劫车者也纳入回收价值，避免其在这条结算路径中消失。 | 无独立键或见原文 | 建议由回收价值所有者统一计算载具和车内劫车者的退款。 | [Hijackers are reimbursed when a unit is grinded](https://ares-developers.github.io/Ares-docs/bugfixes/type2/hijackersarereimbursedwhenaunitisgrinded.html) |
| AR092 隐形步兵进入射程前不反复显形 | 隐形步兵接到攻击命令后，即使目标尚在射程外也会显形再隐形，反复暴露行进。修复采用载具的处理方式，仅在必要时解除隐形。 | 无独立键或见原文 | 建议由隐形攻击所有者统一步兵与载具的开火显形条件。 | [Cloakable Infantry Constantly Uncloaking](https://ares-developers.github.io/Ares-docs/bugfixes/type2/infantrycloaking.html) |
| AR093 服务建筑消失时释放内部步兵 | 军械库或旧式医院在步兵服务期间被摧毁、出售或收起时，内部步兵原会滞留场外却仍占上限及存活计数。修复在这些转换中将其弹出。 | 无独立键或见原文 | 建议由建筑服务所有者在终止生命周期前释放全部内部步兵。 | [Infantry Lost In Special Function Buildings](https://ares-developers.github.io/Ares-docs/bugfixes/type2/infantrylostinspecialfunctionbuildings.html) |
| AR094 国家初始老兵单位显示图标标记 | 通过渗透获得的初始老兵效果原能在生产图标上提示，但国家自带的初始老兵单位没有同样标记。修复让这类单位也正确显示等级提示。 | 无独立键或见原文 | 建议由初始等级所有者统一提供建造等级，供侧栏标记读取。 | [Initial Veteran Cameos](https://ares-developers.github.io/Ares-docs/bugfixes/type2/initialveterancameos.html) |
| AR095 非步兵单位也可安装伊文炸弹 | 伊文炸弹交给非步兵对象时，原版虽会开火却装不上目标。修复支持载具、飞机和建筑发射这类炸弹；建筑仍没有相应炸药交互光标。 | 无独立键或见原文 | 建议由炸弹附着所有者支持各射手类别，并保持光标规则独立。 | [Ivan Bombs Can Only Be Fired By InfantryTypes](https://ares-developers.github.io/Ares-docs/bugfixes/type2/ivanbombscanonlybefiredbyinfantrytypes.html) |
| AR096 伊文隐形后炸弹仍识别原安装者 | 伊文装弹后隐形会使炸弹忘记来源，爆炸时因此可能伤到本应免疫自身伤害的安装者。修复保留来源关系，并继续尊重允许自伤的设置。 | `DamageSelf` | 建议由附着炸弹所有者跨隐形转换保留伤害来源及自伤判定。 | [Ivan Bombs and Cloaked Ivans](https://ares-developers.github.io/Ares-docs/bugfixes/type2/ivanbombscloak.html) |
| AR097 SHP跳跃载具阴影正确投向地面 | 使用精灵图像的跳跃载具原只在着地静止或有炮塔时出现阴影，后者还会把影子画在机身下。修复使阴影按实际高度正确落在地面。 | `Turret` | 建议由跳跃载具绘制所有者统一计算地面投影位置与可见性。 | [Jumpjet Shadows](https://ares-developers.github.io/Ares-docs/bugfixes/type2/jumpjetshadows.html) |
| AR098 散装标记图像资源退出时不崩溃 | 乘员标记、边框和对话气泡等指定图像以散装文件放在游戏目录时，原版退出会崩溃。修复使这些资源脱离归档文件使用也可安全结束。 | 无独立键或见原文 | 建议由图像资源所有者统一散装与归档资源的引用及释放规则。 | [Crash with Loose SHP Files](https://ares-developers.github.io/Ares-docs/bugfixes/type2/looseshpfilecrash.html) |
| AR099 心控寄生者不绘制错误连线 | 心控寄生者原会把受控目标连到最初钻入位置，目标移动后连线明显失真。修复取消此类连线，也可另用永久心控设置显示对应动画。 | `MindControl.Permanent` | 建议由心控效果绘制所有者识别寄生来源并抑制无有效端点的连线。 | [Mind-Controller Parasites](https://ares-developers.github.io/Ares-docs/bugfixes/type2/mindcontrollerparasites.html) |
| AR100 奴役单位易主时奴隶同步易主 | 原版奴隶矿车不可捕获，因而掩盖了奴役者易主后奴隶仍属旧主的问题。修复在奴役单位被捕获或心控时，也更新所带奴隶的归属。 | 无独立键或见原文 | 建议由奴役关系所有者订阅宿主易主并统一转移全部从属单位。 | [Slaves Change Ownership If Enslaver Switches Sides](https://ares-developers.github.io/Ares-docs/bugfixes/type2/mindcontrolslaves.html) |
| AR101 幻影伪装覆盖炮塔和炮管 | 有炮塔载具使用幻影伪装时，原版只伪装车身而露出炮塔，破坏隐藏效果。修复让附属炮塔和炮管跟随主体一起进入伪装显示。 | `Turret` | 建议由载具伪装绘制所有者统一控制车身、炮塔和炮管可见性。 | [Mirage Logic with Turrets/Barrels](https://ares-developers.github.io/Ares-docs/bugfixes/type2/miragelogicwithturretsbarrels.html) |
| AR102 建筑发射导弹子机不崩溃 | 建筑原可放出普通子机，却在子机被定义为导弹时崩溃。修复导弹子机处理，使建筑使用同一生成机制发射导弹也能安全运行。 | `Spawns`、`MissileSpawn` | 建议由子机发射所有者补全建筑宿主的导弹初始化及生命周期。 | [MissileSpawn on Buildings](https://ares-developers.github.io/Ares-docs/bugfixes/type2/missilespawnbuildings.html) |
| AR103 透明光照随对象移动并支持朝向 | 对象光照原停在出生位置，修复后跟随移动，隐形、地形伪装及潜地时关闭。科技对象完整支持，含多朝向帧，动画也受支持；素材须无压缩，移动可能增加延迟。 | `AlphaImage` | 建议由光照绘制所有者跟随对象变换，统一更新朝向及隐藏状态。 | [Moving Alpha Lights](https://ares-developers.github.io/Ares-docs/bugfixes/type2/movingalphalights.html) |
| AR104 AI多工厂克隆与队伍上限可控 | AI同类多工厂原会额外复制产品，可用并行队列开关关闭且默认仍开启。创建队伍则改为遵守建造上限，并可按难度允许绕过，默认均不绕过。 | `AllowParallelAIQueues`、`BuildLimit`、`AllowBypassBuildLimit` | 建议由AI生产所有者统一处理工厂队列与队伍建造上限准入。 | [Multiple AI Factories Clone Units](https://ares-developers.github.io/Ares-docs/bugfixes/type2/multipleaifactoriescloneunits.html) |
| AR105 负伤害武器可配命中动画列表 | 伤害为负的武器若在弹头上设置命中动画列表，原版一开火就可能发生内部错误。修复使治疗等负伤害效果能安全与命中动画配置共存。 | `Damage`、`AnimList` | 建议由弹头命中效果所有者在负伤害路径安全选择并创建动画。 | [Negatively Damaging Weapons with AnimList](https://ares-developers.github.io/Ares-docs/bugfixes/type2/negativelydamagingweaponswithanimlist.html) |
| AR106 减少虚假的新增建造选项提示 | 原版可能在没有对应工厂时播报新选项，建筑所属阵营判断也会误报。修复补上工厂检查并采用新的前置系统；所属阵营仍影响AI基地规划。 | 无独立键或见原文 | 建议由建造资格所有者统一输出真实可建集合，再据变化触发提示。 | [New Construction Options](https://ares-developers.github.io/Ares-docs/bugfixes/type2/newconstructionoptions.html) |
| AR107 无子机替代外形与炮塔可共存 | 无子机替代体素原与炮塔共用存储，导致母舰不能同时拥有这两种表现。修复分开相关数据，使带炮塔母舰也能使用无子机替代模型。 | `NoSpawnAlt`、`Turret` | 建议由载具体素资源所有者分别保存主体替代模型与炮塔引用。 | [NoSpawnAlt and Turret](https://ares-developers.github.io/Ares-docs/bugfixes/type2/nospawnaltturret.html) |
| AR108 不可训练驻军不再积累经验 | 驻军建筑与战斗碉堡内的单位原无条件获得击杀经验，忽略不可训练配置。修复使驻军路径遵守该设置，避免模组单位意外参与晋升。 | `Trainable` | 建议由经验等级所有者在驻军击杀奖励前检查可训练属性。 | [Occupiers are always trained even if Trainable=no](https://ares-developers.github.io/Ares-docs/bugfixes/type2/occupiersalwaystrained.html) |
| AR109 控炮运输车乘客不重复使用时空装置 | 乘客控炮载具会借用乘客的时空装置，若又开放舱内射击，乘客继续开火就因装置缺失崩溃。修复禁止已让出装置的乘客独立射击。 | `Gunner`、`Temporal`、`OpenTopped` | 建议由时空装置所有者明确借用归属，阻止失去装置的乘客开火。 | [OpenTopped Gunner s and Temporal weapons](https://ares-developers.github.io/Ares-docs/bugfixes/type2/opentoppedgunnersandtemporals.html) |
| AR110 乘客开火按要求使开放式载具显形 | 开放式载具隐形时，乘客使用要求开火显形的武器，原版仍让车体保持隐形。修复把该开火要求传给载具，使它随乘客攻击解除隐形。 | `DecloakToFire` | 建议由乘客射击所有者向运输载具传递武器的开火显形要求。 | [OpenTopped Transports Do Not Decloak To Fire](https://ares-developers.github.io/Ares-docs/bugfixes/type2/opentoppedtransportsdonotdecloaktofire.html) |
| AR111 扩展归档可覆盖原版任务列表 | 原版任务列表的资源优先级限制了扩展归档覆盖。修复调整加载次序，使放在扩展归档中的自定义任务列表可以替换语言归档内的原文件。 | 无独立键或见原文 | 建议由归档搜索所有者明确任务列表覆盖顺序，并统一资源查找。 | [Overriding missionsmd.pkt](https://ares-developers.github.io/Ares-docs/bugfixes/type2/overridingmissionsmdpkt.html) |
| AR112 手动装载的空投乘客正确脱离飞机 | 玩家自行装上飞机的单位空投后，原版未完整更新乘员状态，仍与飞机相连，飞机销毁时可能失步或崩溃。修复在投放时解除这段关系。 | 无独立键或见原文 | 建议由空投乘员所有者原子更新归属链、舱内状态和场上实例。 | [Paradropping from Player-Loaded Aircraft](https://ares-developers.github.io/Ares-docs/bugfixes/type2/paradropcrash.html) |
| AR113 多个散装地图列表不重复导入 | 同时使用多个散装地图列表文件时，原版可能重复加入其中内容。修复消除加载器造成的重复，但配置者自己重复定义地图过滤器仍会重复。 | 无独立键或见原文 | 建议由地图列表加载所有者保证文件遍历与条目导入各执行一次。 | [PKT Duplication](https://ares-developers.github.io/Ares-docs/bugfixes/type2/pktduplication.html) |
| AR114 多人事件支持按玩家槽位解析 | 地图事件可使用四四七五至四四八二表示第一至第八玩家；空槽或已败玩家不再引起崩溃而是不触发。脚本、小队和触发器的归属仍不支持此法。 | 无独立键或见原文 | 建议由地图事件所有者集中解析玩家槽位，并对无效玩家返回不满足。 | [Player @ X Support for Multiplayer Map Events](https://ares-developers.github.io/Ares-docs/bugfixes/type2/playeratx.html) |
| AR115 随机路标音效动作不越界 | 随机路标播放声音的动作原可收集四百个路标，却只分配一百个的存储空间，较多路标会崩溃。修复消除候选收集与容量不匹配。 | 无独立键或见原文 | 建议由地图动作所有者使用容量一致的候选容器再进行随机选择。 | [Play Sound at Random Waypoint](https://ares-developers.github.io/Ares-docs/bugfixes/type2/playsoundatrandomwp.html) |
| AR116 产钱计时满额运行并可靠重启 | 产钱延迟原提前一帧到期，时空攻击等情形还可能使计时不复位而永久停产。修复完整延迟与复位流程，并让启动资金为零时也开始周期。 | `ProduceCashDelay`、`Temporal`、`MultiplayPassive`、`ProduceCashStartup` | 建议由建筑产钱所有者统一计时初始化、暂停恢复和周期重置。 | [ProduceCashDelay Problems](https://ares-developers.github.io/Ares-docs/bugfixes/type2/producecash.html) |
| AR117 辐射束和波束使用实际武器开火点 | 辐射束、声波和磁电波原总从当前主武器开火点绘制，副武器等效果因此错位。修复取实际发射武器的开火点；缺失的副武器坐标仍需配置。 | `SecondaryFireFLH` | 建议由武器效果所有者接收实际武器槽，统一解析对应开火点。 | [RadBeams And Waves Using The Wrong FLH](https://ares-developers.github.io/Ares-docs/bugfixes/type2/radbeamsandwavesusingthewrongflh.html) |
| AR118 雪地辐射颜色不再硬编码红蓝分量 | 雪地战区原对辐射颜色的红色与蓝色分量使用硬编码，导致颜色不能按正常配置体现。修复移除这一战区特例，使雪地也遵守颜色数据。 | 无独立键或见原文 | 建议由辐射绘制所有者统一读取颜色，去除战区分量替换。 | [Radiation Color and Snow Theater](https://ares-developers.github.io/Ares-docs/bugfixes/type2/radiationcolorsandsnow.html) |
| AR119 多人增援动作跳过不存在的玩家 | 按玩家槽位指定的普通增援、路标增援和超时空增援，原版会尝试给不存在的玩家创建队伍而崩溃。修复仅在目标玩家存在时生成队伍。 | 无独立键或见原文 | 建议由增援动作所有者先解析有效玩家，再提交队伍创建。 | [Reinforcements and Multiplayer Map Triggers](https://ares-developers.github.io/Ares-docs/bugfixes/type2/reinforcementsandmultiplayermaptriggers.html) |
| AR120 围墙换色使用真实所有者 | 围墙的可换色部分原总使用本机人类玩家的颜色，导致敌我墙色错误。修复以围墙真正所属玩家的颜色绘制，使归属能够正确辨认。 | `Wall` | 建议由围墙绘制所有者读取实例所有权，生成相应玩家换色。 | [Remappable Walls](https://ares-developers.github.io/Ares-docs/bugfixes/type2/remappablewalls.html) |
| AR121 宽压缩图像不再出现竖向缺线 | 原版压缩精灵绘制的辅助缓冲只够二百五十六像素宽，更大图像会越界读取，令同列像素随机透明而出现竖线。修复扩大绘制缓冲。 | 无独立键或见原文 | 建议由精灵解码绘制所有者按实际图像宽度分配并验证缓冲区。 | [Shape Compression 1 Draws Vertical Lines on Large Images](https://ares-developers.github.io/Ares-docs/bugfixes/type2/shpcompression1.html) |
| AR122 声波破墙遵守弹头围墙标志 | 声波原硬编码为总能摧毁围墙覆盖层，忽略弹头不破墙设置。修复只在弹头允许破墙时损毁墙段，使声波与配置的破坏范围一致。 | `Wall` | 建议由声波伤害所有者在处理墙段前统一检查弹头破墙权限。 | [Sonic Waves Respect Wall=no](https://ares-developers.github.io/Ares-docs/bugfixes/type2/sonicwall.html) |
| AR123 声波环境伤害使用实际发射武器 | 声波环境伤害原读取单位当前主武器的弹头和伤害，即使波由其他武器发出也一样。修复让持续伤害使用真正发射该波的武器参数。 | `Warhead`、`AmbientDamage` | 建议由声波实例所有者保留来源武器，统一提供环境伤害参数。 | [Sonic Wave Ambient Damage](https://ares-developers.github.io/Ares-docs/bugfixes/type2/sonicwaveambientdamage.html) |
| AR124 副武器也初始化特殊作战装置 | 心控、寄生和时空武器需要专用装置，原版只为主武器初始化，放副武器会崩溃。修复补上副槽处理；仍不建议给同一对象配置多把同类特殊武器。 | `MindControl`、`Parasite`、`Temporal`、`Primary`、`Secondary` | 建议由特殊武器装置所有者扫描主副槽并明确同类装置占用规则。 | [“Special” Weapons now function in secondary slots](https://ares-developers.github.io/Ares-docs/bugfixes/type2/specialweaponsnowfunctioninsecondaryslots.html) |
| AR125 侦察机数量与盟军伞兵列表分离 | 盟军伞兵数量列表长度曾意外决定侦察机架数，增加空投兵种也会增加侦察机。修复由侦察超级武器自己的数量键决定，默认一架。 | `AllyParaDropNum`、`SpyPlane.Count` | 建议由侦察超级武器所有者独立解析机数，解除伞兵列表耦合。 | [SpyPlane Count Decoupled From AllyParaDropNum](https://ares-developers.github.io/Ares-docs/bugfixes/type2/spyplanecountdecoupledfromallyparadropnum.html) |
| AR126 潜地单位不再错误朝地图外行进 | 潜地运动器有时会使单位转向左上、钻地后一直沿该方向移动，最终离开地图并被销毁。修复这一错误路径，避免正常单位意外丢失。 | 无独立键或见原文 | 建议由潜地运动状态机所有者校验目标与地图边界后推进阶段。 | [Subterranean Units Could Accidentally Leave The Map](https://ares-developers.github.io/Ares-docs/bugfixes/type2/subterraneanleavingmap.html) |
| AR127 召唤空袭不再依赖苏军伞兵兵种 | 移除全局苏军伞兵兵种配置后，鲍里斯等召唤空袭武器原会在使用时发生内部错误。修复解除这项无关依赖，让空袭能独立运行。 | `SovParaDropInf` | 建议由空袭生成所有者使用自身兵种来源，避免读取无关伞兵配置。 | [Summoned Airstrikes no longer dependent on SovParaDropInf](https://ares-developers.github.io/Ares-docs/bugfixes/type2/summonedairstrikesnolongerdependentonsovparadropinf.html) |
| AR128 不可抹除对象不受时空命中副作用 | 不可时空抹除的单位被时空弹头命中时，原仍可能释放其心控目标或毁掉子机。修复在拒绝抹除时也避免这些关联状态受到影响。 | `Temporal`、`Warpable` | 建议由时空攻击所有者先验证可抹除性，再修改心控或子机关系。 | [Temporal Warheads Still Affect Objects That Are Not Warpable](https://ares-developers.github.io/Ares-docs/bugfixes/type2/temporalwarheadsstillaffectobjectsthatarenotwarpable.html) |
| AR129 解除步兵载具飞机类型数量上限 | 原版步兵、载具和飞机类型列表存在俗称百单位限制，妨碍扩充对象类型。修复移除这些列表的固定上限，此页未说明具体容器实现。 | 无独立键或见原文 | 建议由规则类型注册所有者使用可增长存储，统一维护索引稳定性。 | [The 100-Unit Bug](https://ares-developers.github.io/Ares-docs/bugfixes/type2/the100unitbug.html) |
| AR130 过长类型标识给出明确诊断 | 创建标识超过二十四字符的类型时，修复会提示具体标识，涵盖显式列表和自动加入。未自动注册的配置节不保证弹窗，精确查找仍可能失败。 | 无独立键或见原文 | 建议由类型注册所有者集中检查标识长度，并让解析诊断指向来源。 | [Warning When a Too Long ID is Used](https://ares-developers.github.io/Ares-docs/bugfixes/type2/toolongidnames.html) |
| AR131 部署形态和隐藏乘员计入建造上限 | 修复使车辆及展开建筑都同时计入两种形态的建造上限，车内劫车者计入自身上限；机场绑定飞机也不再只受空泊位数量约束，而遵守建造上限。 | `BuildLimit`、`AirportBound` | 建议由建造计数所有者统一追踪形态对应、劫车者及机场绑定飞机。 | [Unit Instances Not Counting Towards BuildLimit](https://ares-developers.github.io/Ares-docs/bugfixes/type2/unitinstancesnotcountingtowardsbuildlimit.html) |
| AR132 被运输的两栖步兵不播放上下水声 | 两栖步兵坐在开放式载具中随运输工具跨越水陆边界，也会播放自身上下水声音。修复只在步兵本体实际进入或离开水面时播放。 | `MovementZone`、`EnterWaterSound`、`ExitWaterSound` | 建议由移动音频所有者区分乘客随行与步兵本体水陆状态转换。 | [Unit Sounds Played At Inappropriate Times](https://ares-developers.github.io/Ares-docs/bugfixes/type2/unitsoundsplayedatinappropriatetimes.html) |
| AR133 载具可为建筑提供过载充能 | 原版步兵可给磁暴线圈等建筑过载充能，但同样逻辑移植到载具时只有武器开火、建筑未被增强。修复让载具来源也能真正完成充能。 | 无独立键或见原文 | 建议由建筑过载所有者通过统一受援入口接受步兵和载具贡献。 | [Units overpowering buildings](https://ares-developers.github.io/Ares-docs/bugfixes/type2/unitsoverpoweringbuildings.html) |
| AR134 车辆空投使用正确落地位置 | 飞机开火投放乘客时原一律使用步兵子格位置，车辆会看似陷入地面。修复按对象类别定位，也覆盖吊运飞机投放车辆的相同问题。 | `Carryall` | 建议由空投放置所有者区分车辆与步兵坐标规则，统一交给落地处理。 | [Vehicle Paradrop Offset](https://ares-developers.github.io/Ares-docs/bugfixes/type2/vehicleparadropoffset.html) |
| AR135 波束按实际射程与细节规则存续 | 磁电波目标出射程后原不断开且可继续伤害，声波又被固定距离截断，激光波低细节时不可见。修复按实际射程断开或终止，激光波也不再随低细节隐藏。 | `IsMagBeam`、`AmbientDamage`、`IsSonic`、`IsLaser`、`IsBigLaser` | 建议由波束实例所有者统一维护来源武器射程、目标连接与绘制资格。 | [IsMagBeam Waves and Moving Targets](https://ares-developers.github.io/Ares-docs/bugfixes/type2/waves.html) |
| AR136 侧栏分页不受七十四项目限制 | 单个侧栏建造分页显示建筑、单位或超级武器超过七十四项时，原版会卡死或崩溃。修复移除这一静态显示上限，允许更大的可用项目集合。 | 无独立键或见原文 | 建议由侧栏项目所有者采用可增长列表，并统一维护分页与选择索引。 | [The Whiteboy Bug](https://ares-developers.github.io/Ares-docs/bugfixes/type2/whiteboybug.html) |

## 性能修复

本节 10 个主题。

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR137 按系统能力选择DirectDraw软件模拟 | 某些替代显示库靠强制软件模拟省略特效以提速。Ares在支持的旧系统默认使用该模式，但Vista及更新系统不默认启用，避免不支持时黑屏。 | 无独立键或见原文 | 建议由显示后端所有者检测能力再选择模式，避免照搬旧系统默认值。 | [DirectDraw and Software Emulation](https://ares-developers.github.io/Ares-docs/bugfixes/type3/ddrawdll.html) |
| AR138 修复高速处理器启动崩溃 | 部分速度特别快的处理器会让原版在启动阶段崩溃，此项修复处理了该兼容问题。官方本页未披露具体触发运算、阈值或修复算法。 | 无独立键或见原文 | 建议由启动平台所有者定位相关速度假设，并保持初始化边界有效。 | [Crash on Fast CPUs](https://ares-developers.github.io/Ares-docs/bugfixes/type3/fastcpucrash.html) |
| AR140 提高地图对象绘制排序效率 | 原版每帧只逐对冒泡推进对象排序，还重复计算比较值，开销高且排序质量有限。修复改用更完整的排序算法，改善速度和绘制结果。 | 无独立键或见原文 | 建议由绘制层排序所有者缓存排序键并使用稳定且高效的重排策略。 | [Inefficient Sorting](https://ares-developers.github.io/Ares-docs/bugfixes/type3/layersort.html) |
| AR141 减少表达式中的重复函数调用 | 部分求最小最大值等路径疑因宏展开把同一函数调用两次，嵌套后浪费继续放大。修复其中部分重复调用，减少无意义的运行开销。 | 无独立键或见原文 | 建议由对应计算所有者保存中间结果，避免参数表达式重复求值。 | [Repeated Function Calls](https://ares-developers.github.io/Ares-docs/bugfixes/type3/macros.html) |
| AR142 地图类型存在事件缓存解析并复用计数 | 类型存在与不存在事件原每帧扫描所有类型解析标识，再扫描对象计数。修复缓存类型，常规计分对象复用玩家计数，其他仅扫描相关对象类别。 | `Insignificant`、`DontScore` | 建议由地图事件所有者缓存类型引用，并读取权威计数或分类索引。 | [Inefficient Map Events](https://ares-developers.github.io/Ares-docs/bugfixes/type3/mapactions.html) |
| AR143 对象取所在格避免重复工作 | 对象取所在格是经常调用的公共函数，原实现却把同一组工作做了两遍。此项修复去掉重复处理，降低广泛调用该查询带来的累积开销。 | 无独立键或见原文 | 建议由对象空间查询所有者复用单次求得的格结果，服务全部调用方。 | [ObjectClass::GetCell](https://ares-developers.github.io/Ares-docs/bugfixes/type3/objectgetcell.html) |
| AR144 乘员标记图像避免连续重复加载 | 乘员标记图像原会连续从磁盘重新读取六次，增加不必要的加载开销。修复取消这种重复读取，使这一共享图像无需在同一初始化链反复装入。 | 无独立键或见原文 | 建议由图像资源所有者复用标记图像句柄，统一初始化与释放。 | [Pips Loading](https://ares-developers.github.io/Ares-docs/bugfixes/type3/pipsloading.html) |
| AR145 随机地图生成减少小地图重绘 | 随机地图生成阶段原把大量时间花在多次重绘小地图上。修复减少这类重复绘制，显著缩短生成等待，而本页未指定新的生成算法。 | 无独立键或见原文 | 建议由地图生成界面所有者合并进度绘制，在必要阶段刷新小地图。 | [Random Map Generator](https://ares-developers.github.io/Ares-docs/bugfixes/type3/randommapgenerator.html) |
| AR146 安全场景省略无效引用通知 | 部分对象失效通知在特定条件下并无必要，仍会增加运行负担。此项修复只在已知安全的位置跳过这些工作，官方未列出具体对象清单。 | 无独立键或见原文 | 建议由对象引用生命周期所有者证明无依赖后，省略相应失效遍历。 | [Skip Invalidation](https://ares-developers.github.io/Ares-docs/bugfixes/type3/skipinvalidation.html) |
| AR147 启动期间减少多余日志写入 | 原版启动过程中存在许多并不必要的调试日志写入调用，增加初始化开销。此项优化减少这些调用，本页未要求关闭所有日志或新增配置。 | 无独立键或见原文 | 建议由启动日志所有者合并无效写入，保留真正需要的诊断信息。 | [Logging Startup](https://ares-developers.github.io/Ares-docs/bugfixes/type3/startuplog.html) |

## 脚本与触发器编号速查

这些编号属于 Ares 3.0 的地图或小队脚本定义，不是 SDK 回调编号。不同家族可以使用相同数字；脚本、动作、事件必须分别解析。事件存在瞬时、持续、记忆及有序触发的区别，实际参数与组合语义见对应官方页。

### 小队脚本动作

[官方编号与参数](https://ares-developers.github.io/Ares-docs/new/scripts.html)。参数为概念提示，具体取值范围按原文核对。

| 编号 | 中文名称 | 参数概念 |
| --- | --- | --- |
| 53 | 敌方集结 | 相对AISafeDistance的距离增量；格；可为负 |
| 54 | 己方集结 | 相对AIFriendlyDistance的距离增量；格；可为负 |
| 55 | 铁幕 | 超级武器所属组号 |
| 65 | 辅助电力 | 小队所属阵营的永久电力增量；可为负；多次累加 |
| 66 | 击杀驾驶员 | 固定占位0 |
| 67 | 接管车辆 | 固定占位0 |
| 68 | 转换类型 | 固定占位0；目标来自各成员Convert.Script |
| 69 | 声呐显形 | 禁止主动隐形的帧数；0结束效果 |
| 70 | 禁用武器 | 禁止开火的帧数；0结束效果 |

### 地图触发动作

[官方编号与参数](https://ares-developers.github.io/Ares-docs/new/triggeractions.html)。参数为概念提示，具体取值范围按原文核对。

| 编号 | 中文名称 | 参数概念 |
| --- | --- | --- |
| 92 | 开启火风暴 | 无业务参数；作用于触发器所属阵营 |
| 93 | 关闭火风暴 | 无业务参数；作用于触发器所属阵营 |
| 146 | 辅助电力 | 阵营索引；永久电力增量，可为负并累加 |
| 147 | 击杀驾驶员 | 新主人阵营索引；-1代表Special |
| 148 | 设置EVA声音 | EVA索引；0盟军、1苏军、2尤里，更大值为自定义；-1关闭 |
| 149 | 设置组号 | 触发该动作对象的新组号 |

### 地图触发事件

[官方编号与参数](https://ares-developers.github.io/Ares-docs/new/triggerevents.html)。下表列编号与名称，参数、瞬时/持续及记忆规则按原文逐项核对。

| 编号 | 中文名称 |
| --- | --- |
| 62 | 受到EMP停用 |
| 63 | 被指定阵营EMP停用 |
| 64 | 被治疗EMP恢复 |
| 65 | 被指定阵营治疗EMP恢复 |
| 66 | 探照灯发现敌人 |
| 67 | 驾驶员死亡 |
| 68 | 驾驶员被指定阵营击杀 |
| 69 | 驾驶者或劫持者进入 |
| 70 | 指定阵营接管车辆 |
| 71 | 被绑架 |
| 72 | 被指定阵营绑架 |
| 73 | 绑架了对象 |
| 74 | 绑架指定阵营对象 |
| 75 | 所属阵营施放超级武器 |
| 76 | 所属阵营关闭超级武器 |
| 77 | 超级武器在路径点附近发动 |
| 78 | 已逆向该类型 |
| 79 | 逆向任意类型成功 |
| 80 | 逆向指定原始类型成功 |
| 81 | 所属阵营至少拥有 |
| 82 | 所属阵营少于门槛 |
| 83 | 被任意攻击命中或摧毁 |
| 84 | 被指定阵营命中或摧毁 |
| 85 | 指定阵营摧毁绑定对象 |
| 86 | 对象数量至多为门槛 |
| 87 | 全部保活对象被摧毁 |
| 88 | 全部保活建筑被摧毁 |

## 从 INI 到功能实现的例子

### 武器弹药成本

以下是在原版 Rules 基础上的配置层，使用原有单位和自定义武器：

```ini
[MTNK]
Ammo=5
InitialAmmo=5
Reload=120
Primary=GuideAmmoGun
ElitePrimary=GuideAmmoGun

[GuideAmmoGun]
Damage=20
ROF=30
Range=6
Speed=100
Projectile=InvisibleLow
Warhead=SA
Ammo=2
```

弹仓初始五发，两次开火后为三发、一发，随后等待装填；把武器成本设为零可以不消耗弹药，且不因开火重置装填。飞机不在这项官方功能范围内；自动切换备用武器另见 [NoAmmoWeapon](https://ares-developers.github.io/Ares-docs/new/noammoweapon.html)。Ares 原生含义见 [武器 Ammo](https://ares-developers.github.io/Ares-docs/new/weapons/ammo.html)。VERA 参考组件另需在已加载的 `[VeraFeatures]` 节开启 `AresAmmo=yes`。

已有代码路径：[`component.rs`](../../sdk/plugins/shield/src/component.rs) 在最终配置阶段分派 → [`features/ammo.rs`](../../sdk/plugins/shield/src/features/ammo.rs) 冻结成本并返回策略 → [Rules 策略绑定](../../src/rules/ruleset.rs) → 引擎原有开火、扣弹和装填。完整示例见 [ammo.ini](../../sdk/plugins/shield/examples/ammo.ini)。

### 每次恢复多发弹药

```ini
[MTNK]
Ammo=6
InitialAmmo=6
Reload=120
EmptyReload=240
ReloadAmount=2
EmptyReloadAmount=3
```

这定义有剩余弹药和空弹仓时每次恢复的数量；不是提高射击消耗。官方允许普通恢复量为负数，而空弹仓恢复量不大于零会停止恢复。这段表达 [Ares 装填数量功能](https://ares-developers.github.io/Ares-docs/new/reload.html)，当前 SDK 的 `WeaponResourcePolicy` 尚无装填数量字段。

后续实现应在现有弹药所有者完成装填时查询恢复量，继续复用最大容量、计时器和读档状态。回归至少区分非空弹仓、空弹仓和暂停后恢复；不能让插件另维护一份当前弹数。

### 限时减速附加效果

```ini
[GuideSlowWH]
Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%
AttachEffect.Duration=150
AttachEffect.SpeedMultiplier=0.75
AttachEffect.Cumulative=no
```

这是 [Ares AttachEffect](https://ares-developers.github.io/Ares-docs/new/attacheffect.html) 的效果片段，还需要由武器实际引用该弹头。当前外置 SDK 尚未提供直接修改移动能力的通用返回值。迁移这项机制时，应让共享效果实例参与移动查询，并处理重施、运输、时空暂停、删除和读档；仓库已有内置实现参考见 [附加效果记录](../ares-attach-effect.md)。

## 建议的后续开发顺序

以下顺序依据现有接口和机制依赖给出，优先级属于开发建议。

| 顺序 | 目标 | 接入前需要完成的公共能力 | 重点回归 |
| --- | --- | --- | --- |
| 1 | 保持现有武器 Ammo 示例 | 已有资源策略与弹药所有者 | 零成本、余量不足、飞机排除、读档 |
| 2 | ReloadAmount / EmptyReloadAmount | 装填完成时的数值策略 | 空与非空分支、容量、暂停、计时 |
| 3 | NoAmmoWeapon | 武器选择阶段的通用查询 | 门槛边界、部署固定武器、弹药许可 |
| 4 | 手动操作限制与建造资格 | 同步命令准入、建造资格查询 | 手动与AI区别、条件组合、失败原因 |
| 5 | EMP、禁武器、声纳、属性倍率 | 共享状态效果、能力查询、确定性计时 | 叠加、免疫、暂停、移除和效果顺序 |
| 6 | 运输、驾驶员与类型转换 | 跨对象引用、所有权、乘员与替换命令 | 当前任务、目标、等级、载荷及清理 |
| 7 | 超武、工厂、矿物和基地系统 | 受校验的领域或世界命令 | 放置、归属、触发顺序、经济及AI |
| 并行 | UI、资源格式和诊断 | 应用及资源层的独立扩展点 | 资源优先级、加载、观察者、平台行为 |

在同一功能链中先找到现有读取者、状态写入者和消费者，再决定接口。新增效果先复用共享的区域目标收集、阵营准入、伤害、计时和清理；玩法规则由插件决定，真实游戏状态由对应引擎所有者提交。

## 单项功能开发清单

1. 记录目标功能、对应官方页与目标版本，完整列出键所在文件和节、默认来源、类型与单位、分层保留及失败策略。
2. 搜索仓库中已有同名机制和原生函数引用。已有内置端口时优先迁移它；文档描述只确定需求，具体兼容实现仍需核对原始指令和实际配置。
3. 画清触发、准入、状态变更、消费者、定时、结束和删除路径。涉及拥有者、乘员、目标、弹体、动画时追踪引用释放。
4. 检查当前 SDK 是否能表达所需事实和结果。需要新能力时先定义通用输入及受校验计划，不把某个 Mod 的专用键硬编码进引擎。
5. 把功能配置和自身状态留在功能模块；把其他领域状态交给已有所有者。注册顺序、目录身份、引用和缓存重建条件要明确。
6. 状态随存档保存并进入联机校验；随机数由规定的引擎随机流按确定顺序消费。显示和音频只读取权威结果。
7. 为边界选择一到三个有区分力的生产路径回归，再运行实际组件与原版资源；结果不得只来自另一份复制的算法。
8. 更新本手册对应条目和 SDK 状态说明，记录仍未覆盖的路径。接口、配置或状态版本发生变化时明确旧存档和旧组件如何继续使用。

## 仓库已有参考位置

下列文件说明现有内置端口及其限制，便于寻找所有者。它们不会因为出现在此表就自动变成外置 SDK 接口。

| 机制 | 现有记录 |
| --- | --- |
| Ares 内置端口总览 | [ares.md](../ares.md) |
| 电磁脉冲 | [ares-emp.md](../ares-emp.md) |
| 自定义装甲 | [ares-custom-armor.md](../ares-custom-armor.md) |
| 附加效果 | [ares-attach-effect.md](../ares-attach-effect.md) |
| 建造前置条件 | [ares-prerequisites.md](../ares-prerequisites.md) |
| 玩家保活与胜负 | [ares-house-survival.md](../ares-house-survival.md) |
| 阵营默认值 | [ares-side-defaults.md](../ares-side-defaults.md) |
| 国家呈现与伞兵 | [ares-country-presentation.md](../ares-country-presentation.md)、[ares-country-paradrop.md](../ares-country-paradrop.md) |
| 超武建筑条件 | [ares-superweapon-building-gates.md](../ares-superweapon-building-gates.md) |
| 超武金钱与投送 | [ares-superweapon-money.md](../ares-superweapon-money.md)、[ares-unit-delivery.md](../ares-unit-delivery.md) |
| 通用弹头超武 | [ares-generic-warhead.md](../ares-generic-warhead.md) |
| EVA、建筑调色板及PCX图标 | [ares-eva.md](../ares-eva.md)、[ares-building-palettes.md](../ares-building-palettes.md)、[ares-pcx-cameos.md](../ares-pcx-cameos.md) |
| INI包含与继承 | [ini-directives.md](../ini-directives.md) |
| 外置SDK架构 | [ARCHITECTURE.zh-CN.md](../../sdk/ARCHITECTURE.zh-CN.md) |

## 官方目录与参考入口

| 编号及功能 | 中文说明 | 代表性 INI 入口 | 后续开发建议 | 官方原文 |
| --- | --- | --- | --- | --- |
| AR001 缺陷修复总索引 | 本页把修复分为原版缺陷、模组中暴露的逻辑缺陷和性能兼容问题，提供各专题入口；泰伯利亚之日恢复特性另列，不归入这些修复。 | 参考入口 | 建议以索引关联机制所有者，逐项登记修复范围及行为边界。 | [Bug Fixes](https://ares-developers.github.io/Ares-docs/bugfixes/index.html) |
| AR021 原版缺陷修复索引 | 本页列出未经修改的尤里的复仇1.001版中已有的程序缺陷，作为逐项查阅入口；这些问题不能仅靠修改配置或图像资源解决。 | 参考入口 | 建议把本索引映射到对应机制所有者，标明自动修复的边界。 | [Fixes to Original Bugs](https://ares-developers.github.io/Ares-docs/bugfixes/type1/index.html) |
| AR091 模组逻辑修复索引 | 本页汇集原版少用或未充分使用、在模组中容易暴露问题的机制修复，提供各专题入口；它说明缺陷来源类别，不代表每项都需新配置键。 | 参考入口 | 建议按既有机制所有者组织本索引，区分自动纠错与配置扩展。 | [Fixes to Mod Logics](https://ares-developers.github.io/Ares-docs/bugfixes/type2/index.html) |
| AR139 性能与平台兼容修复索引 | 本页汇总耗时函数优化、载入提速和现代计算机兼容修复，提供对应专题入口。其范围是减少无效工作或兼容性故障，不以改变玩法为目标。 | 参考入口 | 建议按性能热点与平台所有者组织索引，分开记录耗时和行为验证。 | [Performance Improvements](https://ares-developers.github.io/Ares-docs/bugfixes/type3/index.html) |
| AR148 与其他YR扩展的比较索引 | 官方把其他补丁的能力比较放在独立Wiki；仅作为查阅入口，不把其他项目功能自动算作Ares功能。 | 参考入口 | 比较时分别核对目标版本，不能据此推定VERA已经兼容。 | [Comparison with other YR Hacks and Patches](https://ares-developers.github.io/Ares-docs/comparison.html) |
| AR149 Ares项目与贡献者 | 记录Ares开发者、贡献者和文档协作者；不据整体名单猜测每个功能的具体作者。 | 参考入口 | 保留项目及文件原有署名，功能实现另记录可核实来源。 | [Credits](https://ares-developers.github.io/Ares-docs/credits.html) |
| AR150 官方功能及修复总索引 | 按英文主题交叉索引功能和修复条目，同一功能可能在多个主题下出现；不能按链接数计算独立功能数。 | 参考入口 | 用于查漏和查找原文，不作为运行时数据。 | [Index](https://ares-developers.github.io/Ares-docs/genindex.html) |
| AR151 INI类型 单位与术语 | 说明布尔、整数、浮点、比例、帧、格、lepton及资源引用等语义；同样写为数字的配置不一定采用相同单位。 | 参考入口 | 对照真实读取器选类型，不能统一用标准字符串转换替代原规则。 | [Glossary](https://ares-developers.github.io/Ares-docs/glossary.html) |
| AR152 历史配置迁移 | 记录标签改名、默认值和后层读取等历史变化；旧标签在这里出现，不代表Ares3.0仍接受该写法。 | 参考入口 | 先确定兼容目标版本，再落实缺失、无效、覆盖和引用解析规则。 | [Migration Guide](https://ares-developers.github.io/Ares-docs/migration.html) |
| AR174 建筑扩展功能目录 | 汇集建筑生产、AI布局、维修、驻军、间谍、资源收入及光棱网络等扩展页面，作为查找各项建筑能力的入口。目录同时列出页面内部主题，但没有独立参数定义；具体对象限制和默认值应以对应功能页正文为准。 | 参考入口 | 建议把目录用于能力分类检索，具体实现由建筑相关引擎所有者与SDK扩展承接。 | [Buildings](https://ares-developers.github.io/Ares-docs/new/buildings/index.html) |
| AR201 隐形与探测功能目录 | 汇总主动隐形、隐形发生器及探测阵列的扩展入口，包括高度、声音、过渡阶段、状态条件和侦测警报。该页用于导航与功能定位，具体配置须进入相应子页，不应视为另一个独立的隐形机制。 | 参考入口 | 建议将目录映射为隐形与侦察能力索引，具体状态仍归各机制所有者。 | [Cloak, Stealth and Sensor Arrays](https://ares-developers.github.io/Ares-docs/new/cloak/index.html) |
| AR235 新增与增强逻辑目录 | 本章集中列出Ares新增或增强的游戏机制，覆盖单位、建筑、隐形、武器、弹头、超级武器、资源以及地图脚本等入口。目录帮助按功能定位具体文档，列出的子树不代表本页另有独立开关，参数应以对应功能页为准。 | 参考入口 | 建议将目录作为SDK能力索引，各功能仍由对应领域所有者实现和维护。 | [New & Enhanced In-Game Logic](https://ares-developers.github.io/Ares-docs/new/index.html) |
| AR246 资源与配置加载目录 | 按包含、字符串表、资源覆盖、音频和旧限制查阅加载扩展。 | 参考入口 | 按实际职责定位应用、资源或模拟模块。 | [Miscellaneous Modding Enhancements](https://ares-developers.github.io/Ares-docs/new/misc/index.html) |
| AR266 抛射体目录 | 按动画、弹道、分裂、空爆和粒子拖尾查阅抛射体扩展。 | 参考入口 | 按实际职责定位应用、资源或模拟模块。 | [Projectiles](https://ares-developers.github.io/Ares-docs/new/projectiles/index.html) |
| AR313 超武类型专属设置目录 | 按超武实际类型汇集通用参数默认值和专属字段，涵盖原版战略武器、侦察与反隐、空降增援、火墙、猎杀者、电磁炮和电池等类型。此页仅作导航，不定义独立参数；同名通用字段也可能因具体类型而具有不同默认值或解释。 | 参考入口 | 建议用此目录进行超武类型分流，SDK的能力声明和引擎实现边界需逐类型核对。 | [Settings by Super Weapon Type](https://ares-developers.github.io/Ares-docs/new/superweapons/types/index.html) |
| AR329 矿物与储存功能目录 | 汇集资源储存、筒仓提示、运输或建筑毁坏溢矿、矿物治疗、行走伤害、载矿爆炸及地面连锁反应等页面。此页用于导航恢复自早期游戏的机制及其扩展，不另定义参数；启用条件、单位免疫和资源种类差异需查对应正文。 | 参考入口 | 建议把目录映射到资源、经济、伤害与地图能力，具体机制由对应引擎所有者提供扩展入口。 | [Tiberium](https://ares-developers.github.io/Ares-docs/new/tiberium/index.html) |
| AR347 弹头目录 | 按伤害准入、状态效果、击杀与呈现查阅弹头扩展。 | 参考入口 | 按实际职责定位应用、资源或模拟模块。 | [Warheads](https://ares-developers.github.io/Ares-docs/new/warheads/index.html) |
| AR364 武器目录 | 按武器目录、弹药成本、特殊伤害和光束效果查阅武器扩展。 | 参考入口 | 按实际职责定位应用、资源或模拟模块。 | [Weapons](https://ares-developers.github.io/Ares-docs/new/weapons/index.html) |
| AR372 原DLL环境兼容与必要配置调整 | 说明旧DLL注入方案的环境限制及Mod需补充的注册表、美术和配套资源；系统安全软件建议属于历史环境记录。 | 参考入口 | 复核相关功能依赖，但VERA无需照搬Syringe注入和旧系统部署步骤。 | [Compatibility Notes](https://ares-developers.github.io/Ares-docs/notes.html) |
| AR377 恢复的泰伯利亚之日机制目录 | 汇总从泰伯利亚之日恢复到尤里的复仇的机制，包括EMP、火风暴、激光围栏、多工程师、探照灯、车辆劫持及两栖替图等入口。恢复程度和配置方式可能与原游戏显著不同，应按各专题说明接入，不能直接沿用旧作参数假设。 | 参考入口 | 建议将目录作为兼容能力索引，具体机制归相应模拟或表现所有者维护。 | [Restored Tiberian Sun Logic](https://ares-developers.github.io/Ares-docs/restored/index.html) |
| AR383 官方搜索入口 | 搜索官方文档中的英文功能名和INI键；页面本身不提供新的游戏能力。 | 参考入口 | 保留为参考入口，开发文档可使用全文搜索。 | [Search](https://ares-developers.github.io/Ares-docs/search.html) |
| AR392 界面与诊断目录 | 按战役、菜单、颜色、加载、调试、存档与生成器查阅界面扩展。 | 参考入口 | 按实际职责定位应用、资源或模拟模块。 | [User Interface Features](https://ares-developers.github.io/Ares-docs/ui-features/index.html) |
| AR405 各版本新增内容 | 按版本列出部分重要功能与修复，官方声明该列表并不完整；完整范围仍需结合各功能页和总目录。 | 参考入口 | 用于确认引入版本和迁移背景，不代替逐功能行为核对。 | [What’s New](https://ares-developers.github.io/Ares-docs/whatsnew.html) |


## 来源与署名

行为来源为 [Ares 官方文档](https://ares-developers.github.io/Ares-docs/) 及每行所链接的原页面；本手册用中文重新概述功能，INI键与枚举名保持原样。Ares 项目归属和贡献者见 [官方 Credits](https://ares-developers.github.io/Ares-docs/credits.html)，VERA20K 的扩展署名说明见 [扩展支持与原作者](credits.md)。

“后续开发建议”、分层方案及开发顺序是面向 VERA20K 的设计内容。所列功能尚需逐项实现、集成和验证；没有以本手册替代官方完整标签手册、原始实现或实际兼容测试。
